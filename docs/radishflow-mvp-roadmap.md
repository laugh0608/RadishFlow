# RadishFlow MVP 开发路线图

更新时间：2026-04-26

## 文档目的

本文档用于把 `RadishFlow` 的第一阶段目标拆成可执行的开发路线。

本文档只覆盖 MVP，不覆盖以下内容：

- 第三方 CAPE-OPEN 模型导入
- 完整 Thermo PMC
- recycle 全功能收敛
- 复杂 GUI 风格打磨
- 大规模组件数据库

## MVP 目标声明

MVP 的目标固定为：

构建一个以 Rust 为核心、以 Rust UI 为主界面、以 .NET 10 暴露 CAPE-OPEN Unit Operation PMC 的最小稳态流程模拟闭环，并让至少一个自有单元模型可被外部 PME 识别与调用。

补充系统口径：

- 桌面端主线固定为 Rust 客户端，负责 UI、求解、项目文件读写与本地授权缓存使用
- 认证、授权、离线租约、受控物性包清单与下载票据由外部 `ASP.NET Core / .NET 10` 控制面承担
- 派生资产下载优先走对象存储 / CDN / 下载网关，不把控制面 API 设计成长期大文件出口
- 桌面端最终交付形态默认为“压缩包展开后直接运行”，不以单文件可执行产物为阶段目标

## MVP 功能边界

MVP 建议只包含以下能力：

- 二元体系
- 最小物性参数集
- 简化热力学模型
- `TP Flash`
- 流股对象
- 单元模块：`Feed`、`Mixer`、`Heater/Cooler`、`Valve`、`Flash Drum`
- 无回路流程或极简回路
- JSON 项目格式
- 一个最小 Rust 桌面工作台
- 一个可注册的 CAPE-OPEN Unit Operation PMC

## 不做项

MVP 阶段明确不做：

- 加载外部 CAPE-OPEN 单元
- 加载外部 CAPE-OPEN 物性包
- 完整 Thermodynamics 1.0/1.1 主机兼容
- 完整物性数据库
- 动态模拟
- 复杂报表系统
- 多套热力学包切换

## 总体里程碑

建议把 MVP 分成 5 个里程碑：

1. `M1`：仓库与基础骨架初始化
2. `M2`：二元体系 `TP Flash` 核心跑通
3. `M3`：最小稳态流程闭环跑通
4. `M4`：Rust FFI 与 .NET 10 适配层打通
5. `M5`：外部 PME 识别并调用自有 PMC

## UI 协作闸口

在 `apps/radishflow-studio` 从“状态边界、宿主适配与布局契约冻结”进一步进入“真实 UI 界面和交互逻辑设计”之前，当前规划额外补充一条协作约束：

- 可以继续先推进 GUI-facing 状态模型、宿主入口、平台适配、布局状态机与组件 DTO
- 一旦开始落真实界面层决策，例如窗口结构、面板编排、控件层级、视觉表达、交互流、用户操作反馈和较重的 UI 逻辑取舍，必须先向用户同步当前方向、阶段目标和关键取舍点
- 在该阶段默认保留用户显式干预入口，不把界面和交互方向长期静默推进到既成事实后再回头确认
- 当 GUI-facing 宿主边界、timer glue、窗口布局契约与首版交互流已经稳定，且准备从当前单原生窗口 GUI 壳进一步进入更完整的真实界面或多窗口宿主阶段时，应显式复盘 `egui` 与其他 UI 框架的适配性，再决定是否继续沿用当前路线

这条约束的目的，不是阻塞 Studio 主线，而是把“架构边界冻结”和“产品交互方案拍板”明确分开，避免在内核与宿主边界尚持续演进时，过早把 UI 体验层方案一次性做死。

## M1：仓库与基础骨架初始化

### 目标

建立 `RadishFlow` 新仓库与最小开发骨架。

### 任务

- 创建 `RadishFlow` 新仓库
- 初始化 Rust workspace
- 创建核心 crates
- 创建 `apps/radishflow-studio`
- 创建 `.NET 10` 适配层解决方案目录
- 冻结外部 `.NET 10` 控制面的职责边界与最小 API 契约
- 初始化根文档与 `docs/README.md`
- 建立基础脚本目录与测试目录

### 建议优先创建的 crate

- `rf-types`
- `rf-model`
- `rf-thermo`
- `rf-flash`
- `rf-unitops`
- `rf-flowsheet`
- `rf-solver`
- `rf-ffi`

### 退出标准

- 新仓库可正常克隆
- Rust workspace 可成功 `cargo check`
- .NET 适配层目录已存在并可打开
- 文档已说明 MVP 边界与目录职责
- 外部 `.NET 10` 控制面边界与桌面客户端职责已经冻结

## M2：二元体系 TP Flash 核心跑通

### 目标

先打通最小的热力学与闪蒸核心。

### 任务

- 定义组分与基础物性结构
- 支持二元体系输入
- 实现 Antoine 饱和蒸气压
- 实现简化 `K` 值逻辑
- 实现 Rachford-Rice
- 实现 `TP Flash`
- 输出相分率、汽液相组成、基础焓值
- 建立黄金样例测试

### 推荐 crate 分工

- `rf-types`：基础枚举、ID、相标签
- `rf-model`：流股状态对象
- `rf-thermo`：组分参数与热力学模型
- `rf-flash`：闪蒸算法

### 退出标准

- 至少一个二元样例可稳定计算
- 数值结果可回归测试
- API 已可供单元模块调用
- `tests/thermo-golden` 与 `tests/flash-golden` 已建立首批黄金样例

## M3：最小稳态流程闭环跑通

### 目标

构建最小 flowsheet 求解能力。

### 任务

