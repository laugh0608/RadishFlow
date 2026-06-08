# Studio UI Design Guidelines

更新时间：2026-06-07

## 用途

用途：为 RadishFlow Studio 的首屏、画布、示例管理、面板、按钮、文字和结果审阅建立一致的 UI 设计规范，作为 Studio 工作台重排与实现拆分依据。
读者：负责 Studio shell、Canvas、检查器、Runtime、Result 面板的开发者和设计协作者。
不包含：完整视觉稿、图标资产、主题系统、视觉 token 细则、自由连线编辑器、完整报表系统、商业化多文档工作台或求解 / 物性模型设计。

## 设计定位

RadishFlow Studio 继续走轻量、清晰、工程化的浅色桌面应用风格。当前用户认可的是“干净、直接、带一点现代工具感”的基调，因此后续优化不应把界面改成传统厚重 ribbon 的复制品，也不应转向深色监控大屏或营销型产品页。

参考 Aspen、HYSYS、PRO/II、COFE、DWSIM 等同类软件时，只吸收它们的信息架构、任务分区、状态呈现和流程模拟行业习惯；不复制专有视觉资产、具体图标、颜色组合、面板外观或功能范围。

核心原则：

- 画布是主舞台，其他区域为画布建模、求解运行和结果审阅服务。
- 顶部只放全局身份、主路径命令和运行状态，不堆调试计数、窗口控制或完整命令清单。
- 左右侧栏各有稳定职责，不把对象库、属性编辑、日志、授权、结果和调试信息混在同一列里。
- 结果展示只读消费当前 revision 的最新 `SolveSnapshot`，不新增 Studio shell 私有结果缓存或第二套求解解释；文档编辑后旧快照只用于过期提示，不继续驱动结果审阅、结果表或导出。
- MVP α 阶段优化信息层级和操作路径，不扩自由连线编辑器、完整拖拽布局、自动布线和完整报表。

## 参考图启发

UI 素材当前保存在 `docs/architecture/assets/studio-ui/`。其中 `baseline/` 只放当前 RadishFlow 自身的视觉基线图；同级目录下其他图片均为参考素材或历史对照，不作为实现优先基线。下表使用从本文档出发的相对路径，便于在 Markdown 预览中直接打开。2026-05-23 新增的优秀产品灵感参考已单独整理到 `docs/architecture/ui-inspiration-reference.md`，用于后续 UI 专题评审，不作为当前实现可直接照抄的视觉稿。

| 素材 | 路径 | 用途 |
| --- | --- | --- |
| Studio 视觉系统规范 | [studio-visual-system.md](studio-visual-system.md) | RadishFlow Studio 的视觉定位、token、色彩角色、控件状态和视觉验收口径 |
| 外部优秀产品灵感参考 | [ui-inspiration-reference.md](ui-inspiration-reference.md) | AFFINE、CodexApp、Cloudflare、GitHub、Discourse、1Panel 截图的设计语言提炼，用于后续 UI 专题的视觉、排版和信息密度参考 |
| 当前 RadishFlow UI | [radishflow-current-workbench-20260516.png](assets/studio-ui/radishflow-current-workbench-20260516.png) | 2026-05-16 当前真实 Studio 截图，用于识别首页 / 工作台重排前的混乱分区和测试痛点 |
| RadishFlow 工作台视觉基线 | [radishflow-workbench-concept.png](assets/studio-ui/baseline/radishflow-workbench-concept.png) | 进入项目后的工作台基线图，状态、SI 单位、单文档标题和结果区关系更符合当前规范 |
| RadishFlow Home 概念稿 | [radishflow-home-dashboard-concept.png](assets/studio-ui/radishflow-home-dashboard-concept.png) | 启动首页 / 示例页概念稿，展示 Start actions、Recent Cases、Example Cases、Environment 和 Messages 分区 |
| RadishFlow Home 视觉基线 | [radishflow-home-dashboard-concept-v2-20260516.png](assets/studio-ui/baseline/radishflow-home-dashboard-concept-v2-20260516.png) | 当前 Home Dashboard 视觉基线，收窄 Environment 字段，突出 Continue Last Case / 示例入口和可行动 Messages |
| 外部参考：深色工程工作台 | [reference-dark-engineering-workbench.png](assets/studio-ui/reference-dark-engineering-workbench.png) | 深色工程工作台分区、运行状态和底部结果区参考 |
| 外部参考：浅色 ribbon 项目状态 | [reference-light-ribbon-project-status.png](assets/studio-ui/reference-light-ribbon-project-status.png) | 浅色 ribbon / 左侧模型树 / 右侧状态面板参考 |
| 外部参考：浅色 flowsheet 工作台 | [reference-light-flowsheet-workbench.png](assets/studio-ui/reference-light-flowsheet-workbench.png) | 接近 RadishFlow 方向的浅色流程模拟工作台参考 |
| 外部参考：Aspen flowsheet / 模型面板 | [reference-aspen-flowsheet-model-palette.png](assets/studio-ui/reference-aspen-flowsheet-model-palette.png) | Aspen 类 flowsheet、左侧模拟树和底部模型选项板参考 |
| 外部参考：Aspen 运行控制 | [reference-aspen-run-control-status.png](assets/studio-ui/reference-aspen-run-control-status.png) | Aspen 类控制面板、运行消息和收敛状态参考 |
| 外部参考：HYSYS flowsheet / 对象树 | [reference-hysys-flowsheet-object-tree.png](assets/studio-ui/reference-hysys-flowsheet-object-tree.png) | HYSYS 类流程图、左侧对象树、底部消息区参考 |
| 外部参考：PRO/II 命令 / 消息 / 对象库 | [reference-proii-command-message-object-palette.png](assets/studio-ui/reference-proii-command-message-object-palette.png) | PRO/II 类主命令、消息栏和右侧对象库参考 |
| 外部参考：COFE 轻量工具栏 / 点阵画布 | [reference-cofe-light-toolbar-grid-canvas.png](assets/studio-ui/reference-cofe-light-toolbar-grid-canvas.png) | COFE 类轻量工具栏、点阵画布和底部日志参考 |
| 外部参考：DWSIM 属性 / 对象库工作台 | [reference-dwsim-property-palette-workbench.png](assets/studio-ui/reference-dwsim-property-palette-workbench.png) | DWSIM 类左右双栏、对象属性和 palette 分类参考 |

