# App Architecture

更新时间：2026-05-16

## 当前目标

现阶段 App 方向已经从“先把桌面应用架构边界和未来扩展面定清楚”推进到 MVP α 验收硬化与 Studio 可用性收口。当前仍不做完整商业化界面或复杂交互扩张，但需要把已进入验收路径的 Studio 首页、工作台分区、运行入口、消息反馈和关闭行为规范化，避免用户无法判断如何打开示例、如何运行、结果在哪里、错误在哪里以及窗口是否正常退出。

这意味着当前阶段关注的是：

- App 壳层职责
- UI 与 Core 的边界
- 文档模型与状态组织方式
- 画布、属性面板、运行控制、结果展示之间的关系

而不是立即扩展自由连线编辑器、完整拖拽布局编辑器、完整报表系统或未来型多文档工作台。

## 当前已冻结决策

截至 2026-03-29，以下 App 架构决策已经冻结：

1. MVP 保持单文档工作区，不做多文档容器优先设计
2. “单文档工作区”不等于“单文件实现”，代码仍按职责拆分，避免单文件持续膨胀
3. 属性编辑采用字段级草稿态，只有在语义提交时才写回文档
4. 求解控制采用 `SimulationMode` 与 `RunStatus` 分离建模
5. 求解结果采用独立 `SolveSnapshot`，不直接污染 `FlowsheetDocument`
6. 结果快照保留按步展开能力，为后续结果审阅、差异比较和操作脚本留接口
7. 撤销/重做当前采用 snapshot-backed `CommandHistory`：历史记录仍保留语义 `DocumentCommand`，执行时应用对应 `before / after` flowsheet 快照

## 顶层分层

桌面应用建议固定为三层协作：

1. `apps/radishflow-studio`
2. `crates/rf-ui`
3. `crates/rf-canvas`

并通过 `rf-model`、`rf-solver`、`rf-store` 等核心 crate 提供数据与服务。

## 各层职责

### `apps/radishflow-studio`

这是应用组合根，而不是业务逻辑仓库。

职责：

- 应用启动
- 窗口初始化
- 顶层菜单与工具栏装配
- 工作区与文档生命周期管理
- 将 `rf-ui`、`rf-canvas`、`rf-store`、`rf-solver` 能力组装成桌面应用
- 负责 `AuthSessionState` / `EntitlementState` 与 `StoredAuthCacheIndex` 之间的桥接与同步
- 负责控制面 `entitlement` / `manifest` / `lease` / `offline refresh` 的 HTTP client、协议映射与应用层编排
- 负责把下载租约、下载 fetcher 与本地缓存落盘串成单一路径
- 负责从 `PropertyPackageProvider` 或本地 auth cache 组装最小真实求解链路，并把 `rf-solver::SolveSnapshot` 回写到 `rf-ui::AppState`
- 负责把当前 Studio shell 的用户入口组织为可复现的 MVP α 工作流：启动后默认显示 Home Dashboard，进入 case 后再暴露 `Home / 打开示例 / 新建空白 / 打开项目... / 运行 / 保存 / 另存为... / 视图`，默认隐藏低频命令大全和调试式布局控制，保留命令面板作为高级入口
- 负责在 GUI shell 层提供用户操作与求解审计输出；默认 stderr 日志只作为开发态 smoke 和诊断入口，不替代未来正式审计 / telemetry 设计
- 负责遵守 `eframe` / `winit` 桌面事件循环约束：Windows 事件循环在主线程创建，最后一个 viewport close 请求不得被 `CancelClose` 拦截，关闭前只清理逻辑窗口状态并停止当帧 fallback 布局渲染

不应承担：

- 热力学计算
- 单元求解逻辑
- 画布图元绘制细节
- 项目文件读写细节
- 把视觉主题、按钮样式、面板密度或日志展示直接写成求解 / 文档语义

### Studio Shell UI 规范化边界

2026-05-16 人工 smoke 已确认，当前 Studio 默认首页和进入 case 后的工作台第一轮分区已经落地。shell UI 边界按以下稳定入口治理：

- Home Dashboard：应用启动后的默认首页，只承载 Start actions、Recent Cases、Example Cases、Environment 和 Messages；不读取或解释 `SolveSnapshot`，不直接承载流程图编辑。
- 顶部主路径：进入 case 后只保留用户主路径、当前项目摘要和必要状态，不把调试计数、菜单全集和布局控制置于默认第一视野。
- 操作入口：`Home`、打开示例、新建空白、打开项目、运行、保存、另存为和视图入口保持可发现；低频命令进入命令面板或 `视图` 菜单。
- 工作台分区：左侧 `项目 / 示例项目 / 放置` 负责项目对象、示例入口与 MVP 放置入口；右侧 `检查器 / 结果 / 运行 / 物性包` 负责属性编辑、结果审阅、运行上下文和物性包状态；底部 `消息 / 运行日志 / 结果表 / 诊断` 承接可行动消息、运行日志和表格结果。
- 结果反馈：右侧结果、底部结果表、Active Inspector 继续只读消费 `SolveSnapshot`，UI 优化不得新增第二套结果缓存或求解解释。
- 日志与审计：开发态 stderr 与 GUI activity 可继续服务 smoke，但正式 UI 只展示用户能采取行动的摘要，不把平台 timer 或 host internals 混入主路径
- 关闭行为：最后窗口关闭应自然结束进程；为避免关闭瞬间闪 fallback 布局，shell 可在逻辑窗口清理后停止当帧渲染，但不能拦截原生关闭请求

### `rf-ui`

这是 App 的应用层和交互层。

职责：

- 当前文档状态
- 选择集状态
- 命令分发
- 面板状态
- 属性编辑事务
- 运行请求与结果视图状态

建议未来逐步形成的子模块：

- `app_shell`
- `document_workspace`
- `selection`
- `commands`
- `inspector`
- `run_panel`
- `result_panel`
- `log_panel`

截至 2026-04-02，`rf-ui` 已先落地最小 `run_panel::RunPanelState`，用于承接运行栏摘要；当前又进一步补出 `RunPanelCommandModel`、`RunPanelViewModel`、`RunPanelPresentation` 与 `RunPanelWidgetModel`，把“主动作是谁、按钮是否显示、按钮是否可点，以及最小运行栏该如何渲染和激活这些动作”冻结到 UI 自有模型里。更细的事件流编排继续留在 Studio 层的 `workspace_control` / `run_panel_driver` 收口。