- 定义端口与连接关系
- 实现 `Feed`
- 实现 `Mixer`
- 实现 `Heater/Cooler`
- 实现 `Valve`
- 实现 `Flash Drum`
- 实现流程图完整性校验
- 实现顺序模块法求解
- 建立至少一个最小流程示例
- 形成至少一个可从 `*.rfproj.json` 加载到求解输出的端到端闭环样例

### 推荐示例流程

- `Feed -> Heater -> Flash Drum`
- `Feed -> Valve -> Flash Drum`
- `Feed1 + Feed2 -> Mixer -> Flash Drum`

### 推荐 crate 分工

- `rf-unitops`
- `rf-flowsheet`
- `rf-solver`
- `rf-store`

### 当前收口口径

- `rf-unitops` 第一轮统一接口先围绕标准 `MaterialStreamState` 输入输出与必要热力学服务注入
- `Mixer` 当前先固定为两进一出，`Flash Drum` 当前先固定为一进两出，优先建立第一条可求解闭环
- `Heater/Cooler` 与 `Valve` 当前先沿用“一进一出”最小接口，并以 outlet 流股模板承载当前阶段目标状态设定
- `rf-flowsheet` 第一轮只做 canonical material 端口签名、流股存在性与“一股一源一汇”校验，拓扑排序和顺序模块调度继续留给 `rf-solver`
- `rf-solver` 第一轮先只支持无回路、内建单元和标准材料流股执行，当前已覆盖 `Feed1 + Feed2 -> Mixer -> Flash Drum`、`Feed -> Heater -> Flash Drum` 与 `Feed -> Valve -> Flash Drum`
- `rf-solver` 当前已补最小求解诊断层：`SolveSnapshot` 至少包含 summary、仓库级 diagnostics 和逐步执行 step 明细；失败路径至少带 step 序号、unit id / kind 与 inlet stream 上下文
- `examples/flowsheets` 当前应维护至少三条可直接从 `*.rfproj.json` 载入并求解的示例项目，作为内核闭环回归基线
- `tests/rust-integration` 当前应作为仓库级 Rust 集成测试入口，覆盖“加载项目 -> 求解 -> 读取结果”的示例流程回归
- `rf-ui` 当前已具备把 `rf-solver::SolveSnapshot` 回写为 UI 层结果快照的稳定映射
- `rf-ui` 当前已补出 `RunPanelState`，并由 `AppState` 在文档提交、快照写入、模式切换、失败记录和日志追加后自动刷新运行栏摘要
- `rf-ui` 当前已把运行栏按钮模型冻结为 `RunPanelCommandModel`，让主按钮选择和 `Run/Resume/Hold/Active` 的启用逻辑正式留在 UI 层
- `apps/radishflow-studio` 当前已具备 `StudioAppFacade -> WorkspaceRunCommand -> WorkspaceSolveService -> solver_bridge` 四级应用层入口，可基于 `PropertyPackageProvider` 或本地 `StoredAuthCacheIndex` 执行真实 solve
- Studio 当前把运行触发先区分为 `Manual` / `Automatic`，并把 `SimulationMode`、`pending_reason` 与默认 `snapshot_id` / `sequence` 生成收口到应用层
- Studio 当前默认包选择采取保守策略：只有在唯一候选包明确时才自动选中，多包场景必须显式指定 package
- Studio 当前又已形成 `StudioGuiHost / StudioGuiDriver / StudioGuiSnapshot / StudioGuiWindowModel / StudioGuiWindowLayoutState` 这一条 GUI-facing 宿主与窗口布局契约
- Studio 当前窗口布局状态已覆盖 `panel dock_region/stack_group/visibility/collapsed/order`、stack active tab、region 内 stack placement、`center_area`、`region_weights`、多窗口 `layout scope` 与 GUI-facing `drop target` 摘要推导
- Studio 当前也已把 tab 展示角色冻结到 `StudioGuiWindowPanelLayout`，显式区分 `Standalone / ActiveTab / InactiveTab`，不再让真实 GUI 私自猜测 tab 化 panel 的主次呈现
- Studio 当前也已把 tab strip 交互收口到正式 mutation，至少覆盖 active tab 切换、前后循环、stack 内重排与 unstack，避免真实 GUI 再另写一套 tab strip 私有状态机
- Studio 当前又已把 drop preview 查询从 layout 层只读推导前推到 `StudioGuiWindowDropTargetQuery -> StudioGuiHost / StudioGuiDriver` 显式入口，未来真实 GUI 可直接按 `window_id + hover/anchor/placement` 请求预览
- Studio 当前又已把 drop preview query 结果扩成 `preview_layout_state / preview_window`，让真实 GUI 可直接消费 hover 预览态，而不只拿到 target 摘要
- Studio 当前又已把 drop release 也前推到同一套 query 词汇，新增 `ApplyWindowDropTarget / WindowDropTargetApplyRequested`，让 hover/query 与 release/apply 共用同一份 GUI-facing 契约
- Studio 当前又已补出轻量 drop preview 会话态，新增 `SetWindowDropTargetPreview / ClearWindowDropTargetPreview` 与对应 driver 事件；host 会非持久化缓存当前 hover 预览，并经由 `StudioGuiSnapshot / StudioGuiWindowModel.drop_preview` 对外暴露
- Studio 当前又已把 `drop_preview` 继续推进为正式 presentation，直接携带 `preview_layout + changed_area_ids`，让真实 GUI 不必再自己比对当前/预览两份布局 state
- Studio 当前又已把 `drop_preview` 继续推进为 overlay presentation，直接携带目标 region/stack group、tab 插入位、目标 stack tabs 与高亮 area 集，让真实 GUI 不必再从底层 `drop_target` 手工拆提示语义
- 第一版 `eframe/egui` GUI 壳当前也已直接消费这份 `drop_preview.overlay`，把局部插入条、anchor 顶线、新 stack 占位、target-anchored 浮动 overlay 与局部 hint pill 画在真实落点，而不是继续把预览提示留在标题栏摘要里
- 当前 GUI 壳仍明确停留在“单原生窗口承载逻辑窗口切换”的阶段，不在这一轮 roadmap 内扩张到多原生窗口宿主
- Studio 当前又已补出 `StudioGuiNativeTimerRuntime`、`StudioGuiPlatformHost` 与 `StudioGuiPlatformTimerDriverState` 这一条 GUI-facing 原生 timer 宿主 glue；真实桌面框架后续不必在入口层重写“逻辑 timer -> 平台 request -> native timer id -> callback 回灌”整套桥接
- Studio 当前平台 timer glue 已继续冻结为三层正式消费面：
- `StudioGuiPlatformTimerRequest` / `StudioGuiPlatformTimerCommand`：宿主比较前后 pending binding 后产出平台需要执行的 `Arm / Rearm / Clear`
- `acknowledge_platform_timer_started(...)` / `acknowledge_platform_timer_start_failed(...)`：平台 start success/failure ack 的正式结果面与 cleanup follow-up
- `dispatch_native_timer_elapsed_by_native_id(...)`：平台按 `native_timer_id` 回灌 callback 时的 `Dispatched / IgnoredUnknown / IgnoredStale` 正式 outcome
- Studio 当前又已在其上继续冻结三类更接近真实宿主的组合结果：
- callback / due drain batch：`dispatch_native_timer_elapsed_by_native_ids(...)`、`dispatch_due_native_timer_events_batch(...)` 与对应执行型 batch 结果
- async round：`process_async_platform_round(...)`，统一消费一轮 `started_feedbacks -> start_failed_feedbacks -> native_timer_ids -> due_at`
- executed async round：`process_async_platform_round_and_execute_actions(...)`，直接按 host 冻结的 `follow_up cleanup -> timer request` 顺序执行并返回最终 `snapshot/window`
- `apps/radishflow-studio/src/main.rs` 当前也已切到消费上述 executed async round 结果面，最小宿主样例不再手工拆 callback batch、cleanup 顺序或 timer request 执行逻辑
- Studio 当前窗口布局已独立持久化到 `<project>.rfstudio-layout.json` sidecar，并从基于运行时 `window_id` 的 key 收口到基于 `window_role + layout_slot` 的稳定 key