### 当前 RadishFlow

当前 UI 的优势是浅色、克制、状态 chip 清晰，顶部导航已先收敛为 `文件 / 主页 / 物性 / 流程图 / 运行 / 结果 / 工具 / 设置`，打开 / 新建 / 保存 / 示例、运行、结果和命令面板不再作为同一排快速按钮平铺。问题仍集中在整体排版和分区：画布工具、suggestion、对象列表、运行面板、授权和调试信息还需要继续向成熟建模工作台结构收敛，避免用户难以判断“现在该看哪里、下一步点哪里、结果在哪里”。

后续应保留当前风格基调，但把功能分区从“开发态信息平铺”重排为“建模工作台”。

`radishflow-current-workbench-20260516.png` 是当前真实 UI 备份，用于对照识别首屏信息过载、按钮堆叠、Canvas header 混乱、结果 / 诊断入口分散等问题。`baseline/radishflow-workbench-concept.png` 当前作为真实 UI 重排的视觉基线：保留轻量浅色桌面应用气质、单文档项目标题、顶部主导航与上下文工具栏、中央 flowsheet canvas、左侧 Project / Palette、右侧检查器 tabs、底部 Results Table / Messages 区和 SI 单位展示。后续实现时仍需把它转译为现有 `egui` 组件和 `SolveSnapshot` / window model 边界，不直接把图中所有视觉细节视为代码契约。

### Aspen / HYSYS 类界面

可借鉴点：

- 顶部 ribbon / command bar 把文件、运行、结果、工具按任务分组。
- 左侧项目树和底部模型面板让用户能稳定找到物性、流股、模块、结果和数据表。
- 中央 flowsheet 保持大面积空白，设备和流股标签直接表达流程语义。
- 顶部或底部有统一运行状态、收敛状态、消息和缩放信息。

不直接采用点：

- 完整 ribbon 太重，MVP α 不需要把所有未来功能提前摆到首屏。
- 大量浅蓝渐变和传统 Office 时代控件会削弱 RadishFlow 当前更现代的轻量感。

### PRO/II 类界面

可借鉴点：

- Run、Status、Unconverged Streams and Units、Property Table 这类命令的优先级很明确。
- 左侧消息面板能让错误持续可见，不被弹窗打断。
- 右侧垂直对象库适合流程图放置，但分类必须紧凑、可折叠。

不直接采用点：

- 顶部大按钮过密，且大量边框强调会让 MVP 首屏显得粗糙。
- 右侧竖排标签可发现性弱，RadishFlow 应优先使用可读的 tab 或分段控件。

### COFE 类界面

可借鉴点：

- 工具栏很轻，画布占比高，底部日志保持常驻。
- 点阵画布适合早期建模和对齐。

不直接采用点：

- 图标过小、文字层级弱，不适合新用户第一次上手。
- 日志背景和错误颜色过重，RadishFlow 应把错误摘要、可操作修复和原始日志分开。

### DWSIM 类界面

可借鉴点：

- 左侧当前对象属性、右侧 flowsheet objects palette、中间画布的三栏结构适合流程模拟。
- 属性面板按 `General / Specifications / Connections / Estimates` 等 tab 组织，字段、单位和状态并排。
- 右侧 palette 按 Streams、Pressure Changers、Separators、Mixers、Exchangers 等领域分类，符合流程建模心智。

不直接采用点：

- 顶部工具区和第二层 tab 较多，MVP α 应先保持 compact command bar。
- 画布内说明文字不应常驻占据主视野，示例说明应进入 guide 或可折叠 note。

## 当前优先级

当前 UI 设计是首版 demo 前的第一优先级。原因不是视觉精修，而是现有界面还没有让用户和测试人员稳定理解：

- App 打开后当前处于什么项目状态。
- 示例项目在哪里管理，应该从哪里打开。
- 建模对象、连接建议、运行、结果和错误分别属于哪个区域。
- 人工 smoke 时应该按什么路径操作和观察结果。

