# App Architecture

更新时间：2026-09-12

> 本文定义应用契约与实现边界，后续迭代应保持命令、状态与 snapshot 的一致性；当前任务与优先级见 [当前状态](../status/current.md)。

## 当前目标

用途：记录 MVP β Studio 建模与结果核对的状态、命令和平台边界。
读者：维护 Studio、`rf-ui`、求解桥接和项目生命周期的开发者。
不包含：新的产品排期、完整商业化界面或逐日验证记录。

不扩展自由连线、完整拖拽布局、完整报表或未来型多文档工作台。

## 冻结决策

冻结决策：

1. MVP 保持单文档工作区，不做多文档容器优先设计
2. “单文档工作区”不等于“单文件实现”，代码仍按职责拆分，避免单文件持续膨胀
3. 属性编辑采用字段级草稿态，只有在语义提交时才写回文档
4. 求解控制采用 `SimulationMode` 与 `RunStatus` 分离建模
5. 求解结果采用独立 `SolveSnapshot`，不直接污染 `FlowsheetDocument`
6. 结果快照保留按步展开能力，为后续结果审阅、差异比较和操作脚本留接口
7. 撤销/重做当前采用 snapshot-backed `CommandHistory`：历史记录仍保留语义 `DocumentCommand`，执行时应用对应 `before / after` flowsheet 快照
8. 用户可触达运行入口在求解前统一使用通用 `Flowsheet` readiness 检查真实建模输入，不使用小案例作者清单作为运行 gate

## 顶层分层

桌面应用建议固定为三层协作：

1. `apps/radishflow-studio`
2. `crates/rf-ui`
3. `crates/rf-canvas`

并通过 `rf-model`、`rf-solver`、`rf-store` 等核心 crate 提供数据与服务。

## 各层职责

### `apps/radishflow-studio`

这是应用组合根。

职责：

- 应用启动
- 窗口初始化
- 顶层菜单与工具栏装配
- 工作区与文档生命周期管理
- 将 `rf-ui`、`rf-canvas`、`rf-store`、`rf-solver` 能力组装成桌面应用
- 负责 `AuthSessionState` / `EntitlementState` 与 `StoredAuthCacheIndex` 之间的桥接与同步
- 负责控制面 `entitlement` / `manifest` / `lease` / `offline refresh` 的协议映射、下载租约与本地缓存落盘编排
- 负责从 `PropertyPackageProvider` 或本地 auth cache 组装最小真实求解链路，并把 `rf-solver::SolveSnapshot` 回写到 `rf-ui::AppState`
- 负责把 Studio shell 入口组织为可复现的 MVP α / β 工作流：默认显示 Home，普通空白项目先进入独立 `物性` 页，小案例作者入口可从空白项目直接进入左侧 `模块` 清单，进入 case 后暴露主路径命令
- 负责让 `流程图` / `运行` 上下文工具栏、Run Panel `Resume`、`F5 / Shift+F5`、AppHost、StudioGuiDriver、StudioGuiHost command registry 等正式运行入口复用同一层建模输入 readiness；shell 只负责 notice、focus 和用户反馈，不在各入口复制另一套输入判断
- 负责在 GUI shell 层提供用户操作与求解审计输出；默认 stderr 日志只作为开发态 smoke 和诊断入口，不替代未来正式审计 / telemetry 设计
- 负责遵守 `eframe` / `winit` 事件循环约束：Windows 事件循环在主线程创建；干净最后窗口 close 不得被 `CancelClose` 拦截，关闭前清理逻辑窗口并停止当帧 fallback 布局；脏工作区 close 必须先确认保存 / 舍弃 / 取消
- 负责桌面窗口渲染后端选择：macOS 开发态使用 `eframe/wgpu` + Metal backend；裸二进制系统日志后续由 `.app` bundle / `Info.plist` / 签名打包流程治理

不应承担：

- 热力学计算
- 单元求解逻辑
- 目标上不承担画布图元绘制细节；当前绘制实现仍在 Studio，尚未移至 `rf-canvas`
- 项目文件读写细节
- 把视觉主题、按钮样式、面板密度或日志展示直接写成求解 / 文档语义

### Studio Shell UI 规范化边界

Studio 首页、工作台分区、运行后结果视图和项目切换确认流程已落地。shell UI 边界按以下稳定入口治理：

