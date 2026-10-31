# QS2-C：联立模块工程与正式消费者（D06）

更新时间：2026-10-31

用途：定义 QS2 的方法 / 能力版本、工程保存、冻结运行、结果与 GUI / CLI 接入方案和批准边界。
读者：负责 store、应用计划、运行所有者、结果查询、Studio 和本地自动化的协作者。
不包含：新增数值算法或物理范围、结果历史持久化、跨进程任务服务、FFI / COM 扩展、依赖升级与发布。

- 状态：**2026-10-31 所有者批准 D06-01—06；C-A 存储配置、C-B 运行结果与 CLI v5 已实现。** GUI 局部画板尚未评审，仍未接入 QS2 配置 / 运行 / 详情；桌面打开 QS2 工程保持整份只读。正式门禁通过 1,976 项；记录见文末及 [W44](../../devlogs/2026-10/2026-W44.md)。
- 父专题：[QS2 核心](simultaneous-modular-block.md)拥有 D01—D05、数值门槛、独立基准及故障矩阵；[QS1-C](recycle-project-and-consumers.md)拥有已实现的 v4、独立计划与双资格基础。
- 本篇长期承载 D06 的跨层增量，避免将完整消费者契约继续堆入核心专题。下文为已批准的契约；共享链路与 CLI 已落地，GUI 仍须先完成局部画板评审。

## 实施前代码核对与复用边界

| 位置 | `8c18f223` 的事实 | D06 必需增量 |
| --- | --- | --- |
| [核心](../../../crates/rf-solver/src/simultaneous/mod.rs) / [装配](../../../crates/rf-solver/src/simultaneous/assembly.rs) | `ModularOptions` 只有固定 T/P 和可选完整尺度映射；`assemble` 自动识别内部连接；`solve_controlled` 内部装配并拥有 `ModularPlan` | 用正式配置映射已有选项；不引入 tear、初值或用户迭代预算，不要求新增公开 execute-plan API |
| [v4 codec](../../../crates/rf-store/src/project_v4/mod.rs) / [计划 DTO](../../../crates/rf-store/src/project_v4/plan.rs) | 只有 Acyclic / Partitioned；能力探测早于严格领域解析，未知能力整份只读；writer 推导能力并与内容核对 | 新方法 / 能力及严格 DTO；保留原两种方法和旧外层守卫 |
| [v4 写入](../../../crates/rf-store/src/project_v4/file_io.rs) / [生命周期](../../../apps/radishflow-studio/src/document_lifecycle_driver/flow_plan.rs) | 已比较现有目标的能力增量、保护目标原字节；来源只检查外层版本。来源为 v4、另存到不存在的目标时，没有来源能力增量比较 | 增加来源已保存能力基线的兼容提示，覆盖同外层升级和 Save As；沿现有 staged write |
| [正式计划](../../../crates/rf-ui/src/flow_plan.rs) / [应用转换](../../../apps/radishflow-studio/src/flow_plan_actions.rs) | 独立 generation、修订、历史、草稿及 saved baseline；配置形状校验与图就绪校验分开；引用 ID 保留目前只处理 tear | 新 Simultaneous 配置 / 草稿与尺度绑定；继续复用正式动作和 ID 保留 |
| [运行桥](../../../apps/radishflow-studio/src/solver_bridge/execution.rs) / [worker](../../../apps/radishflow-studio/src/workspace_run_owner/worker.rs) | job 已冻结图 / 配置 / 选包请求，加载包后 Ready 授权，再分派；两种控制与载荷；一次性回执、取消和 join 已存在 | 新 ModularControl / progress / success / failure / cancel 分支；保留一份 worker 名额 |
| [资格](../../../crates/rf-ui/src/flow_run.rs) / [结果](../../../crates/rf-ui/src/flow_results.rs) / [来源](../../../crates/rf-ui/src/result_query.rs) | 输入＋计划资格已贯穿接收和读取；成功载荷两类，失败证据只有 partitioned；方法摘要为已存 DTO 的确定性编码 | 独立联立结果 / 失败证据与实际版本；扩展所有载荷匹配和输出守卫，不能只接成功分支 |
| [规格检查](../../../apps/radishflow-studio/src/modeling_specifications/checks.rs) | 拓扑检查只分派旧两种计划，循环压力 AwaitingExecution 仅识别 Partitioned | 新方法走 QS2 装配；继续区分静态就绪与实际包 / 热状态验证 |
| [CLI 加载](../../../apps/radishflow-studio/src/headless/v3/mod.rs) / [v4 DTO](../../../apps/radishflow-studio/src/headless/v4/protocol.rs) / [结果投影](../../../apps/radishflow-studio/src/headless/v4/results.rs) | v3 拒绝工程 v4；v4 只给共享 parser 设置外层最大值 4，词汇和结果仅有原两种方法 | 新协议独立 DTO；旧 v4 增加固定能力边界，不能随共享 parser 支持 QS2 而扩大语义 |