### `rf-canvas`

这是纯画布能力层，不应承载流程求解或业务决策。

职责：

- 节点绘制
- 端口绘制
- 连线绘制
- 视口缩放、平移、框选
- 命中测试和拖拽反馈

不应承担：

- 物性计算
- 流股状态解释
- 单元求解调度
- 项目持久化

## 文档模型

桌面应用当前建议采用“单文档工作区优先”的模型，而不是一开始就做多文档复杂容器。

当前推荐结构：

- `AppState`
- `AuthSessionState`
- `EntitlementState`
- `WorkspaceState`
- `FlowsheetDocument`
- `UiPanelsState`
- `SelectionState`
- `CommandHistory`
- `SolveSessionState`
- `SolveSnapshot`

这样做的原因：

- MVP 阶段只有一个主流程图文档就足够
- 更容易把编辑态和求解态分开
- 后续要扩展多文档，也可以在外层再包一层文档容器

这里需要特别说明：

- 单文档工作区是产品形态决策
- 单个源码文件原则上不超过 1000 行，这是默认工程实现约束，不是可长期忽略的软建议
- 文件一旦接近或超过 1000 行，后续新增实现应优先拆分职责、提取子模块或测试 helper，而不是继续把新状态和新流程堆进原文件
- `src/` 下源码应按职责做浅层目录分组，优先使用 1 层子目录收纳同域模块，避免长期把所有模块平铺在 `src/` 根下，也避免为了“整齐”堆出过深目录树

两者不能混为一谈。  
即使当前保持单文档模式，源码层面也应拆分 `AppState`、文档状态、命令系统、求解状态和面板状态，而不是堆进一个大文件。

## 推荐状态结构

当前建议先冻结为“一个应用壳 + 一个工作区 + 一个文档真相源 + 一组运行态/交互态子对象”的结构。

对象所有权先明确如下：

- `AppState` 只拥有一个 `WorkspaceState`，同时持有应用级登录态与授权态，不提前做多工作区或多标签容器
- `WorkspaceState` 拥有当前文档、交互态、撤回历史、求解会话和快照历史
- `FlowsheetDocument` 是用户语义提交后的真相源，也是后续 `rf-store` 的主要持久化对象
- `CommandHistory` 只记录语义化文档命令，不吞入运行控制和纯 UI 临时行为
- `SolveSessionState` 只描述“当前文档修订号的运行意图与运行状态”，不拥有快照实体
- `SolveSnapshot` 是不可变结果记录，由 `WorkspaceState` 的快照历史持有

### `AppState`

建议最小字段：

- `workspace: WorkspaceState`
- `auth_session: AuthSessionState`
- `entitlement: EntitlementState`
- `preferences: UserPreferences`
- `log_feed: AppLogFeed`

冻结边界：

- `workspace` 是当前唯一活跃工作区；MVP 不做 `Vec<WorkspaceState>` 或标签页容器
- `auth_session` 只保存桌面端登录态、当前用户摘要和安全凭据引用，不保存明文 token
- `entitlement` 只保存授权快照和派生包清单，不反向充当文件缓存目录真相源
- `preferences` 只保存应用级偏好，不保存当前文档内容、求解结果或草稿值
- `log_feed` 只作为应用级事件与诊断入口，不反向充当文档或结果真相源

### `AuthSessionState`

建议最小字段：

- `status: AuthSessionStatus`
- `authority_url: Option<String>`
- `current_user: Option<AuthenticatedUser>`
- `token_lease: Option<TokenLease>`
- `last_authenticated_at: Option<DateTimeUtc>`
- `last_error: Option<String>`

冻结边界：

- `AuthSessionState` 放在 `AppState` 外层，而不是放进 `WorkspaceState` 或 `FlowsheetDocument`
- `token_lease` 只保存到操作系统安全存储的引用和到期时间，不保存明文 access token / refresh token
- `current_user` 只承载当前用户摘要，不承载完整授权快照或物性包清单

### `EntitlementState`

建议最小字段：

- `status: EntitlementStatus`
- `snapshot: Option<EntitlementSnapshot>`
- `package_manifests: BTreeMap<String, PropertyPackageManifest>`
- `last_synced_at: Option<DateTimeUtc>`
- `last_error: Option<String>`

冻结边界：

- `EntitlementState` 放在 `AppState` 外层，因为它与整个桌面应用授权相关，而不是与单文档绑定
- `snapshot` 只描述授权边界，不承载实际物性包内容
- `package_manifests` 只描述“哪些包可见、来自哪里、何时过期”，不直接替代 `rf-store` 的本地缓存索引

### `WorkspaceState`

建议最小字段：

- `document: FlowsheetDocument`
- `document_path: Option<PathBuf>`
- `last_saved_revision: Option<u64>`
- `selection: SelectionState`
- `panels: UiPanelsState`
- `drafts: InspectorDraftState`
- `command_history: CommandHistory`
- `solve_session: SolveSessionState`
- `snapshot_history: VecDeque<SolveSnapshot>`
- `run_panel: RunPanelState`

冻结边界：

- `document` 是当前唯一打开文档
- `document_path` 和 `last_saved_revision` 属于工作区运行态，不写入 `FlowsheetDocument`
- `selection`、`panels`、`drafts` 都是瞬时 UI 状态，不能污染文档真相源
- `command_history`、`solve_session`、`snapshot_history` 并列存在，互不吞并
- `snapshot_history` 负责持有不可变快照实体，`SolveSessionState` 只保留引用
- `run_panel` 只持有面向运行栏的已派生摘要，不反向取代 `solve_session`、`snapshot_history` 或 `log_feed`
- `run_panel` 当前也负责持有最小按钮/命令模型，不让按钮启用判断散落到 Studio 或最终视图层
- 运行栏最终最小视图入口当前应消费 `RunPanelViewModel`，而不是重新拼装 `can_run_manual` / `can_resume` 之类摘要布尔值

### `FlowsheetDocument`

建议最小字段：

- `revision: u64`
- `flowsheet: Flowsheet`
- `metadata: DocumentMetadata`

冻结边界：

- `revision` 先正式冻结为单调递增 `u64`，每次语义提交成功后递增
- 保存、另存为、切换面板、框选、缩放、草稿字符变化都不递增 `revision`
- `flowsheet` 只承载流程图对象模型、参数、连接和用户显式设定值
- `metadata` 只承载文档元信息，不承载文件路径、选择集、求解态或用户偏好

