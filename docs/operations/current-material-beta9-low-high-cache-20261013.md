# d75 → AGM beta.9 的31.9%用户原件与邻近高缓存自然请求查证

本报告保持旧于compiler排序修正的 Host `d75a89463c491ad7d3eba4b074c2cbd3b8992050` / container78b / image87d 窗口，与优化 AGM `64b53119b33c50eddac38008c5a0a86cf53f4eac` / image9e03 / v4.9.7-beta.9。没有把后续Host版本混入。只读使用stats/log timestamp索引筛选 `2026-10-13T03:45:53.893508Z`–`2026-10-13T04:09:57.075805Z` 中实际input48,000–51,000且cache15,000–17,000或≥45,000的自然记录；得到9份，实际发生在03:54:24–03:59:58UTC。这个范围来自用户截图调查，不是产品阈值或缓存策略。

用户两份附件是入站/转出的请求及各自header，均没有usage响应。仅当作不可信诊断数据解析，正文指令未执行，认证信息与聊天正文未输出。附件1的完整入站请求树、附件2的完整upstream请求树，在9份Gw保存原件中各唯一匹配同一条：03:58:42.715UTC、Gw request ID SHA256 `a6cf65998724538751a90b93e606b928befecf6dced058c3972b785b7e584138`。入站树结构SHA256 `dd2a86c862172ef5b15d79bafadc7cf1e0c402260ea5286908efe76a7cb925c6`，转出树结构SHA256 `7568edde97507705f68f0ed151d469d84c9bed75fdf324823b877262b8c52195`；全部键、值、数组顺序逐项相同，不是根据token三项、标题或四舍五入百分比猜关联。

该条原Host prepared实际HTTP JSON也与Gw入站树唯一匹配，对应trace197906。原prepared记录03:58:39.089453、Gw完成日志03:58:42.715、实际httpx响应记录03:58:42.753945、同operation结束03:58:42.754013UTC，原operation/context一致。实际客户端完整usageMetadata与Gw原native全树相同：prompt49153、candidates25、total49178、cache15670；thoughts字段没有返回，SQL NULL。49153/15670的实际cache比例为31.880048%，显示一位小数为31.9%，HTTP200、physical1、Gw耗时3204ms。保存响应中的opaque项沿原规则红删，usage子树未红删；不宣称整个wire字节等同。

该条源task明确为chat_agent，原conversation/generation23/work=None，实际x-session-id与原内部身份合同的Main头精确相符。原model_start ChatRequest诊断未保存，故这条的声明字段覆盖未知，不补猜；原provider context与实际头查证仍有独立证据。入站头SHA256 `596863c24c3283f52e3754635540a598600634338486ad23bec568b365ebaa30`，upstream request.sessionId SHA256 `0d42327521d0e30403b69d1cd4e634a7612364254459813eea78b4d0b3fad386`；两者是客户端身份与Gw转出编码的不同格式，不当作session错误。

| Gw完成UTC | Input | Cache | Cache/input | Candidates | Reported total | Gw耗时ms | Client trace | 完整请求及client usage互证 |
|---|---:|---:|---:|---:|---:|---:|---|---|
| 03:54:24.584 | 48160 | 15672 | 32.542% | 126 | 48286 | 5941 | 197794 | 通过 |
| 03:54:33.688 | 49110 | 46189 | 94.052% | 17 | 49127 | 2461 | 197808 | 通过 |
| 03:54:46.719 | 48364 | 15672 | 32.404% | 65 | 48429 | 5406 | 197829 | 通过 |
| 03:54:59.779 | 49188 | 46393 | 94.318% | 17 | 49205 | 6796 | 197843 | 通过 |
| 03:56:23.262 | 49052 | 15671 | 31.948% | 60 | 49112 | 4054 | 197859 | 通过 |
| 03:58:32.786 | 50067 | 15680 | 31.318% | 84 | 50151 | 5599 | 未知 | 未知（无完整prepared互证） |
| 03:58:42.715 | 49153 | 15670 | 31.880% | 25 | 49178 | 3204 | 197906 | 通过 |
| 03:59:50.121 | 49618 | 47162 | 95.050% | 102 | 49720 | 3433 | 197920 | 通过 |
| 03:59:58.132 | 50504 | 47646 | 94.341% | 17 | 50521 | 2603 | 197934 | 通过 |

