# Studio Client Main Design Brief

更新时间：2026-06-09

## 用途

用途：为 `docs/architecture/designs/studio-client-main.pen` 提供 Studio 主界面设计输入和评审索引，统一 Home、独立物性页、流程图工作台、模块设置 / 模块结果、运行信息和状态汇总。
读者：准备绘制、评审或实现 RadishFlow Studio 主界面的设计协作者和开发者。
不包含：`.pen` 文件本体、视觉 token、实现代码、完整控件规格、完整物性数据库设计或完整交互动画。

本 brief 对应 `docs/architecture/studio-ui-topic-plan.md` 中的 P0 Studio 客户端本体端点。`.pen` 文件必须继续通过 Pencil MCP 工具创建、维护和验证；本文件只记录设计口径、页面职责和实现前检查。

## 当前结论

`studio-client-main.pen` 是当前唯一活跃的 Studio 主设计稿。它负责承载：

- 启动后的高密度 Home / 首页。
- 顶部导航下的独立 `物性` 页面。
- `流程图` 工作台的画布、模块库、项目树、检查器、模块设置、模块结果、底部运行信息和状态汇总。
- 模块设置 / 模块结果在右侧栏或画布标签页中的主结构。

`unit-module-panel.pen` 不再作为独立完整 Workbench 复制稿维护。后续如果确实需要细化单元模块详情，应新建窄口径 `module-settings-panel.pen`，只聚焦右侧栏、弹窗或画布标签页内的模块设置 / 结果结构，不重复整套 Home / Workbench 壳。

## 当前实现映射

当前已经允许从 presentation / window model 小切片进入代码实现，但仍不做 egui 大布局重排。首批实现只把设计稿中稳定的信息结构投影到 `StudioGuiWindowModel`：

