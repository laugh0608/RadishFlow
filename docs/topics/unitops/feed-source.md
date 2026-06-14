# Feed / 进料源专题

更新时间：2026-06-14

## 用途

用途：定义当前 MVP 中 `Feed` 单元的输入、端口、参数、运行结果和诊断边界。  
读者：负责 `rf-unitops`、Studio 模块设置、Canvas 放置、readiness 和结果审阅的开发者、用户、AI / Agent。  
不包含：完整物料源库、动态进料、外部数据源绑定、批量工况和真实工厂历史数据接口。

## 专题层级

- 层级：二级功能
- 父专题：`docs/topics/flowsheet-modeling-and-solve.md`
- 关联专题：`docs/topics/modeling/material-stream.md`、`docs/topics/property-basis-and-components.md`

## 专题目标

- `Feed` 是当前受控流程图的物料源入口，负责把显式输入的 T/P/F/z 写入 outlet material stream。
- 用户能在普通空白项目中放置 Feed、补齐 source stream 状态、连接下游单元并运行。
- Feed 不承担完整物料数据库、边界条件调度或动态扰动。

## 当前实现快照

已完成：

- `Feed(outlet)` canonical material port。
- Feed source stream 的 temperature、pressure、total molar flow 和 overall composition 进入 readiness。
- Stream Inspector 可编辑基础 SI 字段和组成条目。
- 缺 source stream 或 composition 时 readiness 能定位到具体 stream / unit。

已知缺口：

- 当前只覆盖受控二元 / 少数组分体系。
- 不支持随时间变化的进料、外部文件批量导入或在线数据绑定。

## 范围

本专题纳入：

- Feed outlet port。
- Feed source stream 显式 T/P/F/z 输入。
- 组成归一、缺组成和缺基础状态的诊断。
- Feed 结果作为 downstream 单元 consumed stream。

本专题不纳入：

- 动态进料曲线。
- 多工况批量切换。
- 工厂实时数据接口。
- 完整物料源模板库。

## 设计边界

### 数据与状态

- Feed 输入写入项目文档中的 stream state，不由 UI 私有缓存保存。
- 单位继续使用 SI：K、Pa、mol/s、mole fraction。
- 组成表达以当前项目组分为边界，不从 Feed 自行扩展组分数据库。

### 命令与接口

- 字段修改走 Stream Inspector / document command。
- Feed 放置走 Canvas place unit command。
- 诊断定位走正式 Inspector target / focus command。

### UI 与交互

- Feed 放置入口在左侧 `模块`。
- Feed source stream 编辑主要在流股 Inspector。
- 结果审阅通过结果表、Result Inspector 和 downstream 单元 consumed stream。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 当前 MVP Feed 冻结 | Feed 可作为三类小流程源头完成运行、保存 / 重开 / rerun |
| M2 | 输入诊断复核 | 缺 T/P/F/z 或组成异常能定位到具体对象 |
| M3 | 扩展评估 | 若需要外部数据源或工况模板，先新开专题 |

## 验收标准

- Feed 能从空白项目放置并生成 outlet stream。
- 缺少 source stream 状态时不能被误判为可运行。
- 保存 / 重开后 Feed 输入不丢失。
- Feed result 与 downstream consumed stream 数值一致。

## 验证计划

- focused test：Feed -> Flash Drum happy path、缺 source stream、缺 composition、保存 / 重开 / rerun。
- 仓库级：涉及核心建模改动时执行 `./scripts/check-repo.sh`。

## 状态记录

- 当前状态：Active
- 最近更新：2026-06-14 建立二级功能专题。
- 下一步：只在 Feed 主路径输入或诊断暴露真实 blocker 时推进。
