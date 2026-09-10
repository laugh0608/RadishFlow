# RadishFlow Docs

更新时间：2026-09-10

## 先读什么

如果你是第一次进入仓库，默认先读这几个入口：

1. `status/current.md`
2. `../README.md`
3. `topics/README.md`
4. `guides/studio-quick-start.md`
5. `architecture/overview.md`

阅读原则：

- 想知道“今天做什么”或“当前阶段到哪了”，先读 `status/current.md`
- 想知道 Agent 怎么协作、哪些操作要授权、如何选择验证，读 `development/agent-collaboration.md`
- 想知道“某个功能或开发目标怎么做、做到哪、怎么验收”，先读 `topics/README.md`
- 想知道“现在怎么用 Studio”，先读 `guides/`
- 想知道“字段、单位、结果是什么意思”，先读 `reference/`
- 想知道“为什么这样分层、边界怎么定”，再读 `architecture/`

`AGENTS.md` / `CLAUDE.md` 只保留启动即生效的长期约束，不承担当前说明书、命令手册或历史记录职责。

## Development Topics

Topics 文档回答“一个功能、能力包或开发目标怎么设计、分阶段推进、验收和验证”。当前业务功能开发已停止，历史 Topic 状态保留为停更前的组织和边界记录；新增未来规划单独标记为未排期，不代表存在已激活的产品开发主线。

| 文档 | 说明 |
| --- | --- |
| `topics/README.md` | 当前专题索引、状态定义和新增专题规则 |
| `topics/topic-template.md` | 新增专题文档模板 |
| `topics/studio-main-workflow.md` | Studio 主工作台与空白项目建模主路径 |
| `topics/property-basis-and-components.md` | 项目物性基础、内置 package 和组分选择 |
| `topics/flowsheet-modeling-and-solve.md` | 流程图建模、连接、readiness、Run Panel 和 solver 闭环 |
| `topics/results-review-diagnostics.md` | 结果审阅、诊断定位和 recovery action |
| `topics/project-lifecycle-storage.md` | 项目打开、保存、另存为、最近项目、sidecar 和脏改确认 |
| `topics/capeopen-pmc-adapter.md` | `.NET 10` CAPE-OPEN / COM PMC 适配层和 PME 验证基线 |
| `topics/unitops/` | Feed、Heater / Cooler、Flash Drum、Mixer、Valve 等单元模块专题 |
| `topics/modeling/` | Material Stream 等建模对象专题 |
| `topics/platform/` | Control Plane 后端服务、后端 Web UI 与账户联合登录专题 |
| `topics/platform/account-and-federated-login.md` | 参考 Radish / RadishMind 的未来账户与登录规划；未排期、待架构决策 |

## Development And Collaboration

这组文档承载按任务读取的稳定开发与协作规则，不复制当前阶段状态。

| 文档 | 说明 |
| --- | --- |
| `development/agent-collaboration.md` | Agent 任务推进、操作授权、环境边界、验证选择和文档归位规则 |
| `development/code-style.md` | 跨语言代码风格、命名、抽象和 review 判断标准 |

## Guides

Guide 文档回答“怎么做”，优先面向第一次上手和具体操作路径。

| 文档 | 说明 |
| --- | --- |
| `guides/studio-quick-start.md` | 当前 Studio 的启动方式、能力边界和第一次体验入口 |
| `guides/run-first-flowsheet.md` | 用仓库示例走通一次最小求解闭环 |
| `guides/author-small-cases.md` | 从 Home 小案例作者入口复现 `Mixer-Flash` / `Heater-Flash` 空白项目路径 |
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
| `architecture/studio-ui-topic-plan.md` | Studio UI 专题阶段的端点边界、信息架构、主工作流和 `.pen` 设计稿规则 |
| `architecture/designs/studio-client-main-brief.md` | 已保留 Studio 主设计稿的文字 brief，设计完成不代表所有界面已实现 |
| `architecture/studio-ui-design-guidelines.md` | Studio 首屏、画布、面板、按钮、文字和结果审阅的 UI 设计规范 |
| `architecture/studio-visual-system.md` | Studio 视觉定位、token、色彩角色、控件状态和视觉验收口径 |
| `architecture/ui-inspiration-reference.md` | AFFINE、CodexApp、Cloudflare、GitHub、Discourse、1Panel 等优秀产品截图的 UI 设计灵感参考 |
| `architecture/auth-entitlement-architecture.md` | 桌面登录、授权、控制面与本地求解边界 |
| `architecture/versioning.md` | 版本命名、 tag 与发布轨道约定 |
| `architecture/open-source-references.md` | 可借鉴的开源参考与许可边界 |
| `thermo/mvp-model.md` | 公式与单元近似、样例身份、数值证据及独立验证缺口 |
| `capeopen/boundary.md` | Rust Core 与 `.NET 10` CAPE-OPEN 适配层边界 |

