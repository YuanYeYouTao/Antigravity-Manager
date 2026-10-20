// Gemini Stream Collector
// Used for auto-converting streaming responses to JSON for non-streaming requests

use bytes::Bytes;
use futures::StreamExt;
use serde_json::{json, Value};
use tracing::debug;

use crate::proxy::SignatureCache;

/// Collects a Gemini SSE stream into a complete Gemini Response Value.
pub async fn collect_stream_to_json<S, E>(stream: S, session_id: &str) -> Result<Value, String>
where
    S: futures::Stream<Item = Result<Bytes, E>> + Unpin,
    E: std::fmt::Display,
{
    collect_stream_to_json_with_anchor_and_count(stream, session_id, None, 1).await
}

pub async fn collect_stream_to_json_with_anchor<S, E>(
    stream: S,
    session_id: &str,
    anchor: Option<&str>,
) -> Result<Value, String>
where
    S: futures::Stream<Item = Result<Bytes, E>> + Unpin,
    E: std::fmt::Display,
{
    collect_stream_to_json_with_anchor_and_count(stream, session_id, anchor, 1).await
}

pub async fn collect_stream_to_json_with_anchor_and_count<S, E>(
    mut stream: S,
    session_id: &str,
    anchor: Option<&str>,
    message_count: usize,
) -> Result<Value, String>
where
    S: futures::Stream<Item = Result<Bytes, E>> + Unpin,
    E: std::fmt::Display,
{
    let mut collected_response = json!({"candidates": [{"content": {"parts": []}}]});
    let mut content_parts: Vec<Value> = Vec::new();
    let mut line_buffer = bytes::BytesMut::new();

    // Complete lines and the final unterminated line share the same collection path.
    let mut collect_line = |raw: &[u8]| -> Option<Value> {
        let line_str = String::from_utf8_lossy(raw);
        let line = line_str.trim();
        let json_part = line.strip_prefix("data: ")?.trim();
        if json_part == "[DONE]" {
            return None;
        }
        let mut data = serde_json::from_str::<Value>(json_part).ok()?;
        let actual_data = if let Some(inner) = data.get_mut("response") {
            inner.take()
        } else {
            data
        };
        if actual_data.get("error").is_some() {
            return Some(actual_data);
        }
        if let Some(usage) = actual_data.get("usageMetadata") {
            crate::proxy::monitor::record_upstream_usage(usage);
        }

        if let Some(fields) = actual_data.as_object() {
            for (key, value) in fields {
                if key != "candidates" {
                    if key == "usageMetadata" {
                        if let (Some(current), Some(reported)) = (
                            collected_response
                                .get_mut(key)
                                .and_then(Value::as_object_mut),
                            value.as_object(),
                        ) {
                            current.extend(reported.clone());
                            continue;
                        }
                    }
                    collected_response[key] = value.clone();
                }
            }
        }

        if let Some(candidate) = actual_data
            .get("candidates")
            .and_then(Value::as_array)
            .and_then(|candidates| candidates.first())
        {
            // Candidate metadata is a field snapshot. Replace each supplied value
            // whole, including groundingMetadata and its chunk/support indices.
            if let Some(fields) = candidate.as_object() {
                for (key, value) in fields {
                    if key != "content" {
                        collected_response["candidates"][0][key] = value.clone();
                    }
                }
            }
            if let Some(content) = candidate.get("content").and_then(Value::as_object) {
                for (key, value) in content {
                    if key != "parts" {
                        collected_response["candidates"][0]["content"][key] = value.clone();
                    }
                }
                if let Some(parts) = content.get("parts").and_then(Value::as_array) {
                    for part in parts {
                        // Coalesce only matching plain text/thought chunks. Signed
                        // or extended parts retain their original fields and order.
                        let is_plain = |part: &Value| {
                            part.as_object().is_some_and(|obj| {
                                obj.keys().all(|key| key == "text" || key == "thought")
                            })
                        };
                        if is_plain(part) {
                            if let Some(text) = part.get("text").and_then(Value::as_str) {
                                if let Some(last) = content_parts.last_mut() {
                                    if is_plain(last) && last.get("thought") == part.get("thought")
                                    {
                                        if let Some(Value::String(last_text)) = last.get_mut("text")
                                        {
                                            last_text.push_str(text);
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                        content_parts.push(part.clone());
                    }
                }
            }
        }
        None
    };

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| {
            crate::proxy::mappers::error_classifier::report_stream_error(
                "gemini-collector",
                "collect_stream_to_json_with_anchor",
                &e,
                format!("session={}", session_id),
            )
            .client_message()
        })?;
        line_buffer.extend_from_slice(&chunk);
        while let Some(pos) = line_buffer.iter().position(|&b| b == b'\n') {
            let line = line_buffer.split_to(pos + 1);
            if let Some(error) = collect_line(&line) {
                return Ok(error);
            }
        }
    }
    if !line_buffer.is_empty() {
        if let Some(error) = collect_line(&line_buffer) {
            return Ok(error);
        }
    }

    if content_parts.is_empty()
        && collected_response["candidates"][0]
            .get("finishReason")
            .or_else(|| collected_response["candidates"][0].get("finish_reason"))
            .and_then(Value::as_str)
            .is_none()
    {
        return Err(
            crate::proxy::mappers::error_classifier::report_stream_error(
                "gemini-collector",
                "collect_stream_to_json_with_anchor",
                &"stream terminated prematurely without content or finish reason",
                format!("session={}", session_id),
            )
            .client_message(),
        );
    }

    // Failed streams return before updating either signature or thinking state.
    if !content_parts.is_empty() {
        for part in &content_parts {
            if let Some(sig) = part.get("thoughtSignature").and_then(Value::as_str) {
                SignatureCache::global().cache_session_signature(
                    session_id,
                    sig.to_string(),
                    message_count.max(1),
                );
                debug!(
                    "[Gemini-AutoConverter] Cached signature (len: {}) for session: {}",
                    sig.len(),
                    session_id
                );
            }
        }
        crate::proxy::thinking_store::capture_gemini_parts_with_anchor(
            session_id,
            &content_parts,
            anchor.unwrap_or("root"),
        );
    }
    collected_response["candidates"][0]["content"]["parts"] = json!(content_parts);
    Ok(collected_response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;
    use std::io;

    async fn replay(frames: &[Value]) -> Value {
        let chunks: Vec<Result<Bytes, String>> = frames
            .iter()
            .map(|frame| Ok(Bytes::from(format!("data: {}\n\n", frame))))
            .collect();
        collect_stream_to_json_with_anchor(
            futures::stream::iter(chunks),
            "synthetic-grounding-replay",
            Some("synthetic-grounding-anchor"),
        )
        .await
        .expect("synthetic SSE replay should complete")
    }

    #[tokio::test]
    async fn grounding_in_separate_metadata_frame_preserves_original_shape() {
        let metadata = json!({
            "webSearchQueries": ["synthetic public query"],
            "groundingChunks": [{"web": {"uri": "https://example.invalid/source", "title": "Source"}}],
            "groundingSupports": [{"segment": {"startIndex": 0, "endIndex": 6, "text": "Result"}, "groundingChunkIndices": [0]}],
            "searchEntryPoint": {"renderedContent": "synthetic entry point"},
            "futureMetadata": {"opaque": [1, {"flag": true}]}
        });
        let usage = json!({"promptTokenCount": 78, "candidatesTokenCount": 653});
        let result = replay(&[
            json!({"response": {"candidates": [{"content": {"parts": [{"text": "Res"}]}}]}}),
            json!({"candidates": [{"content": {"parts": [{"text": "ult"}]}}]}),
            json!({"response": {"candidates": [{"groundingMetadata": metadata.clone()}]}}),
            json!({"candidates": [{"finishReason": "STOP"}], "usageMetadata": usage.clone()}),
        ])
        .await;
        assert_eq!(result["candidates"][0]["groundingMetadata"], metadata);
        assert_eq!(
            result["candidates"][0]["content"]["parts"],
            json!([{"text": "Result"}])
        );
        assert_eq!(result["candidates"][0]["finishReason"], "STOP");
        assert_eq!(result["usageMetadata"], usage);
    }

    #[tokio::test]
    async fn grounding_snapshots_replace_atomically_without_reindexing_supports() {
        let first = json!({
            "webSearchQueries": ["first"],
            "groundingChunks": [{"web": {"uri": "https://example.invalid/old"}}],
            "groundingSupports": [{"groundingChunkIndices": [0]}],
            "oldOnly": true
        });
        let latest = json!({
            "webSearchQueries": ["final"],
            "groundingChunks": [
                {"web": {"uri": "https://example.invalid/new-first"}},
                {"web": {"uri": "https://example.invalid/new-second"}}
            ],
            "groundingSupports": [{"groundingChunkIndices": [1]}],
            "unknownSnapshotField": {"value": "preserved"}
        });
        let result = replay(&[
            json!({"candidates": [{"groundingMetadata": first}]}),
            json!({"candidates": [{"content": {"parts": [{"text": "Result"}]}}]}),
            json!({"response": {"candidates": [{"groundingMetadata": latest.clone()}]}}),
            json!({"candidates": [{"finishReason": "STOP"}]}),
        ])
        .await;
        assert_eq!(result["candidates"][0]["groundingMetadata"], latest);
        assert_eq!(
            result["candidates"][0]["groundingMetadata"]["groundingChunks"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            result["candidates"][0]["groundingMetadata"]["groundingSupports"][0]
                ["groundingChunkIndices"],
            json!([1])
        );
        assert!(result["candidates"][0]["groundingMetadata"]
            .get("oldOnly")
            .is_none());
    }

    #[tokio::test]
    async fn missing_grounding_is_not_invented_and_other_response_fields_survive() {
        let parts = json!([
            {"text": "synthetic thought", "thought": true},
            {"functionCall": {"name": "synthetic_read", "args": {"id": 1}}, "thoughtSignature": "synthetic-signature"}
        ]);
        let result = replay(&[
            json!({"candidates": [{"content": {"parts": parts.clone()}}]}),
            json!({"candidates": [{"finishReason": "MALFORMED_FUNCTION_CALL"}], "usageMetadata": {"totalTokenCount": 5}}),
        ])
        .await;
        assert!(result["candidates"][0].get("groundingMetadata").is_none());
        assert_eq!(result["candidates"][0]["content"]["parts"], parts);
        assert_eq!(
            result["candidates"][0]["finishReason"],
            "MALFORMED_FUNCTION_CALL"
        );
        assert_eq!(result["usageMetadata"], json!({"totalTokenCount": 5}));
    }

    #[tokio::test]
    async fn explicit_empty_grounding_snapshot_replaces_earlier_metadata() {
        let result = replay(&[
            json!({"candidates": [{"groundingMetadata": {"groundingChunks": [{"web": {"uri": "https://example.invalid/old"}}]}}]}),
            json!({"candidates": [{"groundingMetadata": {}, "finishReason": "STOP"}]}),
        ])
        .await;
        assert_eq!(result["candidates"][0]["groundingMetadata"], json!({}));
    }

    #[tokio::test]
    async fn test_collect_simple_text_response() {
        let sse_data = vec![
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Hello\"}],\"role\":\"model\"},\"index\":0}]}\n\n",
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\" world!\"}],\"role\":\"model\"},\"finishReason\":\"STOP\",\"index\":0}],\"usageMetadata\":{\"totalTokenCount\":10}}\n\n",
            "data: [DONE]\n\n",
        ];

        let byte_stream = stream::iter(
            sse_data
                .into_iter()
                .map(|s| Ok::<Bytes, io::Error>(Bytes::from(s))),
        );

        let result = collect_stream_to_json(byte_stream, "test_session").await;
        assert!(result.is_ok());

        let resp = result.unwrap();
        let parts = resp["candidates"][0]["content"]["parts"]
            .as_array()
            .unwrap();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0]["text"], "Hello world!");
        assert_eq!(resp["candidates"][0]["finishReason"], "STOP");
        assert_eq!(resp["usageMetadata"]["totalTokenCount"], 10);
    }

    #[tokio::test]
    async fn test_collect_error_payload_propagation_504() {
        let sse_data = vec![
            "data: {\"error\":{\"code\":504,\"message\":\"stream idle timeout\",\"status\":\"DEADLINE_EXCEEDED\"}}\n\n",
            "data: [DONE]\n\n",
        ];

        let byte_stream = stream::iter(
            sse_data
                .into_iter()
                .map(|s| Ok::<Bytes, io::Error>(Bytes::from(s))),
        );

        let result = collect_stream_to_json(byte_stream, "test_session_504").await;
        assert!(result.is_ok());

        let resp = result.unwrap();
        assert!(resp.get("error").is_some());
        assert_eq!(resp["error"]["code"], 504);
        assert_eq!(resp["error"]["status"], "DEADLINE_EXCEEDED");
    }

    #[tokio::test]
    async fn test_collect_error_payload_with_v1internal_wrapper() {
        let sse_data = vec![
            "data: {\"response\":{\"error\":{\"code\":503,\"message\":\"model overloaded\",\"status\":\"UNAVAILABLE\"}}}\n\n",
            "data: [DONE]\n\n",
        ];

        let byte_stream = stream::iter(
            sse_data
                .into_iter()
                .map(|s| Ok::<Bytes, io::Error>(Bytes::from(s))),
        );

        let result = collect_stream_to_json(byte_stream, "test_session_503").await;
        assert!(result.is_ok());

        let resp = result.unwrap();
        assert!(resp.get("error").is_some());
        assert_eq!(resp["error"]["code"], 503);
        assert_eq!(resp["error"]["status"], "UNAVAILABLE");
    }

    #[tokio::test]
    async fn test_collect_premature_empty_stream_fails() {
        let sse_data = vec!["data: [DONE]\n\n"];

        let byte_stream = stream::iter(
            sse_data
                .into_iter()
                .map(|s| Ok::<Bytes, io::Error>(Bytes::from(s))),
        );

        let result = collect_stream_to_json(byte_stream, "test_session_empty").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("stream terminated prematurely"));
    }

    #[tokio::test]
    async fn test_collect_multibyte_chunk_split() {
        let cyrillic_word = "Привет, мир! 🚀";
        let chunk_data = format!(
            "data: {{\"candidates\":[{{\"content\":{{\"parts\":[{{\"text\":\"{}\"}}],\"role\":\"model\"}},\"finishReason\":\"STOP\",\"index\":0}}]}}\n\ndata: [DONE]\n\n",
            cyrillic_word
        );

        let bytes = chunk_data.into_bytes();
        let chunk_size = 13;
        let mut chunks = Vec::new();
        for chunk in bytes.chunks(chunk_size) {
            chunks.push(Ok::<Bytes, io::Error>(Bytes::copy_from_slice(chunk)));
        }

        let byte_stream = stream::iter(chunks);
        let result = collect_stream_to_json(byte_stream, "test_session_multibyte").await;
        assert!(result.is_ok());

        let resp = result.unwrap();
        let parts = resp["candidates"][0]["content"]["parts"]
            .as_array()
            .unwrap();
        assert_eq!(parts[0]["text"], cyrillic_word);
    }

    #[tokio::test]
    async fn unknown_response_candidate_and_content_fields_survive_collection() {
        let result = replay(&[
            json!({
                "modelVersion": "synthetic-version",
                "futureResponse": {"opaque": [1, 2]},
                "candidates": [{
                    "index": 4,
                    "safetyRatings": [{"category": "synthetic", "probability": "LOW"}],
                    "futureCandidate": {"opaque": true},
                    "content": {
                        "role": "model",
                        "futureContent": {"flag": true},
                        "parts": [{"text": "Result"}]
                    }
                }]
            }),
            json!({"candidates": [{"finishReason": "STOP", "finishMessage": "real upstream finish"}]}),
        ]).await;
        assert_eq!(result["modelVersion"], "synthetic-version");
        assert_eq!(result["futureResponse"], json!({"opaque": [1, 2]}));
        assert_eq!(result["candidates"][0]["index"], 4);
        assert_eq!(
            result["candidates"][0]["futureCandidate"],
            json!({"opaque": true})
        );
        assert_eq!(
            result["candidates"][0]["safetyRatings"],
            json!([{"category": "synthetic", "probability": "LOW"}])
        );
        assert_eq!(result["candidates"][0]["content"]["role"], "model");
        assert_eq!(
            result["candidates"][0]["content"]["futureContent"],
            json!({"flag": true})
        );
        assert_eq!(
            result["candidates"][0]["finishMessage"],
            "real upstream finish"
        );
    }

    #[tokio::test]
    async fn text_coalescing_preserves_signed_and_extended_parts_without_inventing_finish() {
        let parts = json!([
            {"text": "signed", "thoughtSignature": "synthetic-signature"},
            {"text": "extended", "futurePart": {"opaque": true}},
            {"text": " plain"}
        ]);
        let result =
            replay(&[json!({"candidates": [{"content": {"parts": parts.clone()}}]})]).await;
        assert_eq!(result["candidates"][0]["content"]["parts"], parts);
        assert!(result["candidates"][0].get("finishReason").is_none());
    }

    #[tokio::test]
    async fn final_unterminated_line_preserves_metadata_and_error_fields() {
        for frame in [
            json!({"response": {
                "candidates": [{
                    "content": {"parts": [{"text": "尾帧"}]},
                    "finishReason": "STOP",
                    "groundingMetadata": {"futureMetadata": {"opaque": true}}
                }]
            }}),
            json!({"response": {
                "error": {"code": 503, "message": "upstream error", "status": "UNAVAILABLE"},
                "futureErrorField": {"opaque": true}
            }}),
        ] {
            let source = stream::iter(vec![Ok::<Bytes, io::Error>(Bytes::from(format!(
                "data: {}",
                frame
            )))]);
            let result = collect_stream_to_json(source, "unterminated-frame")
                .await
                .unwrap();
            if frame["response"].get("error").is_some() {
                assert_eq!(result, frame["response"]);
                assert!(result.get("candidates").is_none());
            } else {
                assert_eq!(result["candidates"][0], frame["response"]["candidates"][0]);
            }
        }
    }

    #[tokio::test]
    async fn test_collect_unsigned_thinking_chunks_merged() {
        let chunk1 = "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Thinking part 1. \",\"thought\":true}],\"role\":\"model\"},\"index\":0}]}\n\n";
        let chunk2 = "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Thinking part 2.\",\"thought\":true}],\"role\":\"model\"},\"index\":0}]}\n\n";
        let chunk3 = "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Final answer.\"}],\"role\":\"model\"},\"finishReason\":\"STOP\",\"index\":0}]}\n\n";

        let chunks = vec![
            Ok::<Bytes, io::Error>(Bytes::from(chunk1)),
            Ok::<Bytes, io::Error>(Bytes::from(chunk2)),
            Ok::<Bytes, io::Error>(Bytes::from(chunk3)),
        ];

        let byte_stream = stream::iter(chunks);
        let result = collect_stream_to_json(byte_stream, "test_session_thinking_merge").await;
        assert!(result.is_ok());

        let resp = result.unwrap();
        let parts = resp["candidates"][0]["content"]["parts"]
            .as_array()
            .unwrap();

        assert_eq!(
            parts.len(),
            2,
            "Expected 2 parts: 1 merged thinking part and 1 answer part, got: {:?}",
            parts
        );
        assert_eq!(parts[0]["text"], "Thinking part 1. Thinking part 2.");
        assert_eq!(parts[0]["thought"], true);
        assert!(parts[0].get("thoughtSignature").is_none());
        assert_eq!(parts[1]["text"], "Final answer.");
    }

    #[tokio::test]
    async fn alternate_finish_reason_is_retained_without_inventing_native_fields() {
        let result = replay(&[json!({"candidates": [{"finish_reason": "MAX_TOKENS"}]})]).await;
        assert_eq!(result["candidates"][0]["finish_reason"], "MAX_TOKENS");
        assert!(result["candidates"][0].get("finishReason").is_none());
    }

    #[tokio::test]
    async fn signed_thinking_parts_remain_original_and_cache_at_actual_message_count() {
        let session = format!("collector-signed-{}", uuid::Uuid::new_v4());
        let signature = "s".repeat(96);
        let signed = json!({
            "text": "signed thought",
            "thought": true,
            "thoughtSignature": signature,
            "futurePart": {"opaque": true}
        });
        let parts = json!([
            {"text": "before", "thought": true},
            signed,
            {"text": "after", "thought": true}
        ]);
        let frame = json!({"candidates": [{
            "content": {"parts": parts},
            "finishReason": "STOP"
        }]});
        let source = stream::iter(vec![Ok::<Bytes, io::Error>(Bytes::from(format!(
            "data: {}",
            frame
        )))]);
        let response = collect_stream_to_json_with_anchor_and_count(source, &session, None, 17)
            .await
            .unwrap();
        assert_eq!(response["candidates"][0]["content"]["parts"], parts);
        let cache = SignatureCache::global();
        assert_eq!(
            cache.get_session_signature_at(&session, 17),
            Some(signature)
        );
        assert!(cache.get_session_signature_at(&session, 1).is_none());
        cache.delete_session_signature(&session);
    }

    #[tokio::test]
    async fn failed_stream_does_not_replace_previous_session_signature() {
        for transport_error in [false, true] {
            let session = format!("collector-failed-{}", uuid::Uuid::new_v4());
            let previous_signature = "p".repeat(96);
            let cache = SignatureCache::global();
            cache.cache_session_signature(&session, previous_signature.clone(), 4);
            let signed_frame = json!({"candidates": [{"content": {"parts": [{
                "thought": true,
                "text": "incomplete thought",
                "thoughtSignature": "n".repeat(96)
            }]}}]});
            let failure = if transport_error {
                Err(io::Error::other("connection reset"))
            } else {
                Ok(Bytes::from("data: {\"error\":{\"code\":503}}\n\n"))
            };
            let source = stream::iter(vec![
                Ok(Bytes::from(format!("data: {}\n\n", signed_frame))),
                failure,
            ]);
            let result =
                collect_stream_to_json_with_anchor_and_count(source, &session, None, 17).await;
            if transport_error {
                assert!(result.is_err());
            } else {
                assert_eq!(result.unwrap()["error"]["code"], 503);
            }
            assert_eq!(
                cache.get_session_signature_at(&session, 4),
                Some(previous_signature)
            );
            assert!(cache.get_session_signature_at(&session, 17).is_none());
            cache.delete_session_signature(&session);
        }
    }
}
