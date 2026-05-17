# RadishFlow MVP Roadmap

更新时间：2026-05-14

## 用途

用途：提供第一阶段 MVP 路线图的轻量入口，说明当前目标、范围、里程碑状态和拆分后的详细文档位置。
读者：需要判断 MVP 当前阶段、下一步推进方向和历史路线图落点的开发者、用户和 AI / Agent。
不包含：详细开发流水、完整里程碑原文、CAPE-OPEN 探测长记录、Studio UI 设计规范和逐日验证日志。

## 当前结论

截至 2026-05-14，M1-M5 都已越过 MVP 的最小完成线：

- Rust 内核和 Studio 可以跑通最小稳态流程。
- `TP Flash`、`SolveSnapshot`、结果审阅和 `rf-ffi` JSON/error 基线已经形成可复验闭环。
- `.NET 10` CAPE-OPEN / COM 适配层已完成 DWSIM / COFE 侧的关键 PME 兼容验证。
- 当前主线已经从“继续扩路线图功能”切到 MVP α 验收与发布硬化。

当前不再把路线图作为每日推进清单。今天做什么、当前验证基线和暂不推进项，以 `docs/status/current.md` 为准；MVP α 具体验收以 `docs/mvp/alpha-acceptance-checklist.md` 为准。

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

后续路线图不再主动扩张新功能范围，而是服务 MVP α 验收：

- 优先复跑 Studio 用户视角 smoke，确认打开示例、运行、结果审阅、保存重开和关闭窗口稳定。
- 继续保持 `Feed/Heater/Cooler/Valve/Mixer -> Flash` 的数值、结果 DTO 和 UI consumer 基线稳定。
- CAPE-OPEN / PME 只修真实验证暴露的 blocker，不继续主动扩第三方宿主矩阵。
- Studio UI 优化只做信息层级、主路径和面板重排，不扩自由连线、完整拖拽布局、自动布线或完整结果报表。

## 拆分后的详细文档

| 文档 | 内容 |
| --- | --- |
| `docs/mvp/roadmap/milestones.md` | M1-M5 原始里程碑、任务、crate 分工和退出标准 |
| `docs/mvp/roadmap/plan-alignment-studio.md` | 2026-03-29 至 2026-04-04 的 Studio / 控制面 / GUI 宿主计划对齐历史 |
| `docs/mvp/roadmap/plan-alignment-capeopen.md` | 2026-04-16 至 2026-04-25 的 `.NET 10` CAPE-OPEN / COM / PME 计划对齐历史 |
| `docs/mvp/roadmap/workflow-and-risks.md` | 推荐工作流、中后期 Studio 交互方向、首批任务拆分、风险和 DoD |

## 相关入口

- 当前阶段和下一步：`docs/status/current.md`
- MVP α 验收：`docs/mvp/alpha-acceptance-checklist.md`
- MVP 冻结范围：`docs/mvp/scope.md`
- Studio UI 规范：`docs/architecture/studio-ui-design-guidelines.md`
- CAPE-OPEN / COM 边界：`docs/capeopen/boundary.md`
- 周志索引：`docs/devlogs/README.md`