### `CommandHistory`

建议最小字段：

- `entries: Vec<CommandHistoryEntry>`
- `cursor: usize`

`CommandHistoryEntry` 当前建议至少携带：

- `revision: u64`
- `command: DocumentCommand`
- `before: Option<Flowsheet>`
- `after: Option<Flowsheet>`

冻结边界：

- `entries` 只保存成功写回 `FlowsheetDocument` 的语义命令
- `command` 保留“用户做了什么”的语义描述，`before / after` 负责执行 undo/redo 时的文档状态回放
- `cursor` 指向“下一条可重做命令”的位置，用于 undo/redo
- 当用户在 undo 后提交新命令时，`cursor` 之后的 redo 尾部必须被截断
- `OpenDocument`、`SaveDocument`、`SetSimulationMode`、框选和缩放都不进入该历史栈

### `SolveSessionState`

建议最小字段：

- `mode: SimulationMode`
- `status: RunStatus`
- `observed_revision: u64`
- `pending_reason: Option<SolvePendingReason>`
- `latest_snapshot: Option<SolveSnapshotId>`
- `latest_diagnostic: Option<DiagnosticSummary>`

冻结边界：

- `mode` 表示当前运行策略，只允许 `Active` / `Hold`
- `status` 表示针对 `observed_revision` 的最近检查或求解状态
- `observed_revision` 明确绑定当前会话状态所描述的文档修订号，避免状态和文档脱节
- `pending_reason` 只解释“为什么当前修订号还需要下一次检查/求解”，不承载失败详情
- `latest_snapshot` 只保存快照引用，不直接内嵌完整 `SolveSnapshot`
- `latest_diagnostic` 只保存摘要；完整诊断明细进入 `SolveSnapshot`

### `SolveSnapshot`

建议最小字段：

- `id: SolveSnapshotId`
- `document_revision: u64`
- `sequence: u64`
- `status: RunStatus`
- `summary: DiagnosticSummary`
- `diagnostics: Vec<DiagnosticSnapshot>`
- `steps: Vec<StepSnapshot>`

冻结边界：

- `SolveSnapshot` 一旦生成即不可变
- `document_revision` 明确绑定该快照对应的文档修订号
- `sequence` 用于区分同一 `document_revision` 上的多次运行
- `summary` 是结果入口摘要，完整诊断和按步执行数据分开放在 `diagnostics` / `steps`
- 快照实体由 `WorkspaceState.snapshot_history` 持有，并按有界窗口保留
- 当前实现允许先由内核 `rf-solver::SolveSnapshot` 生成最小求解结果，再在 `rf-ui` 中映射为 UI 层 `SolveSnapshot`，避免 UI 层直接依赖内核内部执行细节结构

## 最小状态草案

在继续深化 `rf-ui` 之前，当前建议把最小状态对象先冻结到以下轮廓：

```rust
pub struct AppState {
    pub workspace: WorkspaceState,
    pub auth_session: AuthSessionState,
    pub entitlement: EntitlementState,
    pub preferences: UserPreferences,
    pub log_feed: AppLogFeed,
}

pub struct WorkspaceState {
    pub document: FlowsheetDocument,
    pub document_path: Option<PathBuf>,
    pub last_saved_revision: Option<u64>,
    pub selection: SelectionState,
    pub panels: UiPanelsState,
    pub drafts: InspectorDraftState,
    pub command_history: CommandHistory,
    pub solve_session: SolveSessionState,
    pub snapshot_history: VecDeque<SolveSnapshot>,
    pub run_panel: RunPanelState,
}

pub struct FlowsheetDocument {
    pub revision: u64,
    pub flowsheet: Flowsheet,
    pub metadata: DocumentMetadata,
}

pub struct CommandHistory {
    pub entries: Vec<CommandHistoryEntry>,
    pub cursor: usize,
}

pub struct SolveSessionState {
    pub mode: SimulationMode,
    pub status: RunStatus,
    pub observed_revision: u64,
    pub pending_reason: Option<SolvePendingReason>,
    pub latest_snapshot: Option<SolveSnapshotId>,
    pub latest_diagnostic: Option<DiagnosticSummary>,
}

pub struct SolveSnapshot {
    pub id: SolveSnapshotId,
    pub document_revision: u64,
    pub sequence: u64,
    pub status: RunStatus,
    pub summary: DiagnosticSummary,
    pub diagnostics: Vec<DiagnosticSnapshot>,
    pub steps: Vec<StepSnapshot>,
}

pub struct DocumentMetadata {
    pub document_id: DocumentId,
    pub title: String,
    pub schema_version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct UserPreferences {
    pub theme: AppTheme,
    pub locale: LocaleCode,
    pub recent_project_paths: Vec<PathBuf>,
    pub panel_defaults: PanelLayoutPreferences,
    pub snapshot_history_limit: usize,
}

pub struct AuthSessionState {
    pub status: AuthSessionStatus,
    pub authority_url: Option<String>,
    pub current_user: Option<AuthenticatedUser>,
    pub token_lease: Option<TokenLease>,
    pub last_authenticated_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

pub struct EntitlementState {
    pub status: EntitlementStatus,
    pub snapshot: Option<EntitlementSnapshot>,
    pub package_manifests: BTreeMap<String, PropertyPackageManifest>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

pub struct DiagnosticSummary {
    pub document_revision: u64,
    pub highest_severity: DiagnosticSeverity,
    pub primary_message: String,
    pub diagnostic_count: usize,
    pub related_unit_ids: Vec<UnitId>,
}

pub enum SolvePendingReason {
    DocumentRevisionAdvanced,
    ModeActivated,
    ManualRunRequested,
    SnapshotMissing,
}
```

这里有几条已经冻结的实现口径：

- `WorkspaceState` 当前只持有一个 `FlowsheetDocument`
- `AuthSessionState` 和 `EntitlementState` 当前挂在 `AppState` 外层，而不是混入工作区文档态
- `document_path` / `last_saved_revision` 明确留在 `WorkspaceState`，不混进 `DocumentMetadata`
- `FlowsheetDocument` 只表示用户编辑态，不持有求解器内部态或 UI 瞬时态
- `SolveSessionState` 只引用快照，不直接内嵌完整结果对象
- `WorkspaceState.snapshot_history` 明确承担快照所有权
- `CommandHistory` 与 `SolveSessionState` 并列，而不是互相吞并
- `RunPanelState` 当前作为 `WorkspaceState` 的派生 UI 状态对象存在，不额外引入 `rf-ui -> studio` 反向依赖