已具备的 Splitter 字段、ConnectExact、身份分配、单 worker、双资格与结果查询均复用。`rf-model` 不增加 solver 配置；store 不依赖 solver / UI，应用转换连接 wire DTO 与 typed options。只在同语义处复用转换、单位、局部 I/O 和计数；不创建统一求解器大 DTO，不重做 QS1-C。

## D06-01：方法、能力与正式配置

### 版本职责

| 层次 | 已确认值 | 语义 |
| --- | --- | --- |
| 工程外层 | `schemaVersion=4`，document metadata 版本仍为 1 | 复用现有能力保护；本片不需要新工程外层 |
| 正式计划 method | `radishflow.fixed-tp-affine-modular-lu / 1` | 明确限固定温压仿射联立模块和 LU，不借 `partitioned-sm / 1` 或泛称通用非线性 |
| 必需能力 | 新增 `radishflow.solver.fixed-tp-affine-modular-lu / 1` | 选择本方法即要求；同时保留 `radishflow.flow-plan / 1`，图含 Splitter 时要求原 `radishflow.unit.splitter / 1` |
| 验收 / 响应 | `acceptanceContractVersion=1`、`responseContractVersion=1` | 分别锁定核心验收规则和固定温压组分仿射响应契约；不是可调精度 |
| 核心实际方法 | 已存在的 `rf-solver.fixed-tp-affine-modular-lu/1` | 结果来源记录核心返回事实；与工程方法显式映射，不修改旧常量伪装统一命名 |
| CLI 协议 | 新 `schema_version=5` | 与工程外层独立编号；旧 v4 请求 / 响应词汇保持 |

新能力的 version 1 同时要求上述方法、响应与验收版本；任一未知正版本走不支持 / 只读，不回退到最新支持版本。已知方法的缺字段、null、非法版本类型为格式错误；合法未知版本属于不支持能力。未来数值阈值或响应有效域变化须重新审定版本，不悄改 version 1。旧方法的版本、编码及摘要规则保持。

### 保存内容与尺度

`StoredFlowPlan` 与 `FlowPlanConfiguration` 各增加 `Simultaneous` 变体；应用使用现有 `ModularOptions` 表达数值设置，版本由受支持的类型 / codec 控制。正式计划只保存 method、两个契约版本、`fixedBoundary` 与必填 `scaling`：

```json
{
  "method": {"id": "radishflow.fixed-tp-affine-modular-lu", "version": 1},
  "acceptanceContractVersion": 1,
  "responseContractVersion": 1,
  "fixedBoundary": {"temperatureK": 300.0, "pressurePa": 650000.0},
  "scaling": {"kind": "evaluated_fresh_total"}
}
```

这是当前 simulation.steadyStatePlan 的合法片段；完整案例见[公开工程](../../../examples/flowsheets/fixed-tp-cross-recycle-binary-hydrocarbon.rfproj.json)。固定边界为正有限 SI；α / β 仍只在物理 Splitter 参数，配置不替用户修改 Feed 或其他物理规格。

- 默认 `evaluated_fresh_total` 映射 `scales=None`。内部连接和完整登记组分由图自动装配；每块变量 / 残差尺度在本次真实外部边界求出后冻结，保存规则而非上次算出的流量。
- 显式 `scaling={kind:"explicit_couplings",values:[...]}` 映射完整尺度表。每项严格包含 `streamId`、`source:{unitId,port}`、`sink:{unitId,port}`、`componentId`、`variableMolS`、`residualMolS`；后两者为正有限 mol/s，分别对应 `CouplingScale` 的变量与残差尺度。不得合并两者或只保存裸向量索引。
- writer 按核心 `CouplingBinding` 同义的流股 / 源设备端口 / 汇设备端口 / 组分键排序；先拒绝重复项，再构造 map，不能因 collect 覆盖重复键。数组顺序不改变正式值 / 摘要；零值、负值、非有限值、空身份、未知字段和错误类型明确拒绝。
- 显式表必须恰好覆盖**全图所有联立块**内部连接的全部登记组分，缺项、多项、错端口、断开或组分变更导致未就绪；不自动补尺度或只取交集。格式合法但绑定未完成可 Apply / 保存，重开仍显示未就绪。空图、超容量和不支持拓扑也属于静态能力 / 就绪问题，不把可编辑工程变成损坏文件。
- 默认模式没有人工耦合列表；显式模式中的绑定只定义尺度，不选择切边或强行改变装配。可从当前合法装配生成待审草稿，必须用户 Apply，不能将自动发现结果暗中写入正式配置。
- 保留显式表引用的 UnitId / StreamId，包括失效引用及重开；删除 / 新建、改名、Undo / Redo 不按名称或复用 ID 意外重绑。ComponentId 按工程登记身份核验，删除 / 替换组分不得隐式重解释旧尺度。

