# MVP Roadmap Milestones

更新时间：2026-05-14

## 用途

用途：保留第一阶段 MVP 的 M1-M5 里程碑、任务和退出标准。
读者：需要查看原始里程碑拆解和阶段边界的开发者。
不包含：当前状态摘要、每日进度流水和后续 UI 设计规范。

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

### 当前补充收口

- 2026-05-05：M2 的“基础焓值”已按 MVP 边界补齐为常热容显热基线。`rf-thermo` 使用 property package 中的 liquid/vapor heat capacity 计算相对 `298.15 K` 的相 molar enthalpy；`rf-flash` 在 `TP Flash` 结果中写入 liquid/vapor 与按相分率加权的 overall molar enthalpy，并由 `Flash Drum` outlet stream 保留该相焓。该实现只作为当前二元理想体系结果消费面的基础数值，不代表完整焓参考态、相变潜热或真实物性模型已完成。
- 2026-05-07：M2 当前又已在现有 Antoine / Raoult MVP 假设上补出 bubble/dew pressure 与 fixed-pressure bubble/dew temperature 边界；`rf-flash` 的 `TP Flash` 结果会显式携带 `phase region + bubble/dew pressure / temperature`，并把结构化 `bubble_dew_window` 前推到 `rf-model -> rf-solver -> rf-ui::SolveSnapshot`。
- 2026-05-07：Studio bootstrap 内置的 `binary-hydrocarbon-lite-v1` 样例包 Antoine 系数当前也已与上述边界基线对齐，避免空白项目 / Studio run path 与 thermo golden / integration 样例分叉成两套数值假设。
- 2026-05-08：M2 当前又已把 bubble/dew near-boundary `±ΔP / ±ΔT` 小扰动回归扩成正式基线：`tests/thermo-golden`、`tests/flash-golden` 与 focused tests 现在同时覆盖 `binary-hydrocarbon-lite-v1` three-composition two-phase 样例，以及 synthetic `liquid-only / vapor-only` 单相样例，继续锁定 boundary 附近的 `phase_region` 与 `bubble_dew_window` 稳定行为。

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
- `tests/rust-integration` 当前又已把 near-boundary flash inlet 一致性回归前推到 `Heater/Cooler/Valve/Mixer -> Flash` 正式链路：`binary-hydrocarbon-lite-v1` three-composition two-phase 已覆盖 `feed-heater-flash-binary-hydrocarbon` / `feed-cooler-flash-binary-hydrocarbon` / `feed-valve-flash-binary-hydrocarbon` / `feed-mixer-flash-binary-hydrocarbon`，synthetic `liquid-only / vapor-only` 单相已覆盖 `feed-heater-flash` / `feed-cooler-flash` / `feed-valve-flash` / `feed-mixer-flash`；`studio_solver_bridge` 与 `studio_workspace_control` 会继续锁定非 flash 中间流股 `phase_region`、完整 `bubble_dew_window` 与 flash inlet consumed stream 的同一份 DTO
- `rf-ui` 当前已具备把 `rf-solver::SolveSnapshot` 回写为 UI 层结果快照的稳定映射
- Studio 当前又已让 Result Inspector / Active Inspector 只读消费 `SolveSnapshot` 已物化的 `bubble_dew_window`，显式展示 `phase region` 与 bubble/dew pressure / temperature；shell 继续只消费 DTO，不在界面层重算热力学
- Studio 当前又已把这层结果展示边界前推到 `studio_gui_shell::tests::runtime`：runtime 最终渲染只会在 `bubble_dew_window` 已物化时显示 `Bubble/dew window` 区块，`liquid-only / vapor-only` case 的零流量对侧 outlet 保持窗口缺席；同一股流股同时出现在 `Result Inspector` 与 `Active Inspector` 时，shell 通过独立 widget id scope 保持重复渲染稳定，而不是在界面层分叉结果语义
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
- Studio 当前窗口布局和 Canvas placement 坐标已独立持久化到 `<project>.rfstudio-layout.json` sidecar；窗口布局已从基于运行时 `window_id` 的 key 收口到基于 `window_role + layout_slot` 的稳定 key，Canvas 当前只保存已提交单元的 placement 落点
- Studio 当前已把最小真实桌面闭环接到 `egui` Runtime 面板：用户可在内置正向示例项目之间切换，触发既有运行栏动作，并查看 `SolveSnapshot` 映射出的结构化流股结果、求解步骤、诊断、日志与工作区摘要；流股结果当前已按 summary / overall composition / phases 三类 presentation 展示，并已补出当前快照内两股流股的 `T / P / F` 与总体组成差值对比，为后续更完整结果审阅或导出继续留边界。
- Studio 当前又已补出 Runtime 面板内的项目打开入口：用户可输入现有 `*.rfproj.json` 路径，或通过 Windows 原生文件选择器选取项目并重建当前 Studio runtime；打开失败时保留当前工作区并显示错误反馈，内置示例切换复用同一条打开流程；若当前文档存在未保存修订，打开动作会先进入显式确认状态；打开成功后会写入 shell 级最近项目列表，并把该列表保存到独立 Studio preferences 文件；重启后会恢复最近项目，点击最近项目继续复用同一条打开流程和未保存确认保护。
- Studio 当前中文/英文切换先作为 GUI shell 级偏好存在，默认显示中文壳层文案；中文字体通过系统 CJK 字体 fallback 解决，不把字体资产或语言偏好写入 `*.rfproj.json`。
- Studio 当前又已把结果区推进到最小 Result Inspector 和失败结果 presentation，并把诊断目标、活动 Inspector 详情、Stream Inspector 字段 presentation、字段级 draft update / 单字段 commit / 多字段批量 commit command，以及基础 `edit.undo / edit.redo` 文档历史命令接入正式 driver / runtime 边界；Stream Inspector 字段当前覆盖基础状态字段与已有总体组成组分条目，并已提供从 flowsheet 已定义组件中显式添加缺失组成条目、删除非最后组成条目的受控 command surface；Result Inspector 当前还补出中英文可读的 `T / P / F` 摘要标签、产出单元诊断关联和当前快照内两股流股对比；活动 Inspector 当前也会在选中已运行单元时展示最新 `SolveSnapshot` 的执行状态、step 序号、summary、输入流股和产出流股跳转；求解步骤 presentation 当前也已携带单元、输入流股与产出流股 command action，供 Runtime、Active Inspector 与 Result Inspector 统一导航；失败结果 presentation 当前也携带 recovery action 与 recovery target action，分别复用运行栏修复命令和 Inspector target 命令；当前又新增统一 `StudioGuiWindowDiagnosticTargetActionModel`，把失败摘要、Result Inspector、Active Inspector 与求解步骤里的错误定位/导航动作汇总成同一类 presentation；失败结果当前也会结构化显示 latest diagnostic summary 的 revision、severity、primary code、count 与相关 unit / stream / port target；Active Inspector 的 unit port 列表当前也会在诊断修订号仍匹配当前文档时显示对应 port 的只读 attention 摘要。
- 2026-05-05 进一步把 Result Inspector 收口到当前 `SolveSnapshot` 内的结构化审阅：stream comparison 当前会比较 summary / composition / phase rows，并显示已物化 molar enthalpy；stream / comparison / unit selector summary 会显示已物化焓值或最新 step 的输入/产出流股上下文；comparison 正文也为 base stream 与 compared stream 暴露既有 Inspector focus action。上述入口仍只复用 `inspector.focus_stream:*` / `inspector.focus_unit:*` command，不新增结果表格、导出、报表或跨快照历史。
- Studio 当前又已补出 `Save / Save As` 文档生命周期、字段编辑快捷键焦点策略、项目文件 staged write、`Save As` 覆盖确认和保存 / 另存失败恢复：保存只写回项目真相源并刷新保存态，不进入文档历史；`Ctrl+S` 在文本输入焦点下仍保存，`Ctrl+Z / Ctrl+Y` 在文本输入焦点下保留给输入框；覆盖已有非当前项目文件前必须先停在 shell 确认态；写入失败不会污染当前项目路径、保存修订号、文档历史或最近项目。
- Studio Canvas 当前又已补出最小可见、只读扫读层和多单元放置反馈闭环：`CanvasEditIntent` 仍只承载 `Feed / Mixer / Heater / Cooler / Valve / Flash Drum` pending edit / commit 最小路径；已有 unit / material stream / material port / Inspector focus / run diagnostics 会投影为单元块、物流线、端口 marker、focus callout、viewport focus anchor、对象列表、端口 hover、诊断 badge、状态 legend 与对象列表 `All / Attention / Units / Streams` 临时筛选；`CommitPendingEditAt -> DocumentCommand::CreateUnit` 成功、无 pending edit、unsupported unit kind、dispatch 失败和 anchor 过期都会经由统一 `StudioGuiCanvasCommandResultViewModel` 进入 notice / GUI activity / command-surface 只读反馈；2026-05-04 又在不引入自由连线编辑器的前提下补出本地 Canvas suggestions，已覆盖 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum` 与 `Feed + Feed -> Mixer -> Flash Drum` 三类最短可求解建模路径；同日继续补齐空白项目前置基线，无组件项目打开时会初始化 MVP 默认二元组件和本地 `binary-hydrocarbon-lite-v1` 物性包缓存，保存后重新打开仍可运行 `Feed -> Flash Drum` 最短闭环；随后又补出逐条 suggestion Apply，使用户可显式接受指定本地连接或 outlet stream 创建建议，而不依赖当前 focused suggestion；同日还补出 Canvas placement 坐标最小持久化，保存并重开后单元块位置可恢复，且仍可继续 Apply suggestions 与运行；随后又把离散 layout nudge 前推为 `canvas.move_selected_unit.left/right/up/down` widget action / command surface，缺少 sidecar 坐标时先按 transient grid slot pin 出初始位置，只写 Studio layout sidecar，且不污染项目文档 revision/history；Canvas attention 当前会从结构化 unit / stream / port diagnostic target 生成只读摘要，显示在 unit / stream / material port hover 和 object list attention summary 中，不从错误 message 反解析，也不新增端口级命令。
- 这一路径仍限定在“打开已有项目或内置示例 -> 最近项目入口 -> 运行求解 -> 查看结构化结果/诊断 -> Result Inspector 流股视图与 unit-centric 视图并列消费当前快照 -> 当前快照内流股 summary / composition / phase 对比与 Inspector 定位 -> 定位 Inspector -> 审阅单元最新执行结果 -> 从求解步骤跳转到单元、输入流股和产出流股 -> 从失败摘要触发修复或定位目标 -> 在 Canvas 与 Active Inspector 中查看只读 port attention 摘要 -> 编辑 Stream 基础状态与总体组成字段草稿并提交 -> 基础撤销/重做 -> 保存/另存为 -> 快捷键与覆盖确认/失败恢复保护 -> `egui` 画布多单元 pending placement 入口、只读对象扫读/定位反馈、本地建议补齐连接/出口流股、placement 坐标 sidecar 重载与离散 layout nudge”的最小可操作工作台闭环；当前原生文件选择器只覆盖 Windows 打开与另存为，Canvas 仍不做端口点击编辑、自由连线创建、拖拽布局编辑器或视口持久化，也不代表已经进入完整画布编辑器、跨平台文件工作流、结果表格导出、结果报表系统、完整应用偏好系统、跨会话历史持久化或 UI 视觉精修阶段。

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
