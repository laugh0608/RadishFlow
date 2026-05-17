# MVP Roadmap Plan Alignment: Studio

更新时间：2026-05-14

## 用途

用途：归档 2026-03-29 至 2026-04-04 的 Studio / 控制面 / GUI 宿主计划对齐历史。
读者：需要追溯 Studio 应用层和控制面宿主边界演进的开发者。
不包含：最新阶段判断、CAPE-OPEN 适配层历史和用户操作指南。

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
- 同一条 GUI command registry 当前又已扩到 canvas suggestion 交互与 pending edit 取消：`canvas.accept_focused`、`canvas.reject_focused`、`canvas.focus_next`、`canvas.focus_previous`、`canvas.cancel_pending_edit`
- `StudioGuiCanvasWidget`、`StudioGuiShortcutRouter` 与 `StudioGuiHost` 当前都已统一走 `UiCommandRequested { command_id } -> dispatch_ui_command(command_id)`，真实桌面 GUI 后续不应再把 canvas accept/reject/focus/cancel 写成框架私有 shortcut/typed action 分支
- 对 canvas 而言，local-rules suggestion refresh 当前也已收紧为“文档写回或显式重算时才触发”；纯 `focus/reject` 交互不应顺手重刷 suggestion 列表，否则会破坏正式命令面的焦点延续语义
- 画布编辑前置状态当前先冻结为 `rf-ui::CanvasEditIntent`：`BeginPlaceUnit` 只创建 transient pending edit，`CancelPendingEdit` 只清理该状态；`CommitPendingEditAt { position }` 会把当前 `PlaceUnit` 意图提交成 canonical `UnitNode` 与 `DocumentCommand::CreateUnit`，递增 revision、进入撤销栈并打开对应 Inspector；动态 `CanvasPoint` 当前同时进入提交结果和 `<project>.rfstudio-layout.json` sidecar 的 Canvas unit position 列表，不写入项目文档。真实 `egui` GUI 当前通过 `StudioGuiCanvasPlaceUnitPaletteViewModel` 暴露 `Feed / Mixer / Heater / Cooler / Valve / Flash Drum` 的 begin-place command，点击落点仍复用同一条提交路径；提交成功后会复用新单元的 object command target / focus anchor 生成 `StudioGuiCanvasCommandResultViewModel`，统一驱动新建提示、Inspector 焦点、Canvas 一次性定位、GUI activity 与命令面只读反馈；提交失败或 anchor 过期也走同一结果 DTO。当前已有 `canvas.move_selected_unit.left/right/up/down` 作为 layout sidecar 的正式移动边界，只验证选中 unit 并更新 sidecar 坐标；缺少 sidecar 坐标时先 pin 到当前 transient grid slot，再执行一步移动。不改项目文档、不触发 dirty。不做拖拽、不扩 CAPE-OPEN；后续完整画布编辑器仍应继续消费 `StudioGuiCanvasState / StudioGuiCanvasPresentation` 并补正式 layout state 边界
- 当前 Canvas presentation 已把已有 `UnitNode`、material stream binding、canonical material port、活动 Inspector 目标、viewport focus anchor、运行诊断和最近一次 command result 投影为只读画布对象层；真实 `egui` GUI 只渲染单元块、物流线、selection/focus callout、对象列表导航、端口 marker/hover、诊断 badge、状态 legend、一次性滚动高亮和 command-surface 只读摘要，不直接写项目文档，也不把端口/连线/坐标编辑藏进 shell 私有逻辑
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
