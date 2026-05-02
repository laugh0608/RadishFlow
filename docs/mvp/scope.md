# MVP Scope

更新时间：2026-05-01

## MVP 目标

第一阶段 MVP 目标保持不变：

构建一个以 Rust 为核心、以 Rust UI 为主界面、以 .NET 10 暴露 CAPE-OPEN Unit Operation PMC 的最小稳态流程模拟闭环，并让至少一个自有单元模型可被外部 PME 识别与调用。

## 当前冻结范围

第一阶段当前冻结为以下内容：

- 二元体系
- 最小物性参数集
- 简化热力学模型
- `TP Flash`
- 流股对象
- 单元模块：`Feed`、`Mixer`、`Heater/Cooler`、`Valve`、`Flash Drum`
- 无回路或极简回路的顺序模块法
- JSON 项目格式
- 一个可注册的自有 CAPE-OPEN Unit Operation PMC

## 明确不做

以下内容当前阶段明确不做：

- 加载第三方 CAPE-OPEN 单元
- 加载第三方 CAPE-OPEN Thermo/Property Package
- 完整 Thermodynamics PMC
- recycle 全功能收敛
- 动态模拟
- 大规模组分数据库
- UI 视觉精修优先级高于内核闭环

## 当前阶段细化决策

为避免范围漂移，当前阶段补充冻结以下实现细节：

- 统一使用 SI 基本单位，温度用 K，压力用 Pa，摩尔流量用 mol/s
- 流股组成先统一为摩尔分率，不在第一轮引入质量基和体积分率切换
- 相标签当前只保留 `overall`、`liquid`、`vapor`
- `rf-model` 只负责对象模型，不先塞进求解策略和 COM 语义
- `rf-thermo` 与 `rf-flash` 先定接口，再补 Antoine、Raoult 和 Rachford-Rice
- `rf-unitops` 第一轮统一围绕标准 `MaterialStreamState` 输入输出，不提前把 flowsheet 调度或 FFI 细节塞进单元接口
- `Feed`、`Mixer`、`Flash Drum` 当前先冻结为 canonical material ports：`Feed(outlet)`、`Mixer(inlet_a/inlet_b/outlet)`、`Flash Drum(inlet/liquid/vapor)`
- `rf-flowsheet` 第一轮连接校验只覆盖 canonical material ports、流股存在性与“一股一源一汇”；终端产品流允许只有 source、没有 sink
- `.NET 10` 适配层当前允许推进到 `M4/M5` 交界的最小互调与 `UnitOp.Mvp` 宿主语义收口，但范围只限于 `rf-ffi` 薄适配、`UnitOp.Mvp` 对象面、contract/smoke 基线、库内只读宿主模型，以及 action execution request planning / orchestration result / host view snapshot / validation outcome / calculation outcome / host round outcome / follow-up 这类不承担完整宿主生命周期的薄 helper
- `UnitOp.Mvp` 当前应优先把宿主语义收口为正式只读模型、显式请求规划模型或窄边界 outcome 模型，例如 configuration snapshot、action plan、action execution request plan、action execution orchestration result、host view snapshot、port/material snapshot、execution snapshot、session snapshot、canonical session state、validation outcome、calculation outcome、host round outcome、统一 follow-up 与 stop kind，不把组合逻辑散落到 smoke host、测试字面量或未来 PME 入口
- `UnitOp.Mvp` 当前也已补出独立 `SampleHost`，用于证明未来 PME host / 其他宿主可直接复用上述正式消费面，而不是继续依赖 smoke driver
- `SampleHost` 当前又已补出 `PmeLikeUnitOperationHost / PmeLikeUnitOperationSession / PmeLikeUnitOperationInput` 薄宿主入口，把“创建组件、初始化、读取视图、提交参数/端口对象、执行 validate/calculate round、读取正式结果面、终止”整理成更接近 PME host 的最小 session 形状；这层仍只消费 `UnitOp.Mvp` 正式 reader / planner / host round，不引入 COM 注册、PME 自动化互调或第三方模型加载
- `UnitOp.Mvp` 当前已冻结自有 MVP Unit Operation PMC 的 `CLSID / ProgID / Versioned ProgID`，并新增带执行门控的 `RadishFlow.CapeOpen.Registration`；当前默认仍是 dry-run，但已支持在显式 `--execute` + `--confirm` 下执行 `register / unregister`，并收口 preflight fail 阻断、HKLM elevation 检查、registry plan 限界、三棵树 JSON 备份、execution log 与失败 rollback；这不代表当前阶段已经默认注册 COM 或驱动 PME
- 仓库根 `scripts/register-com.ps1` 当前已作为正式注册脚本入口，负责 build、环境变量重定向、confirmation token 提示与 `Registration.exe` 转调；本机 `current-user register/unregister` 闭环验证当前已通过这条入口完成一次顺序复查
- `docs/capeopen/pme-validation.md` 当前已补出目标 PME 人工验证说明，冻结执行前验证基线、dry-run 审查项、执行型注册门控、安装/反安装运行手册、人工 PME 验证路径、通过标准、失败分类与验证记录模板；`examples/pme-validation/` 当前也已补出可复用模板；这一步只把真实 PME 前置路径文档化，不代表当前阶段已经进入默认 COM 注册或 PME 自动化互调
- `DWSIM / COFE` 人工复验当前已把 discovery、activation、placement、端口连接与最小 `Validate / Calculate` 主路径推进到阶段性闭环：两者均能发现并放置当前 PMC，也能连接 `Feed / Product` material streams；COFE material object release warning、outlet not flashed 报错与 mass balance 警告均已在 water/ethanol 复验样例下收敛
- 当前为 DWSIM 画布接受条件已补齐 `Consumes Thermodynamics`、`Supports Thermodynamics 1.0` 与 `Supports Thermodynamics 1.1` 注册分类，但这只是 discovery/acceptance 层兼容 probe，不改变 MVP 不实现完整 Thermo PMC、不加载第三方 property package 的范围边界
- 当前已把 `ICapeUtilities` 前序 slot 调整为 `Parameters get -> SimulationContext set -> Initialize -> Terminate -> Edit`，并把 COFE 需要的 `SimulationContext` getter 保留为 `Edit` 之后的同 `DispId(2)` late-bound getter；端口连接当前允许在连接期间保留 live PME material object 引用，用于短生命周期读取 Feed material 与写回 Product material，并在断开/终止时释放本 UnitOp 持有的 RCW。DWSIM parameter enumeration 要求 `Parameters.Item(i)` 返回对象本身同时支持 `ICapeIdentification / ICapeParameterSpec / ICapeOptionParameterSpec / ICapeParameter`，这一路径已纳入 contract test
- 当前明确不继续线性堆叠 calculation report accessor；若宿主需要更高层语义，应优先在库内增加 reader / snapshot / presentation，而不是继续在 PMC 主类追加 convenience API
- 当前仍不提前展开 COM 注册、PME 互调壳、第三方 CAPE-OPEN 模型加载或完整外部 Thermo/Property Package 宿主兼容

