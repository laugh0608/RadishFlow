# 当前状态

更新时间：2026-05-25

## 用途

用途：为新会话恢复上下文、判断“今天做什么”提供入口。
读者：开发者、用户、AI / Agent。
不包含：完整历史流水、详细设计推演、测试日志和长期说明书。

默认先读本文档。只有当任务需要具体实现细节、历史依据或专题边界时，再读取下方“按需阅读”列表。`AGENTS.md` / `CLAUDE.md` 只保留长期协作规则，不承载当前阶段流水。

## 当前阶段

- 产品定位：以 Rust Core + Rust UI + `.NET 10` CAPE-OPEN/COM 适配层构建稳态流程模拟软件。
- 当前主线：MVP 第一阶段最小闭环和首版 demo 前硬化期已经阶段性收口；当前进入 MVP β：高频建模能力与小案例作者体验。当前尚未达到正式 tag / release 节点标准，历史 `v26.5.1-dev` 打包口径不再作为当前事实源或发布计划依据。
- 当前重点：已收口内建单元参数链路、连接失败恢复、空白项目 Mixer 路径、受控流股断开 / 删除 / 单端唯一候选重连、source / sink 断开提示、重连不可用原因一致展示、关闭脏工作区保护、suggestion 下一步可见性、sidecar 级单元定位 / 拖动 / viewport 记忆、`SolveSnapshot` 轻量文本复制 / 导出。2026-05-25 人工真实窗口轻量 smoke 已通过，MVP β 第一刀已转入可复现小案例作者体验：Home 可从空白项目进入 `Feed + Feed -> Mixer -> Flash Drum` 与 `Feed -> Heater -> Flash Drum` 作者路径，Workbench 放置面板按当前 canvas 状态显示任务清单。下一步优先补同类路径的结果审阅收口或成组高频单元参数；不再继续围绕 hover、提示、按钮等 presentation 细节做开放式补口。仍不做自由连线、自动布线、完整拖拽布局、完整报表、完整参数表或第三方物性包加载。
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
- 2026-05-16 至 2026-05-18 曾补 MVP α Windows 内部便携包 staging 入口和相关说明材料；这些材料只按历史打包草案保留，不代表当前已经形成正式 tag、首版 demo 或对外发布节点。
- 2026-05-16 晚间已完成 Studio Home Dashboard 与 Workbench 第一轮 UI 收口：Home 默认中文、三栏布局稳定，Workbench 信息噪声和最后窗口关闭黑屏已优化。
- 2026-05-17 已收口 Home / Workbench / Canvas 高频 smoke 残余：Canvas 初始居中、默认工艺顺序、流线与终端标签可读性、整行示例卡片双击、未保存确认、运行后结果视图和 Workbench 中文高频文案均已处理。
- 2026-05-18 已收口日终保留的两个 UI blocker：Canvas 中间连接流股在短线段空间不足时不再绘制名称标签，避免 `Feed -> Valve -> Flash Drum` 等紧凑链路遮挡单元块或端口；右侧 Result Inspector 的默认组成 / 相态摘要改为结构化短行，模型层不再生成 `z:` / `phases:` 前缀。`pwsh ./scripts/check-repo.ps1` 已在真实环境通过。
- 2026-05-18 人工从 IDE 启动 `Feed Valve Flash Binary Hydrocarbon Example` 复核通过，未发现新的 UI blocker；同日也验证过 Windows 便携 staging 目录的示例发现、打开示例、运行和结果审阅主路径。该 staging 只作为历史内部验证材料，不作为当前正式版本节点。
- 2026-05-18 首版 demo 前产品可用性评审发现并收口 DocsRepro / Home 示例入口 blocker：默认 Home / Workbench 示例选择器只保留四条 official hydrocarbon 演示路径，避免 synthetic / PME 验证样例以“就绪示例”进入高频入口；`Feed Heater Flash` 首页标题改回单一 `加热器` 语义；quick start、run-first、versioning 与 package README 模板已同步 Windows 内部 staging 边界。
- 2026-05-18 在 demo blocker 收口后转回功能开发：`UnitNode` 新增可选 SI 参数结构，Unit Inspector 暴露 Heater/Cooler outlet temperature 与 Valve outlet pressure 的字段级草稿编辑；提交走 `DocumentCommand::SetUnitParameter`，同步 outlet stream 模板并触发求解 dirty 状态；顺序求解器优先使用已提交单元参数，旧项目无参数时保持原有行为。
- 2026-05-20 至 2026-05-23 已复核单元参数闭环：`rf-store` 锁定单元参数随项目 JSON round-trip，Studio shell focused 回归覆盖官方 Heater / Cooler / Valve / Flash Drum 示例和空白项目 Heater 最短路径从 Unit Inspector 编辑参数、保存、重开到再次运行收敛，并断言求解结果使用保存后的 outlet temperature / outlet pressure / flash pressure；Unit Inspector 字段带 SI 约束提示，Heater / Cooler / Valve outlet pressure 草稿高于已连接 inlet pressure 时标为 invalid；Flash Drum flash pressure 提交后同步 liquid / vapor 两个出口模板。
- 2026-05-20 已补连接类失败恢复路径 focused 覆盖：`missing_upstream_source`、`missing_stream_reference`、`duplicate_upstream_source`、`duplicate_downstream_sink`、`unbound_outlet_port`、`orphan_stream`、`invalid_port_signature`、two-unit cycle 与 self-loop cycle 的失败 detail / Run Panel recovery / Canvas 或端口 attention 回归已覆盖；Canvas attention 不再只从 Run Panel notice 反推单一 recovery target，而是优先使用当前文档 revision 的 solver failure diagnostic，从而保留完整 unit / stream / port targets。
- 2026-05-20 已补空白项目 Mixer 最短建模路径 focused 覆盖：通过 Canvas suggestion 创建 `Feed + Feed -> Mixer -> Flash Drum`，保存后断言 unit / stream / port 绑定，重开后确认 mixer outlet 总摩尔流量为两股入口之和。
- 2026-05-22 已补 Canvas / Inspector 受控流股恢复入口：选中物料流股后可执行 `Disconnect stream` 解除所有物料端口绑定并保留流股规格，或执行 `Delete stream` 解除绑定后删除流股；两者均通过正式 `DocumentCommand` 与 undo history，不做自由连线、自动布线或完整拖拽布局。
- 2026-05-23 人工复核确认流股连接 / 断开交互已经顺滑；同日补关闭脏工作区确认、focused suggestion 下一步显示、单端流股唯一候选重连、cycle-forming 候选过滤、已连接流股 source / sink 端点级断开与不可用原因提示。随后补选中单元在 Canvas 空白处点击定位、直接拖动到 sidecar 坐标，并补空白画布拖拽的 viewport offset 记忆；这些布局 / 视口状态只写 `<project>.rfstudio-layout.json`，不写项目语义、不进 undo、不扩完整拖拽布局编辑器或完整视图持久化系统。
- 2026-05-23 已补当前结果快照轻量复制 / 导出：右侧 `结果` 区可把当前 `SolveSnapshot` 复制到剪贴板或导出 `.txt`；内容只来自结果 DTO，覆盖流股摘要、步骤和诊断，不写项目、不进 undo、不扩报表、模板或批量导出。
- 2026-05-24 已收口 selected stream 重连 presentation 一致性：Canvas / Inspector / shell 对可用、已双端连接、候选不唯一和 cycle-forming 候选等状态使用同一套可用性判断与不可用原因；Inspector 不再隐藏不可用重连动作，而是禁用并说明原因。该补口仍不新增端口选择器、自由连线或自动布线。
- 2026-05-24 轻量人工 smoke 暴露 Canvas 单元直接拖动时模块原地抖动且不能稳定按住拖拽，以及空白点击错误地移动选中单元、关闭脏工作区确认只显示在顶部通知区。现已把单元拖动改为按鼠标 world 坐标跟随并在拖动 active 时禁用 viewport pan；空白点击改为清空选择；关闭脏工作区改为居中确认窗口。拖动仍只写 layout sidecar，不写项目语义、不进 undo。
- 2026-05-24 已补 `SolveSnapshot` 文本复制 / 导出 `Units` 区；仍只消费结果 DTO，不写项目、不进 undo、不扩完整报表系统。
- 2026-05-24 已补 Flash Drum flash temperature 与 Feed source T/P：复用正式参数链路并同步 outlet 模板；不扩完整参数表。
- 2026-05-25 人工真实窗口轻量 smoke 已通过：结果复制 / 导出、Feed / Mixer / Flash Drum 参数重跑、selected stream 重连 / 断开、关闭确认和 sidecar 单元拖动 / viewport 均未发现新的主路径 blocker。
- 2026-05-25 已纠偏版本节点口径：当前没有达到正式 tag / release 标准，`v26.5.1-dev` 相关 release notes / staging 说明只作为历史草案或内部打包材料保留，不再作为当前主线事实源。
- 2026-05-25 MVP β 第一刀已开始落地：空白项目 `Feed + Feed -> Mixer -> Flash Drum` focused 回归扩展为小案例作者路径，覆盖单元参数提交、运行、保存、重开、重跑和当前结果文本导出；`run-first` guide 已补同一路径说明。该能力仍复用正式 command / suggestion / save / export 边界。
- 2026-05-25 已补 Studio Home 小案例作者入口和 Workbench 放置面板任务清单：入口只创建空白项目并切到 `放置`，清单只从当前 canvas 单元 / 流股 / solve snapshot 推导状态，不自动生成 flowsheet，不写项目文档，不进 undo。
- 2026-05-25 已把同一套作者入口扩到 `Feed -> Heater -> Flash Drum`：Home 可选择 Heater-Flash 小案例，Palette 只显示该路径清单；状态仍只读 canvas view，不自动生成 flowsheet。