- Home Dashboard 是启动默认首页，只承载 Start actions、Recent Cases、Example Cases、Environment 和 Messages；不读取 `SolveSnapshot`，不直接承载流程图编辑。`新建项目` 创建普通空白项目后先进入独立 `物性` 页；小案例作者入口只创建空白项目并切到左侧 `模块` 清单，清单只读 canvas，不生成 flowsheet、不写项目、不进 undo。
- 最近项目、当前工作区和示例项目统一由 `StudioGuiWindowHomeCaseTileModel` 承载流程缩影、路径 / 来源、物性包 label、组分和状态。当前工作区 tile 只从 `workspace_document` 派生，不写入 recent projects；保存过的当前项目应直接返回当前 workspace，而不是通过 recent path 重新打开自身。Home tile 的 `package_summary` 只从当前 document / builtin package choice 映射为可读 label，例如 `二元烃 Lite`；稳定 package id 只留在项目文件、command id、运行请求和内部状态边界。
- 新建、打开、case tile 双击、Home `返回工作区`、窗口关闭按钮和 macOS `Cmd+Q` / 应用退出请求共享同一条工作区生命周期语义。若有未保存变更，必须先进入保存并继续 / 舍弃并继续 / 取消确认；取消、保存失败、另存为取消或覆盖确认未完成时保持当前工作区、MRU 和 `FlowsheetDocument` 不变。
- 进入 case 后，第一层只保留 `文件 / 主页 / 物性 / 流程图 / 运行 / 结果 / 工具 / 设置` 八个主入口、当前项目摘要和必要状态。普通空白项目选齐 package 和至少一个项目组分前，`流程图` 入口不可用；禁用原因必须来自 Property page DTO 的同一 readiness。
- `物性 / 流程图 / 运行 / 结果` screen 下方分别消费 `window.property_context_toolbar`、`window.flowsheet_context_toolbar`、`window.run_context_toolbar` 和 `window.result_context_toolbar`。工具栏只展示当前 screen 的主路径命令和状态，不展开调试命令全集；`运行` 工具栏不重复 Monitor 状态 chip，`结果` 工具栏不把所有 stream / unit focus command 展开成长按钮。
- 工作台分区固定为左侧 `模块 / 项目`、中央 Canvas、右侧 `检查器 / 模块设置 / 模块结果`、底部 `消息 / 运行日志 / 收敛 / 建议 / 诊断 / 结果表` 和状态汇总。`模块` 消费 Canvas place-unit palette 并按 `流股源 / 调节单元 / 汇合与分离` 分类；分类和选项 detail 可进入 hover / DTO，不作为首屏常驻说明。`项目` 负责项目输入、示例入口、对象树和审阅状态；项目级输入编辑主入口仍是独立 `物性` screen。左侧 `项目输入` 和独立 `物性` 页同样展示可读 package label，不从运行结果反推第二套物性包状态。
- `模块设置` 只消费 `StudioGuiWindowModuleSettingsModel`，从 active unit Inspector detail 派生参数摘要、字段、端口、连接动作和诊断动作；参数摘要不得混入 latest-result。`模块结果` 只消费 `StudioGuiWindowModuleResultsModel`，从 current-revision latest `SolveSnapshot` 派生 selected unit result、consumed / produced stream、related steps、diagnostics 和 diagnostic actions。
- 成功运行后 shell 可聚焦顶部 `结果` screen、右侧 `模块结果` 和底部 `结果表`，失败后聚焦顶部 `运行` screen、底部运行日志或诊断。结果面只读消费当前 revision 的最新 `SolveSnapshot`；stale snapshot 只显示过期提示，不继续驱动 Result Inspector、结果表、Results commands、复制 / 导出或 `review_summary`。
- 底部 `结果表` 的 stream / unit 行必须派发正式 `inspector.focus_stream:*` / `inspector.focus_unit:*`，分别定位到右侧 `检查器` / `模块结果`，底部仍停留在 `结果表`。底部状态汇总只消费 `StudioGuiWindowStatusSummaryModel`，snapshot 一致性等低高度信息可以放在标题行，不另建 shell 私有摘要。
- 开发态 stderr 与 GUI activity 可继续服务 smoke，但正式 UI 只展示用户能采取行动的摘要，不把平台 timer、`TimerElapsed`、`SystemTime` 或 host internals 混入主路径。

当前 Studio UI 主设计稿已收敛到 `docs/architecture/designs/studio-client-main.pen`。它是设计目标，不代表当前代码已达到最终视觉细化。实现时必须复用既有 `WorkspaceDocument`、inspector draft、command surface、run panel state 和 latest current-revision `SolveSnapshot`；布局变化不得新增私有选择、结果、诊断或参数缓存。

### Studio Main Presentation 边界

