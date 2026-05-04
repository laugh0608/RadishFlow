# Architecture Overview

更新时间：2026-05-01

## 目标

RadishFlow 的目标架构已经冻结为“桌面端三层 + 外部控制面”：

1. Rust Core
2. Rust Studio UI
3. .NET 10 CAPE-OPEN Bridge

第一阶段只要求桌面三层边界清晰，不要求三层都立即进入完整实现。

同时，当前还补充冻结一个 **外部控制面**：

4. RadishFlow Control Plane (`ASP.NET Core / .NET 10`, External)

这不是桌面进程内部的新层，而是产品外部依赖的服务平面，用于承担：

- OIDC 登录
- RadishFlow 专属授权
- 受控物性资产清单与租约
- 派生数据包分发
- 审计与撤销入口

不承担：

- 本地主求解循环
- CAPE-OPEN/COM 适配

## 当前仓库分层

### Rust Core

当前优先建设的 Rust Core 由以下 crate 构成：

| crate | 当前职责 | 当前状态 |
| --- | --- | --- |
| `rf-types` | 基础 ID、枚举、错误类型 | 已建立第一批基础类型 |
| `rf-model` | 组分、流股、单元、流程图对象模型 | 已建立第一批领域数据结构 |
| `rf-thermo` | 热力学数据结构与热力学接口 | 已建立最小 API、内存 provider、基于本地缓存目录/授权缓存索引的 `PropertyPackageProvider` 实现，并用首个真实样例包覆盖装载测试 |
| `rf-flash` | `TP Flash` 输入输出契约与求解器接口 | 已建立最小 API，并已实现最小二元 `TP Flash`、Rachford-Rice 与黄金样例 |
| `rf-unitops` | 单元模块行为抽象 | 已建立内建单元规范、统一流股输入输出接口，并实现 `Feed`、`Mixer`、`Heater/Cooler`、`Valve`、`Flash Drum` 的最小行为边界 |
| `rf-flowsheet` | 连接关系与图结构校验 | 已建立首轮材料端口连接校验，覆盖 canonical port signature、流股存在性与“一股一源一汇”约束 |
| `rf-solver` | 顺序模块法求解器 | 已建立首轮无回路顺序模块法，可执行 `Feed + Mixer + Flash Drum`、`Feed -> Heater -> Flash Drum` 与 `Feed -> Valve -> Flash Drum` 闭环，并产出带 summary / diagnostics / step 明细的最小 `SolveSnapshot`；当前失败路径已继续收口到 solver-stage + 稳定 diagnostic code + unit/port helper 上下文 |
| `rf-store` | JSON 存储与授权缓存索引 | 已建立项目文件 / 授权缓存 / 本地包 `manifest.json` / `payload.rfpkg` 的 JSON 读写、迁移分发、版本校验与相对路径布局 |
| `rf-ffi` | Rust 与 .NET 的 C ABI 边界 | 已建立第一版最小句柄式 C ABI，当前覆盖 `engine_create/destroy`、`flowsheet_load_json`、`property_package_load_from_files`、`property_package_list_json`、`flowsheet_solve`、`flowsheet_get_snapshot_json`、`stream_get_snapshot_json`、`engine_last_error_message`、`engine_last_error_json` 与 `rf_string_free`；当前运行时已同时支持内置 demo package、本地 `manifest/payload` 注册与 package manifest 列表导出 |

当前仓库级集成测试也已正式落到 `tests/rust-integration` workspace crate，并由 `cargo test --workspace`、`scripts/check-repo.ps1` 与 `scripts/check-repo.sh` 自动覆盖五条示例 flowsheet 回归。

### Rust Studio UI

当前 UI 相关 crate 已从“状态骨架与应用边界冻结”推进到第一版最小可操作工作台闭环，但仍不代表已经进入完整视觉设计、任意项目文件工作流或复杂画布编辑阶段：

