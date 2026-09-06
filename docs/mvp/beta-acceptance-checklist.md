# MVP Beta Acceptance Checklist

更新时间：2026-09-06

> 本文保留历史验收标准与当时记录，`Pass` 只说明对应提交和场景通过，不自动外推到当前 Windows / GUI 状态或物理准确性。业务开发停止，维护范围见 [当前状态](../status/current.md)，数值证据边界见 [热力学模型](../thermo/mvp-model.md)。

## 用途

用途：为 MVP β 人工 smoke 与阶段验收提供统一检查清单、通过 / 失败标准和暂不推进项。
读者：准备执行 MVP β 人工复验的开发者、验证人员和 AI / Agent。
不包含：tag 决策、release notes、便携包刷新、完整 UI 视觉验收、PME 注册教程或未来功能规划。

本文回答“失败修复闭环 v0 focused 收口之后，人工应如何最小复验 MVP β 是否真的能被用户走通”。它不替代 `docs/guides/author-small-cases.md` 的操作说明，也不扩张 `docs/mvp/scope.md` 已冻结的范围。

## 当前结论

截至 2026-05-28，MVP β 已完成以下 focused 收口：

- 小案例作者体验 v0
- 建模输入能力 v0
- 结果核对与案例说明 v0 第一版
- 受控连接恢复 v0
- 剩余单元建模闭环 v0
- 失败修复闭环 v0

当前 `718b03b` 已完成人工 Smoke A-D，未发现 blocker。2026-05-28 真实环境 `pwsh ./scripts/check-repo.ps1` 已在 smoke 记录同步工作区通过。在明确发布节点前，不把当前状态升级为正式 tag / release 节点。

截至 2026-05-31，通用小流程建模 v1 已继续覆盖普通空白项目三类受控路径的结果审阅、关键结果合理性、失败态定位和 case-level review summary。本文定义的 MVP β Smoke A-D 后续保留为阶段收口或发布前复验清单，不再作为日常开发 gate；日常推进优先使用与改动风险匹配的 focused 验证。

## 验收原则

- 验收以用户能复现的小流程为主，不继续追逐 hover、按钮文案、局部 selector 或同构作者入口。
- 人工 smoke 只覆盖代表性主路径和代表性失败恢复，不把 focused tests 已覆盖的 12 条恢复生命周期全部手工重跑。
- 自动化验证仍以 `pwsh ./scripts/check-repo.ps1` 为阶段基线；若 smoke 暴露代码问题，先修根因，再复跑与风险匹配的 focused 验证。
- 所有建模语义变更必须继续走正式 `DocumentCommand` / validation / undo；Canvas 布局和 viewport 只作为 sidecar 或 shell-local 状态检查。
- 本文不触发 COM 注册、PME 自动化、第三方物性包加载、打包、发布或 tag。

## 状态标记

| 标记 | 含义 |
| --- | --- |
| `Pass` | 已按本文口径验证通过 |
| `Fail` | 验收路径失败，且需要修复或明确降级 |
| `Blocked` | 受外部环境、权限、工具安装或人工资源阻塞 |
| `Deferred` | 明确不属于 MVP β 人工 smoke v0 |
| `Pending` | 尚未执行 |

## Blocker 分类

| 分类 | 判定口径 |
| --- | --- |
| `StudioLaunch` | Studio 无法从开发态或 IDE 正常启动，或启动后无法进入 Home / Workbench 主路径 |
| `OpenRunReview` | official demo case 无法打开、运行、审阅结果、保存或重开 |
| `AuthoringPath` | `Mixer-Flash` 或 `Heater-Flash` 作者路径无法从空白项目手工走到可求解状态 |
| `ModelingInput` | 物性包、项目组分、Feed composition 或 Unit 参数无法显式提交并保存 / 重开 |
| `ConnectionRecovery` | 受控断开、重连、删除或 recovery action 无法修复代表性连接问题，或破坏项目 |
| `FailureRecovery` | 缺 composition、参数越界、缺物性包等真实 blocker 无结构化诊断、无可定位目标，或修复后无法 rerun |
| `ResultReview` | 最新 `SolveSnapshot` 缺少关键流股、unit step、flash 分割、相态或 `H` 核对对象 |
| `Persistence` | 保存 / 重开后丢失物性包、组分、composition、单元参数、连接或 sidecar 布局 |
| `DocsRepro` | `guides/` 或本文说明无法支撑人工复现路径 |
| `Environment` | 沙盒、GUI、权限、原生文件选择器或本机工具链问题导致无法判断代码状态 |

