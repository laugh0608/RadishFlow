# Flash Drum 闪蒸罐专题

更新时间：2026-09-12

> 本文定义该专题的能力、开发范围与验收要求；具体迭代切片和优先级以 [当前状态](../../status/current.md) 为准。

## 用途

用途：定义当前 MVP 中 `Flash Drum` 的 flash 参数、端口、相态结果、单相缺席语义和结果审阅边界。
读者：负责 `rf-flash`、`rf-unitops`、`rf-solver`、Studio 结果审阅和诊断的开发者、用户、AI / Agent。
不包含：严格气液分离器设计、液位控制、动态 holdup、压降模型、夹带、效率、三相闪蒸和完整相平衡数据库。

## 专题层级

- 层级：二级功能
- 父专题：`docs/topics/flowsheet-modeling-and-solve.md`
- 关联专题：`docs/topics/results-review-diagnostics.md`、`docs/topics/property-basis-and-components.md`

## 专题目标

- 用户能在 `Feed -> Flash Drum` 及下游流程中显式提交 flash temperature / pressure 并运行。
- `Flash Drum` 是当前 MVP 的核心相平衡单元，负责把 inlet stream 按 `TP Flash` 结果分到 liquid / vapor outlet。
- 单相情况下的零流量对侧 outlet 缺席语义必须在结果审阅中可判断。

## 当前实现快照

已完成：

- `Flash Drum(inlet/liquid/vapor)` canonical material ports。
- Unit Inspector 覆盖 `outlet_temperature_k` 和 `outlet_pressure_pa`，语义为 flash T/P。
- 参数提交同步 liquid / vapor outlet 模板。
- `TP Flash` 结果包含 overall / liquid / vapor 相态和 enthalpy。
- 单相 Flash 缺席语义已纳入通用小流程建模 v1 收口。

已知缺口：

- 当前不是完整 separator 设备模型。
- 不支持三相、液位、动态 holdup、设备效率或夹带。

## 范围

本专题纳入：

- flash T/P 参数。
- inlet、liquid outlet、vapor outlet。
- `TP Flash` 相态结果映射。
- 单相 / 两相结果审阅。
- 与 upstream stream 和 terminal stream result 的一致性。

本专题不纳入：

- 设备尺寸、液位、停留时间、压降。
- 三相闪蒸、盐 / 固相 / 电解质。
- 完整 EOS / 活度系数模型。
- 动态分离过程。

## 设计边界

### 数据与状态

- flash T/P 必须进入 `UnitOperationParameters`。
- liquid / vapor outlet 结果来自当前 `SolveSnapshot`，不由 UI 自行推断。
- 单相对侧 outlet 可缺席或零流量，但必须在结果审阅语义上可解释。

### 命令与接口

- 参数提交走正式 unit parameter command。
- 结果定位走 stream / unit focus command。
- flash 失败诊断走 Run Panel / solver diagnostic，不在 UI 层吞掉错误。

### UI 与交互

- Flash Drum 放置入口在左侧 `模块` 的汇合与分离分类。
- 参数编辑在右侧 `模块设置`。
- liquid / vapor result 通过结果表、Result Inspector 和模块结果审阅。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 当前 TP Flash 单元冻结 | `Feed -> Flash Drum` 与上游调节单元路径可稳定运行 |
| M2 | 单相 / 两相审阅复核 | 单相缺席语义和两相 split 结果可明确判断 |
| M3 | 高级分离器评估 | 若推进设备模型或三相 flash，先开新专题 |

## 验收标准

- flash T/P 未提交时不能被误判为可运行。
- 两相结果能审阅 liquid / vapor outlet。
- 单相结果不会制造伪流量或误导 downstream。
- 保存 / 重开 / rerun 后结果入口一致。

## 验证计划

- focused test：Feed-Flash happy path、参数缺失、单相 flash、两相 flash、terminal stream result、保存 / 重开 / rerun。
- 仓库级：涉及 flash、solver 或结果语义时执行 `./scripts/check-repo.sh`。

## 状态记录

- 当前状态：Active
- 最近更新：2026-09-12，恢复开发状态并对齐当前迭代入口；既有能力仍以实现快照和验收记录为准。
- 下一步：按目标体系完善 TP Flash 的适用范围与独立数值基准，并验证结果审阅路径。
