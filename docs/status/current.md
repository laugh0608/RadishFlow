# 当前状态

更新时间：2026-05-23

## 用途

用途：为新会话恢复上下文、判断“今天做什么”提供轻量入口。
读者：人工开发者、用户、AI / Agent。
不包含：完整历史流水、详细设计推演、测试日志和长期说明书。

默认先读本文档。只有当任务需要具体实现细节、历史依据或专题边界时，再读取下方“按需阅读”列表。`AGENTS.md` / `CLAUDE.md` 只保留长期协作规则，不承载当前阶段流水。

## 当前阶段

- 产品定位：以 Rust Core + Rust UI + `.NET 10` CAPE-OPEN/COM 适配层构建稳态流程模拟软件。
- 当前主线：MVP 第一阶段最小闭环已经可验证，`v26.5.1-dev` 已作为内部验收 tag 创建并推送；首版 demo 前的 Home / Workbench 产品可用性 blocker 已收口，当前重新回到功能开发。
- 当前重点：首版 demo 前硬化期已经结束，当前进入“受控扩展高频建模能力”阶段。已收口 `Heater / Cooler / Valve` 参数链路、连接失败恢复、空白项目 Mixer 路径、受控流股断开 / 删除 / source-only 重连、关闭脏工作区保护和 focused suggestion 下一步可见性。后续允许按正式边界推进下一批窄口径单元参数、受控重连细化、sidecar 级布局 / 视口体验和轻量结果审阅增强；仍不做自由连线编辑器、自动布线系统、完整拖拽布局编辑器、完整报表系统或第三方 CAPE-OPEN / 物性包加载。
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
- 2026-05-16 已新增 `docs/releases/v26.5.1-dev.md`，记录内部便携包、验证结果和包内边界；2026-05-18 已在最终仓库级验证与包内 smoke 通过后创建并推送 `v26.5.1-dev` 内部验收 tag。
- 2026-05-16 晚间已完成 Studio Home Dashboard 与 Workbench 第一轮 UI 收口：Home Dashboard 默认中文、三栏布局稳定、Recent / Example / Environment / Messages 分区清晰；Workbench 顶部主路径、左侧 Project、中央 Canvas header、右侧 Inspector / Results / Run / Package 和底部 drawer 已压缩信息噪声；关闭最后窗口前的一帧黑屏也已优化。
- 2026-05-17 已完成 Canvas viewport 初始自动居中：画布在打开项目后的首轮渲染根据当前单元 / 流股 bounds 计算 shell-local viewport transform，让打开示例或项目后的小流程自然位于可视区域中央；后续 layout nudge 复用同一 offset，不会被每帧重新居中抵消。点击放置会反算回原始 sidecar 坐标，不写入项目语义、不进入 CommandHistory，也不引入视口持久化。首页中文文案中的 `打开 Case` / `示例 Case` 等高频残留已改为 `打开项目` / `打开示例` / `示例项目`，Workbench 打开项目消息也已中文化。
- 2026-05-17 人工截图审阅后已修复 `Feed Heater Flash` 示例默认布局顺序：Canvas presentation 现在按物料流依赖给未定位单元排序，`feed-1 / heater-1 / flash-1` 会按工艺顺序从左到右显示；加载本地 sidecar 时也会过滤当前项目已不存在的 unit id。首页示例项目行按钮已从 `打开项目` 改为 `打开示例`。
- 2026-05-17 已完成人工 UI smoke 后的 Canvas 可读性第一轮打磨：流线增加名称标签和白色底衬，单元块略增高以缓解端口与文字拥挤，已绑定端口点击可直接聚焦流股 Inspector；运行命令成功后自动切到右侧结果和底部结果表，失败后切到右侧运行和底部消息。
- 2026-05-17 已按截图审阅建议收口 Home Dashboard 打开路径：左侧只保留 `新建项目`、`打开项目`、`打开示例项目`，最近项目和示例项目列表改为可选择、可双击打开；列表行内不再重复放置打开按钮。同步修正 Canvas 终端流股标签的垂直错位和内容 bounds，降低液相 / 气相出口标签重叠和右侧裁切概率。
- 2026-05-17 人工复测发现首页双击只命中项目标题文本、未覆盖整行卡片；现已改为整张最近项目 / 示例项目卡片响应点击和双击打开。Canvas 终端流股标签同步改为短名称并从端口右侧绘制，避免被 Flash Drum 单元块遮住。后续 smoke 又暴露 Home 在未保存空白项目后只能显示 discard 提示、没有继续 / 取消动作；现已让打开项目、打开示例项目和新建项目统一进入可确认流程。
- 2026-05-17 已按最新截图完成运行后结果视图收口：右侧结果检查器的流股 / 单元 / 对比 selector 不再为每个选项重复渲染 `Inspect`，中文界面中的 inspect 小按钮统一显示为 `检查`；底部结果表改为中文 `流股 / 相态` 表头，并以短相态摘要替代过长 `phases: ...` 原始文本，完整相态仍保留在 tooltip。已补回归覆盖打开示例、运行、查看结果、回 Home、再从最近项目打开并重新运行的路径。
- 2026-05-17 已根据最新截图继续清理 Workbench 高频残余：Canvas 面板头隐藏开发态计数摘要，运行结果摘要改为中文结构化计数，左侧 Project 对象行去掉重复 `检查` 按钮，底部结果表空相态显示为短值 `无`，非空画布不再常驻“选择画布工具”提示，放置按钮中的 MVP 单元名已中文化。
- 2026-05-18 已收口日终保留的两个 UI blocker：Canvas 中间连接流股在短线段空间不足时不再绘制名称标签，避免 `Feed -> Valve -> Flash Drum` 等紧凑链路遮挡单元块或端口；右侧 Result Inspector 的默认组成 / 相态摘要改为结构化短行，模型层不再生成 `z:` / `phases:` 前缀。`pwsh ./scripts/check-repo.ps1` 已在真实环境通过。
- 2026-05-18 人工从 IDE 启动 `Feed Valve Flash Binary Hydrocarbon Example` 复核通过，未发现新的 UI blocker；同日已刷新 `v26.5.1-dev` 内部便携包 staging / zip，并从包目录启动执行 smoke，未发现包内示例发现、打开示例、运行、结果审阅等主路径问题。最终包 manifest 记录 `gitCommit=7479e82`、`gitDirty=false`，`v26.5.1-dev` tag 已推送到远端。
- 2026-05-18 首版 demo 前产品可用性评审发现并收口 DocsRepro / Home 示例入口 blocker：默认 Home / Workbench 示例选择器只保留四条 official hydrocarbon 演示路径，避免 synthetic / PME 验证样例以“就绪示例”进入高频入口；`Feed Heater Flash` 首页标题改回单一 `加热器` 语义；quick start、run-first、versioning、release notes 与 package README 模板已同步当前 Windows 内部便携包和已推送 tag 口径。
- 2026-05-18 在 demo blocker 收口后转回功能开发：`UnitNode` 新增可选 SI 参数结构，Unit Inspector 暴露 Heater/Cooler outlet temperature 与 Valve outlet pressure 的字段级草稿编辑；提交走 `DocumentCommand::SetUnitParameter`，同步 outlet stream 模板并触发求解 dirty 状态；顺序求解器优先使用已提交单元参数，旧项目无参数时保持原有行为。
- 2026-05-20 已复核首批单元参数闭环：`rf-store` 现在锁定单元参数随项目 JSON round-trip，Studio shell focused 回归覆盖官方 Heater / Cooler / Valve 示例和空白项目 Heater 最短路径从 Unit Inspector 编辑参数、保存、重开到再次运行收敛，并断言求解结果使用保存后的 outlet temperature / outlet pressure；Unit Inspector 字段 presentation 现在带 SI 约束提示，Valve outlet pressure 草稿在高于已连接 inlet pressure 时直接标为 invalid；若旧项目或外部编辑仍带入已提交越界参数，求解失败会定位到 `solver.step.parameter`、`valve-1:outlet` / `valve-1:inlet` 和相关流股。
- 2026-05-20 已补连接类失败恢复路径 focused 覆盖：`missing_upstream_source`、`missing_stream_reference`、`duplicate_upstream_source`、`duplicate_downstream_sink`、`unbound_outlet_port`、`orphan_stream`、`invalid_port_signature`、two-unit cycle 与 self-loop cycle 的失败 detail / Run Panel recovery / Canvas 或端口 attention 回归已覆盖；Canvas attention 不再只从 Run Panel notice 反推单一 recovery target，而是优先使用当前文档 revision 的 solver failure diagnostic，从而保留完整 unit / stream / port targets。
- 2026-05-20 已补空白项目 Mixer 最短建模路径 focused 覆盖：通过 Canvas suggestion 创建 `Feed + Feed -> Mixer -> Flash Drum`，保存后断言 unit / stream / port 绑定，重开后确认 mixer outlet 总摩尔流量为两股入口之和。
- 2026-05-22 已补 Canvas / Inspector 受控流股恢复入口：选中物料流股后可执行 `Disconnect stream` 解除所有物料端口绑定并保留流股规格，或执行 `Delete stream` 解除绑定后删除流股；两者均通过正式 `DocumentCommand` 与 undo history，不做自由连线、自动布线或完整拖拽布局。
- 2026-05-23 人工复核确认流股连接 / 断开交互已经顺滑；同日补关闭脏工作区的保存 / 舍弃 / 取消确认，并让 focused Canvas suggestion 在顶部摘要和图例中直接显示 `Connect stream` / `Create stream` 下一步动作，不改变 suggestion / `DocumentCommand` 边界。随后补选中 source-only 流股的受控重连入口：仅当当前流股已有唯一 source、没有 sink，且画布上只有一个可用 material inlet 时启用，写回仍走正式 `ConnectPorts` 与 undo history。

