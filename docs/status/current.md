# 当前状态

更新时间：2026-05-16

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

- Studio 已具备 MVP α 最小可操作闭环：打开示例、新建空白、最短建模、运行、审阅结果、保存 / 另存为和重开项目。
- Canvas 当前覆盖 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum` 三条最短可求解路径；连接仍通过本地 suggestion 和正式 `DocumentCommand` 完成，不是自由连线编辑器。
- Result Inspector / Active Inspector 只读消费同一份 `SolveSnapshot`，可审阅 stream-centric / unit-centric 结果、相结果、`bubble_dew_window`、overall enthalpy、关联步骤和诊断目标。
- `rf-thermo` / `rf-flash` / `rf-solver` / `rf-ffi` 已围绕 official / synthetic near-boundary、phase region、enthalpy、JSON/error 与结构化 stream snapshot 形成当前回归基线。
- Stream Inspector 已收口 composition draft、显式提交、normalize、discard、受控组件添加 / 删除和运行前阻断；不做隐式差值补偿。
- 仓库基础治理已补齐根 `README.md`、文本格式门禁、代码规范、文档体量治理、路线图拆分和周志月份归档；默认入口文档继续保持摘要化。
- 2026-05-13 至 2026-05-14 的人工 Studio smoke blocker 已收口：首屏主路径、运行门控、GUI panic 降级、Windows debug 主线程栈、最后窗口关闭、顶部快速操作、工作台重排和 Inspector 可发现性均已处理。
- 2026-05-16 MVP α Studio 用户视角 Smoke A / B / C 已人工通过；同日中文界面资源已覆盖 smoke 高频路径，结构化 JSON 测试夹具也已避免 IDE 保存字段顺序导致的回归噪声。
- 2026-05-16 已补 MVP α Windows 便携包入口：`scripts/package.ps1` 生成 staging / zip，附带 Studio exe、正向示例、样例物性包、关键文档、候选 release notes 和许可文件；Studio 打包后会优先从 exe 同目录的 `examples/flowsheets` 发现内置示例。`pwsh ./scripts/package.ps1 -Version v26.5.1-dev -Clean` 已通过。
- 2026-05-16 已新增 `docs/releases/v26.5.1-dev.md`，记录 MVP α 候选 tag、验证结果、包内边界和创建 tag 前检查项；打包 manifest 会在对应文件存在时记录 `releaseNotes=docs/releases/v26.5.1-dev.md`。

完整过程和每日验证记录见 `docs/devlogs/2026-05/2026-W20.md` 以及更早周志。

## 下一步建议

1. 从 `artifacts/packages/RadishFlow-v26.5.1-dev-windows-x64/` 直接启动包内 `radishflow-studio.exe` 做最后人工 smoke，重点确认包内示例发现、打开示例、运行、审阅和保存 / 重开。
2. UI 规范化仍只服务 MVP α 验收，不扩自由连线编辑器、完整拖拽布局编辑器、完整报表系统或新的求解范围；结果面继续只读消费 `SolveSnapshot`，不新增 shell 私有结果缓存。
3. 包内 smoke 无 blocker 后，执行创建 `v26.5.1-dev` tag 前最终检查；若要继续推进发布自动化，优先评估是否把 `scripts/package.ps1` 接到 tag 触发的 CI 工件归档，不要在当前便携包脚本中混入安装、COM 注册或 PME 自动化。
4. 只修人工 smoke、打包 dry run 或仓库级验证暴露的真实 blocker；若只是收益递减的 focused 覆盖缺口，先记录而不是继续主动扩矩阵。
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
- 需要最新流水和决策依据：`docs/devlogs/2026-05/2026-W20.md`
- 需要热力学 / 闪蒸细节：`docs/thermo/mvp-model.md`
- 需要 CAPE-OPEN / COM 边界：`docs/capeopen/boundary.md`
- 需要桌面 App / Canvas 交互契约和 Studio UI 规范：`docs/architecture/app-architecture.md`、`docs/architecture/canvas-interaction-contract.md`、`docs/architecture/studio-ui-design-guidelines.md`
- 需要代码风格、命名或抽象判断：`docs/development/code-style.md`
- 需要文档篇幅和拆分规则：`docs/README.md`

## 更新规则

- 本文档目标上限为 8k 字符；超过上限时应优先删减历史流水、重复背景和过细实现细节。
- 本文档只保留当前阶段、最近完成摘要、下一步建议、暂不推进项和按需阅读入口。
- 历史流水写入周志；长期边界写入专题文档；不要把本文档写成长篇进度报告。
- 协作入口文件只保留长期稳定规则；阶段性变化优先更新本文档和对应专题文档，再按需同步入口文件中的引用关系。
- 每次完成重要阶段收口后，优先更新本文档顶部状态和“下一步建议”。