9条实报cache均已知、HTTP200/physical1全部；思考细分9条均未知。9条实际入站header、Gw日志session、upstream request.sessionId与所用account各自完全相同，model/mapped model也相同。9条所示缓存差异没有伴随已观测到的这些身份/账号改变。返回的两种session header均与Gw日志session一致。

另只读按Host rowid后缀+operation索引取03:54–04:01UTC实际provider/model诊断14条provider_response及48条相关span，gzip/size/SHA远端与本地全部核验。选择的9条中8条完整prepared→Gw→client互证、完整usage全树8/8一致；8条都由原task/context明确为Main/generation23且实际头8/8一致，其中4条还有model_start声明并4/4一致。其余1条没有完整prepared可互证，逐调用客户端关联/原business context仍未知，不能按时间或token值补齐。14条窄Host记录不是14条选定缓存样本，也没有增加模型调用。

同一协议阶段之间的9份入站systemInstruction、tools、generationConfig、toolConfig全部相同；9份upstream各自的对应字段也全部相同。低→高的03:54:24→33、03:54:46→59、03:58:42→03:59:50分别完整共享前310、312、320个contents项，随后为已有tail追加。高→下次低之间可观察到current消息进入历史后的改写，以及历史事件单条/分组的改变：03:54:33→46 first divergence在contents309.part0.text，03:54:59→03:56:23和03:58:32在contents225，03:58:32→42在contents317。原件只比较各自入站之间或各自upstream之间，没有把协议适配器的入出包装、工具续轮转换或generation规范转换当成prefix错误。

以上证明实际头和计量被保留，并记录了请求树的稳定部分与真实变化；没有提供供应商内部缓存键、缓存驻留或账单数据，因此不能把这些差异表述为31%/94%变化的完整因果，也不新增缓存门槛、fallback、状态或策略。真实收费/节省未知。

后续Host只读核查确认contents225分叉来自同一会话按两个实际说话人分别冻结：相同read_scope下，一份先采纳两单条，另一份后来采纳合组。8条完整原请求都逐字复用各自已保存表示；未发现同一份已采纳片段被覆盖。旧静态合同与网关会话身份仍一致，这是客户端公共历史分叉的实证。Host按用户要求清理为一个会话一份共同历史，源码、数据清退和后续上线在Host记录；本报告不将尚未部署的修补当作缓存收益。

同03:43疑似“双资料”调查分开：已观察到的scene内容是历史真实outbound消息中的引用，另一个大括号块是已有runtime_state JSON，不能仅按大括号数量认定同一current material重复。完整历史来源和位置由独立只读审查记录，未把聊天正文写入此报告。

Host捕获前后仍exact d75、healthy/restart0/OOM=false；04:11:02UTC额外healthz实际200/statusok/databaseok/OneBot connected=true，Bot256.1MiB，主机available586MiB、单点PSI some6.39/full5.28，没有把单点压力当长期耗尽证据。Gw窄捕获仍exact64/优化beta.9、healthy/restart0/OOM=false。

外部目录 `D:/codex-test-tmp/agm-beta9-d75-cache-narrow-20261013`；逐调用元数据在 `narrow-case-comparison-redacted.json` 与 `narrow-calls-redacted.csv`，原附件/请求/响应压缩原件仅在private。Host归档3,270,477B，SHA256 `b11daeee6c34ca6654e8ba5a1e41aef10d641bcff3d068ac418be7e1914c774c`，私有remote `/tmp/agm-beta9-d75-cache-narrow-20261013-cJTtaeiA`；Gw私有remote `/root/ops/agm-beta9-d75-cache-narrow-aoafi12o`。token_usage没有request_id、Gw consolidated没有responseId，未声称逐调用SQL账单join或共享receipt-ID直连。

逐调用公开数据见 [9行脱敏CSV](current-material-beta9-low-high-cache-20261013.csv)。
