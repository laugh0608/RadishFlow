# 当前状态

更新时间：2026-06-06

## 用途

用途：为新会话恢复上下文、判断“今天做什么”提供入口。  
读者：开发者、用户、AI / Agent。  
不包含：完整历史流水、详细设计推演、测试日志和长期说明书。

默认先读本文档。只有当任务需要具体实现细节、历史依据或专题边界时，再读取下方“按需阅读”列表。`AGENTS.md` / `CLAUDE.md` 只保留长期协作规则，不承载当前阶段流水。

## 阶段结论

- 产品定位：以 Rust Core + Rust UI + `.NET 10` CAPE-OPEN / COM 适配层构建稳态流程模拟软件。
- MVP 第一阶段 M1-M5、MVP α 内部验收和首版 demo 前硬化期已经阶段性收口。
- **MVP β 第一刀：小案例作者体验 v0 已通过。**
- **MVP β 第二刀：建模输入能力 v0 已通过。**
- **MVP β 后续能力包：结果核对与案例说明 v0 已落地第一版。**
- **MVP β 下一组高频建模能力：受控连接恢复 v0 与剩余单元建模闭环 v0 已完成 focused 收口。**
- **MVP β 失败修复闭环 v0 已完成 focused 收口：缺物性包、缺项目组分、缺 composition、参数越界、连接 blocker、cycle 与 invalid port signature 已锁定。**
- **MVP β 人工 smoke v0 已通过：Smoke A-D 均已由人工完成，未发现 blocker。**
- **MVP β 阶段基线验证已通过：2026-05-28 真实环境 `pwsh ./scripts/check-repo.ps1` 通过。**
- **MVP β 下一阶段已切到通用小流程建模 v1：目标从复现指定案例推进到空白项目中受控组合小流程。**
- **通用小流程建模 v1 第十一切片已完成 focused 推进：普通空白项目结果审阅入口已覆盖 Results commands、轻量导出、底部结果表、Result Inspector 呈现、关键结果合理性核对、单相 Flash 零流量出口缺席语义、重开后正式求解失败的诊断上下文定位，以及 case-level review summary。**
- **通用小流程建模 v1 第十二切片已完成 focused 推进：普通空白项目成功求解后修改 Feed composition、unit 参数或连接状态时，旧结果不再作为当前结果入口暴露，并由 stale notice 指向重新运行；rerun 后 Result Inspector、底部结果表、Results commands、轻量导出和 case-level review summary 回到最新 `SolveSnapshot`。**
- **通用小流程建模 v1 第十三切片已完成 focused 推进：Run Panel `Resume` 与手动 `Run` 使用同一 modeling readiness 入口；空白项目待运行状态下的 Resume 不绕过建模输入诊断，不制造正式求解失败。**
- **通用小流程建模 v1 第十四切片已完成 focused 推进：AppHost、StudioGuiDriver、StudioGuiHost command registry 等非 shell 运行入口共用同一 modeling readiness 判断；Feed 输入缺口和必要单元参数缺失不再绕过 readiness 进入正式求解失败，物性包解析、结构性连接、拓扑与求解阶段失败仍保留 Run Panel 正式诊断 / recovery。**
- **通用小流程建模 v1 第十五切片已完成 focused 推进：Mac 真实 UI 字体与普通空白项目 Heater 默认参数提交 / readiness notice 推进人工复核已通过；缺显式 unit parameter 的 Inspector 显示值现在可直接提交为正式参数。**
- **通用小流程建模 v1 第十六切片已完成 focused 推进：Feed、Cooler、Valve、Mixer、Flash Drum 的 Inspector 显示默认 / 模板参数均已通过真实窗口模型提交命令回归，提交后写入正式 `UnitOperationParameters`。**
- **通用小流程建模 v1 当前阶段基线验证已通过：2026-06-02 真实环境 `./scripts/check-repo.sh` 通过。**
- **通用小流程建模 v1 阶段收口复核已通过：普通空白项目 `Feed -> Flash Drum`、`Feed -> Cooler -> Flash Drum`、`Feed -> Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum` 均已完成显式输入、运行、保存、重开、rerun 与结果审阅复核，未发现 blocker。**
- **下一阶段已切到 Studio UI 专题：设计前置首轮已收敛，代码实现只从 presentation / window model 第一刀开始；不在现有 UI 上继续叠加零散补丁。**
- **Studio UI 主设计稿已收敛到 `studio-client-main.pen`：当前稿覆盖高密度 Home、顶部导航下的独立物性页、流程图工作台、模块设置 / 模块结果和底部运行 / 状态分栏；重复的 `unit-module-panel.pen` 不再作为活跃设计稿维护。**
- **Studio UI 实现第六刀已完成 focused 推进并通过人工复核：Home 最近项目已在 shell preferences / recent path 边界内映射为同一 `StudioGuiWindowHomeCaseTileModel`，Home Dashboard 的最近项目和示例项目不再分叉两套 presentation；recent tile 支持 MRU、current project、missing file、项目文件标题 / flowsheet / 组分和拓扑缩影。人工复核已覆盖 Home 空状态、打开示例后生成 recent tile、tile 信息、状态语义和选择 / 打开交互，未发现 blocker。**
- 当前尚未进入正式 tag / release 节点；历史 `v26.5.1-dev` 只作为内部 staging 草案和验证记录保留。

## 当前开发策略

