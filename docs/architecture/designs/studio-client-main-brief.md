# Studio Client Main Design Brief

更新时间：2026-06-03

## 用途

用途：为 P0 `studio-client-main.pen` 信息架构稿提供可评审的文字 brief，确保 Pencil 设计稿创建与评审时已经明确页面、状态来源、主工作流和暂不纳入项。  
读者：准备绘制或评审 `docs/architecture/designs/studio-client-main.pen` 的设计协作者。  
不包含：`.pen` 设计稿本体、视觉 token、实现代码、完整控件规格或完整交互动画。

本 brief 对应 `docs/architecture/studio-ui-topic-plan.md` 中的 P0 Studio 客户端本体端点。`.pen` 文件仍应通过 Pencil MCP 工具创建、维护和验证；本文件只作为设计前置材料和评审索引。

## 设计目标

`studio-client-main.pen` 第一版只解决 Studio 客户端本体的信息架构，不做视觉精修：

- 用户能从 Home 判断环境是否可用，并进入普通空白项目或示例项目。
- 用户能在 Workbench 中看懂项目、画布、检查器、结果和诊断分别在哪里。
- 用户能完成当前已通过的四条普通空白项目主路径。
- readiness、Run Panel 诊断、旧结果失效和最新结果审阅有清晰但克制的状态表达。
- 所有状态都能映射到既有 presentation / command / state 模型。

## 评审记录

### 2026-06-03 Workbench Shell 方向修正

结论：**原 P0 / P1 Workbench 壳仍偏向旧快速操作栏，不满足成熟流程模拟软件的工作台分层要求；后续 P0 / P1 设计稿统一改为两层顶部导航 + 底部左右分栏结构。**

修正依据：

- 进入画布工作区后，`打开项目`、`打开示例` 不应继续作为顶部主按钮；它们属于 `文件` 或 Home，而不是建模工作台的第一视野。
- 顶部参考 Aspen / PRO/II / HYSYS 的任务分组，但不采用完整厚重 ribbon：使用一条窄导航栏加一条上下文工具栏。
- 物性系统是流程模拟软件核心，顶部导航顺序中 `物性` 应前置；语言、单位集、界面偏好等进入独立 `设置`，不混入物性。
- 底部不再是单一 drawer：左侧承载运行日志、收敛 / 迭代、建议、诊断；右侧承载当前案例状态汇总。
- 左侧栏保持当前方向，稳定为 `项目` 与 `放置` 两个 tab。

Workbench 顶部结构：

| 层级 | 内容 | 说明 |
| --- | --- | --- |
| 顶部导航栏 | `首页`、`文件`、`物性`、`流程图`、`设备`、`运行`、`结果`、`工具`、`设置` | `物性` 前置；`设置` 承载语言、单位集和偏好 |
| 上下文工具栏 | 随当前导航显示保存、检查输入、运行、放置设备、画布工具、结果动作等 | 不把 Home / 文件打开动作常驻为工作台主按钮 |

Workbench 底部结构：

| 区域 | Tabs | 说明 |
| --- | --- | --- |
| 左侧底部 | `消息`、`运行日志`、`收敛`、`建议`、`诊断` | 承载运行过程、迭代 / 收敛、建模建议和诊断列表 |
| 右侧底部 | `状态汇总` | 显示当前案例总状态、收敛状态、执行步数、迭代次数、诊断数和 snapshot / revision 一致性 |

状态汇总只消费既有状态来源：Run Panel / workspace control model、shared modeling readiness、formal diagnostics、latest current-revision `SolveSnapshot` 和 stale snapshot presentation。若当前求解器没有真实迭代次数，线框稿显示 `N/A` 或 `Sequential steps`，不得伪造收敛迭代数据。

### 2026-06-03 P0 信息架构评审

结论：**通过 P0 信息架构评审，可以进入 P1 `unit-module-panel.pen` 设计准备；不代表进入 UI 代码实现。**

评审依据：

- Pencil 已打开 `docs/architecture/designs/studio-client-main.pen`，顶层包含 `Home - Ready`、`Workbench - Modeling`、`Workbench - Readiness`、`Workbench - Results` 四个 frame。
- `snapshot_layout` 未报告 layout problems；四个 frame 没有发现明显崩坏、裁切或根级结构异常。
- Home frame 覆盖开始入口、最近项目、示例项目、环境状态和 Messages，职责与 P0 Home 边界一致。
- Workbench Modeling frame 覆盖顶部导航 / 上下文工具栏、Project / Palette、Canvas、Inspector、底部左侧运行信息区和右侧状态汇总；以 `Feed -> Cooler -> Flash Drum` 表达当前受控建模组合，并保留 Feed、Mixer、Cooler、Valve、Flash Drum 的放置入口。
- Workbench Readiness frame 能表达 modeling readiness 阻断、目标对象 attention、Inspector 缺失字段提交入口和底部 notice；没有把结构性连接、拓扑或求解阶段失败混入 readiness。
- Workbench Results frame 能表达 latest `SolveSnapshot` 的 Result Inspector、底部 Results Table、stream / unit 聚焦与轻量导出；评审中已补充旧结果失效的 stale notice 局部变体，明确旧快照不驱动 Inspector 或导出，并指向 rerun。

