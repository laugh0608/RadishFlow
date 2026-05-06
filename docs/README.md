# RadishFlow Docs

## 文档索引

| 文档 | 说明 |
| --- | --- |
| `status/current.md` | 当前阶段、最近状态、今天建议和按需阅读入口 |
| `architecture/overview.md` | 当前仓库的分层、crate 边界与阶段职责 |
| `architecture/open-source-references.md` | 可借鉴的开源流程模拟、物性和算法参考清单及许可边界 |
| `architecture/versioning.md` | 项目版本命名、Git tag 与发布轨道约定 |
| `architecture/app-architecture.md` | 桌面 App 的壳层、状态与模块边界规划 |
| `architecture/canvas-interaction-contract.md` | 流程图画布的视图模式、流线视觉状态与 ghost suggestion 交互契约 |
| `architecture/auth-entitlement-architecture.md` | 桌面登录、授权、远端 `.NET 10` 控制面与本地求解边界 |
| `development/code-style.md` | 跨语言代码风格、抽象边界与 review 判断标准 |
| `mvp/scope.md` | 第一阶段 MVP 的冻结范围、非目标与近期开发节奏 |
| `thermo/mvp-model.md` | 热力学与 `TP Flash` 的最小接口与后续实现顺序 |
| `capeopen/boundary.md` | Rust Core 与 .NET 10 CAPE-OPEN 适配层边界 |
| `capeopen/pme-validation.md` | CAPE-OPEN 注册执行门控、安装/反安装运行手册与人工 PME 验证口径 |
| `adr/0001-branch-and-pr-governance.md` | 分支、PR 与保护规则治理决策 |
| `../.github/rulesets/master-protection.json` | `master` 保护规则模板 |
| `devlogs/README.md` | 按周分文件的开发日志规范 |
| `devlogs/2026-W13.md` | 2026 第 13 周开发日志 |
| `devlogs/2026-W14.md` | 2026 第 14 周开发日志 |
| `radishflow-architecture-draft.md` | RadishFlow 的目标架构草案 |
| `radishflow-startup-checklist.md` | 新仓库启动清单与迁移边界 |
| `radishflow-mvp-roadmap.md` | 第一阶段 MVP 开发路线图 |
| `radishflow-capeopen-asset-checklist.md` | 从 CapeOpenCore 提取 CAPE-OPEN 资产的清单 |

## 当前阅读顺序

如果是第一次进入仓库，默认只读以下最小入口：

1. `../AGENTS.md`
2. `status/current.md`
3. `../README.md`
4. `README.md`

如果任务只是询问“今天做什么”或“下一步推进什么”，读完 `status/current.md` 后即可先给出判断；只有需要依据或实现细节时才补读专题文档。

只有在任务涉及具体领域时，再读取对应专题文档：

- 全局模块边界：`architecture/overview.md`
- MVP 范围和非目标：`mvp/scope.md`
- 最近流水和决策依据：`devlogs/` 中当前周志
- CAPE-OPEN / COM：`capeopen/boundary.md`、`capeopen/pme-validation.md`
- 热力学 / 闪蒸：`thermo/mvp-model.md`
- 桌面 App / Canvas：`architecture/app-architecture.md`、`architecture/canvas-interaction-contract.md`
- 版本、分支与治理：`architecture/versioning.md`、`adr/0001-branch-and-pr-governance.md`
- 实现风格争议或抽象边界判断：`development/code-style.md`

## 文档维护约定

- 优先更新已有文档，不为单次讨论随意新增散文档
- 关键入口文档应尽可能简约，只描述当前阶段、最近进度、稳定边界和下一步
- 历史背景、长篇推演、逐日实现过程和一次性讨论应放入专题文档或周志，不塞进入口文档
- 更新入口文档时应同步删减过期背景和重复叙述，避免新会话为读取背景消耗过多上下文
- 重要阶段变化应同步更新对应索引文档和周志
- 周志按 `docs/devlogs/YYYY-Www.md` 命名

## 语言与代码实践约束

- 详细规范见 `development/code-style.md`；入口文档只保留高层约束
- 新增代码必须遵循对应语言的主流实践和本仓库已有风格，优先使用语言自身的类型系统、错误模型、模块系统和标准库能力
- 禁止新增含义不明的方法、类型或抽象封装；命名应能说明真实领域职责、边界和调用意图
- 不为了“整齐”或“可扩展”预先堆叠 helper、manager、orchestrator、context、adapter 等泛化层；只有在能减少真实重复、隔离稳定边界或表达明确领域概念时才新增抽象
- Rust 代码应优先用 `Result` / `Option`、所有权、枚举和小模块表达约束，避免字符串协议、隐式全局状态和层层 fallback
- C# / `.NET` 代码应保持现代 .NET 风格；COM / CAPE-OPEN 语义只留在适配层，不反向污染 Rust Core 与通用模型
- 文档中的代码约束应写成可执行规则，不写无法落地的抽象口号

## 外部参考

- [CapeOpenCore](https://github.com/laugh0608/CapeOpenCore)
- [Radish](https://github.com/laugh0608/Radish): 当前用于参考 OIDC / Auth / Client Registration 形态的统一平台仓库
- [DWSIM](https://github.com/DanWBR/dwsim): 用于参考 `Interfaces / FlowsheetBase / FlowsheetSolver / UnitOperations` 的职责拆分、自动化测试入口组织方式，以及图形对象与求解对象分离的工程经验

补充约束：

- `DWSIM` 仅作为行为和架构参考，不直接迁移实现代码
- 其 GPL-3.0 许可要求决定了当前仓库不应复制或改写式移植其源码
- 当前阶段只吸收对 `rf-unitops`、`rf-flowsheet`、`rf-solver`、自动化入口与测试组织有帮助的结构经验，不把其完整功能范围提前引入 MVP