### 退出标准

- 能打开一个 JSON 示例流程
- 能完成一次完整求解
- 能输出每股流体的基本状态结果
- 已具备最小集成测试，能覆盖“加载项目 -> 求解 -> 读取结果”的端到端数据流
- 仓库级验证入口 `scripts/check-repo.ps1` / `scripts/check-repo.sh` 已自动覆盖这些集成测试
- Studio 应用层已具备显式运行命令 / 服务边界，不要求最终桌面按钮已接通，但不再由 UI 直接拼接底层 provider/solver 调用

## M4：Rust FFI 与 .NET 10 适配层打通

### 目标

让 Rust 核心可被 .NET 10 适配层调用。

### 任务

- 定义稳定的 C ABI
- 设计句柄式对象生命周期
- 处理字符串与错误码返回
- 定义最小 JSON 快照接口
- 在 `.NET 10` 中建立 `RadishFlow.CapeOpen.Adapter`
- 完成 PInvoke 封装
- 建立最小 smoke / 互调验证入口

### 推荐暴露接口

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

### 退出标准

- .NET 可成功调用 Rust 动态库
- 能从 .NET 获取 package registry、结构化错误与求解结果
- 有最小 smoke 或自动化验证覆盖对象创建、包注册与求解调用

## M5：外部 PME 识别并调用自有 PMC

### 目标

交付第一版可互操作的 CAPE-OPEN Unit Operation PMC。

### 任务

- 建立 `RadishFlow.CapeOpen.Interop`
- 建立 `RadishFlow.CapeOpen.UnitOp.Mvp`
- 设计最小参数与端口映射
- 冻结最小 host-facing 只读语义：configuration/action plan/port-material/execution/session
- 实现 `ICapeIdentification`、`ICapeUtilities`、`ICapeUnit` 相关能力
- 建立注册工具 `RadishFlow.CapeOpen.Registration`
- 完成 COM host 注册流程
- 在目标 PME 中做人工验证

### 建议验证内容

- PME 能发现组件
- PME 能实例化组件
- PME 能读取参数
- PME 能连接端口
- PME 能触发 `Calculate`
- PME 能得到有效结果或可诊断错误

### 退出标准

- 至少一个目标 PME 成功识别并调用 PMC
- 注册/反注册流程可重复执行
- 文档中已写明验证路径

## 推荐开发顺序

建议严格按以下顺序推进：

1. `M1`
2. `M2`
3. `M3`
4. `M4`
5. `M5`

不要跳过 `M2` 和 `M3` 直接做 CAPE-OPEN 外壳。  
如果内核没有先稳定，后面在 PME 里出现错误时会很难定位到底是内核问题还是 COM 适配问题。

## 当前计划对齐

截至 2026-03-29，M1 阶段中与桌面控制面接线相关的地基项已进一步前推，当前已完成：

- 控制面 `entitlement / manifest / lease / offline refresh` HTTP client
- 控制面 JSON 契约到运行时 DTO 的应用层协议映射
- 按 manifest 申请 lease、下载派生包并回写本地缓存索引的应用层编排

截至 2026-03-30，系统技术口径进一步补充冻结为：

- 远端控制面优先采用 `ASP.NET Core / .NET 10`
- 资产分发优先采用对象存储 / CDN / 下载网关 + 短时票据
- 当前仓库继续只保留客户端侧接线、DTO 与缓存编排，不把服务端代码强行并入 Rust workspace

基于当前状态，下一阶段计划进一步对齐为：

- 今天（2026-03-30）优先细化授权刷新后的 UI 事件流、联网失败提示和离线刷新触发策略
- 随后恢复 `rf-thermo` / `rf-flash` 数值主线，避免地基建设继续挤占核心算法推进
- 在数值主线恢复后，再进入 `rf-solver` 无回路顺序模块法和首个可求解 flowsheet 示例

