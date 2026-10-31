# Solve Snapshot Results Reference

更新时间：2026-10-31

## 目的

本文档说明 `rf-ui::FlowResultRecord` 及各方法载荷的稳定语义；`SolveSnapshot` 专指无回路载荷。下文 step 细则只适用于该载荷。

它回答的是：

- `SolveSnapshot.streams` 和 `StepSnapshot` 里的流股各自代表什么
- source stream、非 flash 中间流股、flash outlet、step 输入/输出的正式边界是什么
- `comparison`、`unit-centric` 和 `diagnostic action` 这些结果消费面分别依赖什么
- `Results` command section、palette、menu、command list 和 runtime action button 如何共享同一条定位语义
- `StudioGuiWindowModuleResultsModel` 如何从 current-revision latest `SolveSnapshot` 派生选中单元结果
- 当前快照文本复制 / 导出和 `SolveSnapshot` 的关系
- 结果消费层不应该做哪些“二次组装”或私有状态分叉

它不是运行指南，也不展开 UI 操作步骤。

## 结果对象总览

当前与 Studio 结果审阅直接相关的主要对象是：

- `SolveSnapshot`
- `StreamStateSnapshot`
- `StepSnapshot`
- `BubbleDewWindowSnapshot`
- `PhaseStateSnapshot`

当前 `StepSnapshot` 至少包含：

- `index`
- `unit_id`
- `summary`
- `execution`
- `consumed_streams`
- `streams`

当前 Studio 结果消费面还会围绕同一份快照派生：

- stream-centric 结果视图
- unit-centric 结果视图
- `StudioGuiWindowModuleResultsModel`
- stream comparison
- diagnostic target / focus action
- `Results` command section 中的 stream / unit result navigation
- case-level review summary
- 当前快照文本 formatter，用于复制或导出 `.txt`

## 方法载荷与共同入口

`FlowResultRecord` 包含 Acyclic(SolveSnapshot)、Partitioned(PartitionedFlowSnapshot)、Simultaneous(SimultaneousFlowSnapshot)、EquationOriented(EquationFlowSnapshot)。共同 `FlowSnapshotHeader` 保存来源、ID、输入 revision、sequence、状态、诊断和生产者流股；`unit_result()` 返回真实消费 / 产出。只有 Acyclic 提供 `StepSnapshot`，其他方法保留最终单元复核、块或方程证据，不能用空 steps 判断“没有单元结果”。

QS1 记录撕裂 / 迭代与最终复核；QS2 记录响应、耦合、LU / 条件 / 后向误差；QS3 记录变量 / 方程、规格 / 区间轨迹、条件系统与完整 Jacobian、原值 / 求得比例及实际 I/O。部分失败证据不能作为正式流股发布。方法边界分别见 [QS1-C](../topics/modeling/recycle-project-and-consumers.md)、[QS2-C](../topics/modeling/simultaneous-project-and-consumers.md)、[QS3](../topics/modeling/equation-oriented-block.md)。QS0 独立物料任务与设备结果不属于 FlowResultRecord。

## `SolveSnapshot` 的稳定语义

`SolveSnapshot` 表示“某个文档修订号上的一次不可变求解结果”。

它是无回路方法的结果载荷，不是独立物理基准；`Converged`、相区、H 和泡露点窗口的含义受当前物性与单元近似约束。统一解释见 [热力学模型](../thermo/mvp-model.md)。

当前稳定边界：

- `streams` 表示这次求解后已物化的全局流股结果集合
- `steps` 表示按执行顺序记录的单元求解步骤
- consumer 应把它当成正式结果真相源，而不是中间缓存

当前允许存在的 selector state 只应承担“决定看哪一块结果面”的职责，例如：

- `selected_stream_id`
- `comparison_stream_id`
- `selected_unit_id`

这些状态不应被用来缓存第二份结果，也不应驱动第二次热力学判断。

## 当前资格、来源与动作守卫

`FlowResultRecord` 的数值不可变，但能否作为当前结果由 `WorkspaceState::flow_result()` 决定；文档 ID、revision、snapshot ID 或 sequence 相等都不能单独证明同属一次会话。`FlowResultView` 区分 Current、Stale、Missing、Failed，输入或计划 A→B→A、重开及重算都不能仅凭相同内容恢复旧资格；仅 Current 提供正式数值；Failed 携带该次诊断及来源，不能回退到上次成功值。未记录资格的旧快照明确视为不可用。

共同 `origin` 保留冻结的 `FlowResultOrigin`：输入与计划的会话 / revision / generation，以及可用的执行身份、solver、实际物性包与指纹。当前配置或包标签不得回填旧结果；准备失败可以没有实际包。应用层 `FlowResultReview` 是这一查询的拥有数据投影，不复制领域有效性规则。

