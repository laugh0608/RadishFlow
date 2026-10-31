# QS1-C：回流工程保存与正式消费者

更新时间：2026-10-31

用途：定义 QS1-C 的版本化保存、运行计划与结果资格、建模入口、GUI / CLI 契约及实施门禁。
读者：负责 store、工作区事务、运行所有者、结果查询、Studio 与本地自动化的协作者。
不包含：新增求解算法 / 物理范围、跨进程运行服务、结果历史持久化、公共 FFI / COM 扩展或发布。

- 状态：**所有者已确认 QS1-C01—C06 及五张局部画板；C-A / B / C 实施与 C-D 代表验证完成，正式门禁通过 1,931 项测试。** 实施前核对基线 `81832068`；平台与历史读者缺口见文末。
- 父专题：[QS1 分块回流核心](recycle-and-block-execution.md)。D01—D05 已批准并完成 A/B；本篇展开原 D06，数值范围及验收容差继续以父专题为准。
- 本篇长期承载跨保存 / 任务 / 消费者的契约。父专题已有案例、算法和核心证据，继续加入以下内容会超过专题体量目标；生命周期、结果和 CLI 文档只保留职责及链接，避免分别定义计划身份。

## 实施前核对的断点

| 位置 | 当前事实 | QS1-C 必须处理 |
| --- | --- | --- |
| [store project](../../../crates/rf-store/src/project.rs)、[v3](../../../crates/rf-store/src/project_v3/mod.rs) | v1/v2 与独立 v3 外层；Splitter 写保护在 StoredProjectDocument.validate，共享读 / 写都会调用 | 保持旧入口拒绝，另加 v4；分离通用文档校验与版本能力校验，不能删掉共用守卫后让旧 writer 接受新参数 |
| [生命周期](../../../apps/radishflow-studio/src/document_lifecycle_driver.rs) | 非 v3 交给旧 reader；保存版本只在 2/3 选择；只读打开载荷绑定 ReadOnlyProjectV3 | 显式按 1/2/3/4 分派，保存 / 覆盖确认和只读检查具备版本信息，不用 v3 分支承接所有新格式 |
| [FlowRunQualification](../../../crates/rf-ui/src/flow_run.rs) | session / input / epoch / pending 保护一次性回执；same_input 不含独立计划 | 增加运行专用的计划资格，既要拒绝在途旧完成，也要让已接受结果失效 |
| [FlowEditContext](../../../apps/radishflow-studio/src/flow_actions/mod.rs)、[QS0 资格](../../../crates/rf-ui/src/material_balance/mod.rs) | 前者包裹 FlowRunQualification；后者也复用其输入身份 | 保持输入身份原义，计划改变不应让无关工程编辑或独立物料任务误失效 |
| [执行桥](../../../apps/radishflow-studio/src/solver_bridge/execution.rs)、[worker](../../../apps/radishflow-studio/src/workspace_run_owner/worker.rs) | job 只冻结 flowsheet / StudioSolveRequest，成功类型固定为旧 SolveSnapshot，进度是旧 SolveProgress | 冻结实际计划，类型化分派及结果 / 进度；沿现有单 worker、Ready 授权、取消、终态和释放链路 |
| [结果查询](../../../crates/rf-ui/src/result_query.rs)、[快照](../../../crates/rf-ui/src/run.rs) | Current 只借用旧应用快照；来源写入固定 solver 名；快照含一次性 steps | 新块级载荷及实际算法证据，不能伪造旧 steps 或使用快照序号代表迭代 |
| [连接动作](../../../apps/radishflow-studio/src/modeling_actions.rs)、[编辑检查](../../../crates/rf-ui/src/state/unit_edit.rs) | connect 先从建议列表匹配；无环辅助 / 重连会过滤闭环 | 明确的人工端口连接动作和按计划检查，不能只开放新单元就声称能建回流 |
| [规格检查](../../../apps/radishflow-studio/src/modeling_specifications/checks.rs) | 直接调用 sequential_execution_order；Mixer 压力上下文沿无环传播 | 将通用输入规则与计划可执行性分开；循环边界不能被旧无环检查误报或从模板推定压力 |
| [CLI v3](../../../apps/radishflow-studio/src/headless/v3/mod.rs) | query / execute 固定 schema_version=3，却共用 Studio 项目 parser；六类建模词汇 | 增加新协议分派；Studio 读 v4 后，旧 v3 也须在入口拒绝 v4，防止支持范围被共享 parser 悄然扩大 |

上表记录实施前的源码断点；当前实现与验证落点见文末。

## QS1-C01：项目 v4 与能力保护

### 外层与数据范围