| crate | 当前职责 | 当前状态 |
| --- | --- | --- |
| `rf-ui` | UI 状态与行为逻辑 | 已建立 `AppState`、授权态、求解态与控制面 DTO 骨架；已补 `RunPanelState`、`RunPanelIntent`、`RunPanelCommandModel`、`RunPanelViewModel`、`RunPanelPresentation` 与 `RunPanelWidgetModel`，并可把 `rf-solver::SolveSnapshot` 映射为 UI 层结果快照；当前字段级 Inspector 草稿态已落入 `WorkspaceState.drafts`，Stream Inspector 的有效单字段提交会生成 `DocumentCommand::SetStreamSpecification`；`CommandHistoryEntry` 当前也已携带 `before / after` flowsheet 快照，用于基础 undo/redo |
| `rf-canvas` | 流程图画布能力 | 占位 |
| `apps/radishflow-studio` | 桌面入口程序 | 已建立 auth cache sync 桥接、控制面 HTTP client、entitlement / manifest / lease / offline refresh 编排、下载获取抽象、基于 `reqwest + rustls` 的真实 HTTP transport、HTTP 请求/响应适配层、可重试/不可重试失败分类、下载 JSON 到本地 payload DTO 的协议映射、摘要校验、失败回滚与测试；并已补上 `PropertyPackageProvider -> rf-solver -> rf-ui::AppState` 的最小工作区求解桥接，可直接基于已加载物性包或本地 auth cache 执行真实 solve 并回写 UI 快照/日志；当前又已形成 `StudioGuiHost / StudioGuiDriver / StudioGuiSnapshot / StudioGuiWindowModel / StudioGuiWindowLayoutState` 这一条 GUI-facing 宿主与窗口布局契约，并把窗口布局持久化为项目同目录 sidecar；当前又已把 drop preview 前推为 `StudioGuiWindowDropTargetQuery -> StudioGuiHost / StudioGuiDriver` 的显式查询入口，并在 host 内补出非持久化 preview 会话态；当前又已补出 `StudioGuiNativeTimerRuntime`，让 GUI 可在消费 `StudioGuiNativeTimerEffects` 后继续跟踪逻辑 timer handle、`next_due_at` 和一次性 due callback，而不必在真实框架里从零重写同一套 timer 生命周期；当前又已把原生 timer callback 正式收口为 `StudioGuiEvent::NativeTimerElapsed { window_id, handle_id }`，由 driver 先校验当前绑定再回灌 `TimerElapsed`，避免真实宿主把 stale callback 误灌进 runtime；当前又已补出 `StudioGuiPlatformHost` 与 `StudioGuiPlatformTimerDriverState`，把平台 timer request、native timer id 映射、stale callback 判定与平台 notice 固定为显式状态机；当前又已引入第一版 `eframe/egui` GUI 壳，在单原生窗口内承载逻辑窗口切换，并直接消费 `window_model.drop_preview.overlay / changed_area_ids` 画出局部插入条、anchor 顶线、target-anchored 浮动 overlay 与局部 hint pill；当前 Runtime 面板已能切换内置正向示例项目、输入路径或 Windows 原生文件选择器打开现有 `*.rfproj.json`、在未保存修订存在时先确认打开、打开成功后记录并持久化 shell 级最近项目入口、触发既有运行栏动作，并展示 `SolveSnapshot` 映射出的结构化流股结果、Result Inspector、带修复动作和目标定位的失败结果、诊断目标、活动 Inspector 详情、单元最新执行结果、带单元/产出流股跳转的求解步骤、Stream Inspector 字段草稿/提交、诊断、日志、工作区摘要与平台/授权状态；当前 `edit.undo / edit.redo` 也已进入正式 command surface 并经由 `StudioRuntimeTrigger::DocumentHistory` 执行；当前 shell 还提供中文/英文切换，并通过系统 CJK 字体 fallback 支持中文显示 |

这一路径仍坚持先消费已冻结的应用层、运行栏和 snapshot presentation 边界；真实 UI 只做最小闭环承接，不反向改写内核、求解或项目文件语义。

