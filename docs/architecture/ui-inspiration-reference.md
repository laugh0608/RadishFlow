# UI Inspiration Reference

更新时间：2026-05-23

## 用途

用途：整理一组优秀产品截图给 RadishFlow 后续 UI 专题提供视觉语言、排版、信息密度和交互状态参考。
读者：负责 RadishFlow Studio、单元模块 UI、控制面 UI、移动端或只读视图设计与实现的开发者和设计协作者。
不包含：可直接照搬的界面稿、品牌复刻、图标资产、完整主题系统、`.pen` 设计稿或当前阶段必须立即实现的 UI 任务。

这些截图只作为设计灵感和评审参照。RadishFlow 后续应吸收其中的设计方法，例如布局节奏、留白、分组、状态表达和信息密度控制，而不是复制任何产品的品牌、图标、具体配色、组件外观或页面结构。

## 资产位置

本批参考截图来自用户在 2026-05-22 收集的 AFFINE、1Panel、Discourse、GitHub、Cloudflare 和 CodexApp 界面，共 17 张，已复制到：

`docs/architecture/assets/studio-ui/inspiration-20260522/`

| 来源 | 文件 | 主要参考点 |
| --- | --- | --- |
| AFFINE | [affine-doc-list-sidebar-calendar.png](assets/studio-ui/inspiration-20260522/affine-doc-list-sidebar-calendar.png) | 三栏页面的柔和分区、导航密度、右侧辅助面板和列表扫读节奏 |
| AFFINE | [affine-settings-modal-appearance.png](assets/studio-ui/inspiration-20260522/affine-settings-modal-appearance.png) | 大尺寸设置弹窗、左侧类别导航、圆润控件和低噪声表单布局 |
| AFFINE | [affine-document-properties-editor.png](assets/studio-ui/inspiration-20260522/affine-document-properties-editor.png) | 文档编辑主舞台、右侧属性栏、元数据标签和正文留白 |
| Cloudflare | [cloudflare-account-home-analytics-cards.png](assets/studio-ui/inspiration-20260522/cloudflare-account-home-analytics-cards.png) | 控制台首页的卡片分组、指标摘要、侧栏导航和主操作入口 |
| Cloudflare | [cloudflare-access-overview-actions-metrics.png](assets/studio-ui/inspiration-20260522/cloudflare-access-overview-actions-metrics.png) | 任务导向 overview 页面、快速操作、建议列表和右侧指标列 |
| GitHub | [github-repository-file-list-about.png](assets/studio-ui/inspiration-20260522/github-repository-file-list-about.png) | 顶部导航、仓库文件表、右侧 about 面板和操作区密度 |
| GitHub | [github-profile-repository-list.png](assets/studio-ui/inspiration-20260522/github-profile-repository-list.png) | 左侧身份信息、右侧列表、过滤控件和条目摘要 |
| GitHub | [github-account-settings-sections.png](assets/studio-ui/inspiration-20260522/github-account-settings-sections.png) | 设置页分组、左侧导航、危险操作隔离和表单段落节奏 |
| Discourse | [discourse-topic-list-categories.png](assets/studio-ui/inspiration-20260522/discourse-topic-list-categories.png) | 论坛 topic 列表、分类标签、活动数据和侧栏频道组织 |
| Discourse | [discourse-chat-channel-thread.png](assets/studio-ui/inspiration-20260522/discourse-chat-channel-thread.png) | 聊天频道的轻量侧栏、消息纵向节奏和底部输入区 |
| Discourse | [discourse-admin-dashboard-activity.png](assets/studio-ui/inspiration-20260522/discourse-admin-dashboard-activity.png) | 管理面板的信息中心、统计表、推荐列表和活动摘要 |
| Discourse | [discourse-admin-traffic-analytics.png](assets/studio-ui/inspiration-20260522/discourse-admin-traffic-analytics.png) | 数据看板中的图表密度、网格、标题和时间序列表达 |
| 1Panel | [onepanel-file-manager-table.png](assets/studio-ui/inspiration-20260522/onepanel-file-manager-table.png) | 管理型文件表格、顶部批量命令、路径栏和侧栏模块导航 |
| 1Panel | [onepanel-app-store-grid.png](assets/studio-ui/inspiration-20260522/onepanel-app-store-grid.png) | 应用商店卡片网格、筛选、分页、标签和安装动作 |
| 1Panel | [onepanel-monitoring-dashboard-charts.png](assets/studio-ui/inspiration-20260522/onepanel-monitoring-dashboard-charts.png) | 监控看板图表布局、时间范围控件和低饱和曲线配色 |
| CodexApp | [codex-settings-general-permissions.png](assets/studio-ui/inspiration-20260522/codex-settings-general-permissions.png) | 温和圆润的设置页、宽松内容列、权限分组和开关控件 |
| CodexApp | [codex-connection-permissions-detail.png](assets/studio-ui/inspiration-20260522/codex-connection-permissions-detail.png) | 连接详情页、状态 chip、权限表单和可维护的纵向节奏 |

