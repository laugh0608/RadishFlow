# Studio UI 专题计划

更新时间：2026-06-08

## 用途

用途：为 Studio UI 专题阶段提供设计前置边界，明确端点清单、优先级、信息架构、主工作流、状态模型和 `.pen` 设计稿规则。  
读者：负责 RadishFlow Studio、模块设置 UI、控制面 UI、移动端或只读视图设计与实现的开发者和设计协作者。
不包含：具体视觉稿、实现代码、视觉 token 细则、完整主题系统、自由连线编辑器、完整报表系统或发布计划。

本文是专题计划文档。长期视觉规范仍以 `docs/architecture/studio-ui-design-guidelines.md` 和 `docs/architecture/studio-visual-system.md` 为准；外部设计参考仍以 `docs/architecture/ui-inspiration-reference.md` 为准。

## 阶段目标

Studio UI 专题不再处理零散按钮、文案、hover、局部 selector 或一次性 smoke 面板。首轮设计前置已完成，当前代码实现仍按本计划的端点和状态来源约束做窄口径 presentation / window model 小切片：

- 明确哪些端点进入首批设计稿，哪些端点只保留边界。
- 明确每个端点的主任务、页面职责、状态来源和禁止提前实现的能力。
- 明确 `.pen` 设计稿目录、命名、评审和同步规则。
- 明确设计稿到代码实现的边界：实现必须继续沿用正式 presentation / command / state 模型。

阶段完成标准：

- 本文档列出的首批端点、暂不纳入项和设计稿规则已经稳定。
- 首批 `.pen` 设计稿可以按同一目录和命名规则创建或继续维护。
- 每个设计稿开始前都能回答主任务、状态来源、可操作命令和不包含内容。
- 不需要通过新增 UI 代码来证明本阶段完成。

## 端点清单

### 首批设计端点

首批只覆盖会直接影响当前 Studio 主工作流的端点：

| 端点 | 设计稿建议名 | 优先级 | 目标 |
| --- | --- | --- | --- |
| Studio 客户端本体 | `studio-client-main.pen` | P0 | 统一 Home、独立物性页、流程图工作台、模块设置 / 模块结果、运行信息和状态汇总 |
| 模块设置专项 | `module-settings-panel.pen` | P1 按需 | 在 P0 主稿稳定后，窄口径细化右侧栏、弹窗或画布标签页中的模块设置 / 结果结构 |
| 控制面 / 服务端管理 UI | `control-plane-admin.pen` | P2 | 梳理身份、授权、物性资产、版本、租约和分发状态的管理型页面边界 |
| 移动端或只读视图 | `readonly-mobile-view.pen` | P2 | 梳理项目浏览、运行状态、结果审阅和诊断摘要的只读体验 |

### 本阶段重点

P0 先行。P1 不再维护完整 Workbench 复制稿；只有当 `studio-client-main.pen` 中的模块设置 / 模块结果结构评审后仍不够细，再创建窄口径 `module-settings-panel.pen`。当前右侧栏第一轮实现已收敛为 `检查器 / 模块设置 / 模块结果`：模块设置的参数、端口和诊断来自正式 active Inspector state / command 来源，模块结果来自 `window.module_results`，帮助入口尚无正式 command，因此暂不创建 `module-settings-panel.pen`。左侧栏已收敛为 `模块 / 项目`：`模块` 承接放置 palette、作者任务清单和画布建议，并已把现有受控单元按 `流股源 / 调节单元 / 汇合与分离` 分类和本地筛选组织；`项目` 承接项目输入、示例入口、对象树和审阅状态，继续消费项目对象、项目级输入扫读和可展开示例入口。中央 Canvas 已在左侧分区稳定后收束为建模主舞台：它只渲染 `画布状态` 数量概览、工具条、选择、视口、图例、画布实体和受控建议，不再重复渲染项目对象树。底部运行信息区域已补齐 `收敛 / 建议` 入口，并已完成左右分栏第一刀：左侧继续承接 `消息 / 运行日志 / 收敛 / 建议 / 诊断 / 结果表`，右侧固定消费 `window.status_summary` 展示 case、run、convergence、steps、diagnostics 与 snapshot 一致性；完整收敛曲线、完整建议系统和完整报表仍不进入范围。P2 只做边界和页面职责，不进入完整设计稿细化，除非 P0 / P1 已经评审通过。

