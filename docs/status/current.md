# 当前状态

更新时间：2026-05-31

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
- **通用小流程建模 v1 第九切片已完成 focused 推进：普通空白项目结果审阅入口已覆盖 Results commands、轻量导出、底部结果表、Result Inspector 呈现、关键结果合理性核对，以及单相 Flash 零流量出口缺席语义。**
- 当前尚未进入正式 tag / release 节点；历史 `v26.5.1-dev` 只作为内部 staging 草案和验证记录保留。

β 第一刀通过依据：

- Studio 可从 Home 进入 `Feed + Feed -> Mixer -> Flash Drum` 与 `Feed -> Heater -> Flash Drum` 两条空白项目作者路径。
- 两条路径均复用正式 placement / suggestion / Unit Inspector / run / save / reopen / result review / snapshot export 边界，不是项目向导、自由连线或自动建模系统。
- 运行成功后可在右侧 `结果`、底部 `结果表` 和 `复制快照` / `导出文本` 中审阅同一份 `SolveSnapshot`。
- 2026-05-26 仓库级验证 `pwsh ./scripts/check-repo.ps1` 已在真实环境通过。

β 第二刀通过依据：

- 空白项目初始不预选物性包或组分；用户需显式选择 `binary-hydrocarbon-lite-v1`、methane、ethane。
- 项目级物性包选择、项目组分选择、Feed composition draft / normalize / commit、Unit 参数输入、保存 / 重开和 `Preferred` run 已形成可复验闭环。
- `Heater-Flash` 与 `Mixer-Flash` 两条 official demo case 已写清输入表和结果核对点，并由空白项目 focused 回归覆盖到最新 `SolveSnapshot`。
- 2026-05-27 仓库级验证 `pwsh ./scripts/check-repo.ps1` 已在真实环境通过。

结果核对与案例说明 v0 第一版依据：

- `docs/guides/author-small-cases.md` 已按 official hydrocarbon `Heater-Flash` / `Mixer-Flash` demo case 整理关键输入、关键中间流股、flash 分割、相态和焓值核对路径。
- `docs/guides/review-solve-results.md` 已把小案例审阅顺序从“先看哪些对象”细化为输入流股、非 flash 中间流股、unit step、flash 分割、相态 / `H` 和轻量导出的核对链路。
- focused 验证覆盖 solver `SolveSnapshot` 中 official demo case 的输入、中间流股、unit step 消费 / 产出、flash 分割、相态 / `H`，并覆盖 Studio window-model 轻量导出是否保留这些核对对象。

## 当前开发策略

当前项目由个人开发者推进，后续不再用“持续补细颗粒度体验缺口”的方式消耗主线节奏。已经通过的阶段只修真实 blocker：

- 无法完成主路径建模
- 无法运行或运行结果明显错误
- 保存 / 重开破坏项目
- 文档事实源与代码能力明显冲突
- 仓库级验证或核心 focused test 失败

不再主动追逐 hover、提示、按钮文案、局部 selector、presentation 小瑕疵或更多同构作者入口。`Feed -> Valve -> Flash Drum` 作者路径、更多 UI 小补口和完整结果表增强先降级为 backlog。

## 下阶段目标

**MVP β 人工 smoke v0 已通过。**

建模输入能力 v0、结果核对与案例说明 v0 第一版、受控连接恢复 v0、剩余单元建模闭环 v0、失败修复闭环 v0、MVP β 人工 smoke v0 与仓库级阶段基线验证均已通过。下一步推进 **通用小流程建模 v1**；不回到零散 UI 打磨，也不把工作停在案例说明或验收文档上。

人工 smoke v0 已通过的路径：

- 打开 official demo case，运行、结果审阅、保存 / 重开 / rerun。
- 从 Home 作者入口手工复现 `Mixer-Flash` 和 `Heater-Flash` 空白项目路径。
- 覆盖代表性失败恢复：缺 Feed composition 诊断与修复、selected stream 断开 / 重连 / 保存重开。

通过 / 失败记录以 `docs/mvp/beta-acceptance-checklist.md` 为准；当前仍不推进 tag、release notes、便携包刷新或对外发布自动化。

