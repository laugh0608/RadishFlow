# Unit Module Panel Design Brief

更新时间：2026-06-03

## 用途

用途：为 P1 `unit-module-panel.pen` 信息架构稿提供可评审的文字 brief，统一 Workbench 右侧 Inspector / Results / Diagnostics 面板在不同单元选中态下的参数、端口、结果和诊断表达。
读者：准备绘制或评审 `docs/architecture/designs/unit-module-panel.pen` 的设计协作者。
不包含：`.pen` 设计稿本体、UI 实现代码、独立单元详情页、完整参数表、完整报表系统、自由连线编辑器或单元高级模型设计。

本 brief 对应 `docs/architecture/studio-ui-topic-plan.md` 中的 P1 单元模块 UI 端点。P1 必须沿用 P0 `studio-client-main.pen` 的 Workbench 布局和 Inspector / Result / Diagnostics 区域职责，不为每个单元另造一整页参数面板，也不把单元模块 UI 做成脱离 Canvas 的独立页面。

## 设计目标

`unit-module-panel.pen` 第一版只解决 Workbench 中右侧面板如何适配不同单元：

- 用户始终停留在同一个 Workbench 中，通过 Canvas 选择不同单元，并在右侧 Inspector / Results / Diagnostics 中查看对应内容。
- Feed、Mixer、Heater / Cooler、Valve、Flash Drum 共用同一套右侧面板结构，不出现互相独立的一整页单元参数页。
- 参数字段必须显示单位、当前值来源、草稿状态、提交动作和必要约束提示。
- 端口区必须区分已连接、未连接、suggested、diagnostic attention。
- 结果区只读消费当前 revision 的 latest `SolveSnapshot`，并区分 consumed streams、produced streams 和 terminal outlets。
- 诊断区必须能指向相关 stream、unit、port 或 recovery action。
- 底部 Results Table / Diagnostics 与右侧面板保持协同，不把结果或诊断塞进单元独立页面。

## 设计画板建议

首版 `.pen` 包含 4 个主 frame，全部使用完整 Workbench 壳，只改变当前选择对象和右侧面板内容：

| Frame | 建议尺寸 | 目的 |
| --- | ---: | --- |
| `Workbench 变体 - Heater Inspector` | 1440 x 960 | 对照基线工作台，展示选中 Heater / Cooler 时右侧 Inspector 的紧凑参数、结果和状态 |
| `Workbench 变体 - Feed / Mixer Inspector` | 1440 x 960 | 同一 Workbench 中切换到 Feed / Mixer 的 Inspector 变体，覆盖 source 输入、composition、双入口和 outlet pressure |
| `Workbench 变体 - Flash Drum Inspector` | 1440 x 960 | 同一 Workbench 中切换到 Flash Drum 的 Inspector 变体，覆盖 flash T/P、liquid / vapor outlets 和 readiness 缺口 |
| `Workbench 变体 - Readiness / Diagnostics` | 1440 x 960 | 表达 readiness、正式 Run Panel diagnostics、stale result 在右侧面板、底部左侧诊断区和右侧状态汇总中的关系 |

P1 不画 Home、控制面、移动端、独立单元详情页或完整报表。P1 的视觉方向应贴近 `docs/architecture/assets/studio-ui/baseline/radishflow-workbench-concept.png`，但 Workbench 壳必须采用 P0 最新修正：顶部窄导航栏 + 上下文工具栏，导航顺序为 `首页`、`文件`、`物性`、`流程图`、`设备`、`运行`、`结果`、`工具`、`设置`；底部为左侧运行 / 消息 tabs 与右侧 `状态汇总` 分栏。

## 统一面板结构

P1 单元模块 UI 使用 P0 Workbench 的右侧面板作为承载面。不同单元只切换右侧面板内容，不改变主工作台结构：