当前不把端点拆到更细，例如单独的 Home 设计稿、Canvas 设计稿或 Result 设计稿。原因是 Studio 客户端本体必须先统一主工作流和区域职责，再决定是否拆分子稿。若后续 P0 设计稿过大，再按 Home / Property / Workbench / Result 等域拆分，并保留同一命名前缀。独立物性页当前已完成顶部导航下的第一刀实现，继续消费正式 Property page DTO；`Package` 已不再作为右侧栏主入口，物性主路径由独立 `物性` screen 承担。顶部导航已从旧 `快速操作` 横排按钮收敛到 `文件 / 主页 / 物性 / 流程图 / 运行 / 结果 / 工具 / 设置`：打开 / 新建 / 保存 / 示例归入 `文件`，运行和结果只路由到现有 run command、右侧 `模块结果` 和底部结果表，不新增完整 ribbon 或第二套命令状态。`流程图` screen 已完成上下文工具栏第一刀：`window.flowsheet_context_toolbar` 只从既有 command registry、canvas presentation、Run Panel / status summary、Module Results 和 current / stale / missing snapshot 状态派生，Canvas / Run 只渲染已启用命令，Review 入口只指向右侧 `模块结果` 与底部 `结果表`。`物性` screen 已完成上下文工具栏第一刀：`window.property_context_toolbar` 只从 `StudioGuiWindowPropertyPageModel` 的内置 package、项目组分 command id、selected / remove-enabled 状态和内置来源摘要派生，未引入第二套物性状态源或尚未进入范围的物性能力。`运行` screen 已完成上下文工具栏第一刀：`window.run_context_toolbar` 只从 Run Panel command registry、Run Panel state、status summary、canvas suggestion count 和运行日志派生 Control / Recovery / Monitor 入口，不把运行入口扩成完整控制台、完整日志系统、批量运行、自动调度或完整报表。`结果` screen 已完成上下文工具栏第一刀：`window.result_context_toolbar` 只从 Module Results、底部结果表、当前 / stale / missing `SolveSnapshot` 状态和已启用 Result focus command 派生 Review / Focus / Status 入口，不扩完整报表、跨快照报表、模板、打印或批量导出。`工具 / 设置` 顶层菜单已完成职责对齐第一刀：工具菜单只组织命令面板、Commands panel 可见性和逻辑窗口，设置菜单只组织当前语言选择；help command、插件管理、单位集设置、完整偏好页、账号 / 授权 / 服务器设置和发布入口仍不进入当前实现。

## 暂不纳入项

以下内容不进入本专题第一轮设计，也不应作为设计稿中的当前可用入口：

- 自由连线编辑器、任意端口选择器、自动布线系统、完整拖拽布局编辑器。
- 完整报表系统、跨快照报表、模板导出、打印、批量导出。
- 完整参数表、完整组分数据库、第三方物性包加载、完整 Thermodynamics PMC。
- tag、release notes、便携包刷新、安装器、发布自动化。
- CAPE-OPEN / COM 语义倒灌到 Rust Core 或通用 Studio UI。
- 为尚未进入范围的移动端编辑、多人协作、控制面运营后台提前设计完整功能矩阵。

设计稿可以为这些能力保留架构空间，但不能把它们画成当前已实现能力。

## Studio 客户端本体

Studio 客户端本体设计稿覆盖当前用户从启动、配置物性、建模到结果审阅的主路径。

### 页面职责