Studio UI 专题代码实现从 `StudioGuiWindowModel` 派生 DTO 开始，不先做 egui 大布局重排。当前 `studio_main` 已投影 Home case tile、Property page、四类 context toolbar、底部 status summary、Module Settings、Module Results 和结果表定位语义；详细映射见 `docs/architecture/designs/studio-client-main-brief.md`。

egui shell 可以消费这些 DTO，但不能把它们变成第二套项目、物性、运行、结果或诊断真相源。Home recent tile 当前仍由 shell preferences / `project_open` 映射为同一 DTO，当前工作区 tile 只从 `workspace_document` 派生；Property toolbar、Property page metric、Home case tile 和 Project input sweep 都应从同一 document / choice 状态映射可读 label，不能在 renderer 里直接铺 raw package id。后续细化 `物性`、`流程图`、`运行`、`结果` 与右侧三 tab 时，仍必须继续按正式 presentation / command / state 来源推进。

`studio_gui_shell/panels/runtime/` 当前拆为 `runtime/mod.rs`、`runtime/results.rs` 和 `runtime/inspector.rs`。Module Settings 和 Module Results 细化必须先补正式 window model DTO 和 focused 回归，再让 runtime 子模块消费；不得在 shell 中私造结果、诊断、端口、参数或帮助命令缓存。当前模块帮助没有正式 command surface，只允许在 DTO 中表达为空状态。

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

这是历史目标中的纯画布能力层，不应承载流程求解或业务决策。当前 `rf-canvas` 仍只有占位函数，实际绘制、命中与拖动代码位于 Studio；下列职责是目标拆分，不表示该 crate 已实现这些能力。

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

最小状态对象已在上文按 ownership 和字段清单分别冻结；这里不再重复完整 Rust 结构体草案，避免文档同时维护两份同源字段列表。实现以 `rf-ui` 当前类型定义为准，本文只保留架构边界。

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
- 项目级物性包和组分选择属于文档语义输入；空白项目不自动补 package / components，Stream Inspector 只能从当前 `Flowsheet.components` 中添加组成条目
- Stream Inspector 的 `T / P / F / composition` 也采用草稿提交；普通 Studio 运行入口会在缺少 Feed composition 时先走建模输入 readiness，已经进入求解阶段的 stream 输入不一致仍可归类为 `solver.step.stream_input`，并携带 stream / inlet target
- Unit Inspector 参数：`Feed`、`Heater / Cooler`、`Flash Drum` 写回 `outlet_temperature_k` / `outlet_pressure_pa`，`Mixer`、`Valve` 写回 `outlet_pressure_pa`；提交 `SetUnitParameter` 同步模板，Mixer pressure 不高于 inlet pressure，Heater / Cooler / Valve 不高于 inlet pressure；若字段值来自 outlet stream 模板、内置默认值或其他兼容 fallback，而 unit parameter 尚未显式存在，同值提交仍应生成正式参数命令
- GUI window-model 必须把“显示值有效但缺显式 unit parameter”的字段暴露为可提交状态，并提供正式 `commit_command_id`；这种状态不是普通已同步字段，也不是控件私有 fallback
- Unit Inspector 参数字段必须携带 SI 单位和约束 presentation；无效草稿不写文档/历史/模板。已入文档的无效参数由 `solver.step.parameter` 等诊断暴露，并携带 unit / port / stream context
- 运行前 readiness 只读取已提交的文档态输入，不读取 Inspector 草稿，也不自动补写默认值；未就绪时 shell 显示“模型输入未完成”并聚焦到对应 stream / unit。缺物性包、缓存缺失或多包歧义继续交给正式 run package resolution 和 Run Panel 诊断

采用这个方案的原因：

- 避免用户输入 `100` 时触发 `1 -> 10 -> 100` 三次无意义求解
- 比整页“大草稿后统一应用”更接近工程软件的即时反馈体验
- 能天然接入命令历史，而不是让半成品输入污染撤回栈

当前约定：

- 草稿态不进入命令历史
- 只有成功提交到文档的变更才形成命令
- 只有影响方程系统的提交才触发求解相关检查

### 物性到流程图建模 readiness

普通空白项目的第一条主路径是 `新建项目 -> 物性 -> 选择物性包 / 项目组分 -> 流程图建模`。这层 readiness 只判断用户是否已经完成项目级建模前置输入：

- 当前 `workspace_document` 已选择 property package。
- 当前 `workspace_document` 已选择至少一个 project component。

