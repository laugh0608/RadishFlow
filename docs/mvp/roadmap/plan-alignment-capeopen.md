# MVP Roadmap Plan Alignment: CAPE-OPEN

更新时间：2026-05-14

## 用途

用途：归档 2026-04-16 至 2026-04-25 的 .NET 10 CAPE-OPEN / COM / PME 计划对齐历史。
读者：需要追溯 M4/M5 适配层、注册、TypeLib 和 PME 兼容路径的开发者。
不包含：最新阶段判断、Rust 数值模型说明和操作 runbook。

截至 2026-04-16，M4 与 M5 交界处的 `.NET 10` CAPE-OPEN 地基又进一步前推，当前已完成：

- `rf-ffi` 当前已冻结为以 `engine` 为中心的最小 ABI，至少覆盖 `engine_create/destroy`、最近一次错误文本/JSON、flowsheet 装载、package 注册/列举、求解以及 flowsheet/stream snapshot 导出
- `RadishFlow.CapeOpen.Adapter` 当前已形成可编译的薄适配层，并把 `RfFfiStatus + last_error_message/json` 收口为可复用的 ECape 语义异常
- `RadishFlow.CapeOpen.SmokeTests` 当前已形成最小 smoke console，可配置 native lib 目录，并分别演练 direct adapter 求解与 `UnitOp.Mvp` 求解路径
- `RadishFlow.CapeOpen.Interop` 当前已形成最小 CAPE-OPEN 接口、GUID、HRESULT 与 ECape 异常语义骨架，为后续 PMC/注册层共享
- `RadishFlow.CapeOpen.UnitOp.Mvp` 当前已建立最小 PMC 状态机，并把 `Ports` / `Parameters` 先推进为最小占位对象集合，使 `Validate()` 可基于对象状态检查必填参数与必连端口；`Calculate()` 当前也已能经由 `RadishFlow.CapeOpen.Adapter` 调用 `rf-ffi` 完成最小求解闭环，并把对外结果面收口到稳定的“成功结果 + 失败摘要”双契约：成功时提供 `status / summary / diagnostics`，失败时提供 `error / requestedOperation / nativeStatus / summary`
- `RadishFlow.CapeOpen.UnitOp.Mvp` 当前又已在其上补出统一只读查询面 `GetCalculationReport()`，把 `none / success / failure` 三种状态收口到单一 report DTO，为后续最小 result/report access 与 PME 只读消费面预留稳定入口
- `RadishFlow.CapeOpen.UnitOp.Mvp` 当前又已继续补出 `GetCalculationReportState()` / `GetCalculationReportHeadline()`，作为最小标量元数据消费面，让宿主侧不必为了读取报告状态和标题而依赖自定义 DTO
- `RadishFlow.CapeOpen.UnitOp.Mvp` 当前又已继续补出 `GetCalculationReportDetailKeyCount()` / `GetCalculationReportDetailKey(int)` / `GetCalculationReportDetailValue(string)`，作为可枚举的最小 detail 键值消费面，让宿主侧不必预先硬编码稳定 detail key 列表
- `RadishFlow.CapeOpen.UnitOp.Mvp` 当前又已把 stable detail key 清单冻结为公开 catalog `UnitOperationCalculationReportDetailCatalog`，把 success / failure 两条路径的 canonical key 顺序正式写回代码契约，而不只停留在文档描述
- `RadishFlow.CapeOpen.UnitOp.Mvp` 当前又已继续补出 `GetCalculationReportLines()` / `GetCalculationReportText()`，作为建立在 report DTO 之上的最小宿主可显示文本面，避免最小 host / PME 再自行拼接 headline/detail 字符串
- `RadishFlow.CapeOpen.UnitOp.Mvp` 当前又已继续补出 `GetCalculationReportLineCount()` / `GetCalculationReportLine(int)`，作为更接近宿主逐行读取习惯的标量消费面，避免后续最小 host / PME 对自定义 DTO 或整段字符串形成不必要依赖
- 基于上述连续收口，`UnitOp.Mvp.Calculate()` 之后的最小结果面当前已进入“外部最小 host 消费样例验证通过”的阶段；`SmokeTests` 当前已不再依赖 `LastCalculationResult` / `LastCalculationFailure` 作为主消费面，而是只通过公开 report API 与稳定 key/line 口径完成状态判断和展示
- `RadishFlow.CapeOpen.SmokeTests` 当前又已从单条 console 冒烟脚本收口为最小宿主验证骨架，显式固定 `Initialize -> 配参数 -> 连端口 -> Validate -> Calculate -> 读结果 -> Terminate` 调用顺序，并形成 `UnitOperationSmokeHostDriver + UnitOperationSmokeBoundarySuite + UnitOperationSmokeSession + UnitOperationSmokeScenarioCatalog + Program 调度` 结构
- `RadishFlow.CapeOpen.SmokeTests` 当前已支持 `--unitop-scenario <all|session|recovery|shutdown>` 按宿主时序场景过滤运行，从而把“调用顺序、最小输入、失败分类、恢复与收尾”固定为 smoke 基线，而不是继续堆更多随机 console 脚本
- `RadishFlow.CapeOpen.UnitOp.Mvp` 当前又已把内部生命周期、验证、计算和结果状态迁移继续显式分层：`UnitOperationLifecycleState` 统一 owner 生命周期，`EvaluateValidation()` 已拆为 guard 链，`Calculate()` 已拆为准备/建输入/执行/材料化/失败记录阶段，`ApplyValidationOutcome / ResetCalculationState / RecordCalculationSuccess / RecordCalculationFailure` 统一状态迁移
- `RadishFlow.CapeOpen.UnitOp.Mvp` 当前又已在库内补出 `UnitOperationHostReportReader -> Presenter -> Formatter` 三级最小宿主消费链，用来冻结读取、展示模型与 sectioned 文档格式，而不是继续给 PMC 主类扩 accessor
- 当前又已新增 `RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests` 自举式项目，并锁定 `Validate before Initialize`、validation failure report、native failure report、success report、configuration invalidation 与 `Terminate()` 后阻断这 6 条核心契约；其中 validation/native 两类 failure report 的 detail 字段缺省规则也已正式固定
- 基于上述验证，当前这条“最小宿主驱动路径 + PMC 内部状态/失败语义收口”子线可视为阶段性完成；driver 暂不建议上移到 `UnitOp.Mvp` 库内，下一步应转向 `UnitOp.Mvp` 对象面主线，而不是继续深挖 smoke DSL、注册或 PME 互调
- 本轮已同步回写 `overview`、`boundary`、`architecture draft`、`roadmap` 与周志文档，确保上述骨架状态不只停留在 README 或零散提交说明里

