//! 纯思考空回复流式自愈管道（Thinking Dropout Auto-Healing Pipeline）
//!
//! 核心防御场景：
//! 超长上下文（如 40 万+ tokens / 700+ 轮）下，大模型（尤其是 Gemini Flash 系列）在完成思考后，
//! 概率性因注意力坍缩直接输出结束符 `<end_of_turn>` / `STOP`，导致 `candidatesTokenCount == 0`，
//! 即“只返回了思考块，正文与工具调用皆为空”。
//! 客户端（如 JeikCode / Codex / Claude Code 等）收到仅有思考的回复后，会因状态机无法推进而直接异常断开。
//!
//! 自愈拦截策略（Tail Interception & Auto-Piping）：
//! 1. 思考块正常实时透传给下游客户端，确保首字延迟（TTFT）与实时思考动画丝滑展示；
//! 2. 拦截流末尾的过早终止符（`finishReason: "STOP"` 与 `[DONE]`），不向客户端发射；
//! 3. 向客户端发射 SSE 心跳注释（`: auto-healing empty thinking\n\n`）保持连接存活；
//! 4. 自动构造带原上下文的自愈请求，追加共享非指令协议占位（不代表新增用户输入），在相同账号上发起上游调用；
//! 5. 将上游续跑流无缝缝合至当前下游客户端连接；
//! 6. 严格实施单次自愈上限（`max_auto_heals = 1`）；自愈失败作为流错误传播，不伪造成功正文。

use bytes::{Bytes, BytesMut};
use futures::{Stream, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use crate::proxy::mappers::common_utils::TRANSIT_DEFENSE_FALLBACK_TEXT;
use crate::proxy::upstream::client::UpstreamClient;

/// 思考空回复自愈上下文
pub struct ThinkingAutoHealContext {
    pub upstream: Arc<UpstreamClient>,
    pub method: &'static str,
    pub access_token: String,
    pub original_body: Value,
    pub query_string: Option<&'static str>,
    pub extra_headers: HashMap<String, String>,
    pub account_id: Option<String>,
    pub trace_id: String,
}

/// 保留原上下文与身份，仅为单次自愈追加非指令协议占位。
pub fn create_auto_heal_continuation_body(original_body: &Value) -> Value {
    let mut new_body = original_body.clone();

    // 1. 为 requestId 追加自愈标记，避免上游缓存或去重干扰
    if let Some(req_id) = new_body.get_mut("requestId").and_then(|v| v.as_str()) {
        new_body["requestId"] = json!(format!("{}_heal1", req_id));
    }
    if let Some(req) = new_body.get_mut("request").and_then(|r| r.as_object_mut()) {
        if let Some(req_id) = req.get_mut("requestId").and_then(|v| v.as_str()) {
            req["requestId"] = json!(format!("{}_heal1", req_id));
        }
    }

    // 2. 追加非空协议占位，不伪造用户的继续执行指令。
    let user_turn = json!({
        "role": "user",
        "parts": [{ "text": TRANSIT_DEFENSE_FALLBACK_TEXT }]
    });

    if let Some(contents) = new_body
        .get_mut("request")
        .and_then(|r| r.get_mut("contents"))
        .and_then(|c| c.as_array_mut())
    {
        contents.push(user_turn);
    } else if let Some(contents) = new_body.get_mut("contents").and_then(|c| c.as_array_mut()) {
        contents.push(user_turn);
    }

    new_body
}

/// 检查 Gemini candidate 中的部件类型分布
fn inspect_gemini_candidate_parts(
    candidate: &Value,
    saw_thought: &mut bool,
    saw_content: &mut bool,
    saw_tool_call: &mut bool,
) {
    if let Some(parts) = candidate
        .get("content")
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
    {
        for part in parts {
            let is_thought = part
                .get("thought")
                .and_then(|t| t.as_bool())
                .unwrap_or(false);
            if is_thought {
                *saw_thought = true;
            }
            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                if !is_thought && !text.trim().is_empty() {
                    *saw_content = true;
                }
            }
            if part.get("inlineData").is_some()
                || part.get("inline_data").is_some()
                || part.get("fileData").is_some()
                || part.get("file_data").is_some()
            {
                *saw_content = true;
            }
            if part.get("functionCall").is_some() {
                *saw_tool_call = true;
            }
        }
    }
}

