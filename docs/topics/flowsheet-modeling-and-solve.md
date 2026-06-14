# 流程图建模与求解闭环

更新时间：2026-06-14

## 用途

用途：定义受控流程图建模、单元参数、连接建议、readiness、Run Panel 和 solver 之间的开发边界。  
读者：负责 `rf-model`、`rf-flowsheet`、`rf-unitops`、`rf-solver`、Studio 建模入口和运行门禁的开发者、用户、AI / Agent。  
不包含：自由连线编辑器、任意端口选择器、自动布线系统、完整拖拽布局器、复杂回路收敛和动态模拟。

## 专题目标

- 普通空白项目在受控范围内搭建并运行小流程，而不是依赖固定 demo case gate。
- readiness 只拦截确定的建模输入缺失；结构性连接、拓扑、非法旧项目或求解阶段参数失败交给正式 Run Panel 诊断 / recovery。
- Rust Core 不引入 COM / CAPE-OPEN 语义，继续使用领域模型、单元、连接和 solver 边界表达能力。

## 子专题

本专题是流程图建模与求解的一级轨道。具体单元和建模对象由二级功能专题承载：

| 子专题 | 状态 | 入口 |
| --- | --- | --- |
| Feed / 进料源 | Active | `unitops/feed-source.md` |
| Heater / Cooler 换热器 | Active | `unitops/heater-cooler.md` |
| Flash Drum 闪蒸罐 | Active | `unitops/flash-drum.md` |
| Mixer 混合器 | Active | `unitops/mixer.md` |
| Valve 阀门 | Active | `unitops/valve.md` |
| Material Stream 物流股 | Active | `modeling/material-stream.md` |

后续涉及具体单元行为、参数、端口、结果或诊断时，应优先更新对应子专题；本文件只保留跨单元的建模 / 求解边界。

## 当前实现快照

已完成：

- 支持 `Feed -> Flash Drum`。
- 支持 `Feed -> Heater/Cooler/Valve -> Flash Drum`。
- 支持 `Feed + Feed -> Mixer -> Flash Drum`。
- Feed source stream T/P/F/z、composition 归一和必要单元参数进入 readiness。
- Canvas suggestion 可显式接受连接或创建 outlet stream。
- Unit Inspector 参数提交走正式文档命令，并支持保存 / 重开 / rerun。

已知缺口：

- 受控连接恢复已满足当前 MVP，但不是自由连线编辑器。
- Cooler / Valve 有 focused 覆盖，但不新增同构 Home 作者入口。
- 完整回路、塔器、动态模拟和复杂单元均未进入当前阶段。

## 用户路径

1. 在 Workbench 左侧 `模块` 放置受控单元。
2. 通过 Canvas suggestion 补齐 canonical material ports。
3. 在 Inspector / 模块设置提交 Feed source stream、composition 和单元参数。
4. 运行前 readiness 提示明确输入缺口。
5. 运行后 solver 生成当前 revision 的 `SolveSnapshot`。
6. 若结构、拓扑或求解阶段失败，Run Panel 诊断和 recovery action 指向具体 stream / unit / port。

## 范围

本专题纳入：

- 受控单元：Feed、Mixer、Heater / Cooler、Valve、Flash Drum。
- canonical material ports 和一股一源一汇校验。
- 建模 readiness 与 Run Panel 诊断的边界。
- Canvas suggestion、Inspector focus 和参数提交。
- 保存 / 重开 / rerun 的建模输入稳定性。

本专题不纳入：

- 自由连线、任意端口选择和自动布线。
- 完整拖拽布局编辑器；当前布局仍为 sidecar / GUI state。
- 完整 recycle 收敛、动态模拟、CFD。
- 复杂单元操作、严格塔器和完整物性数据库。
- CAPE-OPEN / COM 宿主语义。

## 设计边界

### 数据与状态

- `rf-model` 承载对象模型，不承载求解策略或 COM 语义。
- `rf-flowsheet` 承载 graph / port / connection 校验。
- `rf-unitops` 围绕 `MaterialStreamState` 和单元参数。
- `rf-solver` 承载顺序模块法执行。
- Studio 只通过 document command 和 presentation 消费这些能力。

### 命令与接口

- 建模修改必须进入 `DocumentCommand` / command history。
- Canvas pending edit 在提交前不写项目语义。
- 连接 suggestion 的接受动作必须明确是连接流股还是创建流股。
- readiness 不扩成第二套 solver。

### UI 与交互

- 放置入口归左侧 `模块`。
- 对象树归左侧 `项目`。
- 选择语义和参数归右侧 Inspector / 模块设置。
- 运行反馈归 Run Panel、底部和诊断，不塞回 Canvas 私有状态。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 现有三类小流程冻结 | 三类路径 focused test、保存 / 重开 / rerun 和结果审阅均通过 |
| M2 | 建模缺口治理 | 只处理真实主路径 blocker，例如输入缺口定位不准或误拦 |
| M3 | 连接能力评估 | 若要推进更自由的连接，先单独开“流程图连接编辑器”专题 |

## 验收标准

- 三类小流程均可从普通空白项目完成。
- readiness 能定位缺 package、缺项目组分、缺 Feed composition、缺 source stream 和必要单元参数。
- 结构性问题和求解阶段问题由 Run Panel 诊断处理。
- 保存 / 重开后建模输入不丢失。
- Rust Core 不出现 COM / CAPE-OPEN 语义倒灌。

## 验证计划

- focused test：三类小流程 happy path、参数缺失、composition 归一、保存 / 重开 / rerun、诊断定位。
- 仓库级：核心阶段执行 `./scripts/check-repo.sh`。
- 人工 smoke：只有新增用户可见建模入口或 connection 行为时执行。

## 状态记录

- 当前状态：Active
- 最近更新：2026-06-14 从 MVP β / 通用小流程建模 v1 流水中拆出独立专题。
- 下一步：继续只修主路径真实 blocker；更自由的连接编辑另开专题。
