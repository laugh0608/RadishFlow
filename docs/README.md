# RadishFlow Docs

## 文档索引

| 文档 | 说明 |
| --- | --- |
| `architecture/overview.md` | 当前仓库的分层、crate 边界与阶段职责 |
| `architecture/app-architecture.md` | 桌面 App 的壳层、状态与模块边界规划 |
| `mvp/scope.md` | 第一阶段 MVP 的冻结范围、非目标与近期开发节奏 |
| `thermo/mvp-model.md` | 热力学与 `TP Flash` 的最小接口与后续实现顺序 |
| `capeopen/boundary.md` | Rust Core 与 .NET 10 CAPE-OPEN 适配层边界 |
| `adr/0001-branch-and-pr-governance.md` | 分支、PR 与保护规则治理决策 |
| `../.github/rulesets/master-protection.json` | `master` 保护规则模板 |
| `devlogs/README.md` | 按周分文件的开发日志规范 |
| `devlogs/2026-W13.md` | 2026 第 13 周开发日志 |
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
5. `architecture/app-architecture.md`
6. `capeopen/boundary.md`
7. `adr/0001-branch-and-pr-governance.md`

## 文档维护约定

- 优先更新已有文档，不为单次讨论随意新增散文档
- 重要阶段变化应同步更新对应索引文档和周志
- 周志按 `docs/devlogs/YYYY-Www.md` 命名

## 外部参考

- [CapeOpenCore](https://github.com/laugh0608/CapeOpenCore)