通用小流程建模 v1 目标：

- 普通空白项目不再进入或自动匹配 `Mixer-Flash` / `Heater-Flash` 小案例状态。
- 运行前检查按当前 `Flowsheet` 的真实建模输入判断，不再按某个案例步骤阻断；物性包选择仍由正式 run package resolution 判断，避免 shell 误拦可由本地唯一缓存包解析的旧示例项目。
- 在受控范围内支持用户自行组合 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum`。
- 缺项目组分、缺 Feed composition、Feed source stream 状态缺口、必要单元参数缺失和组成未归一等建模输入问题应由 readiness 定位到具体 stream / unit；未连接 material port、缺失 stream reference、重复 source / sink、orphan stream 和 cycle 等结构性问题继续进入正式 Run Panel 诊断 / recovery；缺物性包继续走正式运行命令的 package 解析与 Run Panel 诊断。
- 保存 / 重开 / rerun 仍必须稳定；不引入自由连线编辑器、自动布线、完整拖拽布局器或完整报表系统。

当前推进切片：

- 已落地“项目级物性包选择”主路径：`Flowsheet` 承载热力学配置，项目 JSON 可保存 / 重开该选择，`Preferred` 运行解析优先使用项目中保存的 package。
- Studio 已从只读展示推进到受控内置物性包选择 UI：右侧 `物性包` 页展示内置 package 选项，选择写入 `Flowsheet.thermo.property_package_id`，保存 / 重开保持，并由 `Preferred` 运行使用。
- 已落地“项目组分选择 v0”：Studio 左侧 `项目` 面板和右侧 `物性包` 页暴露受控内置 methane / ethane 组分目录，选择写入 `Flowsheet.components`，保存 / 重开保持；删除只允许未被任何 stream composition 引用的组件，Feed composition 的受控添加项继续从项目组件列表派生。
- 已修正空白项目主路径：新建未命名空白项目不再预写默认物性包和默认组分；用户需从受控内置列表显式选择 `binary-hydrocarbon-lite-v1` 与 methane / ethane 后，再进入 Feed composition、单元参数、运行、保存 / 重开路径。
- 已修正 Unit Inspector 参数输入页 blocker：单元参数不再用窄表格挤压长字段说明，改为本地化短标签、输入框、单位、状态和操作的紧凑行布局；约束提示缩短为辅助说明，不再把英文长句挤成竖排。
- 已补 Feed composition 输入主路径回归：从 official Heater-Flash 示例复制临时项目，走真实 Stream Inspector draft update / normalize / save / reopen / Preferred run 路径，把 `stream-feed` 组成从草稿归一到 methane 0.25 / ethane 0.75，并在保存项目和求解结果中核对。
- 已推进 official demo case 复现验收：`docs/guides/author-small-cases.md` 写清 `Heater-Flash` 与 `Mixer-Flash` 两条输入表和结果核对点；空白项目 focused 回归覆盖项目组分、内置物性包、Feed composition、Unit 参数、保存 / 重开、Preferred run 与 `SolveSnapshot` 核对。
- 已推进结果核对与案例说明 v0 第一版：两条 official demo case 的输入、中间流股、flash 分割、相态 / 焓值核对路径已写入 guide，并补 focused test 锁定 solver snapshot 与 Studio 轻量导出的关键审阅对象。
- 已修正小案例作者清单输入就绪缺口：清单不再只看拓扑和快照，也会提示物性包、项目组分、Feed composition 和必要单元参数是否已提交，避免拓扑完成后直接运行才暴露缺组成错误。
- 已修正缺少流股组成时的求解诊断：下游单元消费未提交 composition 的流股时，solver 现在返回 `solver.step.stream_input`，并携带相关 stream 与 inlet 端口；Run Panel 恢复动作聚焦到流股输入，而不是泛化为单元执行失败。
- 已完成受控连接恢复 v0 focused 收口：验证锁定 official Heater-Flash case 中 selected stream 断开 sink、重连唯一 Flash inlet、保存 / 重开 / rerun 后仍由 Flash Drum 消费 heater outlet 的闭环。
- 已完成剩余单元建模闭环 v0 focused 收口：补 focused 验证覆盖空白项目中显式选择内置物性包 / 组分后，手工搭建 `Feed -> Cooler -> Flash Drum` 与 `Feed -> Valve -> Flash Drum`，提交 Feed composition 和单元参数，保存 / 重开 / rerun，并核对中间流股、flash consumed stream、液/汽出口、相态 / `H` 基础审阅对象；2026-05-27 仓库级验证 `pwsh ./scripts/check-repo.ps1` 已在真实环境通过。
- 已完成失败修复闭环 v0 focused 收口：回归覆盖缺物性包、缺项目组分、缺 Feed composition、Valve 参数越界、主要连接 blocker、cycle 和 invalid port signature；2026-05-28 真实环境 `pwsh ./scripts/check-repo.ps1` 通过。
- 已完成 MVP β 人工 smoke v0：Smoke A-D 已由人工执行并通过，未发现 `AuthoringPath`、`ModelingInput`、`FailureRecovery`、`ResultReview` 或 `Persistence` blocker；记录已落到 `docs/mvp/beta-acceptance-checklist.md`。
- 已完成 MVP β 阶段基线验证：2026-05-28 真实环境 `pwsh ./scripts/check-repo.ps1` 通过。
- 已启动通用小流程建模 v1 第一切片：运行按钮不再由 active 小案例清单拦截，改为所有项目共享的建模输入 readiness；普通空白项目与小案例入口都会用“模型输入未完成”指向真实 flowsheet 缺口，property package 解析仍留在正式 run command。
- 已推进通用小流程建模 v1 第二切片：运行前 readiness 已按 Feed source stream 状态检查 T/P/F/z、项目组分引用和 composition 归一，并按 unit kind 要求 Heater / Cooler / Flash Drum 的出口 T/P 以及 Mixer / Valve 的出口压力；官方示例项目同步补齐正式单元参数，普通空白项目不再靠拓扑建议直接运行。
- 已推进通用小流程建模 v1 第三至第九切片：`Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum` 与 `Feed + Feed -> Mixer -> Flash Drum` 均覆盖显式输入、保存端口绑定、重开 rerun、单元 step、结果检查器、Results commands、轻量导出和 outlet T/P/F/H；底部结果表与 Result Inspector focused 回归已锁定输入流股、中间流股、Flash 液 / 汽出口、unit consumed / produced stream references、composition、phase 与 bubble/dew window 入口；结果合理性回归已覆盖 Flash split 总量 / 组分物料衡算、单入口单元出口 T/P/F/z 一致性和 Mixer 流量加权 composition；单相 Flash 零流量 outlet 已锁定不伪造 `H`、phase rows 或 bubble/dew window，右侧 Result Inspector 以 `none` 相态摘要表达缺席语义；host / window-model 层已补回归锁定 blocked modeling input 不启用 failure recovery，结构性连接 / 拓扑错误继续进入正式 Run Panel 诊断 / recovery。

## 验证节奏

- 核心数据、求解、保存和项目格式：必须测试。
- 新能力主路径：至少覆盖一条 happy path focused test。
- UI 展示细节：除非曾经造成 blocker，否则不为单个小展示点新增测试。
- 阶段收口：执行 `pwsh ./scripts/check-repo.ps1`。
- 若仓库级验证在沙盒中出现明显环境性失败，可按协作规则申请真实环境复验。

## 暂不推进

- 不继续在 β 第一刀上追加 `Valve-Flash` 作者入口或同类 checklist；既有小案例清单只作为导航提示，不作为通用建模运行 gate。
- 仍不推进 tag、release notes、便携包刷新或对外发布自动化；这些事项等待后续明确发布节点。
- 不做自由连线编辑器、自动布线系统、完整拖拽布局编辑器、完整报表系统、完整参数表。
- 不引入第三方 CAPE-OPEN 模型、第三方物性包加载、完整组分数据库或完整 Thermodynamics PMC。
- 不把 CAPE-OPEN / COM 语义倒灌到 Rust Core。
- 不为未来可能需求预先堆叠不明意义的 helper / manager / orchestrator / context / adapter。

## 按需阅读

- 最新流水和决策依据：`docs/devlogs/2026-05/2026-W22.md`
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