当前仍明确不提前推进：

- COM 注册 / 反注册
- 默认 `local-machine` 安装路径与安装包化注册器
- 完整 PME 互调与完整 CAPE-OPEN `Collection/Parameter/UnitPort` 运行时
- Rust 侧引入 COM 语义

这一轮回写后，M4 的“最小可调用边界”和 M5 的“PMC 语义骨架前置”边界应视为正式冻结口径。

截至 2026-04-18，`.NET 10` `UnitOp.Mvp` 对象面与宿主交互语义又进一步前推，当前已完成：

- parameter / port definition 已正式收口到 catalog 真相源，runtime placeholder 改为直接绑定 definition，不再复制 metadata
- `UnitOperationHostConfigurationReader` 之上又继续形成 `UnitOperationHostActionPlanReader`、`UnitOperationHostPortMaterialReader`、`UnitOperationHostExecutionReader` 与 `UnitOperationHostSessionReader`
- 宿主侧当前已可直接读取分组动作 checklist、目标 parameter/port/unit、blocking reason、canonical operation name、boundary material 映射、执行摘要与统一 session snapshot
- `UnitOperationHostSessionState` 当前已冻结为 `Constructed / Incomplete / Ready / Failure / Available / Stale / Terminated`，不再要求宿主基于多组布尔位自行判态
- `ContractTests` 当前已锁住 configuration / action-plan / port-material / execution / session 五层只读模型的形状契约；`SmokeTests` 当前也已开始在 timeline 中显式输出 `sessionState`
- 当前这条推进仍明确停留在 M4/M5 交界的“库内宿主只读语义收口”，不代表已经进入 COM 注册、PME 互调壳或第三方 CAPE-OPEN 模型加载阶段

截至 2026-04-20，`UnitOp.Mvp` 对 action execution 编排又做了一次边界收口，当前判断为：