2026-05-16 已完成 Home Dashboard 与 Workbench 第一轮真实 UI 收口。2026-05-17 已继续完成 Canvas viewport 初始居中 / fit-to-content、首页高频中文文案、Home 打开路径、Canvas 可读性、运行后结果视图和 Workbench 高频残余文案收口。2026-05-25 已进入 MVP β 小案例作者体验：Home 可从空白项目进入 `Mixer-Flash` 或 `Heater-Flash` 作者路径，Workbench `模块` 面板显示只读任务清单。后续不再回到“先讨论首页分区”或继续扩画布建模能力的阶段；下一轮应优先做可复现作者路径和真实窗口 smoke，只修阻碍主路径的问题。

2026-05-17 日终截图复核后保留的两项 UI blocker 已在 2026-05-18 收口：

- Canvas 中间流股标签在 `Feed -> Valve -> Flash Drum` 这类短线段空间不足时隐藏，避免被单元块或端口挤压、遮挡或裁切；这仍不是自动布线、自由连线或完整拖拽布局。
- 右侧结果检查器的组成 / 相态摘要不再使用 `z: ...`、`phases: ...` 原始前缀，默认正文改为结构化短行，并继续只读消费 `SolveSnapshot`；不新增 shell-local 结果缓存或完整报表系统。

下一轮优先继续人工视觉 smoke。除非是修复阻塞性错误，不应继续在现有布局上叠加零散按钮、临时说明、调试状态或一次性 smoke 面板。

## 后续 UI 专题与设计稿资产

当前文档记录的是 MVP α / β 阶段已经形成的 Studio UI 规范、真实 UI 收口经验和下一轮主设计稿约束，不等同于完整产品 UI 设计系统。长期视觉系统、token、色彩角色、控件状态和视觉验收口径见 `studio-visual-system.md`。Studio UI 专题阶段的端点边界、主工作流、状态模型和 `.pen` 设计稿规则见 `studio-ui-topic-plan.md`；当前唯一活跃 Studio 主设计稿见 `docs/architecture/designs/studio-client-main.pen`，文字 brief 见 `docs/architecture/designs/studio-client-main-brief.md`。本文继续承担设计规范职责，不承载阶段推进流水。

当前已冻结的 P0 设计稿口径：

- `studio-client-main.pen` 是当前唯一活跃 Studio 主设计稿，包含 `Home - Ready`、`Property - Components and Methods`、`Flowsheet - Modeling`、`Module - Settings and Results` 四个 frame。
- `Property - Components and Methods` 是顶部导航下的独立物性页，不是左侧或右侧栏 tab。
- `unit-module-panel.pen` 不再作为完整 Workbench 复制稿维护；若模块设置 / 结果细节不足，再创建窄口径 `module-settings-panel.pen`。
- 服务端 / 控制面 UI 和移动端 / 只读视图仍是 P2 边界，不进入当前主稿细化。

设计流程应采用设计稿先行：

- 优先使用 `pencil` 工具绘制 `.pen` 设计稿，再根据评审后的设计稿开发。
- 设计稿必须随仓库保存和同步，不能只留在个人本地或聊天记录里。
- 专题启动时应同时读取 `docs/architecture/ui-inspiration-reference.md`，明确每个设计稿借鉴了哪些设计原则，以及哪些视觉元素不得照抄。
- 原则上每个明确端点或界面域维护一个独立 `*.pen` 文件，避免把客户端、服务端管理和移动端视图混在同一个设计稿里；但模块设置这类 Workbench 内部结构应先在主稿中统一，不为每个单元复制整套 Workbench。
- `.pen` 文件目录固定为 `docs/architecture/designs/`；`.pen` 文件只通过 Pencil MCP 读取、生成、验证和导出。
- 每次按设计稿实现 UI 时，应同步记录该实现对应的 `.pen` 文件、评审状态和偏离设计稿的原因。

## 信息架构

Studio UI 设计需要同时区分“当前已实现壳”和“主设计稿目标”。当前代码已以 Home Dashboard、八入口顶部导航、四类上下文工具栏、左侧 `模块 / 项目`、独立 `物性` 页、右侧 `检查器 / 模块设置 / 模块结果`、底部 `消息 / 运行日志 / 收敛 / 建议 / 诊断 / 结果表` 和状态汇总为基础；主设计稿目标仍以 `studio-client-main.pen` 为准，继续统一 Home、独立物性页、Flowsheet 工作台和模块设置 / 结果四个主 frame。

主设计稿的 Workbench 建议分为七个稳定区域：

| 区域 | 设计目标 |
| --- | --- |
| Window Title | 项目名称、窗口标题和平台窗口控制；不重复展示保存状态 |
| Top Navigation | `文件 / 主页 / 物性 / 流程图 / 运行 / 结果 / 工具 / 设置` |
| Context Toolbar | 随当前导航显示保存、检查、运行、画布、结果或物性相关动作 |
| Left Rail | `模块 / 项目`；模块页放流股和按分类折叠的单元操作，项目页放当前对象树 |
| Canvas Stage | 类 IDE 的 flowsheet 标签页、流程图画布和左上角浮动画布工具条 |
| Right Rail | `检查器 / 模块设置 / 模块结果`；不再放独立 `运行` 或 `物性` tab |
| Bottom Area | 左侧运行信息 tabs，右侧当前案例状态汇总 |

### Home Dashboard / Start Page

职责：

- 作为 App 启动后的默认首页，解决“从哪里开始”和“当前环境是否可用”两个问题。
- 展示最近打开的项目、内置示例项目、登录 / 授权状态、客户端信息、服务端信息和当前设备 / 本地环境摘要。
- 在用户打开已有项目、新建空白项目或打开示例后，再进入流程图工作台。

