# 从 Host 分离的历史网关运维记录

来源：Yuki Host `b3d325d` 的下列运维文档，提取日期 2026-10-04。
只保留与网关实现及诊断有关的摘录，不复制整份 Host 报告。

**历史材料，不是现行部署、回退或模型调用指令。**
原镜像、源版本、备份路径和瞬时健康状态不能推断当前服务；此次整理未重跑模型、
部署或生产核查。本记录不含凭据、请求正文或签名原值。

## 用量缺失与计数接口

原路径：`docs/operations/context-cost-simulation-2026-10-01.md`，历史日期 2026-10-01。

- 当时 AGM `request_logs` 最近 2000 条中，缓存字段 NULL 781 条、明确零 0 条；
  另取 `token_usage` 最近 2000 条无 NULL、零 810 条。两个表不是逐条同窗口，
  不能把后者全部认定为未命中。
- 当时公开源码的 `token_stats` 把 cached_tokens 保存为 NOT NULL 数值，统计层不表达
  unknown；这是源码参考，不冒称部署二进制的逐行证明。

原路径：`docs/operations/provider-compaction-audit-2026-10-01.md`，历史日期 2026-10-01。

- 当时 `:countTokens` 与 `/countTokens` 转发到 Cloud Code `v1internal:countTokens`；
  同请求测得加入/去除 system 都为 18383，generation input 为 20927，公共 API 的
  generateContentRequest 包装被内部接口拒绝。这只能说明该样本的计数覆盖不足。
- 统一模型目录的 inputTokenLimit/outputTokenLimit 常量、累计 session 恢复阈值都
  不是单请求真实容量认证；没有大请求边界探测。
- 当时 Gemini native 路径未见 cachedContents 创建/管理入口；cache_manager 名称
  不能证明该调用链采用显式缓存。

## 发送点诊断与历史补丁

原路径：`docs/operations/provider-cutover-worklist-2026-09-29.md`，历史日期 2026-09-29；
补充原路径：`docs/operations/provider-compaction-audit-2026-10-01.md`。

- 当时 v4.8.4 的非流式客户端请求由网关强制 SSE 再聚合；旧 collector 丢弃候选的
  groundingMetadata，修补按事件顺序保留查询、来源块和支持关系。
- 旧签名规则以首个非思考 part 为锚点，函数轮可能把签名移到普通 text；最小补丁
  改为当前轮首个 functionCall，并保留客户端 low。这是历史源码与同报文 A/B 证据，
  不能推断其他形态或当前版本行为。
- 历史补丁 `82741bb` / `359e475` 核对候选输出与 thoughts 合计，避免漏计思考；
  `ed47b91` 区分缺失缓存与显式零。后者真实当时样本只覆盖缓存缺失，正数/零的
  定向单测不能代替自然账单验收。
- `d316d27b026c68ed037acb2fa6bb79884f8a6643` 增加最终序列化点关联：域分隔
  SHA-256(`agm.proxy-log-id.v1\0` + UUID 二进制) 对齐日志行；诊断在
  `serde_json::to_vec` 后、HTTP POST 前，不是 Google 端抓包，不覆盖 HTTP 库隐藏重试。
  旧 Forwarded 预览位于最终 prepare 之前，可能简化或重排，不能当作最终发送字节。
- 上述补丁曾以 `gemini-request-correlation-v4.8.4` 等独立服务镜像部署并保留回退。
  这些名称只是历史出处，不提供可执行部署或回滚步骤。

## schema 方言核查

原路径：`docs/operations/work-context-cache-experiment-2026-10-03.md`，历史日期 2026-10-03。

本地官方源码 `6e8b982` 的 Gemini wrapper 克隆请求，未找到 responseJsonSchema 到
responseSchema 映射；OpenAI 路径则清洗 $defs/$ref 后写入 responseSchema。
这构成当时的方言兼容候选，没有最终上游 wire，不能断定哪个环节忽略 schema。

- [历史 Gemini wrapper](https://github.com/lbjlaq/Antigravity-Manager/blob/6e8b982aee7e3d3d53501825431142eea9a3f9ab/src-tauri/src/proxy/mappers/gemini/wrapper.rs#L40)
- [历史 OpenAI schema 映射](https://github.com/lbjlaq/Antigravity-Manager/blob/6e8b982aee7e3d3d53501825431142eea9a3f9ab/src-tauri/src/proxy/mappers/openai/request.rs#L1038)

## 模型目录容量

原路径：`docs/operations/work-compaction-capacity-2026-10-02.md`，历史日期 2026-10-02。

当时 AGM v4.9.0 目录声明该路由为 1000000 上下文。Host 记录明确没有独立认证上游
实际容量，不能把目录声明当作真实大请求验收或据此扩大模型窗口。