不保存 anchor 初值、tear、外层迭代预算、可调 LU / rcond 阈值、数值矩阵、派生 `ModularPlan`、包对象、试算状态、运行 token 或结果。全零锚点 / 一次解 / 独立复核、128 未知量容量和数值门槛全部沿 D01—D05；默认模式对物理图的修改重装配，不代表自动切换算法。

## D06-02：保存、旧读者与同版本兼容保护

| 路径 | 已实现行为 |
| --- | --- |
| v1—v3 旧工程、v4 原两种计划 | 原 reader / writer 与缺省 Acyclic 保持；打开不脏改、不自动升级 / 选 QS2 |
| 旧来源选择 QS2 | 下一次保存要求 v4 及新能力，明确升级 / 另存 / 取消；无结果持久化，重开 Missing |
| v4 原计划改 QS2 | 提示“新增联立模块能力，旧版本仅能只读检查”；明确是能力升级而非工程 v5 |
| 已有 QS2 工程再次保存 | 无新增所需能力时不重复提示；切回旧方法重新推导能力，保持外层 v4，不以残留方法标记冒充兼容 |
| 旧 QS1-C v4 reader / writer | 新 capability 在领域解析前形成整份只读；未知 method 是第二道保护；不能运行或覆盖，不吞掉 simulation |
| 更早 v1—v3 reader / writer、现有 FFI / COM | 按旧外层 / 模型守卫拒绝；不新增联立块 ABI，不修改旧输出 |
| 新 reader 遇未知能力 / method / 正契约版本 | 整份原字节只读，保留安全元数据和原因；不建立部分可编辑 AppState |
| 已知能力但声明缺失 / 多余、字段非法、重复键 | 格式错误；writer 根据正式内容推导精确能力集合，不能由调用者遗漏标记 |
| 未知外层或目标只读 | 原拒绝和禁止覆盖保持；Save As 也独立检查目标 |

生命周期增加**已加载 / 已成功保存的能力基线**（可由正式 saved 输入推导，不能使用当前草稿或当前计划代替）。同时比较来源基线与现有目标能力，捕获具体新能力 / 版本并提示：来源为旧 v4 且另存到空路径也不能漏提示；若来源本来支持 QS2，但覆盖较旧 v4 目标，仍提示目标能力增量。

保存准备沿既有正式输入、计划 generation、草稿保护、工程区 / 呈现捕获及目标 SHA-256，加入确切来源 / 待写能力集合。确认期间改计划、改物理图、换工程、目标改盘或能力变化须重新准备；一次确认不能批准另一份内容。store 独立保护目标并做严格内容校验，UI 提示不能替代底层守卫。继续 staged write / 原失败恢复，不宣称新增跨进程文件锁。

取消、只读、IO 或目标冲突失败不改变路径、source version、document / engineering / presentation / simulation 保存基线与草稿；只在写成功后一并更新来源能力基线。保留设备任务、采纳固定证据、工程基础、显示单位以及缺省 / 空区语义。未提交草稿不隐式 Apply；保存正式计划不清草稿。计划结果、历史、资格均不进文件或 sidecar。

## D06-03：就绪性、冻结与后台接收

### 三段校验与实际冻结

1. **正式配置校验：** 无包 IO、无求解；检查 DTO / 数值域 / 重复绑定，沿已有 `apply_flow_plan` 事务、独立历史、修订和 `simulation_dirty`。不同正式值产生新 generation，相同规范值为 no-op；方法切换不抹掉尚未处理的编辑草稿。
2. **静态就绪 / 报告：** `validate_for`、规格检查、动作发现与 Run Panel 统一分派到 `simultaneous::assemble`，核对身份 / 拓扑 / 物理规格 / 完整尺度 / 容量与响应声明。报告和建议捕获输入＋计划资格；不重复实现 QS2 能力表。循环压力继续 AwaitingExecution，不从工程模板证明已求温压；静态可装配不等于包有效、外部边界已求或数值可解。
3. **冻结运行：** Selected 期望经正式入口校验后，`FrozenFlowRun` / job 捕获物理图、完整正式配置和实际选包请求；沿现有授权加载一次包，Ready 时再核验会话、输入、计划及受理状态。仅获准才以捕获图 / 选项调用 `solve_controlled`；核心内部装配并独占 `ModularPlan`、响应、完整组分 / 绑定和尺度。外部进料执行后冻结默认尺度及零流来源；不能提前从模板或上次结果补齐。

“冻结计划”包含受理时不可变的正式设置与 worker 中由它生成的数值计划；不要求主线程预先求出依赖实际包的边界，也不让 worker 回读当前 AppState。静态预览产生的装配不是下一次 Run 的权威计划。已加载包 provenance、实际方法 / 响应 / 验收契约归本次结果来源；加载失败时未获得的包来源仍缺失。

