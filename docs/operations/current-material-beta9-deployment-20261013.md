# AGM v4.9.7-beta.9 上线后只读核查

生产镜像 `sha256:9e03cb48d80428ace810854b3b52886fc15a1770b31d57d66c270f2dc31a670d`，revision `64b53119b33c50eddac38008c5a0a86cf53f4eac`。部署由 root 执行；本审计只执行 Docker inspect/logs、HTTP GET /health、SQLite URI mode=ro 的 SELECT/read transaction。未发模型请求、QQ 消息或重跑 Work，未改生产和仓库。

服务 StartedAt `2026-10-12T23:53:58.8755512Z`。实际启动日志在 23:54:00.825 开始代理服务，在 23:54:00.982 记录正在运行。00:11:30 UTC 的 HTTP health=200 / status=ok / version=4.9.7-beta.9；Docker healthy、FailingStreak=0、restart=0、OOM=false。Mihomo 的 image、StartedAt、restart=0、OOM=false、healthy 与审计初始状态一致。主机 available=964 MiB；memory PSI some/full avg10/60/300 均为 0.00。审计未创建容器；先前自己的镜像验证容器已移除。

截至已捕获的 runtime log，ERROR=0。两个启动 WARN 是 abort previous auto-cleanup/proactive refresh；00:05:25–28 的三个 QuotaSummary endpoint 返回 403 WARN。它们应如实保留；本样本的模型请求均 HTTP 200，不把后台 quota 警告称为模型失败。

备份原件位于 `/opt/antigravity-manager/backups/pre-beta9-20261012T235237Z`。备份的五个 DB integrity=ok 是 root 已执行的事实，本审计未重复运行 integrity 或 checkpoint。下面是 00:07:10 UTC 的逐原键、全部旧字段比对；请求/思考报文按行流式 hash，未向报告输出正文或凭证。

| Database / table | Backup | Live | Original rows present | Exact original / declared migration matches | Natural additions |
|---|---:|---:|---:|---:|---:|
| proxy_logs.db / request_logs | 1186 | 1195 | 1186 | 1186 | 9 |
| proxy_logs.db / tool_signatures | 3311 | 3293 | 3289 | 3289 | 4 |
| security.db / ip_access_logs | 0 | 0 | 0 | 0 | 0 |
| security.db / ip_blacklist | 0 | 0 | 0 | 0 | 0 |
| security.db / ip_whitelist | 0 | 0 | 0 | 0 | 0 |
| thinking_store.db / thinking_meta | 1 | 1 | 1 | 1 | 0 |
| thinking_store.db / thinking_records | 11033 | 11042 | 11033 | 11033 | 9 |
| thinking_store.db / thinking_sessions | 3910 | 3910 | 3910 | 3909 | 0 |
| token_stats.db / token_stats_hourly | 339 | 340 | 339 | 338 | 1 |
| token_stats.db / token_usage | 14432 | 14441 | 14432 | 14432 | 9 |
| user_tokens.db / token_ip_bindings | 0 | 0 | 0 | 0 | 0 |
| user_tokens.db / token_usage_logs | 0 | 0 | 0 | 0 | 0 |
| user_tokens.db / user_tokens | 0 | 0 | 0 | 0 | 0 |

1186 条原 request_logs 的全部旧字段逐行保持，原投影 SHA256=`d3f7903b81ad9fc8f2faecfd5d00a8ec23e4821fa4fadaeb799961890a1ceff4`。11033 条原 thinking_records 完全保持，原投影 SHA256=`1010ec386bdfbe65eaf5237a97fd7f8045a53ec1cf995039c9479c8e411a1b73`。3910 个原 thinking_sessions 均在，只有一个现行 session 的 last_accessed 随自然请求推进。339 个原 hourly 均在，只有当前 hour/account bucket 的 input/output/total/request_count 随 23:57 的自然请求增量更新，另有一个自然新 bucket。

14432 条原 token_usage 均在，并且与源码声明的 nullable 迁移逐行匹配；匹配投影 SHA256=`cb1e67bb921be8d9f25352f9c8ea5f5547f2239797a578900723810465992ba2`。旧 reasoning 假 0 → NULL 14405 条、旧未知 cache 0 → NULL 5230 条。原 derived total_tokens 没有改变；所有历史 reported_total_tokens 仍 NULL、cache_reported 均 0，没有把无法恢复的历史判成实报值。