当前项目由个人开发者推进，后续不再用“持续补细颗粒度体验缺口”的方式消耗主线节奏。已经通过的阶段只修真实 blocker：

- 无法完成主路径建模
- 无法运行或运行结果明显错误
- 保存 / 重开破坏项目
- 文档事实源与代码能力明显冲突
- 仓库级验证或核心 focused test 失败

阶段性门禁调整如下：

- `Mixer-Flash` / `Heater-Flash` 作者路径、MVP β Smoke A-D 和结果审阅对象覆盖不再作为日常开发 gate；它们保留为代表性回归和阶段收口参考。
- `Cooler` / `Valve` 已作为通用空白建模的一等受控路径完成复核；当前仍不新增 Home 作者入口。
- UI 工作进入专题设计前置，不在功能推进中继续分散修补按钮、文案、hover、局部 selector 或展示小瑕疵。
- 轻量结果审阅已满足当前主路径判断；完整报表、模板、打印、批量导出和跨快照报表仍不进入当前阶段。
- readiness 只拦截确定的建模输入缺失；结构性连接、拓扑、非法旧项目或求解阶段参数失败继续交给正式 Run Panel 诊断 / recovery。

不再主动追逐 hover、提示、按钮文案、局部 selector、presentation 小瑕疵或更多同构作者入口。

## 下阶段目标

建模输入能力 v0、结果核对与案例说明 v0、受控连接恢复 v0、剩余单元建模闭环 v0、失败修复闭环 v0、MVP β 人工 smoke v0、通用小流程建模 v1 与仓库级阶段基线验证均已通过。下一步推进 **Studio UI 专题设计前置**；不回到零散 UI 打磨，也不把工作停在案例说明或验收文档上。

通用小流程建模 v1 已完成的能力基线：

- 普通空白项目不再进入或自动匹配 `Mixer-Flash` / `Heater-Flash` 小案例状态。
- 运行前检查按当前 `Flowsheet` 的真实建模输入判断，不再按某个案例步骤阻断；物性包选择仍由正式 run package resolution 判断，避免 shell 误拦可由本地唯一缓存包解析的旧示例项目。
- 在受控范围内支持用户自行组合 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum`。
- 缺项目组分、缺 Feed composition、Feed source stream 状态缺口、必要单元参数缺失和组成未归一等建模输入问题应由 readiness 定位到具体 stream / unit；未连接 material port、缺失 stream reference、重复 source / sink、orphan stream 和 cycle 等结构性问题继续进入正式 Run Panel 诊断 / recovery；缺物性包继续走正式运行命令的 package 解析与 Run Panel 诊断。
- 保存 / 重开 / rerun 仍必须稳定；不引入自由连线编辑器、自动布线、完整拖拽布局器或完整报表系统。

Studio UI 专题设计前置目标：

- 先基于 `docs/architecture/studio-ui-topic-plan.md`、`docs/architecture/studio-ui-design-guidelines.md` 与 `docs/architecture/ui-inspiration-reference.md` 整理 Studio 客户端本体、单元模块 UI、服务端 / 控制面 UI、移动端或只读视图的端点边界。
- P0 Studio 主设计稿已按成熟流程模拟软件的信息分层完成方向修正：顶部使用 `文件 / 主页 / 物性 / 流程图 / 运行 / 结果 / 工具 / 设置`，`物性` 是独立页面，底部使用左侧运行信息 tabs 与右侧状态汇总。Home 最近 / 示例案例入口改为 RadishFlow 浅色流程缩影 tile gallery 方向：用 flowsheet thumbnail + 项目名 + 路径 / 来源 + 时间 + 状态 chip 表达案例，不照抄 HYSYS 深蓝文件图标或左侧文件菜单。后续优先评审 `studio-client-main.pen` 的 Home、物性页、流程图工作台、模块设置 / 模块结果结构能否稳定承载参数、端口、运行结果、诊断和帮助入口，再决定是否进入代码实现。
- P0 / P1 评审时继续对照两张 `baseline/` 视觉基线，确保 Home / Workbench 的分区、信息密度、状态 chip 和主操作层级与项目视觉方向一致。
- Studio 客户端本体优先覆盖 Home、Workbench、Canvas、Inspector、Result、Package / Auth 的职责关系，不提前扩自由连线、完整拖拽布局、自动布线、完整参数表或完整结果报表。
- 模块设置 / 模块结果优先先在 `studio-client-main.pen` 中统一参数、端口、运行结果、诊断和帮助入口；若后续细节不足，再按窄口径创建 `module-settings-panel.pen`，不重复整套 Workbench。
- 代码实现已允许从 presentation / window model 小切片推进，但仍不直接做大规模 egui 布局重排；Module Results DTO 已落到正式 window model 并由 Results tab 窄口径消费；Home 最近项目继续由 shell preferences / recent path 持有，但已映射为同一 case tile DTO 并由 Home Dashboard 消费且人工复核通过。下一步转向右侧 Inspector / Module Settings 是否需要专门窄口径 DTO 或 `.pen` 细化；是否把 recent projects 从 shell preferences 上提到正式 snapshot 暂不作为当前 blocker。
- 任何代码实现都必须继续遵守 presentation / command / state 边界；视觉优化不得绕过正式 UI 模型堆 shell 私有状态。

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

- 最新流水和决策依据：`docs/devlogs/2026-06/2026-W23.md`
- 上周阶段收口：`docs/devlogs/2026-05/2026-W22.md`
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
