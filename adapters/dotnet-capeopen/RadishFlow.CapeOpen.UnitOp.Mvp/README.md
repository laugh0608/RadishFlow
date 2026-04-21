# RadishFlow.CapeOpen.UnitOp.Mvp

当前目录已从纯占位推进为第一版最小 `net10.0` PMC 骨架项目，职责只限于：

- 提供一个最小 `ICapeIdentification` + `ICapeUtilities` + `ICapeUnit` 实现类
- 为后续真正的 CAPE-OPEN Unit Operation PMC 留出项目边界和最小状态机
- 提供最小内部 flowsheet/package 配置入口，并通过 `RadishFlow.CapeOpen.Adapter` 接入 `rf-ffi` 求解闭环

当前已包含的最小公共面：

- `RadishFlowCapeOpenUnitOperation`
- `UnitOperationPortPlaceholder` / `UnitOperationParameterPlaceholder`
- `UnitOperationPlaceholderCollection<T>`
- `Initialize / Validate / Calculate / Terminate / Edit` 的第一版状态骨架
- 内部 `LoadFlowsheetJson(...)`、`LoadPropertyPackageFiles(...)`、`SelectPropertyPackage(...)` 配置入口
- `SetPortConnected(...)` 这一类最小端口状态入口
- `ConfigureNativeLibraryDirectory(...)`、`LastCalculationResult`、`LastCalculationFailure`、`GetCalculationReport()`、`GetCalculationReportState()`、`GetCalculationReportHeadline()`、`GetCalculationReportDetailKeyCount()`、`GetCalculationReportDetailKey(int)`、`GetCalculationReportDetailValue(string)`、`GetCalculationReportLineCount()`、`GetCalculationReportLine(int)`、`GetCalculationReportLines()`、`GetCalculationReportText()`，以及公开 stable key catalog `UnitOperationCalculationReportDetailCatalog`
- `UnitOperationHostReportReader.Read(...)`、`UnitOperationHostReportSnapshot` 与 `UnitOperationHostReportDetailEntry`，用于让外部最小 host 基于既有公开 report API 一次性材料化状态、stable detail entries、scalar lines、vector lines 与 text，而不必在每个宿主里重复写同样的读取样板
- `UnitOperationHostReportPresenter.Present(...)` 与 `UnitOperationHostReportPresentation`，用于把 host snapshot 继续整理成更接近 UI / 日志组件的展示模型，明确 `StateLabel`、`RequiresAttention`、`StableDetails` 与 `SupplementalLines`
- `UnitOperationHostReportFormatter.Format(...)`、`UnitOperationHostReportDocument` 与 `UnitOperationHostReportSection`，用于把 presentation 收口成固定 section 输出，便于宿主直接渲染 `Overview / Stable Details / Supplemental` 这类展示分区
- `UnitOperationHostObjectDefinitionReader.Read(...)`、`UnitOperationHostObjectDefinitionSnapshot`、`UnitOperationHostParameterCollectionDefinition`、`UnitOperationHostPortCollectionDefinition`、`UnitOperationHostParameterCapabilities` 与 `UnitOperationHostPortCapabilities`，用于把 parameter/port catalog 的冻结对象定义和 host 可执行能力作为正式只读 host model 暴露出来，而不是让宿主从 runtime snapshot、catalog 静态类型或异常试探间接拼 definition/capability view
- `UnitOperationHostObjectRuntimeReader.Read(...)`、`UnitOperationHostObjectRuntimeSnapshot`、`UnitOperationHostParameterRuntimeEntry` 与 `UnitOperationHostPortRuntimeEntry`，用于把 parameter/port 当前运行时对象语义先收口成正式只读 snapshot，并同步暴露该对象的 capability，再供 configuration/action-plan 等更高层 reader 复用
- `UnitOperationHostObjectMutationDispatcher`、`UnitOperationHostObjectMutationCommand`、`UnitOperationHostObjectMutationOutcome` 与 `UnitOperationHostObjectMutationBatchResult`，用于把 parameter value 写入/reset 与 port connect/disconnect 这组最小对象修改动作收口成统一 host-facing mutation 边界；宿主既可逐条 `Dispatch(...)`，也可按输入顺序 `DispatchBatch(...)` 获取 ordered outcomes 与统一 invalidation 摘要，而不是让宿主侧散落直接调用 placeholder API
- `UnitOperationHostConfigurationReader.Read(...)`、`UnitOperationHostConfigurationSnapshot`、`UnitOperationHostConfigurationParameterEntry`、`UnitOperationHostConfigurationPortEntry` 与 `UnitOperationHostConfigurationIssue`，用于让外部最小 host 直接读取当前配置就绪度、blocking issues、next operations，以及 parameter/port 的只读配置摘要，而不必再自己把 catalog、placeholder 状态和 validation 失败分支重新拼成一套宿主私有判断
- `UnitOperationHostActionDefinitionCatalog`、`UnitOperationHostActionPlanReader.Read(...)`、`UnitOperationHostActionPlan`、`UnitOperationHostActionGroup`、`UnitOperationHostActionItem` 与 `UnitOperationHostActionTarget`，用于在 configuration snapshot 之上继续收口“宿主下一步该做什么”：按 `Lifecycle / Parameters / Ports / Terminal` 分组，直接给出 target kind/name(s)、reason、blocking 标记、canonical operation name 与推荐顺序，而不必再让宿主把 blocking issues 和 next operations 重新折叠成自己的 checklist
- `UnitOperationHostActionMutationBridge`、`UnitOperationHostActionMutationBinding` 与 `UnitOperationHostActionMutationCommandBatch`，用于把 action plan 继续桥接到可执行 mutation translation：显式区分 lifecycle-only、parameter value、port connection 与 unsupported 四类动作，并把可执行 action 收口成正式 `UnitOperationHostObjectMutationCommand` 批次，而不是让宿主继续手写 “action item -> mutation command” 映射
- `UnitOperationHostActionExecutionRequestPlanner`、`UnitOperationHostActionExecutionInputSet`、`UnitOperationHostActionExecutionRequestPlan` 与 `UnitOperationHostActionExecutionRequestPlanEntry`，用于把 action plan、宿主提供的 parameter values / port objects 与正式 execution requests 对齐：库内统一标记 `RequestReady / MissingInputs / LifecycleOperationRequired / Unsupported`，但不替宿主决定具体 flowsheet JSON、package id、连接对象命名或 lifecycle 调用时机
- `UnitOperationHostViewReader.Read(...)` 与 `UnitOperationHostViewSnapshot`，用于把 configuration、action plan、port/material、execution、report 与 session 六块正式 host view 收口到单一快照，避免不同 helper 再分别重复读取与拼装
- `UnitOperationHostFollowUpPlanner` 与 `UnitOperationHostFollowUp`，用于把“宿主下一步该做什么”统一收口为正式模型；当前覆盖 `LifecycleOperation / ProvideInputs / Validate / Calculate / CurrentResults / Terminated`
- `UnitOperationHostActionExecutionOrchestrator` 与 `UnitOperationHostActionExecutionOrchestrationResult`，用于把 request planning、action execution 与刷新后的 host view 一并收口成窄边界 orchestration helper：宿主可一次得到 planned action count、ready request count、missing inputs、mutation invalidation 摘要、执行后的最新 host views 与统一 follow-up，但该 helper 仍不负责 `Initialize / Validate / Calculate / Terminate`
- `UnitOperationHostValidationRunner.Validate(...)`、`UnitOperationHostValidationOutcome`、`UnitOperationHostCalculationRunner.Calculate(...)` 与 `UnitOperationHostCalculationOutcome`，用于把 `Validate()` / `Calculate()` 之后的正式 host view、统一 follow-up 与结果状态继续收口到库内，而不再要求 smoke host / contract tests 在调用后自己补读 session/report/execution 再判断下一步
- `UnitOperationHostRoundOrchestrator.Execute(...)`、`UnitOperationHostRoundRequest`、`UnitOperationHostRoundOutcome` 与 `UnitOperationHostRoundStopKind`，用于把“可选 action execution -> 可选 supplemental mutations -> 可选 validate -> 可选 calculate”这一条最常见宿主 round 主路径继续收口成正式结果：宿主可一次拿到 initial/final views、可选 action/supplemental/validation/calculation outcome、最终 follow-up 与统一 stop kind，但该 helper 仍不扩张成完整 smoke driver 或 PME 生命周期框架
- `UnitOperationHostActionExecutionDispatcher`、`UnitOperationHostActionExecutionRequest`、`UnitOperationHostActionExecutionOutcome` 与 `UnitOperationHostActionExecutionBatchResult`，用于把 action execution 继续收口成正式 helper：对 parameter/port action 直接走 mutation dispatcher，对 lifecycle-only/unsupported action 则返回显式 disposition，而不是让宿主自己在 bridge 结果之上再写一层执行分发
- `UnitOperationHostPortMaterialReader.Read(...)`、`UnitOperationHostPortMaterialSnapshot`、`UnitOperationHostPortMaterialEntry` 与 `UnitOperationHostMaterialStreamEntry`，用于在 calculate 结果面之上继续收口“每个 host port 当前绑定了哪些 boundary streams、这些 streams 是否已有当前 material result、若有则给出最小温压流量/相分率摘要”；宿主不必再自己解析 flowsheet JSON、推断 boundary stream 集或把 native solve snapshot 的 `streams` 数组重新映射回 `Feed/Product`
- `UnitOperationHostExecutionReader.Read(...)`、`UnitOperationHostExecutionSnapshot`、`UnitOperationHostExecutionSummary`、`UnitOperationHostExecutionDiagnosticEntry` 与 `UnitOperationHostExecutionStepEntry`，用于在 calculate 结果面之上继续收口“这次执行做了什么”：宿主可直接读取 `None / Stale / Available / Terminated` 四态、calculation status、summary、diagnostics 与 step-by-step 执行序列，而不必继续从 report supplemental lines 间接反推
- `UnitOperationHostSessionReader.Read(...)`、`UnitOperationHostSessionSnapshot`、`UnitOperationHostSessionSummary` 与 `UnitOperationHostSessionState`，用于在上述 readers 之上继续收口“一次读取当前完整宿主视图”：直接聚合 configuration、action plan、port/material、execution 与 report，并额外给出 canonical session state，以及 `IsReadyForCalculate / HasBlockingActions / HasCurrentResults / RequiresCalculateRefresh / HasFailureReport / RecommendedOperations` 这类宿主摘要
- `Calculate()` 对未满足前置条件的最小 ECape 语义抛错，以及经由 `rf-ffi` 的最小真实求解接线
- `ICapeCollection` / `ICapeParameter` / `ICapeUnitPort` 的第一版最小对象运行时
- placeholder 对象对 unit owner 生命周期的最小访问守卫，以及 `Terminate()` 时的端口连接释放