完整过程和每日验证记录见 `docs/devlogs/2026-05/2026-W21.md`、`docs/devlogs/2026-05/2026-W20.md` 以及更早周志。

## 下一步建议

1. 下一步优先做“单个受控能力”的完整闭环：下一批窄口径单元参数、selected stream 受控重连细化、单元直接拖动位置但只写 layout sidecar、sidecar 级 viewport 记忆，或当前 snapshot 的轻量结果复制 / 导出。
2. 每个新能力必须走正式 command / validation / undo，或明确标记为 shell-local state；同时补 focused tests 和必要文档。
3. 若要刷新新的便携包或 tag，应创建新提交 / 新版本节点，不移动已推送的 `v26.5.1-dev` tag。
4. 不把受控扩展误扩成自由连线编辑器、自动布线系统、完整拖拽布局编辑器、完整报表系统或对外发布自动化。

## 暂不推进

- 不继续堆叠零散按钮、临时面板、调试状态或只为单次 smoke 服务的 presentation；UI 改进应按明确专题推进。
- 不做自由连线编辑器、自动布线系统、完整拖拽布局编辑器、完整报表系统；允许受控重连、sidecar 级单元拖动、sidecar 级 viewport 记忆和轻量结果审阅增强。
- 不把 CAPE-OPEN / COM 语义倒灌到 Rust Core。
- 不引入第三方 CAPE-OPEN 模型加载。
- 不把 smoke test driver、PME 调试路径或单个宿主兼容逻辑提升为通用库 API。
- 不为未来可能需求预先堆叠不明意义的 helper / manager / orchestrator / context / adapter。
- 不再主动扩 near-boundary / command surface / runtime click 的细枝末节测试；除非它们直接暴露 MVP α 验收 blocker。

## 按需阅读

- 需要仓库全局模块边界：`docs/architecture/overview.md`
- 需要 MVP 范围和非目标：`docs/mvp/scope.md`
- 需要 MVP α 验收矩阵：`docs/mvp/alpha-acceptance-checklist.md`
- 需要最新流水和决策依据：`docs/devlogs/2026-05/2026-W21.md`
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