允许偏离点：

- 当前 `.pen` 仍是低保真信息架构稿，不要求达到 `baseline/` 两张视觉基线的最终视觉密度和组件精度。
- P0 没有单独绘制 `Feed -> Flash Drum`、`Feed -> Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum` 的完整 frame；这些路径通过同一 Workbench 区域职责和 P1 单元模块 UI 规则继续承接。
- P0 暂不拆分 Home / Canvas / Result 子稿；若后续实现前发现单个域过大，再按 `studio-client-*` 前缀拆分。

后续要求：

- P1 `unit-module-panel.pen` 应沿用 P0 的 Inspector / Result / Diagnostics 区域职责，不为每个单元新建互不一致的临时面板。
- 进入代码实现前仍必须回看两张 `baseline/` 视觉基线，把 P0 低保真线框转译为当前 `egui` presentation / command / state 边界可承载的真实 UI。

## 设计画板建议

首版 `.pen` 建议包含 4 个主 frame：

| Frame | 建议尺寸 | 目的 |
| --- | ---: | --- |
| `Home - Ready` | 1440 x 960 | 启动首页，展示开始入口、最近项目、示例项目和环境状态 |
| `Workbench - Modeling` | 1440 x 960 | 普通空白项目建模中，展示 Project / Palette、Canvas、Inspector 和底部消息 |
| `Workbench - Readiness` | 1440 x 960 | 缺建模输入时的阻断状态，展示 readiness notice、目标对象聚焦和字段提交路径 |
| `Workbench - Results` | 1440 x 960 | 运行收敛后结果审阅，展示 Result Inspector、底部结果表、轻量导出和 stale notice 入口 |

P0 不拆单独移动端 frame，不画控制面后台 frame。

## 全局线框规格

首版线框稿优先表达信息架构，采用低保真浅色框架，不做最终视觉：

- Frame 背景：`#F5F7FA`。
- 主要面板背景：`#FFFFFF`。
- 分隔线：`#D8DEE8`。
- 主强调：克制蓝色，仅用于主动作和运行状态。
- 字体：系统 sans-serif；中文标签可用现有 Studio 中文文案。
- 圆角：常规面板 6-8 px，按钮 6 px。
- 阴影：不在主面板使用，弹层或菜单后续再定。
- 评审时同时对照 `docs/architecture/assets/studio-ui/baseline/radishflow-home-dashboard-concept-v2-20260516.png` 和 `docs/architecture/assets/studio-ui/baseline/radishflow-workbench-concept.png`，让 Home / Workbench 的分区、信息密度和状态层级逐步靠近这两张基线图。

桌面 frame 使用 `1440 x 960`，建议固定区域：

| 区域 | X | Y | W | H | 说明 |
| --- | ---: | ---: | ---: | ---: | --- |
| Top Navigation | 0 | 0 | 1440 | 36 | Home / File / Property / Flowsheet / Equipment / Run / Results / Tools / Settings |
| Context Toolbar | 0 | 36 | 1440 | 56 | 当前导航下的主路径工具 |
| Left Rail | 0 | 92 | 288 | 648 | Workbench project and placement tabs |
| Main Stage | 288 | 92 | 816 | 648 | Home 列表或 Workbench Canvas |
| Right Rail | 1104 | 92 | 336 | 648 | Inspector / Result / Run / Property |
| Bottom Left | 0 | 740 | 864 | 190 | Messages / Run Log / Convergence / Suggestions / Diagnostics |
| Bottom Right | 864 | 740 | 576 | 190 | Case Status Summary |
| Status Bar | 0 | 930 | 1440 | 30 | Ready / units / mode / zoom / selection |

线框稿中应保留 frame 名称、区域名和状态 chip，避免放长篇说明。具体字段解释进入 brief 或后续评审记录，不塞进画布。

## Home - Ready

### 布局

- Home 顶部栏：应用名、开发构建标识、登录状态、服务端状态、设置入口。
- 左侧 Start：`新建项目`、`打开项目`、`打开示例项目`，小案例作者入口降级为次级入口。
- 中央 Recent / Examples：最近项目列表和示例项目列表，以列表扫读为主，不使用营销卡片。
- 右侧 Environment：Client、Server、Device 三组状态摘要。
- 底部 Messages：最多 3-5 条可行动环境消息。

### 状态来源

