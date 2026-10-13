# 新 Host → AGM beta.9 自然调用增量查证

新 Host 实际 StartedAt `2026-10-13T01:04:51.662179732Z`，container `78b19c8017927ad193b588c68455781da7ce43f46145b64f53b42f83d5345b45`，image `sha256:87d001a4809ef8cafd538b0093c4cd858f1957e346a2a37ed0e643e81dfc62fe`，revision `d75a89463c491ad7d3eba4b074c2cbd3b8992050`。仅查询该时点后的现存 trace/Gw 数据；Host 使用原id后缀和operation/turn索引，Gw使用timestamp索引，没有重复全库原行扫描、制造请求、QQ投递或修改配置。

初次采样尚在冷启动且健康为unhealthy、响应0；随后 root 实际启动日志确认01:06:01 application startup complete、01:06:14 OneBot connected。本子审计01:07:37读取到8080监听、/livez最近5次检查exit0，容器healthy/restart0/OOM=false。两个状态和首个空捕获均保留，不能把冷启动时尚未bind HTTP称永久故障。当前 Gateway精确image9e03/revision64/4.9.7-beta.9也healthy、HTTP200、restart0/OOM=false，Mihomo未变、主机available985MiB/PSI0。

截至 `2026-10-13T01:08:38.670754+00:00` 的新Host真实provider_response共 1 条；原model_start/route payload明确任务为 `memory_extraction`，不是Main/chat_agent。response与原model/turn行的conversation_id/generation/work_id/execution_id/source_event_id/origin均为NULL；model_start内原ChatRequest provider_session_id也为None。Gw实际入站无x-session-id，其存在性与原请求声明一致，但没有Main业务身份可核验；不能补猜或把它算为Main会话头通过。

本样本 trace `195620` / 原turn `49b36efbf873451db511a488e3f511a0`，原请求完整树在Gw原request_body中唯一匹配1/1；所有键、列表、标量原值以及已有opaque/media红删引用按原hash/bytes核对。同operation start/response/end元数据一致，时序为01:06:30.836018 → Gw 01:06:33.698 → Host实际HTTP记录01:06:33.709847 → end01:06:33.709931 UTC。它是完整请求与时序互证，不是三token项猜对应。Gw consolidated未保responseId，共享response-ID仍unknown，不称直接receipt-ID join。

客户端真实HTTP响应完整usageMetadata为 `{"promptTokenCount":2733,"candidatesTokenCount":13,"totalTokenCount":2746}`，与Gw保存的原native usage全树相同。响应仅thoughtSignature红删，usage子树无红删；原gzip/保存size/SHA在远端与本地均验证通过。归档 10523B，SHA256 `3dd563840e4b56f899c797f9eb7add89702d59972e1b8e27d895b2c50251f86f`，原件/完整请求/签名引用保存在private外部。

模型HTTP200、Gw完整请求耗时2845ms、physical=1且覆盖1/1。raw/stat input=2733/output=13/reported total=2746；cache和thoughts未返回，均SQL NULL，cache coverage=0/1、ratio未知，reasoning coverage=0/1。统计已知值multiset与本新窗口log相同；token_usage无request_id，不伪造逐条账单关联。没有将缓存/思考未知列称0，也不推算真实账单或节省。

新Host Main/chat_agent 自然样本仍为0；实际x-session-id对原conversation/generation/work的Main端到端身份核验尚无样本。本辅助memory_extraction调用与此前旧Host→新Gw的9条自然请求分开记录，不能代替Main验收。

证据目录 `D:/codex-test-tmp/agm-beta9-newhost-acceptance-64b5311`；完整诊断、gzip和请求仅在该外部目录的private/及远端自己的 `/tmp/agm-beta9-newhost-client-trace-20261013-EmtJOXoV`。本文及redacted JSON仅包含元数据/usage，没有聊天正文、签名原值或凭据。