## 字段级冻结口径

### `DocumentMetadata`

建议最小字段：

- `document_id`
- `title`
- `schema_version`
- `created_at`
- `updated_at`

冻结边界：

- `DocumentMetadata` 只描述文档身份、标题和序列化兼容信息
- `document_path`、最近打开时间、面板布局、运行模式都不属于 `DocumentMetadata`
- `updated_at` 只在语义化文档提交成功后更新，不因保存、求解或切换选择集而变化

### `UserPreferences`

建议最小字段：

- `theme`
- `locale`
- `recent_project_paths`
- `panel_defaults`
- `snapshot_history_limit`

冻结边界：

- 这里保存的是“用户怎么用 App”，不是“文档当前是什么状态”
- `snapshot_history_limit` 只影响工作区内存中的快照保留窗口，不改变文档语义
- `recent_project_paths` 属于应用级 MRU 列表，不参与项目文件序列化；当前 Studio shell 已用独立 `preferences.rfstudio-preferences.json` 持久化这一字段，但尚未把语言、主题等其他偏好并入完整偏好系统

### `AuthSessionState`

建议最小字段：

- `status`
- `authority_url`
- `current_user`
- `token_lease`
- `last_authenticated_at`
- `last_error`

冻结边界：

- `AuthSessionState` 是应用运行态，不进入项目文件
- 明文 token 不属于该对象；这里只允许保存安全凭据引用和到期时间
- 登录错误和授权错误可以在这里显示摘要，但不替代审计日志

### `EntitlementState`

建议最小字段：

- `status`
- `snapshot`
- `package_manifests`
- `last_synced_at`
- `last_error`

冻结边界：

- `EntitlementState` 是授权控制态，不进入 `FlowsheetDocument`
- `package_manifests` 只描述远端清单，不直接代替本地缓存索引
- 授权过期、离线租约过期和清单同步失败都通过这里驱动 UI 提示

### `DiagnosticSummary`

建议最小字段：

- `document_revision`
- `highest_severity`
- `primary_message`
- `diagnostic_count`
- `related_unit_ids`

冻结边界：

- `DiagnosticSummary` 是轻量摘要对象，用来驱动状态栏、结果摘要栏和运行栏提示
- 失败详情、逐条错误列表、单步执行结果不挤进摘要对象
- `document_revision` 必须与其描述的检查/求解对象保持一致

### `SolvePendingReason`

当前先冻结为以下最小语义集合：

- `DocumentRevisionAdvanced`: 文档发生新的语义提交，现有检查/求解结论失效
- `ModeActivated`: 用户从 `Hold` 切回 `Active`，系统待进入下一轮检查/求解
- `ManualRunRequested`: 用户显式触发一次检查或求解
- `SnapshotMissing`: 当前修订号还没有任何可引用快照

冻结边界：

- `SolvePendingReason` 只解释“为什么还有待办运行”，不重复表达 `Error` / `Unconverged`
- 真正的失败归因放在 `RunStatus` 与 `DiagnosticSummary` / `DiagnosticSnapshot`
- `pending_reason` 在生成与 `observed_revision` 对齐的新结果后应清空

## 命令对象草案

命令系统建议从一开始就区分“文档命令”和“UI 临时行为”。

当前建议进入历史栈的命令最小集合：

- `CreateUnit`
- `DeleteUnit`
- `MoveUnit`
- `ConnectPorts`
- `DisconnectPorts`
- `RenameUnit`
- `SetUnitParameter`
- `SetStreamSpecification`

当前不建议进入历史栈的行为：

- 框选
- 缩放
- 画布平移
- 面板展开/收起
- 临时输入中的草稿字符变化

补充冻结口径：

- `MoveUnit` 进入历史栈，因为节点几何位置属于文档语义的一部分
- 当前 `ConnectPorts` 必须显式携带 `stream_id`，并允许 `to_unit_id / to_port` 为空，以覆盖“复用已有 stream 接到 sink”与“创建 terminal outlet stream”两类正式 material connection 写回
- `SetSimulationMode`、`RunSolve`、`ClearResults` 属于运行控制动作，直接作用于 `SolveSessionState`
- `OpenDocument`、`SaveDocument`、`SaveDocumentAs` 属于文档生命周期动作，不进入 undo/redo 历史

这条边界如果现在不冻结，后面 undo/redo 会很快被无价值噪声淹没。

## 草稿态结构建议

字段级草稿态建议不要散落在控件内部，而是集中表达成可检查对象。

当前建议最小结构：

```rust
pub struct FieldDraft<T> {
    pub original: T,
    pub current: T,
    pub is_dirty: bool,
    pub validation: DraftValidationState,
}
```

这样做的好处：

- 输入校验可以发生在提交前
- 不同控件类型可以共享一套“草稿 -> 提交”语义
- 后续如果要做“批量应用本面板修改”，也还有扩展空间

## 求解快照草案

独立结果快照当前建议至少分为三层：

```rust
pub struct SolveSnapshot {
    pub id: SolveSnapshotId,
    pub document_revision: u64,
    pub sequence: u64,
    pub status: RunStatus,
    pub summary: DiagnosticSummary,
    pub diagnostics: Vec<DiagnosticSnapshot>,
    pub steps: Vec<StepSnapshot>,
}

pub struct StepSnapshot {
    pub index: usize,
    pub unit_id: UnitId,
    pub summary: String,
    pub execution: UnitExecutionSnapshot,
    pub consumed_streams: Vec<StreamStateSnapshot>,
    pub streams: Vec<StreamStateSnapshot>,
}
```

这里最重要的不是字段名字，而是结构关系：

- 快照关联文档修订号
- 快照通过 `sequence` 区分同一修订号上的多次运行
- `summary` 与 `SolveSessionState.latest_diagnostic` 共享同一摘要语义
- 步骤序列保持稳定顺序
- 步骤内部记录单元执行结果，以及结构化输入流股 `consumed_streams` 和输出流股 `streams`
- `StepSnapshot` 的输入/输出流股应由 solver step 直接物化，UI / workspace consumer 只读消费这份 DTO，不再按 stream id 回填或临时拼装第二套 step 结果
- 诊断信息与数值结果并列保存
- 快照实体由 `WorkspaceState.snapshot_history` 持有，并受 `UserPreferences.snapshot_history_limit` 约束

## 关键事件流

### 参数提交流

推荐事件流如下：

