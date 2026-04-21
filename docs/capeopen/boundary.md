# CAPE-OPEN Boundary

更新时间：2026-04-21

## 边界目标

该文档用于冻结 Rust Core 与 `.NET 10` CAPE-OPEN 适配层之间的边界，避免 COM 语义反向污染 Rust 核心。

## 第一阶段原则

第一阶段 CAPE-OPEN 边界必须遵守以下原则：

- Rust 不直接处理 COM
- COM 和 CAPE-OPEN 适配全部放在 `.NET 10`
- 第一阶段只导出自有 Unit Operation PMC
- 第一阶段不支持加载第三方 CAPE-OPEN 模型
- `.NET 10` 负责把 Rust 错误映射为 CAPE-OPEN/ECape 语义

## 规范真相源

RadishFlow 的 CAPE-OPEN 适配层不以任何单个示例项目为蓝本，而以官方规范为唯一真相源，并按“行为语义”和“二进制接口形状”分层校准：

- 官方 CAPE-OPEN PDF 规格书与对应 errata / clarifications 是行为语义真相源
- 官方 IDL、Type Libraries 与 Primary Interop Assemblies 是 COM 接口形状、GUID、签名与 marshalling 真相源
- 官方安装包与已发布接口分发版本可作为本地校准和互操作验证输入，但不改变本仓库当前阶段边界
- 官方示例代码、历史参考实现与外部开源项目只作参考，不作为 RadishFlow 的设计真相源

这意味着：

- 对外暴露给 PME 的 COM / CAPE-OPEN 面，必须尽量严格对齐标准接口、生命周期、调用顺序、异常语义和注册类别
- 对内的领域模型、求解接口、FFI 输入输出和状态机，仍保持 RadishFlow 自主设计，只要不破坏标准兼容面
- 不能因为某个示例项目里“顺手带了某种写法”，就把非标准行为、额外属性或历史包袱直接带入正式接口

## 实现策略

当前阶段 CAPE-OPEN 相关实现遵守以下分层策略：

- Rust Core 只负责对象模型、物性、闪蒸、求解和稳定 ABI，不承载 COM 语义
- `rf-ffi` 只暴露句柄、基础数值、UTF-8 字符串、JSON 快照和稳定错误码
- `RadishFlow.CapeOpen.Interop` 负责沉淀标准接口骨架、GUID、HRESULT 与 ECape 语义契约
- `RadishFlow.CapeOpen.Adapter` 负责把 Rust ABI 收口为 .NET 可消费能力，并映射到 CAPE-OPEN 语义
- `RadishFlow.CapeOpen.UnitOp.Mvp` 负责最小自有 Unit Operation PMC 骨架，不提前扩张到完整 Thermo PMC 或第三方模型加载

对外与对内的自由度边界应明确为：

- 对外接口不乱加非标准语义
- 对内可以保留 RadishFlow 自己的状态机、配置模型、诊断结构和调用编排
- 一切内部抽象都必须隐藏在标准 CAPE-OPEN 兼容面之后，而不是反向污染 Rust 核心

## Rust 与 .NET 的运行时边界

Rust 与 `.NET 10` 之间的正式边界应保持简单稳定：

- 句柄
- 基础数值
- UTF-8 字符串
- JSON 快照
- 明确的错误码

当前第一版 `rf-ffi` 应进一步冻结为以下约束：

- 对象跨边界一律优先使用句柄式生命周期，不直接传递 Rust 结构体
- 字符串跨边界一律使用 UTF-8 编码，并明确由哪一侧负责分配与释放
- 数组跨边界只允许使用“指针 + 长度”形式，并明确只读/可写与所有权规则
- 复杂配置、求解输入输出快照和可扩展元数据优先通过 JSON 传递
- 错误先在 Rust 内部表达为稳定错误类型，再映射为错误码与可选诊断文本

截至 2026-04-14，`rf-ffi` 已开始落地第一版最小运行时边界，当前已实现并冻结以下最小 ABI：