/// 自愈仍无正文或工具调用时，不能先向下游发布成功终止符。
fn auto_heal_output_line(line: &str, saw_output: &mut bool) -> Option<Bytes> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    if let Some(json_part) = line.strip_prefix("data: ") {
        let json_part = json_part.trim();
        if json_part == "[DONE]" {
            return (*saw_output).then(|| Bytes::from("data: [DONE]\n\n"));
        }
        if let Ok(mut json) = serde_json::from_str::<Value>(json_part) {
            let inner = if json.get("response").is_some() {
                json.get_mut("response").unwrap()
            } else {
                &mut json
            };
            let mut has_parts = false;
            let mut has_finish = false;
            if let Some(candidates) = inner.get_mut("candidates").and_then(Value::as_array_mut) {
                for candidate in candidates.iter_mut() {
                    let mut thought = false;
                    let mut content = false;
                    let mut tool_call = false;
                    inspect_gemini_candidate_parts(
                        candidate,
                        &mut thought,
                        &mut content,
                        &mut tool_call,
                    );
                    *saw_output |= content || tool_call;
                    has_parts |= candidate
                        .get("content")
                        .and_then(|c| c.get("parts"))
                        .and_then(Value::as_array)
                        .is_some_and(|parts| !parts.is_empty());
                    has_finish |= candidate.get("finishReason").is_some();
                }
                if !*saw_output && has_finish {
                    if !has_parts {
                        return None;
                    }
                    for candidate in candidates {
                        if let Some(candidate) = candidate.as_object_mut() {
                            candidate.remove("finishReason");
                        }
                    }
                    if let Some(inner) = inner.as_object_mut() {
                        inner.remove("usageMetadata");
                    }
                    return Some(Bytes::from(format!("data: {}\n\n", json)));
                }
            }
        }
    }
    Some(Bytes::from(format!("{}\n\n", line)))
}

/// 检查唯一一次自愈的真实输出，并以无凭据错误报告失败。
fn checked_auto_heal_stream<S, E>(
    mut stream: Pin<Box<S>>,
) -> Pin<Box<dyn Stream<Item = Result<Bytes, String>> + Send>>
where
    S: Stream<Item = Result<Bytes, E>> + Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    Box::pin(async_stream::stream! {
        let mut buffer = BytesMut::new();
        let mut saw_output = false;
        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    buffer.extend_from_slice(&bytes);
                    while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                        let raw = buffer.split_to(pos + 1);
                        match std::str::from_utf8(&raw) {
                            Ok(line) => {
                                if let Some(bytes) = auto_heal_output_line(line, &mut saw_output) {
                                    yield Ok(bytes);
                                }
                            }
                            Err(_) => {
                                yield Ok(raw.freeze());
                            }
                        }
                    }
                }
                Err(_) => {
                    yield Err("auto_heal_stream_read_failed".to_string());
                    return;
                }
            }
        }
        if !buffer.is_empty() {
            match std::str::from_utf8(&buffer) {
                Ok(line) => {
                    if let Some(bytes) = auto_heal_output_line(line, &mut saw_output) {
                        yield Ok(bytes);
                    }
                }
                Err(_) => {
                    yield Ok(buffer.freeze());
                }
            }
        }
        if !saw_output {
            yield Err("auto_heal_empty_response".to_string());
        }
    })
}