| UI | 来源 |
| --- | --- |
| 最近项目 | app preferences / MRU |
| 示例项目 | `examples/flowsheets` discovery |
| 登录状态 | auth / entitlement state |
| local ready / cache ready | runtime environment probe |
| Messages | application event / environment summary |

### 不画

- 完整命令面板。
- 完整控制台导航。
- 完整小案例 checklist。
- 未来 release / package / installer 状态。

### 线框内容

| 区域 | 内容 |
| --- | --- |
| Home Top Bar | `RadishFlow Studio`、`development build`、`Local ready`、`Signed out`、`Settings` |
| Left Rail | 主按钮 `新建项目`；次级按钮 `打开项目`、`打开示例项目`；低权重链接 `创建 Mixer-Flash 小案例`、`创建 Heater-Flash 小案例` |
| Main Stage | 上半区 `最近项目` 列表；下半区 `示例项目` 列表；每行包含名称、路径 / 来源、package、状态 chip |
| Right Rail | `Client`、`Server`、`Device` 三个状态 section；每个 section 2-3 行摘要 |
| Bottom Messages | Messages 列表，包含 severity、domain、summary、action |

示例行建议：

- Recent：`Blank Project` / `binary-hydrocarbon-lite-v1` / `Modified`。
- Example：`Feed -> Cooler -> Flash Drum` / `methane, ethane` / `Ready`。
- Message：`PACKAGE` / `内置物性包缓存可用` / `View packages`。

## Workbench - Modeling

### 布局

- 顶部两层：第一层为窄导航栏，顺序为 `首页`、`文件`、`物性`、`流程图`、`设备`、`运行`、`结果`、`工具`、`设置`；第二层为上下文工具栏，显示当前工作区主路径动作。
- 左栏：`项目` 与 `放置` 两个主 tab。项目 tab 负责 package / components；放置 tab 负责当前内置单元。
- 中央 Canvas：主舞台，显示单元、流股、端口状态和受控 suggestion。
- 右栏 Inspector：当前 stream / unit 的字段、端口和关联动作。
- 底部左右分栏：左侧 tabs 为 `消息`、`运行日志`、`收敛`、`建议`、`诊断`；右侧先只显示 `状态汇总`。

### 必须覆盖的对象

- Feed source stream：T / P / F / composition。
- Cooler / Valve intermediate stream：outlet T / P 或 P。
- Mixer：两个 inlet、outlet pressure。
- Flash Drum：flash temperature / pressure、liquid / vapor outlets。

### 视觉和交互原则

- Canvas 占主面积，侧栏服务于建模，不抢主舞台。
- suggestion 用低噪声状态，不把长说明放在画布内。
- Inspector 字段必须显示单位、草稿状态、提交命令和约束提示。
- 运行状态和保存状态放在顶部，不散落到多个面板。

### 线框内容

| 区域 | 内容 |
| --- | --- |
| Top Navigation | `首页`、`文件`、`物性`、`流程图`、`设备`、`运行`、`结果`、`工具`、`设置`；当前可高亮 `流程图` 或 `设备` |
| Context Toolbar | 项目名 `Blank Project`、保存状态 `Unsaved`、`检查输入`、`运行`、`保存`、run status `Ready to run` |
| Left Rail | Tabs：`项目` / `放置`；项目 tab 显示 package、components；放置 tab 显示 Feed、Mixer、Cooler、Valve、Flash Drum |
| Main Stage | Canvas 网格；单元块 Feed、Cooler、Flash Drum；流股线 `stream-feed-1-outlet`、`stream-cooler-1-outlet`、liquid / vapor outlets；suggestion chip |
| Right Rail | Inspector target：`Unit cooler-1`；字段 `outlet_temperature_k`、`outlet_pressure_pa`；Ports；Actions |
| Bottom Left | `消息` tab active；短行显示最近提交、suggestion 接受、保存状态 |
| Bottom Right | `状态汇总`：case `Ready`、latest run `Not run`、convergence `N/A`、diagnostics `0` |

Canvas 示例结构：

```text
Feed ── stream-feed-1-outlet ── Cooler ── stream-cooler-1-outlet ── Flash Drum
                                                           ├─ stream-flash-1-liquid
                                                           └─ stream-flash-1-vapor
```

Inspector 字段行应包含：

| 字段 | 显示 |
| --- | --- |
| `outlet_temperature_k` | label、input、unit `K`、state `Draft` / `Synced`、commit icon |
| `outlet_pressure_pa` | label、input、unit `Pa`、constraint `<= inlet pressure`、commit icon |
| `Ports` | inlet connected、outlet connected |
| `Related` | latest diagnostics empty、result unavailable before run |

## Workbench - Readiness

### 场景

用户点击 `运行`，但缺少确定的建模输入：

- 缺项目组分。
- 缺 Feed composition。
- 缺 Feed source T / P / F。
- 缺必要单元参数。
- composition 未归一。

