# 当前状态

更新时间：2026-08-18

## 用途

用途：为新会话恢复上下文、判断“今天做什么”提供入口。  
读者：开发者、用户、AI / Agent。  
不包含：完整历史流水、详细设计推演、测试日志和长期说明书。

默认先读本文档。若要进入具体功能或开发目标，再读取 `docs/topics/README.md` 和对应专题文档。`AGENTS.md` / `CLAUDE.md` 只保留长期协作规则，不承载当前阶段流水。

## 阶段结论

- 自 2026-06-12 起业务功能开发保持停止；模拟功能、物性模型、CAPE-OPEN / COM 适配、产品路线与发布能力不再推进。仓库外围基础设施仍可按需维护，包括文档治理、CI、ruleset、安全基线、仓库元数据和工具链兼容性；这不表示恢复产品开发或对外支持。
- 产品定位：以 Rust Core + Rust UI + `.NET 10` CAPE-OPEN / COM 适配层构建稳态流程模拟软件。
- MVP 第一阶段 M1-M5、MVP α、MVP β 人工 smoke、失败修复闭环和通用小流程建模 v1 均已阶段性收口。
- 阶段基线：2026-05-28 真实环境 `pwsh ./scripts/check-repo.ps1` 通过；2026-06-10 `./scripts/check-repo.sh` 通过。
- 通用小流程建模 v1 已支持普通空白项目在受控范围内组合 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum`，并覆盖显式输入、运行、保存、重开、rerun 与结果审阅。
- 2026-06-10 前的 Studio UI 专题实现已把 Home、独立 `物性` 页、流程图工作台、右侧 `模块设置 / 模块结果`、底部运行 / 状态分栏和 package label 展示口径阶段性收束。
- 2026-06-14 起，当前主线从“Studio UI 第 N 刀”调整为 **总进度 + 一级轨道专题 + 二级功能专题** 的开发节奏。具体单元、建模对象、后端服务和后端 Web UI 进入 `docs/topics/` 下的二级专题，实现前按专题阅读目标、范围、非目标、验收和验证。
- 当前尚未进入正式 tag / release 节点；历史 `v26.5.1-dev` 只作为内部 staging 草案和验证记录保留。

## 当前激活专题

当前专题索引：`docs/topics/README.md`。

一级轨道专题：

| 专题 | 状态 | 当前作用 |
| --- | --- | --- |
| `docs/topics/studio-main-workflow.md` | Active | 当前主线入口，约束 Studio 从 Home、物性、建模、运行、结果审阅到保存 / 重开的主路径 |
| `docs/topics/property-basis-and-components.md` | Active | 约束内置 package、项目组分、保存 / 重开和运行请求之间的同一事实源 |
| `docs/topics/flowsheet-modeling-and-solve.md` | Active | 约束受控单元、连接、readiness、Run Panel 和 solver 边界 |
| `docs/topics/results-review-diagnostics.md` | Active | 约束 `SolveSnapshot`、结果表、模块结果、状态汇总、诊断和 recovery action |
| `docs/topics/project-lifecycle-storage.md` | Backlog | 记录打开、保存、另存为、最近项目、sidecar 和脏改确认边界；当前不扩张 |
| `docs/topics/capeopen-pmc-adapter.md` | Frozen / Blocker-only | `.NET 10` CAPE-OPEN / COM 适配层保持基线，只修真实 blocker |

当前已建立的二级功能专题：

- 单元模块：`docs/topics/unitops/feed-source.md`、`docs/topics/unitops/heater-cooler.md`、`docs/topics/unitops/flash-drum.md`、`docs/topics/unitops/mixer.md`、`docs/topics/unitops/valve.md`
- 建模对象：`docs/topics/modeling/material-stream.md`
- 平台与服务：`docs/topics/platform/control-plane-service.md`、`docs/topics/platform/control-plane-web-ui.md`

## 当前策略

- 新增或修改功能前，先确认它属于哪个一级轨道专题和哪个二级功能专题；没有清晰归属时，先补专题文档，不直接写代码。
- 已通过的阶段只修真实 blocker：无法完成主路径建模、无法运行、结果明显错误、保存 / 重开破坏项目、文档事实源与代码能力冲突、仓库级验证或核心 focused test 失败。
- 不再把 hover、按钮文案、局部 selector、presentation 小瑕疵或同构入口作为默认推进内容。
- UI 改动必须服务当前激活专题的主路径、状态一致性或验收标准；不再以“第 N 刀”作为独立目标。
- 架构、接口、项目格式、验证基线或阶段边界变化时，同步更新对应专题文档，再更新本文档摘要和周志。

## 能力基线

通用建模与求解：

- 普通空白项目不再进入或自动匹配小案例状态；运行前检查按当前 `Flowsheet` 的真实建模输入判断。
- 缺项目组分、缺 Feed composition、Feed source stream 状态缺口、必要单元参数缺失和组成未归一由 readiness 定位到具体 stream / unit。
- 缺物性包、结构性连接、拓扑、非法旧项目和求解阶段参数失败继续交给正式 Run Panel 诊断 / recovery；readiness 不扩成第二套 solver。
- 轻量结果审阅已满足当前主路径判断；完整报表、模板、打印、批量导出和跨快照报表仍不进入当前阶段。

Studio 主路径：

- 顶部导航稳定为 `文件 / 主页 / 物性 / 流程图 / 运行 / 结果 / 工具 / 设置`。
- 普通空白项目创建后先进入独立 `物性` 页；Property 页入口、上下文工具栏、顶部 `流程图` 导航、Home `返回工作区` 和内部进入 Workbench 行为共用同一 Property readiness。
- Home recent / current / example case tile、左侧 `项目输入`、独立 `物性` 页和运行结果展示同源 package label；稳定 package id 只留在项目文件、command id、运行请求和内部状态边界。
- Workbench 区域职责稳定为左侧 `模块 / 项目`、中央 Canvas、右侧 `检查器 / 模块设置 / 模块结果`、底部运行信息与状态汇总。
- 结果表定位继续走正式 `inspector.focus_stream:*` / `inspector.focus_unit:*`，不维护第二套 shell 私有选择。

CAPE-OPEN / COM：

- Rust Core 不直接处理 COM。
- `.NET 10` CAPE-OPEN / COM 适配层只在真实 PME / Windows baseline 暴露 blocker 时推进。
- 注册、反注册、PME 人工验证和环境修改仍按协作规则先告知用户。

## 下一步

- 优先补 `docs/topics/studio-main-workflow.md` 对应的主路径 smoke 记录模板，覆盖 Home -> 物性 -> 流程图 -> 运行 -> 结果审阅 -> 保存 / 重开 / rerun。
- 后续代码切片必须绑定到一个当前激活专题的阶段目标和退出标准。
- 若继续修 Studio 主路径，只处理真实窗口或 focused test 暴露的状态冲突、入口重复、结果不可判断或保存 / 重开问题。
- 若要推进新功能，例如更完整的连接编辑、完整报表、物性分析、控制面管理或发布流程，先新增独立专题并明确范围 / 非目标 / 验收。
- 若改动落到具体单元或后端服务，优先更新对应二级功能专题，而不是继续扩写一级轨道专题。

## 验证节奏

- 核心数据、求解、保存和项目格式：必须测试。
- 新能力主路径：至少覆盖一条 happy path focused test。
- UI 展示细节：除非曾经造成 blocker，否则不为单个小展示点新增测试。
- 阶段收口：执行 `./scripts/check-repo.sh`；涉及 Windows `.NET` / CAPE-OPEN baseline 时按专题和协作规则使用 Windows / 真实环境验证。
- 若仓库级验证在沙盒中出现明显环境性失败，可按协作规则申请真实环境复验。

## 暂不推进

- 不继续在 β 第一刀上追加同构 Home 作者入口或同类 checklist；既有小案例清单只作为导航提示，不作为通用建模运行 gate。
- 不做自由连线编辑器、任意端口选择器、自动布线系统、完整拖拽布局编辑器。
- 不做完整报表系统、跨快照报表、模板导出、打印、批量导出。
- 不引入第三方 CAPE-OPEN 模型、第三方物性包加载、完整组分数据库或完整 Thermodynamics PMC。
- 不推进 tag、release notes、便携包刷新、安装器或发布自动化；这些事项等待后续明确发布节点。
- 不把 CAPE-OPEN / COM 语义倒灌到 Rust Core。
- 不为未来可能需求预先堆叠不明意义的 helper / manager / orchestrator / context / adapter。

## 按需阅读

- 专题索引：`docs/topics/README.md`
- 当前主路径专题：`docs/topics/studio-main-workflow.md`
- 物性基础：`docs/topics/property-basis-and-components.md`
- 流程图建模与求解：`docs/topics/flowsheet-modeling-and-solve.md`
- 结果审阅、诊断与恢复：`docs/topics/results-review-diagnostics.md`
- 项目生命周期：`docs/topics/project-lifecycle-storage.md`
- CAPE-OPEN PMC 适配层：`docs/topics/capeopen-pmc-adapter.md`
- 具体单元模块：`docs/topics/unitops/`
- 建模对象：`docs/topics/modeling/`
- 后端服务与管理台：`docs/topics/platform/`
- 最新流水和决策依据：`docs/devlogs/2026-06/2026-W24.md`
- MVP 范围和非目标：`docs/mvp/scope.md`
- MVP 路线图：`docs/radishflow-mvp-roadmap.md`
- 仓库全局模块边界：`docs/architecture/overview.md`
- App / Canvas / UI：`docs/architecture/app-architecture.md`、`docs/architecture/canvas-interaction-contract.md`、`docs/architecture/studio-ui-design-guidelines.md`
- UI 专题设计前置：`docs/architecture/studio-ui-topic-plan.md`
- 热力学 / 闪蒸细节：`docs/thermo/mvp-model.md`
- CAPE-OPEN / COM 边界：`docs/capeopen/boundary.md`
- 代码风格、命名或抽象判断：`docs/development/code-style.md`
- 文档篇幅和拆分规则：`docs/README.md`

## 更新规则

- 本文档只保留当前阶段、当前激活专题、下阶段目标、验证节奏和暂不推进项。
- 功能和开发目标写入 `docs/topics/`；历史流水写入周志；长期边界写入专题架构文档。
- 每次完成重要阶段收口后，优先更新对应专题文档，再同步本文档顶部阶段结论和下阶段目标。