见 `docs/devlogs/2026-05/2026-W22.md`、`docs/devlogs/2026-05/2026-W21.md`。

## 下一步建议

1. 继续推进可复现小案例作者体验：优先细化当前任务清单与结果审阅入口，或把同一结构继续扩到 `Feed -> Valve -> Flash Drum`。
2. 备选能力包是成组 Unit Inspector 参数增强或受控连接编辑设计；继续保持正式 command / validation / undo，或明确 shell-local sidecar state，并补 focused tests 和必要文档。
3. 暂不推进 tag、release notes、便携包刷新或对外发布自动化；若未来要恢复版本节点，必须先明确验收标准和人工确认。
4. 不把 MVP β 误扩成自由连线、自动布线、完整拖拽布局、完整报表、完整参数表、第三方模型加载或对外发布自动化。

## 暂不推进

- 不继续堆叠零散按钮、临时面板、调试状态、hover 说明或只为单次 smoke 服务的 presentation；UI 改进应按明确专题推进。
- 不做自由连线编辑器、自动布线系统、完整拖拽布局编辑器、完整报表系统；允许受控连接编辑设计、sidecar 级单元拖动 / viewport 记忆和轻量结果审阅增强。
- 不把 CAPE-OPEN / COM 语义倒灌到 Rust Core。
- 不引入第三方 CAPE-OPEN 模型加载。
- 不把 smoke test driver、PME 调试路径或单个宿主兼容逻辑提升为通用库 API。
- 不为未来可能需求预先堆叠不明意义的 helper / manager / orchestrator / context / adapter。
- 不再主动扩 near-boundary / command surface / runtime click 的细枝末节测试；除非它们直接暴露 MVP α 验收 blocker。