### 表达

- 顶部运行状态显示 pending / blocked。
- 右侧 Inspector 聚焦真实缺口对象，例如 `flash-1` 或 `stream-feed-1-outlet`。
- 底部 Messages 显示 “模型输入未完成” 的短 notice。
- 字段区显示可提交的 draft / displayed default，而不是只给文本提示。

### 边界

不把结构性连接、拓扑、非法旧项目或求解阶段参数失败画成 readiness。它们继续走正式 Run Panel 诊断 / recovery。

### 线框内容

| 区域 | 内容 |
| --- | --- |
| Context Toolbar | run status `Blocked`；primary action 保持 `运行`，状态摘要说明 `Modeling inputs not ready` |
| Left Rail | 保持当前项目 / 放置上下文，不跳转到 checklist |
| Main Stage | Canvas 中目标对象显示 attention outline，例如 `flash-1` |
| Right Rail | Inspector 聚焦 `Unit flash-1`；缺失字段 `outlet_temperature_k` 显示可提交 displayed default |
| Bottom Left | `诊断` 或 `消息` active；notice 标题 `模型输入未完成`；detail 指向 `Flash Drum 出口温度` |
| Bottom Right | `状态汇总`：case `Blocked`、latest run `N/A`、convergence `N/A`、blocking inputs `1` |

Readiness notice 线框文案只保留短句：

```text
模型输入未完成
Flash Drum flash-1 需要提交出口温度。
```

字段提交后，frame 注释应说明下一状态推进到 `outlet_pressure_pa`，但不需要再画一个额外 frame。

## Workbench - Results

### 运行成功

运行收敛后：

- 顶部运行状态显示 converged，pending reason 清空。
- 右栏切到 Result Inspector 或在 Inspector 中提供清晰结果 tab。
- 底部默认打开 Results Table。
- Results commands 能聚焦 stream / unit。
- 轻量导出入口只消费当前 revision 的 latest `SolveSnapshot`。

### 结果审阅对象

必须能扫读：

- source streams。
- intermediate streams。
- terminal streams。
- unit consumed / produced streams。
- Flash liquid / vapor outlet material balance。
- phase region、bubble / dew window、overall enthalpy。

### 旧结果失效

编辑文档后：

- latest result 区域不继续渲染旧结果。
- stale notice 显示旧 snapshot 和当前 document revision 已不一致。
- 主动作指向 rerun。
- 旧结果不驱动导出或 Result Inspector。

### 线框内容

| 区域 | 内容 |
| --- | --- |
| Top Navigation / Toolbar | `结果` 导航高亮；toolbar 显示 `结果汇总`、`流股表`、`单元结果`、`导出快照`、`重新运行` |
| Left Rail | 项目对象列表可继续导航；选中 stream / unit 同步 Canvas focus |
| Main Stage | Canvas 显示结果 badge：source、intermediate、terminal；不把完整结果数字塞进画布 |
| Right Rail | Result Inspector；stream selector、unit selector、summary rows、phase rows |
| Bottom Left | `收敛` 或结果表相关 tab active；展示 solver、execution steps、diagnostics 摘要 |
| Bottom Right | `状态汇总` 显示 case `Converged`、latest run `Success`、convergence、执行步数、诊断数 |

Result Inspector 示例内容：

| 区块 | 内容 |
| --- | --- |
| Selected stream | `stream-cooler-1-outlet` |
| Summary | `T K`、`P Pa`、`F mol/s`、`H J/mol` |
| Composition | methane、ethane |
| Phase | overall、liquid / vapor when present |
| Unit references | consumed by `flash-1`、produced by `cooler-1` |

Stale notice 可作为 Results frame 内的右栏局部 variant 标注，不需要新增主 frame：

```text
结果已过期
当前项目已编辑，请重新运行以查看最新结果。
```

## 命令与状态映射

| 用户动作 | 命令 / 模型边界 |
| --- | --- |
| 新建项目 | app / shell project command，不依赖登录 |
| 放置单元 | canvas placement command |
| 接受 suggestion | local rules suggestion -> document command |
| 编辑字段 | inspector draft update |
| 提交字段 | inspector draft commit command |
| 运行 | `run_panel.run_manual` -> shared readiness -> solve dispatch |
| 保存 | file save command |
| 结果聚焦 | result command / inspector focus command |
| 导出当前快照 | latest current-revision `SolveSnapshot` export |

## 评审检查

创建 `.pen` 前先确认：

- Home、Modeling、Readiness、Results 四个 frame 是否覆盖当前 P0 主路径。
- 每个 visible 状态是否有明确来源。
- 是否避免画出当前暂不纳入项。
- 是否能从 frame 直接看出下一步动作。
- 是否没有把解释文字堆进 Canvas 主舞台。
- 是否保留足够空间给后续单元模块 UI 设计稿对齐。