建议新增独立 `rf-store::project_v4` codec：沿用 UTF-8 JSON、`.rfproj.json`、`kind=radishflow.project-file`，外层 `schemaVersion=4`，`document.metadata.schemaVersion=1`。保留原 document / presentation / engineering 的已有字段、SI 与来源语义；物理比例仍是 `UnitNode.parameters.outlet_a_split_fraction`，不在计划中复制 α。既有 document 内字段的大小写不迁移；新增外层字段采用 camelCase。新比例只允许出现在 Splitter，v4 不能成为把它挂在其他 kind 上的逃逸入口；原六类既有参数语义保持。

v4 新增必填 `requiredCapabilities` 和 `simulation.steadyStatePlan`。v4 明确保存一份活动稳态计划；首批不增加多计划列表、PlanId、运行记录、试算工作区或重启检查点。

| 字段 | 推荐契约 |
| --- | --- |
| `requiredCapabilities` | `{id,version}` 数组；ID 非空、版本正整数、ID 不重复，writer 按 ID 排序。写出内容与实际模型 / 计划一致，不能靠遗漏能力标记伪装兼容 |
| `document / presentation` | 必填，保留原语义；v4 已知单元参数用严格 DTO 校验，不能依赖会忽略未知字段的领域 serde 直接吞掉新字段 |
| `engineering` | 保留 v3 的省略 / 显式空区分及任务方法版本、固定采纳证据；提取需要共用的工程区纯 codec，不复制设备规则或共用不同外层版本的读写入口 |
| `simulation` | 必填对象，且包含 `steadyStatePlan`；严格拒绝 null、未知字段、错误类型和非有限数 |
| `steadyStatePlan.method` | `{id,version}`；首批为 `radishflow.acyclic-sm / 1` 与 `radishflow.partitioned-sm / 1` |
| 无回路计划 | 只有 method；携带回流参数应拒绝，不静默忽略 |
| 分块回流计划 | method、`acceptanceContractVersion=1`、`fixedBoundary`、`maxIterationEvaluations`、`tears` 均必填；只支持已批准的直接代入 |

拟定能力 ID 为 `radishflow.flow-plan / 1`（v4 活动计划）、`radishflow.unit.splitter / 1`（存在 Splitter 时）、`radishflow.solver.partitioned-sm / 1`（选择分块计划时）。设备方法继续使用工程区已有能力判定。v4 reader 在正式领域反序列化前检查外层和所需能力；writer 对已知内容推导标记并核对，不能原样相信调用方提供的标记。

### 分块计划字段

下列是新增 simulation 区的**拟定片段**，不是当前可读取的完整工程；流股 / 设备身份对应父专题案例：

```json
{
  "simulation": {
    "steadyStatePlan": {
      "method": {"id": "radishflow.partitioned-sm", "version": 1},
      "acceptanceContractVersion": 1,
      "fixedBoundary": {"temperatureK": 300.0, "pressurePa": 650000.0},
      "maxIterationEvaluations": 500,
      "tears": [{
        "streamId": "s-recycle",
        "source": {"unitId": "s1", "port": "outlet_a"},
        "sink": {"unitId": "m1", "port": "inlet_b"},
        "initialization": {"kind": "zero_component_flows"},
        "residualScaling": {"kind": "evaluated_fresh_total"}
      }]
    }
  }
}
```

- fixedBoundary 的温度 / 压力须为正有限 SI 值；它是计算任务的固定边界，不能替用户改 Feed、Mixer 或 Heater 的物理输入；声明目标不一致时定位具体字段，继续沿 A/B 的实际输出温压复核。
- `maxIterationEvaluations` 为整数 1—500，最终复核另计最多一次。`acceptanceContractVersion=1` 冻结 A/B 的双残差、总量容差传播和温压复核；本批不开放容差编辑或新的数值后端。
- 每个 tear 显式保存流股及源 / 汇端口。initialization 为 `zero_component_flows`，或 `component_flows` 加 `valuesMolS:[{componentId,value}]`。residualScaling 为 `evaluated_fresh_total`，或 `component_values` 加同形数组；显式初值有限非负，显式尺度有限正值，均须唯一完整绑定登记组分后才能运行。
- 默认策略存规则，不把上次求出的 5 mol/s 尺度或回流解当作新设置。运行时按本次各块已求新鲜边界固定尺度；零流组成沿 A/B 的明确规则。初值不从旧结果热启动。
- JSON 形状、枚举、数值域、重复绑定属于格式校验；未接完的工程、缺 tear、断开的绑定或组分表变化属于**可编辑但未就绪**。允许保存已提交的未就绪配置，重开显示具体问题，不修复、删除或隐式选择撕裂。
- writer 保留绑定身份；应用分配新单元 / 流股 ID 时，应将计划仍引用的身份纳入保留集合，包括重开后尚未解析的引用，避免删除对象后新对象复用同 ID 而意外重绑。显式重绑与 Undo 恢复原对象分别按正式动作处理，不建立全仓历史墓碑系统。