默认仍 Acyclic；QS2 无循环时按该方法的已有核心边界执行并报告真实无环块，不自动改称 Acyclic 或强行制造耦合报告。缺响应、拓扑不支持、数值失败或热服务失败均明确拒绝，不切换 QS1 / QS0 或其他包。LegacyAcyclic 构造器继续拒绝非 Acyclic 正式计划。

### 双资格与单 worker

复用 `FlowExecutionQualification` 的输入＋计划 generation；不把计划塞入 `FlowRunQualification::same_input`，不另建资格系统。

| 事件 | Flow 接收 / 查询 | 其他消费者 |
| --- | --- | --- |
| 仅 Apply / Undo / Redo / Restore 改计划 | 已接受结果立即 Stale(plan_changed)，撤销旧回执并请求旧 Flow 取消；A→B→A 不复活 | 物理 revision、FlowEditContext、QS0 / 设备原输入资格不变 |
| 物理输入或选包变化 | Stale(input_changed)，在途成功 / 失败 / 取消均拒绝发布；下一次重装配 | 沿既有物理编辑规则失效 |
| 仅草稿 / 显示单位变化 | 不换正式 generation；保留已有草稿门禁和单位呈现语义 | 不隐式应用设置或取消无关任务 |
| 同 ID / 修订重开、换会话或工作区 | 新会话 Missing，旧回执不能发布 | 文件不恢复运行身份 |
| 当前任务真正成功 / 失败 | 成功且载荷与冻结方法匹配才 Current；失败为 Failed 及诊断，无正式部分流股 | GUI / CLI / 变量 / 输出使用同一查询 |
| 用户取消且资格未变 | 取消不创建新成功或 Failed，既有合格结果按 AF3 保留 | 终态事实与结果资格分开 |
| 迟到 / 重复 / 错方法载荷 | 一次性回执与类型匹配共同保护；旧终态不修改当前日志、失败态或来源 | 当前回执错载荷为基础设施错误，不当作物理解 |

`WorkspaceSolveProgress` 增加 `Simultaneous(ModularProgress)`；bridge / WorkerControl 适配 `ModularControl`，终态独立区分 Modular succeeded / failed / cancelled。进度显示真实阶段、块 / 单元 / 列，不复用 QS1 iteration 或伪造百分比。进度仍是可替换槽，关键失败证据必须随终态携带，不能靠最后一条进度重建。

继续 Flow / QS0 共用一个 worker；Busy、重复启动、Ready 授权、取消唤醒、panic / 断连和 join 沿现有所有者。改 Flow 计划不取消在途 QS0；取消或资格撤销后仍占槽直到实际线程退出，Active 才按既有 PlanChanged 调度下一次，Hold 等手动运行。核心已有装配前后、模块调用前后、响应 / LU 列、条件 RHS、最终复核检查点；不承诺中断内部 thermo，也不新增暂停 / 后台 CLI 服务。

## D06-04：真实结果、失败证据和来源

新增应用 `FlowResultRecord::Simultaneous` 和对应快照，复用公共 header / 流股 / 局部 I/O 查询，持有真实 `ModularResult`。失败记录以互斥 typed 载荷区分 Partitioned 与 Simultaneous（允许无核心证据的准备失败），保留现有 QS1 查询行为；不能同时挂两个可选算法证据并让消费者猜测。

| 证据 | 必须保留与投影 |
| --- | --- |
| 正式成功 | 最终生产者 streams、稳定块顺序 / members；无环块没有 coupling，联立块保留真实最终 `consumed / produced` |
| 连接与响应 | 每项完整 CouplingBinding、假设 / 生产者组分流量、SI / 缩放残差、两种固定尺度、固定 T/P、响应版本 / 输入输出域 / 无量纲系数、零流新鲜来源和锚点残差 |
| 线性证据 | dimension、原矩阵范数、主元阈值、真实换行序列、rcond_indicator、原矩阵 backward_error；不报告未实现的精确 rank / 误差界 |
| 成本 | 实际构造、各单元 / thermo / flash 调用、锚点 / 响应 / 分解 / 三角求解 / 最终复核次数，以及已测各阶段耗时；无虚构外层迭代 |
| 当前失败 | ModularFailureKind、结构化 error code / 上下文、progress 的阶段 / 块 / 单元 / 列、completed_blocks、active_units、active_coupling 和已发生成本，均属 diagnostic_evidence |
| 未完成计算 | `linear`、rcond 或 backward_error 未取得时明确 null / 未计算，不补 0，不将失败前完成块提升为当前成功 |
| 来源 | 冻结正式 method / capability / 两个契约版本、核心实际 METHOD、规范配置 SHA-256、本次已加载包证据；实际构建 / commit 不可得则未记录 |