App 与交互层当前进一步冻结以下口径：

- MVP 保持单文档工作区，不急于做多文档容器
- 单文档工作区不等于单文件实现，源码仍按职责拆分
- 属性编辑采用字段级草稿态，语义提交后才写回文档
- 求解控制采用 `SimulationMode(Active/Hold)` 与 `RunStatus` 分离模型
- 求解结果采用独立 `SolveSnapshot`，不直接覆盖文档对象
- 结果快照应保留按步展开能力，为撤回/前进和脚本化扩展留接口
- `DocumentMetadata` 只保存文档身份与序列化元信息，不保存文件路径、求解态和用户偏好
- `UserPreferences` 只保存应用级偏好与快照窗口策略，不污染文档语义
- `CommandHistory` 只记录语义化文档命令，运行控制和文档生命周期动作不进入撤回栈
- `SolveSessionState` 必须绑定当前观察的文档修订号，`SolveSnapshot` 由工作区持有有界历史窗口
- Studio 当前 GUI-facing 宿主边界已形成 `StudioGuiHost + StudioGuiDriver + StudioGuiSnapshot + StudioGuiWindowModel + StudioGuiWindowLayoutState` 这一条正式契约，不再要求 `main.rs` 或未来真实 GUI 手工拼装窗口摘要
- Studio 当前 GUI 命令面也已进一步收口为 `StudioGuiCommandRegistry + StudioGuiShortcutRouter + dispatch_ui_command(command_id)` 这一条统一入口，至少覆盖 run panel 与 canvas suggestion 两类命令；未来真实 GUI 不应再长期保留 widget 私有 typed action 与正式 command id 并行的双轨接线
- Studio 当前窗口布局状态已冻结为独立 UI 状态面，覆盖 `panel dock_region/stack_group/visibility/collapsed/order`、stack active tab、region 内 stack placement、`center_area`、`region_weights`、多窗口 `layout scope` 与 GUI-facing `drop target` 摘要推导
- Studio 当前也已把 tab 展示角色冻结到 `StudioGuiWindowPanelLayout`，显式区分 `Standalone / ActiveTab / InactiveTab`，不让真实 GUI 再自行猜测 tab 化 panel 的展示模式
- Studio 当前也已把 tab strip 交互纳入正式 mutation，至少覆盖 active tab 切换、前后循环、stack 内重排和 unstack，不再把这几类行为留给 GUI 框架私有状态
- Studio 当前又已把 drop preview 查询正式前推到 `StudioGuiWindowDropTargetQuery + StudioGuiHost / StudioGuiDriver` 入口；未来真实 GUI 应按 `window_id + hover/anchor/placement` 请求预览，而不是继续读取 layout 内部状态后手工拼 mutation
- Studio 当前又已把 query 结果扩成 `drop_target + preview_layout_state + preview_window`，让真实 GUI 在 hover 时可以直接消费预览态窗口模型，而不必自己再从摘要重建 tabbed/dock 结果
- Studio 当前又已把 drop release 正式前推到同一套 query 词汇，新增 `ApplyWindowDropTarget / WindowDropTargetApplyRequested`，让 GUI 侧不必继续维护“预览用 query / 落地用 mutation”两套接口
- Studio 当前又已把 hover 预览前推为显式会话态，新增 `SetWindowDropTargetPreview / ClearWindowDropTargetPreview` 与 `WindowDropTargetPreviewRequested / WindowDropTargetPreviewCleared`；host 会非持久化保存当前 preview，并通过 `StudioGuiSnapshot / StudioGuiWindowModel.drop_preview` 直接暴露给 GUI
- Studio 当前又已把 `drop_preview` 继续收口为 GUI-facing presentation，直接携带 `preview_layout + changed_area_ids`，让 GUI 不必自己从当前态/预览态做差分才能画出 hover 预览
- Studio 当前又已把 `drop_preview` 继续补成 overlay DTO，直接携带目标 `dock_region/stack_group/tab_index`、目标 stack tabs、高亮 area 集与 active tab，减少真实 GUI 对底层摘要字段的二次拆解
- 第一版 `eframe/egui` GUI 壳当前已直接消费这份 `drop_preview.overlay`，把局部插入条、anchor 顶线、新 stack 占位、target-anchored 浮动 overlay 与局部 hint pill 画在目标位置，不再主要依赖顶栏说明文本
- 当前 GUI 壳仍冻结在“单原生窗口承载逻辑窗口切换”的边界，不在这一阶段展开多原生窗口宿主
- Studio 当前多窗口布局 scope 已从运行时 `window_id` 收口到基于 `window_role + layout_slot` 的稳定 key，避免布局恢复直接依赖临时窗口号
- Studio 当前又已把原生 timer 宿主 glue 冻结为 `StudioGuiNativeTimerRuntime + StudioGuiPlatformHost + StudioGuiPlatformTimerDriverState` 三层边界，真实桌面框架后续不应再在入口层自行维护逻辑 binding、平台 native timer id 映射和 stale callback 判定
- Studio 当前平台 timer 回灌与执行口径又已进一步冻结为：
- 平台 request / command：`StudioGuiPlatformTimerRequest` / `StudioGuiPlatformTimerCommand`
- start ack / failure ack：`acknowledge_platform_timer_started(...)` / `acknowledge_platform_timer_start_failed(...)`
- callback outcome：`dispatch_native_timer_elapsed_by_native_id(...)`
- batch / round 宿主结果：`dispatch_native_timer_elapsed_by_native_ids(...)`、`dispatch_due_native_timer_events_batch(...)`、`process_async_platform_round(...)`
- batch / round 执行型宿主结果：`dispatch_native_timer_elapsed_by_native_ids_and_execute_platform_timers(...)`、`drain_due_native_timer_events_and_execute_platform_timers(...)`、`process_async_platform_round_and_execute_actions(...)`
- Studio 当前 async round 动作顺序也已冻结为 `follow_up cleanup -> timer request`；真实桌面框架应优先复用 `StudioGuiPlatformAsyncRound::actions()` 或 executed async round，而不是在框架层重复归并和排序
- Studio 当前应用层运行入口先冻结为 `StudioAppFacade + WorkspaceRunCommand + WorkspaceSolveService + solver_bridge` 四层，不让 UI 直接拼接底层 provider/solver 细节
- `rf-ui` 当前运行栏状态先冻结为 `RunPanelState + RunPanelIntent + RunPanelCommandModel + RunPanelWidgetModel`，把按钮意图、主动作、按钮槽位、文本布局和最小渲染/触发所需状态都留在 UI 层，不让视图层或 Studio 侧重复发明一套按钮语义
- Studio 当前对运行栏的最小消费也已前推到 `RunPanelWidgetEvent`，不再只接受裸 `RunPanelIntent`
- Studio 当前也已补出 `run_panel_driver`，把最小运行栏驱动逻辑留在应用层，而不散落在入口层
- 当前最小桌面入口 `run_studio_bootstrap` 也已补出 `StudioBootstrapTrigger`，允许样例入口显式选择“走 intent 触发”“走主按钮触发”或“走指定 widget action 触发”
- Studio 当前运行触发先明确区分 `Manual` / `Automatic`，并把 `SimulationMode` / `pending_reason` 的运行门控收口在应用层
- Studio 当前默认包选择采取保守策略：只有唯一候选包明确时才自动选中；多包场景必须显式指定 package，不在当前阶段隐式猜包
- Studio 当前 Automatic 触发在命中 `HoldMode` / `NoPendingRequest` 时应先返回 skip，再决定是否需要 package 解析，避免多包缓存场景下的无意义失败
- 当前最小桌面入口 `run_studio_bootstrap` 也已改为默认走 `StudioBootstrapTrigger::WidgetPrimaryAction`，并向入口层输出 `RunPanelWidgetModel`，确保“桌面触发点 -> UI 组件动作 -> Studio driver / 控制动作 -> UI 组件 DTO”边界在样例入口里就成立
- 当前 `egui` Studio 壳已开始消费上述运行入口与 `SolveSnapshot` presentation：Runtime 面板可切换仓库内置正向示例项目、触发运行、按 summary / overall composition / phases 结构化显示流股结果，并展示求解步骤、诊断、日志；这一路径继续保持 `StudioAppFacade -> WorkspaceRunCommand -> WorkspaceSolveService -> solver_bridge` 边界不被绕过
- 当前 `egui` Studio 壳又已补出项目打开入口：用户可通过路径输入或 Windows 原生文件选择器打开现有 `*.rfproj.json`，打开会重建当前 Studio runtime，打开失败会保留当前工作区并显示错误反馈；内置示例切换复用这条打开流程；若当前文档存在未保存修订，则先进入显式确认状态，避免静默丢弃当前上下文；打开成功后会记录到 shell 级最近项目列表并写入独立 Studio preferences 文件，重启后会恢复该列表；点击最近项目继续复用相同打开流程与未保存确认保护
- 当前 `egui` Studio 壳又已补出最小 Result Inspector、失败结果 presentation、诊断目标定位命令、活动 Inspector 详情、通用 action DTO、Stream Inspector 字段级 presentation、字段 draft update / 单字段 commit / 多字段批量 commit driver，以及基础 `edit.undo / edit.redo` 文档历史命令；Stream Inspector 字段当前覆盖 `name / temperature_k / pressure_pa / total_molar_flow_mol_s` 与已有 `overall_mole_fractions` 组分条目，所有入口均通过正式 command / driver / runtime 边界执行，不由 shell 直接写 `FlowsheetDocument`
- 当前画布编辑前置状态先冻结为 `CanvasEditIntent` transient state：`begin_place_unit` 只表达“准备放置某类单元”的意图，不写入 `FlowsheetDocument`、不递增 revision、也不进入 `CommandHistory`；`commit_canvas_pending_edit_at(CanvasPoint)` 会把当前 `PlaceUnit` 意图提交为带 canonical ports 的 `DocumentCommand::CreateUnit`，并把动态落点留在 commit result 中供后续布局状态消费；文档语义变化会清理 pending edit。GUI 侧当前已通过 `StudioGuiCanvasState / StudioGuiCanvasPresentation / canvas.cancel_pending_edit` 展示和取消当前意图，并在 `egui` Canvas 面板中接入单类型 `Place Flash Drum -> BeginPlaceUnit -> 点击落点提交` 的最小入口；这仍不代表已经实现完整画布单元创建器或项目级坐标持久化
- 当前 `egui` Studio 壳又已补出 Canvas 最小可见与只读扫读层：已有 unit 会投影为临时布局单元块，已有 material stream 绑定会投影为物流线，活动 Inspector 目标会驱动画布 selection 与 focus callout；对象列表会统一展示 unit / stream，并可按 `All / Attention / Units / Streams` 临时筛选；material port marker、端口 hover、运行/诊断 badge 只帮助扫读已有绑定和诊断目标，不引入端口点击编辑、连线创建、拖拽布局、坐标持久化或项目 schema 扩张
- 当前 `egui` Studio 壳又已补出 `file.save` 与 `Save As` 最小项目持久化闭环：保存命令经由 `StudioRuntimeTrigger::DocumentLifecycle` 写回当前 `*.rfproj.json`，成功后刷新 `last_saved_revision / has_unsaved_changes`；另存为通过 Windows 原生保存选择器写入新路径，并更新当前项目路径、项目路径输入框与最近项目列表；若 `Save As` 目标文件已存在且不是当前项目路径，shell 先进入显式覆盖确认，确认后才写入，取消则保留当前工作区和目标文件
- 当前 `rf-store::write_project_file` 已改为同目录 staged write：先写临时 sibling 并同步，再替换正式项目文件；Unix 类平台使用 `rename` 替换语义，Windows 当前用临时备份做受控替换和失败回滚，避免半写入 JSON 直接污染项目文件
- 当前 Studio 字段编辑快捷键策略已冻结为最小安全闭环：`Ctrl+S` 即使在文本输入焦点下也触发 `file.save`；`Ctrl+Z / Ctrl+Y` 在普通焦点下触发 `edit.undo / edit.redo`，但文本输入焦点下保留给输入框自身的编辑撤销/重做；Stream Inspector 输入框的 `Enter` 只提交当前字段，不隐式触发 `Apply all`
- 中文/英文切换当前只属于 GUI shell 偏好，不写入 `DocumentMetadata`、`UserPreferences` 或项目文件；系统 CJK 字体 fallback 也只在应用启动时配置，不新增仓库字体资产
- 当前仍不把 UI 范围扩张到完整视觉设计、结果导出、跨会话历史持久化或完整画布编辑体验；当前原生文件选择器只覆盖 Windows 打开与另存为，不承诺跨平台文件工作流；当前最近项目持久化只覆盖 shell 级 MRU 路径列表，不等同于完整应用偏好系统；当前 Inspector、undo/redo、保存、快捷键、覆盖确认、画布 pending edit、`egui` 单类型放置入口、最小单元创建提交、只读单元/连线/端口/诊断可视化与对象列表筛选仍是最小可操作边界，后续更完整画布编辑仍应先补正式 presentation / command / state 边界再进入真实 UI

