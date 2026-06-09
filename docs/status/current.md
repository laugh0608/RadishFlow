# 当前状态

更新时间：2026-06-09

## 用途

用途：为新会话恢复上下文、判断“今天做什么”提供入口。  
读者：开发者、用户、AI / Agent。  
不包含：完整历史流水、详细设计推演、测试日志和长期说明书。

默认先读本文档。只有当任务需要具体实现细节、历史依据或专题边界时，再读取下方“按需阅读”列表。`AGENTS.md` / `CLAUDE.md` 只保留长期协作规则，不承载当前阶段流水。

## 阶段结论

- 产品定位：以 Rust Core + Rust UI + `.NET 10` CAPE-OPEN / COM 适配层构建稳态流程模拟软件。
- MVP 第一阶段 M1-M5、MVP α、MVP β 人工 smoke、失败修复闭环和通用小流程建模 v1 均已阶段性收口。
- 阶段基线：2026-05-28 真实环境 `pwsh ./scripts/check-repo.ps1` 通过；2026-06-02 真实环境 `./scripts/check-repo.sh` 通过。
- 通用小流程建模 v1 已支持普通空白项目在受控范围内组合 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum`，并覆盖显式输入、运行、保存、重开、rerun 与结果审阅。
- 当前主线已切到 **Studio UI 专题窄口径实现**。`studio-client-main.pen` 是当前唯一活跃 Studio 主设计稿，覆盖 Home、独立物性页、流程图工作台、模块设置 / 模块结果和底部运行 / 状态分栏。
- 最新进度：**Studio UI 实现第三十六刀已完成运行页顶部收束**。脏项目 `Cmd+Q` 真实窗口 smoke 已复核通过；底部结果表的 stream / unit 定位会派发正式 `inspector.focus_*` command；顶部 `结果工具栏` 不再展开所有结果对象 focus command，顶部 `运行工具栏` 的 Monitor 入口不再重复渲染按钮旁状态 chip，状态统一交给下一行扫读。
- 当前尚未进入正式 tag / release 节点；历史 `v26.5.1-dev` 只作为内部 staging 草案和验证记录保留。

## 当前策略

已经通过的阶段只修真实 blocker：

- 无法完成主路径建模
- 无法运行或运行结果明显错误
- 保存 / 重开破坏项目
- 文档事实源与代码能力明显冲突
- 仓库级验证或核心 focused test 失败

当前不再主动追逐 hover、提示、按钮文案、局部 selector、presentation 小瑕疵或更多同构作者入口。`Mixer-Flash` / `Heater-Flash` 作者路径、MVP β Smoke A-D 和结果审阅对象覆盖保留为代表性回归，不作为日常 gate。

## 能力基线

通用建模与求解：

- 普通空白项目不再进入或自动匹配小案例状态；运行前检查按当前 `Flowsheet` 的真实建模输入判断。
- 缺项目组分、缺 Feed composition、Feed source stream 状态缺口、必要单元参数缺失和组成未归一由 readiness 定位到具体 stream / unit。
- 缺物性包、结构性连接、拓扑、非法旧项目和求解阶段参数失败继续交给正式 Run Panel 诊断 / recovery；readiness 不扩成第二套 solver。
- 轻量结果审阅已满足当前主路径判断；完整报表、模板、打印、批量导出和跨快照报表仍不进入当前阶段。

Studio UI：

- 顶部导航已收敛为 `文件 / 主页 / 物性 / 流程图 / 运行 / 结果 / 工具 / 设置`；打开 / 新建 / 保存 / 示例归入 `文件`。
- 独立 `物性` 页消费 `StudioGuiWindowPropertyPageModel`。普通空白项目创建后先进入 `物性`，Property 页入口、`window.property_context_toolbar`、顶部 `流程图` 导航、Home 显式 `返回工作区` 入口和内部进入 Workbench 行为共用 `flowsheet_modeling_enabled`；Home 返回按钮只在本次会话已打开或新建 case 后显示，当前 MVP 不渲染无效二级物性导航。
- 左侧栏稳定为 `模块 / 项目`：`模块` 承接受控放置 palette、作者任务清单和画布建议；`项目` 承接项目输入、示例入口、对象树和审阅状态。
- 中央 Canvas 只承接 `画布工具`、`画布状态`、`画布操作`、图例、画布实体和受控建议；放置入口留在左侧 `模块`，对象树留在左侧 `项目`，选择语义留在右侧栏。
- 右侧栏稳定为 `检查器 / 模块设置 / 模块结果`，并用同一 `画布选择` 上下文头消费 Canvas current selection / command presentation。
- `模块设置` 继续只消费 active unit Inspector DTO；参数摘要从现有字段、notice 和批量提交 / 放弃 command 派生，不新增第二套参数状态。
- 底部区域稳定为左侧 `消息 / 运行日志 / 收敛 / 建议 / 诊断 / 结果表` 与右侧 `状态汇总` 分栏；结果表的单元区消费 `review_summary.unit_results`，状态汇总额外扫读同源单元结果数量；结果表定位继续走正式 `inspector.focus_stream:*` / `inspector.focus_unit:*`，不维护第二套 shell 私有选择；顶部 `结果工具栏` 不直接铺开所有结果对象定位按钮，顶部 `运行工具栏` 不在 Monitor 按钮旁重复展示状态 chip；底部薄状态栏只做运行、快照、SI 单位、求解器、模式和当前选择扫读。

## 下一步

- 继续沿 `studio-client-main.pen`、brief、正式 DTO / command surface 做 Workbench 窄口径小切片；不直接做大规模 egui 布局重排。
- 若发现真实主路径上的状态不一致、入口重复或首屏密度影响建模判断，只做支撑主路径判断的必要调整。

## 验证节奏

- 核心数据、求解、保存和项目格式：必须测试。
- 新能力主路径：至少覆盖一条 happy path focused test。
- UI 展示细节：除非曾经造成 blocker，否则不为单个小展示点新增测试。
- 阶段收口：执行 `pwsh ./scripts/check-repo.ps1`。
- 若仓库级验证在沙盒中出现明显环境性失败，可按协作规则申请真实环境复验。

## 暂不推进

- 不继续在 β 第一刀上追加同构 Home 作者入口或同类 checklist；既有小案例清单只作为导航提示，不作为通用建模运行 gate。
- 仍不推进 tag、release notes、便携包刷新或对外发布自动化；这些事项等待后续明确发布节点。
- 不做自由连线编辑器、任意端口选择器、自动布线系统、完整拖拽布局编辑器、完整报表系统、跨快照报表、模板导出、完整参数表。
- 不引入第三方 CAPE-OPEN 模型、第三方物性包加载、完整组分数据库或完整 Thermodynamics PMC。
- 不把 CAPE-OPEN / COM 语义倒灌到 Rust Core。
- 不为未来可能需求预先堆叠不明意义的 helper / manager / orchestrator / context / adapter。

## 按需阅读

- 最新流水和决策依据：`docs/devlogs/2026-06/2026-W24.md`
- 上周阶段收口：`docs/devlogs/2026-06/2026-W23.md`
- UI 专题设计前置：`docs/architecture/studio-ui-topic-plan.md`
- MVP β 人工 smoke 与验收标准：`docs/mvp/beta-acceptance-checklist.md`
- MVP 范围和非目标：`docs/mvp/scope.md`
- MVP 路线图：`docs/radishflow-mvp-roadmap.md`
- 仓库全局模块边界：`docs/architecture/overview.md`
- App / Canvas / UI：`docs/architecture/app-architecture.md`、`docs/architecture/canvas-interaction-contract.md`、`docs/architecture/studio-ui-design-guidelines.md`
- 热力学 / 闪蒸细节：`docs/thermo/mvp-model.md`
- CAPE-OPEN / COM 边界：`docs/capeopen/boundary.md`
- 代码风格、命名或抽象判断：`docs/development/code-style.md`
- 文档篇幅和拆分规则：`docs/README.md`

## 更新规则

- 本文档只保留当前阶段、下阶段目标、验证节奏和暂不推进项。
- 历史流水写入周志；长期边界写入专题文档；不要把本文档写成长篇进度报告。
- 每次完成重要阶段收口后，优先更新本文档顶部阶段结论和下阶段目标。