当前明确不包含：

- COM 注册 / 反注册
- 稳定 CLSID / ProgID 策略
- 报告接口的正式实现
- PME 生命周期集成
- 完整 CAPE-OPEN PMC 运行时

说明：

- 当前 `Ports` / `Parameters` 已返回带 `Item(object)` 和 `Count()` 的最小 `ICapeCollection` 风格对象，并支持按 `ComponentName` 或 1-based 索引取项
- 当前参数对象已提供最小 `ICapeParameter` + `ICapeParameterSpec` 语义，端口对象已提供最小 `ICapeUnitPort` 语义，但仍只覆盖 MVP 所需的字符串参数和占位连接对象
- 当前 `ICapeCollection.Item(object)` 已冻结为“1-based 整数索引或 component name”双入口；除 `int/long` 外，也接受能无损落到整数的 `double/float/decimal` 选择子，以贴近 COM 宿主可能传入的数值 Variant 形状；空白名称、越界索引和非整数数值仍按 `CapeInvalidArgumentException` 拒绝
- 在 COM 兼容的 `Item(object)` 之外，当前 `UnitOperationPlaceholderCollection<T>` 又已补出 typed runtime collection 主通路：`ContainsName(...)`、`TryGetByName(...)`、`GetByName(...)` 与 `GetByOneBasedIndex(...)`；后续 `UnitOp.Mvp` 自身、contract tests 和 smoke host 应优先走这条强类型入口，而不是继续在内部到处手写 `ICapeCollection.Item(object)` 选择子
- 当前 `ICapeCollection`、`ICapeParameter` 与 `ICapeUnitPort` 的 `ComponentName/ComponentDescription` 已冻结为运行时不可变元数据；宿主可以重复读取，但不能在 MVP runtime 中修改这些标识字段，从而保持 collection lookup、required port 规则与 stable detail key 不漂移
- 当前 `UnitOperationParameterCatalog` / `UnitOperationPortCatalog` 已进一步从“名字常量表”推进到“完整定义真相源”，把 canonical name、collection order、description、required/value-kind/mode 与 direction/port-type 一并收口，避免这些宿主契约继续散落在构造函数和测试字面量里
- 当前 `UnitOperationPortCatalog` 又已继续吸收 host-facing material 语义：port definition 现显式声明 `BoundaryMaterialRole`，冻结 `Feed -> boundary inputs` 与 `Product -> boundary outputs` 这层映射，不再让 smoke/contract tests 各自猜测 placeholder port 该代表哪一组 flowsheet streams
- 当前 parameter/port placeholder 又已进一步从“复制 catalog 元数据到运行时实例”收口为“直接绑定 catalog definition 对象”；运行时只保留 value / connection 这类可变状态，避免 definition 与 placeholder 元数据再次出现漂移
- 当前 `RadishFlowCapeOpenUnitOperation` 自身也已改成按 `OrderedDefinitions` 构造 parameter/port collection，并通过 catalog 名称回取 canonical placeholder；这样 unit 内部不再额外维护一套私有参数/端口清单，catalog + typed collection 才是唯一真相源
- 当前 parameter/port catalog 又已继续吸收“宿主应调用哪个公开操作来配置该对象”这层语义：parameter definition 现显式声明 `ConfigurationOperationName`，port definition 现显式声明 `ConnectionOperationName`；`Validate()` / `Calculate()` 失败时返回的 `requestedOperation` 已改为从这份 definition 元数据派生，而不是在 unit / smoke / contract tests 中各自硬编码
- 当前 parameter/port catalog 之上又已补出正式 `UnitOperationHostObjectDefinitionReader`；宿主现在可以直接读取 parameter/port collection、object definition 与 capability 的只读形状，不必通过 runtime snapshot 或异常试探间接反推冻结元数据和可操作性
- 当前 parameter/port 对象运行时又已补出正式 `UnitOperationHostObjectRuntimeReader`；configuration reader 现在基于这份 object runtime snapshot 构造配置摘要，不再同时直接混读 catalog definition 与 placeholder 状态
- 当前又已补出 `UnitOperationHostObjectMutationDispatcher`，并进一步冻结 `UnitOperationHostObjectMutationCommand` command model：先把 `SetParameterValue / ResetParameter / ConnectPort / DisconnectPort` 这四个最小 host mutation 收口为统一 `Dispatch(...)` 入口，再补出 `DispatchBatch(...)` 让宿主按顺序提交一组命令，并得到 `AppliedCount`、ordered outcomes、`InvalidatedValidation` 与 `InvalidatedCalculationReport` 这组批量摘要；失败路径仍继续保留既有 ECape 异常语义，不吞异常
- 基于这层 catalog 元数据，当前又已补出正式 `UnitOperationHostConfigurationReader`：宿主现在可以在 `Constructed / Incomplete / Ready / Terminated` 四种 configuration state 上读取 headline、blocking issues、next operations、以及按 catalog 顺序冻结的 parameter/port configuration entries，而不必先调用 `Validate()` 再反解析错误消息
- 基于这份 configuration snapshot，当前又继续补出 `UnitOperationHostActionPlanReader` 与 `UnitOperationHostActionDefinitionCatalog`：宿主现在可以直接读取分组后的 action checklist，而 action group、target kind、group title、group order 与 blocking 语义已收口到 action definition catalog，不再散落在 reader switch 分支里
- 在这份 action checklist 之上，当前又继续补出 `UnitOperationHostActionMutationBridge`：宿主现在可以先读取每条 action 属于 `LifecycleOperation / ParameterValues / PortConnection / Unsupported` 哪一类，再把可执行 action 明确翻译成 `UnitOperationHostObjectMutationCommand` 批次；manifest/payload 这类 companion action 会生成按目标顺序排列的多条 parameter commands，而 terminated / initialize 这类非对象修改动作则会继续显式停留在不可直接 mutation 的桥接状态
- 在这层 bridge 之上，当前又继续补出 `UnitOperationHostActionExecutionRequestPlanner`、`UnitOperationHostActionExecutionDispatcher`、`UnitOperationHostViewReader`、`UnitOperationHostFollowUpPlanner` 与 `UnitOperationHostActionExecutionOrchestrator`：planner 只把 action plan 和宿主显式提供的输入规划成 executable request plan，并清晰暴露缺失输入、lifecycle-only 与 unsupported action；dispatcher 再消费 ready requests，统一决定是返回 `LifecycleOperationRequired / Unsupported`，还是把 parameter/port action 落到 `UnitOperationHostObjectMutationDispatcher.DispatchBatch(...)` 并回传 ordered mutation outcomes、applied mutation count 与 invalidation 摘要；view reader 与 follow-up planner 继续统一刷新并归纳正式 host view / next-step 语义；orchestrator 最终把它们收口到单一结果对象。这样“读 action plan -> 准备宿主输入 -> 规划 execution request -> 执行 -> 刷新宿主视图 -> 判断下一步”这条最小宿主配置动作链已在库内收口，但仍不把 smoke 专用默认输入或完整 driver 生命周期上移成正式 API
- 在上述 action execution orchestration 之外，当前又继续补出 `UnitOperationHostValidationRunner` 与 `UnitOperationHostCalculationRunner`：最小 host 现在可以在 `Validate()` / `Calculate()` 后直接得到正式 `Views + FollowUp` 结果，不必继续手工补读 `session/report/execution` 再判断“下一步该 ProvideInputs、Calculate，还是已经进入 CurrentResults”
- 在 validation/calculation round outcome 之上，当前又继续补出 `UnitOperationHostRoundOrchestrator`：最小 host 现在可以把“先执行 ready actions、再按需要应用 supplemental object mutations、最后再 validate/calculate”这条常见 round 主路径直接收口到 `InitialViews + FinalViews + ActionExecution? + SupplementalMutations? + Validation? + Calculation? + FollowUp + StopKind`，而不必在 smoke host、contract tests 或未来 PME host 里重复写 phase gating、optional mutation 注入和 stop reason 判断
- 基于 flowsheet 配置与 calculate 结果，当前又继续补出 `UnitOperationHostPortMaterialReader`：宿主现在可以直接读取 `None / Stale / Available / Terminated` 四态的 port/material snapshot，以及每个 host port 对应的 boundary stream ids 和当前 material entries，而不必自己重做“flowsheet boundary -> host placeholder port -> solved stream”映射
- 在这层结果对象之上，当前又继续补出 `UnitOperationHostExecutionReader`：宿主现在可以直接读取 execution snapshot，而不必继续依赖 `GetCalculationReportLines()` 或 sectioned report 的 supplemental 文本去推断本次执行包含哪些 unit steps、消费了哪些 streams、生成了哪些 streams
- 在上述宿主只读面之上，当前又继续补出 `UnitOperationHostSessionReader`：宿主现在可以一次读取 configuration/action plan/port-material/execution/report 五块正式快照，并复用统一 summary 与 canonical session state，而不必在外部自己协调多次 reader 调用、汇总 headline 或再拼一层私有 status view
- 当前参数对象内部已补上最小元数据收口：区分 `StructuredJsonText` / `Identifier` / `FilePath` 三类值语义，保留对外 `ICapeParameterSpec.Type = CAPE_OPTION`，并显式记录默认值、是否允许空值与 manifest/payload 这类成对出现约束
- 当前参数对象又已把 `Specification` 从参数实例本身分离为独立只读 spec 对象；宿主重复读取时拿到稳定 spec 引用，而不再让 `ICapeParameter` 与 `ICapeParameterSpec` 混成同一个运行时对象
- 当前参数对象的 `Mode` 也已冻结为运行时不可变元数据；MVP runtime 当前只接受初始化时定义好的 mode，宿主不能在运行过程中把 input 参数改写成 output/input-output
- 当前参数对象的 `Value` / `IsConfigured` 读取也已纳入 owner lifecycle guard；`Terminate()` 后不再允许继续通过这些 helper 旁路访问对象状态
- 当前参数对象的 `Reset()` 语义也已收紧为“无论默认值是否与当前值相同，都把 `ValStatus` 复位到 `NotValidated`”，避免宿主在默认值场景下保留过期 validation 状态
- 当前端口对象已把连接契约收口到最小 `ICapeIdentification` 级别，并要求连接对象提供非空 `ComponentName`；当前仍不提前把运行时扩大到真正 `ICapeThermoMaterialObject`
- 当前端口对象的连接替换语义也已冻结：重复连接同一对象视为 no-op，但若端口已连接到另一对象，则必须先显式 `Disconnect()`，不能静默替换既有连接
- 当前 placeholder 对象已开始收口到宿主生命周期边界：`Terminate()` 后不再允许继续通过集合/参数/端口对象做 CAPE-OPEN 风格访问，并会释放端口上的已连接对象引用
- 当前 `RadishFlowCapeOpenUnitOperation` 内部生命周期又已从分散的 `_initialized / _terminated / _disposed` 布尔位收口为单一 `UnitOperationLifecycleState`，让 `Initialize / Validate / Calculate / Terminate / Dispose`、placeholder access guard 与终止后只读 report 查询共享同一套阶段判断，避免宿主边界继续靠多处布尔分支隐式维持
- 当前 `Calculate()` 内部执行链也已从单方法内联流程收口为显式分段：先做 `PrepareForCalculation()` 前置校验，再构造 `CalculationInputs`，随后执行 native solve、材料化结果并记录失败摘要；这样后续继续扩展宿主/PME 调用路径时，不必再从一段混合逻辑里追踪“校验失败、native 失败、contract parse 失败”分别落在哪个阶段
- 当前 `ValStatus` 与 calculation report 相关内部状态也已继续收口到正式变更入口：`ApplyValidationOutcome()`、`ResetCalculationState()`、`RecordCalculationSuccess()` 与 `RecordCalculationFailure()` 负责统一推进 `ValStatus`、`LastCalculationResult`、`LastCalculationFailure` 与 report 空态，避免 `Initialize / Validate / Calculate / Terminate` 继续各自零散写字段
- `Calculate()` 当前已能在最小前置条件满足后调用 `rf-ffi` 完成求解，并把对外结果面拆成“成功结果”和“失败摘要”两条最小契约：成功时导出 `LastCalculationResult(status / summary / diagnostics / streams / steps)`，失败时导出 `LastCalculationFailure(error / requestedOperation / nativeStatus / summary)`；完整 flowsheet snapshot JSON 与 native error JSON 仍只作为内部桥接输入，不再直接作为 PMC 公开结果面
- 当前又已补出 `GetCalculationReport()` 这一条统一只读查询面，把“尚无结果 / 最近成功 / 最近失败”收口到单一 report DTO；外部宿主若只需要稳定结果状态与结构化 detail，不必自己分支拼装 `LastCalculationResult` 和 `LastCalculationFailure`
- 在这条统一查询面之上，当前又补出 `GetCalculationReportState()` 与 `GetCalculationReportHeadline()` 两条最小标量元数据入口，让最小 host / PME 可以直接读取状态与标题，而不必先取出自定义 report DTO
- 在该元数据面之上，当前又补出 `GetCalculationReportDetailKeyCount()`、`GetCalculationReportDetailKey(int)` 与 `GetCalculationReportDetailValue(string)` 这一组最小 detail 键值读取入口，让宿主既可枚举当前稳定 detail key，又可按 key 直接读取值，而不必硬编码 key 列表或从展示文本里反解析
- 当前又已把 stable detail key 清单正式冻结到公开 catalog `UnitOperationCalculationReportDetailCatalog`：success 路径当前固定按 `status -> highestSeverity -> diagnosticCount -> relatedUnitIds -> relatedStreamIds` 排序；failure 路径当前固定按 `error -> operation -> requestedOperation -> nativeStatus -> diagnosticCode -> relatedUnitIds -> relatedStreamIds -> relatedPortTarget` 排序，其中部分 key 可能按场景缺省
- 在这条 report DTO 之上，当前又继续补出 `GetCalculationReportLines()` 与 `GetCalculationReportText()` 两条最小宿主可显示文本面，统一把 headline/detail lines 收口为可直接展示的行集合或多行文本，避免最小 host / PME 再自己手工拼接显示字符串
- 在上述文本面之上，当前又继续补出 `GetCalculationReportLineCount()` 与 `GetCalculationReportLine(int)` 两条标量读取入口，让最小 host / PME 可以按“行数 + 按索引读取”逐步消费报告文本，而不必依赖自定义 DTO 或一次性整段文本
- 基于上述公开 report API，当前又补出 `UnitOperationHostReportReader.Read(...)` 这一层最小宿主 helper，把 stable detail entries、scalar/vector lines 与 text 的读取样板前推到库内；这样后续外部 host / PME 若只需要消费稳定结果展示面，可以直接复用 helper，而不必在每个入口重复写 detail key 枚举和 line/text 收集逻辑
- 基于 host snapshot，当前又补出 `UnitOperationHostReportPresenter.Present(...)`，把“稳定 detail 行”和“补充展示行”拆开，并显式前推 `StateLabel / RequiresAttention / HasStableDetails / HasSupplementalLines` 这类更接近宿主 UI 和日志组件的展示语义，避免每个宿主再各自推断 failure 高亮、success 附加诊断区或 idle 空态标签
- 基于 presentation，当前又补出 `UnitOperationHostReportFormatter.Format(...)`，把宿主展示继续收口为固定 section 文档；这样最小 host 不只拿到字段化语义，还能直接按 section 渲染 overview、stable details 与 supplemental diagnostics，而不必每个宿主再自己决定分区标题和文本拼接顺序
- 在上述 report helper 之外，当前又补出 `UnitOperationHostConfigurationReader.Read(...)` 这一条配置只读路径，把“当前是否 ready for calculate”“还缺哪些 parameter/port”“下一步应该调用哪个公开操作”这类宿主驱动语义也正式前推到库内，避免这部分逻辑继续散落在 smoke host、未来 PME 适配或其他宿主入口中各自实现
- 当前“完整宿主如何驱动 PMC”的验证型 orchestration 仍故意留在 `RadishFlow.CapeOpen.SmokeTests`：`UnitOperationSmokeHostDriver` 现在通过 `UnitOperationHostActionExecutionRequestPlanner` 与 `UnitOperationHostActionExecutionDispatcher` 应用 parameter/port 类阻塞配置动作，并继续用直接参数/端口改写覆盖非法状态和 stale 状态边界；driver 暂不整体上移到库内，`UnitOp.Mvp` 本身当前只承诺 PMC 对象面、action plan / request planning / action execution helper 与结果读取/展示 helper，不提前承诺更高层宿主驱动 convenience API
- 当前又已补出同目录层级的自举 contract test 入口 `RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests`，用于在不依赖外部 NuGet 测试框架的前提下，直接锁定 `Validate/Calculate/Terminate/report transition` 这类库侧行为契约；当前已覆盖 `Validate before Initialize`、validation failure report、native failure report、success report、配置变更 invalidation 与 `Terminate()` 后阻断 6 条核心 case；后续若继续冻结 `UnitOp.Mvp` 对外行为，应优先在这里补细粒度 contract case，而不是只依赖 smoke console 间接覆盖
- 当前 contract tests 又已继续前推到对象面本身，新增 collection selector、parameter reset/lifecycle access 与 port reconnect 约束这三组契约，确保“最小宿主对象运行时”不再只靠 smoke 路径间接覆盖
- 在上述对象面 contract tests 基础上，当前又已补上 spec 对象稳定性、post-terminate spec access guard 与 parameter mode immutability 三条参数语义约束，避免后续再次把参数对象回退成“值对象和 spec 对象合一”的松散实现
- 当前 `SmokeTests` 与 `ContractTests` 也已切到消费同一份 parameter/port catalog；后续若 canonical order 或对象定义继续演进，应优先改 catalog，而不是在多个 host/test 入口各自追字面量
- 当前 contract tests 又已把 typed runtime collection 本身纳入行为契约：除 `ICapeCollection.Item(object)` 兼容面外，还锁住了 `ContainsName/TryGetByName/GetByName/GetByOneBasedIndex` 的成功/失败语义，并要求 typed lookup 返回的对象与集合中的 placeholder 实例保持同一引用
- 当前 contract tests 又已补上 action execution request planning contract，锁住 constructed/initialized/companion/terminated 下的 request-ready、missing-input、lifecycle-only 与 unsupported 规划语义，并确认 planner 产出的 ready requests 可以直接交给 dispatcher 把 unit 推进到 ready 配置状态
- 当前 contract tests 又已补上 companion validation failure 这条 case，并锁住 requested operation 会从 parameter catalog 的共享 `ConfigurationOperationName` 回读；`SmokeTests` 的 validation/native failure 断言也已同步改成依赖这份 catalog 元数据，而不是私有 `nameof(...)` 字面量
- 当前 contract tests 又已补上 configuration snapshot contract，锁住 constructed/ready/companion-mismatch/terminated 四种 configuration state、blocking issue kinds、next operations 与只读 entry 形状；`SmokeTests` 的 boundary/session 路径当前也已开始优先消费这套 configuration snapshot，而不是只靠散落的 parameter/port 判断
- 在 configuration snapshot contract 之上，当前又新增 action plan contract，进一步锁住 constructed / missing required parameter / companion mismatch / disconnected required port / ready / terminated 六类宿主 checklist 形状；`SmokeTests` 当前也已切到优先断言 action group、target、reason、blocking 与 canonical operation，而不再只盯 `NextOperations`
- 在 action plan contract 之后，当前又新增 port/material snapshot contract，进一步锁住 boundary stream 映射、`None / Stale / Available / Terminated` 四态，以及 success/invalidation/terminate 下的 host port material 形状；`SmokeTests` 当前也已开始直接消费这套 snapshot，而不是继续把 material 语义埋在 report 或样例私有判断里
- 在 port/material snapshot contract 之后，当前又新增 execution snapshot contract，进一步锁住 `steps` 解析、`None / Stale / Available / Terminated` 四态，以及 success/invalidation/terminate 下的 host execution 形状；`SmokeTests` 的 boundary suite 当前也已开始优先消费这套 snapshot，而不是继续从 report supplemental lines 间接判断执行过程
- 在 execution snapshot contract 之后，当前又新增 session snapshot contract，进一步锁住 constructed/native-failure/success/stale/terminated 下的统一宿主视图形状；`SmokeTests` 的 boundary suite 当前也已开始消费这套 session snapshot，而不是继续在外部重复汇总“是否 ready、是否 blocking、是否有当前结果、是否需要 refresh”
- Rust/.NET 边界仍保持为句柄 + UTF-8 + JSON + 状态码，没有在这里提前引入 COM 注册或更宽的跨边界对象传递

最小 contract tests 运行示例：

```powershell
dotnet build .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests.csproj -v minimal
.\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests\bin\Debug\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests.exe --native-lib-dir D:\Code\RadishFlow\target\debug
```