### 读取、升级与覆盖

| 情形 | 推荐行为 |
| --- | --- |
| v1 / v2 / v3 已知工程 | 原 reader 和保存语义保持；内存活动计划为兼容的无回路默认，不因打开就脏改或升级 |
| 全新 / 旧来源且仅原六类、无回路默认 | 继续按已有规则选择 v2 / v3；不因安装新版本就改写格式 |
| 新增 Splitter 或应用分块计划 | 内存正式输入可编辑；下一次保存需要 v4，显式说明旧程序不能打开，可取消或另存为 |
| 已加载 v4 | 后续保持 v4；切回无回路 / 删除 Splitter 也不自动降版 |
| 当前 v1/v2/v3 writer | 继续拒绝 Splitter / 新比例，包含直接文件 serde；不删除参数降版 |
| 历史发行版 reader | 须通过外层版本拒绝 v4；后续使用历史版本代表验证，不以新 reader 的测试替代 |
| 已知 v4、未知 required capability / 计划或验收契约版本 / 设备方法 | 整份只读检查，保留原始 UTF-8 内容和不支持原因；不建立部分可编辑 AppState，不运行、不保存覆盖 |
| 未知外层版本 | 明确拒绝，不先当作 v2 或 v3 尝试 |
| 已知能力却有未知字段、重复键、非法类型 / 数值 | 明确格式错误；不能先变 JSON map 丢重复键，再声称严格解析 |

`StoredProjectDocument::validate()` 的旧调用语义保持。实施时提取通用 metadata / thermo / 身份校验，v4 明确调用通用部分和自身能力校验；v1/v2/v3 文件入口与直接 Serialize 继续调用旧能力守卫。v3 已知工程区的复用不能绕过其原有未知方法保护。

保存继续沿现有 staged write、目标检查、覆盖 / 升级确认与失败恢复；v4 writer 独立检查现有目标，不借 UI 已确认标志覆盖未知格式 / 只读目标。Save As 也要检查目标能力。取消或失败保留原文件、路径、来源版本、各保存基线和草稿；只有成功才一起更新基线。升级确认的准备记录应绑定正式输入 / 计划资格、来源格式、所需能力与目标指纹；确认时重验，变化则重新准备，不用一个通用确认布尔值批准另一份内容。writer 仍需独立检查目标的格式 / 能力，沿现有 staged write 并发语义，不宣称新增跨进程锁。

## QS1-C02：工作区计划、事务与保存状态

计划属于项目的模拟配置；`rf-model` 继续只承载物理对象，不加入求解策略、接受令牌或工作区。建议在 rf-ui 增加 `FlowPlanState`（概念名）：当前正式配置、已保存基线、独立草稿 / Undo / Redo、会话内修订与不可复用 generation。store 使用独立 wire DTO，由应用边界转换；solver 继续只接自己的 typed options，store 不依赖 UI 或 solver。

- 增加计划正式动作 `apply_flow_plan` / `restore_saved_flow_plan`，以独立 `FlowPlanEditContext` 检查文档会话与计划 generation。应用、撤销、重做、恢复已保存值只要正式值发生变化，就产生新 generation；完全相同的提交为 no-op。
- A→B→A 的值可能相同，generation 仍不同；旧结果和旧完成不得恢复 Current。会话内数字修订仅供同会话乐观检查 / 回执展示；Arc 身份、执行令牌、Undo 和草稿不进工程。
- 计划变更不推进物理 document.revision，不调用全局 `input_changed()` 冒充物理改动，也不把 generation 塞进 FlowEditContext 的 same_input。QS0 独立物料配置、设备计算和无关工程编辑按原输入资格继续有效。
- ProjectSaveState 增加 `simulation_dirty`；统一保存摘要、离开保护、菜单状态及 CLI 查询使用同一来源。保存捕获正式计划及其基线，成功后记录保存；显示单位、工程基础和计划各自比较其已保存输入。
- 草稿保留原文和输入单位，非法 / 未应用草稿不被 Save、Run 或配置切换隐式应用。保存正式内容不清草稿；“保存并关闭”仍需处理未提交输入，复用既有离开保护。焦点在计划编辑区时 Undo / Redo 使用该区历史，并明确标注作用域。
- 工程编辑可使计划绑定未就绪；不在编辑时清掉 tear。Undo / Redo 后重验当前对象 / 端口 / 组分绑定，旧资格不因恢复原值复活。ID 分配保护按 C01；不能按名称重绑。