流程图交互增强方向当前补充冻结以下边界：

- 后续允许流程图画布在平面视图与立体投影视图之间切换，但底层继续共享同一份 flowsheet 语义、项目文件与命令历史，不为 3D 单独引入第二套文档模型
- 物流线、能量流线、信号流线的可视化后续允许支持静态/动态双模式，但当前阶段不为了表现层效果提前扩张 `rf-model` / `rf-flowsheet` 的核心语义
- 流线类型与运行状态后续应采用正交表达：类型优先靠主色区分，状态优先靠线型、透明度、饱和度、方向箭头动效或状态徽标区分，不先把单一配色方案写死到产品语义
- 对标准单元放置后的“待补全入口/出口/连线”后续允许以灰态 ghost 形态显示，并通过 `Tab` 或明确接受动作补全；未接受前不直接写回正式文档
- `RadishMind` 后续只作为建议与预测的辅助来源，不替代本地 canonical port 规则、连接校验、求解器诊断与文档命令边界

认证、授权与受控物性资产当前进一步冻结以下口径：

- 桌面端统一走 `OIDC Authorization Code + PKCE`
- RadishFlow 桌面端是 `public client`，不内置长期 `client_secret`
- 登录默认采用系统浏览器 + loopback redirect，不照搬 Web 客户端 `localStorage` token 方案
- Access Token / Refresh Token 只允许落在操作系统安全存储
- 外部控制面默认采用 `ASP.NET Core / .NET 10`，不额外引入 Go 服务主线
- 高价值原始物性资产不默认完整下发到客户端
- 本地求解热路径继续本地执行，远端服务只承担身份、授权、租约、清单和派生包分发
- 派生物性包分发优先采用对象存储 / CDN / 下载网关 + 短时票据，不把控制面 API 设计成长时大文件出口
- 允许引入离线租约与本地派生物性包缓存，但不承诺客户端绝对防提取
- 项目文件继续固定为单文件 `*.rfproj.json` 真相源，授权缓存索引与派生包缓存继续留在应用私有缓存根目录
- MVP 默认不把 `snapshot_history`、token 明文、授权缓存索引或 Studio 窗口布局状态混进项目文件
- Studio 当前窗口布局偏好已冻结为项目同目录 sidecar：`<project>.rfstudio-layout.json`
- 桌面交付默认采用“压缩包 + 主入口 + 附带资源目录”的原生客户端形态，不以单文件可执行为当前阶段目标

