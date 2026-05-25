# Author Small Cases

更新时间：2026-05-25

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
2. 创建一个 MVP 默认空白项目。
3. 进入 Workbench 并切到左侧 `放置` 面板。
4. 在 `放置` 面板显示对应小案例任务清单。

它不会自动生成 flowsheet，不会替用户放置单元，不会写 `FlowsheetDocument`，也不会进入 undo。后续仍需要用户按现有 `放置 -> suggestion -> 单元参数 -> 运行 -> 保存 -> 结果审阅` 工作流完成案例。

## 任务清单如何理解

任务清单是只读提示层。每个任务的完成状态来自当前 canvas view：

- 当前已存在的 unit kind，例如 `feed`、`mixer`、`heater`、`flash_drum`
- 当前 material stream 的 source / sink 端点绑定
- 当前是否已有最新 `SolveSnapshot`

清单不会成为真相源，也不会反向修正文档。如果清单显示未完成，应回到 Canvas suggestion、Inspector 参数或运行结果中完成正式操作。

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
2. 在 `放置` 面板依次放置两个 `Feed`、一个 `Mixer`、一个 `Flash Drum`。
3. 为两个 `Feed` 分别接受 `Create stream` suggestion，创建出口流股。
4. 为 `Mixer` 接受两个 `Connect stream` suggestion，把两个 Feed outlet 接到 `inlet_a / inlet_b`。
5. 接受 `Mixer` 的 `Create stream` suggestion，创建 mixer outlet。
6. 为 `Flash Drum` 接受 `Connect stream` suggestion，把 mixer outlet 接到 flash inlet。
7. 接受 `Flash Drum` 的 liquid / vapor 两个 outlet suggestion。
8. 在右侧 `检查器` 中提交一组 SI 参数：
   - `Feed 1` source temperature = `305 K`，source pressure = `130000 Pa`
   - `Feed 2` source temperature = `315 K`，source pressure = `120000 Pa`
   - `Mixer` outlet pressure = `90000 Pa`
   - `Flash Drum` flash temperature = `300 K`，flash pressure = `85000 Pa`
9. 点击顶部 `运行`。
10. 在右侧 `结果` 和底部 `结果表` 检查收敛结果。
11. 保存项目，重开后再次运行，确认结果仍可复现。
12. 需要交付文本结果时，在右侧 `结果` 区复制当前 `SolveSnapshot` 或导出 `.txt`。

当前可用的最小核对点：

- `stream-mixer-1-outlet` 总摩尔流量应为两股 Feed 入口之和。
- `Flash Drum` 应有 liquid / vapor 两个出口结果。
- 保存重开后，unit / stream / port 绑定和已提交单元参数应保持。

## Heater-Flash 小案例

目标流程：

```text
Feed -> Heater.inlet
Heater.outlet -> Flash Drum.inlet
Flash Drum -> liquid / vapor
```

建议步骤：

1. 在 Home 点击 `创建 Heater-Flash 小案例`。
2. 在 `放置` 面板依次放置一个 `Feed`、一个 `Heater`、一个 `Flash Drum`。
3. 为 `Feed` 接受 `Create stream` suggestion，创建出口流股。
4. 为 `Heater` 接受 `Connect stream` suggestion，把 Feed outlet 接到 heater inlet。
5. 接受 `Heater` 的 `Create stream` suggestion，创建 heater outlet。
6. 为 `Flash Drum` 接受 `Connect stream` suggestion，把 heater outlet 接到 flash inlet。
7. 接受 `Flash Drum` 的 liquid / vapor 两个 outlet suggestion。
8. 在右侧 `检查器` 中提交一组 SI 参数：
   - `Feed` source temperature / source pressure
   - `Heater` outlet temperature / outlet pressure
   - `Flash Drum` flash temperature / flash pressure
9. 点击顶部 `运行`。
10. 在右侧 `结果` 中先看 heater outlet，再看 flash liquid / vapor outlet。
11. 保存项目，重开后再次运行。

当前可用的最小核对点：

- `Heater` outlet 结果应反映已提交的 outlet temperature / outlet pressure。
- `Flash Drum` 的 inlet 应消费 heater outlet，而不是 Feed source stream。
- 若 heater outlet pressure 高于已连接 inlet pressure，草稿会保持 invalid，不写回项目。

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