这是一份活动计划的项目数据，不将 QS0 的会话物料任务改成工程持久化任务，也不把设备结果或显示偏好混入计划输入。

## QS1-C03：执行分派、结果与资格

### 冻结与执行

建议扩展应用运行契约：`StudioSolveRequest` 增加明确的计划期望，`FrozenFlowRun` 冻结正式配置，`WorkspaceSolveJob` 携带不可变的 typed execution plan。旧构造 / 同步辅助入口显式表示 LegacyAcyclic，遇当前分块计划返回不支持；GUI / 新 CLI 捕获 `FlowPlanEditContext`，不得用随调用变化的“当前选择”替代期望。

计划期望建议为 `LegacyAcyclic | Selected(FlowPlanEditContext)`；Rust 精确签名在实施中据所有权收口，但不得省略这两个语义。preflight 检查输入与计划，再产生回执；冻结物理图、正式配置及实际选包请求。包由既有授权 / 装配路径加载，不静默改用其他包。

- Acyclic 调用原 SequentialModularSolver；Partitioned 调用 A/B 的 PartitionedSequentialSolver。未知 / 不支持计划直接阻止，不按失败结果切换策略。
- Frozen plan、包上下文和执行结果均在本次任务内归属。源 / 汇绑定、组分与边界在装配时重验；Ready 授权前再验运行资格。bridge / worker 适配新的 PartitionedSolverControl，进度用 `Acyclic(SolveProgress) | Partitioned(PartitionedProgress)`，不伪造总单元进度。
- 继续复用 Flow 与 QS0 共用的单 worker 名额、Busy / duplicate start、Ready / start、取消、完成与 join。新计划不创建第二个并发运行槽。Active 模式下正式计划改变产生独立 PlanChanged 待运行原因，待旧任务退出后按既有调度执行；Hold 模式保留过期状态直到手动运行。取消仍只在完整局部调用等既定安全点生效；暂停不在本批。
- 计划正式改变时撤销旧 Flow 接收资格并请求捕获旧计划的 Flow worker 取消，但在 worker 退出前仍占名额；不因切模式就释放线程，也不因仅改 Flow 计划取消正在运行的 QS0 任务。

### 两个资格维度

保留 FlowRunQualification 的原输入语义，新增运行专用的 `FlowExecutionQualification`，组合输入资格和计划 generation。receipt、origin、Ready / duplicate 判定、终态接收及输出 guard 都使用它；FlowEditContext 与 QS0 仍使用原输入维度。

| 事件 | 正式 Flow 结果 / 在途回执 | 其他状态 |
| --- | --- | --- |
| 修改物理输入或选用包 | 旧结果 Stale；旧完成拒绝 | 原工程规则保持，QS0 按其物料资格重验 |
| 应用 / 撤销 / 恢复不同的计划 | 旧结果立即 Stale(plan_changed)，旧成功 / 失败 / 取消不发布 | FlowEditContext 不变；QS0 / 设备结果不因仅改计划而失效 |
| 只改显示单位或未提交草稿 | 正式结果资格不改；草稿门禁与呈现提示另行表达 | 不应用正式计划，不伪造一次输入修订 |
| 同 ID / revision / 配置重开或切工程 | 新会话 Missing；旧完成拒绝 | 不从文件恢复运行或结果 |
| 数值成功、双资格仍有效 | 才能发布 Current | 局部收敛不等于物理准确性认证 |
| 当前任务失败 | Failed，保留诊断 / 实际来源，不返回正式流股值 | 上游或部分块只作失败证据 |
| 取消且输入 / 计划未变 | 不创建 Failed 或新成功；既有合格结果沿原 AF3 语义保留 | 取消本身不伪装输入变更 |
| 计划改回旧值、同 sequence 完成或重复交付 | 不复活旧结果；回执只结算一次 | sequence 只用于结果编号，不能充当会话 / 计划身份 |

资格判断与发布保持同一独占应用借用。结果查询必须同时检查输入和计划，结果载荷种类也必须与冻结计划一致；只 revoke pending 而保留旧 Current 是不完整实现。GUI / CLI / 变量浏览器 / 复制 / 导出都从同一结果资格读取，不能各建一套模式判断。

### 结果载荷与来源

建议在应用层新增 `FlowResultRecord::Acyclic(existing SolveSnapshot) | Partitioned(PartitionedFlowSnapshot)`，统一身份 / 资格 / 来源查询。原 core SolveSnapshot 及其 steps 保持原义；应用 snapshot_history、结果查询、运行页、变量读取和输出捕获须消费类型化载荷，不把循环展开成旧步表。

PartitionedFlowSnapshot 的稳定内容为：