截至 2026-04-02，M3 与 Studio 应用层又进一步前推，当前已完成：

- `solver_bridge`：把 `PropertyPackageProvider` / 本地 auth cache 与 `rf-solver`、`rf-ui::AppState` 接通
- `WorkspaceSolveService`：把默认 request、手动/自动触发门控与工作区求解分发收口为应用服务入口
- `WorkspaceRunCommand`：把“触发类型 + package 选择”抽成更接近桌面命令层的对象，并冻结当前默认包选择策略
- `StudioAppFacade`：把 auth cache 上下文、运行命令、结果派发摘要和后续异步执行边界收口为明确的桌面应用入口
- `run_studio_bootstrap` + `apps/radishflow-studio/src/main.rs`：把样例项目加载、本地样例 auth cache 构造和 `StudioBootstrapTrigger -> RunPanelWidgetEvent / RunPanelIntent -> run_panel_driver / WorkspaceControlAction -> StudioAppFacade` 链路接成最小桌面进程触发点
- Studio 当前已把 Automatic skip 原因补进 `AppLogFeed`，避免“入口触发但未运行”完全静默
- Studio 当前已把 `StudioWorkspaceRunDispatch` 扩成结构化摘要，统一派发最新 snapshot 摘要和最新日志摘要给入口层消费
- Studio 当前已把 `StudioAppCommand` 扩成显式运行控制命令，覆盖 `RunWorkspace`、`ResumeWorkspace` 与 `SetWorkspaceSimulationMode`
- Studio 当前已补出 `ResumeWorkspace` 这一条 `Hold -> Active` 恢复路径，并把 `simulation_mode` / `pending_reason` 一并纳入结果派发
- Studio 当前已补出 `WorkspaceControlAction` / `WorkspaceControlState` 这一层更接近运行栏 / 状态栏的 UI 入口契约，并在 bootstrap 入口中直接消费
- Studio 当前已把 `WorkspaceControlState` 映射回 `rf_ui::RunPanelState`，从而让 `rf-ui` 正式消费最小运行栏状态对象，而不反向依赖 Studio 类型
- `rf-ui` 当前已补出 `RunPanelIntent` / `RunPanelPackageSelection`，Studio 当前已提供对应映射与分发入口，为后续按钮、菜单或快捷键接线保留稳定接口
- `rf-ui` 当前已不只拥有运行意图类型，还拥有最小按钮模型、文本渲染 DTO、`RunPanelPresentation` 与 `RunPanelWidgetModel`；后续真正视图层接线时不必再重新发明运行栏启用判断、按钮槽位组织、文本布局或按钮激活语义
- 当前最小桌面入口 `run_studio_bootstrap` / `main.rs` 已开始直接消费 `RunPanelWidgetModel`，作为真正的最小运行栏组件/消费层
- Studio 当前也已补出 `RunPanelWidgetEvent -> WorkspaceControlAction` 的分发桥，使最小 widget 交互结果能够回到既有应用命令链路
- Studio 当前也已补出 `run_panel_driver`，把最小运行栏组件驱动流程收口成单独应用层入口
- `run_studio_bootstrap` 当前也已补出 `StudioBootstrapTrigger::{Intent, WidgetPrimaryAction, WidgetAction}`，默认通过 `WidgetPrimaryAction` 驱动这条链路，而不再只验证裸 `RunPanelIntent`
- Studio 当前也已补出 entitlement control / panel / startup preflight / session scheduler / session driver，使 entitlement panel 动作、启动预检、失败退避与下一次检查时机正式留在 Studio 应用层
- Studio 当前也已补出 `EntitlementSessionEvent::{SessionStarted, LoginCompleted, TimerElapsed, EntitlementCommandCompleted}`，让 bootstrap 自动预检和 entitlement 命令完成后的 session state 推进共享统一事件入口
- Studio 当前也已补出 `entitlement_session_host`，把 entitlement session event 与 entitlement panel 动作统一收口为宿主触发入口，并补出 `NetworkRestored` / `WindowForegrounded` 到既有 session tick 语义的映射；host 当前也已能把 `next_check_at` 收口为 timer arm 摘要、宿主建议 notice 与 `Schedule / Reschedule / Keep / Clear` 定时器决策，并进一步聚合成单一 `EntitlementSessionHostSnapshot`；当前又补出 `EntitlementSessionHostContext`，把当前已挂 timer 与上一份 host snapshot 的推进逻辑收回 host 模块，使 bootstrap / `main.rs` 与后续 GUI 宿主不再手写 control plane runtime 分发、多字段拼装或上下文记忆；bootstrap 最小入口当前也已能直接演练 `Reschedule / Keep` 语义而不是每次都从 `Schedule` 起步
- Studio 当前也已补出 `EntitlementSessionHostPresentation / TextView`，把 `main.rs` 仍在分散拼装的 entitlement host schedule/timer/notice 文本输出收回宿主模块，最小入口直接消费正式 host presentation
- Studio 当前也已补出 `EntitlementSessionHostRuntimeOutput / TimerEffect / Runtime`，把 `Schedule / Reschedule / Keep / Clear` 继续收成更贴近桌面宿主的 timer effect，并让 bootstrap 不再直接解释低层 host context
- Studio 当前也已把 bootstrap 最小入口继续收成可复用 session，并开始覆盖多宿主事件序列测试，验证共享 runtime 下 `TimerElapsed / NetworkRestored / WindowForegrounded` 的连续推进语义
- Studio 当前也已补出 `StudioRuntime`，把 bootstrap 内部可复用 session 上提为共享顶层入口，让 `run_studio_bootstrap(...)` 与 `main.rs` 统一走同一条 `StudioBootstrapTrigger -> entitlement host runtime / app facade` 分发链
- Studio 当前也已在 `StudioRuntime` 顶层补出 `StudioRuntimeOutput / StudioRuntimeEffect`，把 entitlement timer effect 正式作为运行时宿主输出暴露出来，后续真实桌面框架不必继续从 bootstrap 报告里翻字段取 effect
- Studio 当前也已给顶层 runtime 补出正式 `StudioRuntimeConfig / Trigger / Report / Dispatch` 命名，让真实宿主可先脱离 `StudioBootstrap*` 命名耦合，再继续推进具体 GUI 接线
- Studio 当前也已给顶层宿主输出补出最小 apply/ack 协议：`StudioRuntimeHostEffect` 现带稳定 `id`、`follow_up trigger` 与 `ack` 状态回报，先把 entitlement timer 接线闭环固定下来
- Studio 当前也已在其上补出 `StudioRuntimeTimerHostCommand`，把 entitlement timer 的宿主动作进一步收成专门 timer 薄层，真实 GUI 不必再先理解 generic host effect 列表
- Studio 当前也已在其上补出 `StudioRuntimeTimerHostState / Transition`，把 timer 当前槽位状态与 stale command 处理显式化，真实 GUI 可以直接照着“apply command -> 更新 state -> ack”接线
- Studio 当前也已进一步补出 `StudioWindowHostState + StudioRuntimeHostPort`，把单窗口宿主实例对 timer state 的创建、替换与销毁清理收成正式容器，最小入口不再手工拼接 runtime、timer state 与 ack
- Studio 当前也已继续把 `StudioRuntimeHostPort` 上提为应用级多窗口宿主容器，并冻结首版所有权规则：entitlement timer 全局只允许一个 owner 窗口持有；owner 销毁时优先转移到剩余窗口，否则 parked 并在下一个 owner 窗口打开时恢复
- Studio 当前也已继续把 GUI adapter 口径补齐，新增 `StudioWindowHostTimerDriverCommand` 与 `StudioWindowHostLifecycleEvent`，把窗口宿主该如何操作原生 timer handle 以及如何回灌生命周期事件进一步固定为正式 host port 契约
- Studio 当前也已继续把原生 timer adapter state/ack 收成独立层，新增 `StudioWindowTimerDriverState / Transition / Ack`，让 GUI 在消费 host port 后只需执行明确的 native timer 操作并回写 handle
- Studio 当前也已继续把两层 adapter 再收为单一 `StudioWindowSession` 会话入口，让未来 GUI 以“窗口会话”而不是“分别操作 host port 和 timer driver state”的方式接线
- Studio 当前也已继续把单窗口会话上提到应用级多窗口入口，新增 `StudioAppWindowHostManager`，把窗口注册表、foreground window 和全局宿主事件路由统一收口
- Studio 当前也已继续把 `StudioAppWindowHostManager` 上提为标准 app host 命令面，新增 `StudioAppWindowHostCommand / Outcome`，让未来 GUI 通过单一入口处理打开/关闭窗口、前后台切换、runtime trigger 与全局事件，而不是外层自己拼调用顺序
- Studio 当前也已继续把 app host 命令面再收成正式顶层容器，新增 `StudioAppHost + StudioAppHostSnapshot`，让 GUI 在执行命令后能直接拿到 registered windows、每个窗口的 role/foreground/timer slot、timer-owner 与 parked timer 快照，而不是再分别查询 manager 与 host port
- Studio 当前也已继续把 app host 输出推进到正式变更边界，新增 `StudioAppHostChangeSet`，让 GUI 在执行命令后可直接消费窗口新增/移除/更新、foreground 迁移、timer-owner 迁移与 parked timer 变化，而不是自行 diff snapshot
- Studio 当前也已继续把 app host 输出进一步收口为正式宿主状态层，新增 `StudioAppHostState + StudioAppHostStore + StudioAppHostProjection`，让 GUI 可持有单一 app host state，并由 store 统一推进 `StudioAppHostOutput -> state/projection`，不再自己组合 `outcome + snapshot + changes` 与 timer owner / parked timer 语义
- Studio 当前也已继续把 app host 命令入口进一步收口为 `StudioAppHostController`，并补出按 GUI 意图命名的 typed result，让最小入口不再直接消费 `StudioAppHostOutput` 或 match raw command outcome
- Studio 当前也已继续把 GUI 宿主副作用收口到 controller 返回值，补出 dispatch/close effect summary，让最小入口不再直接翻读 `StudioWindowSessionDispatch`、`StudioRuntimeHostPortOutput` 或 close raw shutdown 细节
- Studio 当前也已继续把 GUI-facing 窗口状态收口为 `snapshot -> window model -> layout state` 三层契约，并让布局 sidecar 与项目文档分离保存
- Studio 当前也已继续把布局变更事件正式收口到 `WindowLayoutMutationRequested -> WindowLayoutUpdated(...)`，覆盖 `panel dock_region/stack_group/visibility/collapsed/order`、region 内 stack placement、`center_area` 与 `region_weights`
- Studio 当前也已继续把 drop preview 查询正式收口到 `StudioGuiHostCommand::QueryWindowDropTarget` 与 `StudioGuiEvent::WindowDropTargetQueryRequested`，GUI 不再需要先抓取 layout state 再自行拼 mutation 预览
- Studio 当前也已继续把 drop preview 查询结果前推为 `WindowDropTargetQueried(...)` 上的 `preview_layout_state / preview_window`，让 GUI 不必再根据 target 摘要手工重建整份 preview 布局
- Studio 当前也已继续把 drop release 收口到 `StudioGuiHostCommand::ApplyWindowDropTarget` 与 `StudioGuiEvent::WindowDropTargetApplyRequested`，让真实 GUI 不必在 query 之外再单独维护一套 mutation 翻译层
- Studio 当前也已继续把 hover 预览前推为正式会话态，新增 `StudioGuiHostCommand::SetWindowDropTargetPreview / ClearWindowDropTargetPreview` 与 `StudioGuiEvent::WindowDropTargetPreviewRequested / WindowDropTargetPreviewCleared`，让 GUI 可以直接从 `window_model` 读取当前 preview，而不必自己缓存 hover 阶段的影子布局
- Studio 当前也已继续把 `window_model.drop_preview` 扩成 `preview_layout + changed_area_ids`，让 GUI 后续既能直接渲染预览态布局，也能只按变化 area 做最小重绘/高亮
- Studio 当前也已继续把 `window_model.drop_preview` 扩成 `overlay`，让 GUI 后续可以直接读取目标 stack/tabs/插入位与高亮集，而不必再自行遍历 preview layout 和 drop target 摘要拼 overlay