`StudioGuiWindowPropertyPageModel` 负责从当前 `workspace_document.property_package_choices`、`project_component_choices`、已选 package 和已选 components 派生 `flowsheet_modeling_enabled`、status label 和 detail。独立 `物性` 页摘要、`window.property_context_toolbar` 的 `Modeling` 分组、顶部 `流程图` 导航和 Home 当前工作区 tile 返回入口都必须消费这同一份 DTO 状态。

这层 readiness 只决定是否允许用户从 Property 主路径进入 Flowsheet authoring，不替代正式运行命令的 package resolution、求解前建模输入检查、结构连接诊断或求解阶段失败诊断。进入 Workbench 后，左侧 `项目` 面板可以扫读 package / components，但不应复制另一套项目级物性状态。

### 运行前 readiness

Studio 的用户可触达运行入口在调用正式 Run Panel 求解命令前，会先做一层通用建模输入检查。当前包括 `流程图` / `运行` 上下文工具栏中的 `运行当前流程`、Run Panel `Resume`、`F5 / Shift+F5`、命令面板、AppHost、StudioGuiDriver 与 StudioGuiHost command registry 等正式入口。它的职责是阻止明显未完成的建模输入进入求解器，让用户先回到具体 stream / unit 补齐输入。

当前检查范围：

- 至少存在一个 unit。
- 项目必须至少选择一组 project components；stream composition 中引用的 component 必须已经进入项目组分列表。
- Feed source stream 必须具备正有限 `temperature_k`、`pressure_pa`、`total_molar_flow_mol_s`，并具备非空、数值有效、归一到 1 的 `overall_mole_fractions`。
- `Heater / Cooler / Flash Drum` 必须提交 `outlet_temperature_k` 和 `outlet_pressure_pa`。
- `Mixer / Valve` 必须提交 `outlet_pressure_pa`。

明确不属于这层 readiness 的内容：

- 不解析或选择 property package。缺物性包、缓存缺失或多包歧义继续交给正式 run package resolution 和 Run Panel 诊断。
- 不替代结构性连接 / 拓扑诊断。缺 material port 绑定、坏 stream reference、重复 source / sink、orphan stream、cycle 等问题继续走正式 Run Panel 诊断 / recovery。
- 不消费 `Mixer-Flash` / `Heater-Flash` 作者清单状态。作者清单只作为导航提示，不作为普通空白项目的运行 gate。
- 不隐式归一 composition，不隐式写入 unit 参数，不用 outlet stream template 代替用户提交的 `UnitOperationParameters`。求解器保留对旧项目的兼容 fallback，但 Studio 运行前检查以用户已提交的文档态建模输入为准。

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
- `ResumeWorkspace` 当前会先复用同一层 package / readiness preflight；如果建模输入未就绪则保持 `Hold` / pending 状态并返回 modeling notice，只有 preflight 通过后才切到 `Active` 并按 Automatic 语义发起运行
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
- `run_studio_bootstrap(...)` 通过 `run_panel_driver` 直接消费 `RunPanelWidgetModel + WorkspaceControlState`，作为桌面入口样例
- `rf-ui` 已把运行栏收口为 `RunPanelState`、`RunPanelIntent`、`RunPanelCommandModel`、`RunPanelViewModel`、`RunPanelPresentation` 和 `RunPanelWidgetModel`；Studio 侧只派发 widget 事件，不重复判断按钮和 intent。

当前已落地与仍待细化的边界：

