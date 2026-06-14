# Heater / Cooler 换热器专题

更新时间：2026-06-14

## 用途

用途：定义当前 MVP 中 `Heater` / `Cooler` 单元的建模、参数、求解、结果和诊断边界。  
读者：负责 `rf-unitops`、Studio 模块设置、流程图建模、结果审阅和测试的开发者、用户、AI / Agent。  
不包含：严格换热器设计、UA / 面积 / LMTD 计算、多股换热网络、能量流股、动态换热和详细设备选型。

## 专题层级

- 层级：二级功能
- 父专题：`docs/topics/flowsheet-modeling-and-solve.md`
- 关联专题：`docs/topics/modeling/material-stream.md`、`docs/topics/results-review-diagnostics.md`

## 专题目标

- 用户能在普通空白项目中搭建 `Feed -> Heater/Cooler -> Flash Drum`，显式提交 outlet temperature / outlet pressure 并运行。
- `Heater` / `Cooler` 当前作为受控调节单元，只表达目标出口 T/P，不提前变成完整换热器设备模型。
- 中间 outlet stream 必须作为正式结果对象进入 downstream flash 和结果审阅。

## 当前实现快照

已完成：

- `Heater` / `Cooler` 支持 inlet / outlet material path。
- Unit Inspector 覆盖 `outlet_temperature_k` 和 `outlet_pressure_pa`。
- outlet pressure 高于已连接 inlet pressure 时停留在 invalid draft。
- 提交参数会同步对应 outlet stream 模板。
- `Heater/Cooler -> Flash Drum` 路径已覆盖保存 / 重开 / rerun 与结果审阅。

已知缺口：

- 当前没有 UA、热负荷、能量流股或严格换热面积模型。
- `Cooler` / `Valve` 有 focused 覆盖，但不新增同构 Home 作者入口。

## 范围

本专题纳入：

- 目标 outlet T/P 参数。
- 参数约束、invalid draft 和求解阶段诊断。
- outlet stream 作为正式中间结果。
- 与 Flash Drum downstream 消费数值一致性。

本专题不纳入：

- 传热面积、传热系数、LMTD、pinch analysis。
- 能量流股和公用工程系统。
- 多股换热器、换热网络和设备 sizing。
- 动态换热过程。

## 设计边界

### 数据与状态

- 参数进入 `UnitOperationParameters`。
- outlet stream 模板只是显示和下游初始化来源，不代表未提交参数已经生效。
- 运行结果以当前 `SolveSnapshot` 为准。

### 命令与接口

- 参数提交走 `DocumentCommand::SetUnitParameter`。
- 结果定位走 `inspector.focus_unit:*` / `inspector.focus_stream:*`。
- readiness 只处理必要参数缺失；数值求解失败交给 Run Panel 诊断。

### UI 与交互

- 放置入口在左侧 `模块` 的调节单元分类。
- 参数编辑在右侧 `模块设置` / Inspector。
- 当前结果在右侧 `模块结果` 和底部结果表审阅。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 当前 T/P 调节模型冻结 | `Feed -> Heater/Cooler -> Flash Drum` 可运行、保存 / 重开 / rerun |
| M2 | 结果一致性复核 | outlet stream、unit step produced stream 和 flash consumed stream 同源 |
| M3 | 严格换热器评估 | 若推进 UA / duty / energy stream，先开新专题 |

## 验收标准

- 必须显式提交 outlet T/P 才算参数完成。
- outlet pressure 约束不被 UI fallback 掩盖。
- 中间 outlet stream 可在结果表和 Inspector 审阅。
- 保存 / 重开后参数与结果入口稳定。

## 验证计划

- focused test：Heater-Flash happy path、Cooler-Flash happy path、参数缺失、pressure invalid draft、保存 / 重开 / rerun、结果一致性。
- 仓库级：涉及核心单元或求解改动时执行 `./scripts/check-repo.sh`。

## 状态记录

- 当前状态：Active
- 最近更新：2026-06-14 建立二级功能专题。
- 下一步：只修 T/P 调节主路径真实 blocker；完整换热器模型另开专题。