/// 统一纯思考空回复自愈流式包装器（Pipeline First）
///
/// 对上游原始 Gemini SSE 流进行透明包装：
/// - 思考内容实时透传；
/// - 若正常产出正文或工具调用，全流程无任何额外开销；
/// - 若发现仅产出思考后立即返回终止符，截留终止信号并在同一连接中自动续跑；
/// - 续跑上限严格为 1 次；失败通过流错误报告，不生成模型正文。
pub fn wrap_stream_with_empty_thinking_auto_heal<S, E>(
    stream: Pin<Box<S>>,
    ctx: ThinkingAutoHealContext,
) -> Pin<Box<dyn Stream<Item = Result<Bytes, String>> + Send>>
where
    S: Stream<Item = Result<Bytes, E>> + Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    let stream = async_stream::stream! {
        let mut buffer = BytesMut::new();
        let mut saw_thought = false;
        let mut saw_content = false;
        let mut saw_tool_call = false;
        let mut saw_finish_reason = false;
        let mut finish_reason_val: Option<String> = None;
        let mut auto_healed = false;

        let mut stream1 = stream;
        while let Some(item) = stream1.next().await {
            match item {
                Ok(bytes) => {
                    buffer.extend_from_slice(&bytes);
                    while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                        let line_raw = buffer.split_to(pos + 1);
                        let line_str = match std::str::from_utf8(&line_raw) {
                            Ok(s) => s,
                            Err(_) => {
                                yield Ok(line_raw.freeze());
                                continue;
                            }
                        };
                        let line = line_str.trim();
                        if line.is_empty() {
                            continue;
                        }

                        if line.starts_with("data: ") {
                            let json_part = line.trim_start_matches("data: ").trim();
                            if json_part == "[DONE]" {
                                if saw_content || saw_tool_call {
                                    yield Ok(Bytes::from("data: [DONE]\n\n"));
                                }
                                continue;
                            }

                            match serde_json::from_str::<Value>(json_part) {
                                Ok(mut json) => {
                                    let has_resp = json.get("response").is_some();
                                    let inner = if has_resp {
                                        json.get_mut("response").unwrap()
                                    } else {
                                        &mut json
                                    };
                                    let mut chunk_has_finish = false;
                                    let mut chunk_has_parts = false;

                                    if let Some(candidates) = inner.get_mut("candidates").and_then(|c| c.as_array_mut()) {
                                        for cand in candidates.iter_mut() {
                                            inspect_gemini_candidate_parts(
                                                cand,
                                                &mut saw_thought,
                                                &mut saw_content,
                                                &mut saw_tool_call,
                                            );
                                            if let Some(parts) = cand.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                                                if !parts.is_empty() {
                                                    chunk_has_parts = true;
                                                }
                                            }
                                            if let Some(fr) = cand.get("finishReason").and_then(|f| f.as_str()) {
                                                chunk_has_finish = true;
                                                saw_finish_reason = true;
                                                finish_reason_val = Some(fr.to_string());
                                            }
                                        }
                                    }

                                    if saw_content || saw_tool_call {
                                        // 正常回复流程：已看到正文或工具调用，原样放行
                                        yield Ok(Bytes::from(format!("data: {}\n\n", json_part)));
                                    } else if saw_thought && chunk_has_finish {
                                        // 思考空回复异常候选帧：
                                        // 若当前帧还包含思考部件，清洗剥离 finishReason 与 usageMetadata 后放行思考，不发射终止信号
                                        if chunk_has_parts {
                                            if let Some(candidates) = inner.get_mut("candidates").and_then(|c| c.as_array_mut()) {
                                                for cand in candidates.iter_mut() {
                                                    if let Some(obj) = cand.as_object_mut() {
                                                        obj.remove("finishReason");
                                                    }
                                                }
                                            }
                                            if let Some(obj) = inner.as_object_mut() {
                                                obj.remove("usageMetadata");
                                            }
                                            let sanitized = serde_json::to_string(&json).unwrap_or_default();
                                            yield Ok(Bytes::from(format!("data: {}\n\n", sanitized)));
                                        }
                                        // 若无有效部件仅有 finishReason，彻底拦截暂存，不往下游发送
                                    } else {
                                        // 普通思考块或中间流帧：原样透传
                                        yield Ok(Bytes::from(format!("data: {}\n\n", json_part)));
                                    }
                                }
                                Err(_) => {
                                    yield Ok(Bytes::from(format!("{}\n\n", line)));
                                }
                            }
                        } else {
                            // 保持非 data 行（如保活注释）正常下发
                            yield Ok(Bytes::from(format!("{}\n\n", line)));
                        }
                    }
                }
                Err(e) => {
                    yield Err(e.to_string());
                    return;
                }
            }
        }

        // 冲刷残留缓冲区
        if !buffer.is_empty() {
            if let Ok(line_str) = std::str::from_utf8(&buffer) {
                let line = line_str.trim();
                if !line.is_empty() {
                    yield Ok(Bytes::from(format!("{}\n\n", line)));
                }
            }
        }

        // 核心自愈判断条件：
        // 1. 存在思考块 (saw_thought)
        // 2. 无任何正文 (saw_content == false)
        // 3. 无任何工具调用 (saw_tool_call == false)
        // 4. 上游正常返回了结束符 (saw_finish_reason == true)
        // 5. 尚未执行过自愈 (auto_healed == false，严格 1 次上限)
        if saw_thought && !saw_content && !saw_tool_call && saw_finish_reason && !auto_healed {
            auto_healed = true;
            tracing::warn!(
                "[{}] [Stream-AutoHeal] 🚨 Detected empty thinking completion (thought present, 0 content, 0 tool_calls, finishReason={:?}). Triggering bounded auto-heal with protocol placeholder 1/1...",
                ctx.trace_id, finish_reason_val
            );

            // 发射 SSE 心跳注释保持客户端下游连接存活
            yield Ok(Bytes::from(": auto-healing empty thinking\n\n"));

            let heal_body = create_auto_heal_continuation_body(&ctx.original_body);

            let call_res = ctx.upstream.call_v1_internal_with_headers(
                ctx.method,
                &ctx.access_token,
                heal_body,
                ctx.query_string,
                ctx.extra_headers.clone(),
                ctx.account_id.as_deref(),
            ).await;

            match call_res {
                Ok(call_success) if call_success.response.status().is_success() => {
                    tracing::info!(
                        "[{}] [Stream-AutoHeal] ✓ Auto-heal request succeeded (HTTP 200), piping healed stream directly into client connection...",
                        ctx.trace_id
                    );
                    let mut stream2 = checked_auto_heal_stream(Box::pin(call_success.response.bytes_stream()));
                    while let Some(item) = stream2.next().await {
                        let failed = item.is_err();
                        yield item;
                        if failed {
                            return;
                        }
                    }
                }
                Ok(call_fail) => {
                    let status = call_fail.response.status();
                    tracing::error!(
                        "[{}] [Stream-AutoHeal] Auto-heal request returned HTTP {}.",
                        ctx.trace_id, status
                    );
                    yield Err(format!("auto_heal_upstream_http_error: {}", status.as_u16()));
                    return;
                }
                Err(_) => {
                    tracing::error!(
                        "[{}] [Stream-AutoHeal] Auto-heal upstream request failed.",
                        ctx.trace_id
                    );
                    yield Err("auto_heal_upstream_request_failed".to_string());
                    return;
                }
            }
        }
    };
    Box::pin(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_context() -> ThinkingAutoHealContext {
        ThinkingAutoHealContext {
            upstream: Arc::new(UpstreamClient::new(None, None)),
            method: "streamGenerateContent",
            access_token: "unused-offline-fixture".to_string(),
            original_body: json!({"contents": []}),
            query_string: Some("alt=sse"),
            extra_headers: HashMap::new(),
            account_id: None,
            trace_id: "offline-auto-heal".to_string(),
        }
    }

    fn sse(value: &Value) -> Bytes {
        Bytes::from(format!("data: {}\n\n", value))
    }

    #[tokio::test]
    async fn test_first_stream_error_is_preserved_without_healing() {
        let source = futures::stream::iter(vec![Err::<Bytes, _>(std::io::Error::other(
            "original-stream-error",
        ))]);
        let results =
            wrap_stream_with_empty_thinking_auto_heal(Box::pin(source), fixture_context())
                .collect::<Vec<_>>()
                .await;
        assert_eq!(results, vec![Err("original-stream-error".to_string())]);
    }

    #[tokio::test]
    async fn test_normal_first_stream_preserves_content_tools_and_media_without_healing() {
        for part in [
            json!({"text": "Original response."}),
            json!({"functionCall": {"id": "call_1", "name": "send_message", "args": {}},
                "thoughtSignature": "original-signature"}),
            json!({"inlineData": {"mimeType": "image/png", "data": "fixture"}}),
            json!({"fileData": {"mimeType": "image/png", "fileUri": "https://example.invalid/fixture.png"}}),
        ] {
            let frames = vec![
                sse(&json!({"candidates": [{
                    "content": {"role": "model", "parts": [part]}, "finishReason": "STOP"
                }]})),
                Bytes::from("data: [DONE]\n\n"),
            ];
            let source = futures::stream::iter(
                frames
                    .iter()
                    .cloned()
                    .map(Ok::<_, std::io::Error>)
                    .collect::<Vec<_>>(),
            );
            let results =
                wrap_stream_with_empty_thinking_auto_heal(Box::pin(source), fixture_context())
                    .collect::<Vec<_>>()
                    .await;
            assert_eq!(results, frames.into_iter().map(Ok).collect::<Vec<_>>());
        }
    }

    #[tokio::test]
    async fn test_healed_stream_preserves_real_output_and_fragmented_terminal_frames() {
        for part in [
            json!({"text": "Original response."}),
            json!({"functionCall": {"id": "call_1", "name": "send_message", "args": {}},
                "thoughtSignature": "original-signature"}),
            json!({"inlineData": {"mimeType": "image/png", "data": "fixture"}}),
            json!({"fileData": {"mimeType": "image/png", "fileUri": "https://example.invalid/fixture.png"}}),
        ] {
            let frame = sse(&json!({"response": {"candidates": [{
                "content": {"role": "model", "parts": [part]}, "finishReason": "STOP"
            }], "usageMetadata": {"totalTokenCount": 17}}}));
            let chunks = vec![
                Ok::<_, std::io::Error>(frame.slice(..11)),
                Ok(frame.slice(11..)),
                Ok(Bytes::from("data: [DONE]")),
            ];
            let results = checked_auto_heal_stream(Box::pin(futures::stream::iter(chunks)))
                .collect::<Vec<_>>()
                .await;
            assert_eq!(
                results,
                vec![Ok(frame), Ok(Bytes::from("data: [DONE]\n\n"))]
            );
        }
    }

    #[tokio::test]
    async fn test_empty_healed_stream_reports_error_without_publishing_success() {
        for parts in [
            json!([]),
            json!([{"thought": true, "text": "Original thought."}]),
        ] {
            let frame = sse(&json!({"candidates": [{
                "content": {"role": "model", "parts": parts}, "finishReason": "STOP"
            }]}));
            let source = futures::stream::iter(vec![
                Ok::<_, std::io::Error>(frame),
                Ok(Bytes::from("data: [DONE]\n\n")),
            ]);
            let results = checked_auto_heal_stream(Box::pin(source))
                .collect::<Vec<_>>()
                .await;
            assert_eq!(
                results.last(),
                Some(&Err("auto_heal_empty_response".to_string()))
            );
            for bytes in results.iter().filter_map(|result| result.as_ref().ok()) {
                let text = std::str::from_utf8(bytes).unwrap();
                assert!(!text.contains("finishReason"));
                assert!(!text.contains("[DONE]"));
                assert!(!text.contains("task ready"));
            }
            let gemini = crate::proxy::mappers::gemini::collector::collect_stream_to_json(
                futures::stream::iter(results),
                "offline-auto-heal",
            )
            .await;
            assert!(gemini.is_err());
        }
    }

    #[tokio::test]
    async fn test_healed_read_error_is_terminal_and_does_not_leak_transport_details() {
        let frame =
            sse(&json!({"candidates": [{"content": {"parts": [{"text": "Actual output."}]}}]}));
        let source = futures::stream::iter(vec![
            Ok(frame.clone()),
            Err(std::io::Error::other(
                "access_token=private-fixture https://private-endpoint.invalid",
            )),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ]);
        let results = checked_auto_heal_stream(Box::pin(source))
            .collect::<Vec<_>>()
            .await;
        assert_eq!(
            results,
            vec![Ok(frame), Err("auto_heal_stream_read_failed".to_string())]
        );
    }

    #[tokio::test]
    async fn test_claude_collector_receives_empty_heal_error_before_message_stop() {
        let frame = sse(&json!({"candidates": [{"finishReason": "STOP"}]}));
        let source = futures::stream::iter(vec![
            Ok::<_, std::io::Error>(frame),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ]);
        let checked = checked_auto_heal_stream(Box::pin(source));
        let claude = crate::proxy::mappers::claude::create_claude_sse_stream(
            checked,
            "offline-auto-heal".to_string(),
            "fixture@example.invalid".to_string(),
            None,
            false,
            128_000,
            None,
            0,
            None,
            vec![],
        );
        let response = crate::proxy::mappers::claude::collector::collect_stream_to_json(
            claude.map(|item| item.map_err(std::io::Error::other)),
        )
        .await;
        assert!(response.is_err());
    }

    #[test]
    fn test_create_auto_heal_continuation_body() {
        let original = json!({
            "requestId": "agent/1234/1",
            "sessionId": "original-session",
            "request": {
                "requestId": "agent/1234/1",
                "systemInstruction": {"parts": [{"text": "Original system instruction."}]},
                "tools": [{"functionDeclarations": [{"name": "send_message"}]}],
                "generationConfig": {"temperature": 0.4},
                "contents": [
                    { "role": "user", "parts": [{ "text": "Hello" }] },
                    { "role": "model", "parts": [{"functionCall": {
                        "id": "call_send", "name": "send_message", "args": {"text": "Original message."}
                    }, "thoughtSignature": "original-signature"}] },
                    { "role": "model", "parts": [{"functionResponse": {
                        "id": "call_send", "name": "send_message", "response": {"delivered": true}
                    }}] }
                ]
            }
        });
        let frozen = original.clone();
        let healed = create_auto_heal_continuation_body(&original);
        assert_eq!(healed["requestId"], "agent/1234/1_heal1");
        assert_eq!(healed["request"]["requestId"], "agent/1234/1_heal1");
        let mut restored = healed;
        let placeholder = restored["request"]["contents"]
            .as_array_mut()
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(
            placeholder,
            json!({
                "role": "user",
                "parts": [{"text": "[Protocol placeholder: no additional user input.]"}]
            })
        );
        restored["requestId"] = original["requestId"].clone();
        restored["request"]["requestId"] = original["request"]["requestId"].clone();
        assert_eq!(restored, original);
        assert_eq!(original, frozen);
    }

    #[test]
    fn test_inspect_candidate_parts_pure_thought() {
        let cand = json!({
            "content": {
                "parts": [
                    { "thought": true, "text": "I should run git pull" }
                ]
            }
        });
        let mut saw_thought = false;
        let mut saw_content = false;
        let mut saw_tool_call = false;
        inspect_gemini_candidate_parts(
            &cand,
            &mut saw_thought,
            &mut saw_content,
            &mut saw_tool_call,
        );
        assert!(saw_thought);
        assert!(!saw_content);
        assert!(!saw_tool_call);
    }

    #[test]
    fn test_inspect_candidate_parts_with_tool_call() {
        let cand = json!({
            "content": {
                "parts": [
                    { "thought": true, "text": "I will call bash" },
                    { "functionCall": { "name": "run_command", "args": {} } }
                ]
            }
        });
        let mut saw_thought = false;
        let mut saw_content = false;
        let mut saw_tool_call = false;
        inspect_gemini_candidate_parts(
            &cand,
            &mut saw_thought,
            &mut saw_content,
            &mut saw_tool_call,
        );
        assert!(saw_thought);
        assert!(!saw_content);
        assert!(saw_tool_call);
    }

    #[test]
    fn test_inspect_candidate_parts_with_content() {
        let cand = json!({
            "content": {
                "parts": [
                    { "thought": true, "text": "thinking..." },
                    { "text": "Here is the response" }
                ]
            }
        });
        let mut saw_thought = false;
        let mut saw_content = false;
        let mut saw_tool_call = false;
        inspect_gemini_candidate_parts(
            &cand,
            &mut saw_thought,
            &mut saw_content,
            &mut saw_tool_call,
        );
        assert!(saw_thought);
        assert!(saw_content);
        assert!(!saw_tool_call);
    }

    #[test]
    fn test_inspect_candidate_parts_inline_data() {
        let cand = json!({
            "content": {
                "parts": [
                    { "thought": true, "text": "generating image..." },
                    { "inlineData": { "mimeType": "image/png", "data": "base64..." } }
                ]
            }
        });
        let mut saw_thought = false;
        let mut saw_content = false;
        let mut saw_tool_call = false;
        inspect_gemini_candidate_parts(
            &cand,
            &mut saw_thought,
            &mut saw_content,
            &mut saw_tool_call,
        );
        assert!(saw_thought);
        assert!(saw_content);
        assert!(!saw_tool_call);
    }

    #[test]
    fn test_create_auto_heal_continuation_body_root_contents() {
        let original = json!({
            "requestId": "req_root_123",
            "contents": [
                { "role": "user", "parts": [{ "text": "看看这条状态" }] }
            ]
        });
        let healed = create_auto_heal_continuation_body(&original);
        assert_eq!(healed["requestId"], "req_root_123_heal1");
        let contents = healed["contents"].as_array().unwrap();
        assert_eq!(contents.len(), 2);
        assert_eq!(contents[1]["role"], "user");
        assert_eq!(contents[0], original["contents"][0]);
        assert_eq!(
            contents[1]["parts"][0]["text"],
            TRANSIT_DEFENSE_FALLBACK_TEXT
        );
        assert_eq!(original["requestId"], "req_root_123");
    }

    #[test]
    fn test_auto_heal_without_request_id_only_adds_protocol_placeholder() {
        let original = json!({
            "request": {
                "contents": []
            }
        });
        let healed = create_auto_heal_continuation_body(&original);
        assert!(healed.get("requestId").is_none());
        assert!(healed["request"].get("requestId").is_none());
        assert_eq!(
            healed["request"]["contents"],
            json!([{
                "role": "user", "parts": [{"text": TRANSIT_DEFENSE_FALLBACK_TEXT}]
            }])
        );
        assert!(original["request"]["contents"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}