- `engine_create`
- `engine_destroy`
- `engine_last_error_message`
- `engine_last_error_json`
- `rf_string_free`
- `flowsheet_load_json`
- `property_package_load_from_files`
- `property_package_list_json`
- `flowsheet_solve`
- `flowsheet_get_snapshot_json`
- `stream_get_snapshot_json`

当前这版 ABI 的额外口径为：

- 输入字符串使用 `pointer + length` 传入，解释为 UTF-8
- 输出字符串由 Rust 侧分配为 UTF-8 C string，并统一通过 `rf_string_free` 释放
- 最近一次错误当前同时支持文本导出和结构化 JSON 导出，后续 `.NET` 适配层不必只靠错误文本分支
- `flowsheet_load_json` 当前加载 `StoredProjectFile` JSON
- `property_package_load_from_files` 当前允许把本地 `manifest.json + payload.rfpkg` 注册到 engine 内的 package registry
- `property_package_list_json` 当前导出 engine 内可用 package manifest 列表，供上层列包与选包
- `flowsheet_solve` 当前按 `package_id` 选择物性包，并把最新 `SolveSnapshot` 留在 engine 内
- `flowsheet_get_snapshot_json` 当前导出最近一次成功求解的整份 `SolveSnapshot` JSON
- `stream_get_snapshot_json` 当前从最近一次成功求解的 `SolveSnapshot` 导出单股流体 JSON
- 返回状态码当前分为两层：FFI 前置错误（如空指针、非法 UTF-8、未加载 flowsheet / 未生成 snapshot）与 `rf_types::ErrorCode` 映射的内核错误；结构化错误 JSON 当前会额外带出 `ffiStatus`、`code`、`diagnosticCode`、`relatedUnitIds`、`relatedStreamIds` 与 `relatedPortTargets`

当前这版运行时仍是最小实现，额外明确以下暂时约束：

