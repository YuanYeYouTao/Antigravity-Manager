# AGM fork 适配官方 v4.9.7-beta.8

## 当前状态

[PR #3](https://github.com/YuanYeYouTao/Antigravity-Manager/pull/3)已合入 fork 的 `beta` 预览分支，
适配源码为 `b2e57ec30c17c37d145a33443603b06943af6d0b`，生产已切换至 `4.9.7-beta.8`。
启动时间为 `2026-10-12T05:56:12.538016713Z`；本次检查 health 正常、Docker healthy、restart=0。
[CI 38209415109](https://github.com/YuanYeYouTao/Antigravity-Manager/actions/runs/38209415109)的7个job成功；
[Windows Installer 38209415047](https://github.com/YuanYeYouTao/Antigravity-Manager/actions/runs/38209415047)在本记录时仍pending。
未创建 tag/Release；真实 QQ 行为和缓存改善尚未验收。

## 基线与保留项

从生产来源 `aab8d1b6d544fd1cc2ab86dbc9fa79e8cfa079b5` 整合官方
[v4.9.7-beta.8](https://github.com/lbjlaq/Antigravity-Manager/releases/tag/v4.9.7-beta.8)，
官方源码为 `15f4bc938ecfa0286097ec6246a3d1583bc683a7`。
这是预览版；正式版 v4.9.6 不包含本次需要的全部续接修复，不将本次适配混入 `main`。
[旧占位修复记录](agm-protocol-placeholder-2026-10-04.md)仅作历史背景。

以官方为准，只保留具体错误修正和已确认的数据保留需求：

| 项目 | 本次处理 |
| --- | --- |
| 尾轮与空结构 | 采用官方取消 model 尾轮强加 user 的行为；空结构共用 beta.8 的 `.`，保留真实非空 `·`、`(no content)`。删除旧 append、纯 functionResponse 特判与废代码。 |
| auto-heal | 保留已有一次尝试，共用 `.`，删除无用状态及 Continue/task ready 文本。第二次流按官方原样透传，删除额外正文/FC 检查及合成空输出错误，不剥 STOP、usage、DONE；真实 HTTP、request、read 错误仍传播。 |
| Native Gemini 收集 | 保留官方错误传播及生产 grounding 修正；保留 grounding/citation、unknown metadata、signed part，完整行与 EOF 共用解析，删除默认 STOP。 |
| 模型与进站 | model_specs、Gemini wrapper 保持官方 beta.8；删除旧 Flash 默认提前 override。inbound 仅删除 6 倍 budget/max 意图猜测和空工具输出伪造 `Command executed successfully.`，更新原测试。 |
| 时序与发布 | 保留 monitor 的 x-timing 头下划线改连字符修正。Windows workflow 的普通分支 push 不发布 Release，预览 Release 不设为 latest。 |
| 官方新功能 | 保留 beta.8 统计、调度与配置功能，不扩大 fork 策略。 |

[Dockerfile.protocol-placeholder](../../Dockerfile.protocol-placeholder)仅替换后端，前端及运行依赖采用官方 beta.8 runtime。
固定多架构 digest 为 `sha256:e1a74221a6f2df268874e712eaff16edb0fc3447447bd0615b3769d8f3e5fd7d`，
linux/amd64 为 `sha256:f65cad3803a93f83d5bfc8ec4c4c6f204b2a5abd26893e89ec78119f45801283`。
来源见[官方 Docker 标签元数据](https://hub.docker.com/v2/repositories/lbjlaq/antigravity-manager/tags/v4.9.7-beta.8)；
源码 base label 同步至上述官方修订，`.dockerignore` 排除 `.cache`。
线上标签为 `antigravity-manager:beta8-b2e57ec3`，实际 image ID 为
`sha256:4c6439eead5e6d78617c2b66f7e8990384296bdcbcf993bddc7533d41ecf7f5d`。
线上源码标签及后端 SHA256 已与本次构建记录核对，不混用本地 Docker manifest descriptor 与服务器 image ID。

## 真实验证

| 项目 | 已确认结果与边界 |
| --- | --- |
| 官方来源 | tag/source SHA、Release 资产、preview updater 和双架构 Docker 元数据已核对；不等于生产验证。 |
| 最终 Rust 检查 | fmt、Clippy all-targets/all-features、85 项定向测试实际通过：6 specs、44 inbound、1 timing、12 defense、10 auto-heal、12 collector。记录在 `.cache/final-minimal-build.log`。 |
| 官方 inbound 局部对照 | 只替换 beta.8 原始 inbound，其余为本次源码；41 passed / 3 failed，记录在 `.cache/inbound-baseline-corrected.log`。这是局部失败复现，不是全仓官方验收。 |
| 前端 | `npm run build` 实际通过。 |
| 最终候选烟测 | 本地无账号、network none：health 版本 `4.9.7-beta.8`，`/` HTTP 200，admin stats 返回 JSON，restart=0。没有 Provider/QQ 调用。 |
| 最终 release | `cargo build --release --bin antigravity-tools --locked` 通过；线上后端 SHA256 为 `9f9015f22199431a4176a902ecdc5ab706a4462ed7db98b00ec38f8d8d4d8b4e`。 |
| 数据迁移与旧版读回 | 5 个数据库在无账号、断网副本上完成 beta.8 迁移及旧4.9.0读回；两版 stats query、quick_check 与已有表行数保持检查通过。 |
| 生产检查 | health、Docker健康、首页 HTTP 200、admin stats JSON通过；restart=0，Compose仅image变化，环境、挂载及Mihomo身份保持。 |
| 合成协议探测 | 三项均 HTTP 200、无error：FC 3.813s且signature保留；空FR 2.546s且marker返回；Native Google Search 14.948s且grounding为1 chunk、2 supports。 |

构建修正 Cargo 缓存路径为实际的 `/usr/local/cargo/registry`、`/usr/local/cargo/git`。
官方 inbound 对照借旧 grounding builder 的编译缓存，重新复制本次源码与锁文件，
不沿用旧源码或旧通过结果。旧 helper 只复用构建输入核对、真实 Compose 读取与单服务回退方法，
不沿用旧 4.9.0 runtime、image/hash 或硬编码验收状态。

构建、迁移、部署、线上检查和合成探测记录分别保存在 ignored `.cache/` 的
`build-manifest.json`、`rehearsal.json`、`deployment.json`、`live-health.json`、`protocol-probes.json`。
不含凭据的汇总证据已保存为 [agm-beta8-adaptation-20261012.json](evidence/agm-beta8-adaptation-20261012.json)。
合成探测不等于真实 QQ 验收；缓存交替的根因仍待观察，不声称已排除完整链路的签名副作用。
auto-heal 会产生额外物理上游请求，最后一份 usage 或一次客户端 HTTP 不能代表全部物理消耗。

## 部署与回退

已执行一次性 `.cache/rollout_agm_beta8.py`，从当前容器读取真实 Compose，仅切 AGM 的 image。
切换前已停写并保存完整当前私有数据备份，位于服务器 mode700 的私有 STAGE 内；没有恢复旧数据。
验收后已删除本次演练数据副本和上传镜像归档，保留切换前备份及原镜像供回退。
隔离副本复制数据库及现配置，两种认证改为测试凭据、无账号、network none；迁移新增 reasoning 字段，
完整性和已有行数检查通过。行数相等不代表所有内容不变。

启动实际配置变化仅涉及 `custom_mapping`、`experimental`、`stream_timeouts`。
官方移除 legacy `gemini-3.x-flash` 映射；现有其余映射、thinking与认证设置保持，账号文件身份保持。
回退使用原4.9.0镜像与当前数据，不恢复旧数据库、不重放会话或重发回执；旧镜像读回已在副本验证。
服务没有可证明的请求排空接口，重启不能称为无中断升级。

本次未修改 Yuki 或代理，未主动清空签名/思考缓存；私有备份与凭据不进入仓库或公开输出。
本适配遵循上游 [CC-BY-NC-SA-4.0 许可](https://github.com/lbjlaq/Antigravity-Manager/blob/15f4bc938ecfa0286097ec6246a3d1583bc683a7/LICENSE)。
