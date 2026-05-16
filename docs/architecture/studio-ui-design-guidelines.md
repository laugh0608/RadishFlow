# Studio UI Design Guidelines

更新时间：2026-05-14

## 用途

用途：为 RadishFlow Studio 的首屏、画布、示例管理、面板、按钮、文字和结果审阅建立一致的 UI 设计规范，作为下一轮 Studio 工作台重排与实现拆分依据。
读者：负责 Studio shell、Canvas、Inspector、Runtime、Result 面板的开发者和设计协作者。
不包含：完整视觉稿、图标资产、主题系统、自由连线编辑器、完整报表系统、商业化多文档工作台或求解 / 物性模型设计。

## 设计定位

RadishFlow Studio 继续走轻量、清晰、工程化的浅色桌面应用风格。当前用户认可的是“干净、直接、带一点现代工具感”的基调，因此后续优化不应把界面改成传统厚重 ribbon 的复制品，也不应转向深色监控大屏或营销型产品页。

参考 Aspen、HYSYS、PRO/II、COFE、DWSIM 等同类软件时，只吸收它们的信息架构、任务分区、状态呈现和流程模拟行业习惯；不复制专有视觉资产、具体图标、颜色组合、面板外观或功能范围。

核心原则：

- 画布是主舞台，其他区域为画布建模、求解运行和结果审阅服务。
- 顶部只放全局身份、主路径命令和运行状态，不堆调试计数、窗口控制或完整命令清单。
- 左右侧栏各有稳定职责，不把对象库、属性编辑、日志、授权、结果和调试信息混在同一列里。
- 结果展示只读消费 `SolveSnapshot`，不新增 Studio shell 私有结果缓存或第二套求解解释。
- MVP α 阶段优化信息层级和操作路径，不扩自由连线编辑器、完整拖拽布局、自动布线和完整报表。

## 参考图启发

参考素材当前保存在 `adapters/reference/`。下表使用从本文档出发的相对路径，便于在 Markdown 预览中直接打开：

| 素材 | 路径 | 用途 |
| --- | --- | --- |
| 当前 RadishFlow UI | [radish-flow-1.png](../../adapters/reference/radish-flow-1.png) | 当前界面基线，保留轻量浅色风格并识别粗糙分区 |
| RadishFlow 生成稿 1 | [radish-flow-2.png](../../adapters/reference/radish-flow-2.png) | 第一轮 AI 生成 UI 初稿，布局方向基本正确，但存在状态语义、单位和底部 tab 关系问题 |
| RadishFlow 生成稿 2 | [radish-flow-3.png](../../adapters/reference/radish-flow-3.png) | 修正后的 UI 初稿推荐基线，状态、SI 单位、单文档标题和结果区关系更符合当前规范 |
| 参考 1 | [cankao-1.png](../../adapters/reference/cankao-1.png) | 深色工程工作台分区、运行状态和底部结果区参考 |
| 参考 2 | [cankao-2.png](../../adapters/reference/cankao-2.png) | 浅色 ribbon / 左侧模型树 / 右侧状态面板参考 |
| 参考 3 | [cankao-3.png](../../adapters/reference/cankao-3.png) | 接近 RadishFlow 方向的浅色流程模拟工作台参考 |
| 参考 4 | [cankao-4.png](../../adapters/reference/cankao-4.png) | Aspen 类 flowsheet、左侧模拟树和底部模型选项板参考 |
| 参考 5 | [cankao-5.png](../../adapters/reference/cankao-5.png) | Aspen 类控制面板、运行消息和收敛状态参考 |
| 参考 6 | [cankao-6.png](../../adapters/reference/cankao-6.png) | HYSYS 类流程图、左侧对象树、底部消息区参考 |
| 参考 7 | [cankao-7.png](../../adapters/reference/cankao-7.png) | PRO/II 类主命令、消息栏和右侧对象库参考 |
| 参考 8 | [cankao-8.png](../../adapters/reference/cankao-8.png) | COFE 类轻量工具栏、点阵画布和底部日志参考 |
| 参考 9 | [cankao-9.png](../../adapters/reference/cankao-9.png) | DWSIM 类左右双栏、对象属性和 palette 分类参考 |

### 当前 RadishFlow