1. 用户编辑字段，形成草稿态
2. 用户触发语义提交
3. UI 生成命令并写回 `FlowsheetDocument`
4. `FlowsheetDocument.revision` 递增，`DocumentMetadata.updated_at` 更新
5. 命令写入 `CommandHistory`，若此前处于 undo 状态则截断 redo 尾部
6. `SolveSessionState.observed_revision` 更新为当前修订号，`pending_reason = DocumentRevisionAdvanced`
7. `RunStatus` 进入 `Dirty`
8. 若 `SimulationMode = Active`，立即进入检查与求解流程

### 撤销/重做流

当前 undo / redo 不把逆向逻辑散落到每一种文档命令里，而是沿 `CommandHistoryEntry.before / after` 回放文档快照。

推荐事件流如下：

1. 用户触发 `edit.undo` 或 `edit.redo`
2. UI 通过正式 command surface 派发命令，不直接修改文档
3. 应用层读取 `CommandHistory.cursor` 对应的 `before` 或 `after` 快照
4. `FlowsheetDocument` 替换为目标快照并递增新 `revision`
5. `CommandHistory.cursor` 前移或后移
6. 字段草稿清空，仍存在的 inspector target 继续保留，不存在的目标被清理
7. `SolveSessionState` 进入 `DocumentRevisionAdvanced / Dirty`

### 自动求解流

当模式为 `Active` 时：

1. 将 `SolveSessionState.observed_revision` 绑定到当前 `FlowsheetDocument.revision`
2. 进入结构校验和自由度检查，`RunStatus = Checking`
3. 若可解，状态进入 `Runnable`，随后进入 `Solving`
4. 生成新的 `SolveSnapshot` 并追加到 `snapshot_history`
5. 依据 `snapshot_history_limit` 裁剪最旧快照
6. 更新 `latest_snapshot`、`latest_diagnostic`，清空 `pending_reason`
7. 更新 `RunStatus`

### 失败转 Hold 流

当求解报错或不收敛时：

1. 记录 `DiagnosticSummary`，并尽可能生成失败快照
2. `latest_diagnostic` 更新到当前 `observed_revision`
3. `RunStatus` 置为 `Error` 或 `Unconverged`
4. `pending_reason` 清空
5. `SimulationMode` 自动切换到 `Hold`

### Hold 恢复流

当用户修正参数后：

1. 文档继续递增修订号
2. `SolveSessionState.observed_revision` 跟进到最新修订号
3. `RunStatus` 进入 `Dirty`
4. `pending_reason = DocumentRevisionAdvanced`
5. 系统不自动继续求解
6. 用户手动切换 `SimulationMode = Active`
7. `pending_reason = ModeActivated`
8. 再次进入检查与求解

## 属性编辑模型

属性编辑当前正式采用“字段级草稿态 + 语义提交”的模式。

具体规则：

- 用户输入过程中，UI 内部持有草稿值
- 草稿值集中放在 `WorkspaceState.drafts`，而不是散落在控件私有状态里
- 草稿值不立即写回 `FlowsheetDocument`
- 当发生 `Enter`、失焦、点击应用等语义提交时，才生成命令并写回文档
- 写回文档后再决定是否触发结构检查与自动求解

采用这个方案的原因：

- 避免用户输入 `100` 时触发 `1 -> 10 -> 100` 三次无意义求解
- 比整页“大草稿后统一应用”更接近工程软件的即时反馈体验
- 能天然接入命令历史，而不是让半成品输入污染撤回栈

当前约定：

- 草稿态不进入命令历史
- 只有成功提交到文档的变更才形成命令
- 只有影响方程系统的提交才触发求解相关检查

## 求解模式与运行状态

求解控制当前明确采用“模式”和“状态”分离建模。

建议的最小模型：

- `SimulationMode`
  - `Active`
  - `Hold`
- `RunStatus`
  - `Idle`
  - `Dirty`
  - `Checking`
  - `Runnable`
  - `Solving`
  - `Converged`
  - `UnderSpecified`
  - `OverSpecified`
  - `Unconverged`
  - `Error`

这两个对象的职责必须分开：

- `SimulationMode` 表示系统当前采用何种运行策略
- `RunStatus` 表示最近一次检查/求解后的真实状态

`SolvePendingReason` 的职责再补一条：

- `SolvePendingReason` 只解释当前修订号为什么还有待办求解，不取代 `RunStatus`

当前行为约定：

1. 当系统处于 `Active` 时，提交影响模型的参数后先做结构检查和自由度检查
2. 若检查通过，则自动进入求解
3. 若求解报错或不收敛，则 `RunStatus` 更新为 `Error` 或 `Unconverged`
4. 同时系统自动切换到 `Hold`
5. 用户修改参数后，`observed_revision` 跟进到最新文档修订号，状态进入 `Dirty`
6. 用户手动切回 `Active` 后，再继续检查和求解

这样做的目的，是保留 HYSYS 式交互体验，同时避免把失败状态和运行模式混在一起。

## 当前已落地的求解桥接

截至 2026-04-02，`apps/radishflow-studio` 已经实现最小应用层求解桥接，并通过单元测试与仓库级验证覆盖。

当前已落地入口：

- `StudioAppFacade::{execute_with_auth_cache, run_workspace_from_auth_cache}`
- `StudioAppFacade::{resume_workspace_from_auth_cache, set_workspace_simulation_mode}`
- `WorkspaceControlAction::{RunManual, Resume, SetMode}`
- `snapshot_workspace_control_state(...)`
- `dispatch_workspace_control_action_with_auth_cache(...)`
- `WorkspaceRunCommand::{manual, automatic_preferred}`
- `RunPanelIntent::{run_manual, resume, set_mode}`
- `dispatch_workspace_run_from_auth_cache(...)`
- `WorkspaceSolveService::{build_request, run_with_property_package, run_from_auth_cache}`
- `WorkspaceSolveService::{dispatch_with_property_package, dispatch_from_auth_cache}`
- `solve_workspace_with_property_package(...)`
- `solve_workspace_from_auth_cache(...)`
- `next_solver_snapshot_sequence(...)`
- `run_studio_bootstrap(...)`
- `src/main.rs` 当前最小 bootstrap 运行入口

当前桥接行为：

