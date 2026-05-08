# 当前状态

更新时间：2026-05-08

## 用途

本文档是新会话恢复上下文、判断“今天做什么”的轻量入口，也是当前阶段、当前重点、当前验证基线和暂不推进项的默认真相源。

默认先读本文档。只有当任务需要具体实现细节、历史依据或专题边界时，再读取下方列出的专题文档。`AGENTS.md` / `CLAUDE.md` 只保留长期协作规则，不再重复承载这类易过期内容。

## 当前阶段

- 产品定位：以 Rust Core + Rust UI + `.NET 10` CAPE-OPEN/COM 适配层构建稳态流程模拟软件。
- 当前主线：在已具备最短可运行建模路径和 Stream Inspector 组成编辑安全边界后，转回数值主线和结果审阅收口。
- 当前重点：保持 Canvas 与 Stream Inspector 不继续横向扩张，把精力放到 `SolveSnapshot` 结果消费、热力学 / 闪蒸基础能力和可验证闭环上。
- 当前验证基线：功能改动优先执行相关 focused tests；阶段性收口执行 `pwsh ./scripts/check-repo.ps1`。

## 最近已完成

- Studio Canvas 已支持最短可求解建模路径：`Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum`。
- 空白项目打开时已补 MVP 默认二元组件与本地 `binary-hydrocarbon-lite-v1` 物性包缓存，可保存后重新打开继续运行最短闭环。
- Canvas placement 坐标已保存到 `<project>.rfstudio-layout.json` sidecar；离散 nudge 只更新 sidecar，不改项目文档、不进入 undo/redo。
- Result Inspector 已补 stream-centric 与 unit-centric 两个消费面，可审阅当前快照内输入/输出流股、相结果、关联步骤和关联诊断。
- `rf-thermo` / `rf-flash` 已补 MVP 常热容显热焓值；`Flash Drum` outlet 会传递 liquid / vapor / overall molar enthalpy。
- `rf-thermo` 已补基于当前 Antoine / Raoult MVP 假设的 bubble/dew pressure 边界估算，以及 fixed-pressure bubble/dew temperature 边界估算；`rf-flash` 的 `TP Flash` 结果现在会显式物化 `liquid-only / two-phase / vapor-only` phase region 与对应 bubble/dew pressure / temperature 窗口，并已补齐 golden / focused tests。
- `tests/thermo-golden` 与 `tests/flash-golden` 现在都已覆盖 `liquid-only / two-phase / vapor-only` 三类正式金样；`rf-flash` 与 `rf-types` focused tests 也已锁定 exact bubble/dew boundary 和 tolerance 内外的 phase region 语义，避免边界漂移被集成层才发现。
- `tests/thermo-golden` 与 `tests/flash-golden` 已继续补齐 near-boundary `±ΔP / ±ΔT` 小扰动金样；当前不仅覆盖 `binary-hydrocarbon-lite-v1` 原有 `z=[0.2, 0.8]` two-phase 基线，还补到靠 bubble / dew 两侧的 `z=[0.195, 0.805]` 与 `z=[0.23, 0.77]` 两组 two-phase 组成，并继续覆盖现有 synthetic `liquid-only / vapor-only` 样例；`rf-thermo` 与 `rf-flash` focused tests 也已锁定 bubble/dew 两侧跨 boundary 前后的 phase region 与 `bubble_dew_window` 稳定行为，避免边界附近漂移只在集成层或 UI 审阅时才暴露。
- `rf-model::MaterialStreamState`、`rf-solver` 与 `rf-ui::SolveSnapshot` 现在已为 flash 产物流股正式透传结构化 `bubble_dew_window`；`Flash Drum` liquid / vapor outlet 会按各自 outlet 组成重算并携带这组窗口，而不是复用 overall flash feed 的边界。
- `Mixer`、`Heater/Cooler` 与 `Valve` 的 outlet 结果现在也会在 unit operation 层直接物化同一组结构化 `bubble_dew_window`，并通过 `rf-solver -> rf-ui::SolveSnapshot -> Result Inspector / Active Inspector` 只读透传；这层继续只消费正式 thermo DTO，不在 shell / UI 中重算或分叉第二套相平衡语义。
- `tests/rust-integration` 与 workspace run path 现在已把 `binary-hydrocarbon-lite-v1` 三组 two-phase 组成 `z=[0.195, 0.805] / [0.2, 0.8] / [0.23, 0.77]` 的 near-boundary `±ΔP / ±ΔT` case 前推到 `feed-heater-flash-binary-hydrocarbon` 正式链路；同一套回归也已补到 synthetic `liquid-only / vapor-only` 单相样例的 `feed-heater-flash` / `feed-cooler-flash` / `feed-valve-flash` 链路。`studio_solver_bridge` 与 `studio_workspace_control` 都会锁定非 flash 中间流股 `phase_region` / `bubble_dew_window` 与后续 flash inlet consumed stream 的同一份 DTO，不在集成层分叉第二套窗口判断。
- Result Inspector / Active Inspector 现在会只读消费 `SolveSnapshot` 已物化的 `bubble_dew_window`，显式展示 `phase_region` 与 bubble/dew pressure / temperature；这层继续只消费 DTO，不在 shell 中重算热力学或分叉第二套相平衡语义。
- Studio bootstrap 内置的 `binary-hydrocarbon-lite-v1` 样例包 Antoine 系数现在也已与当前 bubble/dew temperature 数值基线对齐，空白项目和 shell/solver 回归继续共享同一套相平衡假设。
- Result Inspector / Active Inspector 的流股相结果与相对比现在会显式展示各相摩尔流量，并继续只消费 `SolveSnapshot` 已物化的 phase fraction / molar enthalpy，不在 shell 中重算热力学。
- Solve step / Active Inspector / unit-centric Result Inspector 现在会为输入和输出流股显式展示 `T / P / F / H` 结果摘要，便于直接审阅单元前后变化；这层仍只消费已有 DTO 与既有 `InspectorTarget` command。
- Diagnostics 列表与 failure diagnostic 现在会前推相关流股数值上下文：成功路径直接显示 `SolveSnapshot` 已物化的 `T / P / F / H` 摘要，失败路径在诊断 revision 与当前文档匹配时显示文档态 `T / P / F / z` 与 port 绑定流股上下文；这层仍只消费结构化 snapshot，不在 shell 中反查文档或反解析错误消息。
- `rf-thermo` / `rf-flash` 已收紧直接数值 API 的 mole fraction 输入契约，未归一组成会被拒绝；unit operation 层继续在调用 flash 前归一化文档流股组成。
- Stream Inspector 已补正式 composition normalize / draft discard command surface，并展示当前组成总和、归一化预览与只读草稿提示；写回、归一化或丢弃草稿都必须由用户显式触发。
- Stream Inspector 已补受控组分添加 / 删除入口：添加只能从当前 flowsheet 已定义但流股组成尚未包含的组件中显式选择；删除不能移除最后一个组成条目；新增或删除都不自动补偿其他组分。
- Studio 运行入口现在会在存在未提交/无效 Inspector 草稿或文档流股总体组成未归一时通过既有 run panel notice 阻塞运行，不做隐式自动补偿。
- 仓库基础治理已继续收口：根 `README.md` 改回稳定入口，`.gitattributes` / `.gitignore` 补齐基础规则，`*.idl` 已纳入 UTF-8 / LF 文本门禁。
- 协作文档已新增 Docs 简约入口约束与代码规范专题文档：`docs/development/code-style.md`。