当前 UI 的优势是浅色、克制、状态 chip 清晰，顶部快速操作已经能表达打开示例、打开项目、运行、保存和命令面板这些主路径。问题集中在排版和分区：按钮像连续灰色标签，画布工具、suggestion、对象列表、运行面板、授权和调试信息混杂，导致用户很难判断“现在该看哪里、下一步点哪里、结果在哪里”。

后续应保留当前风格基调，但把功能分区从“开发态信息平铺”重排为“建模工作台”。

`radish-flow-3.png` 当前可作为下一轮真实 UI 重排的视觉基线：保留轻量浅色桌面应用气质、单文档项目标题、顶部主路径命令、中央 flowsheet canvas、左侧 Project / Palette、右侧 Inspector tabs、底部 Results Table / Messages 区和 SI 单位展示。后续实现时仍需把它转译为现有 `egui` 组件和 `SolveSnapshot` / window model 边界，不直接把图中所有视觉细节视为代码契约。

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

下一轮先讨论并冻结工作台信息架构，再进入代码实现。除非是修复阻塞性错误，不应继续在现有杂乱布局上叠加零散按钮、临时说明、调试状态或一次性 smoke 面板。

## 信息架构

Studio 默认工作台建议分为六个稳定区域。

### 顶部 App Bar

职责：

- 应用和项目身份：项目名、脏状态、运行模式、当前单位集。
- 全局主路径：新建空白、打开示例、打开项目、运行、保存、另存为、命令面板。
- 全局状态：运行状态、pending work、授权 / 物性包摘要、最近错误入口。

规则：

- 顶部默认不显示完整项目路径；路径放入项目详情或 tooltip。
- 运行按钮必须在顶部保持可见，禁用时给出短原因。
- 低频视图切换、语言切换、逻辑窗口、调试窗口进入菜单或命令面板。
- 状态 chip 数量控制在 3-5 个，避免把每个内部计数都做成 chip。

### 左侧 Navigator / Examples / Palette

职责：

- 示例管理：内置示例列表、最近项目、打开示例入口和示例说明摘要。
- 当前项目树：Components、Property Package、Streams、Units、Results、Diagnostics。
- 建模对象库：Feed、Mixer、Heater/Cooler、Valve、Flash Drum 等 MVP 对象。
- 搜索和分类过滤。

规则：

- 左侧默认宽度建议 240-280 px。
- 对象库按钮应有领域图标和短标签，不使用长句说明。
- suggestion 是辅助建模入口，不和对象库按钮混排成一列命令。
- 示例、项目树与对象库可以用 tab 或分段控件切换，避免同时展开造成拥挤。
- 打开示例不应只依赖顶部按钮；左侧必须有稳定、可扫读的示例管理入口。

### 中央 Flowsheet Canvas

职责：

- 展示和操作流程图。
- 呈现设备、流股、连接关系、选择状态、错误状态和关键结果标签。

规则：

- 画布默认占窗口最大面积，左右侧栏和底部面板不得压缩到只剩小预览。
- 画布工具条应以图标或短标签表达选择、放置、连接、平移、缩放、适配视图。
- 本地建模 suggestion 的接受动作应使用明确的 `连接` / `Connect` 或等价动词，不用泛化的 `Apply` 让用户猜测会改写什么。
- 长说明、状态解释和开发态计数不直接堆在画布上方；进入 legend、tooltip 或底部消息。
- 流股标签优先显示名称；求解后可在缩放足够时显示关键 `T / P / F / H` 摘要。
- 设备图形保持简化 process symbol 风格：清晰、平面、少装饰，可区分类型，不追求拟物渲染。
- 错误、未配置、未收敛、未授权等状态用统一 badge 和 outline 表达，不用整块高饱和背景。

### 右侧 Inspector

职责：

- 当前选择对象的属性编辑。
- 运行上下文摘要。
- 最新结果详情。
- 授权 / 物性包的可操作状态。

规则：