- `UnitOperationSmokeHostDriver` 仍不整体上移为 `UnitOp.Mvp` 正式 API，因为它同时承担 smoke 默认输入、调用顺序、失败分类和场景验证职责，仍属于验证型宿主样板
- 已稳定的 action plan 到 execution request 的公共部分上移为 `UnitOperationHostActionExecutionRequestPlanner`
- 新增 `UnitOperationHostActionExecutionInputSet` 与 request plan / entry 模型，显式区分 `RequestReady / MissingInputs / LifecycleOperationRequired / Unsupported`
- 这层 helper 只消费宿主显式提供的 parameter values 与 port objects，不替宿主选择 flowsheet JSON、package id、连接对象命名或 lifecycle 调用时机
- `SmokeTests` 当前已改为先由 driver 准备输入集合，再通过 planner 生成 ready requests，最后交给 dispatcher 执行；这样 smoke host 不再私有维护 action item 到 execution request 的分类映射
- `ContractTests` 当前已新增 action execution request planning contract，锁住 constructed、initialized、companion mismatch 与 terminated 下的规划语义，并验证 planner 产物可直接驱动 dispatcher 把 unit 推进到 ready 配置状态

同日后续又继续前推一步：

- 在 request planning 与 dispatcher 之上，当前又补出 `UnitOperationHostActionExecutionOrchestrator`
- orchestrator 当前会统一返回 request plan、execution batch outcome，以及刷新后的 configuration / action plan / session，不再要求 smoke host 在执行动作后自己重读并拼这三块正式视图
- 在 orchestration result 之上，当前又补出正式 `FollowUp` 模型，显式区分 `LifecycleOperation / ProvideInputs / Validate / Calculate / CurrentResults / Terminated`
- `FollowUp` 当前还会带出 `MissingInputNames`、`RecommendedOperations`、`CanValidate` 与 `CanCalculate`，把“宿主下一步该做什么”前推到库内，而不是继续由 smoke timeline 或未来 PME 宿主各自判断
- `SmokeTests` 当前已开始最小消费这层 orchestration result / follow-up；`ContractTests` 也已新增 orchestration contract，锁住执行后刷新与 follow-up 语义
- 在 action execution 之外，当前又补出 `UnitOperationHostViewReader`、`UnitOperationHostValidationRunner` 与 `UnitOperationHostCalculationRunner`，把 `Validate()` / `Calculate()` 后的 host views 与 follow-up 也正式收口到库内，而不再只覆盖 action execution
- 在 validation/calculation outcome 之上，当前又补出 `UnitOperationHostRoundOrchestrator`、`UnitOperationHostRoundRequest`、`UnitOperationHostRoundOutcome` 与 `UnitOperationHostRoundStopKind`，把“可选 action execution -> 可选 supplemental object mutations -> 可选 validate -> 可选 calculate”这一条最常见宿主 round 主路径继续收口到库内
- `SmokeTests` 当前已开始在 `UnitOperationSmokeSession` 里优先消费 round outcome；`ContractTests` 也已新增 host-round contract，锁住 lifecycle gate、missing inputs、success、native failure 与 terminated 下的统一 stop/follow-up 语义
- `HostRoundRequest` 当前又已补出 supplemental mutation commands，用来容纳这类不属于 blocking action plan、但宿主仍需在 validation/calculation 前显式写入的配置；`SmokeTests` 已用它替换原先私有的 optional package-file 写入补丁
- 当前又已补出独立 `RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost` console，直接复用 `host view / request planner / host round` 正式消费面来演示最小外部 host 路径，不再让“如何脱离 smoke DSL 消费库内正式边界”只停留在 README 建议
- 基于 `SampleHost` 已验证路径，当前又补出 `PmeLikeUnitOperationHost / PmeLikeUnitOperationSession / PmeLikeUnitOperationInput` 薄宿主入口，把“创建组件、初始化、读取视图、提交参数/端口对象、执行 validate/calculate round、读取正式结果面、终止”整理成更接近 PME host 的最小 session 形状；这层仍只复用 `UnitOp.Mvp` 正式 reader / planner / host round，不进入 COM 注册、PME 自动化互调、第三方模型加载或 smoke driver DSL
- 当前又把 `RadishFlowCapeOpenUnitOperation` 的注册前置身份冻结到 `UnitOperationComIdentity`，并新增 `RadishFlow.CapeOpen.Registration` dry-run/preflight console，输出 `CLSID / ProgID / Versioned ProgID`、CAPE-OPEN categories 与已实现接口清单；这一步不写注册表、不注册 COM、不启动 PME，也不加载第三方 CAPE-OPEN 模型

基于上述进展，当前下一阶段计划进一步对齐为：

