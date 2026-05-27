# Author Small Cases

更新时间：2026-05-27

## 用途

用途：说明如何使用 Studio Home 的小案例作者入口，从空白项目手工复现当前 MVP β 的两条可求解建模路径。
读者：第一次跑通内置示例后，想自己从空白项目搭一个小流程的用户、测试人员和开发者。
不包含：项目向导设计、自动建模、自由连线、自动布线、完整参数表、完整报表系统或发布计划。

## 作者入口是什么

Home 左侧 `开始` 区当前提供两个小案例作者入口：

- `创建 Mixer-Flash 小案例`
- `创建 Heater-Flash 小案例`

点击后，Studio 会：

1. 若当前工作区有未保存更改，先进入继续 / 取消确认。
2. 创建一个未命名空白项目。
3. 进入 Workbench 并切到左侧 `放置` 面板。
4. 在 `放置` 面板显示对应小案例任务清单。

它不会自动生成 flowsheet，不会替用户放置单元，不会写 `FlowsheetDocument`，也不会进入 undo。后续仍需要用户按现有 `放置 -> suggestion -> 单元参数 -> 运行 -> 保存 -> 结果审阅` 工作流完成案例。

## 任务清单如何理解

任务清单是只读提示层。每个任务的完成状态来自当前 canvas view：

- 当前已存在的 unit kind，例如 `feed`、`mixer`、`heater`、`flash_drum`
- 当前 material stream 的 source / sink 端点绑定
- 当前是否已有最新 `SolveSnapshot`

清单不会成为真相源，也不会反向修正文档。如果清单显示未完成，应回到 Canvas suggestion、Inspector 参数或运行结果中完成正式操作。

## 建模输入前置项

空白项目不会预写默认物性包或默认组分。开始放置单元前，先完成项目级输入选择：

- 左侧 `项目` 面板直接显示当前物性包和 `项目组分`；未选择时会显示可选的内置项。
- 右侧 `物性包` tab 也提供同一受控组分选择入口和内置 package 选择入口。
- 选择 `binary-hydrocarbon-lite-v1` 会写入 `Flowsheet.thermo.property_package_id`。
- 选择 methane / ethane 会写入 `Flowsheet.components`；Stream Inspector 中的 composition 添加动作只从这份项目组分列表派生。
- Feed composition 的数值修改仍在选中对应 stream 后，通过右侧 `检查器` 的字段草稿、`Normalize composition` 和提交命令完成。

## β 第二刀验收口径

下面两条案例用于验收“建模输入能力 v0”，重点不是自动生成流程，而是确认用户能用受控输入能力复现 official demo case：

- 项目级输入必须显式选择 `binary-hydrocarbon-lite-v1`、methane、ethane。
- Feed composition 和 Unit 参数必须通过 Inspector draft / commit / normalize 进入项目。
- 保存 / 重开后再次运行必须收敛。
- 结果核对必须来自最新 `SolveSnapshot`，而不是读取静态示例文件。

## 结果核对与案例说明 v0

本节面向已经运行成功的用户，说明如何把结果从“有数值”核对到“路径、相态和焓值都可解释”。这里的 official demo case 指仓库内置示例文件：

- `examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json`
- `examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json`

Home 作者入口从空白项目复现时，流股 ID 会按当前 canvas 生成，例如 `stream-feed-1-outlet`、`stream-heater-1-outlet`、`stream-mixer-1-outlet`。这些 ID 不需要和 official demo 文件完全相同；核对时看同一类对象和同一条 `SolveSnapshot` 链路：

- `Streams`：关键输入流股、非 flash 中间流股、flash outlet 的 `T / P / F / H / composition / phases / bubble_dew_window`
- `Units` / `Steps`：每个单元实际消费和产出的流股引用
- `Flash Drum`：flash inlet 是否来自上游中间流股，以及 liquid / vapor outlet 的流量分割和相态
- `复制快照` / `导出文本`：应包含同一份快照里的流股、单元、步骤和诊断，不写项目文件，也不重新求解

## Mixer-Flash 小案例

目标流程：

```text
Feed 1 -> Mixer.inlet_a
Feed 2 -> Mixer.inlet_b
Mixer.outlet -> Flash Drum.inlet
Flash Drum -> liquid / vapor
```

