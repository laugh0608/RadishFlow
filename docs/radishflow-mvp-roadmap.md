# RadishFlow MVP Roadmap

更新时间：2026-06-08

## 用途

用途：提供第一阶段 MVP 路线图的轻量入口，说明当前目标、范围、里程碑状态和拆分后的详细文档位置。
读者：需要判断 MVP 当前阶段、下一步推进方向和历史路线图落点的开发者、用户和 AI / Agent。
不包含：详细开发流水、完整里程碑原文、CAPE-OPEN 探测长记录、Studio UI 设计规范和逐日验证日志。

## 当前结论

截至 2026-06-07，M1-M5 都已越过 MVP 的基本完成线，MVP α 用户视角 smoke、首版 demo 前硬化期、MVP β 多组能力包和通用小流程建模 v1 都已阶段性收口，Studio UI 专题已完成首轮设计前置并进入 presentation / window model 小切片实现：

- Rust 内核和 Studio 可以跑通最小稳态流程。
- `TP Flash`、`SolveSnapshot`、结果审阅和 `rf-ffi` JSON/error 基线已经形成可复验闭环。
- `.NET 10` CAPE-OPEN / COM 适配层已完成 DWSIM / COFE 侧的关键 PME 兼容验证。
- MVP β 第一刀：小案例作者体验 v0 已通过，当前不继续围绕同一阶段做细颗粒度打磨。
- MVP β 第二刀：建模输入能力 v0 已通过；项目级物性包选择、项目组分选择、Feed composition 输入、Unit 参数输入和 2 条 official demo case 复现验收已经形成 focused 回归，并通过 2026-05-27 仓库级验证。
- MVP β 后续能力包已完成多组 focused 收口：结果核对与案例说明 v0 第一版、受控连接恢复 v0、剩余单元建模闭环 v0、失败修复闭环 v0。
- MVP β 人工 smoke 与仓库级阶段基线验证已通过；通用小流程建模 v1 已完成普通空白项目主路径复核，而不是继续推进 tag、release notes、便携包或零散 UI 打磨。
- 通用小流程建模 v1 已完成到第十六切片并通过阶段收口复核：运行按钮、`Resume`、F5 / Shift+F5、AppHost、StudioGuiDriver、StudioGuiHost command registry 等用户可触达运行入口已共用通用 `Flowsheet` readiness，普通空白项目不再自动匹配小案例 gate；`Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum` 已覆盖显式输入、保存 / 重开 / rerun、结果审阅入口、关键结果合理性、单相 Flash 缺席语义、失败态定位、case-level review summary、编辑后旧结果失效语义，以及 Inspector 显示 outlet stream 默认值但缺显式 unit parameter 时的同值提交语义。
- 阶段性约束已从“围绕 β 验收补覆盖”调整为“已通过阶段只修真实 blocker，下一组推进 Studio UI 专题设计前置和窄口径实现切片”：指定小案例入口、MVP β Smoke A-D 和结果审阅覆盖面不再作为每日 gate。
- Studio UI 专题当前只允许沿评审后的 `studio-client-main.pen` 与 brief 做窄口径 implementation slices：Home case tile、Property page、status summary、Module Results、Module Settings、右侧栏职责、底部运行信息与状态汇总分栏、顶部导航入口、流程图 / 物性 / 运行 / 结果上下文工具栏，以及 `工具 / 设置` 菜单职责已按既有 DTO / command surface / shell state 分步收敛；仍不做完整 ribbon、完整右侧栏重排、完整参数表、自由连线、完整偏好页或完整报表。
- 当前尚未达到正式 tag / release 节点标准，历史 `v26.5.1-dev` staging 材料不作为当前路线图事实源。

当前不再把路线图作为每日推进清单。今天做什么、当前验证基线、下一阶段和暂不推进项，以 `docs/status/current.md` 为准；MVP α 验收清单保留为内部验收记录，不再作为当前日常推进主线。

## MVP 目标声明

第一阶段 MVP 的目标是：

- 能定义最小组分与热力学模型。
- 能对二元体系做 TP Flash。
- 能在最小 flowsheet 中连接物料流、简单单元和 flash。
- 能从 Rust 内核通过 FFI 暴露给 `.NET 10` 适配层。
- 能被外部 PME 以 CAPE-OPEN PMC 形式识别和调用。

更复杂的商业化 UI、完整物性包系统、动态模拟、CFD 模拟、复杂单元操作和完整 CAPE-OPEN 生态适配不属于第一阶段目标。

## MVP 功能边界

第一阶段只覆盖：