| 区域 | 职责 | 不承担 |
| --- | --- | --- |
| Workbench Shell | 顶部导航 / 上下文工具栏、左侧项目树 / 放置入口、中间 Canvas、右侧 Inspector / Results、底部左右分栏和状态栏的稳定壳 | 为单元切换重建整页布局 |
| Inspector Header | 单元名称、类型、状态、当前选择 | 全局运行按钮、项目保存、完整命令面板 |
| Specifications / Parameters | 当前单元可编辑输入、单位、草稿状态、提交动作、约束提示 | 完整高级参数表、未来模型选项 |
| Ports | inlet / outlet、连接对象、suggestion、diagnostic attention | 自由端口选择器、任意连线编辑 |
| Results | 当前 `SolveSnapshot` 中与该单元相关的关键结果；底部 Results Table 继续承担表格扫读 | 历史快照比较、跨快照报表、shell 私有缓存 |
| Diagnostics | readiness 或正式 solve diagnostics 的关联摘要和跳转动作；底部 Diagnostics 继续承担列表扫读 | 原始 trace 常驻、开发态内部状态墙 |

## 细节结构要求

P1 信息架构稿必须避免用长文本和空格模拟真实控件：

- 顶部使用两层 Workbench 导航：窄导航栏前置 `物性`，独立 `设置` 承载语言、单位集和偏好；下方上下文工具栏随当前导航显示项目名、保存、检查输入、运行和当前状态。
- 左侧 Project tree 使用稳定分组、计数 badge、对象行和选中态，不使用纯文本项目符号清单。
- 左侧栏只保留 `项目` 与 `放置` 两个 tab：`项目` 展示当前项目物性包、组分、流股和模块；`放置` 展示流股和单元模块 palette。
- 右侧 Inspector 的 `General`、`Specifications`、`Ports`、`Results` 使用稳定行结构，字段名、值、单位和状态分列表达。
- 底部左侧分 tabs：`消息`、`运行日志`、`收敛`、`建议`、`诊断`；其中结果表或诊断列表必须使用明确列结构，不通过空格对齐。
- 底部右侧 `状态汇总` 显示当前案例总状态、最新运行、收敛状态、迭代次数或 `N/A`、执行步数、诊断数、snapshot / revision 一致性；不得伪造现有模型没有提供的数据。
- Diagnostics notice 只出现在诊断、readiness 或 stale 状态中，不覆盖结果表或状态汇总。
- 底部状态栏按左侧状态 / 中间模式 / 右侧缩放与选择分区，不靠空格堆在同一文本块里。

## 覆盖对象

### Feed

参数重点：

- `source_temperature_k`
- `source_pressure_pa`
- `molar_flow_mol_s`
- `composition`

端口重点：

- outlet material stream。

结果重点：

- source stream 的 T / P / F / composition / phase summary。

### Mixer

参数重点：

- `inlet_a`
- `inlet_b`
- `outlet_pressure_pa`

端口重点：

- 两个 inlet 与一个 outlet 必须清楚区分 connected / missing / diagnostic attention。

结果重点：

- consumed streams、mixed outlet、weighted outlet result。

### Heater / Cooler

参数重点：

- inlet stream。
- `outlet_temperature_k`
- `outlet_pressure_pa`

端口重点：

- inlet、outlet、intermediate stream result。

结果重点：

- produced outlet stream、temperature / pressure change、相关 unit step。

### Valve

参数重点：

- inlet stream。
- `outlet_pressure_pa`

端口重点：

- inlet、outlet、throttled stream result。

结果重点：

- produced outlet stream、pressure drop、phase summary。

### Flash Drum

参数重点：

- inlet stream。
- `flash_temperature_k`
- `flash_pressure_pa`

端口重点：

- inlet、liquid outlet、vapor outlet。

结果重点：

- liquid / vapor outlet material balance。
- phase region、bubble / dew window、overall enthalpy。
- 单相 Flash 中零流量 outlet 的缺席语义。

## 状态表达

参数字段状态：

| 状态 | 含义 |
| --- | --- |
| `Draft` | 用户已编辑但尚未提交 |
| `Displayed default` | Inspector 显示默认 / 模板值，可直接提交为正式参数 |
| `Synced` | 当前字段已写入正式参数且与文档同步 |
| `Invalid` | 字段值越界或组成未归一 |
| `Blocked` | 缺少运行前确定建模输入 |

端口状态：

| 状态 | 含义 |
| --- | --- |
| `Connected` | 已连接到具体 stream / unit |
| `Missing` | 必需连接缺失 |
| `Suggested` | 有受控 suggestion 可接受 |
| `Attention` | 连接存在诊断或 recovery 入口 |

结果状态：