- snapshot ID / document revision / sequence / 本次来源、最终生产者流股；
- 实际块顺序、成员、最终局部输入 / 输出，tear 绑定、assumed / returned、零流规则、尺度 / 容差；
- 迭代 / 独立复核次数、有界标量轨迹、实际构造 / 单元 / thermo 数值 / flash 调用及分阶段耗时；
- 成功需整次通过；失败仅在专用诊断载荷保留阶段、块 / 设备 / 流股 / 端口 / ComponentId、计数及残差轨迹，正式结果 fields 不返回部分流股表。

来源记录**实际执行**的 solver / contract version、`direct_substitution`、验收契约、冻结配置摘要和已求尺度；物性沿 AF5a 冻结实际包版本、payload 指纹、方法和 thermo / flash 实现。可获得的应用构建 / core 实现版本随本次来源记录，未知则明确未记录，不拿 Cargo 版本或当前分支名冒充提交证据。配置摘要采用规范 typed 内容编码，排除显示文案、耗时和临时运行 ID，只供复现与说明，不能代替不可复用资格。加载失败时未取得的来源保持未记录，不能从当前下拉框补值。

变量 / 结果读取中，回流流股采用本次生产者 returned；块详情如实显示 Mixer 消费的 assumed 及容差差异。复制 / 导出捕获还须包含实际结果载荷类型、计划资格、会话与结果身份，写前重验；改计划、重算或同 ID 重开后旧捕获不得写出。GUI 原文本输出增加计划 / 块及来源内容；CLI 新结构化结果按 C06 发布。

## QS1-C04：Splitter 编辑、显式连接与计划就绪性

- 开放正式 Splitter 创建、规范端口、删除与历史；增加 `VariableField::OutletASplitFraction`、Inspector 数值会话、动作发现 / 写入，量类型 DimensionlessRatio、规范单位 1、有限 [0,1]，缺值显示未指定。只写物理参数，不在计划编辑器提供第二个 α。
- 新能力在内存中需要 v4；实际保存仍走 C01 确认。旧 CLI / FFI 的六类词汇和旧 writer 保持边界，不因目录第七类存在就自动开放旧适配器。
- 新增明确的端口连接动作（概念名 ConnectExactMaterialPorts），GUI 人工端口选择与 CLI v4 共用。它按显式 source / sink / StreamId 校验规范端口、方向、类型、一源一汇和当前编辑资格，不要求边已出现在无环建议表中。人工连接可形成闭环、允许编辑未完成工程；是否受支持由当前计划的 readiness 判断，不在连接时假装完成全图求解。
- 现有无环自动建议 / 唯一重连继续保持原能力；不为支持人工回流而自动闭合未知回路，不自动选择 tear。旧消费者仍使用原受控路径。
- 建议拆分 `通用物理输入检查` 与 `按所选计划的可执行性检查`。无回路计划保留原循环诊断；分块计划调用核心装配 / 能力检查，回流边界不走无环压力推断或使用旧出口模板。缺连接、未配置 tear、绑定失配、超出单 Mixer / Splitter SCC 范围、α=1、固定边界不一致分别定位。
- 检查报告、建模辅助建议和运行按钮状态需要绑定计划 generation；计划变化后旧“可运行”报告失效。物理字段编辑会话仍按 FlowEditContext 守卫，不能因仅切计划就丢草稿；涉及运行能力的建议需独立重验。

首批分块执行能力继续限 A/B 的固定 T/P、单显式 tear 的 Mixer / Splitter 循环块及其串联；人工能够画出复杂回路，不意味着已有复杂回路求解能力。

## QS1-C05：GUI 局部交互方案

入口保持当前运行区：Run / F5 使用**已应用**活动稳态计划。已有无回路工程默认行为保持；QS0 独立物料区继续独立。计划设置、运行状态、结果资格和保存升级提示各自有明确来源。

| 局部区域 | 交互与状态 |
| --- | --- |
| 运行计划 | 下拉选择“无回路顺序” / “分块回流（固定温压）”；编辑后应用或恢复，显示已保存 / 未保存与未应用草稿。切换草稿不自动执行 |
| 回流设置 | 固定 T/P、预算、显式 tear 表、初值 / 尺度规则；候选按稳定 ID / 规范口显示并辅以名称。多个串联 SCC 各自配置，表中不按名称推测回流角色 |
| 数值输入 | 复用单位 / 数值会话；容差为验收契约只读说明。非法 / 未提交计划草稿阻止该计划运行，保留文本；旧正式资格不变，界面明确结果对应已提交计划，草稿期间暂不提供当前数值输出 |
| 建模 | Splitter 有明确入口与一入两出标识；人工选择源 / 汇端口完成回流。比例编辑在单元 Inspector，tear 在计划区，能互相定位 |
| 后台运行 | Preparing / Ready / Running / Cancelling / terminal；分块显示块号 / 成员、迭代或最终复核与残差，不以 500 为必须走满的百分比 |
| 结果 | Current 显示最终流股及块详情；assumed / returned 并列，实际调用 / 包来源就近展开。Stale / Missing / Failed 隐藏正式数值，失败保留诊断定位 |
| 保存 / 重开 | 首次 v4 升级说明旧版本不兼容；取消不落盘。重开恢复正式计划、比例与单位，结果 Missing，点击运行后重新验证 |