## 当前阶段优先目标

在真正恢复主线功能推进前，当前阶段优先目标曾调整为仓库地基建设：

- 完善仓库规范
- 完善代码与文档格式规范
- 建立分支、PR 和 CI 基线
- 完善 App 架构规划
- 完善设计文档与进度文档

当前判断逻辑是：

- 这些工作不直接产出功能，但会决定后续功能开发是否可持续
- 在仓库还很新时完成这些约束，成本远低于中后期补治理
- 当前主线还没有复杂历史包袱，适合现在就冻结工程基础口径

截至 2026-04-27，CAPE-OPEN / PME 验证基线已阶段性冻结，仓库地基也已足以支撑短线主线回到 Rust Studio 的最小可操作工作台闭环；后续应继续保持边界清晰和验证稳定，但不再把“地基建设”作为阻止 Studio 可见闭环推进的理由。

## 近期开发节奏

当前建议以周为单位推进，先把主线拆细：

### 2026-W13

- 完成仓库骨架初始化提交
- 建立第一批 Rust 基础类型和领域模型骨架
- 完善初始化文档、协作约定与周志体系

### 2026-W14

- 完善分支与 PR 治理规则
- 建立 GitHub Actions PR 检查
- 建立文本编码、文件格式与 Rust 基础验证脚本
- 完善 App 架构与当前阶段开发规划文档

