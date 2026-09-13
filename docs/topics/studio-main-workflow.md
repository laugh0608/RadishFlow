# Studio 主工作台与空白项目建模主路径

更新时间：2026-09-13

> 本文定义该专题的能力、开发范围与验收要求；具体迭代切片和优先级以 [当前状态](../status/current.md) 为准。

## 用途

用途：定义 Studio 主工作台从启动、物性配置、流程图建模、运行、结果审阅到保存 / 重开的主路径开发边界。
读者：负责 Studio UI、window model、command surface、主路径验证和真实窗口 smoke 的开发者、用户、AI / Agent。
不包含：完整视觉系统、自动布线、完整报表、控制面管理 UI 和发布计划；超出现有受控连接的设计由建模专题承接。

## 专题目标

- 普通空白项目不再围绕某个 demo case 或 UI 小补丁推进，而是以用户可复现的完整建模路径为验收对象。
- 用户可以从 Home 新建项目，进入 `物性`，选择内置 package 和项目组分，再进入 `流程图` 放置单元、补齐输入、运行、审阅结果、保存、重开和 rerun。
- 本专题把 Studio UI 实现从“第 N 刀”改为主路径能力切片；局部 UI 调整只有影响主路径判断时才进入。

## 当前实现快照

已完成：

- Home、独立 `物性` 页、`流程图` 工作台、`运行`、`结果`、`工具`、`设置` 已形成顶层导航。
- 普通空白项目创建后先进入 `物性`，物性 readiness 通过后才能进入流程图建模。
- Workbench 已稳定为左侧 `模块 / 项目`、中央 Canvas、右侧 `检查器 / 模块设置 / 模块结果`、底部运行信息与状态汇总。
- `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum` 已有受控建模能力和 focused 回归。
- `Cmd+Q` 与窗口关闭共用脏工作区确认路径。

已知缺口：

- 当前仍有大量结论散落在 `status/current.md`、路线图、周志和 UI 专题计划中，后续应逐步迁移到本专题和相关子专题。
- 真实窗口 smoke 仍主要依赖人工截图反馈，尚未形成稳定的主路径 smoke 记录模板。
- Workbench 视觉密度可继续修正，但只有影响主路径理解、状态一致性或操作入口时才进入本专题。

相关入口：

- `docs/architecture/studio-ui-topic-plan.md`
- `docs/architecture/designs/studio-client-main-brief.md`
- `docs/guides/studio-quick-start.md`
- `docs/topics/property-basis-and-components.md`
- `docs/topics/flowsheet-modeling-and-solve.md`
- `docs/topics/results-review-diagnostics.md`

## 用户路径

1. 启动 Studio，在 Home 判断环境、登录 / 授权状态、当前工作区和最近项目。
2. 新建普通空白项目，进入独立 `物性` 页面。
3. 选择内置 `binary-hydrocarbon-lite-v1` package 和 methane / ethane 项目组分。
4. 进入 `流程图`，从左侧 `模块` 放置 Feed、Heater / Cooler / Valve、Mixer、Flash Drum。
5. 通过 Canvas suggestion、Inspector 和模块设置补齐连接、Feed source stream、composition 和单元参数。
6. 运行求解，在右侧模块结果、底部结果表、状态汇总和诊断中审阅同一份当前 `SolveSnapshot`。
7. 保存、重开、rerun，确认结果和旧快照状态一致。

## 范围

本专题纳入：

- Studio 主工作流的页面职责、入口关系和区域分工。
- 普通空白项目主路径所需的 UI 状态展示和正式 command 消费。
- 影响主路径的 Home、Property、Workbench、Run、Result 页面衔接。
- 真实 blocker 的 UI / DTO / command 修正。
- 主路径 focused test 与必要人工 smoke。

本专题不纳入：

- 完整 ribbon、完整偏好页、完整账号 / 授权设置页。
- 自动布线与完整拖拽布局；更自由的连接 / 端口选择由建模专题明确设计，本专题只负责用户路径集成。
- 完整参数表、完整报表、跨快照报表、模板、打印和批量导出。
- 第三方物性包、完整组分数据库、完整 Thermodynamics PMC。
- 控制面运营后台和移动端编辑体验。

## 设计边界

### 数据与状态

- 项目标题、脏状态、revision 来自当前 workspace document。
- 物性选择来自 `workspace_document.property_package_choices` 和 `project_component_choices`。
- 建模输入缺口来自 shared modeling readiness。
- 运行和失败状态来自 Run Panel / workspace control model。
- 当前结果来自当前 revision 的 latest `SolveSnapshot`。
- Workbench 不维护第二套项目、物性、运行或结果状态。

### 命令与接口

- UI 操作通过正式 command id 或 driver 入口派发。
- Canvas 放置、连接 suggestion、Inspector focus、run、result focus、保存和关闭确认都必须走既有 command / driver 边界。
- 若需要新增入口，先明确它属于 Home、Property、Canvas、Inspector、Run Panel、Result Review 还是 Document Lifecycle。

### UI 与交互

- Home 是开始和返回工作区入口，不是营销页。
- `物性` 是独立页面，不退回侧栏。
- 左侧 `模块` 负责放置，左侧 `项目` 负责项目输入扫读、对象树和审阅状态。
- 中央 Canvas 负责建模主舞台，不重复对象树和完整结果表。
- 右侧栏负责当前对象输入、模块设置和模块结果。
- 底部负责消息、运行日志、收敛、建议、诊断、结果表和状态汇总。