**C05 确认只批准交互方向，不代替画板认可。** 方向确认后先补五张局部 Pencil 画板：配置 / 绑定、Current 块详情、草稿 / 过期、取消 / 失败、升级 / 只读；覆盖中英、400 px 窄区、字号放大和键盘。画板认可后再接 GUI；按现有 UI 专题计划，不重画全局工作台，不因等待画板阻塞已批准的存储 / 核心应用工作。

## QS1-C06：CLI v4 与兼容消费

建议为新能力使用 `schema_version=4` 的 query / execute 协议，沿现有 `--headless query / execute <request>`。协议版本与工程版本是两个维度，不能假定相等；请求先只读一次字节、严格解析并按 3/4 分派，保持旧入口的回应与退出码。

| 适配器 | 读取 / 执行边界 |
| --- | --- |
| inspect / run v1/v2、现有项目 FFI | 原能力保持；v4 工程明确不支持，不新增块级结果 ABI |
| query / execute v3 | 保留原 v1/v2/v3 工程与旧动作 / 结果；在任何编辑 / 资源读取 / 运行前拒绝 v4，不通过新版 Studio parser 偷偷接受 |
| query / execute v4 | 可读 v1—v4 的已知能力工程；旧工程默认为无回路。新增 Splitter、精确端口连接、计划查询 / 应用和分块结果，经共享应用动作执行 |
| v4 只读工程 | 仅返回安全元数据、原格式版本和不支持原因；不创建可编辑 AppState，不执行步骤 |

拟定 v4 增量：

1. 查询增加 `flow_plan` scope 的正式配置、会话内 `plan_revision`、未就绪原因、方法 / 能力与动作描述；unit field / create kind 增加 Splitter 与分流比例；Flow results 区分 `execution_kind=acyclic_sm|partitioned_sm`。
2. `configure_flow_plan` 原子提交整份候选配置，要求 `expected_plan_revision`；`run_flow` 也明确要求计划期望，不能从未声明的 GUI 选择取值。版本引用可沿用前序步骤 receipt 机制；输入内容为规范 SI，不接收文本草稿、任意单位或直接替换整个内部 workspace。
3. 成功分块结果包含生产者流股、块 / 成员、真实复核和调用证据；Stale / Missing / Failed 沿统一资格返回原因 / 来源，正式 fields 不含旧值。失败残差 / 调用数归 `diagnostic_evidence`，与 Current 结果载荷区分。临时运行身份继续用请求步骤引用，不序列化 Arc 或跨进程令牌。
4. `expected_project` 继续校验入口原始工程字节指纹与 document ID；计划加入工程后也受整文件条件保护。本次执行仍冻结内存副本，后续外部改盘不并入。原文件的 `project_schema_version` / 指纹保持入口事实，另报告 `in_memory_required_project_schema_version`，不能把一次内存编辑说成已保存 v4。
5. 影响确认复用领域提案与规范摘要编码，v4 使用独立 `headless-v4-confirm-v1:` 命名空间，覆盖前序动作、正式计划及能力变化；不接受旧 v3 摘要或宽泛 confirm / force。重复键、未知字段 / 版本、前向引用、类型错配、冲突、只读和不支持能力均有结构化错误。
6. CLI 继续是一次性本地自动化：响应给 stdout，错误分类与既有退出码体系一致；不增加 Save、工程 JSON 下载、sidecar / 偏好 / 认证缓存写入或后台取消协议。共享读取 / 编辑不要求包，实际 Run 才要求现有显式资源配置与授权。

v4 不复制整套 v3 执行逻辑；按需要提取传输中立的解析 / 引用 / 动作适配，旧 v3 DTO、严格词汇与 golden 保持。CLI 和 GUI 经同一配置、就绪性、运行所有者及结果查询，避免各自实现一套收敛 / 资格判定。本批无新依赖或独立 CLI crate。

## 实施顺序与验证门禁

