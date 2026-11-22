# Heater / Cooler 换热器专题

更新时间：2026-11-19

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
- `Heater` / `Cooler` 是单侧物料调节单元，普通工程支持目标出口温压，Rust Core 单相入口支持显热衡算。
- 中间 outlet stream 必须作为正式结果对象进入 downstream flash 和结果审阅。

## 规划衔接（尚未实现）

2026-09-16 已将双侧热平衡、面积、几何及热力／水力设计纳入设备工程规划，由 [换热器设计与性能校核](../equipment/heat-exchanger-design-and-rating.md) 管理。单侧物料调节和显热衡算按所需物性依赖推进。

## 当前实现快照

已完成：

- `Heater` / `Cooler` 支持 inlet / outlet material path。
- Unit Inspector 覆盖 `outlet_temperature_k` 和 `outlet_pressure_pa`。
- outlet pressure 高于已连接 inlet pressure 时停留在 invalid draft；提交上游 Feed / 流股压力后，保留草稿会按最新约束重验，保留原始输入且不自动提交。
- 提交参数会同步对应 outlet stream 模板。
- `Heater/Cooler -> Flash Drum` 自动化已覆盖保存 / 重开 / rerun 与结果审阅；2026-09-14 的 B2-3 将正式结果输出断言扩展到 Cooler，并覆盖其压力草稿联动。
- macOS 普通空白来源的 Heater 与 Cooler 主路径已验证；Cooler 温压编辑、失败恢复、原生保存重开 / 重跑与输出于 2026-09-15 完成 B2-5 验收。

已知缺口：

- UA、能量流股和严格换热器设计按各自专题推进。
- `Cooler` 复用普通空白建模入口，不新增同构 Home 作者入口；自动化覆盖不代替原生窗口逐项验收。

## Rust Core 单相显热规格