## 基础功能完善切片

2026-09-13 起，基础功能闭环为近期主线。原 MVP 的受控路径仍作为回归基线，阶段收口不再解释为只允许修 blocker。长期子系统边界见 [模拟平台规划](../architecture/simulation-platform.md)。新增动作需明确对象身份、变量描述、命令事务和失败语义，为变量浏览树、COM 自动化与操作录制复用；这些目标不要求 B1 先实现完整脚本系统。

下表是 B0 复核入口：文档与代码证据只证明相应层已存在，本轮没有执行 GUI 验收；不能把“待复核”直接登记为缺陷，也不能因为 command 存在就认定用户路径完整。

| 能力 | 现有依据 | B0 需要复核的操作与状态 | 所属专题 |
| --- | --- | --- | --- |
| 组分与物性选择 | 内置 package、项目组分、readiness 已有实现 | 查询 / 选择 / 移除入口、组分变化影响、非法组合反馈、保存重开一致性 | [物性基础](property-basis-and-components.md) |
| 对象与连接编辑 | [DocumentCommand](../../crates/rf-ui/src/commands.rs) 已含创建、重命名、连接、断开和删除命令 | 每种操作的 GUI 可达性、删除关联影响、无效连接拒绝、undo / redo 与持久化 | [建模与求解](flowsheet-modeling-and-solve.md) |
| 参数与规格输入 | 字段草稿、Feed T/P/F/z 与单元参数提交 | 单位与量纲展示、错误定位、草稿提交 / 放弃、跨面板一致性 | [App 架构](../architecture/app-architecture.md) |
| 运行与失败处理 | readiness、Run Panel、Active / Hold、revision 已有实现 | 修改后状态、执行中的交互、错误后修复再运行；同步执行限制如实记录 | [建模与求解](flowsheet-modeling-and-solve.md) |
| 结果审阅与输出 | 当前快照、stale notice、结果表及轻量输出已有基线 | 输入 / 结果对应、单位、错误目标跳转、导出内容和结果身份 | [结果与诊断](results-review-diagnostics.md) |
| 文档生命周期 | 新建、打开、保存、另存为、脏改确认与 sidecar | 覆盖、取消、保存失败、重开、历史和布局；保留已完成恢复修复 | [项目生命周期](project-lifecycle-storage.md) |

逐项登记采用“已验证可用 / 部分可用 / 待验证 / 缺失”及代码入口、测试、操作步骤、期望行为、平台与证据日期；实现台账留在对应专题，运行流水留在周志，不复制成多套总清单。

| 切片 | 工作 | 退出标准 |
| --- | --- | --- |
| B0 基础能力复核 | 顺着普通空白项目核对上表，首个候选为对象与连接编辑 | 区分现有能力和真实缺口，确定首个实现切片的边界、错误路径与最小验收；未运行的平台明确待验证 |
| B1 编辑与连接闭环 | 按 B0 结果补齐创建、编辑、连接、断开、删除影响与命令消费 | 成功 / 拒绝路径、undo / redo、保存 / 重开、结果旧化一致；新增连接范围先更新建模专题 |
| B2 配置、运行与结果闭环 | 按依赖补齐组分配置、参数反馈、运行恢复和基础结果输出 | 用户无需开发者解释即可完成当前支持的小流程并修正典型错误 |
| B3 运行与自动化衔接 | 复核变量 / 动作描述、共享命令与任务边界，接入路线图 P2 | 输入快照、任务归属、取消 / 旧结果，以及浏览、读写、创建 / 运行和可录制动作的首个范围明确 |

基础功能可使用演示物性验证软件行为，不把工业工况精度研究设为编辑 / 保存的前置条件。单位、有限值、引用完整性、状态一致性和保存安全持续验证；涉及新的物理模型时同步补对应数值依据。

## 历史 MVP 切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 文档节奏收束 | 本专题和相关子专题成为实现前默认阅读入口 |
| M2 | 主路径 smoke 记录 | 新增或更新主路径人工 smoke 记录模板，明确截图 / 操作 / 通过标准 |
| M3 | Workbench blocker 修正 | 只处理主路径仍暴露的入口重复、状态冲突或无法判断结果的问题 |
| M4 | 阶段冻结 | 普通空白项目主路径可稳定复现，后续只修 blocker |

## 验收标准

- 用户可以从 Home 新建普通空白项目并完成物性配置。
- 用户可以在受控范围内搭建三类小流程并运行。
- 保存 / 重开 / rerun 不破坏项目输入、布局 sidecar 或结果审阅入口。
- readiness、Run Panel 诊断和结果审阅不互相制造冲突状态。
- 文档入口能说明当前做什么、为什么做、如何验收。

## 验证计划

- focused test 优先覆盖 window model、command dispatch、主路径 readiness、保存 / 重开和结果审阅。
- UI 展示细节只在曾造成 blocker 或影响主路径判断时补测试。
- 阶段收口执行 `./scripts/check-repo.sh`；Windows `.NET` / PME 相关验证按 CAPE-OPEN 专题执行。

## 状态记录

- 当前状态：Active
- 最近更新：2026-09-13，确立基础功能优先的 B0-B3 路径，保留已有 MVP 证据和长期扩展边界。
- 下一步：执行 B0，优先复核流程图基础编辑与连接管理的代码、测试和真实窗口路径，再确定 B1 的具体实现。
