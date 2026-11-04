# 混合分区 CLI v7 契约

更新时间：2026-11-04

用途：说明 QS4 Mixed 计划、边界和结果的新协议词汇及兼容限制。
读者：本地自动化脚本与正式消费者的维护者。
不包含：旧版协议、项目保存、公共网络 API 或后台取消；共用动作与错误见 [CLI 总参考](headless-cli.md)。

[QS4 首片](../topics/modeling/mixed-partition-execution.md)使用 `schema_version=7`，工程仍为 v4。继承既有查询 / 正式动作，新增 `radishflow.fixed-tp-mixed-material/1`：`acceptance_contract_version=1`、`temperature_k`、`pressure_pa` 与显式 `blocks`。每块有稳定 `id`、`members: IdentityRef[]`，`method.kind` 为 `local` 或 `equation_oriented`；EO 块还须 `equation_contract_version=1` 及 `scaling`。重复 / 空身份、重叠成员、未知字段、显式 null 和非法尺度不转成默认配置。

尺度为 `evaluated_fresh_total`，或覆盖确切变量 / 方程的 `explicit_equations`。变量仍为端口组分；新增 `boundary` 方程绑定包含 `stream`、`source`、`sink`、`component_id`，端口引用保留原单元和端口名。首片只支持固定比例正向 EO、Local 单单元及块间 DAG；不提供块内 QS1 / QS2、反算 / 采纳、跨块环或自动补分区。

`results(flow_plan)` 返回规范配置、计划修订、DAG 块序、原身份的 EO 符号布局及能力要求。布局标注 `numerical_solvability=not_proven`；不使用工程出口模板模拟上游已产出数据。`results(flow)` 的 Current 为 `execution_kind=mixed_material`，包含 `streams`、Local / EO 类型化 `blocks`、生产者 / 实际消耗者 `boundaries`、全局物料及真实 `costs`。Failed 的部分块只在 `diagnostic_evidence`，Stale / Missing 不提供当前数值。执行来源包含独立 Mixed capability 与实际 `core_method`。

旧 v1—v6 词汇和错误保持：v3 不支持外层 v4；v4—v6 对 Mixed 工程可查询只读元数据，execute 在步骤 / 资源读取前以 `read_only_project` 拒绝，隐藏 capability 不能绕过方法检查。确认前缀为 `headless-v7-confirm-v1:`。全部动作仅影响请求内存和 stdout；没有 Save、后台取消或持久化结果。

公开模板：[查询](../../examples/automation/query-mixed-material-v7.request.json)、[已有工程运行](../../examples/automation/run-mixed-material-v7.request.template.json)、[从基础建图运行](../../examples/automation/build-mixed-material-v7.request.template.json)。运行模板需填写已有有效缓存；工程字节变化后重新取得 SHA。真实进程已验证全目录条目与字节不变，详见 W45。
