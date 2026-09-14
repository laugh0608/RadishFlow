# Heater / Cooler 换热器专题

更新时间：2026-09-14

> 本文定义该专题的能力、开发范围与验收要求；具体迭代切片和优先级以 [当前状态](../../status/current.md) 为准。

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
- outlet pressure 高于已连接 inlet pressure 时停留在 invalid draft；提交上游 Feed / 流股压力后，保留草稿会按最新约束重验，保留原始输入且不自动提交。
- 提交参数会同步对应 outlet stream 模板。
- `Heater/Cooler -> Flash Drum` 自动化已覆盖保存 / 重开 / rerun 与结果审阅；2026-09-14 的 B2-3 将正式结果输出断言扩展到 Cooler，并覆盖其压力草稿联动。
- macOS 普通空白来源的 Heater 主路径已验证；Cooler 独立空白建模、温压编辑和原生保存 / 输出实窗仍待 B2-5 验收。

已知缺口：

- 当前没有 UA、热负荷、能量流股或严格换热面积模型。
- `Cooler` 复用普通空白建模入口，不新增同构 Home 作者入口；自动化覆盖不代替原生窗口逐项验收。

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

## B2-5（候选）：Cooler 空白建模与温压结果闭环

安排在 2026-09-15，承接 B2-4；尚未开始实施或实窗验收。先复核已有自动化，仅对发现的真实缺口补实现和回归。

- 从普通空白项目选择物性 / 组分，搭建 `Feed -> Cooler -> Flash Drum`；显式提交出口 T/P，确认降温后的中间流股由 Flash 消费。
- 核对缺参数提示、非法温度 / 超入口压力草稿拒绝，以及上游压力变化后保留草稿双向重验；提交、文档 Undo / Redo 与结果旧化保持一致。
- 制造可恢复的参数或连接失败，核对 F5 / F8 定位、修正后重跑及结果面板反馈。
- 原生保存、重开、重跑，核对 Cooler 产出、Flash 消费、结果表及复制 / 文本导出的当前 snapshot / revision、SI 数值和物料一致性；未提交草稿不入项目，旧结果不能输出。
- macOS 实窗前先告知，使用独立临时工程并读回保存 / 导出文件；按实际代码影响补定向验证，需要阶段收口时执行当时的仓库基线，证据归周志。

本切片验证现有目标 T/P 调节模型的软件闭环，不引入 UA、热负荷或能量流股；Windows / Linux 原生路径另行安排，未测项保持待验证。

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
- 最近更新：2026-09-14，补齐共享压力草稿重验、Cooler 自动化输出覆盖与尚待实窗的区分。
- 下一步：2026-09-15 优先 B2-5；更完整的换热模型按后续目标工况单独确定范围与能量验收。
