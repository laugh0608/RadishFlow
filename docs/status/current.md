# 当前状态

更新时间：2026-05-16

## 用途

用途：为新会话恢复上下文、判断“今天做什么”提供轻量入口。
读者：人工开发者、用户、AI / Agent。
不包含：完整历史流水、详细设计推演、测试日志和长期说明书。

默认先读本文档。只有当任务需要具体实现细节、历史依据或专题边界时，再读取下方“按需阅读”列表。`AGENTS.md` / `CLAUDE.md` 只保留长期协作规则，不承载当前阶段流水。

## 当前阶段

- 产品定位：以 Rust Core + Rust UI + `.NET 10` CAPE-OPEN/COM 适配层构建稳态流程模拟软件。
- 当前主线：MVP 第一阶段最小闭环已经可验证，但尚未达到首版 demo 的产品可用水准；当前主线是 Studio 首页与工作台信息架构的可用性收口，发布 / tag 暂缓。
- 当前重点：Home Dashboard 与进入 case 后的 Workbench 第一轮真实 UI 已落地；下一步优先把 Canvas viewport 初始居中 / fit-to-content 和少量中英混合文案收口，降低人工测试和首版 demo 认知成本。
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
- 2026-05-16 已补 MVP α Windows 便携包入口：`scripts/package.ps1` 生成 staging / zip，附带 Studio exe、正向示例、样例物性包、关键文档、内部包记录和许可文件；Studio 打包后会优先从 exe 同目录的 `examples/flowsheets` 发现内置示例。该包仅作为内部验证产物，不代表首版 demo 或对外发布。
- 2026-05-16 已新增 `docs/releases/v26.5.1-dev.md`，记录内部便携包、验证结果和包内边界；当前暂缓创建 `v26.5.1-dev` tag，后续等首版 demo 功能和 UI 可用性达到标准后再重新评估版本节点。
- 2026-05-16 晚间已完成 Studio Home Dashboard 与 Workbench 第一轮 UI 收口：Home Dashboard 默认中文、三栏布局稳定、Recent / Example / Environment / Messages 分区清晰；Workbench 顶部主路径、左侧 Project、中央 Canvas header、右侧 Inspector / Results / Run / Package 和底部 drawer 已压缩信息噪声；关闭最后窗口前的一帧黑屏也已优化。

完整过程和每日验证记录见 `docs/devlogs/2026-05/2026-W20.md` 以及更早周志。

## 下一步建议

1. 明天优先推进 Canvas viewport 初始自动居中 / fit-to-content：打开示例或项目后，小流程应自然位于画布可视区域中央，不再固定偏左上；不扩自动布线、自由连线或视口持久化。
2. 继续复核 Home / Workbench 残余文案和按钮语义，例如首页示例卡片的 `打开 Case` 是否改成 `打开示例`、消息区 `Project opened` 等英文是否中文化。
3. 做一轮视觉 smoke，确认 Home Dashboard、进入示例、运行、结果 / 消息 / 物性包入口和关闭窗口路径没有回归；代码改动后继续跑 `cargo fmt --all` 与 `cargo test -p radishflow-studio studio_gui_shell`。
4. 便携包和 `docs/releases/v26.5.1-dev.md` 暂作为内部验证资产保留，不创建 tag，不推进对外发布自动化。
5. 结果面继续只读消费 `SolveSnapshot`，不新增 shell 私有结果缓存；Canvas 下一步只做 viewport 呈现收口，不扩大建模能力边界。

## 暂不推进

- 在 UI 信息架构未定稿前，不继续堆叠零散按钮、临时面板、调试状态或只为单次 smoke 服务的 presentation。
- 不把当前 UI 重排误扩成完整自由连线编辑器、完整拖拽布局编辑器、自动布线、视口持久化或完整结果报表。
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