| 批次 | 内容 | 退出证据 |
| --- | --- | --- |
| C-A 存储与正式配置 | C01 / C02：v4 codec、能力探测 / 只读、计划状态 / 事务 / 保存脏标记、升级 / 目标保护 | v1—v4 兼容矩阵、严格反序列化、设备 / 单位完整保留、计划和比例往返、升级取消 / IO 失败 / 未知目标保护；重开结果 Missing |
| C-B 运行与共享建模 / 结果 | C03 / C04：typed job / 载荷 / 控制、双资格、按计划 readiness、Splitter 字段与精确连接 | A/B 解析案例从正式动作构建并运行；计划改变立即 Stale、A→B→A、同 ID 重开、Ready 竞态、重复 / 后发成功 / 失败 / 取消及错类型载荷均拒绝；单 worker 占用和释放无回归 |
| C-C CLI / GUI 接线 | C06 与经局部认可的 C05；共用动作 / 查询，复制 / 导出守卫 | v4 正例 / 故障 JSON、旧 v1/v2/v3 golden 与拒绝矩阵、磁盘不变；GUI 计划→构建回流→运行→块审阅→保存重开→rerun，窄区 / 中英 / 键盘与草稿往返 |
| C-D 阶段收口 | 定向验证后正式 check-repo、文档、平台代表证据 | 不以模型测试替代 GUI 或历史读者；真实窗口先告知，平台未测项明确保留，未获授权不发布 |

实施中必须覆盖：

- 默认 / 显式初值、α=0 / 0.6 / 0.9 / 0.95 / 0.99 / 1、第三零组分、串联 SCC 与第二块晚失败；保持 A/B 的尺度 / 容差，不以消费者接入放宽数值范围。
- plan-only 编辑保持物理 revision 与已捕获 FlowEditContext 有效、QS0 / 设备结果不误失效；反向验证物理编辑会使计划绑定 / 运行结果重验。解绑、删除、改名、重开后新 ID 分配及 Undo / Redo 不错误重绑。
- 已接受结果与在途结果均检查计划资格；配置改回、同 snapshot ID / sequence、事务克隆、独立工作区、被撤销的 Ready、并发 Busy、线程 panic / 断连不发布成功。
- 输出准备后改计划、重算、换会话、覆盖确认期间变化必须拒绝旧捕获；非 Current 不向变量 / CLI 当前 fields 泄露旧数值，失败诊断与正式数据分离。
- 已知 v4 + 未知能力只读、未知外层拒绝、已知能力未知字段报错；完整保留 v3 工程基础 / 设备 / 固定证据和呈现单位，不丢字段或降版覆盖。
- 当前可用历史 reader / CLI / FFI 代表对新格式的拒绝、保存重开与计划 / 物理字段验证；可用性与数值验证分别记录，缺少历史可执行环境时保留缺口。

## 确认项与实施边界

| 决策 | 已确认的具体变化 |
| --- | --- |
| QS1-C01 | 新工程外层 v4、必填能力清单及 simulation 区、严格新 DTO / 只读 / 升级与旧 writer 守卫拆分 |
| QS1-C02 | 活动稳态计划作为独立项目模拟配置；独立事务 / 草稿 / 历史 / generation、simulation_dirty 与保存基线 |
| QS1-C03 | Studio 请求期望、冻结计划 / typed job 与块载荷、运行专用双资格、实际来源、结果 / 输出守卫 |
| QS1-C04 | Splitter 正式字段 / 创建 / 历史、精确人工闭环连接、引用 ID 保护、按计划 readiness 与检查报告资格 |
| QS1-C05 | Run / F5 使用已应用计划及上述局部交互；确认方向后出画板，画板认可后再接 GUI |
| QS1-C06 | CLI 新 schema_version=4、计划 / 块级结果与新动作；旧协议入口拒绝新工程，FFI / COM 范围保持 |

C01—C06 已获所有者明确批准，五张局部画板也已认可；按上述批次实施，新增实质范围仍先确认。实施证据归本篇各阶段状态和 W44，未验内容不能以方案或 A/B 基线代替。

### 局部画板（2026-10-31，已认可）

源文件为 [studio-client-main.pen](../../architecture/designs/studio-client-main.pen)，新增五个 QS1-C 分区；保留原设计和 QS0 分区。静态检查确认布局、文字可读性与边界，PNG 是评审稿，不是运行态截图：

- [配置 / 绑定](../../architecture/designs/reviews/2026-10-31-qs1-c/V53oyj.png)：计划 / 显式 tear、物理比例独立、原 Run / QS0。
- [Current 块详情](../../architecture/designs/reviews/2026-10-31-qs1-c/ugP6O.png)：生产者结果、assumed / returned、残差与实际调用。
- [400 px 英文草稿 / 过期](../../architecture/designs/reviews/2026-10-31-qs1-c/L5y3ww.png)：未完成指数、输入单位、应用后过期与键盘。
- [取消 / 失败](../../architecture/designs/reviews/2026-10-31-qs1-c/aCfoN.png)：名额占用到退出、预算失败、正式值与诊断分开。
- [升级 / 只读](../../architecture/designs/reviews/2026-10-31-qs1-c/YhPvL.png)：取消为默认焦点，未知能力不建立部分可编辑工作区。

