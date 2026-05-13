# MVP Alpha Acceptance Checklist

更新时间：2026-05-13

## 用途

用途：为 MVP α 验收提供统一检查清单、执行记录口径和 release blocker 分类。
读者：准备执行 MVP α 验收的开发者、验证人员和 AI / Agent。
不包含：详细架构推演、完整测试日志、PME 安装教程和未来功能规划。

本文档用于回答“这一轮 α 是否已经具备可复现交付条件”。它不替代 `docs/guides/` 的操作说明，也不扩张 `docs/mvp/scope.md` 已冻结的 MVP 范围。

## 验收原则

- 以用户可复现路径为主，不再主动扩 near-boundary / command surface / runtime click 的开放式细节矩阵。
- 自动化验证先以 `pwsh ./scripts/check-repo.ps1` 为阶段性入口；若某项失败，先按真实 blocker 分类，不直接补兜底。
- 手动 smoke 只记录能代表 MVP α 闭环的路径，不追求覆盖完整产品体验。
- 涉及 COM 注册、PME 启动、Windows Registry 写入或外部宿主操作时，继续遵守 `docs/capeopen/pme-validation.md` 的门控，不把这些动作纳入默认自动验证。
- 验收只确认当前冻结 MVP：二元体系、最小物性包、`TP Flash`、`Feed / Mixer / Heater / Cooler / Valve / Flash Drum`、JSON 项目、Rust Studio 最小工作台和自有 CAPE-OPEN Unit Operation PMC。

## 状态标记

| 标记 | 含义 |
| --- | --- |
| `Pass` | 已按本文口径验证通过 |
| `Fail` | 验收路径失败，且需要修复或明确降级 |
| `Blocked` | 受外部环境、权限、工具安装或人工资源阻塞 |
| `Deferred` | 明确不属于本轮 MVP α 入口条件 |
| `Pending` | 尚未执行 |

## Release Blocker 分类

| 分类 | 示例 |
| --- | --- |
| `StudioOpenRunSave` | Studio 无法打开示例、运行、保存或重开项目 |
| `StudioModelingPath` | 三条最短建模路径无法由用户动作走到可求解状态 |
| `ResultReview` | Result Inspector / Active Inspector 无法审阅当前 `SolveSnapshot` 的核心结果 |
| `StreamInspectorGuard` | 未提交草稿或未归一组成未能阻断运行，或 normalize / discard 行为漂移 |
| `NumericalBaseline` | official / synthetic 数值基线、phase region、`bubble_dew_window` 或 enthalpy 漂移 |
| `FfiJsonError` | `rf-ffi` JSON 导出、错误状态或 last-error 结构化语义漂移 |
| `CapeOpenPme` | 自有 PMC discovery / activation / validate / calculate 回归 |
| `DocsRepro` | quick start、运行指南、结果审阅或 PME runbook 不足以复现验收路径 |
| `Environment` | 沙盒、缺少外部 PME、权限或 native path 导致无法判断代码状态 |

## 自动化验证矩阵

| 项目 | 命令或入口 | 当前状态 | 通过标准 | 记录 |
| --- | --- | --- | --- | --- |
| 仓库级验证 | `pwsh ./scripts/check-repo.ps1` | Pass | Rust / 文本 / 仓库治理基线通过 | 2026-05-13 已通过；输出 `Repository checks passed.` |
| 文本格式检查 | `git diff --check` | Pass | 无 whitespace error | 2026-05-13 已通过 |
| 文档体量报告 | `pwsh ./scripts/check-doc-size.ps1` | Pass | 默认入口未重新膨胀；既有超限项可解释 | 2026-05-13 已运行；仍只报告既有 roadmap 与历史周志超限 |
| `rf-ffi` JSON/error 基线 | `pwsh ./scripts/check-repo.ps1` 覆盖；必要时补 `cargo test -p rf-ffi` | Pass | solve snapshot / stream JSON 与 structured error 回归稳定 | 2026-05-13 仓库级验证通过 |
| official / synthetic 数值基线 | `pwsh ./scripts/check-repo.ps1` 覆盖 | Pass | golden、raw solver 与 Studio focused 回归稳定 | 2026-05-13 仓库级验证通过 |

## Studio 手动 Smoke

### Smoke A：打开示例、运行、审阅、保存重开

| 项 | 记录 |
| --- | --- |
| 状态 | Pending |
| 推荐项目 | `examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json` |
| 步骤 | 启动 Studio；打开项目；执行运行；查看 Runtime / Result Inspector / Active Inspector；保存；重开 |
| 通过标准 | 运行成功生成最新 `SolveSnapshot`；能审阅 `T / P / F / H`、phase rows 与 `bubble_dew_window`；保存和重开不破坏项目或 sidecar |
| Blocker 分类 | `StudioOpenRunSave`、`ResultReview`、`DocsRepro` |
| 记录 | 待人工执行 |

### Smoke B：空白建模最短闭环

| 项 | 记录 |
| --- | --- |
| 状态 | Pending |
| 推荐路径 | `Feed -> Flash Drum`，再按需验证 `Feed -> Heater/Cooler/Valve -> Flash Drum` 与 `Feed + Feed -> Mixer -> Flash Drum` |
| 步骤 | 从空白项目放置单元；显式接受 Canvas suggestion；运行；查看 flash outlet |
| 通过标准 | 空白项目具备 MVP 默认二元组件与本地 `binary-hydrocarbon-lite-v1` 缓存；suggestion 可补齐最短链路；运行可收敛或给出结构化诊断 |
| Blocker 分类 | `StudioModelingPath`、`NumericalBaseline`、`ResultReview` |
| 记录 | 待人工执行 |