失败中的**未计算**与**已计算但非有限**须区分：[dense 后端](../../../crates/rf-solver/src/simultaneous/dense.rs)在有限性拒绝前已把候选 rcond / backward_error 写入证据，异常算术可能留下 `Some(NaN / Infinity)`。不得让 JSON 序列化把它与 None 一并静默转成 null。QS2 诊断数值 wire 定为有限 JSON number、未取得的 null、或 `{ "non_finite": "nan" | "positive_infinity" | "negative_infinity" }` 三态；该 tagged 对象仅用于失败 / 取消证据，适用于其中的局部 I/O、残差、尺度、响应与线性数值。GUI / 文本显示“未计算”或明确非有限类别，保留原错误上下文；正式成功数值只允许有限 number。不改旧 v4 / QS1 DTO，也不因此修补数值结果或放宽核心验收。

规范摘要使用确定性 typed 编码，SI 数值、完整身份和显式尺度稳定排序；不纳入耗时、显示文案、草稿或执行 token。摘要用于复现，不替代不可复用 generation。新方法摘要不得混入旧方法摘要规则；缺省尺度的数值在运行证据中记录，不伪造为保存设置。规范 SI 零的编码统一，配置比较与摘要一致。

计划请求与已执行事实分开：加载 / Ready 前失败可以记录冻结请求配置，但不得宣称核心已执行或响应已求；核心调用后才附实际方法事实，获得哪些块 / 线性量就记录哪些。当前包下拉框和当前分支不能补填旧来源。

成功只在整次数值与独立复核通过、双资格仍有效后发布。晚块 / 下游失败时，上游块仅供诊断；失败查询正式 fields 无旧值或部分值。温度、焓和相字段如实来自局部输出，并明确验收仅覆盖固定 T/P 物料边界，不声称闭合通用能量 / 相平衡。

取消证据跟随本次完成对象 / 既有受理终态生命周期可检查，原成功结果资格不被覆盖；不借取消建立新 Failed 或结果历史库。被拒绝的迟到完成只保留执行对象事实，不回填当前工作区诊断。包加载错误、线程基础设施错误保持原分类，不伪造 ModularFailure 或求解成本。

变量 / Inspector / 表格使用生产者实际值；块详情保留消费者假设与实际出口的差异，不覆写为零残差。文本复制 / 导出复用捕获和写前校验，增加载荷类型、计划资格与新证据投影；改计划、重算、换会话或覆盖等待期间变化均拒绝旧捕获。Stale / Missing / Failed 不能通过另一入口泄露 Current fields；旧 steps、QS1 tears / iteration_evaluations、QS0 结果及设备采纳语义保持。

## D06-05：GUI 契约与局部画板门禁

继续在现有 Run / F5 使用已应用的活动计划，新增“固定温压仿射联立模块”选项并显示有限能力说明；不新增独立 Solve 按钮或自动算法。复用 Splitter 创建 / 比例编辑、ConnectExact 和正式检查建立主案例，不在计划面板复制物理 α / β。

计划编辑只提供固定 T/P、默认 / 显式尺度，默认自动发现块和耦合边为只读预览；显式表按连接 / ComponentId 编辑两个 mol/s 尺度。支持既有数值原文、输入单位、未完成指数、Apply / 恢复 / Undo / Redo；绑定失效保留原文并定位，不代选新对象。无 tear / 初值 / 500 次预算控件。Apply 和 Run 对草稿沿原规则处理，不暗中运行未应用值。

结果沿 Current / Stale / Missing / Failed 与独立运行状态显示；联立块详情展示实际 I/O、连接残差、尺度、响应、LU / 条件 / 后向误差和真实成本。失败与取消显示已执行阶段和诊断，取消请求后明确仍在等待退出。所有数值遵循现有量纲 / 呈现转换；dimensionless 指标不附流量单位。来源与复制 / 导出复用 D06-04。

批准交互方向后，在原 `.pen` 保留 QS0 / QS1 画板并新增局部评审，至少覆盖：

| 画板 | 必须审阅的内容 |
| --- | --- |
| 配置 / 自动耦合预览 | 四成员、八未知量，默认尺度与固定边界、显式尺度入口；原 Run / QS0 区仍可理解 |
| Current 详情 | 假设与生产者两列、两种尺度、rcond / backward_error、锚点 / 分解 / 复核成本，无虚构迭代 |
| 400 px 中英草稿 / Stale | 长身份与横向信息布局、单位、未完成输入、失效绑定、键盘焦点及 Undo 范围 |
| 失败 / 取消 | 包失败、近奇异 / 最终复核失败、未计算指标、诊断部分块与正式结果分离、等待 join |
| 同版本能力升级 / 只读 | v4 新能力、Save As 提示、取消默认焦点、旧 reader 只读原因与禁止覆盖 |

**本轮没有生成或认可这些画板。D06 批准不自动等于画板批准；局部画板经所有者认可后才接 GUI。** 自动帧不替代原生窗口，实窗先告知；Windows / Linux 未验时保留缺口。

## D06-06：CLI v5 与旧消费者隔离

