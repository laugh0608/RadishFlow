# RadishFlow MVP Roadmap

更新时间：2026-05-28

## 用途

用途：提供第一阶段 MVP 路线图的轻量入口，说明当前目标、范围、里程碑状态和拆分后的详细文档位置。
读者：需要判断 MVP 当前阶段、下一步推进方向和历史路线图落点的开发者、用户和 AI / Agent。
不包含：详细开发流水、完整里程碑原文、CAPE-OPEN 探测长记录、Studio UI 设计规范和逐日验证日志。

## 当前结论

截至 2026-05-28，M1-M5 都已越过 MVP 的最小完成线，MVP α 用户视角 smoke、首版 demo 前硬化期和 MVP β 多组能力包也已阶段性收口：

- Rust 内核和 Studio 可以跑通最小稳态流程。
- `TP Flash`、`SolveSnapshot`、结果审阅和 `rf-ffi` JSON/error 基线已经形成可复验闭环。
- `.NET 10` CAPE-OPEN / COM 适配层已完成 DWSIM / COFE 侧的关键 PME 兼容验证。
- MVP β 第一刀：小案例作者体验 v0 已通过，当前不继续围绕同一阶段做细颗粒度打磨。
- MVP β 第二刀：建模输入能力 v0 已通过；项目级物性包选择、项目组分选择、Feed composition 输入、Unit 参数输入和 2 条 official demo case 复现验收已经形成 focused 回归，并通过 2026-05-27 仓库级验证。
- MVP β 后续能力包已完成多组 focused 收口：结果核对与案例说明 v0 第一版、受控连接恢复 v0、剩余单元建模闭环 v0、失败修复闭环 v0。
- MVP β 人工 smoke 与验收标准 v0 已定义；下一步执行 `docs/mvp/beta-acceptance-checklist.md` 中的 Smoke A-D，而不是推进 tag、release notes、便携包或零散 UI 打磨。
- 当前尚未达到正式 tag / release 节点标准，历史 `v26.5.1-dev` staging 材料不作为当前路线图事实源。

当前不再把路线图作为每日推进清单。今天做什么、当前验证基线和暂不推进项，以 `docs/status/current.md` 为准；MVP α 验收清单保留为内部验收记录，不再作为当前日常推进主线。

## MVP 目标声明

第一阶段 MVP 的目标是：

- 能定义最小组分与热力学模型。
- 能对二元体系做 TP Flash。
- 能在最小 flowsheet 中连接物料流、简单单元和 flash。
- 能从 Rust 内核通过 FFI 暴露给 `.NET 10` 适配层。
- 能被外部 PME 以 CAPE-OPEN PMC 形式识别和调用。

更复杂的商业化 UI、完整物性包系统、动态模拟、复杂单元操作和完整 CAPE-OPEN 生态适配不属于第一阶段目标。

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
- MVP β 下一步验收以 `docs/mvp/beta-acceptance-checklist.md` 为准：official demo 打开 / 运行 / 审阅 / 保存重开，`Mixer-Flash` 与 `Heater-Flash` 空白作者路径，缺 composition 恢复，以及 selected stream 断开 / 重连代表路径。
- 更多作者 checklist、同构 Home 入口和轻量报表增强先进入 backlog；只有当它们服务真实 blocker 修复或阶段验收时再推进。
- CAPE-OPEN / PME 只修真实验证暴露的 blocker，不继续主动扩第三方宿主矩阵。
- Studio UI 优化只做主路径和明确专题；不扩自由连线、完整拖拽布局、自动布线、完整参数表或完整结果报表。

## 后续 UI 专题阶段

MVP β 功能推进不应把 UI 问题长期拆成零散按钮、临时面板或局部样式补丁。等成组高频建模能力和小案例作者体验稳定后，应安排一个独立 UI 专题阶段，专门统一各端页面、单元模块界面和主要工作流的设计。

该专题阶段的目标：

- 统一 Studio 客户端本体、Home、Workbench、Canvas、Inspector、Result、Package / Auth 等主要页面的信息架构、视觉语言和交互状态。
- 统一单元模块 UI 的参数编辑、端口连接、运行结果、诊断和帮助入口，不让每个单元各自形成一套临时界面。
- 梳理服务端 / 控制面 UI 管理页面、移动端或只读视图等未来端点的边界；具体端点清单在专题启动时重新评估，不在当前路线图中冻结。
- 用设计稿先行替代边实现边调整：优先使用 `pencil` 工具产出 `.pen` 设计稿，经评审后再进入代码实现。
- 设计稿应随项目保存和同步。原则上每个明确端点或界面域维护一个独立 `*.pen` 文件，例如客户端本体、单元模块、服务端 UI 管理页面、移动端视图等；最终拆分粒度和目录命名在专题启动时确定。
- 专题启动时应读取 `docs/architecture/ui-inspiration-reference.md`，吸收 AFFINE、CodexApp、Cloudflare、GitHub、Discourse、1Panel 等优秀产品在排版、留白、信息密度、状态表达和管理型页面组织上的设计方法，但不得复制其品牌、图标、具体配色或页面结构。
- 代码实现应以评审后的设计稿和 `docs/architecture/studio-ui-design-guidelines.md` 为依据，并继续遵守既有 presentation / command / state 边界，不把视觉优化变成 shell 私有状态扩张。

## 拆分后的详细文档

| 文档 | 内容 |
| --- | --- |
| `docs/mvp/roadmap/milestones.md` | M1-M5 原始里程碑、任务、crate 分工和退出标准 |
| `docs/mvp/roadmap/plan-alignment-studio.md` | 2026-03-29 至 2026-04-04 的 Studio / 控制面 / GUI 宿主计划对齐历史 |
| `docs/mvp/roadmap/plan-alignment-capeopen.md` | 2026-04-16 至 2026-04-25 的 `.NET 10` CAPE-OPEN / COM / PME 计划对齐历史 |
| `docs/mvp/roadmap/workflow-and-risks.md` | 推荐工作流、中后期 Studio 交互方向、首批任务拆分、风险和 DoD |

## 相关入口

- 当前阶段和下一步：`docs/status/current.md`
- MVP α 验收记录：`docs/mvp/alpha-acceptance-checklist.md`
- MVP β 人工 smoke 与验收标准：`docs/mvp/beta-acceptance-checklist.md`
- MVP 冻结范围：`docs/mvp/scope.md`
- Studio UI 规范：`docs/architecture/studio-ui-design-guidelines.md`
- UI 灵感参考：`docs/architecture/ui-inspiration-reference.md`
- CAPE-OPEN / COM 边界：`docs/capeopen/boundary.md`
- 周志索引：`docs/devlogs/README.md`
