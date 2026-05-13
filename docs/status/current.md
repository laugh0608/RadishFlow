# 当前状态

更新时间：2026-05-13

## 用途

用途：为新会话恢复上下文、判断“今天做什么”提供轻量入口。
读者：人工开发者、用户、AI / Agent。
不包含：完整历史流水、详细设计推演、测试日志和长期说明书。

默认先读本文档。只有当任务需要具体实现细节、历史依据或专题边界时，再读取下方“按需阅读”列表。`AGENTS.md` / `CLAUDE.md` 只保留长期协作规则，不承载当前阶段流水。

## 当前阶段

- 产品定位：以 Rust Core + Rust UI + `.NET 10` CAPE-OPEN/COM 适配层构建稳态流程模拟软件。
- 当前主线：MVP 第一阶段的核心里程碑已越过最小线，当前转入 MVP α 验收与发布硬化，而不是继续主动扩展周边 presentation。
- 当前重点：用用户可复现路径和仓库级验证确认 Rust Studio、数值基线、`rf-ffi` JSON/error 与 CAPE-OPEN / PME 回归基线仍能闭环；只修验收暴露的真实 blocker。
- 当前验证基线：功能改动优先执行相关 focused tests；阶段性收口执行 `pwsh ./scripts/check-repo.ps1`。

## 最近完成摘要