### 2026-W15

- 冻结 `AppState`、`WorkspaceState`、`FlowsheetDocument`、`DocumentMetadata`、`UserPreferences` 的字段边界
- 冻结字段级草稿提交流程、`CommandHistory` 边界和工作区保存态口径
- 冻结 `SolveSessionState`、`DiagnosticSummary`、`SolvePendingReason` 与 `SolveSnapshot` 的关系
- 明确 UI 交互层、求解层和快照历史之间的数据所有权
- 冻结桌面登录、授权、离线租约与远端资产控制面的总体边界
- 冻结 `StoredProjectFile` / `StoredAuthCacheIndex` JSON DTO、相对缓存路径布局、客户端注册与 scope 命名

补充说明：

- 截至 2026-03-29，上述大部分基础冻结项已提前完成，不再视为后续待办
- 截至 2026-03-29，本地 `PropertyPackageManifest` / `payload` 实体读写、`PropertyPackageProvider` 的本地缓存接线、下载落盘路径、下载 JSON 到本地 payload 的映射和首个真实样例包也已提前完成
- 截至 2026-03-29，下载获取抽象、基于规范化 payload 的摘要校验、失败回滚和样例摘要也已提前收口完成
- 截至 2026-03-29，下载抓取失败分类与有限次重试策略也已提前收口完成
- 截至 2026-03-29，原始 HTTP 请求/响应适配层与状态码到失败分类的映射也已提前收口完成
- 截至 2026-03-29，基于 `reqwest + rustls` 的真实 HTTP client adapter 也已提前收口完成
- 截至 2026-03-29，控制面 `entitlement / manifest / lease / offline refresh` HTTP client 与应用层编排也已提前收口完成
- 当前剩余重点已经进一步转向联网失败策略细化、数值主线和求解闭环，而不是继续停留在第一轮 DTO 草案