## Status, Scope, Logs

这组文档回答“当前做到哪了、这一阶段做什么、不做什么、最近怎么演进”。

| 文档 | 说明 |
| --- | --- |
| `status/current.md` | 当前阶段、重点、验证基线和按需阅读入口 |
| `topics/README.md` | 已有能力与历史功能专题索引，不表示当前存在产品排期 |
| `mvp/scope.md` | 已保留的 MVP 范围、模型限制与验收含义 |
| `mvp/alpha-acceptance-checklist.md` | MVP α 验收矩阵、smoke 记录口径和 release blocker 分类 |
| `mvp/beta-acceptance-checklist.md` | MVP β 人工 smoke、通过 / 失败标准和暂不推进项 |
| `radishflow-mvp-roadmap.md` | 历史里程碑与未排期的后续决策参考 |
| `devlogs/README.md` | 按月份分组的周志索引与命名规则 |
| `releases/v26.5.1-dev.md` | 历史 `v26.5.1-dev` 便携 staging 草案和验证记录；不作为当前正式版本节点事实源 |

## Governance

| 文档 | 说明 |
| --- | --- |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | 当前维护状态、潜在贡献边界、许可证与验证要求 |
| [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) | 项目空间中的社区交流规范与行为问题报告方式 |
| [SECURITY.md](../SECURITY.md) | 漏洞私下报告入口、安全问题范围与披露边界 |
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
- `development/`：开发协作、验证选择、命名和实现规范
- `topics/`：一级开发轨道和二级功能专题的设计、范围、验收和验证
- `reference/`：字段、参数、单位、结果、格式是什么
- `architecture/`：为什么这样设计、边界如何划分
- `status/` / `mvp/` / `devlogs/`：当前阶段、范围和演进记录

不要继续把“使用说明”“字段参考”和“架构边界”混写进同一篇大文档。

当前校准已将架构与 MVP 入口中的重复开发流水收敛为历史引用。实现限制与静态风险保留在对应领域真相源，未排期建议不写成已批准方案，历史验收不重标为本次验证；文档更新时间仅表示说明被维护。

## 文档体量约束

文档按“默认阅读成本”和“职责单一性”治理，不按源码行数类比处理。中文 Markdown 优先看字符数、默认入口权重和是否混入历史流水。

| 类型 | 目标上限 | 处理方式 |
| --- | ---: | --- |
| 协作入口：`AGENTS.md` / `CLAUDE.md` | 14k 字符 | 只保留长期规则，阶段内容挪到 `status/current.md` |
| 当前状态入口：`status/current.md` | 8k 字符 | 只保留当前阶段、最近摘要、下一步和按需阅读 |
| 文档目录入口：`README.md` | 10k 字符 | 只做导航和维护规则，不承载长解释 |
| Topics 专题 | 15k-25k 字符 | 每篇只承载一个功能或开发目标，复杂专题拆成子专题 |
| Guide / Runbook / 协作指南 | 15k 字符 | 每篇只讲一个任务流或一类稳定协作规则，多个职责拆文档 |
| Reference / Architecture / Boundary | 25k-30k 字符 | 超限时拆成入口摘要和专题正文 |
| ADR | 12k 字符 | 一事一议，不写成历史报告 |
| Devlog / 历史草案 | 可更长 | 不作为默认入口，默认体量检查只报告受约束文档；需要时用 advisory 检查查看历史材料 |

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
- 周志按 `docs/devlogs/YYYY-MM/YYYY-Www.md` 命名

## 外部参考

- [CapeOpenCore](https://github.com/laugh0608/CapeOpenCore)
- [Radish](https://github.com/laugh0608/Radish)
- [DWSIM](https://github.com/DanWBR/dwsim)

补充约束：

- `DWSIM` 仅作行为和架构参考，不直接迁移实现代码
- 其 GPL-3.0 许可决定了当前仓库不应复制或改写式移植其源码
- 当前阶段只吸收对 `rf-unitops`、`rf-flowsheet`、`rf-solver`、自动化入口与测试组织有帮助的结构经验