tool_signatures 有 22 条原键移除和 4 条自然新增，其他 3289 条原键/签名/时间字段逐行保持。这 22 条 created_at 在 2026-10-05 23:51:56.068–2026-10-06 00:01:32.727 UTC，全部早于采样时间减七天的 2026-10-06 00:07:10.339；与 beta8 已存在的七天 TTL 清理合同一致。部分时间晚于新服务启动减七天，因此不声称它们全在启动瞬间清理，也不宣称工具签名表的总 hash 未变。已保存 beta8 对应 proxy_db.rs 原文和差异私有证据。

实际 schema：token_usage 的 input_tokens, output_tokens, cached_tokens, reasoning_tokens, total_tokens, reported_total_tokens 允许 NULL；request_logs 的新字段 reasoning_tokens, physical_requests, total_tokens 允许 NULL。hourly 已有 cache_reported_requests, cache_reported_input_tokens, cache_reported_tokens, reported_total_tokens, total_reported_requests, output_reported_requests, input_reported_requests。原历史 cache classification 不回填；raw 实报 total 与旧 derived total 分开读取。

自然请求样本为旧 Host → 新 AGM，模型 gemini-3.8-flash 映射 gemini-3.8-flash-low，均为 native generateContent。此证据不宣称新 Host 已上线验收。

| UTC time | HTTP | Physical | Input | Output | Cache | Reasoning | Reported total |
|---|---:|---:|---:|---:|---:|---:|---:|
| 23:57:47.079 | 200 | 1 | 34780 | 21 | NULL / 未返回 | NULL / 未返回 | 34801 |
| 00:00:38.339 | 200 | 1 | 32484 | 56 | 16070 | NULL / 未返回 | 32540 |
| 00:00:44.431 | 200 | 1 | 33108 | 17 | 30499 | NULL / 未返回 | 33125 |
| 00:01:10.244 | 200 | 1 | 32622 | 58 | 16069 | NULL / 未返回 | 32680 |
| 00:01:16.531 | 200 | 1 | 33253 | 17 | 30637 | NULL / 未返回 | 33270 |
| 00:01:41.974 | 200 | 1 | 32775 | 69 | 16067 | NULL / 未返回 | 32844 |
| 00:01:47.623 | 200 | 1 | 33419 | 17 | 30791 | NULL / 未返回 | 33436 |
| 00:02:11.870 | 200 | 1 | 32939 | 64 | 16065 | NULL / 未返回 | 33003 |
| 00:02:15.960 | 200 | 1 | 33577 | 17 | 30955 | NULL / 未返回 | 33594 |

9 条新增 request log 的 Gemini 原 usage 数值字段与 raw 计量值一致；9 条新增 token_usage 的已知用量 multiset 与这批日志一致。token_usage 没有 request_id，因此 multiset 一致仅是聚合事实，不伪造逐条 receipt join。累计 input=298957、output=336、reported total=299293。cache 实报 8/9，总 187153；首条未返 cache，raw=SQL NULL。thoughts 9 条均未返回，raw reasoning 均 NULL，不声称实际 reasoning=0。hourly input/output/total coverage=9，cache coverage=8、known cache input=264177、cache tokens=187153，实报 total=299293。

9 条旧 Host 请求均无任何会话头、无 body 显式稳定 session 字段。AGM 返回同一 client session，实际 response x-session-id/x-antigravity-session-id 与 log.session_id 9/9 一致；上游 sessionId 9 条相同，账号也相同。runtime 9 次 ensured binding、8 次 successful reuse。它证明当前缺省内容 fallback 路径的实际粘性，不证明新 Host 内部 execution 身份贯通；新 Host 会话头与完整客户端 usage 子树由 root 读取实际 provider_response HTTPResponseSnapshot 另行核对。

Monitor 的 response_body 在持久化前转换为 consolidated 诊断报文，并以原上游 capture 填 usage。因此本报告称它为“保存的上游用量/监控投影”，不称完整客户端 wire 原件。[客户端查证](current-material-beta9-client-usage-20261013.md)已沿Host实际HTTP响应快照核验完整usage子树9/9一致；新Host身份验收另记。

私有证据保存在外部 `D:/codex-test-tmp/agm-beta9-postdeploy-64b5311/private` 与远端 `/root/ops/agm-beta9-release-64b5311-20261013-jrikTbDg/postdeploy`，不在仓库中。公开报告只有时间、用量、schema、hash 和 redacted 状态；没有请求正文、签名、账号名或凭证。
