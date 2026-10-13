# 新 Host d75 → AGM beta.9 自然调用查证（固定窗口）

本报告只包含新 Host `d75a89463c491ad7d3eba4b074c2cbd3b8992050` → 优化 AGM `64b53119b33c50eddac38008c5a0a86cf53f4eac` / `v4.9.7-beta.9`。窗口为 `2026-10-13T01:04:51.662179732Z`–`2026-10-13T03:45:53.893508Z`。旧 Host 的9条、此前新 Host 第一条 Memory 辅助报告、后续 compiler 排序修正版本都不混作本窗口的不同版本验收；本窗口按新 Host 启动时点累计，因此包含此前单独报告的那条 Memory。

只读读取已存在的 execution_trace_entries 和 Gw request_logs/token_usage，使用 Host rowid 后缀及 operation/turn 索引、Gw timestamp 索引。没有制造入站、发送 QQ、增添模型调用、重跑 Work、修改 trace/config 或生产数据。原始诊断、完整请求和凭据仅保存在外部 private 目录。

实际 Host provider_response 152条，HTTP200为152/152；Gw对应窗口日志和新 raw stats各152条、Gw HTTP200为152/152，实报physical覆盖152/152且每条为1，合计152。客户端完整 usageMetadata 没有红删，152份保存的gzip/解压size/SHA在远端和本地二次验证通过。provider_start 保存141条，同operation provider_end保存152条；两者是诊断覆盖事实，不冒称141条都已完成全树互证。

| 核验项目 | 实际覆盖 |
|---|---:|
| 完整实际 HTTP prepared 请求与 Gw received 原请求树唯一匹配 | 128/152 |
| 上述128条原operation/context元数据与记录时序一致 | 128/128 |
| 上述128条实际客户端完整usageMetadata与Gw原native usage全树相同 | 128/128 |
| 其中由原模型诊断明确为chat_agent | 122条（117 Main、5 Work） |
| 这122条真实x-session-id与原conversation/generation/work一致 | 122/122 |
| 其中还保存原model_start ChatRequest.provider_session_id | 88条，声明与实际header 88/88一致 |
| 这128条返回x-session-id与x-antigravity-session-id均等于Gw日志session | 128/128 |
| 未完成完整请求树查证 | 24条（11缺prepared、13媒体日志红删） |

会话合同按原内部身份构造 `yuki:{conversation_id}:g{generation}:main` 或 `yuki:{conversation_id}:g{generation}:work:{work_id}`。122条实际 Gw 入站头与各自原 trace 的内部 conversation/generation/work 精确相等，不从平台ID、正文或token三项猜归属；其中88条再与已保存原ChatRequest声明核对。Gw出站session值由网关编码，核对的是实际返回头与日志session一致，没有把原入站Yuki字符串与网关编码后的值混为同一格式。

152份同父模型operation的已保存诊断明确145条chat_agent（140 Main、5 Work）及6条memory_extraction；另1条任务诊断未保存，原provider context有conversation/generation且work为空，任务类别仍未知。整体146条 Gw实际入站x-session-id存在、6条无头；未完成全树匹配的24条，不仅凭这些总数升级为逐调用身份验收。6条辅助Memory没有Main身份，不能代替Main验收。

13条无法全树匹配的差异已经逐路径定位：候选请求其余键、列表、标量与原prepared树相同，差异仅在1个或8个inlineData媒体项。Host现有诊断保存原媒体JSON的SHA/bytes引用，Gw现有 `src-tauri/src/proxy/payload_audit.rs` 把媒体data存为 `[base64 image: N bytes]`，没有保存原媒体hash。原媒体内容不可由这个长度占位重建，故严格完整树覆盖仍为128，不把可见部分一致称为媒体原值已验证，也不据此称实际HTTP被改写。12条记录时序内候选唯一，1条时序内另有Memory候选；时序线索均不单独作为全树通过依据。逐路径附录是外部 `prepared-request-divergence-redacted.json`。缺prepared的11条trace ID为 `196019, 196044, 196227, 196404, 196459, 196629, 196640, 196821, 197176, 197223, 197587`，媒体红删13条为 `196207, 196217, 196240, 196300, 196310, 196323, 196336, 196350, 196363, 196378, 196391, 196617, 197210`。

| 原始计量字段 | 实报覆盖 | 缺失条数 | 已实报量之和 |
|---|---:|---:|---:|
| 输入 | 152/152 | 0 | 5998485 |
| 输出 | 152/152 | 0 | 10209 |
| 上游实报 total | 152/152 | 0 | 6008694 |
| cache | 133/152 | 19 | 4190698 |
| thoughts/reasoning | 0/152 | 152 | 未知 |

全窗口152份客户端完整native usage子树的多集与Gw152份原native usage多集一致，包含字段存在性、完整cache项及其实际值；Raw与Log实报input/output/cache/reasoning/reported-total/account/model的多集也一致。这是整体多集事实，不能替代未完成的24条逐调用匹配。token_usage没有request_id，Gw consolidated没有responseId；因此不宣称共享request/receipt ID直接join、不按input/output/total猜一一对应，也不宣称整个wire字节保持相同。

cache已实报133/152；这133条input之和5,475,569、cache之和4,190,698，已知配对量比例 `76.534475230%`。其余19条cache未知，152条thoughts/reasoning均未返回、SQL NULL；不能把没有思考细分当实际0，也不把上游实报total重新相加覆盖。上述比例只描述有实报配对的请求量，不是完整账单覆盖，真实计费与节省金额仍未知。

Gw完整请求耗时范围1,984–21,179ms，中位3,919ms；已完整互证的122条chat耗时2,042–21,179ms，中位3,984.5ms。physical=1的实报覆盖152/152，不据此推断供应商缓存路由或后台计费策略。

捕获前后Host均是container `78b19c8017927ad193b588c68455781da7ce43f46145b64f53b42f83d5345b45` / image `sha256:87d001a4809ef8cafd538b0093c4cd858f1957e346a2a37ed0e643e81dfc62fe`，revision d75，healthy/restart0/OOM=false。Gw于03:47:51UTC仍是优化image `sha256:9e03cb48d80428ace810854b3b52886fc15a1770b31d57d66c270f2dc31a670d` / revision64 / beta.9，healthy、真实health HTTP200、restart0/OOM=false；Mihomo image/StartedAt/restart/health未变。Gw主机available892MiB、memory PSI avg10/60/300均0。最初冷启动未监听HTTP的空样本另存，不能与当前healthy状态混淆。

trace归档11,668,932B，SHA256 `728fcc9e939eb53db5cc74f648500af34be659d50e6d116d4d3575c9d242c2be`；283份原model_start/route诊断另归档10,463,983B，SHA256 `5d964b34fc449042c895aafcabb7813c089878832850da0c4989cbfb227d2b91`。该283是诊断span数，不能写成283次模型调用。完整压缩原件与解压保存size/SHA均已核验。

外部证据目录 `D:/codex-test-tmp/agm-beta9-newhost-main-acceptance-64b5311`，Host私有远端目录 `/tmp/agm-beta9-newhost-main-client-trace-20261013-QP68labg`。逐调用红删元数据/usage/哈希在 `newhost-main-natural-comparison-redacted.json`，概览在 `newhost-main-summary-redacted.json`，03:43附近索引在 `current-material-0343-time-index-redacted.json`。正文、签名、平台身份和认证信息未写入本报告。该报告不把后续31%/94%缓存调查或新compiler版本验收冒称已完成。

逐调用公开数据见 [152行脱敏CSV](current-material-beta9-newhost-main-calls-20261013.csv)。