1. 二元或少数组分体系。
2. 简化 `K-value` 或可替换的最小热力学模型。
3. TP Flash。
4. 基础物流对象。
5. 少量最小单元操作，例如 `Feed`、`Mixer`、`Heater/Cooler`、`Valve`、`Flash Drum`。
6. 无回路顺序模块法求解。
7. C ABI FFI。
8. `.NET 10` CAPE-OPEN PMC 骨架。
9. 外部 PME 手工验证。

## 不做项

第一阶段明确不做：

- 完整物性数据库。
- 完整方程状态模型。
- 严格塔器模型。
- 复杂回路收敛器。
- 动态模拟。
- CFD 模拟。
- 完整 flowsheet GUI。
- 自研 PME。
- 完整 CAPE-OPEN 所有接口覆盖。
- 第三方 CAPE-OPEN 模型加载。

## 里程碑状态

| 里程碑 | 目标 | 当前状态 |
| --- | --- | --- |
| M1 | 仓库与基础骨架初始化 | 已越过最小线，仓库治理和基础 crate 边界已形成 |
| M2 | 二元体系 TP Flash 核心跑通 | 已越过最小线，official / synthetic golden 与边界容差回归已覆盖 |
| M3 | 最小稳态流程闭环跑通 | 已越过最小线，Studio、solver bridge、workspace run path 与结果审阅已闭环 |
| M4 | Rust FFI 与 `.NET 10` 适配层打通 | 已越过最小线，`rf-ffi` JSON/error 和 native 装载路径已有回归基线 |
| M5 | 外部 PME 识别并调用自有 PMC | 已越过最小线，DWSIM / COFE 关键人工验证路径已阶段性跑通 |

完整 M1-M5 任务和退出标准见 `docs/mvp/roadmap/milestones.md`。

## 当前计划边界

后续路线图不再服务开放式扩张，而是服务更大颗粒的 MVP β 能力包：

