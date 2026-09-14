# Mixer 混合器专题

更新时间：2026-09-14

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
- `Feed + Feed -> Mixer -> Flash Drum` 已覆盖显式输入、保存 / 重开 / rerun 和结果审阅；B2-4 补齐不等流量编辑、两入口最低压力切换和 macOS 普通空白用户闭环。

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
- 最近更新：2026-09-14，B2-4 完成下述基础路径验收和回归，未修改产品运行逻辑。
- 下一步：按基础功能优先级继续 Cooler；任意入口与混合能量模型待独立切片。


## B2-4：双进料建模与结果联动

- 普通空白项目选择二元烃 Lite 和两组分后，从模块面板放置 Feed、Mixer、第二个 Feed、Flash Drum；只有一股来源时不生成 Mixer 入口建议。补齐第二股后，用户分别接受 `inlet_a / inlet_b` 绑定，两入口齐全后才建议创建混合出口。三来源歧义继续由既有 local-rules 回归验证，未新增入口选择策略。
- 缺显式 outlet pressure 时 F5 提示补齐并定位 Mixer；两入口为 150000 / 120000 Pa 时，130000 Pa 草稿不可应用。保留压力草稿对两入口约束的重验沿用 B2-3，实现不变；新增测试覆盖最低压力来源切换、只提高另一股仍不能解禁、等于最低压力时可提交。
- macOS 不等流量案例：第一股为 300 K、150000 Pa、2 mol/s、甲烷 0.5；第二股为 330 K、120000 Pa、1 mol/s、甲烷 0.8。Mixer 显式 90000 Pa，产出 310 K、3 mol/s、甲烷 / 乙烷 0.6 / 0.4，Flash 入口消费同一结果。
- 显式断开第二股目标端后，运行报 `solver.connection_validation.unbound_inlet_port` 并定位 `mixer-1:inlet_b`；F8 定位 Mixer，选中原流股重连后恢复收敛。
- 保存 revision 26、原生重开清空快照、F5 rerun、当前复制及原生文本导出通过。模块结果、结果表和文本中的混合出口均为 310 K、90000 Pa、3 mol/s、567.615 J/mol；不将当前温度加权近似解释为总焓守恒。
- [同域 Mixer 回归](../../../apps/radishflow-studio/src/studio_gui_shell/tests/canvas/mixer.rs) 保留既有连接 / 保存 / 审阅覆盖，并验证流量从 2:1 改为 1:1 后旧结果不可导出；保存重开重跑得到 315 K、2 mol/s、甲烷 0.65，Mixer 产出与 Flash 消费引用相等，输出不改工程文件。
- Windows / Linux 原生路径、多来源歧义逐项实窗、零流量边界和独立物理准确性不在本轮实窗结论内。详细验证与文件证据见 [W38](../../devlogs/2026-09/2026-W38.md#2026-09-14-b2-4-mixer-双进料建模与结果联动)。
