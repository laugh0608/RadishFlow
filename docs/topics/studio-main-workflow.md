# Studio 主工作台与空白项目建模主路径

更新时间：2026-09-12

> 本文定义该专题的能力、开发范围与验收要求；具体迭代切片和优先级以 [当前状态](../status/current.md) 为准。

## 用途

用途：定义 Studio 主工作台从启动、物性配置、流程图建模、运行、结果审阅到保存 / 重开的主路径开发边界。
读者：负责 Studio UI、window model、command surface、主路径验证和真实窗口 smoke 的开发者、用户、AI / Agent。
不包含：完整视觉系统、自由连线编辑器、完整报表、完整参数表、控制面管理 UI 和发布计划。

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
- 自由连线、任意端口选择、自动布线、完整拖拽布局。
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

## 分阶段切片

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
- 最近更新：2026-09-12，恢复开发状态并对齐当前迭代入口；既有能力仍以实现快照和验收记录为准。
- 下一步：补主路径 smoke 记录模板，并把后续代码切片绑定到本专题或相关子专题的退出标准。
