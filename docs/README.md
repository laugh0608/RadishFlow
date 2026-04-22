# RadishFlow Docs

## 文档索引

| 文档 | 说明 |
| --- | --- |
| `architecture/overview.md` | 当前仓库的分层、crate 边界与阶段职责 |
| `architecture/open-source-references.md` | 可借鉴的开源流程模拟、物性和算法参考清单及许可边界 |
| `architecture/versioning.md` | 项目版本命名、Git tag 与发布轨道约定 |
| `architecture/app-architecture.md` | 桌面 App 的壳层、状态与模块边界规划 |
| `architecture/canvas-interaction-contract.md` | 流程图画布的视图模式、流线视觉状态与 ghost suggestion 交互契约 |
| `architecture/auth-entitlement-architecture.md` | 桌面登录、授权、远端 `.NET 10` 控制面与本地求解边界 |
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

如果是第一次进入仓库，建议按以下顺序阅读：

1. `../AGENTS.md`
2. `radishflow-architecture-draft.md`
3. `radishflow-startup-checklist.md`
4. `mvp/scope.md`
5. `architecture/auth-entitlement-architecture.md`
6. `architecture/app-architecture.md`
7. `architecture/canvas-interaction-contract.md`
8. `architecture/versioning.md`
9. `capeopen/boundary.md`
10. `adr/0001-branch-and-pr-governance.md`

## 文档维护约定

- 优先更新已有文档，不为单次讨论随意新增散文档
- 重要阶段变化应同步更新对应索引文档和周志
- 周志按 `docs/devlogs/YYYY-Www.md` 命名

## 外部参考

- [CapeOpenCore](https://github.com/laugh0608/CapeOpenCore)
- `D:\Code\Radish`: 当前用于参考 OIDC / Auth / Client Registration 形态的统一平台仓库
- [DWSIM](https://github.com/DanWBR/dwsim): 用于参考 `Interfaces / FlowsheetBase / FlowsheetSolver / UnitOperations` 的职责拆分、自动化测试入口组织方式，以及图形对象与求解对象分离的工程经验

补充约束：

- `DWSIM` 仅作为行为和架构参考，不直接迁移实现代码
- 其 GPL-3.0 许可要求决定了当前仓库不应复制或改写式移植其源码
- 当前阶段只吸收对 `rf-unitops`、`rf-flowsheet`、`rf-solver`、自动化入口与测试组织有帮助的结构经验，不把其完整功能范围提前引入 MVP