| 设计区域 | 当前实现映射 | 仍未完成 |
| --- | --- | --- |
| Home 示例 / 最近 tile | `StudioGuiWindowHomeModel` / `StudioGuiWindowHomeCaseTileModel` 已统一承载示例项目、最近项目和当前 workspace 返回 tile；示例项目从 `runtime.example_projects` 派生，最近项目仍由 shell preferences / `project_open` 持有并映射为同一 DTO，未保存当前项目只从当前 `workspace_document` 派生 `Current` tile，不写入 recent projects；egui Home Dashboard 已消费统一 tile presentation，并在本次会话已打开或新建 case 后于左侧开始区提供显式 `返回工作区` 操作 | 最近项目尚未上提到 `StudioGuiSnapshot`；Home 仍未按设计稿重排为完整 tile gallery 或工作区历史管理 |
| 顶部导航 / 流程图 / 运行 / 结果上下文 / 工具设置菜单 | Studio shell 顶部已从旧 `快速操作` 横排按钮收敛为 `文件 / 主页 / 物性 / 流程图 / 运行 / 结果 / 工具 / 设置`；打开 / 新建 / 保存 / 示例归入 `文件`；`流程图` screen 顶部导航下方已新增 `window.flowsheet_context_toolbar`，Canvas 段只消费当前可用 suggestion / pending-edit command，不再重复左侧 `模块` 放置 palette 或中央 `画布操作` 选中对象动作，Run 只消费已启用 Run Panel command，Review 只消费 Module Results、结果表和 snapshot 状态；`物性` screen 的 `window.property_context_toolbar` 继续消费 package / component command 和 Property page 状态，并已增加同源进入流程图建模 readiness；`运行` 已从下拉菜单收敛为 screen，`window.run_context_toolbar` 只消费 Run Panel command / state、status summary、canvas suggestion count 和运行日志，顶部 Monitor 入口不在按钮旁重复展示状态 chip；`结果` 已从下拉菜单收敛为 screen，`window.result_context_toolbar` 顶部只展示 Module Results、底部结果表、当前 / stale / missing `SolveSnapshot` 状态和结果数量扫读，不把所有 Result focus command 展开成长按钮；具体 stream / unit 定位继续由结果表、右侧相关 action、项目树和命令面板派发正式 focus command；`工具 / 设置` 菜单内容已拆成可测试分组，分别消费 command palette shell state、Commands panel layout visibility、AppHost logical windows 和当前 shell locale | 厚重 ribbon、完整多页上下文工具栏、单位集设置、偏好页、help command、插件管理、账号 / 授权 / 服务器设置、完整运行控制台、完整日志系统、批量运行、完整报表、跨快照报表、模板、打印和批量导出尚未进入范围 |
| 左侧模块 / 项目 | Studio shell 左侧顶层入口已从 `项目 / 示例项目 / 放置` 收敛为 `模块 / 项目`；`模块` 继续消费既有 Canvas place-unit palette、authoring checklist 和 suggestion action，并已按 `流股源 / 调节单元 / 汇合与分离` 分类现有受控单元，支持本地筛选，是当前受控单元的放置入口；`项目` 已整理为 `项目输入 / 示例入口 / 对象树 / 审阅状态`，继续消费项目对象树、项目物性包 / 组分扫读、示例入口和当前 run / snapshot 状态 | 完整模块库、完整项目浏览器、自由连线、自动布线和完整拖拽布局尚未进入范围 |
| 中央 Canvas | 中央 Canvas 已移除重复的对象树副本，改为从既有 canvas presentation 派生 `画布状态` 数量概览；首屏已从独立 `选择 / 视口` 详情行收束为 `画布工具`、`画布状态` 与 `画布操作`：工具区只承接 `适应内容` 视图命令和当前建议命令，不再重复左侧 `模块` 面板的放置 palette；状态行承接对象数量、运行状态、视口模式和布局状态；操作条承接选中对象的聚焦、移动、断开、重连和删除命令，且这些选中对象动作不再被顶部 `流程图工具栏` 重复渲染；画布主体继续渲染图例、单元 / 物料线实体和受控建议，流股 / 单元对象导航由左侧 `项目` 面板和画布实体点击承担，选择语义详情由右侧栏承接；普通空白项目进入空 Workbench 后的首屏职责已由 focused test 锁定，中央不重复 palette、对象树、右侧栏或结果表 | 自由连线、自动布线、完整项目浏览器、完整拖拽布局、完整工具条体系、shell 私有对象状态和第二套对象树尚未进入范围 |
| 独立物性页 | `StudioGuiWindowPropertyPageModel` 从 `workspace_document.property_package_choices`、`project_component_choices` 和已选 package / components 派生 package、component、metric、future section 和进入流程图建模 readiness；Studio shell 已新增顶部 `主页 / 物性 / 流程图` 导航，独立 `物性` screen 消费同一 DTO；普通空白项目创建后先进入 `物性`，Property 页摘要入口、顶部 `流程图` 导航、Home 当前 workspace 返回入口和内部进入 Workbench 行为共用同一 `flowsheet_modeling_enabled` 判断；缺 package / 项目组分 readiness 也聚焦该页面；命令仍走既有 package / component command id；右侧栏已移除 Package 主入口；顶部导航下方已新增 `window.property_context_toolbar`，只渲染当前可用 package / component 命令、进入建模入口和 package / component / modeling / source 状态；当前 egui MVP 页面不渲染无操作价值的左侧二级导航，只保留 package / 项目组分选择区和摘要区 | 物性页长期分析控件、更完整视觉重排、完整组分数据库、第三方物性包加载、完整 Thermodynamics PMC 和完整参数表尚未进入范围 |
| 底部运行信息 / 状态汇总 | `StudioGuiWindowStatusSummaryModel` 从 document saved/revision、run panel view、latest current-revision `SolveSnapshot`、stale snapshot 或 latest failure 派生；底部抽屉已收敛为左侧 `消息 / 运行日志 / 收敛 / 建议 / 诊断 / 结果表` 与右侧 `状态汇总` 分栏，其中 `收敛` 消费 status summary 与 snapshot 状态，`建议` 消费 Run Panel notice 和 canvas suggestions，`结果表` 的单元区消费 `snapshot.review_summary.unit_results` 而不是在 shell 内重新扫描 steps；结果表 stream / unit 点击派发正式 `inspector.focus_stream:*` / `inspector.focus_unit:*`，stream 定位到右侧 `检查器`，unit 定位到右侧 `模块结果`，底部保持 `结果表`；右侧状态汇总只消费同一 DTO 的 case、run、convergence、steps、unit result count、diagnostics 和 snapshot 一致性；底部薄状态栏只展示 run、snapshot、SI 单位、求解器、流程图模式和当前选择扫读，不再重复完整状态汇总 | 完整收敛曲线、完整建议系统、完整报表和跨快照报表尚未进入范围 |
| 右侧 Inspector / Module Settings | 右侧栏主入口已收敛为 `检查器 / 模块设置 / 模块结果`，并在三入口正文前用同一 `画布选择` 上下文头消费 Canvas current selection / command presentation；`检查器` 继续消费 active inspector detail；`StudioGuiWindowModuleSettingsModel` 已从 active unit Inspector detail 派生参数摘要、参数字段、端口、连接动作、诊断动作和空帮助状态，并由右侧 `模块设置` tab 消费；参数摘要只统计现有字段、notice 和 batch command，不新增第二套参数状态；流股选择时模块设置保持已有 unit-only 空状态 | help command 还没有正式 command surface；完整视觉重排、完整参数表和第二套对象状态尚未进入范围 |
| Module Results | `StudioGuiWindowModuleResultsModel` 已从 current-revision `SolveSnapshot` 派生 selected unit result、consumed / produced stream chips、related steps、diagnostics 和 diagnostic actions；右侧 `模块结果` tab 已消费该 DTO，且其 latest unit step 与底部 `结果表` 使用的 `review_summary.unit_results` 保持同源；stale snapshot 不渲染旧 unit result；右侧 `画布选择` 上下文只说明当前 Canvas 选择，流股选择不伪造单元结果；旧 `Run` 不在右侧栏继续扩展，运行日志 / 消息 / 结果表继续由底部区域承接 | 尚未新增独立模块结果页或画布模块详情标签页；完整报表、跨快照结果和第二套结果状态仍不进入范围 |