| 区域 | 职责 | 不承担 |
| --- | --- | --- |
| Home | 开始项目、打开示例、最近项目、环境状态、登录入口；最近 / 示例案例使用 RadishFlow 浅色流程缩影 tile gallery 展示项目名、路径 / 来源、时间和状态 | 营销页、完整控制台、完整命令面板、HYSYS 深蓝文件图标或左侧文件菜单复刻 |
| Property Page | 独立物性工作区：组分、项目组分、方法、参数、来源、分析入口 | 完整组分数据库、第三方物性包加载、完整 Thermodynamics PMC |
| Top Navigation | 窄导航栏与上下文工具栏；导航顺序为 `文件`、`主页`、`物性`、`流程图`、`运行`、`结果`、`工具`、`设置` | 完整厚重 ribbon、调试计数、所有低频命令 |
| Left Rail | `模块 / 项目` tabs；模块页负责流股和单元放置，项目页负责项目输入、示例入口、对象树和审阅状态 | 物性主配置、检查器字段编辑、结果审阅详情 |
| Canvas | 流程图主舞台、可切换画布标签页、浮动工具条、画布状态概览、单元和流股实体、受控 suggestion | 对象树副本、自由连线、完整拖拽布局、长说明文本 |
| Right Rail | `检查器 / 模块设置 / 模块结果`，负责当前对象输入、端口、关联诊断和最新结果 | 运行日志、物性页、完整参数表、第二套结果解释 |
| Bottom Area | 左侧消息 / 运行日志 / 收敛 / 建议 / 诊断 tabs，右侧当前案例状态汇总 | 原始 trace 常驻、开发态内部状态墙、完整报表 |

### 主工作流

P0 设计稿必须覆盖这些用户路径：

1. 启动 Studio，判断本地环境、登录状态和最近项目。
2. 进入独立 `物性` 页面，显式选择内置 package 和项目组分。
3. 放置 `Feed -> Flash Drum`，补齐输入，运行并查看结果。
4. 放置 `Feed -> Cooler / Valve -> Flash Drum`，补齐单元参数，运行并查看中间流股结果。
5. 放置 `Feed + Feed -> Mixer -> Flash Drum`，补齐双入口、Mixer 参数，运行并查看 Mixer outlet 和 Flash split。
6. 保存、重开、rerun，并确认旧结果失效提示和最新结果入口。
7. 在 readiness 阻断和 Run Panel 正式诊断之间区分建模输入缺口、结构连接问题和求解阶段失败。

### 物性页长期空间

物性页是独立页面，不属于侧栏。它长期需要承载：

- 组分查询与选择。
- 项目组分列表。
- 组分物性编辑。
- 物性方法选择与参数调整。
- 交互参数预览。
- 自定义组分、方法或参数。
- 物性数据参考文献 / 来源。
- 计算公式展示。
- 物性分析，例如纯组分物性曲线、混合组分 Txy / Pxy 相图等。

当前设计稿只表达信息架构和未来承载空间；MVP 实现仍只覆盖受控内置 package 与小型组分目录。

### 状态来源

设计稿中的状态必须能映射到既有模型，不新增第二套真相源：

| UI 状态 | 来源 |
| --- | --- |
| 项目标题、脏状态、revision | `WorkspaceDocument` / document metadata |
| 运行状态、pending reason、failure | Run Panel / workspace control model |
| 建模输入缺口 | shared modeling readiness |
| 结构性连接、拓扑、求解失败 | formal solve diagnostics / Run Panel recovery |
| 当前结果 | latest current-revision `SolveSnapshot` |
| 旧结果失效提示 | stale snapshot presentation |
| Canvas unit / stream / suggestion | canvas presentation / local rules suggestions |
| Inspector 字段、草稿、提交命令 | inspector draft / command surface |
| package / components | flowsheet thermo and component catalog |
| 登录 / 授权 | auth / entitlement state |

## 模块设置专项

模块设置专项不是每个单元一整页的独立参数面板，也不是完整 Workbench 的第二份复制稿。默认先在 `studio-client-main.pen` 内通过 `Module - Settings and Results` frame 承载。

若后续需要创建 `module-settings-panel.pen`，它只聚焦：