规则：

- Home Dashboard 不是营销欢迎页，也不是完整控制台；它是工程软件启动面板。
- Home Dashboard 默认使用中文界面；工程术语、包名、版本、路径和文件扩展名保留原文，用户动作、状态、消息和环境字段使用中文。
- 页面应保留 `baseline/radishflow-workbench-concept.png` 的轻量浅色风格、克制蓝色主强调、状态 chip 和清晰分区，但不显示流程图画布。
- `baseline/radishflow-home-dashboard-concept-v2-20260516.png` 当前作为启动首页视觉基线：它比早期概念稿更接近当前信息架构，Start actions、Recent Cases、Example Cases、Environment 和 Messages 的职责边界更清楚。
- `radishflow-home-dashboard-concept.png` 保留为早期概念稿参考：它的信息架构方向正确，但字段和示例数据偏概念演示，不作为后续实现的优先基线。
- Start actions 只保留当前主路径：`新建项目`、当前小案例作者入口、`打开项目`、`打开示例项目`；登录放在顶部 App Bar，不把完整命令面板或调试入口放进第一视野。最近项目的继续入口由 `最近项目` tile 承载，不再作为左侧重复按钮常驻。当前评审结论是 Home 信息密度和工作台一致性仍不足，后续应优先对照 Home v2 基线做 focused 修正，而不是继续堆低密度欢迎文案。
- 小案例作者入口只负责创建空白项目并打开对应任务清单，不自动生成 flowsheet，不写 `FlowsheetDocument`，不进入 undo，也不替代 placement / suggestion / parameter / run / save / export 工作流。
- 最近项目和示例项目必须可扫读：优先使用流程缩影 tile gallery，而不是纯文本列表或通用文件图标。每个 tile 包含浅色 flowsheet thumbnail、名称、路径或来源、最后打开时间、流程摘要、组分 / 物性包摘要和状态标签；单击选择，双击打开。
- 客户端 / 服务端 / 设备信息默认以状态卡或紧凑 section 呈现；详细路径、backend、cache 细节和诊断信息进入展开项。
- 登录入口应优先是 `登录` 按钮，而不是内嵌账号密码表单；桌面登录继续遵守 OIDC Authorization Code + PKCE + 系统浏览器 + loopback redirect 的边界。
- 未登录、服务端不可用、物性包缓存缺失、示例目录缺失等问题应显示为可行动状态，不使用开发态错误文本。

建议首屏布局：

- 顶部 App Bar：`RadishFlow Studio`、版本 / build commit、登录状态、服务端状态、语言 / 设置入口。
- 左侧 Start Actions：新建项目、`Mixer-Flash` / `Heater-Flash` 小案例作者入口、打开项目、打开示例项目。
- 中央内容：最近打开的项目列表和示例项目列表；首版 demo 前默认只暴露 official hydrocarbon 演示路径，即 `Feed -> Heater/Cooler/Valve -> Flash Drum` 与 `Feed + Feed -> Mixer -> Flash Drum`。Synthetic / PME 验证样例可保留为文件或专题入口，但不进入首页高频示例列表。
- 右侧 Environment Status：客户端信息、服务端信息、设备信息、本地缓存 / 示例路径状态。
- 底部 Messages：最近环境警告、登录 / 授权提示、示例路径或物性包缓存诊断。

实现契约：

| 区域 | 第一屏保留字段 | 折叠或二级字段 |
| --- | --- | --- |
| Top App Bar | 应用名、`development build` 或当前人工确认的版本标识、`Local ready`、`Server offline`、`Signed out`、单位集、登录 / 设置 / 帮助入口 | 完整 build commit、完整控制面 URL、语言高级设置、开发诊断 |
| Start Actions | `新建项目`、`创建 Mixer-Flash 小案例`、`创建 Heater-Flash 小案例`、`打开项目`、`打开示例项目`；无最近项目时 `打开示例项目` 或小案例作者入口为优先入口 | 命令面板、最近工作区完整列表、保存 / 另存为、运行按钮 |
| Recent Cases | 浅色流程缩影、项目名称、路径或来源、最后打开时间、物性包、状态；选中态和双击打开行为 | 流股数、单元数、诊断数、最新求解摘要、完整路径展开 |
| Example Cases | 浅色流程缩影、示例类型、短流程图摘要、组件摘要、物性包、状态；选中态和双击打开行为 | 长说明、教程步骤、完整 flowsheet 预览、PME 操作说明 |
| Environment | `Client`、`Server`、`Device` 三组健康摘要；只显示影响“能否开始”的状态 | cache 根目录、examples 绝对路径、backend 细节、原始错误文本、设备资源曲线 |
| Messages | 最近 3-5 条可行动消息、严重度、领域标签、短动作入口 | 原始日志、GUI activity、platform timer、host internals、完整 trace |

状态枚举：

- Recent Cases：`Ready`、`Modified`、`Missing file`、`Missing package`、`Version warning`、`Error`。
- Example Cases：`Ready`、`Requires package`、`Requires external PME`、`Missing file`、`Error`。
- Client：`Ready`、`Portable`、`Dev`、`Examples missing`、`Error`。
- Server：`Signed out`、`Signed in`、`Reachable`、`Offline`、`Local only`、`Sync needed`。
- Device：`Runtime ready`、`Cache ready`、`Cache missing`、`Runtime error`。
- Messages：`AUTH`、`EXAMPLES`、`CACHE`、`PACKAGE`、`SERVER`、`RUNTIME`。