这些 DTO 只服务展示和命令绑定，不是第二套项目、物性、运行、结果或诊断真相源。后续实现必须继续先确认状态来源，再补 presentation 字段和 focused 回归，最后才调整 egui 布局。若没有正式 command surface，例如当前模块帮助入口，就只能在 DTO 中显式表达为空状态，不能临时伪造按钮。

## 设计目标

`studio-client-main.pen` 当前版本解决 Studio 客户端主信息架构，并作为上述小切片实现的设计依据：

- Home 信息密度、风格和工作台保持一致，不再像低密度欢迎页。
- 物性作为流程模拟核心能力，成为顶部导航下的独立页面，而不是塞进左侧栏或右侧栏。
- 工作台顶部采用窄导航栏 + 上下文工具栏，学习成熟流程模拟软件的信息分层，但不直接照抄厚重 ribbon。
- 左侧栏稳定为 `模块 / 项目`：模块页中物料流在上方，单元操作按分类折叠；项目页负责项目输入、示例入口、对象树和审阅状态。
- 中央画布采用类似 IDE 的可切换标签页，保留轻量画布状态条和工具区，项目对象树留在左侧 `项目` 面板，受控单元放置入口留在左侧 `模块` 面板。
- 右侧栏稳定为 `检查器 / 模块设置 / 模块结果`，不保留独立 `运行` 或 `物性` tab。
- 底部拆成左右两栏：左侧为运行日志、收敛、建议、诊断等 tabs；右侧为当前案例状态汇总；薄状态栏只做窗口级扫读。
- 所有状态必须能映射到既有 presentation / command / state 模型，不新增第二套 UI 真相源。

## 顶部导航

进入工作区后的顶部不再常驻 `打开项目`、`打开示例` 作为主按钮；这些命令属于 `文件` 或 Home。当前 egui shell 已完成窄导航栏入口收敛，并已从 `流程图`、`物性`、`运行` 和 `结果` screen 完成上下文工具栏第一刀；`工具 / 设置` 保持顶层菜单形态，只组织既有 shell / window 状态。顶部结构分两层：

| 层级 | 内容 | 说明 |
| --- | --- | --- |
| 窄导航栏 | `文件`、`主页`、`物性`、`流程图`、`运行`、`结果`、`工具`、`设置` | `物性` 前置；取消意义不清的 `设备`；当前 `工具` 只承接命令面板、Commands panel 和逻辑窗口；当前 `设置` 只承接语言选择，单位集和偏好保留为未来空间 |
| 上下文工具栏 | 随当前导航变化 | 当前 `流程图` 工具栏显示 Canvas suggestion / pending-edit command、Run 可用命令、Module Results / 结果表入口和状态摘要，不承接放置 palette 或选中对象移动 / 连接动作；`物性` 工具栏显示既有 package / component 命令、Property page 状态和进入流程图建模 readiness；`运行` 第一刀显示既有 Run Panel command、运行状态、收敛 / 建议 / 诊断 / 日志入口；`结果` 第一刀显示既有 Module Results / 结果表入口、Result focus command 和 snapshot 状态 |