UI 观感小问题、局部文案不顺、非阻断 tooltip、低频面板排序和未进入本文路径的功能缺口，不默认列为 MVP β blocker。

## 自动化前置验证

| 项目 | 当前状态 | 通过标准 | 记录 |
| --- | --- | --- | --- |
| 仓库级验证 | Pass | `pwsh ./scripts/check-repo.ps1` 通过 | 2026-05-28 真实环境已在 smoke 记录同步工作区通过 |
| 失败修复 focused | Pass | `cargo test -p radishflow-studio failure_recovery_lifecycle` 覆盖缺物性包、缺组分、缺 composition、参数越界、连接 blocker、cycle 与 invalid port signature 生命周期 | 2026-05-28 已通过，作为人工 smoke 的前置信心，不替代人工主路径复验 |
| 文档口径 | Pass | `docs/status/current.md`、本文、`docs/devlogs/2026-05/2026-W22.md` 不再把下一步指向已完成的失败修复闭环 | 2026-05-28 建立本文 |

## 最小人工 Smoke 路径

### Smoke A：official demo 打开、运行、审阅、保存重开

| 项 | 记录 |
| --- | --- |
| 状态 | Pass |
| 推荐项目 | `examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json`，再抽查 `examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json` |
| 步骤 | 启动 Studio；打开示例；执行 `运行`；查看右侧 `结果`、底部 `结果表` 和当前对象 `检查器`；保存到临时项目；重开并 rerun |
| 通过标准 | 运行成功生成最新 `SolveSnapshot`；能审阅输入流股、非 flash 中间流股、unit step、flash liquid / vapor、相态和 `H`；保存 / 重开不破坏项目或 sidecar |
| 失败标准 | 无法打开 / 运行 / 保存 / 重开；结果只显示空壳或过期快照；flash step 消费错误流股；保存后丢失参数或连接 |
| Blocker 分类 | `OpenRunReview`、`ResultReview`、`Persistence`、`DocsRepro` |
| 记录 | 2026-05-28 人工执行通过；打开、运行、结果审阅、保存 / 重开 / rerun 未发现 blocker |

### Smoke B：Mixer-Flash 空白作者路径

| 项 | 记录 |
| --- | --- |
| 状态 | Pass |
| 入口 | Home `创建 Mixer-Flash 小案例` |
| 步骤 | 从空白项目显式选择 `binary-hydrocarbon-lite-v1`、methane、ethane；放置两个 `Feed`、一个 `Mixer`、一个 `Flash Drum`；通过 suggestion 创建 / 连接流股；提交两股 Feed composition、Feed 参数、Mixer outlet pressure、Flash Drum flash T/P；运行；复制或导出当前快照；保存、重开、rerun |
| 通过标准 | 清单状态从项目真实 canvas / input / snapshot 推导；运行收敛；mixer outlet 流量为两股 Feed 之和；composition 为摩尔流量加权结果；Flash Drum 消费 mixer outlet；导出文本与右侧结果来自同一份快照 |
| 失败标准 | 作者入口自动写项目语义；空白项目被预填物性包或组分；suggestion 无法完成双入口连接；参数或组成只停留在草稿；保存重开后 rerun 漂移 |
| Blocker 分类 | `AuthoringPath`、`ModelingInput`、`ResultReview`、`Persistence` |
| 记录 | 2026-05-28 人工执行通过；空白作者路径、输入提交、运行、导出 / 保存 / 重开 / rerun 未发现 blocker |

### Smoke C：Heater-Flash 空白作者路径与缺 composition 恢复

| 项 | 记录 |
| --- | --- |
| 状态 | Pass |
| 入口 | Home `创建 Heater-Flash 小案例` |
| 步骤 | 从空白项目显式选择 package 和项目组分；放置 `Feed -> Heater -> Flash Drum` 并创建 liquid / vapor outlet；先故意不提交 Feed composition 后运行；确认诊断定位到被消费的 stream；通过 recovery / Inspector 补齐 composition 并 normalize；提交 Feed、Heater、Flash Drum 参数；运行、保存、重开、rerun |
| 通过标准 | 缺 composition 时产生 `solver.step.stream_input`，并能定位到相关 stream / inlet；补齐后 rerun 收敛；Heater outlet 温压反映已提交参数；Flash Drum 消费 heater outlet；vapor-only / two-phase 结果按 guide 可核对 |
| 失败标准 | 缺 composition 被下游单元泛化失败吞掉；recovery target 聚焦错误对象；补齐后仍无法保存 / 重开 / rerun；Flash Drum 直接消费 Feed outlet |
| Blocker 分类 | `AuthoringPath`、`ModelingInput`、`FailureRecovery`、`ResultReview`、`Persistence` |
| 记录 | 2026-05-28 人工执行通过；缺 Feed composition 诊断与恢复、补齐后运行 / 保存 / 重开 / rerun 未发现 blocker |