## 总体设计启发

### 可吸收的设计语言

- 浅色中性背景上使用低饱和浅灰、淡蓝灰或轻微冷色侧栏，让界面更温和，不靠大面积强色建立层级。
- 主内容区保持白色或近白色，大块内容用边界、间距、标题和极浅色底分组，不依赖厚重卡片堆叠。
- 关键操作使用明确主色，次级操作和状态 chip 保持克制，避免每个按钮都抢占视觉焦点。
- 圆角可以比传统工程软件更柔和，但仍应服务可读性：常规面板和卡片建议 6-8 px，弹窗和顶层容器可略大，表格行和字段组不需要过度圆角。
- 图标线条保持轻量、统一尺寸和统一描边，尽量减少填充图标、复杂徽章和多套风格混用。

### 信息密度

- RadishFlow 是工程工具，不适合做稀疏营销页；信息密度应接近 AFFINE / CodexApp 的设置页和 Cloudflare / GitHub 的管理页：紧凑但不拥挤。
- 列表、表格和属性页优先做到一屏可扫读，再把低频解释放进 tooltip、展开项或底部消息。
- 主区域标题应短而明确；辅助说明只解释用户决策，不重复字段名，不展示内部实现术语。
- 三栏布局中，左侧负责导航和对象选择，中央负责主任务，右侧负责上下文详情；不要让三栏同时承担主操作。
- 卡片网格适合 Home、示例、应用包或控制面资源列表；求解结果、流股、端口和诊断更适合表格或属性行，不应全部卡片化。

### 视觉节奏

- 页面顶部保留一条稳定命令带或标题区，但高度要克制，避免把桌面工程软件做成厚重 ribbon。
- 左侧导航项应有图标、短标签和清晰选中态；二级导航可缩进，但不应过深。
- 中央内容列的最大宽度要受控。设置、权限、属性这类页面可采用 CodexApp / AFFINE 式居中内容列；Workbench Canvas 则必须保持全宽主舞台。
- 面板边界优先使用极浅分隔线和背景层次，不使用大量阴影。阴影只用于弹窗、浮层和临时菜单。
- 状态、标签、单位和时间应形成稳定的微型排版系统，不能每个面板各自临时拼接。

## 对 RadishFlow 的转译规则

### Home Dashboard

可借鉴 AFFINE 的清爽首页和 Cloudflare 的控制台 overview：

- 左侧只放开始路径和工作区入口，避免把所有开发态命令放进第一视野。
- 最近项目、示例项目、环境状态和消息使用清晰分区；每个分区只保留能帮助用户开始工作的字段。
- 示例列表应保持 AFFINE / GitHub 式列表扫读，而不是过度装饰的营销卡片。
- 环境状态可采用 Cloudflare 式 compact metric cards，但状态数要少，优先回答“本地是否可运行、示例是否可用、是否已登录”。

不采用：

- 不使用 Cloudflare 式大范围产品入口矩阵来承载当前 MVP 未实现能力。
- 不把 Home 做成 AFFINE 文档库复制品；RadishFlow 的主入口仍是工程项目和流程模拟示例。

### Workbench

可借鉴 AFFINE 的三栏协调感、GitHub 的操作密度和 CodexApp 的温和设置页：

- Canvas 必须是主舞台，中央区域保留最大面积；左侧 Project / Palette 和右侧 Inspector / Result 只是上下文面板。
- 顶部命令带保持 GitHub 式任务分组和 CodexApp 式低噪声，不把运行、保存、打开、调试、授权和所有面板切换平铺成同等权重按钮。
- Inspector 字段组采用 AFFINE / CodexApp 的表单节奏：标题、短说明、输入控件和状态成组出现，低频字段折叠。
- 底部 Messages / Results 可以借鉴 GitHub / Discourse 的列表密度，强调可扫读行、短状态和明确跳转动作。

