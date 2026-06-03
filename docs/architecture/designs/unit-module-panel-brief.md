# Unit Module Panel Design Brief

更新时间：2026-06-03

## 用途

用途：为 P1 `unit-module-panel.pen` 信息架构稿提供可评审的文字 brief，统一当前可编辑单元的参数、端口、结果、诊断和帮助入口界面规则。
读者：准备绘制或评审 `docs/architecture/designs/unit-module-panel.pen` 的设计协作者。
不包含：`.pen` 设计稿本体、UI 实现代码、完整参数表、完整报表系统、自由连线编辑器或单元高级模型设计。

本 brief 对应 `docs/architecture/studio-ui-topic-plan.md` 中的 P1 单元模块 UI 端点。P1 必须沿用 P0 `studio-client-main.pen` 的 Inspector / Result / Diagnostics 区域职责，不为每个单元另造一套临时面板。

## 设计目标

`unit-module-panel.pen` 第一版只解决单元模块 UI 的统一界面规则：

- 用户能在同一套面板结构中理解 Feed、Mixer、Heater / Cooler、Valve、Flash Drum 的输入、端口和结果。
- 参数字段必须显示单位、当前值来源、草稿状态、提交动作和必要约束提示。
- 端口区必须区分已连接、未连接、suggested、diagnostic attention。
- 结果区只读消费当前 revision 的 latest `SolveSnapshot`，并区分 consumed streams、produced streams 和 terminal outlets。
- 诊断区必须能指向相关 stream、unit、port 或 recovery action。
- 帮助入口只解释当前字段和主路径，不放长篇教程或未来功能说明。

## 设计画板建议

首版 `.pen` 建议包含 4 个主 frame：

| Frame | 建议尺寸 | 目的 |
| --- | ---: | --- |
| `Unit Panel - Shared Anatomy` | 1440 x 960 | 统一单元模块面板结构：header、parameters、ports、results、diagnostics、help |
| `Unit Panel - Feed and Mixer` | 1440 x 960 | 覆盖 Feed source 输入、composition、Mixer 双入口和 outlet pressure |
| `Unit Panel - Cooler Valve` | 1440 x 960 | 覆盖 Heater / Cooler outlet T/P、Valve outlet P 和中间流股结果 |
| `Unit Panel - Flash Drum` | 1440 x 960 | 覆盖 Flash temperature / pressure、liquid / vapor outlets、phase split 结果和 readiness 缺口 |

P1 不画 Home、完整 Workbench、控制面、移动端或完整报表。

## 统一面板结构

P1 单元模块 UI 使用同一套纵向结构，供 P0 Workbench 右侧 Inspector / Result 区域承载：

| 区域 | 职责 | 不承担 |
| --- | --- | --- |
| Unit Header | 单元名称、类型、状态、当前选择、主动作 | 全局运行按钮、项目保存、完整命令面板 |
| Parameters | 当前单元的可编辑输入、单位、草稿状态、提交动作、约束提示 | 完整高级参数表、未来模型选项 |
| Ports | inlet / outlet、连接对象、suggestion、diagnostic attention | 自由端口选择器、任意连线编辑 |
| Results | 当前 `SolveSnapshot` 中与该单元相关的 consumed / produced streams 和关键结果 | 历史快照比较、跨快照报表、shell 私有缓存 |
| Diagnostics | readiness 或正式 solve diagnostics 的关联摘要和跳转动作 | 原始 trace 常驻、开发态内部状态墙 |
| Help | 当前字段和主路径的短说明 | 教程长文、未来功能说明、完整物理模型教材 |

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

## 暂不纳入

- 不画完整参数表或高级模型配置。
- 不画自由连线、任意端口选择器、自动布线或完整拖拽布局。
- 不画完整报表、跨快照报表、模板导出、打印或批量导出。
- 不扩第三方物性包、完整组分数据库或 CAPE-OPEN / COM 语义。
- 不把每个单元做成互相无关的独立 UI 风格。

## 评审检查

创建或评审 `.pen` 时确认：

- 四个 frame 是否覆盖当前可编辑单元和共享面板结构。
- Feed、Mixer、Heater / Cooler、Valve、Flash Drum 是否共用同一套参数 / 端口 / 结果 / 诊断表达。
- 参数字段是否能看出单位、值来源、草稿状态和提交动作。
- 端口区是否能区分 connected、missing、suggested、attention。
- 结果区是否只消费当前 `SolveSnapshot`，并有 stale / unavailable 状态。
- 设计稿是否没有提前画出暂不纳入项。