补充对齐：

- 今天（2026-03-30）优先在已接通的控制面 client 与应用层编排之上，细化授权刷新后的 UI 事件流、联网失败提示和离线刷新触发策略
- 之后优先恢复 `rf-thermo` / `rf-flash` 数值主线，再进入 `rf-solver` 的无回路顺序模块法与首个可求解 flowsheet 示例

进一步补充：

- 截至 2026-03-31，`rf-thermo` / `rf-flash` 的最小二元数值主线、黄金样例、`rf-unitops` / `rf-flowsheet` 的第一轮边界，以及 `rf-solver` 的首个无回路闭环都已提前推进
- 当前近期主线已从“补第一条求解闭环”切换为“扩第二个内建单元、增加第二个示例 flowsheet，并细化求解结果与诊断口径”

截至 2026-04-01，再补充对齐：

- `rf-ui` 已能把内核 `SolveSnapshot` 映射并回写到 `AppState`
- `apps/radishflow-studio` 已具备从物性包加载、工作区运行命令、运行门控到结果回写的最小应用层闭环
- 当前近期 Studio 主线已从“只有求解 bridge”切换为“继续把运行命令、结果派发和后续异步执行边界收口”

### 2026-W16

- 已提前完成 `rf-thermo` 中的 Antoine 饱和蒸气压与理想体系 `K` 值估算
- 已提前完成 `rf-flash` 中的 Rachford-Rice 和最小二元 `TP Flash`
- 已提前建立 `tests/thermo-golden` 与 `tests/flash-golden` 的首批黄金样例