跨帧结果动作和待写输出携带 `FlowResultGuard`，激活 / 写入时重新查询当前工作区；定位还检查目标存在。设备使用独立 `EquipmentResultGuard` 和原计算 / 矩阵输出捕获机制，按相关工况资格重验，不用流程 revision 作为设备全局锁。RunId / CalculationId 不透明且不持久化，技术详情 / 文本只说明身份已记录、会话内使用，不输出内存地址或可恢复令牌。

## `StreamStateSnapshot` 的稳定语义

单条 `StreamStateSnapshot` 当前稳定包含：

- `stream_id`
- `label`
- `temperature_k`
- `pressure_pa`
- `total_molar_flow_mol_s`
- `overall_mole_fractions`
- `phases`
- `bubble_dew_window`

其中：

- `phases` 表示当前已物化的相结果
- `bubble_dew_window` 表示当前 overall composition 在给定 `T / P` 下的平衡边界窗口

如果 `bubble_dew_window` 缺席，consumer 应按“当前没有窗口 DTO”处理，而不是猜测默认边界。

## `StepSnapshot` 的稳定语义

当前 step 级结果要分成两类看：

- `consumed_streams`：该单元实际消费的输入流股快照
- `streams`：该单元实际产出的输出流股快照

它们都应由 solver step 直接物化。  
当前 workspace run path、Studio window-model、Result Inspector 和 Module Results 都应继续只读消费这两组结构化 DTO。Module Settings 可以展示与选中单元相关的诊断目标，但不应重建 step 输入/输出结果，也不应把 latest-result 内容混入参数设置面。

当前不应做的事：

- 不按 stream id 从全局 `SolveSnapshot.streams` 反查并重建 step 输入
- 不按 stream id 从全局 `SolveSnapshot.streams` 反查并重建 step 输出
- 不在 UI / shell 中为 step stream 合成 placeholder 结果

## 四类常见流股的正式边界

### 1. Source stream

source stream 当前不是“只有文档态输入，没有结果态”的对象。  
在 solved path 上，它应能按正式数值链路物化：

- `T / P / F / z`
- 已物化的相结果
- `overall` phase 上的 `H`（若已物化）
- `bubble_dew_window`（若已物化）

当 source stream 被 first consumer 消费时，consumer step 的对应输入流股应继续保持同一份 DTO 语义。

### 2. 非 flash 中间流股

`Heater`、`Cooler`、`Valve`、`Mixer` 的 flowing outlet 当前都属于正式结果对象，而不是给 downstream flash 准备的临时桥接值。

当前这类流股应继续沿同一条正式链路透传：

- upstream unit output
- `SolveSnapshot.streams`
- downstream step `consumed_streams`
- Studio consumer

这几处不应各自维护第二套 `H / phase_region / bubble_dew_window` 口径。

当前 official demo case 与空白作者路径都应按这条链路核对 `Heater -> Flash`、`Mixer -> Flash`、`Cooler -> Flash` 和 `Valve -> Flash`：上游单元产出的中间流股、全局 stream 结果和 flash step 消费的输入流股必须保持同一份数值语义。

### 3. Flash outlet

`Flash Drum` outlet 需要区分 flowing outlet 和零流量对侧 outlet。

flowing outlet：

- 可带 phase rows
- 可带 `H`
- 可带 `bubble_dew_window`

零流量对侧 outlet：

- 当前允许 `bubble_dew_window` 缺席
- 当前允许 phase rows 缺席
- 当前允许 `H` 缺席

consumer 不应为了“显示完整”而补造这些缺席字段。

### 4. Step 输入/输出流股

step 输入/输出当前的职责是表达“这一步执行时实际看到的流股快照”，不是全局 stream map 的另一种索引视图。

因此：

- 若上游 output 与下游 consumed 指向同一股流，它们应保持同一份结果语义
- step DTO 的作用是锁定这个一致性，不是给 consumer 再做一轮拼装

## comparison / unit-centric / diagnostic action 的稳定语义

### comparison

当前 comparison 的正式边界是：

- 只比较当前同一份 `SolveSnapshot` 内已经存在的两股 `StreamStateSnapshot`
- 只消费已有 summary / composition / phase rows
- 只暴露定位到这两股流的 focus action

因此 comparison 不应：

- 触发重新求解
- 为缺失字段补造默认值
- 引入跨快照或跨运行的隐式结果混合

### unit-centric

unit-centric 视图当前只是在同一份快照里按单元重新组织结果面。  
它应继续只读消费：

- 最新 unit result
- related steps
- related diagnostics
- consumed / produced stream results