建议步骤：

1. 在 Home 点击 `创建 Mixer-Flash 小案例`。
2. 在左侧 `项目` 或右侧 `物性包` tab 中选择 `binary-hydrocarbon-lite-v1` 与 methane / ethane。
3. 在 `放置` 面板依次放置两个 `Feed`、一个 `Mixer`、一个 `Flash Drum`。
4. 为两个 `Feed` 分别接受 `Create stream` suggestion，创建出口流股。
5. 为 `Mixer` 接受两个 `Connect stream` suggestion，把两个 Feed outlet 接到 `inlet_a / inlet_b`。
6. 接受 `Mixer` 的 `Create stream` suggestion，创建 mixer outlet。
7. 为 `Flash Drum` 接受 `Connect stream` suggestion，把 mixer outlet 接到 flash inlet。
8. 接受 `Flash Drum` 的 liquid / vapor 两个 outlet suggestion。
9. 在右侧 `检查器` 中提交这组 SI 输入：

| 对象 | 字段 | 输入 |
| --- | --- | ---: |
| 物性包 | package | `binary-hydrocarbon-lite-v1` |
| 项目组分 | components | methane, ethane |
| Feed 1 outlet composition | methane / ethane | draft `0.2 / 0.6`，Normalize 后 `0.25 / 0.75` |
| Feed 2 outlet composition | methane / ethane | draft `0.7 / 0.1`，Normalize 后 `0.875 / 0.125` |
| Feed 1 | source temperature / pressure | `305 K` / `130000 Pa` |
| Feed 2 | source temperature / pressure | `315 K` / `120000 Pa` |
| Mixer | outlet pressure | `90000 Pa` |
| Flash Drum | flash temperature / pressure | `300 K` / `85000 Pa` |

10. 点击顶部 `运行`。
11. 在右侧 `结果` 和底部 `结果表` 检查收敛结果；底部表应同时显示流股结果和单元最新步骤。
12. 保存项目，重开后再次运行，确认结果仍可复现。
13. 需要交付文本结果时，在右侧 `结果` 区复制当前 `SolveSnapshot` 或导出 `.txt`。

当前可用的最小核对点：

- `stream-mixer-1-outlet` 总摩尔流量应为两股 Feed 入口之和。
- `stream-mixer-1-outlet` methane / ethane 组成应为 `0.5625 / 0.4375`。
- `stream-mixer-1-outlet` pressure 应为 `90000 Pa`。
- flash liquid outlet 的 temperature / pressure 应为 `300 K` / `85000 Pa`。
- `Flash Drum` 应有 liquid / vapor 两个出口结果。
- 保存重开后，unit / stream / port 绑定和已提交单元参数应保持。

Official demo case 的结果核对路径：

| 核对对象 | 预期 |
| --- | --- |
| 输入流股 | `stream-feed-a` 为 `300 K / 650000 Pa / 2 mol/s / z=0.2 methane, 0.8 ethane`；`stream-feed-b` 为 `300 K / 650000 Pa / 3 mol/s / z=0.2 methane, 0.8 ethane` |
| Mixer step | `mixer-1` 消费 `stream-feed-a`、`stream-feed-b`，产出 `stream-mix-out` |
| 中间流股 | `stream-mix-out` 为 `300 K / 650000 Pa / 5 mol/s / z=0.2 methane, 0.8 ethane`，并带有已物化 `H` 与 `bubble_dew_window` |
| Flash step | `flash-1` 消费 `stream-mix-out`，产出 `stream-liquid`、`stream-vapor` |
| Flash 分割 | liquid 与 vapor 均为非零流量，二者总摩尔流量之和等于 `stream-mix-out` |
| 相态 / 焓值 | liquid outlet 应有 Liquid phase row，vapor outlet 应有 Vapor phase row；两股 flowing outlet 都应带 `H`，窗口 `phase_region` 为 two-phase |

## Heater-Flash 小案例

目标流程：

```text
Feed -> Heater.inlet
Heater.outlet -> Flash Drum.inlet
Flash Drum -> liquid / vapor
```

建议步骤：