项目名称放在最顶部窗口标题栏；保存状态仍使用右侧状态 chip，例如 `已保存`、`有未保存更改`、`旧结果`。

## 设计画板

当前 `.pen` 包含 4 个主 frame：

| Frame | 目的 |
| --- | --- |
| `Home - Ready` | 启动首页，展示开始入口、最近项目、示例项目、环境状态和可行动消息 |
| `Property - Components and Methods` | 独立物性页，展示组分、项目组分表、物性方法、交互参数、数据来源和分析入口 |
| `Flowsheet - Modeling` | 流程图工作台，展示模块库、项目树、画布标签页、画布工具条、检查器和底部状态 |
| `Module - Settings and Results` | 模块设置 / 模块结果变体，展示右侧栏或画布标签页中的字段、端口、结果和诊断结构 |

P0 不画移动端 frame，不画控制面后台 frame，不画完整物性数据库或完整报表系统。

四个主 frame 应使用同一套浅色工作台语言：顶部标题栏、窄导航、上下文工具栏、左侧 card navigation / palette、中央主内容、右侧 summary / inspector cards、底部 status cards 的视觉节奏保持一致。不要让某一页退回表格堆叠或松散文字说明风格。

## Home

Home 不是营销页，也不是低密度欢迎页。它应使用与工作台一致的顶栏、分隔、状态 chip 和信息密度。最近项目和示例项目优先采用流程缩影 tile gallery：用 RadishFlow 自己的浅色 flowsheet thumbnail 表达案例结构，再配合项目名称、更新时间、路径 / 来源和状态 chip；不照抄 HYSYS 的深蓝文件图标、左侧文件菜单或视觉资产。

| 区域 | 内容 |
| --- | --- |
| 顶部 | 应用名、当前页、环境状态、登录状态、保存 / 服务状态 |
| 左侧开始区 | 本次会话已打开或新建 case 后显示 `返回工作区`；`新建项目`、`打开项目`、`打开示例项目`，小案例作者入口降级为次级链接 |
| 中央 | 最近项目、示例项目，使用可扫读的流程缩影 tile gallery，不使用通用文件图标或营销型大卡片 |
| 右侧 | 客户端、服务端、设备 / 缓存状态 |
| 底部 | 可行动消息，例如登录、示例、缓存、物性包状态 |

案例 tile 结构：

| 区域 | 内容 |
| --- | --- |
| 缩影 | 轻量 flowsheet thumbnail，例如 `Feed -> Flash Drum`、`Feed -> Cooler -> Flash Drum` 或 `Feed + Feed -> Mixer -> Flash Drum`；缩影只表达流程拓扑和关键单元，不承担完整画布预览 |
| 标题 | 项目名或示例名，最多两行，文件扩展名可保留 |
| 元信息 | 最近打开时间、路径 / 来源、物性包或组分摘要 |
| 状态 | `Ready`、`Modified`、`Missing package`、`Missing file`、`Version warning`、`Error` 等小型 chip |
| 操作 | 单击选择，双击打开；缺失文件或缺包时不静默失败，进入对应状态和 Messages |

tile gallery 的目标是让流程模拟用户能通过缩影快速识别案例类型，而不是只靠文件名和路径判断。缩影应使用项目自己的浅色画布语言、SI 单位和简化单元符号；如果暂时没有真实项目缩略图，可由内置流程摘要生成稳定的简化图，不新增第二套项目真相源。

## 独立物性页

物性页面是长期大工程，不能用一个侧栏 tab 承载。当前只画信息架构，为未来扩展留空间，不表示完整物性系统已实现。

物性页长期需要承载：

- 组分查询与选择。
- 项目组分列表。
- 组分物性编辑。
- 物性方法选择。
- 物性方法参数调整。
- 二元 / 交互参数预览。
- 自定义组分、方法或参数。
- 物性数据参考文献 / 来源。
- 计算公式展示。
- 物性分析，例如纯组分物性曲线、混合组分 Txy / Pxy 相图等。

长期线框建议采用三层组织：