- engine 当前内置一份与仓库示例 flowsheet 对齐的 demo property package，用于打通 Rust Core 调用链
- engine 当前同时允许从本地 `manifest/payload` 文件注册真实 package；相同 `package_id` 会覆盖当前 registry 中已有条目
- 当前还未引入 auth cache、本地缓存目录、COM 注册流程或 CAPE-OPEN 接口编排
- 当前 `rf-ffi` 仍已导出整份 `SolveSnapshot` JSON 与单股 stream snapshot JSON，但 `UnitOp.Mvp` 对外结果面已先收口为两条最小契约：成功时是 `status / summary / diagnostics`，失败时是 `error / requestedOperation / nativeStatus / summary`；不再直接把整份 snapshot JSON 或 native error JSON 作为 PMC 公开结果面
- 当前 `UnitOp.Mvp` 之上又已补出统一只读查询面 `GetCalculationReport()`，把“尚无结果 / 最近成功 / 最近失败”收口到单一 report DTO，供后续最小 host / PME 只读消费面复用
- 在该统一查询面之上，当前又补出 `GetCalculationReportState()` 与 `GetCalculationReportHeadline()` 两条最小标量元数据入口，让后续最小 host / PME 可直接读取报告状态与标题，不必先消费自定义 DTO
- 在该元数据面之上，当前又补出 `GetCalculationReportDetailKeyCount()`、`GetCalculationReportDetailKey(int)` 与 `GetCalculationReportDetailValue(string)` 这一组最小 detail 键值读取入口，让后续最小 host / PME 既可枚举稳定 detail key，又可按 key 读取值，而不必再从展示文本里反解析 `status`、`highestSeverity`、`diagnosticCount`、`requestedOperation` 或 `nativeStatus`
- 当前又已把 stable detail key 清单正式冻结到公开 catalog `UnitOperationCalculationReportDetailCatalog`，用来声明 success / failure 两条路径的 canonical key 顺序，避免宿主只能靠 README 或周志文本猜测 key 名字
- 在该 report DTO 之上，当前又补出 `GetCalculationReportLines()` 与 `GetCalculationReportText()` 两条最小宿主可显示文本面，优先把 headline/detail lines 的拼接责任留在 PMC 内部，而不是继续让最小 host / PME 自己重复组织显示字符串
- 在该文本面之上，当前又补出 `GetCalculationReportLineCount()` 与 `GetCalculationReportLine(int)` 两条标量读取入口，让后续最小 host / PME 可以按“line count + line(index)”逐步读取报告文本，而不提前要求消费自定义 DTO 或整段拼接文本
- 在上述公开 report API 之上，当前又补出 `UnitOperationHostReportReader -> UnitOperationHostReportPresenter -> UnitOperationHostReportFormatter` 三级库内 helper，把最小宿主读取、展示模型与分区格式收口为稳定复用层，而不是继续给 PMC 主类追加 convenience accessor
- 在上述 report/configuration/action-plan/port-material/execution readers 之上，当前又补出 `UnitOperationHostSessionReader`，把统一宿主整体视图继续收口到库内；最小 host 现在可以一次读取 configuration、action plan、port/material、execution 与 report，并直接复用 canonical session state 与 `IsReadyForCalculate / HasBlockingActions / HasCurrentResults / RequiresCalculateRefresh / HasFailureReport / RecommendedOperations` 这类摘要，而不必在外部再协调多次读取并拼一层私有 session state
- 在这组 readers 之上，当前又补出 `UnitOperationHostViewReader`，把 configuration/action plan/port-material/execution/report/session 六块正式 host view 继续收口到单一快照，避免 action execution、validate、calculate 三条宿主路径各自重复补读和拼装
- 在 action plan 与 action execution dispatcher 之间，当前又补出 `UnitOperationHostActionExecutionRequestPlanner`，把“哪些 action 已能执行、哪些 action 仍缺 parameter value 或 port object、哪些 action 只是 lifecycle 提示或 terminal unsupported 状态”收口成正式 request plan；这层只消费宿主显式提供的输入，不替宿主选择 flowsheet JSON、package id、连接对象或生命周期调用时机
- 在 request plan 与单次 action execution 之上，当前又补出 `UnitOperationHostActionExecutionOrchestrator` 与正式 `FollowUp` 模型，把“执行 ready requests 后宿主下一眼该看什么、下一步该补输入/做 validate/做 calculate 还是只剩 lifecycle/terminated”继续收口为正式 result；这层统一返回 request plan、execution batch outcome、刷新后的 host view，以及 `LifecycleOperation / ProvideInputs / Validate / Calculate / CurrentResults / Terminated` 六类 follow-up，但仍不负责 `Initialize / Validate / Calculate / Terminate`
- 在 action execution 之外，当前又补出 `UnitOperationHostValidationRunner` 与 `UnitOperationHostCalculationRunner`，把 `Validate()` / `Calculate()` 之后的正式 `host view + follow-up` 一并收口到库内；最小 host 现在不必在调用后继续手工补读 `session/report/execution` 再判断下一步
- 在 validation/calculation outcome 之上，当前又补出 `UnitOperationHostRoundOrchestrator`、`UnitOperationHostRoundRequest`、`UnitOperationHostRoundOutcome` 与 `UnitOperationHostRoundStopKind`，把“可选 action execution -> 可选 supplemental object mutations -> 可选 validate -> 可选 calculate”这一条最常见宿主 round 主路径继续收口为正式结果；这层统一返回 initial/final host views、可选 phase outcome、最终 follow-up 与 stop kind，但仍不扩张成完整 smoke driver 或 PME 生命周期框架
- 在独立 `SampleHost` 之上，当前又补出 `PmeLikeUnitOperationHost / PmeLikeUnitOperationSession / PmeLikeUnitOperationInput` 薄宿主入口，把外部宿主最常见的“打开 unit session -> 提供 flowsheet/package/port material object -> 执行正式 host round -> 读取正式结果 -> 关闭 session”整理为更接近 PME host 的最小接线蓝本；这层只复用正式 reader / planner / round outcome，不复用 `SmokeTests` driver DSL
- `UnitOp.Mvp` 内部当前又已把 `_initialized / _terminated / _disposed` 三布尔状态收口为 `UnitOperationLifecycleState`，并将 `EvaluateValidation()` 与 `Calculate()` 各自拆成显式阶段 helper；validation/calculation/report 的状态迁移也已统一进入正式 transition helper，避免宿主主线继续推进时出现隐式状态漂移

