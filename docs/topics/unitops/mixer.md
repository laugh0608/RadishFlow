# Mixer 混合器专题

更新时间：2026-09-12

> 本文定义该专题的能力、开发范围与验收要求；具体迭代切片和优先级以 [当前状态](../../status/current.md) 为准。

## 用途

用途：定义当前 MVP 中 `Mixer` 的双入口、出口压力、混合结果、连接建议和诊断边界。
读者：负责 `rf-unitops`、`rf-flowsheet`、Studio Canvas suggestion、模块设置和结果审阅的开发者、用户、AI / Agent。
不包含：多入口任意混合器、反应混合、能量损失模型、压降网络和复杂管网求解。

## 专题层级

- 层级：二级功能
- 父专题：`docs/topics/flowsheet-modeling-and-solve.md`
- 关联专题：`docs/topics/modeling/material-stream.md`、`docs/topics/results-review-diagnostics.md`

## 专题目标

- 用户能在普通空白项目中搭建 `Feed + Feed -> Mixer -> Flash Drum`。
- 当前 Mixer 只覆盖 canonical `inlet_a / inlet_b / outlet`，并用显式 outlet pressure 作为必要参数。
- 连接建议不能在多来源场景中静默猜测入口。

## 当前实现快照

已完成：

- `Mixer(inlet_a/inlet_b/outlet)` canonical material ports。
- Unit Inspector 覆盖 `outlet_pressure_pa`。
- outlet pressure 高于两股 inlet pressure 较低值时停留在 invalid draft。
- Mixer suggestion 只在 source-only stream 数量与未绑定 inlet 数量一致时生成。
- `Feed + Feed -> Mixer -> Flash Drum` 已覆盖显式输入、保存 / 重开 / rerun 和结果审阅。

已知缺口：

- 当前不支持任意数量入口。
- 不做反应、热损失或复杂压降网络。
- 当前出口温度按摩尔流量加权入口温度，未通过总焓平衡求解；不能把两股流量与组成守恒、结果透传一致解释为绝热混合能量守恒。数值真相源见 [热力学模型](../../thermo/mvp-model.md#单元近似与能量解释)。

## 范围

本专题纳入：

- 两股物料混合。
- outlet pressure 参数。
- 双入口连接 completeness。
- Mixer outlet 作为 downstream flash consumed stream。
- 混合结果的 stream / unit 审阅。

本专题不纳入：

- 任意 N 入口混合器。
- 化学反应或相间传质模型。
- 管网压降、泵、压缩机或能量设备。
- 自动入口选择器。

## 设计边界

### 数据与状态

- inlet 绑定由 flowsheet connection 表达。
- outlet pressure 进入 `UnitOperationParameters`。
- Mixer outlet result 来自 solver step produced stream。

### 命令与接口

- 连接继续走受控 Canvas suggestion / DocumentCommand。
- 参数提交走正式 unit parameter command。
- 入口不足或参数缺失由 readiness / Run Panel 诊断定位。

### UI 与交互

- Mixer 放置入口在左侧 `模块` 的汇合与分离分类。
- 入口连接不通过自由端口选择器完成。
- 结果审阅通过右侧模块结果和底部结果表。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 当前双入口 Mixer 冻结 | Mixer-Flash 主路径可运行、保存 / 重开 / rerun |
| M2 | 连接建议复核 | 多来源场景不静默猜测入口 |
| M3 | 任意入口评估 | 若推进 N 入口 Mixer，先开新专题 |

## 验收标准

- 两个 inlet 都有明确来源后才进入可运行路径。
- outlet pressure 约束不被忽略。
- Mixer outlet、Flash inlet consumed stream 和结果表数值一致。
- 保存 / 重开后连接和参数稳定。

## 验证计划

- focused test：Mixer-Flash happy path、缺 inlet、pressure invalid draft、多来源 suggestion、保存 / 重开 / rerun、结果一致性。
- 仓库级：涉及 Mixer 或 connection 语义时执行 `./scripts/check-repo.sh`。

## 状态记录

- 当前状态：Active
- 最近更新：2026-09-12，恢复开发状态并对齐当前迭代入口；既有能力仍以实现快照和验收记录为准。
- 下一步：在双入口 Mixer 基线上明确混合计算的物性与能量要求，再实施对应模型与回归。
