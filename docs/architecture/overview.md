# Architecture Overview

更新时间：2026-04-08

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
| `rf-ffi` | Rust 与 .NET 的 C ABI 边界 | 仍为占位 |

当前仓库级集成测试也已正式落到 `tests/rust-integration` workspace crate，并由 `cargo test --workspace`、`scripts/check-repo.ps1` 与 `scripts/check-repo.sh` 自动覆盖五条示例 flowsheet 回归。

### Rust Studio UI

当前 UI 相关 crate 已进入“状态骨架与应用边界冻结”阶段，但仍未进入具体界面实现主线：

| crate | 当前职责 | 当前状态 |
| --- | --- | --- |
| `rf-ui` | UI 状态与行为逻辑 | 已建立 `AppState`、授权态、求解态与控制面 DTO 骨架；已补 `RunPanelState`、`RunPanelIntent`、`RunPanelCommandModel`、`RunPanelViewModel`、`RunPanelPresentation` 与 `RunPanelWidgetModel`，并可把 `rf-solver::SolveSnapshot` 映射为 UI 层结果快照 |
| `rf-canvas` | 流程图画布能力 | 占位 |
| `apps/radishflow-studio` | 桌面入口程序 | 已建立 auth cache sync 桥接、控制面 HTTP client、entitlement / manifest / lease / offline refresh 编排、下载获取抽象、基于 `reqwest + rustls` 的真实 HTTP transport、HTTP 请求/响应适配层、可重试/不可重试失败分类、下载 JSON 到本地 payload DTO 的协议映射、摘要校验、失败回滚与测试；并已补上 `PropertyPackageProvider -> rf-solver -> rf-ui::AppState` 的最小工作区求解桥接，可直接基于已加载物性包或本地 auth cache 执行真实 solve 并回写 UI 快照/日志；当前又已形成 `StudioGuiHost / StudioGuiDriver / StudioGuiSnapshot / StudioGuiWindowModel / StudioGuiWindowLayoutState` 这一条 GUI-facing 宿主与窗口布局契约，并把窗口布局持久化为项目同目录 sidecar；当前又已把 drop preview 前推为 `StudioGuiWindowDropTargetQuery -> StudioGuiHost / StudioGuiDriver` 的显式查询入口，并在 host 内补出非持久化 preview 会话态；当前又已补出 `StudioGuiNativeTimerRuntime`，让 GUI 可在消费 `StudioGuiNativeTimerEffects` 后继续跟踪逻辑 timer handle、`next_due_at` 和一次性 due callback，而不必在真实框架里从零重写同一套 timer 生命周期；当前又已把原生 timer callback 正式收口为 `StudioGuiEvent::NativeTimerElapsed { window_id, handle_id }`，由 driver 先校验当前绑定再回灌 `TimerElapsed`，避免真实宿主把 stale callback 误灌进 runtime；当前又已补出 `StudioGuiPlatformHost`，把“driver 派发后比较下一次 pending binding，并向平台发出携带 `window_id + handle_id + slot/due_at` 的 `Arm/Rearm/Clear` 请求”的逻辑固定在平台适配层，未来真实 GUI 不必再在框架入口手工维护 timer 差分或丢失 callback 身份；当前又已补出 `StudioGuiPlatformTimerDriverState`，把平台 native timer id 与逻辑 binding 的映射、rearm/clear 所需的旧平台句柄，以及“native timer callback -> `window_id + handle_id` 回灌目标”的桥接固定为显式状态机；当前这层状态机也已被 `StudioGuiPlatformHost` 正式持有，平台 timer 创建失败除了并入 `snapshot/window_model.runtime.log_entries`，还会直接进入 `runtime.platform_notice`；native callback 若命中不存在或过期的 `native_timer_id`，当前也已收口为 `StudioGuiPlatformNativeTimerCallbackOutcome::{IgnoredUnknownNativeTimer, IgnoredStaleNativeTimer}`，不再把这类真实平台边界当成上抛错误 |