当前不允许在边界上直接传递以下内容：

- COM 接口对象
- `IDispatch`
- `VARIANT`
- `SAFEARRAY`
- 复杂对象图

## 当前仓库阶段约束

截至 2026-04-16，`.NET 10` 适配层已从“纯目录占位”推进到“薄适配 + 冒烟闭环 + 最小互操作语义骨架”，但仍未进入完整 COM/PMC 运行时。

当前允许推进的内容：

- 文档
- 目录结构
- README 说明
- 依据官方规格书、errata、IDL、TLB、PIA 对最小接口骨架和异常语义做持续校准
- 最小 `.NET 10` `LibraryImport` / PInvoke 薄封装
- 最小 `.NET 10` smoke console，用于验证 `rf-ffi` 调用闭环
- `RadishFlow.CapeOpen.Interop` 中最小 `ICapeIdentification`、`ICapeUtilities`、`ICapeUnit` 接口骨架
- `ICapeUtilities` / `ICapeUnit` 的最小 `IDispatch` marshalling 形状校准
- 已确认 CAPE-OPEN interface/category GUID 常量唯一来源
- 第一版 ECape 异常基类、HRESULT 常量与语义化异常类型
- `ECapeRoot` / `ECapeUser` / `ECapeBadInvOrder` / `ECapeBadCOParameter` 等最小异常契约、IID 与 DispId 校准
- `RadishFlow.CapeOpen.UnitOp.Mvp` 中不含注册的最小 PMC 类骨架、状态机与内部配置入口
- `UnitOp.Mvp` 中用于 `Ports` / `Parameters` 的最小占位对象集合，以及基于对象状态的 `Validate()` 前置检查
- `UnitOp.Mvp` 中经由 `RadishFlow.CapeOpen.Adapter` 调用 `rf-ffi` 的最小 `Calculate()` 求解接线，以及基于 native snapshot JSON / error JSON 材料化出的最小成功结果契约与失败摘要契约
- `UnitOp.Mvp` 中基于上述结果对象继续收口出的最小只读 result/report access，以及建立在其上的标量元数据入口、可枚举 detail 键值读取入口、最小文本导出面与标量逐行读取入口，不要求外部宿主自己拼装成功结果、失败摘要或 headline/detail 文本
- 建立在公开 report API 之上的库内宿主消费 helper，以及基于该 helper 的最小 sectioned host report 口径
- 建立在 configuration/action-plan/port-material/execution/report 正式快照之上的库内统一 host session snapshot，用于减少外部宿主在边界层重复汇总整体状态
- 建立在 action plan 之上的 action execution request planning helper，用于把宿主输入显式规划为可执行 request batch，并报告 missing inputs / lifecycle-only / unsupported action；该 helper 是库内正式边界，区别于 smoke driver 的完整生命周期编排
- 建立在 action execution / validation / calculation outcome 之上的 host round orchestration helper，用于把最常见的宿主 round 主路径收口到统一结果；该 helper 是库内正式边界，但不替代 smoke driver、PME adapter 或完整工作流框架
- 独立的 `RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost` console，用于演示外部 host 如何只复用正式 `host view / request planner / round outcome / session-report-execution-material readers`，而不依赖 smoke driver
- `SampleHost` 内的 PME-like 薄宿主 session，用于把上述正式消费路径整理成更接近真实 PME host 的入口形状；它仍不做 COM 注册、不驱动外部 PME、不加载第三方 CAPE-OPEN 模型，也不把完整 PME 生命周期框架提前塞进 `UnitOp.Mvp`
- `UnitOp.Mvp` 中的 `UnitOperationComIdentity`，用于冻结自有 MVP Unit Operation PMC 的 `CLSID / ProgID / Versioned ProgID` 与 COM-visible class 元数据
- `RadishFlow.CapeOpen.Registration` 的 dry-run / preflight console，用于输出组件注册计划、CAPE-OPEN categories 与已实现接口清单；当前只读输出，不写注册表、不注册 COM、不启动 PME
- `SmokeTests` 中更接近真实宿主的最小 driver 路径，用于固定 `Initialize -> 配参数 -> 连端口 -> Validate -> Calculate -> 读结果 -> Terminate` 正式调用顺序、最小必需输入与 `InvocationOrder / Validation / Native` 三类失败分类
- `RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests` 这种不依赖外部 NuGet 测试框架的库侧 contract baseline，用于锁定 `UnitOp.Mvp` 的行为语义，而不是把这部分契约只留在 console 输出里