### Smoke D：受控连接恢复代表路径

| 项 | 记录 |
| --- | --- |
| 状态 | Pass |
| 推荐项目 | Smoke C 成功后的 Heater-Flash 临时项目 |
| 步骤 | 选中 heater outlet stream；执行 `Disconnect sink`；确认 rerun 暴露可理解的未绑定入口或连接诊断；执行 `Reconnect stream`；保存、重开、rerun |
| 通过标准 | 断开 / 重连通过正式 command 进入 undo；`Reconnect stream` 只在唯一合法候选时可用；保存 / 重开后 Flash Drum 继续消费 heater outlet 并运行成功 |
| 失败标准 | 断开或重连绕过文档历史；自动猜错端点；形成 cycle；重开后连接丢失或 rerun 失败 |
| Blocker 分类 | `ConnectionRecovery`、`FailureRecovery`、`Persistence` |
| 记录 | 2026-05-28 人工执行通过；selected stream 断开 / 诊断 / 重连 / 保存 / 重开 / rerun 未发现 blocker |

## 整体通过标准

MVP β 人工 smoke v0 可记为 `Pass`，需要同时满足：

- Smoke A-D 均为 `Pass`，或有明确 `Blocked` 环境原因且不影响判断代码状态。
- 没有 `AuthoringPath`、`ModelingInput`、`FailureRecovery`、`ResultReview` 或 `Persistence` 类未修复 blocker。
- 人工记录中能说明使用的提交、启动方式、项目路径和关键结果核对点。
- 若修复了 blocker，相关 focused 验证和必要仓库级验证已重新通过。

出现以下任一情况，应记为 `Fail` 并先修复：

- 用户无法从空白项目完成 `Mixer-Flash` 或 `Heater-Flash` 主路径。
- 缺输入或错连场景没有稳定诊断、定位或恢复入口。
- 运行成功但关键 `SolveSnapshot` 对象缺失或互相矛盾。
- 保存 / 重开破坏建模输入、连接、参数或结果复现。
- 文档说明与当前 UI / 行为明显冲突，导致 smoke 无法复现。

## 暂不推进

本轮 smoke 已完成。后续仍不推进：

- tag、release notes、便携包刷新或对外发布自动化
- Windows 安装器、自动更新、正式 release channel
- COM 注册、PME 自动化互调、第三方 CAPE-OPEN 模型或第三方物性包加载
- `Cooler-Flash` / `Valve-Flash` Home 作者入口，或绑定草稿 UI 细节的专门操作 guide
- 自由连线编辑器、任意端口选择器、自动布线、完整拖拽布局编辑器
- 完整结果报表、跨快照报表、打印系统或模板导出
- hover、tooltip、按钮文案、局部 selector、视觉微调等非 blocker UI 打磨

后续可放宽的事项：

- `Cooler` / `Valve` 可作为通用空白项目受控路径继续补说明或 focused 回归，但仍不新增 Home 作者入口。
- 可补服务建模正确性的状态表达，例如旧结果失效提示、结果新旧状态标识和单次 `SolveSnapshot` 轻量审阅摘要；这些不视为完整报表系统。

## 记录模板

| 日期 | 提交 | Smoke | 状态 | 记录 |
| --- | --- | --- | --- | --- |
| 2026-05-28 | `19d8986` | 自动化前置 | Pass | 真实环境 `pwsh ./scripts/check-repo.ps1` 已通过；本文建立最小人工 smoke 与验收标准 v0 |
| 2026-05-28 | `718b03b` | Smoke A-D | Pass | 人工启动 Studio 执行 official demo、Mixer-Flash 作者路径、Heater-Flash 缺 composition 恢复与受控连接恢复，未发现 blocker |

## 相关文档

- `docs/status/current.md`
- `docs/guides/author-small-cases.md`
- `docs/guides/review-solve-results.md`
- `docs/guides/studio-quick-start.md`
- `docs/mvp/scope.md`
- `docs/devlogs/2026-05/2026-W22.md`