截至 2026-05-03，Studio Canvas 已补出最小可见、只读扫读层和多单元放置反馈闭环：单元块、物流线、Inspector 焦点高亮、对象列表导航、焦点气泡、material port marker、端口 hover、运行/诊断 badge、状态 legend、viewport focus anchor、对象列表 `All / Attention / Units / Streams` 临时筛选，以及 `Feed / Mixer / Heater / Cooler / Valve / Flash Drum` 的 pending edit palette / commit 路径都通过 GUI-facing presentation 暴露并由 `egui` 渲染。`CommitPendingEditAt -> DocumentCommand::CreateUnit` 成功后会复用新单元的 object command target 和 focus anchor，统一生成 `StudioGuiCanvasCommandResultViewModel`，从而让新建提示、Inspector 焦点、Canvas 一次性定位、GUI activity 与命令面只读反馈走同一条结果路径；无 pending edit、unsupported unit kind、dispatch 失败或 anchor 过期也使用同一套 warning / error result。当前已补多单元 placement 提交端回归矩阵，逐类锁定 `CreateUnit kind`、canonical ports、Inspector 焦点、Canvas focus anchor 和 command result 反馈。2026-05-04 继续在同一边界上补出本地 Canvas suggestions，已能通过正式 `DocumentCommand::ConnectPorts` / outlet stream 创建走通 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum` 和 `Feed + Feed -> Mixer -> Flash Drum` 三类可求解建模路径；`Mixer` 多来源场景只在来源数量与未绑定 inlet 数量匹配时给入口建议。同日又补齐真正空白项目的 MVP 前置基线：无组件项目打开时初始化 `component-a / component-b`，并用初始化后的 flowsheet 生成本地 `binary-hydrocarbon-lite-v1` 物性包缓存，保存后重新打开仍可运行 `Feed -> Flash Drum` 最短闭环；随后又补出逐条 suggestion Apply，GUI 可显式接受指定本地建议，而不再只能接受当前 focused suggestion；同日还补出 placement 坐标最小持久化，已提交单元落点保存到 `<project>.rfstudio-layout.json` sidecar，重开后 Canvas 优先按 sidecar 坐标渲染；随后又补出 `CanvasUnitLayoutMoveRequested -> MoveCanvasUnitLayout` 离散布局移动边界，shell 可对当前选中且已有 sidecar 坐标的单元执行 nudge，保存后重开继续恢复新位置。当前画布仍只把已有 `FlowsheetDocument` 对象投影到轻量布局状态，定位滚动和高亮属于 shell-local 一次性状态；端口点击编辑、自由连线创建、拖拽布局编辑器、视口持久化、项目 schema 扩张、完整组件库、完整物性包选择器和 CAPE-OPEN 扩张都不属于当前阶段。这一层当前应视为最小建模路径收口，后续不再继续扩 hover、legend、focus 或 command feedback 细节，除非它们直接服务下一条可求解建模闭环。

同时补充一条当前协作闸口：在 `apps/radishflow-studio` 还处于 GUI-facing 边界、宿主桥接和布局状态契约冻结阶段时，可以继续直接推进；但一旦工作重心切到真实界面布局、控件组织、视觉表达、交互流和较重的 UI 逻辑设计，后续实现前必须先向用户同步方向与关键取舍，并保留用户干预窗口，不把产品交互方案静默固化。

不过，中后期 Studio 交互方向已经可以先在架构上明确三条原则：

- 流程图画布后续允许在平面视图和立体投影视图之间切换，但两者共享同一套 flowsheet 语义与项目文件，不分裂出第二套编辑模型
- 物流线、能量流线与信号流线后续应支持静态/动态两种可视化模式，并把“流线类型”和“收敛/待求解/异常状态”拆成正交表达，而不是只靠单一颜色承担所有语义
- 与 `RadishMind` 的结合后续优先落在“灰态候选补全 + 显式接受”的辅助建模上，例如放置标准单元后显示待补全端口/连线，并由 `Tab` 接受；模型建议不应绕过本地连接校验和文档命令边界

不过 App 架构层面的关键口径已经开始冻结，当前包括：

- 单文档工作区优先
- 字段级草稿提交
- `SimulationMode` / `RunStatus` 分离
- 独立 `SolveSnapshot`
- OIDC / 授权 / 远端资产保护作为外部控制面，而不是塞进 Rust Core

同时，Studio 应用层当前已经具备一条可测试的最小真实求解链路：

- 从 `PropertyPackageProvider` 或本地 `StoredAuthCacheIndex` 加载 `ThermoSystem`
- 组装 `PlaceholderThermoProvider + PlaceholderTpFlashSolver + SequentialModularSolver`
- 通过 `StudioAppFacade -> WorkspaceRunCommand -> WorkspaceSolveService -> solver_bridge` 完成 auth cache 上下文承接、包选择解析、默认 request 生成和手动/自动触发门控，再将内核 `SolveSnapshot` 回写到 `rf-ui::AppState`
- 当前最小桌面入口 `run_studio_bootstrap` / `main.rs` 已改为默认通过 `WidgetPrimaryAction -> RunPanelWidgetEvent -> run_panel_driver -> WorkspaceControlAction -> StudioAppFacade` 触发这条链路，同时保留显式 `RunPanelIntent` 兼容入口，避免桌面入口绕过 UI 组件层
- `rf-ui` 当前已把 `Run / Resume / Hold / Active` 的主动作选择、可见性与可用性冻结为 `RunPanelCommandModel`，避免按钮判断散落在 Studio 或最终视图层
- `rf-ui` 当前也已补出 `RunPanelViewModel`、`RunPanelTextView`、`RunPanelPresentation` 与 `RunPanelWidgetModel`，让最小入口可以直接消费主按钮/次按钮槽位、文本布局和最小交互语义，而不是继续读取摘要布尔值自行拼装动作
- `run_studio_bootstrap` / `main.rs` 当前已开始直接消费这套运行栏组件 DTO，而 `run_panel_driver` 已负责回收新的 widget/control state，形成第一处最小真实组件驱动入口
- 在求解成功/失败时同步更新 `SolveSessionState` 与 `AppLogFeed`

同时，Studio GUI-facing 状态边界当前也已进一步冻结为：

- `StudioGuiHost` / `StudioGuiDriver` 作为 GUI 面向的平台事件与宿主命令入口
- 当前 GUI 正式命令面已进一步冻结为 `StudioGuiEvent::UiCommandRequested { command_id } -> StudioGuiHostCommand::DispatchUiCommand { command_id }`，至少覆盖 `run_panel.recover_failure`、`entitlement.sync` 与 `entitlement.refresh_offline_lease`；GUI 壳不再继续保留 entitlement 专用事件/命令旁路
- `StudioGuiPlatformHost` 作为平台 timer 调度适配层，负责把下一条 pending timer binding 的前后变化收口为平台侧 `Arm / Rearm / Clear` 请求，并持有平台 timer adapter、平台失败日志与 GUI 可直接消费的 `platform_notice`
- `StudioGuiPlatformTimerDriverState` 作为平台 native timer 适配层，负责消费上述请求、保存当前 native timer id 与逻辑 binding 的映射，并在 callback 到来时反查回 `window_id + handle_id`
- 平台若按 `native_timer_id` 回灌 callback，当前应优先消费 `StudioGuiPlatformHost::dispatch_native_timer_elapsed_by_native_id(...)` 的正式 outcome；命中有效映射时继续分发，命中不存在或过期 id 时返回显式 ignored outcome，而不是把平台层常见竞态继续上抛为 `RfError`
- 平台在 native timer 启动成功或失败后回灌 ack 时，当前也应优先消费 `StudioGuiPlatformHost::acknowledge_platform_timer_started(...)` / `acknowledge_platform_timer_start_failed(...)` 的正式 outcome；尤其是启动成功但 pending schedule 已 missing/stale 时，outcome 当前又可继续派生正式 `follow_up_command`，把“立即清理刚创建的 native timer id”收口成稳定平台命令，而不是只留一句注释语义
- 若平台一次性回灌多条 start success/failure ack，`StudioGuiPlatformHost` 当前又已补出 `acknowledge_platform_timer_started_feedbacks(...)` / `acknowledge_platform_timer_start_failed_feedbacks(...)`，并继续支持对 success ack 批量执行 follow-up cleanup；真实宿主不必再自己循环累计结果、收集清理命令后回查最终 `snapshot`
- 若平台是异步型 timer API，当前也可直接批量消费不执行型结果：`dispatch_native_timer_elapsed_by_native_ids(...)` 与 `dispatch_due_native_timer_events_batch(...)` 会在同一份正式结果里带出逐条 callback/dispatch outcome、最终 `snapshot`、下一次 schedule，以及已整理好的 `native_timer_requests()`；真实宿主不必再自己一边循环结果、一边手工提取 request
- 若真实宿主希望把一轮消息循环里的 start ack / fail ack / native callback / due drain 一次性提交，`StudioGuiPlatformHost` 当前又已补出 `process_async_platform_round(...)`；这层会按固定顺序完成状态推进，并把最终 `snapshot`、聚合后的 `native_timer_requests()` 与 `follow_up_commands()` 固定在单一正式结果里
- 上述 async round 当前又已进一步补出 `actions()`，把 `follow_up cleanup -> native_timer_requests` 的宿主执行顺序固定为正式动作清单；真实宿主不必自己再归并或排序这两类平台动作
- 若宿主收到的是异步型 callback / ack 批处理，但平台 timer API 仍可同步执行，`StudioGuiPlatformHost` 当前又已补出 `process_async_platform_round_and_execute_actions(...)`；宿主可直接复用 host 内冻结的动作顺序，并拿到执行后的最终 `snapshot/window`
- 对于同步型平台 timer API，`StudioGuiPlatformHost` 当前又已补出 `execute_platform_timer_request(...) + StudioGuiPlatformTimerExecutor`；平台若能在同一次调用里直接拿到 `native_timer_id` 或启动失败细节，就不必再在 `main.rs` 或未来 GUI 入口手工串接 `apply_request -> execute -> acknowledge -> follow-up`
- 上述同步型 glue 当前又进一步支持 `dispatch_event_and_execute_platform_timer(...)`、`dispatch_native_timer_elapsed_by_native_id_and_execute_platform_timer(...)` 与 `dispatch_due_native_timer_events_and_execute_platform_timers(...)`；若平台入口本来就是“派发 GUI 事件后立刻调用同步 timer API”的风格，`main.rs` 或未来真实 GUI 宿主也不必再手工拆成 `dispatch -> 取 native_timer_request -> execute`
- 这组组合入口当前还会把返回结果里的 `snapshot/window` 刷新为 timer 执行后的最终 GUI-facing 视图；若同步执行里触发了平台 notice / runtime log 变化，真实 GUI 不必再额外回查 host 才能拿到更新后的可显示状态
- 对于更接近真实宿主的“批量平台 callback / due timer drain”场景，`StudioGuiPlatformHost` 当前又已补出 `dispatch_native_timer_elapsed_by_native_ids_and_execute_platform_timers(...)` 与 `drain_due_native_timer_events_and_execute_platform_timers(...)`；平台可在同一份正式结果里拿到逐条 outcome、最终 `snapshot` 与下一次 native timer schedule，而不必自己再写循环后回查 host
- `StudioGuiSnapshot` 作为跨模块聚合快照真相源
- `StudioGuiWindowModel` 作为窗口内容分区模型
- `StudioGuiWindowDiagnosticTargetActionModel` 当前作为结果审阅/错误定位的统一 action presentation，汇总失败恢复、Inspector 目标、求解步骤单元和产出流股跳转；真实 GUI 继续按既有 `command_id` 派发，不新增导航或 recovery 私有状态机
- `StudioGuiWindowFailureDiagnosticDetailModel` 当前作为失败详情只读 presentation，直接承接 latest diagnostic summary 的 code / revision / severity / related targets，避免 GUI 从失败 message 中反解析结构化信息
- Canvas attention presentation 当前也消费同一组结构化 diagnostic target：unit / stream hover、material port hover 与 object list attention summary 会展示 `related_port_targets` 归并出的只读 port 摘要，但定位仍复用现有 `InspectorTarget` command，不新增端口级私有命令
- `StudioGuiWindowLayoutState` 作为正式布局状态契约，覆盖 `panel dock_region/stack_group/visibility/collapsed/order`、stack active tab、region 内 stack placement、`center_area`、`region_weights`、多窗口 `layout scope` 与 GUI 可直接消费的 `drop target` 摘要推导
- runtime 区域当前也会把 `platform_notice` 前推到窗口布局摘要与 badge，真实 GUI 在 panel 折叠或 tab strip 状态下不必退回日志列表才能感知平台 timer 异常
- `StudioGuiWindowLayoutModel` / `StudioGuiWindowPanelLayout` 当前也已冻结 tab 展示语义，显式区分 `Standalone / ActiveTab / InactiveTab`，让真实 GUI 不必自己再猜非 active tab 的展示角色
- tab strip 当前也已进入正式布局状态机，至少覆盖 `SetActivePanelInStack`、`ActivateNextPanelInStack`、`ActivatePreviousPanelInStack`、`MovePanelWithinStack` 与 `UnstackPanelFromGroup`，不再把 tab 切换、循环和重排留给 GUI 私有状态
- `StudioGuiWindowDropTargetQuery` 当前也已冻结为 GUI-facing 预览查询口径，并由 `StudioGuiHostCommand::QueryWindowDropTarget` / `StudioGuiEvent::WindowDropTargetQueryRequested` 暴露显式查询入口，未来真实 GUI 可按 hover/anchor/placement 请求 drop preview，而不再自己拼内部布局状态
- 上述 query 结果当前又已直接携带 `preview_layout_state / preview_window`，让 GUI 在 hover 时可以直接消费预览态，而不是只拿到 target 摘要后再自行反推整份布局
- 上述同一份 query 当前也已可直接通过 `StudioGuiHostCommand::ApplyWindowDropTarget` / `StudioGuiEvent::WindowDropTargetApplyRequested` 落地成正式布局更新，未来真实 GUI 的 hover/query 与 release/apply 不必再维护两套拖放词汇
- `StudioGuiHost` / `StudioGuiDriver` 当前又已补出 `SetWindowDropTargetPreview / ClearWindowDropTargetPreview` 与对应事件，host 会持有非持久化 preview 会话态，并把它通过 `StudioGuiSnapshot / StudioGuiWindowModel.drop_preview` 暴露给 GUI；真实 GUI 不必自己缓存当前 hover 预览
- `StudioGuiWindowModel.drop_preview` 当前又已进一步携带 `preview_layout + changed_area_ids`，让真实 GUI 可以直接消费预览态布局 DTO 与最小变化集，而不必自己再从两份 layout state 做二次重建或比对
- `StudioGuiWindowModel.drop_preview` 当前又已补出 `overlay`，显式带出目标 region/stack group、tab 插入位、高亮 area 集与目标 active tab；真实 GUI 不必再从 `drop_target + preview_layout` 手工拆 overlay 提示语义
- 第一版 `eframe/egui` GUI 壳当前也已直接消费这份 `drop_preview.overlay`，把局部插入竖条、anchor 顶线、新 stack 占位、target-anchored 浮动 preview 与局部 hint pill 直接画在目标位置，而不是继续依赖顶栏摘要或壳层私有推导
- 当前 GUI 壳仍明确停留在“单原生窗口承载逻辑窗口切换”的阶段，不在这一轮把范围扩张到多原生窗口宿主
- `StudioAppHostController` / `StudioAppWindowHostManager` 当前也已把前台 entitlement、前台 recovery 与按窗口 recovery 的历史包装器压回既有 `dispatch_ui_action(...)`、`dispatch_window_trigger(...)` 与 `StudioRuntimeTrigger` 主通路；后续若新增 GUI-facing 动作，应优先复用稳定 `command_id`、`UiAction` 或 trigger，而不是再新增一层“foreground wrapper”命令
- `StudioGuiPlatformHost` 当前会在每次事件派发和 due timer 排空后比较前后 pending timer binding，把平台真正需要执行的 timer 调度差异收口为显式 `native_timer_request`，并继续携带 `window_id / handle_id / slot`
- `StudioGuiPlatformTimerDriverState` 当前会把这份 request 继续收口为平台可执行的 `Arm / Rearm / Clear` 命令，并在 native timer 创建后记录 `native_timer_id -> logical binding` 映射；平台若创建失败，也已有显式 failure ack 用于清理 pending 状态
- 平台 native timer callback 当前也已继续收口为 `Dispatched / IgnoredUnknownNativeTimer / IgnoredStaleNativeTimer` 三类正式结果，真实 GUI 或框架 glue 可直接按 outcome 决定是否忽略，无需再把 stale/missing callback 包装成错误流
- 平台 native timer start ack 当前也已继续收口为 `Applied / IgnoredMissingPendingSchedule / IgnoredStalePendingSchedule` 结果；若平台为过期调度创建了 native timer，这层 outcome 还可继续产出 `StudioGuiPlatformTimerFollowUpCommand::ClearNativeTimer`，避免真实平台 glue 再手工推断资源回收动作
- 同步型平台 glue 当前也已具备一条更短的消费链：`native_timer_request -> execute_platform_timer_request(...) -> host_outcome/follow_up_command`，至少可覆盖立即返回平台 timer id 的 `set_timer` / `kill_timer` 风格 API
- 对于直接从宿主事件入口接同步 timer API 的平台，这条最短消费链当前又可继续收口为 `event/native callback -> dispatch_*_and_execute_platform_timer(...) -> dispatch + timer_execution`，让事件派发结果和平台 timer 执行结果继续留在同一层正式结果面
- 窗口布局、Canvas placement 坐标持久化和 Canvas unit layout nudge 继续与项目文档语义分离，当前保存到 `<project>.rfstudio-layout.json` sidecar，而不是混入 `*.rfproj.json`
- 多窗口布局 key 当前已从运行时 `window_id` 收口为基于 `window_role + layout_slot` 的稳定 scope，避免跨 host 重建时直接依赖临时窗口号

这意味着当前仓库已从“只有 UI 层快照映射桥”推进到“真实 `egui` 壳层可打开已有项目、持久化 shell 级最近项目、运行求解、查看结构化结果/诊断、定位 Inspector、审阅流股结果与单元最新执行结果、从求解步骤跳转到单元和产出流股、从失败摘要触发修复或定位目标、编辑 Stream 字段草稿并提交、执行基础撤销/重做、保存/另存为，并在画布中查看已有单元/流股/端口/诊断状态、执行多单元 pending placement 和离散调整已有 sidecar 单元位置”。最近项目写入独立 Studio preferences 文件，画布筛选、对比选择、focus navigation、layout nudge 与最近一次 command result 展示等状态仍属于 shell 临时状态或 Studio layout sidecar / presentation 反馈，不进入 `*.rfproj.json`。不过这仍是最小可操作工作台闭环，不等同于完整跨平台文件工作流、完整应用偏好系统、完整画布编辑器、结果报表系统、跨会话历史持久化或最终视觉方案。

这些决定的目的是先把 UI 和求解层之间的长期接口边界定清楚，再决定具体控件和交互实现。

### .NET 10 CAPE-OPEN Bridge

当前 `.NET 10` 目录已从单纯项目边界推进到 M4/M5 交界处的最小互调、正式 host-facing 消费面、外部 host 样例、受控 COM 注册工具和真实 `DWSIM / COFE` 人工复验阶段性收敛；但仍不默认进入 PME 自动化互调、第三方 CAPE-OPEN 模型加载或完整 Thermo PMC。

| 目录 | 当前职责 | 当前状态 |
| --- | --- | --- |
| `RadishFlow.CapeOpen.Interop` | 接口、GUID、异常语义 | 已建立第一版 `net10.0` 公共语义项目，当前覆盖 `ICapeIdentification`、`ICapeUtilities`、`ICapeUnit` 的最小接口骨架、最小 `IDispatch` marshalling 形状、已确认 CAPE-OPEN interface/category GUID 常量源，以及 `ECapeRoot` / `ECapeUser` / `ECapeBadInvOrder` / `ECapeBadCOParameter` 等最小异常契约、HRESULT 与语义化派生异常；当前也已声明与冻结 MVP TLB 一致的 assembly-level `Guid / TypeLibVersion`，并补齐 `Consumes Thermodynamics` / `Supports Thermodynamics 1.0` / `Supports Thermodynamics 1.1` category 常量，用于支持 classic late-bound 宿主与 DWSIM 画布接受条件 probe |
| `RadishFlow.CapeOpen.Adapter` | PInvoke 与句柄封装 | 已建立第一版 `net10.0` 薄适配项目，当前覆盖 native engine 句柄生命周期、UTF-8 字符串分配释放、`LibraryImport` 对 `rf-ffi` 的最小调用面，以及 `RfFfiStatus + last_error_message/json` 到更细粒度 ECape 语义异常的收口 |
| `RadishFlow.CapeOpen.UnitOp.Mvp` | 第一版自有 PMC | 已建立第一版 `net10.0` 最小 PMC 骨架项目，当前提供单个 `ICapeIdentification` + `ICapeUtilities` + `ICapeUnit` 实现类、最小状态机、最小 `ICapeCollection` / `ICapeParameter` / `ICapeUnitPort` 对象运行时、宿主生命周期访问守卫，以及经由 `RadishFlow.CapeOpen.Adapter` 调用 `rf-ffi` 的最小求解接线；`Calculate()` 对外结果面当前已收口为稳定的“成功结果 + 失败摘要”双契约，并进一步提供统一 report API、stable detail catalog、sectioned host report、configuration/action-plan/port-material/execution/session/view readers、action execution request planner / orchestrator、validation/calculation runner、host round orchestrator、supplemental mutation phase 与统一 follow-up / stop kind；当前又已通过 `UnitOperationComIdentity` 冻结 `CLSID / ProgID / Versioned ProgID / TypeLibraryId / TypeLibraryVersion`，并补入冻结的 `IDL` 真相源、可脚本再生成的 `TLB` 产物与 `scripts/gen-typelib.ps1`；当前 `UnitOp.Mvp` 程序集与主要 COM-visible class 已补齐 assembly-level TLB identity 与 `ComDefaultInterface`，Windows PowerShell 5 默认 `ICapeUtilities` late-bound 探测已通过，`ICapeUnit` 也已通过 `QueryInterface` 返回 `S_OK`；真实 PME 侧当前已确认 `DWSIM / COFE` 均可正常放置当前 PMC、连接 `Feed / Product` material streams，并在 water/ethanol 复验样例下完成 `Validate / Calculate` 收敛；COFE material object release warning、outlet not flashed 报错与 mass balance 警告已消失；DWSIM 兼容面要求 `ICapeUtilities` 前序 slot 保持 `Parameters get -> SimulationContext set -> Initialize -> Terminate -> Edit`，且 `Parameters.Item(i)` 返回对象本身可直接暴露 `ICapeIdentification / ICapeParameterSpec / ICapeOptionParameterSpec / ICapeParameter` |
| `RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests` | 库侧行为契约验证 | 已建立第一版 `net10.0` 自举式 contract test runner，不依赖外部 NuGet 测试框架；当前锁定 `Validate before Initialize`、validation failure、native failure、success、配置变更 invalidation 与 `Terminate()` 后阻断等核心契约，并固定 validation/native 两类 failure report 的 detail 字段缺省规则；当前还覆盖 assembly COM identity、PME material publication、feed material boundary overlay 与 water/ethanol 样例路径 |
| `RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost` | 最小外部 host 样例 | 已建立独立 `net10.0` console，当前通过 `PmeLikeUnitOperationHost / PmeLikeUnitOperationSession / PmeLikeUnitOperationInput` 演示外部宿主如何不依赖 `SmokeTests` driver DSL，直接复用正式 `host view / request planner / host round / session-execution-port-material-report` 消费面完成“创建组件、初始化、提交输入、validate/calculate round、读取结果、终止”；该样例仍不做 COM 注册、不驱动真实 PME、不加载第三方 CAPE-OPEN 模型 |
| `RadishFlow.CapeOpen.Registration` | 注册与反注册工具 | 已建立第一版 `net10.0` 注册工具，当前默认仍为 dry-run / preflight，但已支持在显式 `--execute` + `--confirm` 下执行 `register / unregister`；当前可输出 MVP Unit Operation PMC 的 `CLSID / ProgID / Versioned ProgID`、CAPE-OPEN categories、已实现接口清单、`register / unregister` 与 `current-user / local-machine` 下的 registry key plan，并检查真实 `UnitOp.Mvp` 输出目录中的 `.NET comhost` 路径、PE 机器类型、`UnitOp.Mvp.runtimeconfig/deps` sidecar、当前进程位数、scope 权限口径、目标 registry key 现状和备份范围；当前执行边界已收口到 confirmation token、preflight fail 阻断、HKLM elevation 检查、`CLSID / ProgID / Versioned ProgID / TypeLib` 四棵树备份、execution log 与失败 rollback。仓库根 `scripts/register-com.ps1` 与 `scripts/pme-register-latest.ps1` 当前已作为正式脚本入口封装这条路径；当前 `TypeLib` 标准注册、`CLSID\TypeLib` 关联、路径问题、Windows PowerShell 5 默认 `ICapeUtilities` late-bound 探测、thermodynamics implemented categories plan 与用户侧 `DWSIM / COFE` placement/connection/calculate 复验均已通过，并已沉淀为 `examples/pme-validation/` 下的正式验证记录；临时 COM trace 当前也已改为显式环境变量开关 |
| `RadishFlow.CapeOpen.SmokeTests` | 冒烟测试 | 已建立第一版最小 `net10.0` console，当前可配置 native library 目录、加载示例 flowsheet 与本地 package 文件、列出 package registry，并分别覆盖 direct adapter 的 flowsheet / stream snapshot 导出，以及 `UnitOp.Mvp` 的最小成功结果契约、失败摘要契约、统一只读 report access、configuration/action-plan/port-material/execution/session 五条宿主只读路径；其中 session 当前已进一步带 canonical state；`unitop` 路径当前又已收口为“最小外部宿主验证骨架”，显式固定 `Initialize -> 配参数 -> 连端口 -> Validate -> Calculate -> 读结果 -> Terminate` 调用顺序，形成 `driver + boundary suite + session catalog + Program 调度` 四层结构，并支持 `--unitop-scenario <all|session|recovery|shutdown>` 按宿主时序场景过滤 |

### External .NET 10 Control Plane

外部控制面当前不在本仓库内实现，但系统级职责与技术口径已经冻结：

| 组件 | 当前职责 | 当前状态 |
| --- | --- | --- |
| `Radish.Auth` | OIDC 身份源与统一登录 | 外部平台依赖 |
| `RadishFlow Control Plane` | `ASP.NET Core / .NET 10` 授权、manifest、lease、offline refresh、audit API | 体系结构已冻结；当前仓库已有客户端 DTO 与 HTTP 接线 |
| `Asset Delivery` | 物性派生包下载入口，优先承载为对象存储 / CDN / 下载网关 | 体系结构已冻结；当前仓库只消费下载协议 |

这里的关键点不是“把服务端代码也搬进当前 Monorepo”，而是先把客户端、桥接层与控制面之间的长期契约固定住。

## 当前关键边界

第一阶段必须严格遵守以下边界：

- Rust 不直接处理 COM、`IDispatch`、`VARIANT`、`SAFEARRAY`
- CAPE-OPEN/COM 适配全部放在 `.NET 10` 中
- 第一阶段只导出自有 Unit Operation PMC，不支持加载第三方 CAPE-OPEN 模型
- Rust 与 .NET 边界只允许句柄、基础数值、UTF-8 字符串和 JSON
- 桌面端登录统一走 OIDC Authorization Code + PKCE，不内置长期 `client_secret`
- 高价值物性资产不默认完整下发到客户端
- 远端服务只承担控制面与资产分发面，不吞掉本地求解热路径
- 外部控制面建议采用 `ASP.NET Core / .NET 10`，不额外引入新的 Go 服务主线
- 资产分发优先采用对象存储 / CDN + 短时票据，而不是让控制面 API 长期直出大文件

## 当前开发策略

当前开发顺序不是“谁都做一点”，而是明显偏向内核优先：

1. 先稳定 `rf-types`、`rf-model`、`rf-thermo`、`rf-flash`
2. 再进入 `rf-unitops`、`rf-flowsheet`、`rf-solver`
3. 再做 `rf-ffi`
4. 最后才让 `.NET 10` 适配层真正接入运行时
5. 在桌面边界稳定后，按既有契约单独推进外部 `.NET 10` 控制面落地与部署

这个顺序的目的，是把数值问题和 COM 互操作问题分开定位，避免后期排错混杂。

## 当前阶段优先级调整

虽然主线顺序仍然保持不变，但当前短期优先级已从单纯“地基建设优先”调整为“在已冻结的边界上恢复 Rust Studio 最小工作台闭环”。

当前应优先推进的内容：

- 保持仓库治理、分支与 PR 规则、基础 CI、代码与文档格式规范稳定
- 沿 Studio 既有应用层边界继续补“打开项目 / 运行求解 / 查看结果与诊断”的可见工作台闭环
- 重要阶段性边界变化继续同步到正式设计、规划和开发日志文档

原因：

- 当前仓库仍处于早期演化阶段，过早推进主线功能，后续反而要回头返工协作规则和工程基础设施
- App 主界面、内核、适配层都还没有稳定的工程协作口径，先定规则更划算

这并不意味着放弃主线，而是先把主线开发赖以生存的仓库地基补完整。

对 Studio 来说，这也意味着短期内仍优先沿既有 `StudioAppFacade`、运行栏 DTO、GUI-facing snapshot 和 presentation 边界补用户可见闭环；等真正进入完整界面设计和交互方案收口阶段，再显式拉用户一起确认，而不是在无感知状态下直接把产品 UI 方案推到深水区。

## 初始化阶段结论

截至 2026-03-29，仓库初始化阶段已经从“纯目录骨架”进入“可继续开发的基础结构”阶段。  
接下来的重点不再是增加目录，而是继续补充算法、测试覆盖和最小闭环样例厚度。
