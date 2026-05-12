# 当前状态

更新时间：2026-05-12

## 用途

用途：为新会话恢复上下文、判断“今天做什么”提供轻量入口。
读者：人工开发者、用户、AI / Agent。
不包含：完整历史流水、详细设计推演、测试日志和长期说明书。

默认先读本文档。只有当任务需要具体实现细节、历史依据或专题边界时，再读取下方“按需阅读”列表。`AGENTS.md` / `CLAUDE.md` 只保留长期协作规则，不承载当前阶段流水。

## 当前阶段

- 产品定位：以 Rust Core + Rust UI + `.NET 10` CAPE-OPEN/COM 适配层构建稳态流程模拟软件。
- 当前主线：在已具备最短可运行建模路径和 Stream Inspector 组成编辑安全边界后，转回数值主线和结果审阅收口。
- 当前重点：保持 Canvas 与 Stream Inspector 不继续横向扩张，把精力放到 `SolveSnapshot` 结果消费、热力学 / 闪蒸基础能力和可验证闭环上。
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

完整过程和每日验证记录见 `docs/devlogs/2026-W20.md` 以及更早周志。

## 下一步建议

1. 优先跑一轮阶段性仓库验证：`pwsh ./scripts/check-repo.ps1`。若出现疑似环境性失败，再区分真实代码回归与沙盒/本机环境差异。
2. 若继续推进 `rf-thermo` / `rf-flash`，保持 golden 目录、focused tests、integration tests 与 Studio consumer 使用同一套 near-boundary phase region / bubble-dew window / enthalpy 语义，不分叉第二套估算或判断路径。
3. 若继续推进 `SolveSnapshot` consumer，优先沿 selector state、focus command、diagnostic action 和 runtime 渲染这条正式消费链补薄弱环节，不新增 shell 私有状态机。
4. 若继续推进 Stream Inspector，优先收紧 flowsheet component catalog / presentation 边界；不要提前做完整组件库、项目级组件删除迁移或隐式差值补偿。
5. 若发现入口文档继续膨胀，先瘦身 `docs/status/current.md` 和对应专题文档，再把历史流水写入周志；不要把长篇背景写回 `overview.md`、`scope.md` 或协作入口文件。

## 暂不推进

- 不扩自由连线编辑器、拖拽布局编辑器、自动布线、视口持久化或完整结果报表。
- 不把 CAPE-OPEN / COM 语义倒灌到 Rust Core。
- 不引入第三方 CAPE-OPEN 模型加载。
- 不把 smoke test driver、PME 调试路径或单个宿主兼容逻辑提升为通用库 API。
- 不为未来可能需求预先堆叠不明意义的 helper / manager / orchestrator / context / adapter。

## 按需阅读

- 需要仓库全局模块边界：`docs/architecture/overview.md`
- 需要 MVP 范围和非目标：`docs/mvp/scope.md`
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
