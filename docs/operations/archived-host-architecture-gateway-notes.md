# 从 Host 架构文档迁出的历史代理说明

归档日期：2026-10-04。来源为 Yuki Host 基线 `b3d325d`，摘取代理专属段落，未复制完整任务书。
以下内容保留原日期、部署入口快照与证据范围，仅作历史资料。原文中的“本轮必做”、
授权、修复和部署步骤属于当时 Host 任务，不能作为本 fork 当前实施或上线指令。
镜像、账户、路径、计量与协议能力均须以当前仓库及获准实际证据重新核对。
原 Host 文档已移除代理专属运维内容；下列原路径仅用于追溯出处。

## 2026-10-01 代理逐跳核查片段

来源：`docs/architecture/long-task-harness-compaction-taskbook.md`，原 §2.4
“Antigravity Manager 代理侧核查（本轮必做）”。以下为该历史段落摘录：

只读核查证据见 代理逐跳审计（2026-10-01）（原 Host 路径 `docs/operations/provider-compaction-audit-2026-10-01.md`）。真实同一 turn 的三请求已关联至代理 UUID 与最终发送点，并逐项核对 payload/usage；报告另列脱敏、原始 Google wire usage、实际容量认证和上线后自然缓存观察的证据边界。`countTokens` 可达但当前实测只数 contents，列表 `inputTokenLimit` 不能当完整实际请求容量认证；这些能力边界不冒充已完成在线容量/缓存验收。

用户已指定实施期间读取代理服务器数据，不能只凭 Yuki 侧日志归因缓存差距。2026-10-01 已从本机以 `ssh antigravity-server` 只读连通，当前容器为 `antigravity-manager`、镜像标签 `antigravity-manager:gemini-request-correlation-v4.8.4`，同机有 `mihomo-host`；当前挂载为宿主 `/opt/antigravity-manager/data` → 容器 `/root/.antigravity_tools`。这些是入口快照，采集前再次核对实际镜像 digest、容器、挂载与服务配置。历史补丁/回滚记录见 供应商切换记录（原 Host 路径 `docs/operations/provider-cutover-worklist-2026-09-29.md`）；不照搬历史 schema、版本或账户状态，也不重复引入已修复的缓存缺失/显式零混同。

采集复用现有代理请求日志/数据库和最终序列化审计，时间有界、查询分页，生产只读。核查以下事实并形成可复查的逐跳对账：

1. Yuki 实际 Profile endpoint → SSH tunnel → AGM → Google 的有效链路。确认模型映射、协议转换、重试及账号选择，不将配置文件的名义路由等同于最终请求。
2. Yuki 原 request/turn/execution ID、代理日志 UUID 与已有最终发送点关联摘要。优先沿现有 request correlation 对账；时间只能筛候选，不能作为唯一匹配依据。多次转发/重试逐项记录，不把一次 Yuki 逻辑请求当作仅一次上游请求。
3. 对照客户端输入与 AGM 最终发送结构：system、工具/schema 顺序、contents 顺序、thinking/signature、模型/请求设置，核查新增包装、默认字段、裁剪及跨轮变化。沿既有脱敏摘要与受控本地比较，不输出密钥、OAuth token 或私密聊天正文；比较不到完整上游字节时明确证据边界。
4. 对照 Google 原 usage、AGM 保存与返还 usage、Yuki model_invocations：input/cached/output/thinking/total 各字段是否准确映射，缺失是否曾转换成零、缓存字段是否被漏记、思考 token 是否重复合计。缓存率统一用 token 加权并披露各层计量覆盖率；缓存缺失不是自动命中或自动零。
5. 核查路由/账号切换、间隔、重试与命中变化的关联，以及代理支持的实际输入限额、countTokens/显式缓存端点能力。先查已有数据与实现，不为测缓存向群里发消息、不制造保温流量，不凭账号切换的相关性断言它必然造成缓存失效。

代理补丁属于发现真实转换/计量缺陷后的授权修复范围：按其独立源仓库/开发约束定向验证、版本化补丁与回滚记录，部署只替换 AGM 服务，保留 Mihomo 和 Yuki/SnowLuma；不将代理变更混入 Yuki 镜像发布。无必要缺陷则只交付审计结论。最终报告区分 harness 前缀变化、代理转换/计量问题、上游缓存行为和仍无法归因部分；上线后重复同口径自然流量观察，不承诺恢复某个固定命中率。

## 2026-10-01 结构输出微小能力探测

来源：`docs/architecture/conversation-rollup.md` 的结构摘要能力说明；历史结果不证明当前
长群史摘要质量、完整容量或缓存收益。

2026-10-01 的一次无 QQ、无工具微小能力请求通过当前 Gemini 3.8/AGM 路由接受
`responseMimeType`/`responseJsonSchema` 并返回可校验 JSON；它只确认结构输出能力，
不构成群史摘要质量、长窗口容量或缓存改善验收。