截至 2026-04-04，Studio entitlement 宿主边界已进一步形成一条可直接面向真实 GUI 的正式分层：

- `StudioRuntime / StudioRuntimeHostPort` 负责共享 runtime 与多窗口 timer owner 规则
- `StudioWindowSession / StudioAppWindowHostManager` 负责窗口会话与 app 级窗口注册表、foreground、全局事件路由
- `StudioAppHost / StudioAppHostState / Store / Projection` 负责 app 级宿主真相源与状态推进
- `StudioAppHostController + effect summary` 负责 GUI 命令入口与宿主副作用消费面

Studio 当前又已继续把这条 GUI 命令入口推进为稳定 host command registry：

- `StudioAppHostController` 当前已提供 `dispatch_ui_command(command_id)`，让菜单、快捷键和命令面板后续可直接按稳定 command id 触发宿主动作
- 首批已接成真实宿主命令的 run panel command id 为 `run_panel.run_manual`、`run_panel.resume_workspace`、`run_panel.set_hold`、`run_panel.set_active` 与 `run_panel.recover_failure`
- 同一条 GUI command registry 当前又已扩到 canvas suggestion 交互：`canvas.accept_focused`、`canvas.reject_focused`、`canvas.focus_next`、`canvas.focus_previous`
- `StudioGuiCanvasWidget`、`StudioGuiShortcutRouter` 与 `StudioGuiHost` 当前都已统一走 `UiCommandRequested { command_id } -> dispatch_ui_command(command_id)`，真实桌面 GUI 后续不应再把 canvas accept/reject/focus 写成框架私有 shortcut/typed action 分支
- 对 canvas 而言，local-rules suggestion refresh 当前也已收紧为“文档写回或显式重算时才触发”；纯 `focus/reject` 交互不应顺手重刷 suggestion 列表，否则会破坏正式命令面的焦点延续语义
- `rf-ui` 当前也已把 run panel 动作展示继续冻结到 GUI-facing `label/detail/enabled` 口径，第一版 `eframe/egui` GUI 壳已直接消费这份动作详情，而不再在壳层重复拼按钮说明文本
- `rf-ui` 当前也已把 entitlement panel 动作展示继续冻结到 GUI-facing `label/detail/enabled` 口径，第一版 `eframe/egui` GUI 壳已直接消费这份动作详情，并沿既有 foreground host routing 回灌 Studio runtime
- 后续真实桌面框架在建立原生命令绑定时，应优先复用这组 `UiCommandModel` / command registry，而不是继续让各入口重复拼装运行栏 availability、disabled reason 或窗口前景派发逻辑
- 当前已把第一批 GUI command surface 的 shell 等价基线补齐：`run_panel.set_active`、`run_panel.recover_failure` 与 `canvas.accept/reject/focus` 在 menu / toolbar / palette / shortcut 间共享同一派发链；disabled 状态下 menu / toolbar / palette 也已锁定为 no-op，不再允许某个 GUI 入口绕过 registry / host gate 私改状态
- Studio 当前也已开始按“单文件不超过 1000 行 + `src/` 浅层职责分组”的工程治理收口，把 `bootstrap`、`studio_gui_shell`、`studio_gui_host`、`studio_gui_driver`、`studio_gui_window_layout`、`studio_window_host_manager`、`entitlement_session_host`、`property_package_download_client`、`auth_cache_sync`、`app_facade` 与 `control_plane_client` 分批拆成目录模块；后续推进真实 GUI 宿主时应继续沿这条模块边界深化，而不是回退到单文件堆叠