数据来源和所有权：

- Recent Cases 来自应用级偏好或 MRU 索引，不写入 `FlowsheetDocument`；读取文件元数据失败时只降级该行状态，不阻断首页。
- Example Cases 来自内置 `examples/flowsheets` 发现结果；打包后优先从 exe 同目录发现，开发态再使用仓库路径。
- Environment 的 Client 摘要来自应用版本、包模式和示例目录探测；Server 摘要来自 `AuthSessionState` / `EntitlementState` 和控制面可达性；Device 摘要来自本地缓存、runtime/native 依赖和操作系统摘要。
- Messages 消费应用级事件和环境探测摘要，不反向成为环境、授权、文档或结果的真相源。
- Home Dashboard 不读取或解释 `SolveSnapshot`；打开项目后的结果审阅仍只在工作台中消费 `SolveSnapshot`。

动作契约：

- `New Project` / `新建项目` 创建 MVP 默认空白项目，进入工作台；该动作不依赖登录或服务端。
- 小案例作者入口创建 MVP 默认空白项目，进入工作台并切到左侧 `模块`；当前支持 `Feed + Feed -> Mixer -> Flash Drum` 与 `Feed -> Heater -> Flash Drum` 两条作者清单。清单状态必须从当前 canvas unit / stream / solve snapshot 推导，不反向修改项目语义。
- `Open Project` / `打开项目` 使用系统文件选择器打开用户项目，进入工作台；打开成功后更新 MRU。
- `Open Example Project` / `打开示例项目` 从 Example Cases 选择或打开示例文件夹，成功后进入工作台并更新 MRU 来源。
- `Recent Cases` 和 `Example Cases` 的行点击只改变选择态，整行双击触发打开；文件缺失时不静默失败，应把该项目行标为 `Missing file` 并产生 `Messages` 行。
- 若当前工作区存在未保存变更，`新建项目`、小案例作者入口、`打开项目`、`打开示例项目` 和列表双击打开都必须进入显式继续 / 取消确认，不得只显示无法行动的 discard 提示。
- `Sign in` 只启动 OIDC / PKCE 系统浏览器登录；不在首页内嵌账号密码表单。
- `Messages` 中的 `Sign in`、`Open Examples Folder`、`Open Cache Folder` 等动作应复用正式 command surface；若实现阶段暂需过渡，也不能长期保留和菜单 / 命令面板平行的私有分支。

空状态和错误状态：

- 无最近项目：Start Actions 中不显示失效的主按钮，改由 `Open Example Project` 成为主入口；Recent Cases 区域显示简短空状态和 `Open Project` / `Open Example Project` 动作。
- 示例目录缺失：Example Cases 显示 `Examples missing`，Messages 给出路径和可行动入口；不要用 panic、debug path 或原始 IO error 占据第一屏。
- 未登录或服务端离线：只影响云端包、团队项目和授权同步；若本地示例与本地缓存可用，应明确展示 `Local ready`，避免用户误以为不能开始。
- 物性包缺失：Recent / Example 对应行显示 `Missing package` 或 `Requires package`；打开项目后再进入工作台处理详细修复。
- 本地 runtime 失败：Environment 标为 `Runtime error`，Messages 给出最短诊断入口；不在首页展开完整日志。

进入工作台的规则：

- 当前代码打开项目或示例后进入常规工作台：顶部 `文件 / 主页 / 物性 / 流程图 / 运行 / 结果 / 工具 / 设置` 主导航、当前 screen 下的上下文工具栏、左侧示例 / 项目 / 放置、中央 Flowsheet Canvas、右侧 `检查器 / 模块设置 / 模块结果`、底部 `消息 / 运行日志 / 收敛 / 建议 / 诊断 / 结果表`。
- 文件 / 示例入口进入 `文件` 或 Home，运行控制进入 `流程图` / `运行` 上下文工具栏，物性进入独立 `物性` 页面，结果审阅进入顶部 `结果` screen、右侧 `模块结果` 和底部 `结果表`。
- 工作台内仍可通过顶部或左侧返回 Home Dashboard，但 Home Dashboard 不直接承载流程图编辑。
- 若启动时发现最近项目，可以先进入 Home Dashboard 并突出 `最近项目` 列表的首项，不要静默跳过首页，也不要在左侧重复生成继续入口按钮。

### 顶部 App Bar

当前已实现职责：

- 应用和项目身份：项目名、脏状态、运行模式、当前单位集。
- 顶部主导航：`文件 / 主页 / 物性 / 流程图 / 运行 / 结果 / 工具 / 设置`。
- 上下文工具栏：`物性 / 流程图 / 运行 / 结果` screen 分别消费已有 property page、command registry、Run Panel state、Module Results、结果表状态和 `SolveSnapshot` 状态。
- 全局菜单：项目生命周期命令进入 `文件`；命令面板、Commands 面板和逻辑窗口进入 `工具`；语言进入 `设置`。
- 全局状态：运行状态、pending work、授权 / 物性包摘要、最近错误入口。

规则：