它不应自己再从全局 stream map 组一套“更完整的单元结果”。

### Module Results

`StudioGuiWindowModuleResultsModel` 是 Studio window-model 针对当前选中单元派生的 unit-centric presentation。它只读取 `WorkspaceState::flow_result()` 判定为 Current 的 `FlowResultRecord`，并围绕 selected unit 暴露：

- `selected_unit_result`
- consumed / produced stream chip
- related steps
- related diagnostics
- diagnostic actions

当前状态语义：

- `NoUnitSelected`：没有选中单元，不展示模块结果。
- `NoCurrentResult`：当前工作区查询无 Current 结果，提示按当前资格检查或重跑；不能仅比较 revision。
- `Stale`：当前查询判定已有快照失效；展示 stale notice，不渲染旧 selected unit result、stream chip、related steps 或 diagnostic actions。
- `NoUnitResult`：当前载荷中没有该单元的实际结果，可展示 related steps / diagnostics，但不补造 unit result。
- `Current`：当前载荷中存在该单元结果，正常展示 selected unit result 和关联对象。

这层 DTO 不承担完整报表、跨快照历史、重新求解或结果补算职责。若 Module Results 与 Result Inspector / 底部结果表对同一单元的 consumed / produced stream 表达不一致，应优先按消费层 bug 排查，而不是让 Module Results 自行拼装另一套结果。

### case-level review summary

case-level `review_summary` 是 Studio window-model 从同一份 Current `FlowResultRecord` 派生的审阅摘要，用于让用户先按流程链路确认“哪些结果对象应该被看见”。

当前分组语义：

- `source_stream_results`：source unit 产出、且被下游单元消费的流股，例如 Feed outlet。
- `intermediate_stream_results`：既由上游单元产出、又被下游单元消费的非 source 流股，例如 heater / cooler / valve / mixer outlet。
- `terminal_stream_results`：由单元产出、但不再被下游单元消费的终端产品流股，例如 Flash Drum liquid / vapor outlet。
- `unit_results`：每个单元的真实执行位置（顺序 step、依赖块或 EO verification）、状态、摘要及消费 / 产出流股；底部结果表的单元区应消费这层摘要，而不是在 shell 内另扫一遍 steps。
- `diagnostic_count`：当前快照中的诊断条目数量。

这层摘要不改变 `SolveSnapshot.streams`、`StepSnapshot.consumed_streams` 或 `StepSnapshot.streams` 的正式语义，也不承担结果推导、重新计算或报表模板职责。若某个流股在 `Review` 分组和 step 明细中表现不一致，应优先按消费层 bug 排查，而不是让 `Review` 自行补造结果。

底部结果表的定位行为也必须复用正式 focus command：点击 stream 行应派发 `inspector.focus_stream:*` 并进入右侧 `检查器`，点击 unit 行应派发 `inspector.focus_unit:*` 并进入右侧 `模块结果`，底部仍停留在 `结果表`。这保证底部结果表、Canvas 当前选择、右侧 `画布选择` 和 Inspector / Module Results 消费同一条定位语义。

### diagnostic target / focus action

当前 `inspector.focus_stream:*` 与 `inspector.focus_unit:*` 的正式语义只是“定位到某个当前已有结果对象”。

它们通常来自：

- stream comparison 的 `Inspect`
- unit-centric 视图的输入/输出流股 action
- `DiagnosticTargets`
- `Module Settings` / `Module Results` / `Result Inspector` / step 列表中的相关 action
- 底部结果表中的 stream / unit 行定位
- `Results` command section、command palette、menu 和 command list 中的 result navigation

稳定边界：

- action target 应从当前方法载荷、related steps 或 related diagnostics 派生；捕获的结果动作须先重验守卫和目标，再走正式 focus command，失效不能静默定位同 ID 的新对象
- `Results` command section 只能把当前载荷内已有的 stream / unit target 暴露为 `inspector.focus_stream:*` / `inspector.focus_unit:*`
- palette、menu、command list、runtime action button 与底部结果表都应继续走 host `dispatch_ui_command`，不为各自入口复制一套 target 解析
- 顶部 `结果工具栏` 可以扫读 focus command 是否可用，但不应把当前 snapshot 的所有 stream / unit focus command 直接展开成一排长按钮；具体对象定位应由结果表、右侧相关 action、项目树或命令面板承担
- `DiagnosticTargets` section 只汇总这组已存在 target，不另造 shell 私有状态机
- runtime 最终渲染面的 `Inspect` 标签和 `source | target | summary` 文本只负责展示这组 action，不重写其语义

当前稳定诊断口径：

