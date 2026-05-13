# RadishFlow Docs

更新时间：2026-05-13

## 先读什么

如果你是第一次进入仓库，默认先读这几个入口：

1. `status/current.md`
2. `../README.md`
3. `guides/studio-quick-start.md`
4. `architecture/overview.md`

阅读原则：

- 想知道“今天做什么”或“当前阶段到哪了”，先读 `status/current.md`
- 想知道“现在怎么用 Studio”，先读 `guides/`
- 想知道“字段、单位、结果是什么意思”，先读 `reference/`
- 想知道“为什么这样分层、边界怎么定”，再读 `architecture/`

`AGENTS.md` / `CLAUDE.md` 只保留长期协作规则，不承担当前说明书职责。

## Guides

Guide 文档回答“怎么做”，优先面向第一次上手和具体操作路径。

| 文档 | 说明 |
| --- | --- |
| `guides/studio-quick-start.md` | 当前 Studio 的启动方式、能力边界和第一次体验入口 |
| `guides/run-first-flowsheet.md` | 用仓库示例走通一次最小求解闭环 |
| `guides/review-solve-results.md` | 在 Studio 中按 source/intermediate/step/outlet 四类对象审阅结果 |
| `capeopen/pme-validation.md` | CAPE-OPEN / PME 人工验证 runbook |

## Reference

Reference 文档回答“字段、单位、结果、格式分别是什么”，不承担架构推演。

| 文档 | 说明 |
| --- | --- |
| `reference/units-and-conventions.md` | 当前稳定的单位、相标签、组成与字段后缀约定 |
| `reference/solve-snapshot-results.md` | `SolveSnapshot`、step 输入/输出与结果 DTO 的稳定语义 |

## Architecture And Boundaries

Architecture 文档回答“系统如何分层、边界为何这样定”，不是产品使用手册。

| 文档 | 说明 |
| --- | --- |
| `architecture/overview.md` | 当前仓库分层、crate 边界与阶段职责 |
| `architecture/app-architecture.md` | 桌面 App 的状态、命令与模块边界 |
| `architecture/canvas-interaction-contract.md` | 画布视图模式、流线状态与 suggestion 契约 |
| `architecture/auth-entitlement-architecture.md` | 桌面登录、授权、控制面与本地求解边界 |
| `architecture/versioning.md` | 版本命名、 tag 与发布轨道约定 |
| `architecture/open-source-references.md` | 可借鉴的开源参考与许可边界 |
| `thermo/mvp-model.md` | 热力学与 `TP Flash` 当前最小模型和数值口径 |
| `capeopen/boundary.md` | Rust Core 与 `.NET 10` CAPE-OPEN 适配层边界 |
| `development/code-style.md` | 跨语言代码风格、命名和抽象判断标准 |

## Status, Scope, Logs

这组文档回答“当前做到哪了、这一阶段做什么、不做什么、最近怎么演进”。

| 文档 | 说明 |
| --- | --- |
| `status/current.md` | 当前阶段、重点、验证基线和按需阅读入口 |
| `mvp/scope.md` | 第一阶段 MVP 冻结范围、非目标与开发节奏 |
| `mvp/alpha-acceptance-checklist.md` | MVP α 验收矩阵、smoke 记录口径和 release blocker 分类 |
| `radishflow-mvp-roadmap.md` | 第一阶段 MVP 路线图 |
| `devlogs/README.md` | 周志索引与命名规则 |

## Governance

| 文档 | 说明 |
| --- | --- |
| `adr/0001-branch-and-pr-governance.md` | 分支、PR 与保护规则治理决策 |
| `../.github/rulesets/master-protection.json` | `master` 保护规则模板 |

## 历史草案与迁移材料

以下文档保留为历史背景、迁移记录或较早期的草案，不再作为默认入口：

| 文档 | 说明 |
| --- | --- |
| `radishflow-architecture-draft.md` | 较早期目标架构草案 |
| `radishflow-startup-checklist.md` | 新仓库启动清单与迁移边界 |
| `radishflow-capeopen-asset-checklist.md` | 从 `CapeOpenCore` 提取 CAPE-OPEN 资产的清单 |

## 文档分层约定

新增或更新文档时，优先按下面的职责落位：

- `guides/`：怎么做
- `reference/`：字段、参数、单位、结果、格式是什么
- `architecture/`：为什么这样设计、边界如何划分
- `status/` / `mvp/` / `devlogs/`：当前阶段、范围和演进记录

不要继续把“使用说明”“字段参考”和“架构边界”混写进同一篇大文档。

## 文档体量约束

文档按“默认阅读成本”和“职责单一性”治理，不按源码行数类比处理。中文 Markdown 优先看字符数、默认入口权重和是否混入历史流水。

| 类型 | 目标上限 | 处理方式 |
| --- | ---: | --- |
| 协作入口：`AGENTS.md` / `CLAUDE.md` | 14k 字符 | 只保留长期规则，阶段内容挪到 `status/current.md` |
| 当前状态入口：`status/current.md` | 8k 字符 | 只保留当前阶段、最近摘要、下一步和按需阅读 |
| 文档目录入口：`README.md` | 10k 字符 | 只做导航和维护规则，不承载长解释 |
| Guide / Runbook | 15k 字符 | 每篇只讲一个任务流，多个任务流拆文档 |
| Reference / Architecture / Boundary | 25k-30k 字符 | 超限时拆成入口摘要和专题正文 |
| ADR | 12k 字符 | 一事一议，不写成历史报告 |
| Devlog / 历史草案 | 可更长 | 不作为默认入口，顶部必须有摘要和索引 |

每篇新增或大改文档应在开头说明：

- 用途：这篇文档解决什么问题
- 读者：什么情况下应该读它
- 不包含：哪些内容应放到其他文档

若一篇文档超过目标上限，优先判断它是否混入了历史流水、使用说明、字段参考或架构推演。能删减时先删减；确实需要保留时，拆到更明确的专题文档或周志。

## 维护约定

- 优先更新已有文档，不为一次性讨论随意新增散文档
- 关键入口文档保持简洁，避免重新膨胀为大杂烩
- 可用 `scripts/check-doc-size.ps1` 或 `scripts/check-doc-size.sh` 查看文档体量报告；该检查默认报告超限项，后续可在 CI 中逐步提升为硬门禁
- 如果代码与文档冲突，先判断是代码偏离文档，还是文档已过期，再统一修正
- 重要阶段变化除了更新专题文档，也应同步更新 `status/current.md`
- 周志按 `docs/devlogs/YYYY-Www.md` 命名

## 外部参考

- [CapeOpenCore](https://github.com/laugh0608/CapeOpenCore)
- [Radish](https://github.com/laugh0608/Radish)
- [DWSIM](https://github.com/DanWBR/dwsim)

补充约束：

- `DWSIM` 仅作行为和架构参考，不直接迁移实现代码
- 其 GPL-3.0 许可决定了当前仓库不应复制或改写式移植其源码
- 当前阶段只吸收对 `rf-unitops`、`rf-flowsheet`、`rf-solver`、自动化入口与测试组织有帮助的结构经验