- 顶部默认不显示完整项目路径；路径放入项目详情或 tooltip。
- 运行按钮必须在顶部保持可见，禁用时给出短原因。
- 低频视图切换、逻辑窗口和调试窗口进入 `工具` 或命令面板；语言切换进入 `设置`。
- 状态 chip 数量控制在 3-5 个，避免把每个内部计数都做成 chip。

后续细化目标：

- 继续收紧上下文工具栏密度，不把 `打开项目`、`打开示例` 常驻为工作区第一按钮。
- 项目名称放在窗口标题或项目摘要中；保存状态使用右侧状态 chip，不在标题处重复加 `*` 或长状态说明。
- `设备` 不作为一级导航；模块放置和模块设置分别归入左侧 `模块` 与右侧 `模块设置`。

### 左侧 Navigator / Examples / Palette

当前已实现职责：

- 示例管理：内置示例列表、最近项目、打开示例入口和示例说明摘要。
- 当前项目树：Project components、Property Package、Streams、Units、Results、Diagnostics。
- 建模对象库：Feed、Mixer、Heater/Cooler、Valve、Flash Drum 等 MVP 对象。
- 搜索和分类过滤。

规则：

- 左侧默认宽度建议 240-280 px。
- 对象库按钮应有领域图标和短标签，不使用长句说明。
- suggestion 是辅助建模入口，不和对象库按钮混排成一列命令。
- 示例、项目树与对象库可以用 tab 或分段控件切换，避免同时展开造成拥挤。
- 打开示例不应只依赖顶部按钮；左侧必须有稳定、可扫读的示例管理入口。
- 项目级输入不应只藏在单一面板。当前左侧 `项目` 面板必须能扫读受控项目组分和当前物性包，独立 `物性` 页提供正式编辑入口，且组分选择 / 移除入口应直接作用于 `Flowsheet.components`，不引入 shell 私有组分状态。

后续细化目标：

- 左侧栏收敛为 `模块 / 项目` 两个 tab。
- `模块` 替代旧 `放置` 命名，顶部提供搜索；物料流、能量流、信号流放在单元操作前面，单元操作按分类折叠。
- `项目` 只负责当前项目对象树、结果对象和诊断对象导航；物性主配置移到独立 `物性` 页面。

### 中央 Flowsheet Canvas

职责：

- 展示和操作流程图。
- 呈现设备、流股、连接关系、选择状态、错误状态和关键结果标签。

规则：

- 画布默认占窗口最大面积，左右侧栏和底部面板不得压缩到只剩小预览。
- 打开示例或项目后，Canvas viewport 应根据当前单元与流股 bounds 做初始 fit-to-content / center；小流程不应固定在左上角。用户拖动空白画布后的 viewport offset 可写入 layout sidecar，但仍只属于呈现状态，不引入自动布线、自由连线或完整视图持久化。
- 画布工具条应以图标或短标签表达选择、放置、suggestion 接受 / 拒绝、平移、缩放、适配视图和受控恢复动作；不要把当前 MVP 误设计成自由连线工具条。
- 本地建模 suggestion 的接受动作应使用明确的 `连接流股` / `Connect stream` 或 `创建流股` / `Create stream`，不用泛化的 `Apply` 让用户猜测会改写什么。
- 选中 material stream 后可暴露 `Disconnect stream`、`Disconnect source`、`Disconnect sink`、`Reconnect stream` 与 `Delete stream`；`Reconnect stream` 只允许把单端缺口接到唯一、未占用且不会形成 unit dependency cycle 的端点。文案必须体现这些动作属于受控恢复，不是任意端口重连、自动布线或完整拖拽布局编辑。
- 长说明、状态解释和开发态计数不直接堆在画布上方；进入 legend、tooltip 或底部消息。
- 流股标签优先显示名称；短线段空间不足时应避开单元块和端口，或自动短化 / 隐藏，不让标签被裁切或遮挡。求解后可在缩放足够时显示关键 `T / P / F / H` 摘要。
- 设备图形保持简化 process symbol 风格：清晰、平面、少装饰，可区分类型，不追求拟物渲染。
- 错误、未配置、未收敛、未授权等状态用统一 badge 和 outline 表达，不用整块高饱和背景。

### 右侧检查器

当前已实现职责：

- 当前选择对象的属性编辑。
- 当前单元的 Module Settings。
- 当前单元的 Module Results。
- 当前对象、端口、诊断和结果定位动作。

规则：