- 已通过的 `Mixer-Flash` 与 `Heater-Flash` 作者入口只修真实 blocker，不继续补同构入口、hover、按钮文案或 selector 小细节。
- 建模输入能力已完成 v0 收口：项目组分选择、内置物性方法 / package 选择、Feed composition 输入和单元参数输入已形成可保存 / 重开 / 运行的受控工作流。
- 组分输入 v0 只覆盖内置小型组分目录，不做完整组分数据库；物性方法 v0 只覆盖内置方法 / package，不加载第三方 Property Package。
- 当前 Studio 已把内置 `binary-hydrocarbon-lite-v1` 物性包选择和 methane / ethane 项目组分选择暴露在左侧 `项目` 面板和右侧 `物性包` 页；空白项目初始不预选，用户显式选择后才写入项目。这只是受控输入能力，不代表完整组分数据库或物性包浏览器。
- demo case 作为验收方式：`Heater-Flash` 与 `Mixer-Flash` 已覆盖 official demo case 输入和结果核对；`Cooler-Flash` 与 `Valve-Flash` 已用内部 focused test 覆盖空白项目建模闭环，但不新增 Home 作者入口或用户 guide。
- 失败修复闭环已完成 focused 收口；人工 smoke 只覆盖代表性恢复路径，不把 focused tests 已覆盖的所有恢复生命周期全部手工重跑。
- 通用小流程建模 v1 已完成阶段收口：普通空白项目必须按真实 `Flowsheet` readiness 运行，不再按某个作者案例阻断；用户已能在受控范围内自行组合 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum`，显式补齐 Feed source stream 和必要单元参数后保存 / 重开 / rerun，并通过右侧 Result Inspector、底部结果表、Results commands、轻量导出和 case-level review summary 审阅当前 revision 的同一份 `SolveSnapshot`。文档编辑后旧结果只用于 stale notice，不继续驱动结果审阅或导出入口；Unit Inspector 中由 outlet stream 模板派生的显示值不等于已提交单元参数，用户提交同一显示值时仍应写入正式 `UnitOperationParameters`。
- `Cooler` / `Valve` 已作为通用空白项目受控路径完成复核，但不新增同构 Home 作者入口或 checklist。既有小案例清单只作为导航提示，不再作为通用建模能力的运行 gate。
- 允许服务建模正确性的 UI 状态表达和轻量结果审阅材料改进，但当前不继续扩同类补丁；视觉精修、大改版、完整报表、模板、打印、批量导出和跨快照报表仍不进入当前阶段。
- readiness 后续只能继续承担确定的建模输入缺失门禁；结构性连接、拓扑、非法旧项目或求解阶段参数失败继续由正式 Run Panel 诊断 / recovery 承担，不把 readiness 扩成第二套 solver。
- CAPE-OPEN / PME 只修真实验证暴露的 blocker，不继续主动扩第三方宿主矩阵。
- 下一组已进入 Studio UI 专题：先按 `docs/architecture/studio-ui-topic-plan.md` 和 `docs/architecture/designs/studio-client-main-brief.md` 维护端点清单、信息架构、主工作流、状态模型和 `.pen` 保存规则，再按评审后的设计稿做 presentation / window model 小切片；不扩自由连线、任意端口选择器、完整拖拽布局、自动布线、完整参数表或完整结果报表。

## 远期产品方向

以下方向进入 RadishFlow 长期产品规划，但不属于 MVP β、通用小流程建模 v1 或下一阶段执行任务：

- 动态模拟：在稳态 flowsheet、物性包、单元模型、结果审阅、保存格式和诊断基础稳定后，再评估动态状态变量、时间积分、控制系统、扰动场景、动态结果记录与 UI 交互。
- CFD 模拟：作为更远期的高保真流动、传热、多相局部场模拟方向，需要独立的网格、边界条件、数值求解、后处理和算力 / 数据管理架构；不应提前压入当前 Rust Core 的稳态 flowsheet 主线。
- 二者与当前稳态流程模拟主线的关系：未来可通过稳定的项目模型、结果数据、物性接口和单元边界做耦合或旁路分析；当前不新增 crate、schema、UI 入口或测试基线。

## 后续 UI 专题阶段

MVP β 功能推进不应把 UI 问题长期拆成零散按钮、临时面板或局部样式补丁。等成组高频建模能力和小案例作者体验稳定后，应安排一个独立 UI 专题阶段，专门统一各端页面、单元模块界面和主要工作流的设计。

该专题阶段的目标：

- 统一 Studio 客户端本体、Home、Workbench、Canvas、Inspector、Result、Package / Auth 等主要页面的信息架构、视觉语言和交互状态。
- 统一单元模块 UI 的参数编辑、端口连接、运行结果、诊断和帮助入口，不让每个单元各自形成一套临时界面。
- 梳理服务端 / 控制面 UI 管理页面、移动端或只读视图等未来端点的边界；具体端点清单在专题启动时重新评估，不在当前路线图中冻结。
- 用设计稿先行替代边实现边调整：优先使用 `pencil` 工具产出 `.pen` 设计稿，经评审后再进入代码实现。
- 设计稿应随项目保存和同步。原则上每个明确端点或界面域维护一个独立 `*.pen` 文件，例如客户端本体、单元模块、服务端 UI 管理页面、移动端视图等；最终拆分粒度和目录命名在专题启动时确定。
- 专题启动时应读取 `docs/architecture/ui-inspiration-reference.md`，吸收 AFFINE、CodexApp、Cloudflare、GitHub、Discourse、1Panel 等优秀产品在排版、留白、信息密度、状态表达和管理型页面组织上的设计方法，但不得复制其品牌、图标、具体配色或页面结构。
- 代码实现应以评审后的设计稿和 `docs/architecture/studio-ui-design-guidelines.md` 为依据，并继续遵守既有 presentation / command / state 边界，不把视觉优化变成 shell 私有状态扩张。当前已经进入的实现切片只服务于正式 DTO 和既有命令消费，不代表完整 Workbench 重排、完整参数表或完整报表进入范围。

## 拆分后的详细文档

| 文档 | 内容 |
| --- | --- |
| `docs/mvp/roadmap/milestones.md` | M1-M5 原始里程碑、任务、crate 分工和退出标准 |
| `docs/mvp/roadmap/plan-alignment-studio.md` | 2026-03-29 至 2026-04-04 的 Studio / 控制面 / GUI 宿主计划对齐历史 |
| `docs/mvp/roadmap/plan-alignment-capeopen.md` | 2026-04-16 至 2026-04-25 的 `.NET 10` CAPE-OPEN / COM / PME 计划对齐历史 |
| `docs/mvp/roadmap/workflow-and-risks.md` | 推荐工作流、中后期 Studio 交互方向、首批任务拆分、风险和 DoD |

## 相关入口

- 当前阶段和下一步：`docs/status/current.md`
- Studio UI 专题计划：`docs/architecture/studio-ui-topic-plan.md`
- MVP α 验收记录：`docs/mvp/alpha-acceptance-checklist.md`
- MVP β 人工 smoke 与验收标准：`docs/mvp/beta-acceptance-checklist.md`
- MVP 冻结范围：`docs/mvp/scope.md`
- Studio UI 规范：`docs/architecture/studio-ui-design-guidelines.md`
- UI 灵感参考：`docs/architecture/ui-inspiration-reference.md`
- CAPE-OPEN / COM 边界：`docs/capeopen/boundary.md`
- 周志索引：`docs/devlogs/README.md`
