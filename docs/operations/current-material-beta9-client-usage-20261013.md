# 旧 Host → AGM beta.9 实际 HTTP 客户端用量查证

只读读取 yuki-server `/opt/yuki-qqbot/data/qq_ai_bot.db` 已存在 execution_trace_entries，时间窗口 `2026-10-12 23:53:58`–`2026-10-13 00:21:02.895738` UTC。没有修改 trace 配置、制造入站、模型调用或重跑。捕获前后仍是旧 Host `ops-e30c15b4` / image `sha256:52fb83741180b494bb6e1b134da668dd6a3b3b984b45f34663428367e7bef346`，healthy、restart=0、OOM=false。本证据不表示新 Host 验收。

实际 provider_response 9 条、同 operation provider_start/end 18 条。9 个响应 payload_status=redacted，原 gzip、保存的 payload_bytes/payload_sha256 均完整；远端解压与本地下载后二次验证全部通过。归档 661932 B / SHA256 `063407c567f727a466830b39e8b9cc67a09fb6e7062008ed62dd39545d82f9b1`。压缩原件、解压完整诊断和原请求仅保存在 private/，报告不输出正文、签名或凭证。

record_http_response 在实际 httpx 返回后、协议解析前保存 HTTPResponseSnapshot。9 个响应的 redaction 均仅涉及 thoughtSignature，完整 usageMetadata 子树没有 redaction。它证明已保存 HTTP JSON 中的完整 usage 值；JSON 编码与签名红删意味着不宣称整份 wire 字节完全相同。

9/9 个 Host provider_start 原请求树与 Gw received request_body 唯一匹配；被红删的 opaque/media 值按保存的原 bytes/hash引用逐个校验，其余键、列表顺序、标量原值全部相同。各响应与 start/end 的原 operation/turn/execution 一致，Host start → Gw 完成日志 → Host response record → Host end 的 wall-clock顺序9/9成立。没有仅按 input/output/total 猜测关联。

Gw consolidated response 未保存 responseId，Host 实际响应均有 responseId。因此 shared responseId=unknown，不能把这一跨系统内容/时序互证称为共享 request/receipt ID 的直接数据库 join。原 Host operation/turn/execution/source_event_id 来自真实持久元数据，没有从平台 ID 或正文猜归属。

| Internal trace ID | Client response UTC | Unique full request candidates | Prompt | Candidates | Cache | Reported total | Whole usage tree equal |
|---|---|---:|---:|---:|---:|---:|---|
| 195503 | 23:57:47.101491 | 1 | 34780 | 21 | 未返回 | 34801 | True |
| 195516 | 00:00:38.364871 | 1 | 32484 | 56 | 16070 | 32540 | True |
| 195529 | 00:00:44.452558 | 1 | 33108 | 17 | 30499 | 33125 | True |
| 195543 | 00:01:10.268804 | 1 | 32622 | 58 | 16069 | 32680 | True |
| 195556 | 00:01:16.553292 | 1 | 33253 | 17 | 30637 | 33270 | True |
| 195570 | 00:01:41.998675 | 1 | 32775 | 69 | 16067 | 32844 | True |
| 195583 | 00:01:47.652441 | 1 | 33419 | 17 | 30791 | 33436 | True |
| 195597 | 00:02:11.903192 | 1 | 32939 | 64 | 16065 | 33003 | True |
| 195610 | 00:02:15.986010 | 1 | 33577 | 17 | 30955 | 33594 | True |

9/9 个客户端完整 usageMetadata 子树与 Gw 保存的原 native usage 全树相同：prompt/candidates/reported total，以及 8 条实报 cache 值；第一条没有 cache 字段，9 条都没有 thoughtsTokenCount。Gw raw/SQL 对这些缺项保持 NULL。此前 Gw 的9条 raw 计量、coverage 与这些实报用量已经验证一致，累计 input=298957/output=336/reported total=299293、cache187153且coverage8/9。token_usage无request_id，因此仍只把SQL一致性称为该自然批次的聚合事实。

外部证据目录 `D:/codex-test-tmp/agm-beta9-oldhost-client-trace-64b5311`；远端自己的私有目录 `/tmp/agm-beta9-oldhost-client-trace-20261013-xQ6u1mtg`。helper第一次归档自包含目录触发tar目录mtime提示，后续仅对已捕获文件重新归档并校验，没有再次查询生产DB或掩盖产品失败。
