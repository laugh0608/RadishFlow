# 当前状态

更新时间：2026-05-06

## 用途

本文档是新会话恢复上下文、判断“今天做什么”的轻量入口。

默认先读本文档。只有当任务需要具体实现细节、历史依据或专题边界时，再读取下方列出的专题文档。

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
- `rf-thermo` / `rf-flash` 已收紧直接数值 API 的 mole fraction 输入契约，未归一组成会被拒绝；unit operation 层继续在调用 flash 前归一化文档流股组成。
- Stream Inspector 已补正式 composition normalize / draft discard command surface，并展示当前组成总和、归一化预览与只读草稿提示；写回、归一化或丢弃草稿都必须由用户显式触发。
- Stream Inspector 已补受控组分添加 / 删除入口：添加只能从当前 flowsheet 已定义但流股组成尚未包含的组件中显式选择；删除不能移除最后一个组成条目；新增或删除都不自动补偿其他组分。
- Studio 运行入口现在会在存在未提交/无效 Inspector 草稿或文档流股总体组成未归一时通过既有 run panel notice 阻塞运行，不做隐式自动补偿。
- 协作文档已新增 Docs 简约入口约束与代码规范专题文档：`docs/development/code-style.md`。

## 下一步建议

1. 优先回到数值与结果主线，而不是继续扩 Canvas UI。
2. 在当前 `SolveSnapshot` 边界内补齐结果审阅、错误定位或热力学结果展示的明显缺口。
3. 若推进 `rf-thermo` / `rf-flash`，优先补可验证的 MVP 数值能力和 golden tests，不提前引入完整 EOS、活度模型或复杂物性包选择器。
4. 若继续推进 Stream Inspector，优先收紧 flowsheet component catalog / presentation 边界；不要提前做完整组件库、项目级组件删除迁移或隐式差值补偿。
5. 若推进 Studio，优先消费已结构化 DTO 和既有 command surface，不新增第二套 shell 私有状态机。
6. 若发现入口文档继续膨胀，优先更新本文档和对应专题文档，不把长篇历史写回 `overview.md` 或 `scope.md`。

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
- 每次完成重要阶段收口后，优先更新本文档顶部状态和“下一步建议”。