- 显式选择单相常热容模型后，每台单元须在 `UnitOperationParameters::outlet_temperature_k` 与本次运行的 `heat_duties_w` 中选一种规格。缺规格、同时指定、非有限热负荷及错误单元引用均返回结构化诊断。
- 指定出口温度时返回带符号的热负荷。指定热负荷时反求温度，零负荷且入口有流量时出口等温。零入口流量时，热负荷无法确定出口温度。温区边界由模型声明，越域规格拒绝运行。
- Core 步骤记录进出口焓流、带符号热负荷、能量残差和缩放信息。两种规格均使用实际出口焓复核。原工程由既有 TP 路径运行。 [显热模型与容差](../../thermo/mvp-model.md#单相稳态显热模型rust-core) · [双进料验收](../../devlogs/2026-11/2026-W47.md#2026-11-19-单相-heater-和-cooler-正反算)

### v4 保存与重开

`simulation.steadyStatePlan.method` 为 `radishflow.acyclic-single-vapor-sensible` v1，要求 `radishflow.solver.single-vapor-sensible` v1。`singlePhase` 保存 `minTemperatureK`、`maxTemperatureK`、`pressurePa`、按组分 ID 的 `heatCapacitiesJPerMolK`，以及按 Heater／Cooler ID 的 `heatDutiesW`。版本 1 固定 `298.15 K` 气相焓参考态，温区含端点。热容与固定压力须正且有限，热负荷为带符号有限值。两种映射按 ID 排序写出。

指定温度在单元的 `parameters.outlet_temperature_k` 保存。指定热负荷使用 `heatDutiesW`，对应单元的出口温度字段缺席。两个输入同在时拒绝读写。尚未完成的单元允许两个输入均缺席，执行时报告缺规格。热容和热负荷引用必须指向工程中的组分与对应单元。实际焓、反求温度、能量残差和运行快照由显式运行生成。重开后结果为 Missing。

已有 v4 TP 工程沿用原方法。缺少单相能力的 v4 读者对新工程提供整份只读保护并拒绝覆盖。首次覆盖已有工程以引入该能力时需确认。移除该能力后另存新路径。 [双进料工程](../../../examples/flowsheets/two-feed-single-vapor-sensible.rfproj.json)

### Studio

Studio 在「稳态运行计划」中选择「单相显热序贯」，录入适用温压与组分热容。在模块设置提交指定出口温度。切换至热负荷时，在运行计划中清除出口温度、录入热负荷并应用计划。切回温度规格时先移除热负荷并应用，再提交出口温度。显式运行后，「流程结果」显示实际出口温度、进出口焓流、带符号热负荷和两种残差。

### CLI v11

`query` 的 `flow_plan` 返回规范配置、配置 SHA-256、就绪状态及诊断。`execute` 的 `run_flow` 校验工程 ID、文件 SHA-256、输入修订与计划修订。运行已保存的单相计划时省略 `flow_resources`，物性取自工程。`flow` 为 Current 时包含实际流股温压与流量、按单元的 `energy_balances`，以及带配置摘要的执行来源。失败只发布诊断。CLI 在内存运行，重新启动查询仍为 Missing。v10 及更早的 v4 读者对新工程整份只读。

v11 可用 `configure_flow_plan` 应用本次请求内的单相计划。`plan.method` 为 `radishflow.acyclic-single-vapor-sensible/1`，`plan.single_phase` 使用 v4 `singlePhase` 的字段。温度规格沿用单元已有参数，热负荷规格从计划读取。 [通用命令契约](../../reference/headless-cli.md)

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
| M3 | 严格换热器评估 | 双侧热工、面积与校核按 [换热器专题](../equipment/heat-exchanger-design-and-rating.md) 推进 |

## B2-5：Cooler 空白建模与温压结果闭环

2026-09-15 完成，承接 B2-4。已有单入口矩阵与压力草稿回归通过；macOS 普通空白项目完成以下验收，未发现需要修改产品逻辑的新缺陷。

- 从普通空白项目选择物性 / 组分，搭建 `Feed -> Cooler -> Flash Drum`；显式提交出口 T/P，确认降温后的中间流股由 Flash 消费。
- 核对缺参数提示、非法温度 / 超入口压力草稿拒绝，以及上游压力变化后保留草稿双向重验；提交、文档 Undo / Redo 与结果旧化保持一致。
- 制造可恢复的参数或连接失败，核对 F5 / F8 定位、修正后重跑及结果面板反馈。
- 原生保存、重开、重跑，核对 Cooler 产出、Flash 消费、结果表及复制 / 文本导出的当前 snapshot / revision、SI 数值和物料一致性；未提交草稿不入项目，旧结果不能输出。
- macOS 实窗前先告知，使用独立临时工程并读回保存 / 导出文件；按实际代码影响补定向验证，需要阶段收口时执行当时的仓库基线，证据归周志。

新增 [Cooler 温度编辑回归](../../../apps/radishflow-studio/src/studio_gui_shell/tests/canvas/cooler.rs)：非法温度不改 revision 或保存输入，有效温度修改使旧结果不可导出，保存重开重跑后焓重新计算、Cooler 产出与 Flash 消费一致，并核对物料平衡及正式输出。压力草稿双向重验和 Undo / Redo 继续复用既有回归；原生验证步骤、SI 数值和文件读回证据见 [W38](../../devlogs/2026-09/2026-W38.md#2026-09-15-b2-5-cooler-空白建模与温压结果闭环)。

本切片验证现有目标 T/P 调节模型的软件闭环，不构成独立物理准确性证明，不引入 UA、热负荷或能量流股；Windows / Linux 原生路径另行安排，未测项保持待验证。

## 普通工程验收标准

- 必须显式提交 outlet T/P 才算参数完成。
- outlet pressure 约束不被 UI fallback 掩盖。
- 中间 outlet stream 可在结果表和 Inspector 审阅。
- 保存 / 重开后参数与结果入口稳定。

## 验证计划

- focused test：Heater-Flash happy path、Cooler-Flash happy path、参数缺失、pressure invalid draft、保存 / 重开 / rerun、结果一致性。
- 仓库级：涉及核心单元或求解改动时执行 `./scripts/check-repo.sh`。

## 状态记录

- 当前状态：Active
- 2026-11-19 完成单相显热正反算的 Core、v4 保存、CLI v11 与 Studio 接入。普通工程的 B2-5 证据见上文。
- 普通工程跨平台原生路径随对应平台节点补证。