- 手动运行已经进入真实 GUI 工作台主路径：`流程图` / `运行` 上下文工具栏中的 `运行当前流程` 派发 `run_panel.run_manual`，并通过 command registry 的 availability / disabled reason 控制按钮状态。`StudioAppFacade`、`WorkspaceControlAction`、`WorkspaceControlState`、`RunPanelWidgetModel` 与 `run_panel_driver` 构成稳定链路；后台调度、取消、自动运行与 `Hold -> Active` 恢复仍留给后续 GUI 交互细化。
- Studio app-host GUI 动作入口统一为 `StudioAppHostController::dispatch_ui_command(command_id)`。run panel command registry 首批稳定命令为 `run_panel.run_manual`、`run_panel.resume_workspace`、`run_panel.set_hold`、`run_panel.set_active` 与 `run_panel.recover_failure`；菜单、快捷键、命令面板、palette 和 runtime 小型 action button 都应复用这条派发链。
- Canvas suggestion、layout nudge、单元拖动和选中流股恢复已纳入同一条 command surface。layout nudge / 单元拖动只写 `<project>.rfstudio-layout.json` sidecar；`canvas.disconnect_selected_stream*`、`canvas.reconnect_selected_stream` 和 `canvas.delete_selected_stream` 是无自由连线阶段的受控恢复动作，进入 `CommandHistory`，但不得扩成端口选择器、自由连线、自动布线或完整拖拽布局。
- 结果审阅、错误定位和诊断目标都必须复用 `StudioGuiWindowDiagnosticTargetActionModel`、`inspector.focus_stream:*`、`inspector.focus_unit:*` 或既有 focus action。`selected_stream / comparison_stream / selected_unit` 只是 shell-local selector state，不缓存第二份结果；comparison 复位不代表结果语义变化。
- `StudioGuiCommandRegistry` 从最新 `SolveSnapshot` 派生 `Results` command section；result stream / unit navigation 只暴露为正式 focus command。顶部 `结果` 工具栏只扫读结果入口和状态，不承担所有对象定位按钮。
- Module Settings、Module Results、case-level `review_summary` 和 `stale_solve_snapshot` 的边界见本文上方 `Studio Shell UI 规范化边界` 与 `docs/reference/solve-snapshot-results.md`。它们都服务当前 revision 的结果审阅和轻量导出，不成为第二套结果缓存或报表模型。
- 失败详情只消费 `latest_diagnostic`，显示 primary code、revision、severity、count 与相关 target；GUI 不从 message 文本反解析或私造端口级 command。Run Panel recovery action 必须区分聚焦与修复，用户主动选中流股后的恢复动作走对应 `canvas.*selected_stream*` 命令，不复用 failure-only recovery command。
- `StudioAppHostController` 对 `DispatchCanvasInteraction` 不应无条件 `refresh_local_canvas_suggestions()`；local-rules refresh 只应发生在真正改写文档或显式要求重算 suggestion 的路径上，避免破坏 GUI 命令面的连续交互语义。
- `studio_gui_shell` 已通过 shell 级等价回归锁定 run panel、canvas suggestion、layout nudge、选中流股恢复和 disabled gate 在菜单、工具栏、命令面板、Canvas / Inspector 入口之间的共享派发语义；后续提示应停留在 presentation 层，不越过 disabled gate 改状态。
- 字段编辑快捷键策略当前冻结为：`Ctrl+S` 始终保存；`Ctrl+Z / Ctrl+Y` 在文本输入焦点下由输入框处理，普通焦点、画布焦点和 Inspector 面板焦点下才派发文档历史命令；`Enter` 在 Stream Inspector 字段输入中只提交当前字段。
- `apps/radishflow-studio/src` 已开始按职责做浅层目录治理；`bootstrap`、`studio_gui_shell`、`studio_gui_host`、`studio_gui_driver`、`studio_gui_window_layout`、`studio_window_host_manager`、`entitlement_session_host`、`property_package_download_client`、`auth_cache_sync`、`app_facade` 与 `control_plane_client` 已转为目录模块。后续新增实现应优先并入同域子目录。

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

## 维护性与同步执行现状

`WorkspaceSolveService` 在 dispatch 中同步调用求解桥接并回写 AppState；GUI event dispatch 也沿现有 host / driver 路径同步执行。当前运行状态名称和 timer 状态机不代表已有后台求解、取消或并行执行能力。本次未测量 GUI 帧耗时、复杂流程延迟或快照内存，不能据此声称已出现性能回归。

若以后获准处理复杂度，应按用户行为追踪命令链，保留承担独立状态、平台隔离和共享契约的层，评估纯转发与重复投影。大型文件按项目生命周期、建模命令、结果审阅或平台 IO 等职责划分，测试按行为和失败模式组织，不为缩短文件机械切片。现有 snapshot-backed undo 与结果快照在更大负载下的成本也应先测量。

较重求解若进入范围，再确认后台执行、取消与旧 revision 结果丢弃的正式边界；不得把本段建议当作已经批准的架构改动。

## 未排期事项

以下问题保留为历史待评估项，不授权启动实现：

- 保存失败恢复、跨平台文件选择与偏好范围见 [项目生命周期专题](../topics/project-lifecycle-storage.md)。
- Inspector 多字段提交与草稿继续复用正式命令，不在 shell 建立另一份输入状态。
- `AppLogFeed` 的脚本导出用途、真实授权刷新 UI 和后台求解仍需在实际需求进入范围时确认。
- 画布视图与 suggestion 的目标契约见 [Canvas 交互边界](canvas-interaction-contract.md)，当前不扩展复杂 Dock、多文档或运行时插件化 UI。