当前最小入口 `apps/radishflow-studio/src/main.rs` 已开始直接消费上述正式边界，而不再继续翻读 bootstrap、session 或 raw command outcome 细节。

Studio 当前又已继续把 runtime 区剩余卡片往更真实的 GUI-facing 宿主反馈面收口：

- `StudioGuiRuntimeSnapshot / StudioGuiWindowRuntimeAreaModel` 当前又已正式携带 `platform_timer_lines` 与 `gui_activity_lines`，不再只让 `egui` 壳私有缓存平台 timer 状态和宿主活动字符串
- `StudioGuiWindowRuntimeAreaModel` 当前又已补出 `host_actions`，把 `Foreground current / Login completed / Network restored / Trigger timer` 四类 scheduler/宿主动作收口为正式 GUI-facing 动作模型，并显式带出目标窗口路由说明和启用状态
- `StudioGuiPlatformHost` 当前又已把平台 timer request、执行结果、ignored callback 与失败路径统一追加到 `gui_activity_lines`，并把当前 schedule/native binding 摘要前推到 runtime snapshot/window model
- 第一版 `eframe/egui` GUI 壳当前已改为直接消费这套 runtime host feedback DTO；`Scheduler / Platform / GUI activity` 三块不再主要依赖 shell 私有状态，而是复用正式 snapshot/window model
- `StudioGuiWindowLayoutModel` 当前也已把 runtime 面板摘要补进 `activity` 与 `platform-timer` 两个维度，使 runtime 区在折叠或 tab 化时仍能透出宿主调度活跃度与平台 timer 状态

基于上述进展，当前下一阶段计划调整为：