- 右侧栏 `模块设置 / 模块结果` 的字段结构。
- 字段单位、当前值来源、草稿状态、提交动作和约束提示。
- 端口已连接、未连接、suggested、diagnostic attention 的表达。
- 当前 `SolveSnapshot` 下的 consumed streams、produced streams 和 terminal outlets。
- 诊断跳转到相关 stream、unit、port 或 recovery action 的入口。
- 当字段量较大时，画布标签页或弹窗中的同一套内容如何展开。

覆盖对象仍是 Feed、Mixer、Heater / Cooler、Valve、Flash Drum。不进入完整参数表、高级模型配置全集、完整报表或未来复杂单元。

## 控制面与只读端点

P2 端点当前只做边界，不进入功能扩张。

控制面 / 服务端管理 UI 可以借鉴 Cloudflare、1Panel 和 Discourse Admin 的管理型页面组织，但第一轮只明确：

- 身份和授权状态如何展示。
- 物性资产、版本、租约和分发状态是否需要同一页面。
- 哪些状态只属于服务端，不能放进 Studio 客户端主工作台。

移动端或只读视图第一轮只明确：

- 只读项目浏览和运行状态摘要。
- 最新结果审阅和诊断摘要。
- 不提供建模编辑、运行提交、保存或包管理操作。

## `.pen` 设计稿规则

设计稿目录：

```text
docs/architecture/designs/
```

首批命名：

```text
docs/architecture/designs/studio-client-main.pen
docs/architecture/designs/module-settings-panel.pen
docs/architecture/designs/control-plane-admin.pen
docs/architecture/designs/readonly-mobile-view.pen
```

当前唯一活跃主设计稿：

```text
docs/architecture/designs/studio-client-main.pen
docs/architecture/designs/studio-client-main-brief.md
```

规则：

- `.pen` 文件只通过 Pencil MCP 工具读取、生成、验证和导出，不用普通文本工具读取或改写。
- 若 `.pen` 创建前需要先评审信息架构，可在同目录维护对应 Markdown brief；brief 只记录设计输入和评审问题，不替代 `.pen` 设计稿。
- 设计稿必须随仓库保存和同步，不只保留在个人本地或聊天记录里。
- 每个设计稿必须在评审记录中说明参考了哪些原则，不能复制外部产品品牌、图标、具体配色或页面结构。
- P0 设计稿先产出信息架构和主工作流，再细化视觉。
- 若设计稿拆分，使用稳定前缀，例如 `studio-client-home.pen`、`studio-client-property.pen`、`studio-client-workbench.pen`。
- 设计稿进入代码实现前，必须记录评审状态、实现范围和允许偏离点。

## 评审顺序

1. 评审本文档的端点边界和暂不纳入项。
2. 创建并评审 `studio-client-main.pen` 的信息架构稿。
3. 对照 `docs/architecture/studio-ui-design-guidelines.md`、`docs/architecture/assets/studio-ui/baseline/` 和 `docs/architecture/ui-inspiration-reference.md`，确认 Home、独立物性页、流程图工作台、模块设置 / 模块结果的视觉方向没有冲突。
4. 决定是否拆分 P0 子稿。
5. 若模块设置 / 模块结果细节不足，再创建并评审窄口径 `module-settings-panel.pen`。
6. 仅在 P0 / P1 稳定后，评估 P2 是否需要进入设计稿细化。

## 实现前检查

任何 UI 代码实现前，必须先确认：

- 设计稿对应的 `.pen` 文件已存在且评审通过。
- 物性页仍是顶部导航下的独立页面，不退回侧栏 tab。
- 改动不会引入自由连线、完整拖拽布局、自动布线、完整参数表或完整报表。
- UI 状态能映射到既有 presentation / command / state 模型。
- 需要新增 presentation 字段时，字段职责明确，不是 shell 私有补丁。
- 需要新增命令时，命令进入正式 command surface，不只藏在单个面板里。
- 结果审阅仍只消费当前 revision 的 latest `SolveSnapshot`。
- 文档、测试和周志会随实现同步更新。