原因很直接：在 `M2/M3` 之前过早推进 UI，会掩盖内核尚未定型的问题。

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
- `StudioGuiPlatformHost` 作为平台 timer 调度适配层，负责把下一条 pending timer binding 的前后变化收口为平台侧 `Arm / Rearm / Clear` 请求，并持有平台 timer adapter、平台失败日志与 GUI 可直接消费的 `platform_notice`
- `StudioGuiPlatformTimerDriverState` 作为平台 native timer 适配层，负责消费上述请求、保存当前 native timer id 与逻辑 binding 的映射，并在 callback 到来时反查回 `window_id + handle_id`
- 平台若按 `native_timer_id` 回灌 callback，当前应优先消费 `StudioGuiPlatformHost::dispatch_native_timer_elapsed_by_native_id(...)` 的正式 outcome；命中有效映射时继续分发，命中不存在或过期 id 时返回显式 ignored outcome，而不是把平台层常见竞态继续上抛为 `RfError`
- `StudioGuiSnapshot` 作为跨模块聚合快照真相源
- `StudioGuiWindowModel` 作为窗口内容分区模型
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
- `StudioGuiPlatformHost` 当前会在每次事件派发和 due timer 排空后比较前后 pending timer binding，把平台真正需要执行的 timer 调度差异收口为显式 `native_timer_request`，并继续携带 `window_id / handle_id / slot`
- `StudioGuiPlatformTimerDriverState` 当前会把这份 request 继续收口为平台可执行的 `Arm / Rearm / Clear` 命令，并在 native timer 创建后记录 `native_timer_id -> logical binding` 映射；平台若创建失败，也已有显式 failure ack 用于清理 pending 状态
- 平台 native timer callback 当前也已继续收口为 `Dispatched / IgnoredUnknownNativeTimer / IgnoredStaleNativeTimer` 三类正式结果，真实 GUI 或框架 glue 可直接按 outcome 决定是否忽略，无需再把 stale/missing callback 包装成错误流
- 窗口布局持久化继续与项目文档语义分离，当前保存到 `<project>.rfstudio-layout.json` sidecar，而不是混入 `*.rfproj.json`
- 多窗口布局 key 当前已从运行时 `window_id` 收口为基于 `window_role + layout_slot` 的稳定 scope，避免跨 host 重建时直接依赖临时窗口号

这意味着当前仓库已从“只有 UI 层快照映射桥”推进到“应用组合层已有 facade / command 入口可驱动真实求解”，但仍未把这条入口接成最终桌面命令或交互动作。

这些决定的目的是先把 UI 和求解层之间的长期接口边界定清楚，再决定具体控件和交互实现。

### .NET 10 CAPE-OPEN Bridge

当前 `.NET 10` 目录只冻结项目边界，不进入复杂实现：

| 目录 | 当前职责 | 当前状态 |
| --- | --- | --- |
| `RadishFlow.CapeOpen.Interop` | 接口、GUID、异常语义 | 目录占位 |
| `RadishFlow.CapeOpen.Adapter` | PInvoke 与句柄封装 | 目录占位 |
| `RadishFlow.CapeOpen.UnitOp.Mvp` | 第一版自有 PMC | 目录占位 |
| `RadishFlow.CapeOpen.Registration` | 注册与反注册工具 | 目录占位 |
| `RadishFlow.CapeOpen.SmokeTests` | 冒烟测试 | 目录占位 |

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

虽然主线顺序仍然保持不变，但当前短期优先级已调整为“地基建设优先”。

当前应优先推进的内容：

- 仓库治理
- 分支与 PR 规则
- 基础 CI
- 代码与文档格式规范
- App 架构规划
- 设计与进度文档完善

原因：

- 当前仓库仍处于早期演化阶段，过早推进主线功能，后续反而要回头返工协作规则和工程基础设施
- App 主界面、内核、适配层都还没有稳定的工程协作口径，先定规则更划算

这并不意味着放弃主线，而是先把主线开发赖以生存的仓库地基补完整。

## 初始化阶段结论

截至 2026-03-29，仓库初始化阶段已经从“纯目录骨架”进入“可继续开发的基础结构”阶段。  
接下来的重点不再是增加目录，而是继续补充算法、测试覆盖和最小闭环样例厚度。