| 状态 | 含义 |
| --- | --- |
| `Unavailable before run` | 尚无当前 revision 的 `SolveSnapshot` |
| `Current` | 来自 latest current-revision `SolveSnapshot` |
| `Stale` | 旧快照与当前 document revision 不一致，只显示 rerun 提示 |

## 命令与状态来源

| UI 状态 / 动作 | 来源 |
| --- | --- |
| 单元名称、类型、端口 | canvas presentation / flowsheet model |
| 参数字段、草稿、提交 | inspector draft / command surface |
| 建模输入缺口 | shared modeling readiness |
| 结构性连接、拓扑、求解失败 | formal solve diagnostics / Run Panel recovery |
| 当前结果 | latest current-revision `SolveSnapshot` |
| stale notice | stale snapshot presentation |
| 帮助入口 | 当前字段 / 主路径短说明，不成为状态真相源 |

## 实现前拆解

P1 设计稿进入代码实现前，先按现有边界拆成小切片，不直接把新视觉一次性堆进 `runtime.rs`：

| 切片 | 目标 | 主要落点 |
| --- | --- | --- |
| Workbench shell 分区 | 顶部导航 / 工具栏、左侧 Project / Palette、中间 Canvas、右侧 tabs、底部左右分栏与状态栏先形成稳定容器 | `apps/radishflow-studio/src/studio_gui_shell/panels/` |
| Active Inspector 结构化行 | `General`、`Specifications`、`Ports`、`Results`、`Diagnostics` 使用统一行结构和状态 chip | `StudioGuiWindowInspectorTargetDetailModel`、`studio_gui_shell/panels/runtime.rs` 的 inspector 渲染函数 |
| Bottom split | 底部左侧 tabs 与右侧 `状态汇总` 分开，状态汇总只消费正式状态 / snapshot | Run Panel、readiness、diagnostics、`SolveSnapshot` presentation |
| Results Table | 左侧底部结果 / 诊断表按列渲染，不靠字符串空格排版 | 现有 bottom results table 渲染与 `SolveSnapshot` presentation |
| Diagnostics notice | readiness / stale / formal diagnostics 只在对应区域呈现，不覆盖结果表或状态汇总 | modeling readiness、Run Panel diagnostics、bottom drawer |
| 渲染文件拆分 | `runtime.rs` 已明显过大，实现 P1 前应先按 inspector / result / diagnostics 或 bottom drawer 拆分子模块 | `apps/radishflow-studio/src/studio_gui_shell/panels/` |

实现顺序建议：

1. 先拆分渲染职责，保持行为不变，并用现有 UI rendering tests 验证。
2. 再调整底部 Results Table 和状态栏结构，因为这两处主要是渲染结构问题，风险相对低。
3. 再调整 Active Inspector 的字段、端口和最新结果区，必要时补 presentation 字段；新增字段必须来自 `StudioGuiWindowInspectorTargetDetailModel` 或正式 snapshot，不做 shell 私有状态。
4. 最后处理 Diagnostics / stale notice 的版式协同，继续复用 shared modeling readiness、formal solve diagnostics 和 stale snapshot presentation。

## 暂不纳入

- 不画独立单元详情页、完整参数表或高级模型配置。
- 不画自由连线、任意端口选择器、自动布线或完整拖拽布局。
- 不画完整报表、跨快照报表、模板导出、打印或批量导出。
- 不扩第三方物性包、完整组分数据库或 CAPE-OPEN / COM 语义。
- 不把每个单元做成互相无关的独立 UI 风格，也不让每个参数面板占据整页。

## 评审检查

创建或评审 `.pen` 时确认：

- 四个 frame 是否都保留完整 Workbench 壳，并只改变当前选择对象与右侧面板内容。
- Feed、Mixer、Heater / Cooler、Valve、Flash Drum 是否共用同一套 Inspector / Results / Diagnostics 表达。
- 参数字段是否能看出单位、值来源、草稿状态和提交动作。
- 端口区是否能区分 connected、missing、suggested、attention。
- 结果区是否只消费当前 `SolveSnapshot`，并有 stale / unavailable 状态。
- 底部 Results Table / Diagnostics 是否继续承担横向扫读，而不是被单元独立页替代。
- 设计稿是否没有提前画出暂不纳入项。