## 下一步建议

1. 若继续推进 `rf-thermo` / `rf-flash`，优先把当前 two-phase 与 synthetic 单相样例都已前推到 `tests/rust-integration` / workspace run path 的 near-boundary `±ΔP / ±ΔT` 基线维持为正式回归；若要继续扩展，优先沿同一模式补到 `Mixer`，并视需要为 `Cooler` / `Valve` 再补 dedicated binary-hydrocarbon two-phase 链路，而不是分叉第二套判断语义。
2. 若继续推进 `rf-thermo` / `rf-flash`，保持现有 golden 目录多样例遍历、focused tests 与非 flash 中间流股到 flash inlet 的端到端一致性回归为同一套正式基线，不额外分叉第二套窗口估算或判断语义。
3. 若继续推进 Stream Inspector，优先收紧 flowsheet component catalog / presentation 边界；不要提前做完整组件库、项目级组件删除迁移或隐式差值补偿。
4. 若推进 Studio，优先继续消费已结构化 DTO 和既有 command surface，不新增第二套 shell 私有状态机。
5. 若发现入口文档继续膨胀，优先更新本文档和对应专题文档，不把长篇历史写回 `overview.md` 或 `scope.md`。

## 暂不推进

- 不扩自由连线编辑器、拖拽布局编辑器、自动布线、视口持久化或完整结果报表。
- 不把 CAPE-OPEN / COM 语义倒灌到 Rust Core。
- 不引入第三方 CAPE-OPEN 模型加载。
- 不把 smoke test driver、PME 调试路径或单个宿主兼容逻辑提升为通用库 API。
- 不为未来可能需求预先堆叠不明意义的 helper / manager / orchestrator / context / adapter。

## 按需阅读

- 需要仓库全局模块边界：`docs/architecture/overview.md`
- 需要 MVP 范围和非目标：`docs/mvp/scope.md`
- 需要最新流水和决策依据：`docs/devlogs/2026-W19.md`
- 需要热力学 / 闪蒸细节：`docs/thermo/mvp-model.md`
- 需要 CAPE-OPEN / COM 边界：`docs/capeopen/boundary.md`
- 需要桌面 App / Canvas 交互契约：`docs/architecture/app-architecture.md`、`docs/architecture/canvas-interaction-contract.md`
- 需要代码风格、命名或抽象判断：`docs/development/code-style.md`

## 更新规则

- 本文档只保留当前阶段、最近状态、下一步建议和按需阅读入口。
- 历史流水写入周志；长期边界写入专题文档；不要把本文档写成长篇进度报告。
- 协作入口文件只保留长期稳定规则；阶段性变化优先更新本文档和对应专题文档，再按需同步入口文件中的引用关系。
- 每次完成重要阶段收口后，优先更新本文档顶部状态和“下一步建议”。