- 以 `AppState.workspace.document` 作为当前求解输入
- 由 `StudioAppFacade` 作为当前明确的桌面应用命令入口，统一承接 auth cache 上下文、运行命令执行、结果派发摘要和后续异步执行边界占位
- `StudioAppCommand` 当前已显式区分 `RunWorkspace`、`ResumeWorkspace` 和 `SetWorkspaceSimulationMode` 三类应用命令，便于后续 UI 直接绑定运行控制动作
- `workspace_control` 模块当前已把这三类应用命令进一步收口为更接近运行栏/状态栏的 `WorkspaceControlAction`
- 由 `WorkspaceRunCommand` 承接“触发类型 + package 选择”这一层更接近 UI 的运行请求
- `WorkspaceRunCommand` 当前已改为在 Automatic 且命中 `HoldMode` / `NoPendingRequest` 时先返回 skip，再决定是否需要 package 解析，避免多包场景下因无意义的 preferred 解析而提前失败
- 默认包选择当前采取保守策略：无 entitlement 时仅在本地缓存中唯一包可选时自动选中；有 entitlement 时仅在“本地缓存 ∩ entitlement manifests”唯一时自动选中，多包场景必须显式指定 package
- 由 `WorkspaceSolveService` 负责生成默认 `snapshot_id` / `sequence`
- `WorkspaceSolveService` 明确区分 `Manual` / `Automatic` 触发，并把 `SimulationMode` 与 `pending_reason` 的运行门控收口在应用层
- `ResumeWorkspace` 当前会先把工作区切到 `Active`，再按 Automatic 语义发起运行，作为 `Hold -> Active` 恢复路径的第一版显式应用入口
- 先把 `SolveSessionState` 推进到 `Checking -> Runnable -> Solving`
- 通过 `PropertyPackageProvider` 或 `CachedPropertyPackageProvider` 加载 `ThermoSystem`
- 组装 `PlaceholderThermoProvider + PlaceholderTpFlashSolver + SequentialModularSolver`
- 成功时调用 `AppState::store_solver_snapshot(...)`，把求解结果映射为 UI 层 `SolveSnapshot`
- 失败时调用 `record_failure(...)` 并追加 `AppLogFeed` 错误日志
- Automatic 命中 `HoldMode` / `NoPendingRequest` 的 skip 当前也会写入 `AppLogFeed`
- `main.rs` 当前已通过 `run_studio_bootstrap(...)` + `StudioBootstrapTrigger` 把这条链路接到一个明确的桌面进程触发点，并输出最小运行摘要
- `StudioWorkspaceRunDispatch` 当前已补充 `simulation_mode`、`pending_reason`、`latest_snapshot_summary`、`log_entry_count` 与 `latest_log_entry`，让入口层先消费结构化运行摘要，而不是直接翻读完整 `AppState`
- `StudioWorkspaceModeDispatch` 当前已作为独立结果派发对象承接模式切换结果，避免 UI 侧把“切换模式”和“发起运行”混成同一种返回值
- `WorkspaceControlState` 当前已作为运行栏/状态栏摘要对象，统一提供 mode、status、pending、最新快照摘要和当前可触发动作集合
- `run_studio_bootstrap(...)` 当前也已把 `StudioBootstrapTrigger::{Intent, WidgetPrimaryAction, WidgetAction}` 作为配置入口，并通过 `run_panel_driver` 回收 `RunPanelWidgetModel + WorkspaceControlState`，作为最小桌面入口对运行栏契约的直接消费样例
- `rf-ui` 当前已新增 `RunPanelState`，并由 `AppState::refresh_run_panel_state(...)` 基于 `SolveSessionState`、最新 `SolveSnapshot` 和最新日志自动推导；Studio 也可通过 `WorkspaceControlState -> RunPanelState` 的映射把控制面摘要写回 UI 状态
- `rf-ui` 当前也已补出自有 `RunPanelIntent` / `RunPanelPackageSelection`；Studio 继续只负责把这些 UI 意图映射为 `WorkspaceControlAction` 并执行，避免 `rf-ui` 反向依赖 Studio 类型
- `rf-ui` 当前已把运行栏按钮模型冻结为 `RunPanelCommandModel`：`Run`、`Resume`、`Hold`、`Active` 的按钮描述、可见性、可用性和默认主动作都由 UI 层派生，不再依赖 Studio 侧临时判断
- `rf-ui` 当前也已补出 `RunPanelViewModel`，把主按钮/次按钮槽位、状态标签和最小渲染所需的运行栏数据冻结为 UI 内部展示 DTO
- `rf-ui` 当前进一步补出 `RunPanelTextView`，把当前 bootstrap/CLI 入口所需的最小文本渲染组织也收回 UI 层
- `rf-ui` 当前进一步把“动作是否可触发、触发后产出哪个 `RunPanelIntent`”也冻结进 `RunPanelViewModel` / `RunPanelRenderableAction`，避免最终 widget 再抄一遍启用判断
- `rf-ui` 当前已用 `RunPanelPresentation` 把 `view + text + dispatchable intent` 收口为单一运行栏组件入口
- `rf-ui` 当前进一步补出 `RunPanelWidgetModel` / `RunPanelWidgetEvent`，把最小 widget 激活语义也收回 UI 层
- Studio 当前也已补出 `dispatch_run_panel_widget_event_with_auth_cache(...)`，把 widget 激活结果正式接回 `WorkspaceControlAction` 链路
- Studio 当前进一步补出 `run_panel_driver`，把“构 widget -> 激活动作 -> 分发事件 -> 回收新 widget/control_state”收口为单独模块
- `run_studio_bootstrap(...)` 当前已补出 `StudioBootstrapTrigger::{Intent, WidgetPrimaryAction, WidgetAction}`，并默认走 `WidgetPrimaryAction` 路径，作为最小桌面入口对运行栏组件驱动的第一版接线
- `run_studio_bootstrap(...)` 与 `main.rs` 当前已开始直接消费这组运行栏组件/交互 DTO，而不再只打印控制面布尔摘要或在 Studio 里手拼文本布局

当前已落地与仍待细化的边界：