| 区域 | 当前内容 | 长期扩展 |
| --- | --- | --- |
| 左侧导航 | `组分`、`方法`、`参数`、`分析`、`来源` | 可扩展为物性工作区内的二级导航 |
| 主区域 | 组分查询、项目组分表、方法选择、参数预览 | 后续可切换到组分详情、方法表、曲线分析、相图分析 |
| 右侧摘要 | 当前 package、方法、组分数、数据来源、适用范围 | 后续放 warning、适用性、版本、引用、缓存状态 |

当前 MVP 仍只支持受控内置 package 和 methane / ethane 等小型目录；设计稿不能暗示第三方物性包加载、完整组分数据库或完整 Thermodynamics PMC 已进入当前实现范围。

当前 egui MVP 实现不渲染只有未来含义的左侧二级导航，而是使用两列结构：左侧承接 package 和项目组分选择，右侧承接当前 package、组分数、来源和进入流程图建模 readiness。长期设计稿仍可保留物性工作区导航空间，但只有当对应组分详情、方法、参数、来源或分析视图进入实现范围时才落到可见 UI。

## Flowsheet Workbench

工作台是建模主界面。

| 区域 | 职责 | 不承担 |
| --- | --- | --- |
| 左侧 `模块` | 搜索、物料流、能量流、信号流、按分类折叠的单元操作；用 palette item 和 category card 呈现，不做纯文字清单 | 项目级物性配置、检查器字段编辑 |
| 左侧 `项目` | 当前项目流股、单元、结果、诊断对象导航 | 模块放置库 |
| 中央画布 | 流程图、单元、流股、端口、标签、画布状态概览、受控 suggestion、当前选择的轻量操作 | 对象树副本、当前选择详情面板、自由连线编辑器、完整自动布线系统 |
| 画布标签页 | 类似 IDE 的 flowsheet / 分析 / 模块详情标签 | 多文档复杂工作区管理 |
| 画布浮动工具条 | 选择、框选、平移、放大、缩小、适应流程、网格；浅色面板、轻描边、当前工具浅蓝高亮 | 大量低频命令、深色高对比悬浮条 |
| 右侧 `检查器` | 当前对象概览、关键字段、端口、关联诊断；用对象摘要卡、metric card、端口连接卡和状态 chip 组织 | 完整参数表、松散 label/value 文字堆叠 |
| 右侧 `模块设置` | 当前模块可编辑参数、单位、约束、提交动作 | 高级模型配置全集 |
| 右侧 `模块结果` | 当前模块和相关流股的 latest snapshot 结果 | 跨快照报表 |
| 底部左侧 | `消息`、`运行日志`、`收敛`、`建议`、`诊断` | 原始 trace 常驻墙 |
| 底部右侧 | `状态汇总`：案例状态、运行状态、收敛、执行步数、诊断数、snapshot / revision 一致性 | 完整报表 |

若当前求解器没有真实迭代次数，状态汇总显示 `N/A` 或 `Sequential steps`，不得伪造迭代数据。

## 模块设置和结果

模块设置不应该被设计成每个单元独占一整页的孤立界面。优先顺序：

1. 默认放在右侧栏 `模块设置 / 模块结果`。
2. 当字段量较大时，可在中央画布区域打开类似 IDE 的模块详情标签页。
3. 后续确有必要时，再评估 HYSYS 式弹窗或 Aspen 式画布内窗口。

当前覆盖对象仍是 Feed、Mixer、Heater / Cooler、Valve、Flash Drum。字段必须显示单位、来源、草稿状态、提交动作和约束提示；结果只读消费当前 revision 的 latest `SolveSnapshot`。

模块结果、环境摘要和状态摘要不应退化为多行 label/value 文字直排。默认使用紧凑信息块：状态 chip 表示收敛、登录、缓存和诊断状态；metric card 表示 `Duty`、`Outlet T`、诊断数等关键数值；stream chip 表示 produced / consumed streams。文字说明只用于补充语义，不作为主要视觉结构。

## 实现前检查

任何 UI 代码实现前必须确认：

- `studio-client-main.pen` 已完成评审并保存到仓库。
- 物性页仍是独立页面，未退回右侧栏或左侧栏 tab。
- 顶部导航、左侧栏、右侧栏、底部分栏与本 brief 一致。
- UI 状态能映射到既有 presentation / command / state 模型。
- 不引入自由连线、完整拖拽布局、自动布线、完整参数表、完整报表或第三方物性包加载。
- 需要新增 presentation 字段时，字段职责明确，不是 shell 私有补丁。
- 需要新增命令时，命令进入正式 command surface。
- 文档和周志随实现同步更新。
