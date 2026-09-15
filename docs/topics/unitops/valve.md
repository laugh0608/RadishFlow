# Valve 阀门专题

更新时间：2026-09-15

> 本文定义该专题的能力、开发范围与验收要求；具体迭代切片和优先级以 [当前状态](../../status/current.md) 为准。

## 用途

用途：定义当前 MVP 中 `Valve` 的出口压力调节、连接、求解结果和诊断边界。
读者：负责 `rf-unitops`、Studio 模块设置、流程图建模、结果审阅和测试的开发者、用户、AI / Agent。
不包含：严格节流阀 sizing、Cv/Kv、两相阀模型、噪声、临界流、控制阀动态和控制系统。

## 专题层级

- 层级：二级功能
- 父专题：`docs/topics/flowsheet-modeling-and-solve.md`
- 关联专题：`docs/topics/modeling/material-stream.md`、`docs/topics/results-review-diagnostics.md`

## 专题目标

- 用户能在普通空白项目中搭建 `Feed -> Valve -> Flash Drum`，显式提交 outlet pressure 并运行。
- 当前 Valve 是受控压力调节单元，不提前实现完整控制阀设备模型。
- Valve outlet stream 必须作为正式中间结果进入 downstream flash 和结果审阅。

## 当前实现快照

已完成：

- Valve 支持 inlet / outlet material path。
- Unit Inspector 覆盖 `outlet_pressure_pa`。
- outlet pressure 高于已连接 inlet pressure 时停留在 invalid draft。
- Valve-Flash 已有 focused 覆盖；B2-3 完成 macOS 普通空白建模、参数联动、保存重开重跑和轻量结果输出复核。

已知缺口：

- 当前不支持 Cv/Kv、阀开度、临界流、控制回路或动态阀门。
- 当前保持入口温度并降低压力，未通过 PH Flash 求等焓出口状态；受控压力调节不等同于绝热节流。数值边界见 [热力学模型](../../thermo/mvp-model.md#单元近似与能量解释)。
- 不新增同构 Home 作者入口。

## 范围

本专题纳入：

- outlet pressure 参数。
- pressure constraint。
- Valve outlet 作为 downstream flash consumed stream。
- 保存 / 重开 / rerun 和结果审阅。

本专题不纳入：

- 控制阀 sizing。
- 阀门开度、Cv/Kv 和临界流。
- 控制系统、动态模拟。
- 噪声、空化或机械设备选型。

## 设计边界

### 数据与状态

- outlet pressure 进入 `UnitOperationParameters`。
- 当前不引入能量流股或控制信号流股。
- Valve result 来自当前 `SolveSnapshot`。

### 命令与接口

- 参数提交走正式 unit parameter command。
- invalid draft 不写回项目。
- Feed 参数或流股输入提交后，保留的单元数值草稿按最新文档重算约束；入口上限变动可使草稿失效或恢复可应用。保留原始输入，不自动提交，不额外增加文档修订或历史；缺失显式参数的同值草稿仍需应用。
- 求解失败由 Run Panel 诊断，不由 UI fallback 掩盖。

### UI 与交互

- Valve 放置入口在左侧 `模块` 的调节单元分类。
- 参数编辑在右侧 `模块设置`。
- 结果审阅在右侧 `模块结果` 和底部结果表。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 当前压力调节模型冻结 | Valve-Flash 可运行、保存 / 重开 / rerun |
| M2 | 结果一致性复核 | Valve outlet 和 downstream flash consumed stream 同源 |
| M3 | 控制阀模型评估 | 若推进 Cv / control loop，先开新专题 |

## 验收标准

- outlet pressure 必须显式提交。
- pressure constraint 可阻断 invalid draft。
- Valve outlet result 可审阅。
- 不把 Valve 扩成控制系统入口。

## 验证计划

- focused test：Valve-Flash happy path、参数缺失、pressure invalid draft、保存 / 重开 / rerun、结果一致性。
- 仓库级：涉及 Valve 或 solver 语义时执行 `./scripts/check-repo.sh`。

## 状态记录

- 当前状态：Active
- 最近更新：2026-09-14，B2-3 修复跨对象保留压力草稿的动态校验，完成下述 macOS 用户闭环。
- 后续衔接：Mixer / Cooler 已分别完成 B2-4 / B2-5 macOS 验收，下一优先项见当前状态；保持当前压力调节模型，PH Flash 与等焓节流留待数值能力切片。


## B2-3：空白建模与参数联动

- 普通 `新建项目` 选择内置二元烃物性包及 Methane / Ethane，再从模块面板放置 Feed、Valve、Flash Drum，接受正式连接与出口建议。
- Valve 缺少显式出口压力时，F5 提示补齐并定位单元；150000 Pa 高于 130000 Pa 入口时无效且不可应用。上游降为 85000 Pa 后，已提交的 90000 Pa 出口由正式求解诊断拒绝，F8 定位 Valve。
- 修复并复验：95000 Pa 保留草稿在入口 85000 / 120000 Pa 间切换时，分别变为无效 / 可应用；不会自动改写 Valve 参数或其出口模板。
- 最终 revision 22 保存并通过原生打开命令重载；重载清空运行快照，F5 重跑收敛。Valve 结果与 Flash 消费流股均为 310 K、95000 Pa、1 mol/s、601.388 J/mol，组成各 0.5；当前复制与原生文本导出通过。
- [压力草稿回归](../../../crates/rf-ui/src/tests/unit_pressure_drafts.rs) 覆盖 Feed / stream 两类入口编辑、四类受限单元、无效数值、缺显式参数及单次提交 / Undo / Redo；既有空白保存重跑矩阵增加 Valve / Cooler 导出断言。
- macOS 全仓 1,162 项测试及严格 clippy 通过。Windows / Linux 原生路径未复验；这些证据验证软件状态一致性，不替代物理准确性或完整节流模型验收。过程与环境见 [W38](../../devlogs/2026-09/2026-W38.md#2026-09-14-b2-3-valve-空白建模与参数联动)。