- 在已接通的 `main.rs` 最小 bootstrap 入口、`RunPanelWidgetModel`、`run_panel_driver`、entitlement session driver 与 `StudioBootstrapTrigger` 契约基础上，继续把 `StudioAppFacade + WorkspaceRunCommand + WorkspaceSolveService` 和 entitlement session event 接到更完整的 UI 运行命令、登录完成事件与定时调度入口
- 在已补出的 `StudioRuntimeConfig + Trigger + Report + Output` 共享入口基础上，继续决定真实桌面框架里的 timer 句柄、窗口生命周期事件与后台任务宿主如何接到这条统一 runtime 链路
- 在已补出的 host effect `id + follow_up + ack` 协议基础上，继续决定真实桌面框架里的 timer 句柄生命周期、apply 时机与 ack 回写时机
- 在已补出的 `StudioRuntimeTimerHostCommand` 薄层基础上，继续决定真实桌面框架里的 timer 句柄保存位置与多窗口/单窗口宿主边界
- 在已补出的 `StudioRuntimeTimerHostState / Transition` 与 `StudioWindowHostState + StudioRuntimeHostPort` 基础上，继续决定真实桌面框架里的具体 GUI timer handle、窗口重建迁移策略与多窗口所有权口径
- 在已补出的多窗口 `StudioRuntimeHostPort` 所有权/转移/park 语义基础上，继续决定真实桌面框架里的具体 GUI timer handle 绑定与窗口前后台事件来源
- 在已补出的 `StudioAppWindowHostManager + StudioAppWindowHostCommand` 基础上，继续决定真实桌面框架里的 app 生命周期、窗口创建销毁与后台任务事件如何统一接到这条宿主命令面
- 在已补出的 `StudioAppHost + StudioAppHostSnapshot` 基础上，继续决定真实桌面框架里的 app state store、窗口 registry 与后台任务宿主是否直接复用这份快照作为单一真相源
- 在已补出的 `StudioAppHost + StudioAppHostSnapshot + StudioAppHostChangeSet` 基础上，继续决定真实桌面框架里的 app state store、窗口 registry 与后台任务宿主如何直接消费正式 snapshot/change 输出，而不是在 GUI 层自行做二次 diff
- 在已形成的 `StudioGuiWindowLayoutState + drop target query/preview session` 摘要基础上，继续补真实 dock 编排契约，例如 tabbed group 标题/可关闭策略、更细的拖拽预览/命中层、跨窗口布局模板与更完整的标题栏/窗口 scope 语义
- 在已补出的 `StudioAppHostState + StudioAppHostStore + StudioAppHostProjection` 基础上，继续决定真实桌面框架里的 app 生命周期宿主、窗口创建销毁入口与后台任务桥接是否直接围绕这份正式 state/projection 接线
- 在已补出的 `StudioAppHostController + StudioAppHostState + StudioAppHostStore + StudioAppHostProjection` 基础上，继续决定真实桌面框架里的原生窗口事件源、app 生命周期宿主与后台任务入口如何直接走这条正式 controller 边界
- 在已补出的 app host effect summary 基础上，继续决定真实桌面框架里的 native timer handle、后台任务调度和 close retirement 提示如何直接接到这组正式 GUI 宿主副作用
- 在已补出的 `StudioGuiNativeTimerRuntime + StudioGuiPlatformHost + StudioGuiPlatformTimerDriverState` 基础上，继续把真实桌面框架 timer API、消息循环与批量回灌入口接到既有 batch/round glue；优先补宿主消费面，不提前进入真实 UI 布局和控件组织设计
- 在已补出的 runtime host feedback DTO 基础上，继续把 shell 内残余的宿主私有展示状态压回正式 snapshot/window model，避免后续真实桌面框架再复制一层“平台活动日志 / 调度状态说明”的影子状态
- 继续冻结运行结果派发、日志入口与后续异步执行边界
- 在不打乱当前边界的前提下，再恢复更完整的 Studio 交互流、联网提示位置与内核主线推进

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
- `UnitOp.Mvp` 当前已补入冻结的 `typelib/RadishFlow.CapeOpen.UnitOp.Mvp.idl`、首份 `typelib/RadishFlow.CapeOpen.UnitOp.Mvp.tlb`，并已验证 `<ComHostTypeLibrary ...>` 可接入 `.NET comhost` 构建
- `Registration` 当前已继续前推到真实 `TypeLib` 注册链路：dry-run 会优先解析真实 `UnitOp.Mvp` 输出目录中的 `ResolvedComHostPath / ResolvedTypeLibraryPath`、校验 `comhost runtime layout` 与 `TypeLib GUID/version`，execute 会按 scope 调用 `RegisterTypeLib(ForUser)` / `UnRegisterTypeLib(ForUser)`，并把 `TypeLib` 纳入备份/回滚范围
- `RadishFlow.CapeOpen.UnitOp.Mvp.tlb` 当前也已随 `UnitOp.Mvp` 与 `Registration` 输出目录一起复制，减少脚本和执行入口对源码路径猜测的依赖
- 本机工具链当前已确认可用：`D:\Windows Kits\10` 下的 `midl.exe / rc.exe` 与现有 Visual Studio `cl.exe` 已可用于继续推进 `IDL -> TLB`；问题不再是“本机没有工具”
- 同日真实复验又确认 `pwsh` 的 `0x800080A5` 是宿主预加载 `.NET 9.0.10` 与 PMC 目标 `.NET 10.0.0` runtime 不兼容导致的假阴性；native COM / PME 类宿主的后续探测应改用 `Windows PowerShell 5` 或其他非预加载 .NET 宿主
- 在 `Windows PowerShell 5` 下，current-user `TypeLib` 已出现在 `HKCR` 合并视图，且 `CLSID\{...}\TypeLib` 关联也已补上，但 `New-Object -ComObject` 仍报 `0x80131165`；因此剩余主线已进一步收敛到 classic late-bound COM / typelib 兼容细节，而不是默认 comhost 路径或 runtime sidecar 缺失