- `solver.step.stream_input` 表示下游单元消费的输入流股缺少可用 stream specification，例如已连接 stream 没有至少一项 overall mole fraction。target 应优先携带相关 stream 与 inlet port，让 Run Panel recovery / focus 进入可编辑的 stream 输入，而不是把问题误归因为下游单元故障。
- `solver.step.parameter` 表示已进入项目文档的单元参数不满足当前单元约束，例如 pressure / temperature 不是正有限值，或 outlet pressure 高于已连接 inlet pressure。target 应携带 unit / port / stream context，修复入口是 Unit Inspector 的参数字段。

UI / shell 不应通过解析错误文本反推出这些分类；分类、summary 和 target 应来自结构化 diagnostic DTO。

### text copy / export

当前快照复制和文本导出只消费同一份 Current `FlowResultRecord`，并把已物化的流股摘要、case-level review summary、单元结果、求解步骤和诊断格式化为纯文本。

结果 DTO 数值始终为 SI。常规数量按 U3 生成 6 位有效数字文本；方法证据保留各自科学计数法与精度，不能把文本当成无损 SI 协议；缺失和转换错误分开表示，温差不使用绝对温度偏移。`light_text_export()` 返回结构化错误或完整文本，转换失败不会生成带错误占位的成功产物。

文本头包含 `snapshot_id`、`sequence`、`document_revision`、状态与摘要，以及 `display_units`、`number_format`。结果页和 Runtime 复用输出操作；发起时、剪贴板写入或文件选择器之前捕获工程身份、快照、项目单位和完整文本，不使用 Inspector 临时覆盖。等待期间显示单位变化不重写已捕获文本；文件写入前复验当前工作区、执行身份、快照及 revision；相同文档 ID / revision 的重开仍拒绝旧输出。平台选择器、覆盖和失败恢复契约见 [B2-1](../topics/results-review-diagnostics.md#b2-1轻量结果输出闭环)。

稳定 section 口径：

- `Streams`：来自当前载荷 common.streams 的流股摘要
- `Review`：按 source / intermediate / terminal streams、latest unit results 和 diagnostics count 汇总同一份快照
- `Units`：按单元列出真实执行位置、状态、summary、输入流股和输出流股；原 step 列在非顺序方法中表示块 / EO verification，不能按整数步骤解析
- `Steps`：仅 Acyclic 来自 StepSnapshot；其他方法不造步骤，另附分块 / 联立 / EO 的真实证据
- `Diagnostics`：来自当前快照的诊断条目
- `Execution origin`：冻结的执行来源、实际包 / 方法 / 指纹；未记录明确说明，保持文本审阅用途，不建立机器协议

稳定边界：

- 不创建独立的物理结果真相；待确认输出仅保留本次操作的不可变呈现数据
- 不重新计算热力学、相平衡、焓值或诊断
- 不写项目文件或 layout sidecar
- 不进入 `CommandHistory`
- 不承担完整结果报表、模板、批量导出或跨快照历史职责

## `H`、`phase_region` 和 `bubble_dew_window`

### `H`

当前结果区里说的 `H` 指：

- `molar_enthalpy_j_per_mol`

它来自已物化的相结果或 `overall` phase。  
当前 shell / UI 不应自行重算焓值。

### `phase_region`

`phase_region` 当前使用以下稳定词汇：

- `liquid-only`
- `two-phase`
- `vapor-only`

它属于 `bubble_dew_window` 的窗口语义，不是 phase rows 的直接替身。

### `bubble_dew_window`

窗口内至少包含：

- `phase_region`
- `bubble_pressure_pa`
- `dew_pressure_pa`
- `bubble_temperature_k`
- `dew_temperature_k`

对 two-phase flash outlet，当前允许窗口继续落在饱和边界附近；这时窗口语义和“实际只显示哪几行 phase rows”不必强行做成同一个概念。

## 对结果消费层的约束

当前稳定规则是：

- 只读消费 solver 已物化 DTO
- 不在集成层重算 `H`
- 不在集成层重算 `bubble/dew`
- 不在 UI / shell 中再组装第二套 step input/output
- 不在 UI / shell 中把 selector state 变成第二份结果缓存
- 不在 UI / shell 中新造一套 independent diagnostic/focus 语义
- 缺失字段按缺失处理，不伪造“更完整”的显示值

如果需要排查漂移，优先先比对：

1. `SolveSnapshot.streams`
2. step `consumed_streams`
3. step `streams`
4. 最终 Studio 展示

## 相关文档

- `docs/guides/review-solve-results.md`
- `docs/reference/units-and-conventions.md`
- `docs/architecture/app-architecture.md`
- `docs/thermo/mvp-model.md`