- 手动运行已经进入真实 GUI 工作台主路径：顶部 `运行` 直接派发 `run_panel.run_manual`，并通过 command registry 的 availability / disabled reason 控制按钮状态
- Home Dashboard 当前是 Studio shell 的默认第一视野；`新建空白 Case / 打开 Case / 打开示例 Case / 继续上次 Case` 只触发文档生命周期动作或 MRU 打开，不写入当前 `FlowsheetDocument`
- `Home / 打开示例 / 新建空白 / 打开项目... / 保存 / 另存为... / 视图 / 命令面板` 当前作为进入 case 后的 Studio shell 主路径；默认隐藏命令大全只是 shell 启动时的 host-local transient layout preference，不写入项目文档语义
- `StudioAppFacade`、`WorkspaceControlAction`、`WorkspaceControlState`、`RunPanelWidgetModel` 与 `run_panel_driver` 已经构成手动运行入口的稳定链路；后续仍待细化的是后台调度、取消、自动运行与 `Hold -> Active` 恢复在最终 GUI 中的完整交互表达
- Studio 当前又已把 app-host 侧 GUI 动作入口进一步冻结为 `StudioAppHostController::dispatch_ui_command(command_id)`，让菜单、快捷键和命令面板后续都可以直接按稳定 command id 触发，而不必继续持有 `UiAction` 枚举或回退到 raw host outcome
- 当前首批已接成真实宿主命令的 run panel command registry 为 `run_panel.run_manual`、`run_panel.resume_workspace`、`run_panel.set_hold`、`run_panel.set_active` 与 `run_panel.recover_failure`；后续桌面命令绑定应优先复用这组 registry，而不是在各入口重复解释 availability、disabled reason 或底层 widget 事件
- Studio 当前又已把 canvas suggestion 与离散 layout nudge 交互正式纳入同一条 command surface：`canvas.accept_focused`、`canvas.reject_focused`、`canvas.focus_next`、`canvas.focus_previous` 与 `canvas.move_selected_unit.left/right/up/down` 当前也应通过 `dispatch_ui_command(command_id)`、`StudioGuiCommandRegistry` 与对应 widget action 统一派发，而不再保留一条长期并行的 widget/shortcut 私有 typed 事件主线；layout nudge 只写 `<project>.rfstudio-layout.json` sidecar，缺少 sidecar 坐标时先按 transient grid slot pin 出初始位置，不进入项目文档 revision/history；selection presentation 与 command result 应显式暴露 `sidecar position` / `transient grid` 来源，避免真实 GUI 再自行猜测 pin 语义
- Studio 当前结果审阅/错误定位入口也已进一步收口为统一 `StudioGuiWindowDiagnosticTargetActionModel`：失败摘要、Result Inspector、Active Inspector 与求解步骤都会汇总可执行诊断目标 action，真实 GUI 只消费这份 presentation 并继续通过既有 `command_id` 派发，不新增一条错误处理或导航私有分支
- Studio 当前 `Result Inspector` 的 `selected_stream / comparison_stream / selected_unit` 也已冻结为 shell-local 视图选择态：它们只决定当前显示哪一块 `SolveSnapshot` 结果面，不缓存第二份结果；若 base stream 切换成当前 compared stream，comparison 允许按现有规则清空，但这仍只是 selector state 复位，不代表结果语义变化
- Studio 当前 near-boundary 结果消费链又已进一步收口到 `window_model -> shell runtime` 的同一条 action surface：`inspector.focus_stream:*` / `inspector.focus_unit:*`、comparison `Inspect`、`DiagnosticTargets` section，以及 shared diagnostic action renderer 的 `source | target | summary` 文本都应继续从同一份 `StudioGuiWindowDiagnosticTargetActionModel` 或既有 focus action 派生；真实 GUI 不应在 runtime 最终渲染层再发明一套 target 语义或导航分支
- Studio 当前 `StudioGuiCommandRegistry` 也会从最新 `SolveSnapshot` 派生 `Results` command section：result stream / unit navigation 只暴露为既有 `inspector.focus_stream:*` / `inspector.focus_unit:*` command，palette、menu、command list 与 runtime 小型 action button 都继续通过 `dispatch_ui_command(command_id)` 进入同一条 host 派发链，不在各自入口复制 target 解析
- Studio 当前失败详情展示也只消费 `WorkspaceControlState.latest_diagnostic` 中已有结构化字段，直接显示 primary code、revision、severity、count 与相关 unit / stream / port target；port target 当前只定位所属 unit，真实 GUI 不应从错误 message 文本里反解析或私造端口级 command
- `StudioAppHostController` 当前对 `DispatchCanvasInteraction` 不应再无条件 `refresh_local_canvas_suggestions()`；local-rules refresh 只应发生在真正改写文档或显式要求重算 suggestion 的路径上，否则会把 `FocusNext/Reject` 刚生成的正式焦点状态冲回首条 suggestion，破坏 GUI 命令面的连续交互语义
- `studio_gui_shell` 当前也已通过 shell 级等价回归锁定 `run_panel.set_active`、`run_panel.recover_failure`、`canvas.accept_focused`、`canvas.reject_focused`、`canvas.focus_next` 与 `canvas.focus_previous` 在菜单、工具栏、命令面板与快捷键之间的共享派发语义；真实 GUI 后续不应再为某个入口保留“看起来一样、实际另走一条逻辑”的私有状态改写分支
- 同时已锁定 disabled 状态下 menu / toolbar / palette 不会偷偷改变工作区或 suggestion 焦点；后续若某个入口需要提示用户，也应停留在 presentation 层，而不是越过 disabled gate 直接改状态
- Studio 当前已冻结第一版字段编辑快捷键策略：`Ctrl+S` 绑定 `file.save`，即使焦点在文本输入框内也继续通过正式 command surface 保存当前项目；`Ctrl+Z / Ctrl+Y` 绑定 `edit.undo / edit.redo`，但文本输入焦点下由输入框自身保留撤销/重做语义，不进入文档历史；普通焦点、画布焦点和 Inspector 面板焦点下才派发文档历史命令；`Enter` 在 Stream Inspector 字段输入中继续只提交当前字段，`Apply all` 仍保持显式按钮/命令，不设置隐式 Enter 批量提交
- `apps/radishflow-studio/src` 当前也已开始按职责做浅层目录治理；`bootstrap`、`studio_gui_shell`、`studio_gui_host`、`studio_gui_driver`、`studio_gui_window_layout`、`studio_window_host_manager`、`entitlement_session_host`、`property_package_download_client`、`auth_cache_sync`、`app_facade` 与 `control_plane_client` 已转为目录模块。后续新增实现应优先并入同域子目录，而不是把大型模块重新铺回 `src/` 根
- 当前虽然已有 `StudioAppFacade`，但结果派发对象仍是最小摘要形态，真正的后台任务调度、取消和更细的事件总线还没有冻结

## 结果快照模型

求解结果当前正式规定为独立快照，而不是直接写回文档对象。