- 继续优先深化 `UnitOp.Mvp` 对象运行时与宿主交互语义的一致性，不把主线拉回继续堆 report accessor、测试私有 helper 或 host round 兜底
- 把已形成的 host-facing 只读模型、request planning 模型、host view snapshot，以及 orchestration / validation / calculation / host-round outcome + follow-up 模型视为正式库内消费面，未来 smoke host / PME host / 其他宿主入口都应优先复用，而不是各自重组配置、动作、输入规划、执行后刷新、material、execution 与 session 语义
- 当前 PME-like 薄宿主入口与 registration preflight 已足以证明正式消费路径和组件身份口径可继续承接 M5；`Registration` 当前已能 dry-run 生成 `register / unregister` 与 `current-user / local-machine` 的 registry key plan，并输出 comhost、位数、权限口径、目标 registry key 与备份范围的只读 preflight。下一步若继续推进，应转向执行型 COM 注册工具的确认门控、真实写入实现、安装/反安装脚本、以及目标 PME 人工验证说明，而不是继续深挖 host round 兜底或 smoke driver 抽取
- 当前已补出 `docs/capeopen/pme-validation.md`，把目标 PME 人工验证路径、执行前验证基线、dry-run 审查项、执行型注册门控、通过标准、失败分类和记录模板先冻结到正式文档；下一步若继续推进 M5，应优先实现执行型注册工具的显式确认、备份/回滚和权限检查，而不是继续扩展 host round 兜底
- 继续维持当前阶段边界：不提前扩张到 COM 注册、PME 自动化互调壳、第三方 CAPE-OPEN 模型加载或把 COM 语义倒灌回 Rust

截至 2026-04-22，M5 的注册入口与人工验证准备又进一步前推，当前已完成：

- `RadishFlow.CapeOpen.Registration` 当前已从“只读 dry-run/preflight”前推为“默认 dry-run、显式 `--execute` 才写入”的受限执行工具，并收口 confirmation token、preflight `Fail` 阻断、HKLM elevation 检查、`CLSID / ProgID / Versioned ProgID` 三棵树 JSON 备份、execution log 与失败 rollback
- 当前已在真实本机环境完成一次底层 `Registration.exe` 的 `current-user register/unregister` 闭环验证，并顺序复查三棵 registry tree 的创建与删除结果
- 仓库根 `scripts/register-com.ps1` 当前也已从占位入口补成正式脚本，统一 build、环境变量重定向、token 提示与执行命令组装；脚本入口的 `current-user register/unregister` 闭环验证当前也已独立跑通
- `docs/capeopen/pme-validation.md` 当前又已继续补成脚本化安装/反安装运行手册，明确 dry-run -> register -> 顺序复查 -> PME 人工验证 -> unregister -> 顺序清理 这条推荐路径
- `examples/pme-validation/` 当前已补出正式目录说明与验证记录模板，后续目标 PME 记录不再只停留在文档正文里的字段清单

基于上述进展，当前下一阶段计划进一步对齐为：

- 不再把 M5 的“下一步”定义为设计 execute 工具；这部分当前已阶段性完成
- 优先选择首个目标 PME，并按 `examples/pme-validation/` 模板记录一次完整人工验证
- 对 `local-machine` 路径只做策略评估，不把 HKLM 注册升级为默认安装路径
- 除非发现正式消费面仍有具体缺口，否则不再继续围绕 host round 兜底或 smoke driver 做扩展

截至 2026-04-24，M5 当前又进一步收敛到 `TypeLib` 主线，新增结论如下：