## 已实现的链路与验证落点（2026-10-31）

- **保存与配置：** [v4 codec](../../../crates/rf-store/src/project_v4/mod.rs)严格区分已知可编辑与未知能力只读，复用 v3 工程 / 单位区而不放宽旧格式守卫。[生命周期](../../../apps/radishflow-studio/src/document_lifecycle_driver/flow_plan.rs)按版本分派、显式升级并检查目标原字节；v4 往返覆盖 rating / adoption 固定输入和全部显示单位。正式计划、saved baseline、独立历史与数值草稿位于 [flow_plan](../../../crates/rf-ui/src/flow_plan.rs)，计划引用的失效身份在重开后仍保留给分配器；保存不提交或丢弃草稿，重开无运行结果。
- **运行与结果：** 正式请求不论 Acyclic / Partitioned 都捕获 Selected 计划期望；旧 StudioSolveRequest::new 保持 LegacyAcyclic。job 冻结实际计划与图，沿既有授权 / Ready / 单 worker 分派；计划变化撤销 Flow，释放仍等待 worker 实退，QS0 不误取消。[FlowResultRecord](../../../crates/rf-ui/src/flow_results.rs)类型区分旧 steps 与真实分块；结果来源包含实际方法、验收契约、规范配置 SHA-256 和加载包证据。缺少构建 / commit 证据时不伪造版本事实。
- **建模与报告：** Splitter 通过正式创建 / 字段 / 既有数值编辑与历史链路，比例无缺省值且限定 [0,1]。ConnectExact 是独立显式事务，检查规范端口、方向、占用、流股唯一端点与身份保留；旧自动建议仍要求无环。工作区规格报告、检查 ticket 与建议预览绑定计划 generation；循环入口压力不从旧模板推出证明。
- **GUI：** 五张画板已逐一认可后接线。运行页配置 / tear 绑定、输入单位、逐组分初值 / 尺度、应用 / 恢复 / Undo / Redo、草稿保留、显式连接、Current / Stale / Failed 与块详情已接通。块详情保留最终真实局部 I/O、assumed / returned、残差 / 容差 / 轨迹与实际调用；状态汇总按实际载荷显示块数和算法。结果表 / Inspector / 来源与轻量输出复用资格；写前计划改变拒绝旧捕获。
- **CLI：** [v4 DTO / 适配](../../../apps/radishflow-studio/src/headless/v4/mod.rs)与旧 DTO 分离，共用有序执行、引用、领域动作、授权与影响确认。查询 / 执行只动本次内存副本，入口原文件版本 / SHA 与内存所需保存版本分列；v3 在资源读取前拒绝 v4。配置与运行均检查 plan_revision，设备提案使用独立 headless-v4-confirm-v1 摘要并纳入前序计划动作。成功输出块和生产者流股；失败的试算值仅在 diagnostic_evidence，非 Current 不给正式 fields。

[公开回流工程](../../../examples/flowsheets/fixed-tp-recycle-binary-hydrocarbon.rfproj.json)为 300 K / 650000 Pa、两股 2 / 3 mol/s 新鲜进料、Mixer + Splitter 的 0.6 回流与下游 Heater；非进料模板故意偏离解。正式动作构建同一场景的应用测试与 CLI v4 均得到五块、48 次迭代 + 1 次独立复核、102 次局部调用，重复求值一致；0.99 达预算失败与 1 无有限稳态不发布结果。数值边界与串联 SCC 的完整 A/B 矩阵继续由父专题维护，接线不放宽核心范围。

定向验证还覆盖计划 A→B→A、Ready 前撤销后等待 join、错载荷类型、报告过期、保留失效 ID、未完成数值 / 输入单位、计划编辑不误伤在途 QS0、旧输出 / 覆盖确认资格与旧 v1/v2/v3 代表。400 px 中英 egui 帧验证草稿保留；macOS 实窗验证开 v4 → Run → 循环块 48+1 → 预算 500 改 499 后 Stale → 另存临时副本 → 重开 499 / Missing → F5 重算 Current。历史独立可执行版本、Windows / Linux 原生窗口及 .NET / COM / PME 未在本轮复验；固定温压物料通过不表示完整能量求解或独立工业物理准确性。正式门禁、命令和最终数量在 [W44](../../devlogs/2026-10/2026-W44.md)记录。