独立 query / execute `schema_version=5`，请求及响应均声明 5；工程仍为 v4。理由是旧 v4 的方法枚举、结果和来源形状严格限定 QS1，直接扩展会使旧客户端收到无法解释的载荷。共享排序、引用、应用动作、授权、资源及错误框架，不复制整套 v4 执行引擎，也不新增 CLI crate。

### 入口兼容矩阵

| 消费者 | 允许与拒绝 |
| --- | --- |
| CLI v1 / v2 / v3 | 既有工程版本、词汇和 golden 保持；仍在资源读取 / 动作前拒绝工程 v4 |
| CLI v4（新程序中的旧协议） | 保留原已知工程与两种方法；QS2 v4 文件按协议固定能力清单整份只读，安全元数据查询可用，执行 / 新计划动作明确 UnsupportedCapability 或既有 ReadOnlyProject；拒绝发生在资源读取和任一步骤前 |
| 历史 QS1-C v4 可执行程序 | 原能力探测预期整份只读；需独立程序验证，不能用新程序 v4 模式代替历史证据 |
| CLI v5 | 可读 v1—v4 已知能力，支持三种计划；未知能力整份只读，未知外层拒绝；只读只给安全元数据 / 原因 |
| 同步旧接口 / FFI / COM | LegacyAcyclic 及原格式 / DTO 保持，不接新方法或结果 |

v4 的能力限制必须在**共享 parser 创建可编辑工作区之前**执行，并核对 method / contract，不能仅看外层 <=4，也不能让查询流程对新变体 panic。能力清单缺失 / 伪造不能绕过方法检查；已知形状损坏仍格式错误。旧 v4 动作发现不列 QS2；v4 的配置请求直接拒绝新方法和新字段。

### v5 增量契约

1. `flow_plan` 查询保留正式配置、`plan_revision`、simulation_dirty、静态就绪及原因，新增本方法 / 能力 / 契约版本与耦合预览。预览不读取包或求解；按输入＋计划 qualification 失效。
2. Plan 新严格枚举 `method:"radishflow.fixed-tp-affine-modular-lu/1"`，必填 `acceptance_contract_version`、`response_contract_version`、`temperature_k`、`pressure_pa`、`scaling`。scaling 为 `{kind:"evaluated_fresh_total"}` 或 `{kind:"explicit_couplings",values:[...]}`；每项 `stream` / `source` / `sink` 使用现有 IdentityRef / Port，另含 `component_id`、`variable_mol_s`、`residual_mol_s`。身份引用可指前序步骤，禁止前向引用 / 类型错配；重复绑定在解析转换时拒绝。
3. `configure_flow_plan` 原子提交整份配置，继续必填 `expected_plan_revision`；`run_flow` 同时必填 `expected_revision` 与 `expected_plan_revision`，支持前序 receipt 引用。旧两种方法在 v5 保留其原输入含义，不把 QS1 budget / tears 接受为 QS2 可忽略字段。
4. 成功 `execution_kind="simultaneous_modular"`，包含生产者 streams、blocks（`kind=acyclic|coupled`）、final_units、coupling 和 costs。coupling / linear / failure 与 D06-04 对齐，新增字段按 snake_case；未知量 / 残差绑定均带领域身份，未计算值用 null，诊断中的非有限值用上述 tagged 对象。旧 `steps` 不构造循环步，QS2 costs 不带 QS1 iteration_evaluations。
5. Failed 的 `diagnostic_evidence` 保留 failure_kind、底层 error_code、诊断 code / 上下文、stage / block / unit_id / column、completed_blocks、active_units、active_coupling、costs。Stale / Missing / Failed 不提供正式 fields / streams；加载失败无核心证据。取消回执可携带已有的本次诊断，结果查询仍按原取消资格，不伪装 Failed。
6. origin 记录冻结计划、能力 / 响应 / 验收版本、规范摘要与已获得执行事实。Project 元数据保留入口原文件 `project_schema_version`、SHA-256 / document ID，另提供 `in_memory_required_project_schema_version` 和 `in_memory_required_capabilities`；v5 不把内存配置变化表述为已保存。新增能力即使不变外层也可被客户端发现。
7. `expected_project` 继续保护入口原字节；一次请求内冻结副本，外部后续改盘不合并。影响确认使用独立 `headless-v5-confirm-v1:` 摘要，覆盖前序动作、规范正式计划 / 能力、资源与领域影响预览；不接受 v3 / v4 摘要或泛化 force。仍只在既有需要影响确认的动作上要求，不额外给普通配置 / Run 加审批。
8. 请求重复键、未知字段 / 版本、null、非法数值 / 引用、冲突与不支持能力都沿结构化错误及既有退出码体系。CLI 仍一次性内存自动化，stdout 响应；不增加 Save、工程 JSON 下载、sidecar / 缓存 / 偏好写入或后台取消协议。只有实际 Run 使用已声明资源和授权。