- 目标 PME 优先级当前已固定为 `DWSIM 9.0.2 + COFE`；`Aspen Plus` 不再作为默认开发验证路径，只保留给用户侧必要阶段性手工复验
- `DWSIM / COFE` 当前已不再是“完全发现不到组件”；真实现象已推进到“能发现，但添加无反应/崩溃”
- 经过真实 COM 探测，当前已确认 `ProgID -> CLSID` 映射与 discovery 注册树问题基本打通；`CoCreateInstance` 已可成功
- 当前真正阻塞点已收敛为晚绑定 `IDispatch` 首次调用时报 `0x80131165 Type library is not registered`
- `UnitOp.Mvp` 当前已补入冻结的 `typelib/RadishFlow.CapeOpen.UnitOp.Mvp.idl`、`typelib/RadishFlow.CapeOpen.UnitOp.Mvp.tlb`，并已验证 `<ComHostTypeLibrary ...>` 可接入 `.NET comhost` 构建
- `Registration` 当前已继续前推到真实 `TypeLib` 注册链路：dry-run 会优先解析真实 `UnitOp.Mvp` 输出目录中的 `ResolvedComHostPath / ResolvedTypeLibraryPath`、校验 `comhost runtime layout` 与 `TypeLib GUID/version`，execute 会按 scope 调用 `RegisterTypeLib(ForUser)` / `UnRegisterTypeLib(ForUser)`，并把 `TypeLib` 纳入备份/回滚范围
- `RadishFlow.CapeOpen.UnitOp.Mvp.tlb` 当前也已随 `UnitOp.Mvp` 与 `Registration` 输出目录一起复制，减少脚本和执行入口对源码路径猜测的依赖
- 本机工具链当前已确认可用：`D:\Windows Kits\10` 下的 `midl.exe / rc.exe` 与现有 Visual Studio `cl.exe` 已可用于执行 `scripts/gen-typelib.ps1`，问题不再是“本机没有工具”或“生成步骤只能手工执行”
- 同日真实复验又确认 `pwsh` 的 `0x800080A5` 是宿主预加载 `.NET 9.0.10` 与 PMC 目标 `.NET 10.0.0` runtime 不兼容导致的假阴性；native COM / PME 类宿主的后续探测应改用 `Windows PowerShell 5` 或其他非预加载 .NET 宿主
- 在 `Windows PowerShell 5` 下，current-user `TypeLib` 已出现在 `HKCR` 合并视图，且 `CLSID\{...}\TypeLib` 关联也已补上，但 `New-Object -ComObject` 仍报 `0x80131165`；因此剩余主线已进一步收敛到 classic late-bound COM / typelib 兼容细节，而不是默认 comhost 路径或 runtime sidecar 缺失

截至 2026-04-25，M5 的 late-bound COM 探测又进一步前推，当前新增结论如下：

- VS 2026 Insiders 当前可提供更新的 `cl.exe`，但未直接提供 `midl.exe / rc.exe`；`IDL -> TLB` 组合工具链当前仍应优先采用 `D:\Windows Kits\10\bin\10.0.26100.0\x64` 下的 `midl.exe / rc.exe`，再配合 VS `cl.exe`，并由 `scripts/gen-typelib.ps1` 统一封装
- `UnitOp.Mvp\bin` 下的 `comhost.dll` 曾出现落后于 `obj` 最新嵌入 TLB 产物的情况；后续做注册复验前应优先执行 `UnitOp.Mvp` rebuild，并确认 `bin / obj` comhost hash 一致
- 先前 `0x80131165` 的直接修正点已收敛为 assembly-level COM identity 与默认 interface 口径：`Interop` / `UnitOp.Mvp` 程序集当前均显式声明与冻结 TLB 一致的 `Guid / TypeLibVersion`
- `RadishFlowCapeOpenUnitOperation`、parameter/port collection、parameter/port placeholder 当前均已补出 `ComDefaultInterface`，避免 classic late-bound 宿主只能看到空的 `__ComObject` 或错误默认面
- `ContractTests` 当前新增 `assembly-com-identity-contract`，锁住上述 TLB identity 与默认 interface 选择
- Windows PowerShell 5 真实环境下，`New-Object -ComObject`、默认 `ICapeUtilities.Initialize()`、`Parameters.Count()`、`Parameters.Item(1).Specification` 与 `Terminate()` 当前均已通过，先前 `0x80131165` 已不再复现
- `ICapeUnit` 当前通过 `QueryInterface` 返回 `S_OK`；但 PowerShell 默认 late binding 只代表默认 `ICapeUtilities` 面，`Ports / Validate / Calculate` 仍应在真实 PME 或强类型宿主路径中复验

同日后续真实 `DWSIM / COFE` dump 采集曾显示，阻塞点一度超出普通 TypeLib / optional interface 补齐范畴：

- `DWSIM` dump 指向 `.NET 10` `coreclr.dll` 内部崩溃，且 DWSIM 同进程同时承载 .NET Framework 4.x CLR 与 `.NET 10` CoreCLR/comhost
- `COFE` dump 指向 `COFE.exe` native 侧空指针访问，trace 仍只到 RadishFlow 主对象 constructor exit
- `DWSIM` trace 仍只到 `IPersistStreamInit.InitNew()` exit，未进入后续 `IOleObject` 或 CAPE-OPEN automation 成员
- `dotnet-dump` 复读 DWSIM dump 后确认崩溃线程停在 `System.StubHelpers.InterfaceMarshaler.ConvertToManaged -> IL_STUB_COMtoCLR -> ComMethodFrame`，当前先把无状态 `IPersistStreamInit.Load / Save` managed 签名改为 raw `IntPtr` stream，避免进入方法体前触发 `IStream` interface marshaling
- 该阶段后续已继续收敛到 `ICapeUtilities` vtable、`SimulationContext` raw pointer setter、parameter placeholder 多接口暴露与端口连接 COM 入参释放；截至 2026-04-26，`DWSIM / COFE` placement 与 port connection 已通过，因此暂不把 out-of-proc shim 作为当前优先路线