截至 2026-04-25，M5 的 late-bound COM 探测又进一步前推，当前新增结论如下：

- VS 2026 Insiders 当前可提供更新的 `cl.exe`，但未直接提供 `midl.exe / rc.exe`；`IDL -> TLB` 组合工具链当前仍应优先采用 `D:\Windows Kits\10\bin\10.0.26100.0\x64` 下的 `midl.exe / rc.exe`，再配合 VS `cl.exe`
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

## 推荐工作流

### 内核优先

先保证以下三层完全可本地跑通：

- `rf-thermo`
- `rf-flash`
- `rf-solver`

### FFI 第二

等 Rust 内核最小流程闭环稳定后，再导出 FFI。

### CAPE-OPEN 第三

等 FFI 稳定后，再实现 `.NET 10` 的 CAPE-OPEN 外壳。

### Studio 第四

Rust Studio UI 可以在 `M2` 后并行开始，但不要阻塞 `M4/M5` 主线。

## 中后期 Studio 交互演进方向

以下方向纳入正式规划，但不作为当前 `M2/M3` 的阻塞退出条件：

### 1. 画布视图模式

- 流程图画布后续允许在平面视图与立体投影视图之间切换
- 立体模式优先理解为同一份 flowsheet 的增强展示，而不是单独的 3D 建模系统
- 项目文件、端口语义、连接关系、求解输入输出与命令历史继续只有一套真相源

### 2. 流线状态可视化

- 后续把物流线、能量流线、信号流线纳入统一的画布表现体系
- 每类流线都应支持静态/动态两种显示模式，动态模式用于表达方向、活动性与状态变化
- 推荐把“流线类型”和“运行状态”拆成正交编码，而不是只靠单一颜色承担全部语义
- 推荐配色方向可先按主类型区分：物流偏青绿系、能量流偏琥珀/橙系、信号流偏紫/石板系
- 推荐状态表达优先结合低饱和/半透明、虚线、箭头动效、发光强弱或小型状态徽标；例如未求解或待补全可先走灰态/低饱和表达，收敛后恢复对应类型主色并增强可读性
- 具体配色方案留待真实 UI 主题阶段再做可访问性校验，不在当前路线图里提前写死

### 3. 与 RadishMind 的辅助建模联动

- 对标准单元放置后的常见后续动作，后续允许提供“行为预测 / 候选补全”式辅助
- 第一批建议从标准端口拓扑最稳定的单元开始，例如 `Flash Drum` 的 `inlet / vapor / liquid`
- 候选补全应先以灰态 ghost 入口、出口或连线显示，等待用户按 `Tab` 或显式确认后再写入正式文档
- `RadishMind` 的角色优先是建议排序、命名补全和常见建模模式提示，不直接绕过本地连接校验、端口规则和命令系统
- 这部分需要同步到 `RadishMind` 项目，单独补出 suggestion schema、接受/拒绝动作和 prompt 契约

### 4. 建议的前置任务

- 在 `rf-model` / `rf-flowsheet` 中继续冻结标准单元 canonical ports 与后续能量/信号端口扩展策略
- 在 `rf-ui` 中为 ghost suggestion、接受/拒绝动作和状态叠加预留正式 DTO / state 边界
- 在 `rf-canvas` 中为视图模式、流线样式层和状态叠加层预留分层渲染口径
- 在 `RadishMind` 侧补出“放置单元 -> 返回候选补全列表 -> 用户接受某项”的最小提示词与输出结构

## 建议的首批任务拆分

### Sprint A：基础骨架

- 初始化仓库
- 初始化 workspace
- 建立核心 crates
- 建立文档骨架

### Sprint B：热力学与闪蒸

- 二元组分参数结构
- Antoine
- Rachford-Rice
- TP Flash
- 黄金样例测试

### Sprint C：流程求解

- 流股对象
- 单元模块
- 流程连接
- 顺序模块法
- JSON 示例流程
- 最小集成测试与端到端闭环样例

### Sprint D：互操作边界

- Rust FFI
- .NET Adapter
- 基础互调测试

### Sprint E：PMC 暴露

- Unit Operation PMC
- 注册工具
- PME 冒烟验证

## 风险清单

| 风险 | 说明 | 应对策略 |
| --- | --- | --- |
| 范围膨胀 | 同时想做 PME、Thermo PMC、外部模型加载 | 严格锁死 MVP 不做项 |
| 热力学模型漂移 | 计算结果无法稳定复现 | 建立黄金样例测试 |
| FFI 设计过早复杂化 | 边界接口难维护 | 第一版只做句柄 + JSON |
| UI 干扰主线 | 过早投入画布与视觉细节 | UI 不阻塞内核与 CAPE-OPEN 主线 |
| 交互创新反向侵入内核 | 为 3D 画布、动态流线或 AI 建议过早扩张核心语义 | 先冻结共享 flowsheet 语义与 suggestion 契约，再逐层推进视图与模型实验 |
| PME 兼容不确定 | 外部软件实际行为偏标准之外 | 先选一个目标 PME 做主验证 |

## 建议的首批交付物

MVP 第一轮完成时，建议至少交付：

- 一个可运行的 Rust workspace
- 一个可运行的最小桌面程序壳
- 一个可求解的二元流程示例
- 一个 Rust FFI 动态库
- 一个 .NET 10 CAPE-OPEN Unit Operation PMC
- 一份目标 PME 验证说明

## Definition of Done

以下条件同时满足时，可认为 MVP 完成：

- Rust 内核可完成最小稳态流程求解
- Rust UI 可打开、编辑并运行至少一个示例流程
- .NET 10 适配层可成功调用 Rust 内核
- 自有 Unit Operation PMC 可被目标 PME 识别并调用
- 有最小自动化测试与人工验证记录