当前暂不推进的内容：

- COM host 注册细节
- 完整 ECape 接口/异常面与所有标准 IID 校准
- 执行型 COM 注册 / 反注册工具
- 端口集合、参数集合、报告接口与 `Collection/Parameter/UnitPort` 语义的完整 CAPE-OPEN 实现
- PME 互调测试代码
- 将当前验证型 `UnitOperationSmokeHostDriver` 直接上移为 `UnitOp.Mvp` 正式库 API；在它被证明不仅服务当前 smoke 验证样板之前，先继续保留在 `SmokeTests`
- 把 COBIA 作为当前主线运行时或因此提前改写既定 COM 兼容路径

当前推荐的最小外部 host 路径为：

- 宿主自行负责 `Initialize()` 与 `Terminate()`
- 通过 `UnitOperationHostViewReader.Read(...)` 或 `UnitOperationHostSessionReader.Read(...)` 读取当前只读视图
- 通过 `UnitOperationHostActionExecutionRequestPlanner.Plan(...)` 准备 parameter values / port objects 到 ready requests 的映射
- 如需写入不属于 blocking action plan 的宿主配置，通过 `UnitOperationHostRoundRequest.SupplementalMutationCommands` 注入
- 最终优先通过 `UnitOperationHostRoundOrchestrator.Execute(...)` 收口 `ready actions -> supplemental mutations -> Validate -> Calculate`
- 如果需要一个更接近 PME host、但仍不依赖真实 COM/PME 环境的入口样板，优先参考 `SampleHost` 中的 `PmeLikeUnitOperationHost` / `PmeLikeUnitOperationSession`；它已经把上述步骤包成单个薄 session，而没有引入 smoke DSL 或外部环境副作用

## 对 Rust Core 的约束

为了给后续 `rf-ffi` 留出干净边界，Rust Core 当前应坚持以下约束：

- 领域模型不带 COM 类型
- 错误先在 Rust 内部表达为统一错误类型
- 输出结果优先落在普通 Rust 数据结构与 JSON 友好结构上
- 单元与流股对象先面向内核求解，不直接面向 CAPE-OPEN 接口建模
- 第一版 FFI 接口优先围绕 `engine` / `flowsheet` / `stream snapshot` 等稳定能力展开，不提前暴露过细的内部实现细节

## 结论

第一阶段真正要做的是“让 `.NET 10` 适配层按官方规范对外兼容 CAPE-OPEN，并稳定调用 Rust Core”，不是“复制示例代码”或“让 Rust 看起来像 COM 组件”。  
这条边界如果现在守不住，后面 FFI、PMC 和 UI 都会被一起拖复杂。