同日晚间继续沿真实 `DWSIM / COFE` trace 收敛，当前进一步结论如下：

- `ICapeUtilities.SimulationContext` setter 必须保持 raw `IntPtr`，避免 DWSIM 在进入 setter 前触发 CLR interface marshaler；同时 `ICapeUtilities` 的 COM vtable 必须保持 CAPE-OPEN PIA 兼容顺序：`parameters get -> simulationContext set -> Initialize -> Terminate -> Edit`，否则 DWSIM 调用 `Initialize()` 会误入 `SimulationContext` setter
- `DWSIM / COFE` 当前均已验证可发现、放置当前 PMC，并连接 `Feed / Product` material streams；COFE 关闭 case 的 material object release warning 已消失
- 端口连接当前保留 material object 的 `ICapeIdentification` 快照和连接期间的 live PME material object 引用；后者用于短生命周期读取 Feed material 与写回 Product material，并在 Disconnect / Terminate 时释放本 UnitOp 持有的 COM RCW
- DWSIM 真实调用顺序要求 `ICapeUtilities` 前序 vtable 保持 `Parameters get -> SimulationContext set -> Initialize -> Terminate -> Edit`；COFE 需要的 `SimulationContext` getter 保留在 `Edit` 之后的同一 `DispId(2)` late-bound getter
- 注册计划当前已补齐 `Consumes Thermodynamics`、`Supports Thermodynamics 1.0` 与 `Supports Thermodynamics 1.1` 三个 CAPE-OPEN implemented categories，作为 DWSIM 画布接受条件 probe；这不代表 MVP 已实现 Thermo PMC 或第三方 property package 加载
- 对照本地 DWSIM `CapeOpenUO.GetParams()` 后，当前已确认 DWSIM 要求 `myparms.Item(i)` 返回的参数对象本身直接支持 `ICapeIdentification`、`ICapeParameterSpec`、type-specific spec 与 `ICapeParameter`；因此当前已让 parameter placeholder 同时实现 `ICapeParameterSpec` 与 `ICapeOptionParameterSpec`，并在 contract test 中锁住该 DWSIM-style enumeration 形状
- DWSIM 日志中的 `AutomaticTranslation.AutomaticTranslator.SetMainWindow(...)` `NullReferenceException` 来自主窗口 extender 初始化路径，发生在 RadishFlow UnitOp activation 前；当前只作为宿主侧启动噪声记录，不作为 RadishFlow CAPE-OPEN blocker

同日最终沿 Product material publication 与质量衡算继续收敛，当前进一步结论如下：

- `DWSIM / COFE` 在 water/ethanol 复验样例下均已能完成 `Validate / Calculate` 并收敛；COFE 先前的 outlet not flashed 报错已通过 Product material 写回与 CAPE-OPEN 1.1 `CalcEquilibrium(TP)` 解决
- COFE 侧后续 mass balance 红字来自自包含 RadishFlow flowsheet JSON 与 PME `Feed` material object 不一致；当前 `UnitOp.Mvp` 已在计算前安全读取 connected `Feed` material 的 `temperature / pressure / totalFlow / fraction / compound list`，并临时覆盖 native boundary input 后再调用 Rust solve
- 当前新增 water/ethanol 样例 package 与 flowsheet，作为 `DWSIM / COFE` 人工 PME 复验的统一输入；对应 contract tests 已覆盖 feed material overlay 与 product `totalFlow / fraction` 发布一致性

基于上述进展，后续优先计划进一步对齐为：

- 后续新增 PME 复验时，继续按 `examples/pme-validation/` 模板沉淀正式验证记录
- 继续防止 `SimulationContext` getter、DWSIM vtable 顺序和 parameter placeholder 多接口暴露出现回归
- `current-user` 下的标准 `TypeLib` 注册方案当前已落到代码；若必须评估 `local-machine`，后续只作为次选策略单独判断
- 不再把问题粗略归为 `TypeLib` 注册缺失，也不再继续盲补 COFE simulation context optional interfaces；后续若再出现 PME 侧失败，应优先按参数配置、material 读写、flash、report 或持久化调用点分类记录
