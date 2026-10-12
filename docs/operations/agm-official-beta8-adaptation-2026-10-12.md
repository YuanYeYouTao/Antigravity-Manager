# AGM fork 适配官方 v4.9.7-beta.8

## 当前状态

终审已删除第二次 auto-heal 流的附加输出检查，最终 fmt、Clippy 和 85 项定向测试通过。
最终 locked release 编译及候选隔离烟测通过。
提交、推送、PR、CI、合并和生产部署仍待完成；目标为 fork 的 `beta` 预览分支。

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
最终 `SOURCE_REVISION`、`PATCH_SHA256`、image ID 和 binary SHA256 需对应本次实际构建输入。

## 真实验证

| 项目 | 已确认结果与边界 |
| --- | --- |
| 官方来源 | tag/source SHA、Release 资产、preview updater 和双架构 Docker 元数据已核对；不等于生产验证。 |
| 最终 Rust 检查 | fmt、Clippy all-targets/all-features、85 项定向测试实际通过：6 specs、44 inbound、1 timing、12 defense、10 auto-heal、12 collector。记录在 `.cache/final-minimal-build.log`。 |
| 官方 inbound 局部对照 | 只替换 beta.8 原始 inbound，其余为本次源码；41 passed / 3 failed，记录在 `.cache/inbound-baseline-corrected.log`。这是局部失败复现，不是全仓官方验收。 |
| 前端 | `npm run build` 实际通过。 |
| 最终候选烟测 | 本地无账号、network none：health 版本 `4.9.7-beta.8`，`/` HTTP 200，admin stats 返回 JSON，restart=0。没有 Provider/QQ 调用。 |
| 最终 release | `cargo build --release --bin antigravity-tools --locked` 通过，后端 SHA256 为 `9f9015f22199431a4176a902ecdc5ab706a4462ed7db98b00ec38f8d8d4d8b4e`。 |
| 数据与上线 | 隔离 schema 迁移、旧镜像读回、PR/CI/合并、生产切换和自然 QQ 观察待完成。 |

构建修正 Cargo 缓存路径为实际的 `/usr/local/cargo/registry`、`/usr/local/cargo/git`。
官方 inbound 对照借旧 grounding builder 的编译缓存，重新复制本次源码与锁文件，
不沿用旧源码或旧通过结果。旧 helper 只复用构建输入核对、真实 Compose 读取与单服务回退方法，
不沿用旧 4.9.0 runtime、image/hash 或硬编码验收状态。

缓存交替的根因与真实 QQ 效果仍待观察，不声称本次已排除完整链路的签名副作用。
auto-heal 会产生额外物理上游请求，最后一份 usage 或一次客户端 HTTP 不能代表全部物理消耗。

## 部署与回退

一次性 `.cache/rollout_agm_beta8.py` 已通过 Python 语法检查及静态复核，尚未上传或执行。
它从当前容器读取真实 Compose，仅覆盖 AGM 的 image；按实际环境、挂载和 peer 身份比较。
临时容器只按本次创建的 CID 清理，命令 timeout 为 60 秒，备份及 rehearsal 使用独立时间目录。
上传前由执行方建立 mode700 的服务器 STAGE；完整私有数据备份始终位于该私有父目录。
隔离迁移复制数据库及现配置，将两种认证改为测试凭据，无账号、network none；SQLite 完整性、schema 与行数需按实际结果记录。
行数相等不代表所有内容不变。

beta.8 启动会添加 reasoning 统计字段，也可能迁移旧 32k thinking 默认配置。
正式切换前需在隔离副本确认新镜像迁移和旧镜像可读，再停止 AGM 写入并备份完整当前数据，
包含必要 WAL/SHM；不在仓库或输出中记录账号、API Key 等凭据。
上线仅切 AGM 服务并核对旧/新 image、实际版本、health、restart 和同机 peer。
回退使用旧镜像与当前数据，不恢复旧数据库、不重放会话或重发回执。
服务没有可证明的请求排空接口，重启不能称为无中断升级。

本次不修改 Yuki、账号、路由或代理，不清空签名/思考缓存。
本适配遵循上游 [CC-BY-NC-SA-4.0 许可](https://github.com/lbjlaq/Antigravity-Manager/blob/15f4bc938ecfa0286097ec6246a3d1583bc683a7/LICENSE)。