- 右侧默认只展示和当前选择或当前任务相关的信息。
- 建议以 `Inspector / Run / Results / Package` tab 或等价分段组织；授权 / entitlement 在当前 demo 主路径中低频，默认不应压过物性包和结果审阅。
- 属性字段采用 label + input + unit + validation 的行结构；单位必须紧贴数值，不藏在说明文字里。
- 从左侧 Project、Canvas 对象列表或结果定位动作选择 stream / unit 后，应自然切换到对应 Inspector；stream 优先暴露 `T / P / F`、组成草稿和提交/归一化动作，unit 优先暴露端口、关联步骤、关联诊断和最新只读结果。
- 草稿态、未归一组成、运行阻断和只读结果要有稳定视觉语义。
- Runtime 中的开发态活动、平台 timer、GUI activity、原始项目路径编辑默认折叠。

### 底部 Workbench Drawer

职责：

- Messages
- Run Log
- Results Table
- Convergence / Diagnostics

规则：

- 默认高度建议 180-260 px，可折叠。
- Messages 放用户可行动摘要，Run Log 放较原始的运行过程。
- 结果表格按 stream-centric / unit-centric 组织，保持和 `SolveSnapshot` 语义一致。
- 底部面板不应默认展示整屏原始日志；原始日志作为展开详情或复制入口。

### 底部 Status Bar

职责：

- 当前单位集、solver 状态、收敛摘要、缩放比例、选择数量、后台任务状态。

规则：

- 状态栏只放短文本或图标，不承载操作主路径。
- 错误入口可以常驻，但点击后跳转到底部 Messages 或右侧 Inspector。

## 功能分区规范

### 主命令

主命令按任务组织，而不是按实现模块平铺。

建议分组：

- Project：新建空白、打开示例、打开项目、保存、另存为。
- Build：选择、放置对象、连接、删除、撤销、重做。
- Run：运行、暂停 / hold、恢复、检查状态、清除运行消息。
- Results：查看流股结果、查看单元结果、结果表、诊断。
- View：缩放、适配视图、面板显示、语言。
- Tools：命令面板、开发诊断、导出调试信息。

MVP α 默认只把 Project / Run / Results 的核心动作摆到第一视野，Build 和 Tools 逐步进入画布工具条、侧栏或命令面板。

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

- 面板标题短而稳定，例如 `Canvas`、`Inspector`、`Results`、`Messages`。
- 面板内部用 section 分组，不做卡片套卡片。
- 可折叠 section 默认只折叠低频和开发态内容。
- 面板宽度不足时优先换行和收起说明，不让文字挤出或覆盖控件。

## 视觉语言

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

1. 先冻结首屏草图和分区职责：顶部、左侧、中央、右侧、底部各放什么，不先写代码。
2. 顶部 command bar 定型：保留项目身份、主路径命令和运行状态，清理调试式状态常驻。
3. 左侧区域改为 Examples / Project / Palette 三入口：示例管理、项目树和对象库分职责，不再展示完整命令大全。
4. Canvas header / toolbar 重排：把 Place / suggestion / selection / zoom / legend 分开，长说明进入 tooltip 或底部 Messages。
5. 右侧 Runtime 面板拆成任务化 Inspector：Inspector、Run、Results、Package 只显示当前相关内容。
6. 底部建立 Messages / Run Log / Results drawer：把运行日志、GUI activity 和结果摘要从右侧长列表中迁出。
7. 画布对象视觉升级：先统一 MVP 单元图形、流股标签、选中态和错误 badge，再考虑更多设备符号。
8. 结果审阅表格化：在不改变 `SolveSnapshot` 语义的前提下提供 stream / unit 表格和诊断摘要。

## 验收检查

每轮 UI 改动完成后，至少按下面问题做自检：

- 启动后 5 秒内能否看懂如何打开示例、新建空白、运行、查看结果、保存 / 另存为？
- 画布是否仍是首屏最大区域？
- 顶部是否只保留全局身份、主路径命令和关键状态？
- 左侧是否只承担项目导航或对象库，而不是命令垃圾桶？
- 右侧是否只展示当前选择、运行或结果的相关信息？
- 原始日志和开发态活动是否没有压过用户主路径？
- `Snapshot missing`、pending、draft、unnormalized、error 是否有统一且可解释的视觉语义？
- 禁用运行或保存时，用户是否能知道原因？
- 选择流股或单元后，是否能自然进入 Inspector 并看到可编辑参数、组成和端口关联？
- 结果入口是否直接来自 `SolveSnapshot`，没有新增 shell-local 结果真相源？
- 改动是否仍遵守 MVP α 非目标：不扩自由连线、完整拖拽布局、自动布线和完整报表？