1. 在 Home 点击 `创建 Heater-Flash 小案例`。
2. 在左侧 `项目` 或右侧 `物性包` tab 中选择 `binary-hydrocarbon-lite-v1` 与 methane / ethane。
3. 在 `放置` 面板依次放置一个 `Feed`、一个 `Heater`、一个 `Flash Drum`。
4. 为 `Feed` 接受 `Create stream` suggestion，创建出口流股。
5. 为 `Heater` 接受 `Connect stream` suggestion，把 Feed outlet 接到 heater inlet。
6. 接受 `Heater` 的 `Create stream` suggestion，创建 heater outlet。
7. 为 `Flash Drum` 接受 `Connect stream` suggestion，把 heater outlet 接到 flash inlet。
8. 接受 `Flash Drum` 的 liquid / vapor 两个 outlet suggestion。
9. 在右侧 `检查器` 中提交这组 SI 输入：

| 对象 | 字段 | 输入 |
| --- | --- | ---: |
| 物性包 | package | `binary-hydrocarbon-lite-v1` |
| 项目组分 | components | methane, ethane |
| Feed outlet composition | methane / ethane | draft `0.2 / 0.6`，Normalize 后 `0.25 / 0.75` |
| Feed | source temperature / pressure | `310 K` / `130000 Pa` |
| Heater | outlet temperature / pressure | `358.5 K` / `90000 Pa` |
| Flash Drum | flash temperature / pressure | `300 K` / `85000 Pa` |

10. 点击顶部 `运行`。
11. 在右侧 `结果` 中先看 heater outlet，再看 flash liquid / vapor outlet；底部 `结果表` 可同时核对 Heater 与 Flash Drum 的消费 / 产出流股。
12. 保存项目，重开后再次运行。

当前可用的最小核对点：

- Feed outlet 结果中 methane / ethane 组成应为 `0.25 / 0.75`，temperature / pressure 应为 `310 K` / `130000 Pa`。
- `Heater` outlet 结果应反映已提交的 outlet temperature / outlet pressure。
- `Flash Drum` 的 inlet 应消费 heater outlet，而不是 Feed source stream。
- flash liquid / vapor outlet 的 temperature / pressure 应为 `300 K` / `85000 Pa`，两股 outlet 的总摩尔流量之和应等于 Feed outlet 总摩尔流量。
- 若 heater outlet pressure 高于已连接 inlet pressure，草稿会保持 invalid，不写回项目。

Official demo case 的结果核对路径：

| 核对对象 | 预期 |
| --- | --- |
| 输入流股 | `stream-feed` 为 `300 K / 120000 Pa / 5 mol/s / z=0.35 methane, 0.65 ethane`，并带有已物化 `H` 与窗口 |
| Heater step | `heater-1` 消费 `stream-feed`，产出 `stream-heated` |
| 中间流股 | `stream-heated` 为 `345 K / 95000 Pa / 5 mol/s / z=0.35 methane, 0.65 ethane`，并带有已物化 `H` 与 `bubble_dew_window` |
| Flash step | `flash-1` 消费 `stream-heated`，产出 `stream-liquid`、`stream-vapor` |
| Flash 分割 | 当前 official hydrocarbon 条件为 vapor-only：`stream-liquid` 总摩尔流量为 `0`，`stream-vapor` 总摩尔流量等于 `stream-heated` |
| 相态 / 焓值 | `stream-liquid` 允许缺席 phase rows、`H` 和窗口；`stream-vapor` 应有 Vapor phase row、已物化 `H`，窗口 `phase_region` 为 vapor-only |

## 常见误解

- 作者入口不是自动建模系统：它只打开空白项目和清单。
- 任务清单不是文档语义：它只从当前 canvas / solve snapshot 推导状态。
- `Connect stream` / `Create stream` 仍是正式 suggestion action；接受后才会通过文档命令写回。
- 单元参数必须显式提交；仅在输入框中修改草稿不会改变运行结果。
- 结果复制 / 导出只消费当前 `SolveSnapshot`，不写项目，也不是完整报表系统。

## 相关文档

- `docs/guides/studio-quick-start.md`
- `docs/guides/run-first-flowsheet.md`
- `docs/guides/review-solve-results.md`
- `docs/architecture/canvas-interaction-contract.md`
