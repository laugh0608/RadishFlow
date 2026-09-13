# 流程图建模与求解闭环

更新时间：2026-09-13

> 本文定义该专题的能力、开发范围与验收要求；具体迭代切片和优先级以 [当前状态](../status/current.md) 为准。

2026-09-13 起，以 [Studio 基础功能切片](studio-main-workflow.md#基础功能完善切片) 复核和完善编辑 / 连接闭环。下文受控端口与无回路限制描述现有基线；扩展端口选择或连接规则前在本专题定义相应范围和验收。长期动态、瞬态和多求解策略见 [模拟平台规划](../architecture/simulation-platform.md)，不属于本次基础编辑实现。

后续执行计划优先演进为组合块、循环块与原子求解块构成的递归求解树，定义见 [递归分块与原子求解块](../architecture/simulation-platform.md#递归分块与原子求解块)。按真实依赖分块并处理局部收敛；变量浏览树与画布分组不直接决定求解树。本轮仍未实现循环块或塔模型。

## 用途

用途：定义受控流程图建模、单元参数、连接建议、readiness、Run Panel 和 solver 之间的开发边界。
读者：负责 `rf-model`、`rf-flowsheet`、`rf-unitops`、`rf-solver`、Studio 建模入口和运行门禁的开发者、用户、AI / Agent。
不包含：自由连线编辑器、任意端口选择器、自动布线系统、完整拖拽布局器、复杂回路收敛和动态模拟。

## 专题目标

- 普通空白项目在受控范围内搭建并运行小流程，而不是依赖固定 demo case gate。
- readiness 只拦截确定的建模输入缺失；结构性连接、拓扑、非法旧项目或求解阶段参数失败交给正式 Run Panel 诊断 / recovery。
- Rust Core 不引入 COM / CAPE-OPEN 语义，继续使用领域模型、单元、连接和 solver 边界表达能力。

## 子专题

本专题是流程图建模与求解的一级轨道。具体单元和建模对象由二级功能专题承载：

| 子专题 | 状态 | 入口 |
| --- | --- | --- |
| Feed / 进料源 | Active | `unitops/feed-source.md` |
| Heater / Cooler 换热器 | Active | `unitops/heater-cooler.md` |
| Flash Drum 闪蒸罐 | Active | `unitops/flash-drum.md` |
| Mixer 混合器 | Active | `unitops/mixer.md` |
| Valve 阀门 | Active | `unitops/valve.md` |
| Material Stream 物流股 | Active | `modeling/material-stream.md` |

后续涉及具体单元行为、参数、端口、结果或诊断时，应优先更新对应子专题；本文件只保留跨单元的建模 / 求解边界。

## 当前实现快照

已完成：

- 支持 `Feed -> Flash Drum`。
- 支持 `Feed -> Heater/Cooler/Valve -> Flash Drum`。
- 支持 `Feed + Feed -> Mixer -> Flash Drum`。
- Feed source stream T/P/F/z、composition 归一和必要单元参数进入 readiness。
- Canvas suggestion 可显式接受连接或创建 outlet stream。
- Unit Inspector 参数提交走正式文档命令，并支持保存 / 重开 / rerun。

已知缺口：

- 受控连接恢复已满足当前 MVP，但不是自由连线编辑器。
- Cooler / Valve 有 focused 覆盖，但不新增同构 Home 作者入口。
- 完整回路、塔器、动态模拟和复杂单元均未进入当前阶段。

## 用户路径

1. 在 Workbench 左侧 `模块` 放置受控单元。
2. 通过 Canvas suggestion 补齐 canonical material ports。
3. 在 Inspector / 模块设置提交 Feed source stream、composition 和单元参数。
4. 运行前 readiness 提示明确输入缺口。
5. 运行后 solver 生成当前 revision 的 `SolveSnapshot`。
6. 若结构、拓扑或求解阶段失败，Run Panel 诊断和 recovery action 指向具体 stream / unit / port。

## 范围

本专题纳入：

- 受控单元：Feed、Mixer、Heater / Cooler、Valve、Flash Drum。
- canonical material ports 和一股一源一汇校验。
- 建模 readiness 与 Run Panel 诊断的边界。
- Canvas suggestion、Inspector focus 和参数提交。
- 保存 / 重开 / rerun 的建模输入稳定性。

本专题不纳入：

- 自由连线、任意端口选择和自动布线。
- 完整拖拽布局编辑器；当前布局仍为 sidecar / GUI state。
- 完整 recycle 收敛、动态模拟、CFD。
- 复杂单元操作、严格塔器和完整物性数据库。
- CAPE-OPEN / COM 宿主语义。

## 设计边界

### 数据与状态

- `rf-model` 承载对象模型，不承载求解策略或 COM 语义。
- `rf-flowsheet` 承载 graph / port / connection 校验。
- `rf-unitops` 围绕 `MaterialStreamState` 和单元参数。
- `rf-solver` 承载顺序模块法执行。
- Studio 只通过 document command 和 presentation 消费这些能力。

### 命令与接口

- 建模修改必须进入 `DocumentCommand` / command history。
- Canvas pending edit 在提交前不写项目语义。
- 连接 suggestion 的接受动作必须明确是连接流股还是创建流股。
- readiness 不扩成第二套 solver。

### UI 与交互

- 放置入口归左侧 `模块`。
- 对象树归左侧 `项目`。
- 选择语义和参数归右侧 Inspector / 模块设置。
- 运行反馈归 Run Panel、底部和诊断，不塞回 Canvas 私有状态。

## 分阶段切片

### B0 编辑能力复核（2026-09-13）

复核基线为 `15ac7b97`，平台为 macOS ARM64、Rust 1.96.0。真实窗口从 Home 的“新建项目”开始，未使用作者案例或预置流程；操作流水见 [W37 B0 记录](../devlogs/2026-09/2026-W37.md#2026-09-13-b0-基础能力复核)。下表状态只适用于写明的范围，测试通过不替代其他平台实窗验收。

| 能力与状态 | 代码 / 测试依据 | 本轮实窗结论与剩余边界 |
| --- | --- | --- |
| Feed、Flash 创建：已验证可用 | [Canvas 创建与六种单元矩阵](../../crates/rf-ui/src/tests/canvas.rs)、[空白项目 shell 测试](../../apps/radishflow-studio/src/studio_gui_shell/tests/canvas.rs) | Palette 进入放置态，点击画布提交；得到 `feed-1`、`flash-1`，端口和选择同步。Heater / Cooler / Valve / Mixer 创建矩阵测试通过，四种单元本轮实窗待验证 |
| 参数编辑：部分可用 | [Inspector 测试](../../crates/rf-ui/src/tests/inspector.rs)、上述空白项目测试 | Feed 温度 `-1 K` 草稿显示错误且无应用入口；修正为 `250 K` 后关联流股同步。Flash `200 K / 100000 Pa` 提交后进入求解。Feed F/z、其他单元参数有测试，本轮未逐字段实窗提交；不能称所有参数均已 GUI 验收 |
| 受控连接：已验证可用 | [连接应用与唯一端点规则](../../crates/rf-ui/src/state/actions.rs)、[恢复测试](../../crates/rf-ui/src/tests/recovery.rs) | 建议创建 Feed source stream，连接 Flash inlet，再创建 liquid / vapor 两出口。完整连接时重连禁用；无自由端口选择器，不能将受控建议等同任意连线 |
| 单端断开与唯一目标重连：已验证可用 | 上述恢复测试、[GUI 消费测试](../../apps/radishflow-studio/src/studio_gui_host/interaction_tests.rs) | 实窗断开目标端保留源端与 T/P/F/z；重连恢复同一流股和目标。源端断开、整股断开由定向测试验证，本轮实窗待验证 |
| 无效 / 歧义连接保护：部分可用 | 恢复测试覆盖缺端无操作、无端整股断开无操作、歧义 source 和形成环的 sink 不提交 | 唯一候选才允许重连；相关测试通过。多候选、自环及占用端口的逐项实窗拒绝路径待验证，未登记为缺陷 |
| 流股删除及关联清理：已验证可用 | `apply_delete_stream_and_disconnect_ports_mutation`、恢复与 GUI 消费测试 | 删除已连接 Feed 流股，两单元保留、两端绑定清除、流股从 3 条变 2 条、选择清空；一次 Undo 恢复同一流股与两端绑定，rerun 通过 |
| 单元删除 / 重命名：缺失 | [DocumentCommand](../../crates/rf-ui/src/commands.rs) 只有对应类型；`DeleteUnit` 仅被历史容器测试使用，`RenameUnit` 另见自动运行测试标签 | 未找到模型删除方法、领域动作或 Studio 用户命令消费；实窗单元操作只有聚焦和移动等，没有删除 / 重命名。历史中记录一个命令标签不证明该动作可执行 |
| Undo / Redo：部分可用 | [文档历史 driver](../../apps/radishflow-studio/src/document_history_driver.rs)、恢复测试 | 命令面板撤销 / 重做目标端断开及撤销流股删除通过；快捷键注入未观察到文档变化，焦点和原生键盘路径待专项验证，不据此判定快捷键实现缺陷 |
| 结果旧化与 rerun：已验证可用 | 空白项目 `blank_project_editing_inputs_invalidates_current_results_until_rerun` 与连接旧化测试 | 求解后断开即显示过期；Undo 恢复拓扑仍保持过期，重连不冒充新结果；删除撤销后 rerun 生成新快照并恢复当前。所有状态都来自文档 revision 与快照身份 |
| 流股导航反馈：部分可用 | [Canvas 导航与 presentation](../../apps/radishflow-studio/src/studio_gui_canvas_presentation/mod.rs) | Inspector 能选中并操作真实存在的 Feed 流股，但 Canvas 显示 `Canvas navigation anchor expired`；记录为反馈不一致，根因与其他流股范围待定位，不等同流股已丢失 |

保存 / 重开能力分层结果见 [生命周期 B0 复核](project-lifecycle-storage.md#b0-保存与重开复核2026-09-13)。本轮证据使用演示物性验证软件行为，流量有限、非负、组成归一及出口总摩尔流量一致；不作为真实物性精度或独立能量模型认证。

### B1-1 单元删除与关联清理（2026-09-13）

具体缺口是误放单元后不能直接删除，也无法移除已有设备继续编辑。首刀补完整的单元删除事务，复用已验证的流股断开 / 删除、文档历史与状态投影。重命名、自由连线、快捷键和文件选择器分别收口，不一次扩成完整编辑器。

已确认并实现的范围与行为：

- 为六种现有单元提供统一“删除单元”动作，经现有 driver / document command 提交；项目树和画布选择共享同一入口与可用性。
- 删除前展示对象名称 / ID 和关联流股影响；确认后仅删除所选单元及其自身端口绑定，保留所有流股 ID、规格与其他单元上的绑定。孤立或缺源流股交由现有 readiness / Run Panel 说明，用户可显式删除流股，不自动级联删除相邻设备或流股。
- 一个成功删除对应一个 history entry 和一次 revision 推进；取消、对象已失效或校验失败不改变文档、历史、脏状态与当前结果。错误使用结构化领域上下文，不通过展示字符串驱动事务。
- 提交后清理已失效选择、Inspector 草稿及连接建议；旧结果保持过期且不可作为当前结果读取。画布不显示已删除单元，Undo 后对象、参数、连接和位置可恢复；布局继续属于 sidecar，不混入工程输入。
- 保存重开后删除结果不复活；历史沿用现有会话语义，不新增跨重开的撤销承诺。删除后重新创建对象时，不得错误继承旧对象的草稿、结果或布局。
- 继续使用 SI、稳定对象 ID 和正式命令事务，保留未来变量树 / 录制动作可复用语义；不引入脚本运行时、COM 自动化、求解树、动态模型、公共万能 API 或项目格式占位字段。

最小验收：

1. 从空白项目分别创建六种单元，删除未连接单元；取消不变，成功后计数 / 选择正确，Undo / Redo 可重复。
2. 在 `Feed -> Heater -> Flash` 删除中间 Heater；两关联流股保留原规格和外侧绑定，其他设备不变。缺源 / 缺输入能定位，不静默重接；Undo 一次恢复全部关联并可 rerun。
3. 删除 Feed、有多个 inlet 的 Mixer、双出口 Flash，逐项验证全部自身绑定的影响；测试 stale ID、重复点击、未提交草稿、删除后新建及历史分支截断。
4. 已求解项目执行删除 / Undo / Redo：过期结果不能重新标成当前；修复后 rerun 生成新身份并满足现有守恒 / 非有限输入回归。
5. 已保存项目和空白项目都覆盖删除后的保存重开，再覆盖 Undo 后保存重开；验证 ID、参数、端口、sidecar 与 rerun。底层测试必须使用真实临时文件，实窗必须记录平台及文件选择器路径。
6. 定向模型 / command / driver / shell 测试通过后执行 `check-repo`；真实窗口验收不以 mock picker 代替。macOS 原生 picker 已独立补齐，Windows / Linux 新增路径仍需分别验证。

项目所有者确认后已实现以上事务与 Studio 入口。`Flowsheet::remove_unit` 保留所有流股；AppState 通过 `DeleteUnit` 提交单次变更；确认绑定文档 ID、revision 和当前选择，失效或重复确认不提交。会话内保留已分配单元 ID，历史分支截断也不复用；已删除布局留在会话中供 Undo，写入 sidecar 时只保留当前单元。项目格式不变。

| 验证层 | 状态与证据 |
| --- | --- |
| 六类单元及事务边界 | 已验证可用：[rf-ui 删除测试](../../crates/rf-ui/src/tests/unit_deletion.rs) 与 [shell 删除测试](../../apps/radishflow-studio/src/studio_gui_shell/tests/unit_deletion.rs) 覆盖六类空白创建 / 删除、全部端口关联保留、取消 / 失效确认、草稿清理、历史分支、ID 和布局恢复 |
| 中间设备 GUI 删除闭环 | 已验证可用：macOS 普通空白 Feed–Heater–Flash，影响确认 / 取消、删除、Undo / Redo、旧化、保存删除版与恢复版、原生重开 / rerun；删除版报告 `solver.connection_validation.missing_upstream_source`，不静默补线 |
| 其余五类单元逐项实窗 | 待验证；自动化矩阵通过不等同全部 GUI 路径验收 |
| 原生持久化 | macOS 本轮主路径已验证，扩展边界见 [生命周期 B1](project-lifecycle-storage.md#b1-macos-原生项目文件选择器2026-09-13)；Windows / Linux 本轮待验证 |

B0 表是旧基线记录，其中“删除缺失”已由本节覆盖，“重命名缺失”由下述 B1-3 覆盖。

### B1-2 Canvas 导航反馈（2026-09-13）

已修复存在的单元 / 流股被误报 `Canvas navigation anchor expired`。根因有两处：同一帧前面的控件已更新选择，画布却仍用旧快照校验新导航；拓扑变化使布局序号改变，旧逻辑只比较锚点字符串，没有按对象身份重新解析。

- 画布几何绘制与导航校验共同读取当前 presentation；项目树、本帧早先的工具栏动作不再造成误报。
- 使用对象种类及 ID 在当前对象列表定位；同一对象的布局锚点变化时重新请求定位。普通非导航命令不清除有效导航状态，清空或改变选择不称为对象已消失。
- 真正删除流股后仍给出失效提示；Undo 恢复后清除旧警告，重新选择可再次定位。选择与反馈不写入工程输入，不改变求解、文件格式或已有布局策略。
- 定向测试覆盖旧帧绘制、普通空白 Feed–Heater–Flash 连接后的布局重排、清空选择、删除 / Undo；macOS 实窗验证项目树 Heater、流股定位、断开重连、直接点击画布 Feed、删除恢复与 rerun。Windows / Linux 本轮实窗未执行，超出当前视口的远距离平移仍待专项验证。

### B1-3 单元重命名（2026-09-13）

已接通六类现有单元的 Inspector 名称编辑，复用草稿、应用与放弃入口，并以正式 `RenameUnit` 命令提交。名称沿用现有流股规则：拒绝空字符串与全空白，允许重名，完整保留非空白输入中的中文和空格；名称是显示属性，对象身份继续使用 ID。

- 有效改名只改变所选单元的 `name`，保留 ID、参数、端口、绑定、相邻对象、流股名称和布局。项目树、画布、Inspector 和命令面板从当前文档刷新显示。
- 一次提交对应一条历史和一次 revision；相同名称、无效输入、放弃草稿与失效对象不提交。Undo / Redo 使用既有文档历史；有效改名后结果旧化，Undo 不将旧快照重新当作当前结果，rerun 生成当前快照。
- 名称随项目保存，位置仍由 sidecar 保存；项目格式、依赖和会话历史持久化边界不变。不增加 ID 重命名、自动级联命名、自由连接或变量树 / 录制框架。

| 验证层 | 状态与证据 |
| --- | --- |
| 六类单元名称事务 | 已验证可用：[rf-ui 重命名测试](../../crates/rf-ui/src/tests/unit_rename.rs) 覆盖中文含空格、单次历史 / revision、Undo / Redo、空白 / 同名 / 取消 / 失效对象、允许重名 |
| 显示、连接与持久化 | 已验证可用：[shell 重命名测试](../../apps/radishflow-studio/src/studio_gui_shell/tests/unit_rename.rs) 覆盖已连接 Heater 的文档除名称外完全一致、树 / 画布 / Inspector 同步、结果旧化、真实临时文件保存重开、sidecar 和 rerun |
| macOS 实窗主路径 | 已验证可用：重开先前由普通空白创建的 Feed–Heater–Flash 项目，空白拒绝 / 放弃、中文改名、命令面板显示及 Undo / Redo、原生另存为、跨进程重开和 rerun 均通过；最终版本中文提示与三字段显示已复核 |
| 其他五类逐项 GUI、Windows / Linux | 待验证；六类自动化矩阵不等于全部实窗覆盖。重命名整体仍属部分平台 / 操作路径已验证 |

### 快捷键与文本焦点复核（2026-09-13）

在 B1-3 提交 `2c15af67` 上完成代码、定向测试与 macOS 实窗复核，复用由普通空白创建的 Feed–Heater–Flash 项目；未修改产品键位或焦点契约。

| 能力 | 状态与证据 |
| --- | --- |
| 文本草稿撤销 / 重做 | 已验证可用：名称与数值输入中 `⌘Z / ⇧⌘Z` 只改变草稿，项目名称 / 工程数值及当前求解快照保持；[路由](../../apps/radishflow-studio/src/studio_gui_shortcut_router.rs) 把历史键交给 TextInput，shell 焦点测试覆盖文本优先级 |
| 项目树 / 画布文档历史 | 已验证可用：两处 `⌘Z / ⌘Y` 撤销 / 重做名称提交，revision 每次加一、连接和位置保留，旧快照不恢复为当前；重开后无会话历史，面板正确禁用 Undo / Redo |
| Enter 与取消 | 已验证可用（现有语义）：有效名称与温度 Enter 提交，`-1 K` Enter 不提交；显式“丢弃”恢复文档值。名称 / 数值 Escape 退出文本焦点但保留草稿，不承诺按 Escape 丢弃编辑；命令面板 Escape 关闭面板 |
| 保存与重开 | 已验证可用：带 `350 K` 草稿另存为，真实文件仍为已提交 `345 K`；随后 Enter 提交 `350 K`、`⌘S` 保存，跨进程重开保留名称、参数、绑定、sidecar，rerun 收敛 |
| 平台键位展示 | 部分可用，已确认显示缺口：macOS 面板显示 `Ctrl+S / Ctrl+Z / Ctrl+Y`，实际使用 `⌘`。输入层把 egui command 映射到 registry Ctrl，展示层直接输出 Ctrl，未区分主修饰键与物理 Ctrl |
| macOS `⇧⌘Z` 文档重做 | 缺失：实窗无动作，registry 仅注册 Ctrl+Y；`⌘Y` 能重做，因此不登记为全部文档重做不可用 |
| 扩展路径 | 待验证：Windows / Linux 原生键盘、IME 候选确认与 Enter 同帧、长按重复事件、跨窗口焦点、对话框组合及其余单元逐项实窗；未测项不登记为缺陷 |

定向运行 `cargo test --locked -p radishflow-studio shortcut`（25 项）与 `focus_context`（4 项），共 29 项通过。未改产品代码，无需重复上一轮 1,142 项全仓基线；详细 revision 与临时文件证据进入周志。

### B1-4 候选：平台快捷键展示与文档重做

具体缺口、范围和验收已收口，尚未实施。下一实现需明确主修饰键语义和兼容键位，避免将物理 Ctrl 的画布快捷键一并替换。

- 范围：在现有 shortcut model / registry / shell 中区分平台主修饰键与物理 Ctrl；macOS 的保存、撤销、重做提示分别使用 `⌘S / ⌘Z / ⇧⌘Z`，保留 `⌘Y` 兼容入口；Windows / Linux 保留现有 Ctrl+S / Ctrl+Z / Ctrl+Y。只调整现有命令的绑定与展示，不引入用户自定义键位、脚本录制或通用事件框架。
- 焦点边界：文本框继续拥有文本 Undo / Redo；文档命令只在非文本且非模态的适用上下文触发。Escape 保留草稿和显式丢弃的行为维持现状。
- 自动化验收：平台绑定 / 标签一致；主修饰键和物理 Ctrl 不混淆，Ctrl+Tab 等原有动作不被重写；主键与兼容键各触发同一命令且只提交一次；文本、面板和对话框内不泄漏文档历史命令。
- 实窗验收：macOS 名称 / 数值文本撤销和文档重做使用同一常用组合，各自保持正确作用域；树和画布历史、旧化、保存重开一致，面板提示与实际键位一致。Windows / Linux 需分别补原生运行证据，未执行时明确保留待验证。

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 现有三类小流程冻结 | 三类路径 focused test、保存 / 重开 / rerun 和结果审阅均通过 |
| M2 | 建模缺口治理 | 覆盖选定案例的建模缺口，输入定位、连接与运行门禁行为有回归证据 |
| M3 | 连接能力评估 | 若要推进更自由的连接，先单独开“流程图连接编辑器”专题 |

## 验收标准

- 三类小流程均可从普通空白项目完成。
- readiness 能定位缺 package、缺项目组分、缺 Feed composition、缺 source stream 和必要单元参数。
- 结构性问题和求解阶段问题由 Run Panel 诊断处理。
- 保存 / 重开后建模输入不丢失。
- Rust Core 不出现 COM / CAPE-OPEN 语义倒灌。

## 验证计划

- focused test：三类小流程 happy path、参数缺失、composition 归一、保存 / 重开 / rerun、诊断定位。
- 仓库级：核心阶段执行 `./scripts/check-repo.sh`。
- 人工 smoke：只有新增用户可见建模入口或 connection 行为时执行。

## 状态记录

- 当前状态：Active
- 最近更新：2026-09-13，B0 后完成 B1-1 删除、B1-2 导航反馈和 B1-3 重命名；实现与分层证据见上文。
- 下一步：按 B1-4 候选收口平台主修饰键、显示和文档重做兼容键位后实施；其他单元逐项 GUI、远距离视口与跨平台新增路径仍待验证。