建议最小结构：

- `SolveSnapshot`
- `StepSnapshot`
- `UnitExecutionSnapshot`
- `StreamStateSnapshot`
- `DiagnosticSnapshot`

其中：

- `FlowsheetDocument` 表示“用户当前编辑的真相源”
- `SolveSnapshot` 表示“某个文档版本的一次求解结果”
- `WorkspaceState.snapshot_history` 表示“当前工作区保留的不可变结果窗口”
- `SolveSessionState.latest_snapshot` 只是指向当前结果入口的引用

这种分离有几个直接好处：

- 支持撤回/重做时保留结果快照边界
- 支持未来比较两次求解差异
- 支持按步回放求解过程
- 支持未来的脚本录制与自动化回放
- 避免求解结果污染用户尚在编辑的文档状态

## 按步操作与后续扩展

既然目标是吸收 Aspen、HYSYS、PRO/II 的长处，就不应只停在“得到最终结果”。

当前建议从一开始就保留以下扩展点：

- 每次提交形成显式命令
- 每次求解形成独立快照
- 快照内部保留步骤序列
- 步骤内保留单元执行和流股状态结果

这样后续可以自然扩展：

- undo / redo
- 单步回放
- 操作脚本
- 结果差异比较
- 自动化验证脚本

## 命令与状态变更

建议 UI 侧逐步建立显式命令模型，而不是任由控件直接修改底层对象。

当前建议的动作类别：

- 文档生命周期动作：新建、打开、保存、另存为
- 可撤回文档命令：新增单元、删除节点、连接端口、移动节点、编辑参数
- 运行控制动作：切换 `SimulationMode`、校验、运行、停止、清空结果
- 纯 UI 动作：框选、缩放、平移、面板展开/收起

这样做的好处：

- 便于后续加入 undo/redo
- 便于把 UI 操作映射为可测试事务
- 便于将来接入自动化或脚本入口

补充约定：

- 只有语义提交后的有效变更才进入命令历史
- 文档生命周期动作和运行控制动作都不进入 `CommandHistory`
- 纯画布浏览行为不进入命令历史
- 纯 UI 布局变化默认不触发求解
- 纯几何移动正式归入文档命令，因为它改变流程图持久化几何信息

## Core 与 UI 的数据边界

App 不应直接操作底层求解细节，而应通过稳定的数据结构与服务入口交互。

当前建议边界：

- `rf-model` 提供文档级对象模型
- `rf-store` 提供保存与加载
- `rf-solver` 提供运行入口
- `rf-ui` 只持有对这些能力的调用结果和展示态

求解结果边界进一步补充为：

- `rf-solver` 输出结果快照
- `rf-ui` 决定如何展示快照
- MVP 先冻结为只持久化 `StoredProjectFile` 文档真相源，不默认持久化 `snapshot_history`
- `rf-model` 不直接吞入求解器内部态

认证与授权边界进一步补充为：

- `rf-ui` 持有 `AuthSessionState` / `EntitlementState`
- `rf-store` 持久化授权缓存索引和物性包缓存元信息，但不持久化明文 token
- `apps/radishflow-studio` 作为组合根承接两者之间的桥接，不让 `rf-ui` 直接依赖 `rf-store`
- 项目文件继续采用用户选择路径下的单文件 `*.rfproj.json`，授权缓存与包缓存继续放在应用私有缓存根目录，不混回项目目录
- 授权缓存索引只记录相对缓存路径和安全凭据引用，不把绝对路径和 token 明文写回项目文件
- `rf-thermo` 只通过稳定接口读取已授权物性包，不自行触发 OIDC 流程

## 当前阶段不急着做的 UI 能力

- 复杂 Dock 系统
- 多文档标签页
- 可停靠工具窗口
- 复杂主题系统
- 高级快捷键系统
- 运行时插件化 UI

这些内容未来可能需要，但当前阶段会分散地基建设注意力。

## 近期建议

在继续深化 `rf-ui` 和 `rf-canvas` 代码之前，当前更值得优先推进以下基础设计项：

截至 2026-04-30，Studio 已补出 `DocumentLifecycle(Save / SaveAs)`，保存动作写回 `StoredProjectFile` 并更新 `last_saved_revision`，不进入 `CommandHistory`；同时已补出 `Ctrl+S / Ctrl+Z / Ctrl+Y` 的第一版焦点归属策略，避免文本输入框撤销/重做被文档历史抢走。

项目写入当前采用 `rf-store` 同目录 staged write：先写入临时 sibling 文件并同步，再替换正式 `*.rfproj.json`。在 Unix 类平台上替换走 `rename` 语义；Windows 当前用临时备份做受控替换和失败回滚，不把半写入 JSON 直接覆盖到目标路径。`Save As` 若目标文件已存在且不是当前项目路径，shell 必须先进入显式覆盖确认，确认后才走正式 `DocumentLifecycle`；取消覆盖不改变当前工作区，也不改写目标文件。

1. 在已有字段级草稿、单字段提交、多字段批量提交、基础 undo/redo、保存 / 另存为、快捷键焦点归属、staged project write 和 Save As 覆盖确认闭环上，继续明确更细的保存失败恢复提示和跨平台文件选择策略
2. 继续观察 Stream Inspector 多字段批量提交的 UX 口径；它应继续复用 `InspectorDraftState` 和正式 document command 边界，而不是在 shell 面板里新增私有缓存
3. 在已接通的授权缓存桥接和控制面编排之上，细化 entitlement 刷新后的 UI 事件流与错误呈现口径
4. 在现有 `StudioAppFacade + WorkspaceRunCommand + WorkspaceSolveService` 基础上，继续收口结果派发与后续异步执行边界
5. 冻结求解入口只由应用层触发，画布层仍只处理几何与交互
6. 继续以 `canvas-interaction-contract.md` 作为画布视图模式、流线视觉状态与 suggestion 交互的正式设计入口，避免 `rf-ui`、`rf-canvas` 与未来 `RadishMind` 接线各自发明语义

## 当前仍待细化的问题

以下问题仍值得在正式进入 App 主线前继续细化，但不再属于“方向未定”：

1. `AppLogFeed` 是否只服务 UI 展示，还是也作为后续自动化脚本的导出源
2. 当前 Windows 原生保存选择器只负责选路径，不负责完整跨平台文件工作流；Linux/macOS 文件选择、覆盖确认 UI 的最终视觉形态和更细的失败恢复文案仍需后续细化