- 右侧默认只展示和当前选择或当前任务相关的信息。
- 当前代码以 `检查器 / 模块设置 / 模块结果` tab 组织右侧栏；`运行` 和 `物性包` 不再作为右侧常驻 tab。
- `物性` 页承载本地 package 摘要、内置 package 选择和同一套项目组分选择入口；用户刚进入 Workbench 时仍应能先从左侧项目树发现项目组分状态。
- 属性字段采用 label + input + unit + validation 的行结构；单位必须紧贴数值，不藏在说明文字里。
- 从左侧 Project、Canvas 对象列表或结果定位动作选择 stream / unit 后，应自然切换到对应检查器；stream 优先暴露 `T / P / F`、组成草稿和提交/归一化动作，unit 的可编辑参数进入 `模块设置`，latest unit result 进入 `模块结果`。
- Unit Inspector 当前只把 `Feed` 的 source temperature / pressure、`Heater / Cooler` 的 outlet temperature / outlet pressure、`Mixer / Valve` 的 outlet pressure 与 `Flash Drum` 的 flash temperature / pressure 作为可编辑参数行；字段必须显示 SI 单位和约束提示，提交走正式文档命令并同步对应 outlet stream 模板。若字段值来自 outlet stream 模板 / fallback 而 unit parameter 尚未显式存在，同值提交仍应写入正式 `SetUnitParameter`。其余单元信息仍以端口、关联步骤、关联诊断和最新只读结果为主，不提前设计完整单元参数表。
- 结果检查器中面向用户的组成、相态和摘要行应优先使用本地化结构化短句；`z: ...`、`phases: ...` 这类原始调试文本只应进入 hover、日志或开发诊断，不应作为默认结果正文。
- 模块结果、环境摘要和状态摘要应使用状态 chip、metric card、stream chip 和短说明组合；避免把 `状态 / Duty / Outlet T / Diagnostics` 或 `客户端 / 服务端 / 缓存` 做成松散的两列文字直排。
- 草稿态、未归一组成、运行阻断和只读结果要有稳定视觉语义。
- Runtime 中的开发态活动、平台 timer、GUI activity、原始项目路径编辑默认折叠。

后续细化目标：

- 继续细化 `检查器 / 模块设置 / 模块结果` 的密度、空状态、诊断目标和跨入口定位反馈。
- 运行主按钮、Hold / Resume、检查输入和清理诊断保留在顶部 `运行` 上下文工具栏，详细日志进入底部运行信息区。
- 项目物性、组分、方法、交互参数、来源和分析保留在独立 `物性` 页面。
- 左侧模块库不做松散文本列表；物料流、能量流、信号流使用 palette item，单元操作分类使用 category card，选中态、计数和简短描述保持稳定位置。
- 右侧检查器不做松散 label/value 文本堆叠；当前对象用 summary card，关键字段用 metric card，端口用 connection card，状态用 chip。
- Home、Property、Flowsheet、Module / Results 四个主 frame 必须共享同一套浅色 shell 和卡片密度；Property 页面也应使用导航卡、组分卡、metric card 和 summary card，不保留旧式大表格 / 长段落正文作为主视觉。

### 底部 Workbench Drawer

职责：

- Messages
- Run Log
- Results Table
- Convergence / Diagnostics

规则：

- 默认 Messages 可更紧凑；无错误、无运行日志堆积时建议约 130-160 px 保持可行动摘要。Results / Diagnostics 可按内容需要更高，后续再评估用户可手动折叠 / resize。
- Messages 放用户可行动摘要，Run Log 放较原始的运行过程。
- 结果表格按 stream-centric / unit-centric 组织，保持和当前 `SolveSnapshot` 语义一致；当前快照复制 / 导出可包含 `Streams / Review / Units / Steps / Diagnostics` 轻量文本区，但不应设计成完整报表、模板或批量导出系统。若文档已编辑且结果过期，结果表应显示重新运行提示，而不是继续展示旧表格。
- 底部面板不应默认展示整屏原始日志；原始日志作为展开详情或复制入口。

后续细化目标：

- 底部继续从单一 drawer 向运行信息区 + 状态汇总的清晰分栏细化。
- 运行信息 tabs 保持 `消息`、`运行日志`、`收敛`、`建议`、`诊断`、`结果表` 的职责边界。
- 右侧：`状态汇总`，显示当前案例状态、最新运行、收敛 / 迭代信息、诊断数和 snapshot / revision 一致性；没有真实迭代次数时显示 `N/A` 或 `Sequential steps`，不得伪造收敛数据。

### 画布浮动工具条

规则：

- 浮动工具条必须服从浅色工作台基调，默认使用白色或极浅灰底、浅描边和深灰图标。
- 当前工具使用浅蓝选中底和蓝色图标即可，不使用黑色高对比胶囊形工具条。
- 工具条只放高频画布工具：选择、框选、平移、放大、缩小、适应流程、网格；低频命令进入顶部上下文工具栏或菜单。

### 底部 Status Bar

职责：

- 当前单位集、solver 状态、收敛摘要、缩放比例、选择数量、后台任务状态。

规则：

- 状态栏只放短文本或图标，不承载操作主路径。
- 错误入口可以常驻，但点击后跳转到底部 Messages 或右侧检查器。

## 功能分区规范

### 主命令

主命令按任务组织，而不是按实现模块平铺。

建议分组：

- Project：新建空白、打开示例、打开项目、保存、另存为。
- Build：选择、放置对象、接受 / 拒绝建议、受控断开 / 删除流股、撤销、重做。
- Run：运行、暂停 / hold、恢复、检查状态、清除运行消息。
- Results：查看流股结果、查看单元结果、结果表、诊断。
- View：缩放、适配视图、面板显示、语言。
- Tools：命令面板、开发诊断、导出调试信息。

当前默认只把 Project / Run / Results 的核心动作摆到第一视野，Build 和 Tools 进入画布工具条、侧栏、上下文工具栏或命令面板。

### 文案

规则：

- 按钮使用动词或名词短语，例如 `打开示例`、`运行`、`保存`、`结果`。
- 状态使用稳定名词，例如 `Idle`、`Running`、`Hold`、`Snapshot missing`。
- 不在默认界面展示调试式文案，例如 `activity=1`、`entitlement=attached`、`logs=0`。
- 说明文字只用于帮助用户做决定；内部实现状态进入 tooltip、日志或开发诊断。