## 按需阅读

- 需要仓库全局模块边界：`docs/architecture/overview.md`
- 需要 MVP 范围和非目标：`docs/mvp/scope.md`
- 需要 MVP α 验收矩阵：`docs/mvp/alpha-acceptance-checklist.md`
- 需要最新流水和决策依据：`docs/devlogs/2026-05/2026-W22.md`
- 需要热力学 / 闪蒸细节：`docs/thermo/mvp-model.md`
- 需要 CAPE-OPEN / COM 边界：`docs/capeopen/boundary.md`
- App/Canvas/UI：`docs/architecture/app-architecture.md`、`docs/architecture/canvas-interaction-contract.md`、`docs/architecture/studio-ui-design-guidelines.md`、`docs/architecture/studio-visual-system.md`
- 需要代码风格、命名或抽象判断：`docs/development/code-style.md`
- 需要文档篇幅和拆分规则：`docs/README.md`

## 更新规则

- 本文档目标上限为 8k 字符；超过上限时应优先删减历史流水、重复背景和过细实现细节。
- 本文档只保留当前阶段、最近完成摘要、下一步建议、暂不推进项和按需阅读入口。
- 历史流水写入周志；长期边界写入专题文档；不要把本文档写成长篇进度报告。
- 协作入口文件只保留长期稳定规则；阶段性变化优先更新本文档和对应专题文档，再按需同步入口文件中的引用关系。
- 每次完成重要阶段收口后，优先更新本文档顶部状态和“下一步建议”。