### 2026-W17

- 已提前完成 `rf-unitops` 中 `Feed`、`Mixer`、`Flash Drum` 的最小统一接口
- 已提前完成 `rf-flowsheet` 中的端口连接与基本校验
- 已提前明确单元输入输出的标准流股接口与 canonical material ports

### 2026-W18

- 已提前完成 `rf-solver` 中首轮无回路顺序模块法
- 已提前增加第一个可直接从 `*.rfproj.json` 载入并求解的示例 flowsheet
- 完成 `DWSIM / COFE` water/ethanol 人工 PME 验证记录、PME trace 开关化、TypeLib 生成脚本化与 CAPE-OPEN / PME 阶段性冻结
- 回到 Rust Studio 主线，补出“打开示例项目 -> 运行求解 -> 查看结果/诊断”的第一版真实 `egui` 可见闭环，并补中文 shell 选项与 CJK 字体 fallback；2026-04-28 继续补出路径输入式项目打开入口、Windows 原生打开选择器、打开反馈、未保存改动打开前确认、shell 级最近项目列表及其独立 preferences 持久化、结构化流股结果 presentation 与结果区基础本地化；2026-04-29 继续补出 Result Inspector、失败结果、诊断目标命令、活动 Inspector 详情、Stream Inspector 字段级草稿更新/提交和基础文档历史 undo/redo；2026-04-30 继续补出保存 / 另存为生命周期、Stream Inspector 多字段批量提交、字段编辑快捷键焦点策略、项目文件 staged write 与 Save As 覆盖确认；2026-05-01 继续补出保存 / 另存失败恢复、Result Inspector 摘要可读性、产出单元诊断关联、当前快照内两股流股对比、Stream Inspector 总体组成字段编辑边界，以及 Canvas pending edit、单类型放置、单元块、物流线、选择反馈、对象列表、焦点气泡、端口 marker、端口 hover、运行/诊断 badge 和对象列表临时筛选，并通过 `pwsh ./scripts/check-repo.ps1` 完成仓库级验证

### 2026-W19 以后

- 设计 `rf-ffi` 的句柄式 C ABI
- 衔接 `.NET 10` 适配层
- 再开始 PME 侧人工验证

## 当前阶段的判断标准

当前不是“做得多”就对，而是满足以下判断标准才算推进正确：

- 边界清晰
- 工作区始终可 `cargo check`
- 文档、代码和阶段目标互相一致
- 不把 `M4/M5` 的复杂度提前压进 `M2/M3`

补充对 `.NET 10` `UnitOp.Mvp` 当前子线的判断口径：

- 若新增的是库内正式只读宿主语义，且能被 contract tests 与 smoke host 共同消费，方向通常正确
- 若新增的是只为某个 smoke 场景或临时宿主脚本服务的 helper / accessor / 字面量折叠，应优先回收到正式 reader / snapshot / catalog 再继续推进
- 若开始需要依赖 COM 注册、PME 自动化互调、第三方模型加载或额外系统环境副作用才能证明价值，则大概率已经越过当前阶段边界