公开[主案例工程](../../../examples/flowsheets/fixed-tp-cross-recycle-binary-hydrocarbon.rfproj.json)及[正式建图](../../../examples/automation/build-cross-recycle-v5.request.template.json)、[运行](../../../examples/automation/run-cross-recycle-v5.request.template.json)、[查询](../../../examples/automation/query-cross-recycle-v5.request.json)模板已加入 examples。v5 的 `connect_exact` 允许省略 `sink` 绑定产品出口，明确 null 仍拒绝；底层沿用 `MaterialConnectionTarget.sink=None`，不依赖旧无环候选提示。v4 仍必填 `sink`，不扩大旧词汇。资源路径须指向用户已有有效缓存，CLI 不生成演示包。

## 验证矩阵与实施顺序

下表为验收范围；本轮证据与未验项在文末区分，不能把矩阵要求视为全部已通过。核心独立算术 / 扰动 / 故障测试继续以父专题为准；消费者验证检查真实接入和身份边界，不在每层复制全数值矩阵。

| 编号 / 层 | 必须退出证据 |
| --- | --- |
| V01 codec / 严格性 | 默认和完整显式尺度往返，稳定顺序 / 摘要；两个契约版本、重复键 / 绑定、未知字段 / null / 非有限 / 非正尺度；已知缺能力报错、未知能力或合法未知方法 / 契约只读；设备 / 单位 / 固定证据保留 |
| V02 生命周期 | v1—v3→QS2 v4、QS1 v4→QS2、旧 v4 来源 Save As 空路径、QS2 覆盖旧目标、新能力取消、目标变更 / IO 失败；保存基线与草稿不丢，重开 Missing，切回旧方法不降外层；只读目标底层 writer 也拒绝 |
| V03 配置 / 身份 | 默认自动发现、两块显式完整映射、缺 / 多 / 错绑定未就绪、容量 / 不支持拓扑定位；删除 / 重建 / 改名 / 重开 / Undo 的 ID 保留，组分删换不暗重绑；相同规范配置 no-op |
| V04 资格 / readiness | QS1↔QS2、默认↔显式、A→B→A、同 ID / sequence 重开、克隆和另一工作区；旧报告 / 建议 / 已接受及在途结果均失效；plan-only 不改物理 revision / FlowEditContext，不取消或失效在途 QS0 / 设备 |
| V05 worker / 资源 | 真正后台 Ready / start / join、Busy / duplicate、取消加载前 / Ready 竞态 / 核心阶段、计划变化后迟到成功 / 失败 / 取消、错载荷、panic / 断连；加载 / 过期 / 包登记错误无伪来源，名额只在退出后释放 |
| V06 成功接入 | 从正式动作建立四成员主例，保存重开 rerun 与 CLI v5 运行；三块、八未知量、主例各流股独立分数基准，10 次局部调用 / 4 份响应 / 1 次分解 / 9 次三角求解；默认 / 显式得到相符解与真实 I/O |
| V07 失败与来源 | 双 0.99、近奇异拒绝、双 1 无有限稳态、最终复核故障、晚块 / 下游失败；注入的核心证据经 bridge / 查询无丢失，None 不变 0，NaN / ±Infinity 与 None 的 GUI / JSON / 文本投影可区分；部分块与旧成功不进入正式 fields，加载失败不伪造核心方法已执行 |
| V08 旧消费者 / QS0 | QS1 α=0.6 的 48+1 / 102 次与 0.99 原预算失败保持；QS0、Acyclic、旧 CLI v1—v4 golden / 词汇 / FFI 代表保持；旧 v4 在共享 parser 扩展后仍提前拒绝 QS2；历史独立 reader / writer 验证另列 |
| V09 查询 / 输出 / CLI | producer / assumed 区分；旧捕获在改计划 / 重算 / 重开 / 覆盖等待后拒绝；v5 严格解析、引用、期望冲突、独立确认摘要、原文件版本与内存能力分列；真实 CLI 子进程前后比较工程、目录条目、缓存 / sidecar / 偏好字节不变 |
| V10 GUI / 平台 | 画板批准后：配置→正式建图→Run→块详情→改计划 Stale→保存重开 Missing→F5 rerun；默认 / 显式尺度、草稿 / 单位 / 失效绑定、中英 400 px / 键盘、同版本升级 / 只读与取消；原生窗口先告知，未验平台明确保留 |

实施依次为：C-A（D06-01/02，codec / 配置 / 保存与 V01—03）→ C-B（D06-03/04，共享运行 / 结果与 V04—07）→ C-C（D06-06 CLI、经画板认可的 D06-05 GUI，与 V08—10）→ C-D（定向结果核对后一次正式 check-repo、代表实窗 / 历史 reader、状态归档）。CLI 与 GUI 均消费已验证共享链路，历史 reader 缺少可执行环境则明确记录缺口，不用新 parser 测试冒充通过。