不采用：

- 不把 Workbench 做成 1Panel 管理后台式全卡片页面；流程图软件的主体验仍是 Canvas。
- 不把 Discourse 的内容流样式带入核心建模区；消息流只适合底部消息或协作评论类未来功能。

### Canvas

可借鉴 1Panel / Cloudflare 的低噪声面板和状态表达，但 Canvas 本身必须服务流程模拟：

- 画布工具条采用小图标、短标签、tooltip 和分组，默认只保留选择、放置、适配视图、受控恢复和 suggestion 接受。
- 状态 badge 用低饱和颜色、轻描边和短文本，不使用高对比大色块压过设备和流股。
- 画布外的信息区应收敛，避免把长说明、调试计数和临时提示堆在 Canvas header。

不采用：

- 不把 1Panel 侧栏模块导航直接套到 Canvas；流程模拟的对象库、项目树和诊断需要领域分组。
- 不使用 Discourse 式话题标签作为画布对象主视觉，流股和单元标签仍以工程语义优先。

### Inspector / Settings / Package / Auth

可重点借鉴 AFFINE 和 CodexApp：

- 设置页、授权页、物性包页和单元参数页使用温和的纵向表单节奏，一组设置一个清晰 section。
- 左侧导航只承担定位，右侧正文按照任务顺序排列，不把每个字段都放进独立卡片。
- 开关、分段控件、下拉、状态 chip 和危险操作要有稳定样式；危险操作必须隔离在独立 section。
- 权限、连接、物性包下载、缓存路径和离线租约这类复杂状态优先做成 CodexApp 式可解释表单，而不是日志列表。

不采用：

- 不把 AFFINE 的文档属性系统直接套给流股和单元；RadishFlow 的字段仍由 `FlowsheetDocument`、`SolveSnapshot` 和正式 presentation DTO 决定。
- 不把 CodexApp 的权限术语迁移到 RadishFlow；只借鉴其权限页的分组、开关和解释节奏。

### Control Plane / Admin UI

若后续推进服务端 / 控制面 UI，可借鉴 Cloudflare、1Panel 和 Discourse Admin：

- Cloudflare 适合身份、授权、访问策略、租约和资产分发 overview。
- 1Panel 适合资源管理、文件/包列表、应用包安装和运行状态。
- Discourse Admin 适合社区、活动、用量、报表和审计类页面。

不采用：

- 不在 Studio 客户端主工作台里提前塞入完整控制台导航。
- 不把管理后台的信息密度带入 Home 的第一屏主路径。

## 未来 UI 专题中的使用方式

启动后续 UI 专题时，应把本文件作为设计前置材料之一，并按以下顺序使用：

1. 先确定端点清单：Studio 客户端本体、单元模块 UI、服务端 / 控制面、移动端或只读视图。
2. 为每个端点建立 `.pen` 设计稿，不把多个端点混在一个文件里。
3. 设计稿必须先解决信息架构和工作流，再细化视觉样式。
4. 每个设计稿评审时都要说明参考了哪些截图，以及只吸收了哪些设计原则。
5. 代码实现前，将评审结论同步到 `docs/architecture/studio-ui-design-guidelines.md` 或对应专题文档。
6. 实现时继续遵守 presentation / command / state 边界；视觉改动不得绕过正式 UI 模型直接堆 shell 私有状态。

## 评审清单

后续 `.pen` 设计稿或真实 UI 改动可以用下面问题自检：

- 这个页面是否一眼能看出主任务、当前状态和下一步动作？
- 侧栏、主内容和详情面板是否各自职责稳定？
- 信息密度是否接近 AFFINE / CodexApp 的温和紧凑，而不是过稀或过满？
- 状态色是否只表达状态，没有成为装饰色？
- 主色按钮是否只留给真正的主动作？
- 表格、列表、卡片和表单是否按内容类型选择，而不是统一卡片化？
- 低频说明、原始日志和开发态信息是否被折叠或降级？
- 是否有明确说明“借鉴了什么、没有照抄什么”？
- 是否仍符合 RadishFlow 当前 MVP 非目标：不扩自由连线、完整拖拽布局、自动布线和完整结果报表？