### Smoke C：Stream Inspector 组成阻断

| 项 | 记录 |
| --- | --- |
| 状态 | Pending |
| 推荐项目 | 任一 official hydrocarbon 示例 |
| 步骤 | 修改流股组成形成未提交草稿；尝试运行；提交为未归一组成；再次尝试运行；显式 normalize；再次运行 |
| 通过标准 | 未提交草稿被标记为 `Draft` 并阻断运行；已提交但未归一组成被标记为 `Unnormalized` 并阻断运行；normalize 后可继续运行 |
| Blocker 分类 | `StreamInspectorGuard`、`StudioOpenRunSave` |
| 记录 | 待人工执行 |

## `rf-ffi` / CAPE-OPEN 验收

| 项目 | 当前状态 | 通过标准 | 记录 |
| --- | --- | --- | --- |
| `rf-ffi` solve success JSON | Pending | `flowsheet_solve` 后 stream/full snapshot JSON 保留 `bubble_dew_window`、phase list 与 overall enthalpy | 自动化基线覆盖，必要时补 focused test |
| `rf-ffi` solve failure error | Pending | runtime 物性包缺关键热容时在 solve 阶段返回 `Thermo` 与结构化 last-error，不生成可导出 snapshot | 自动化基线覆盖，必要时补 focused test |
| CAPE-OPEN contract / sample host | Pending | contract tests 与 sample host 可消费正式 host-facing 面 | 按 `docs/capeopen/pme-validation.md` 执行 |
| PME 人工复验 | Pending | 目标 PME discovery、activation、placement、validate、calculate 与 unregister 路径可复现 | 外部 PME 与 registry 操作不纳入默认自动验证 |

## 文档复现检查

| 文档 | 当前状态 | 通过标准 | 记录 |
| --- | --- | --- | --- |
| `docs/guides/studio-quick-start.md` | Pass | 能说明启动方式、当前能力和首次体验入口 | 2026-05-13 已同步顶部快速操作、默认隐藏 Commands、运行反馈和关闭口径；明确当前是开发态启动，不暗示正式安装包 |
| `docs/guides/run-first-flowsheet.md` | Pass | 能指导用户打开示例、运行、审阅、保存重开 | 2026-05-13 已同步 `Open Example / Open Project / Run / Save / Command Palette` 主路径和 stderr 排查信号 |
| `docs/guides/review-solve-results.md` | Pass | 能解释 source / intermediate / step / outlet 结果审阅顺序 | 2026-05-13 已复查 |
| `docs/capeopen/pme-validation.md` | Pass | 能说明 PME 验证门控、dry-run、register/unregister 和记录模板 | 2026-05-13 已复查；外部 PME 与 registry 操作仍需人工门控 |
| 发布包形态说明 | Pass | 能说明当前仍是开发态或压缩包式交付边界，不暗示已存在完整安装器 | 2026-05-13 已确认 `docs/mvp/scope.md` 与 `docs/architecture/versioning.md` 保留当前发布边界；正式发布操作清单仍是后续事项 |

## 今日执行记录

### 2026-05-13

| 项目 | 状态 | 记录 |
| --- | --- | --- |
| 建立 MVP α 验收清单 | Pass | 新增本文档，先把自动化验证、Studio smoke、`rf-ffi` / CAPE-OPEN 与文档复现检查收束为同一套验收口径 |
| 仓库级验证 | Pass | `git diff --check`、`pwsh ./scripts/check-doc-size.ps1` 与 `pwsh ./scripts/check-repo.ps1` 已执行；仓库级验证通过，文档体量脚本仅报告既有超限项 |
| 用户视角 smoke | Pending | 待人工启动 Studio 执行；默认不在非交互验证中启动桌面 UI |
| release blocker | Pass | 2026-05-13 人工启动后发现首屏信息过载、打开示例和运行入口不清晰，已按 `StudioOpenRunSave` UX blocker 修复：顶部新增快速操作条，命令大全默认隐藏 |
| 运行与关闭 smoke blocker | Pass | 2026-05-13 人工点击顶部运行后暴露 GUI 回调异常、缺少控制台审计、Windows debug 栈溢出和最后窗口关闭异常；已补 GUI panic 降级、命令可用性门控、默认 stderr 审计线、主线程栈保留、跳过启动 entitlement preflight，并修正最后 viewport close 不再拦截原生关闭请求 |
| UI 规范化后续 | Pending | 当前 blocker 已收敛；下一步先做 Studio UI 信息层级、面板密度、主路径按钮状态、结果/日志展示和视觉一致性规范化，不扩 MVP 求解范围或自由连线编辑器 |

## 下一步

1. 下一轮先做 Studio UI 优化和规范化，优先收敛首屏层级、快速操作条、左右面板默认状态、Runtime / Result Inspector / Active Inspector 的信息密度和状态文案。
2. UI 规范化后按 Smoke A / B / C 至少完成两条用户视角记录。
3. 若人工 smoke 暴露 blocker，按分类修复；若没有 blocker，进入 MVP α 发布包形态与发布说明收口。
4. 后续补发布操作清单时，继续沿 `docs/architecture/versioning.md` 的 tag / release 轨道口径展开，不在 quick start 中暗示已存在完整安装器。