### 按钮

规则：

- 高频命令使用图标 + 文字；重复工具使用图标按钮并提供 tooltip。
- 同一行按钮数量超过 5 个时，应分组、折叠或改为菜单。
- 禁用按钮必须保留可发现性，并在 tooltip 或相邻说明中给出原因。
- 灰色 pill 只用于次级 chip 或状态，不作为所有按钮的统一形态。

### 面板

规则：

- 面板标题短而稳定，例如 `Canvas`、`检查器`、`结果`、`消息`。
- 面板内部用 section 分组，不做卡片套卡片。
- 可折叠 section 默认只折叠低频和开发态内容。
- 面板宽度不足时优先换行和收起说明，不让文字挤出或覆盖控件。

## 视觉语言

本节只保留 Studio UI 信息架构层面的视觉底线，避免后续页面重排偏离当前浅色工程工具定位。更完整的视觉系统、设计 token、控件状态、Canvas 图元和 i18n 长度规则见 `studio-visual-system.md`。

### 色彩

RadishFlow 默认浅色中性底，搭配少量语义色。

建议：

- 背景：白色 / 极浅灰。
- 主强调：克制蓝色，用于当前选择、主按钮和链接。
- 成功 / 可运行：绿色。
- 等待 / hold / warning：琥珀色。
- 错误 / 阻断：红色。
- 未配置 / 草稿 / 未保存：中性灰或淡蓝灰。

限制：

- 不做大面积单一蓝色、紫色、棕色或深色主题。
- 不用渐变球、装饰 blob、营销式 hero 背景。
- 状态色只表达状态，不做装饰。

### 字体层级

建议：

- App / 项目标题：18-22 px。
- 面板标题：13-15 px。
- 正文和字段：12-14 px。
- chip / 状态 / 表格辅助文字：11-12 px。
- 画布标签：随缩放裁剪或隐藏，不随 viewport 宽度动态放大。

规则：

- 不使用 hero 级大字。
- 字符间距保持 0。
- 紧凑面板内标题必须小于首屏标题。

### 间距和尺寸

建议：

- 基础间距采用 4 / 8 / 12 / 16 px 阶梯。
- 面板内边距 12-16 px。
- 工具按钮最小点击区 28-32 px。
- 卡片和面板圆角不超过 8 px。
- 常驻侧栏默认宽度不低于 220 px。

规则：

- 固定格式元素使用稳定尺寸或 min/max 约束，避免 hover、长标签或状态变化导致布局跳动。
- 按钮文字超长时优先缩短文案或折叠进菜单，不压缩成不可读控件。

## MVP α 重排顺序

后续实现应按以下顺序拆分，避免一次性重写 UI：

1. 已落地：首屏 Home Dashboard 分区、顶部 App Bar、Start actions、Recent / Example / Environment / Messages。
2. 已落地：Workbench 顶部八入口主导航、`物性 / 流程图 / 运行 / 结果` 上下文工具栏、左侧 Project / 示例入口、右侧 `检查器 / 模块设置 / 模块结果`、底部 `消息 / 运行日志 / 收敛 / 建议 / 诊断 / 结果表` drawer。
3. 已落地：Canvas header / toolbar 第一轮压缩，Place / suggestion / selection / legend 不再以开发态长文本平铺。
4. 已落地：Canvas viewport 初始 fit-to-content / center，让打开示例后的流程自然处于可视区域中央；该行为只属于 shell-local 呈现，不写项目、不进历史、不做视口持久化。
5. 已落地：补齐首页高频中文文案和打开项目消息，避免 `打开 Case`、`Project opened` 等残留混入中文界面。
6. 已落地：画布对象视觉升级，MVP 单元块、端口、流线标签、运行 / 诊断 badge 和终端标签已有第一轮真实 UI；短线段中间流股空间不足时隐藏标签，避免遮挡 / 裁切。
7. 已落地：结果审阅表格化，右侧结果检查器 selector、底部结果表和默认组成 / 相态短摘要已完成第一轮收口；默认正文不再显示原始 `z:` / `phases:` 摘要，不扩完整报表。

## 验收检查

每轮 UI 改动完成后，至少按下面问题做自检：

- 启动后 5 秒内能否看懂如何打开示例、新建空白、运行、查看结果、保存 / 另存为？
- 画布是否仍是首屏最大区域？
- 顶部是否只保留全局身份、八个主导航入口、当前 screen 上下文工具栏和关键状态？
- 左侧是否只承担项目导航或对象库，而不是命令垃圾桶？
- 右侧是否只展示当前选择、运行或结果的相关信息？
- 原始日志和开发态活动是否没有压过用户主路径？
- `Snapshot missing`、pending、draft、unnormalized、error 是否有统一且可解释的视觉语义？
- 禁用运行或保存时，用户是否能知道原因？
- 选择流股或单元后，是否能自然进入检查器并看到可编辑参数、组成和端口关联？
- 结果入口是否直接来自 `SolveSnapshot`，没有新增 shell-local 结果真相源？
- 文档编辑后旧结果是否被明确标记为过期，并且不会继续驱动结果表、Result Inspector、Results commands 或导出？
- 改动是否仍遵守 MVP α 非目标：不扩自由连线、完整拖拽布局、自动布线和完整报表？