各片做风险相称的定向验证，阶段收口按当前正式门禁执行；不放宽 D01—D05 的数值门槛，不将本机测试或固定温压成功当作 Windows / Linux / COM、工业物理或通用非线性验收。本轮消费者验证与既有 1,954 项核心阶段基线分开记录。

## 已批准项与影响

| 决策 | 2026-10-31 批准范围 | 影响边界 |
| --- | --- | --- |
| D06-01 | 独立方法 / capability / 两个契约 version 1；正式 Simultaneous 变体、自动耦合及默认 / 全量显式双尺度 | 新 store / UI 类型与严格字段；不新增物理参数、tear / 预算或核心算法 |
| D06-02 | 工程沿 v4；未知能力整份只读；同外层与 Save As 的来源 / 目标能力增量提示及资格绑定 | 新保存兼容基线 / 提示，旧格式 / writer 守卫和 staged write 保持 |
| D06-03 | 复用双资格、Ready 授权和单 worker，新增 Modular 分派 / 进度 / 取消，统一 readiness | Rust 应用运行接口增量；不改 QS0 输入资格、不增并发槽 / 暂停 |
| D06-04 | typed 联立快照 / 互斥失败证据、实际方法 / 响应 / 验收与包来源、诊断未计算 / 非有限区分、查询 / 输出投影 | 应用结果接口增量；旧 steps / tears 不变，无持久结果或外部 ABI |
| D06-05 | 现有 Run / F5 内新方法、尺度与块详情、升级 / 只读交互方向 | 先产出局部画板，经认可才接 GUI；本表批准不替代画板评审 |
| D06-06 | 独立 CLI v5、旧 v4 固定能力边界、类型化结果、独立确认摘要、内存能力查询 | 新 JSON 协议；工程外层不升 v5，旧协议词汇 / 退出码与磁盘边界保持 |

以上六项共同展开原 QS2-D06，已获明确批准。本轮按 C-A→C-B→CLI v5 实施；D06-05 方向批准不替代 GUI 画板认可，GUI 接线另行推进。未推送 / 创建 PR；AF1、I5、C2-A 原结论、C2-B / C3 与发布门禁不变。


## 本轮实现与验证落点（2026-10-31）

- C-A：`rf-store/project_v4` 增加严格 Simultaneous DTO、默认 / 显式双尺度、规范排序和能力推导；`rf-ui/flow_plan` 增加独立计划 / 草稿与失效引用保留。生命周期记录加载 / 成功保存能力基线，同版本 Save As 也检查来源增量，失败不提交基线。
- C-B：应用 bridge 与原单 worker 分派 Modular 进度 / 成功 / 失败 / 取消，输入＋计划双资格继续统管接收；互斥失败证据、真实最终局部 I/O 和包 / 实际方法来源进入查询。输出 guard 加载荷种类；联立 JSON 投影统一区分有限、缺失和非有限诊断。GUI 仅补穷尽匹配及旧能力只读保护，不新增联立展示。
- CLI：v5 独立 Plan 词汇，共享 `headless/planned.rs` 的顺序、引用、事务、资源与影响确认流程；v4 DTO 不接受联立字段，加载前固定 Sequential 能力。v5 原文件身份与内存所需能力分列，确认摘要为 `headless-v5-confirm-v1:`；无磁盘保存动作。
- 定向验证：codec / 配置 / 草稿 / 来源与目标升级、保存重开 Missing；四成员主例默认 / 显式双尺度对独立分数组分基准、三块 / 八未知量及 10 / 4 / 1 / 9 成本；Ready 改计划并 join、迟到成功 / 失败、错载荷、核心六阶段取消保留原 Current、加载失败无伪来源、非有限诊断透传。v5 严格字段 / 引用 / 规范顺序 no-op / 确认命名空间，旧 v3 / v4 回归及真实 CLI 建图 / 保存工程 rerun / 失败 / Stale / 重开 Missing 均有测试；子进程核对整个临时目录的文件字节和条目不变。
- 历史独立证据：从 `8c18f223` 的 `git archive` 在临时目录编译旧 `rf-store` 测试可执行程序，实际读取新公开工程返回整份 ReadOnly，原 JSON 不丢失，旧 serializer / `prepare_write` 拒绝输出 / 覆盖。该证据属于历史存储库可执行测试，不冒称历史 Studio GUI / CLI 二进制运行。
- GUI V10、原生窗口、Windows / Linux、FFI / COM / PME 和远端 CI 未执行；取消 / 错误的 GUI 文本、块详情及 GUI 导出等待画板后验证。数值核心故障矩阵沿 QS2-A/B，本片新增应用与 CLI 证据，不新增物理精度或通用非线性声明。正式 `check-repo` 通过 1,976 项、严格 Clippy / 治理 / 文本 / 格式，总耗时 35.59 s（不含外层 Cargo 构建 xtask）；命令及统计归 W44。