- Studio Canvas 已支持 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum` 的最短可求解建模路径；空白项目也会补 MVP 默认二元组件与本地 `binary-hydrocarbon-lite-v1` 物性包缓存。
- Result Inspector / Active Inspector 已形成 stream-centric 与 unit-centric 两个消费面，可审阅当前 `SolveSnapshot` 内输入/输出流股、相结果、关联步骤、关联诊断和关键 `T / P / F / H` 摘要。
- `rf-thermo` / `rf-flash` 已补 MVP 常热容显热焓值、bubble/dew pressure 与 fixed-pressure bubble/dew temperature 边界估算；`TP Flash` 会显式物化 `liquid-only / two-phase / vapor-only` phase region 与对应边界窗口。
- `tests/thermo-golden`、`tests/flash-golden`、Rust integration 与 Studio focused tests 已把 official `binary-hydrocarbon-lite-v1` near-boundary `±ΔP / ±ΔT` case 前推到 `Feed/Heater/Cooler/Valve/Mixer -> Flash` 正式链路。
- `rf-model::MaterialStreamState`、`rf-solver`、`rf-ui::SolveSnapshot` 与 Studio consumer 现在会透传结构化 `bubble_dew_window`、phase result 与 overall molar enthalpy；source stream、非 flash 中间流股、flash inlet consumed stream 和 outlet stream 继续走同一份 DTO 语义。
- `rf-solver::UnitSolveStep` 已移除内部 `consumed_stream_ids / produced_stream_ids` 并改为从结构化 stream 快照派生；`rf-ffi` 与 Studio 若仍需 id 列表，也只把它们视为 presentation / interop 派生字段。
- official / synthetic fixture 语义已基本收口：official hydrocarbon case 统一使用 `binary-hydrocarbon-lite-v1` 与 `methane / ethane`，intentional synthetic demo 族改用显式 `binary-hydrocarbon-synthetic-demo-v1` 与对应文件名 / helper。
- Stream Inspector 已补 composition normalize、draft discard、受控组分添加 / 删除和运行前组成校验；写回、归一化和丢弃草稿都必须由用户显式触发，不做隐式自动补偿。
- 仓库基础治理已补齐根 `README.md`、`.gitattributes`、`.gitignore`、文本格式门禁、代码规范专题文档和文档篇幅治理规则；默认入口文档应继续保持摘要化。
- 阶段性仓库验证已恢复通过：failure fixture 换行已归一化，既有 Rust 格式漂移已由 `cargo fmt --all` 收口，`pwsh ./scripts/check-repo.ps1` 当前通过。
- synthetic `liquid-only / vapor-only` near-boundary 单相样例已补齐到 raw solver、Studio solver bridge 和 workspace run path 的 `Feed/Heater/Cooler/Valve/Mixer -> Flash` dedicated 回归，继续锁定 source / intermediate / flash inlet / outlet 的同一份 DTO 语义。
- Studio 最终 runtime 渲染面已补 synthetic 单相 `Feed/Heater/Cooler/Valve/Mixer -> Flash` 覆盖，继续锁定 Result Inspector / Active Inspector 的窗口 section、overall H 和零流量对侧 outlet 不展示伪窗口。
- `TP Flash` boundary drift / tolerance-focused 覆盖已完成一轮盘点与补强：official / synthetic golden 当前仍覆盖 `±ΔP / ±ΔT`，raw solver 另补边界容差带内的 phase region / zero-fraction phase materialization 回归；`rf-flash` 继续复用 `rf-types` 的 tolerance 语义，不引入第二套窗口估算或 fallback。
- `rf-ffi` thermo solve 失败基线已复核并补强：成功路径会继续导出同一份 `bubble_dew_window` / overall enthalpy JSON，运行时物性包若能加载但缺少求解所需热容，会在 `flowsheet_solve` 阶段稳定返回 `Thermo` 状态与结构化 last-error。
- synthetic 单相 near-boundary 的 `SolveSnapshot -> window_model` consumer 已补 focused 覆盖：`Feed/Heater/Cooler/Valve/Mixer -> Flash` 现在会锁定 Result Inspector、comparison、unit result、Active Inspector 与 `inspector.focus_*` diagnostic action 都继续消费同一份 flash inlet / outlet / unit DTO。
- synthetic 单相 near-boundary 的 shell selector state 已补 focused 覆盖：flowing outlet / zero-flow outlet 之间切换和重新挂 comparison 时，会继续保持 flash unit 选择、窗口缺席语义和 comparison DTO 一致。
- Stream Inspector component catalog / presentation 已完成一轮边界收口：可添加组件继续只从 `flowsheet.components` 中尚未出现在当前流股组成的条目派生；component add/remove、normalize、discard 与运行前组成校验仍沿现有 DTO / 命令语义工作，已提交但未归一化的组成在 presentation 中标为 `Unnormalized`，不再和未提交 `Draft` 混淆。
- `SolveSnapshot` consumer 的 command surface 边界已补一层收口：最新求解快照中的 result stream / unit 目标现在以 `Results` command section 暴露，并已覆盖 palette、menu 和 command list 到同一条 `inspector.focus_*` host dispatch，不新增 shell 私有结果缓存或导航分支。
- `SolveSnapshot` consumer 的 runtime 点击交互已补 focused 覆盖：Result Inspector / diagnostic action 共用的小型 command action button 现在用真实 `egui` pointer click 回归锁定到同一条 `dispatch_ui_command -> inspector.focus_* -> Active Inspector` 链路，不新增 runtime 私有解释层。
- 2026-05-12 阶段复盘结论：近期 focused 收口仍在整体规划内，但继续沿 near-boundary / command surface / runtime click 细节扩测试会进入收益递减；下一轮应切到 MVP α 验收矩阵和交付硬化。
- 2026-05-13 已新增 `docs/mvp/alpha-acceptance-checklist.md`，把自动化验证、Studio 用户视角 smoke、`rf-ffi` JSON/error、CAPE-OPEN / PME 与文档复现检查收束为 MVP α 验收入口。
- 2026-05-13 阶段性自动验证已通过：`git diff --check`、`pwsh ./scripts/check-doc-size.ps1` 与 `pwsh ./scripts/check-repo.ps1` 均已执行；文档体量脚本仅报告既有 roadmap / 历史周志超限项。
- 2026-05-13 人工启动 Studio 后暴露首屏 UX blocker：顶部调试信息、命令大全和布局控制压过主路径；已把最终 shell 收敛为快速操作入口，默认显示 `打开示例 / 打开项目 / 运行 / 保存 / 命令面板`，并默认隐藏左侧命令大全。
- 2026-05-13 人工点击顶部 `运行` 和启动聚焦后暴露新的 smoke blocker：GUI 回调异常时缺少外层防护，控制台没有用户操作 / 求解审计输出，且 Windows debug 构建在 bootstrap runtime dispatch 上出现栈溢出；已补 GUI panic 防护、命令可用性门控、默认 stderr 审计线，停止由 viewport focus 自动派发 foreground entitlement tick，并把默认隐藏 Commands 面板改为 shell 启动时的 host-local transient layout preference，不再通过启动时 `SetPanelVisibility` dispatch 实现。GUI shell 启动 / 打开项目现在跳过自动 entitlement preflight，Windows `radishflow-studio` 二进制显式保留 16 MiB 主线程栈，以适配 eframe 必须在主线程创建事件循环的约束；此前尝试把 `eframe` 放到后台 UI 线程会触发 `winit` 主线程约束，已回退。
- 2026-05-13 关闭 Studio 时发现最后一帧会短暂显示默认 Commands 左栏；根因是关闭最后一个逻辑窗口后仍继续渲染当帧，`window_model()` 回退到默认布局。当前已在 viewport close 处理返回“停止渲染”信号，最后窗口关闭时直接结束当帧，且不再对最后窗口关闭请求发送 `CancelClose`，避免黑屏但进程不退出。

完整过程和每日验证记录见 `docs/devlogs/2026-W20.md` 以及更早周志。

## 下一步建议

1. 下一步先做 Studio UI 优化和规范化：收敛首屏层级、快速操作条、左右面板默认状态、Runtime / Result Inspector / Active Inspector 的信息密度和状态文案，让打开示例、运行、查看结果和保存重开成为清晰主路径。
2. UI 规范化仍只服务 MVP α 验收，不扩自由连线编辑器、完整拖拽布局编辑器、完整报表系统或新的求解范围；结果面继续只读消费 `SolveSnapshot`，不新增 shell 私有结果缓存。
3. UI 规范化后按 `docs/mvp/alpha-acceptance-checklist.md` 复跑 Studio 用户视角手动 smoke，确认新首屏、运行、控制台审计、结果审阅、保存重开和窗口关闭都稳定。
4. 只修人工 smoke 或仓库级验证暴露的真实 blocker；若只是收益递减的 focused 覆盖缺口，先记录而不是继续主动扩矩阵。
5. 保持 `TP Flash` official / synthetic golden、raw solver focused tolerance 与 `rf-ffi` JSON/error 基线稳定，但不主动扩无限 near-boundary 矩阵。

## 暂不推进

- 不扩自由连线编辑器、拖拽布局编辑器、自动布线、视口持久化或完整结果报表。
- 不把 CAPE-OPEN / COM 语义倒灌到 Rust Core。
- 不引入第三方 CAPE-OPEN 模型加载。
- 不把 smoke test driver、PME 调试路径或单个宿主兼容逻辑提升为通用库 API。
- 不为未来可能需求预先堆叠不明意义的 helper / manager / orchestrator / context / adapter。
- 不再主动扩 near-boundary / command surface / runtime click 的细枝末节测试；除非它们直接暴露 MVP α 验收 blocker。

## 按需阅读

- 需要仓库全局模块边界：`docs/architecture/overview.md`
- 需要 MVP 范围和非目标：`docs/mvp/scope.md`
- 需要 MVP α 验收矩阵：`docs/mvp/alpha-acceptance-checklist.md`
- 需要最新流水和决策依据：`docs/devlogs/2026-W20.md`
- 需要热力学 / 闪蒸细节：`docs/thermo/mvp-model.md`
- 需要 CAPE-OPEN / COM 边界：`docs/capeopen/boundary.md`
- 需要桌面 App / Canvas 交互契约：`docs/architecture/app-architecture.md`、`docs/architecture/canvas-interaction-contract.md`
- 需要代码风格、命名或抽象判断：`docs/development/code-style.md`
- 需要文档篇幅和拆分规则：`docs/README.md`

## 更新规则

- 本文档目标上限为 8k 字符；超过上限时应优先删减历史流水、重复背景和过细实现细节。
- 本文档只保留当前阶段、最近完成摘要、下一步建议、暂不推进项和按需阅读入口。
- 历史流水写入周志；长期边界写入专题文档；不要把本文档写成长篇进度报告。
- 协作入口文件只保留长期稳定规则；阶段性变化优先更新本文档和对应专题文档，再按需同步入口文件中的引用关系。
- 每次完成重要阶段收口后，优先更新本文档顶部状态和“下一步建议”。
