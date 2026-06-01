# 当前状态

更新时间：2026-06-01

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
- **阶段性门禁已调整：MVP β smoke、指定小案例入口和结果审阅覆盖面不再作为日常推进 gate；后续以普通空白项目真实建模缺口、结果新旧状态表达和 readiness / Run Panel 边界为主线。**
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
- `Cooler` / `Valve` 已可作为通用空白建模的一等受控路径继续推进；当前仍不新增 Home 作者入口。
- UI 约束从“不做 UI”调整为“不做视觉精修和大改版”；允许服务建模正确性的状态表达、旧结果失效提示、结果新旧标识和轻量审阅材料改进。
- 轻量结果审阅可继续增强单次 `SolveSnapshot` 摘要；完整报表、模板、打印、批量导出和跨快照报表仍不进入当前阶段。
- readiness 只拦截确定的建模输入缺失；结构性连接、拓扑、非法旧项目或求解阶段参数失败继续交给正式 Run Panel 诊断 / recovery。

不再主动追逐 hover、提示、按钮文案、局部 selector、presentation 小瑕疵或更多同构作者入口。

## 下阶段目标

建模输入能力 v0、结果核对与案例说明 v0、受控连接恢复 v0、剩余单元建模闭环 v0、失败修复闭环 v0、MVP β 人工 smoke v0 与仓库级阶段基线验证均已通过。当前继续推进 **通用小流程建模 v1**；不回到零散 UI 打磨，也不把工作停在案例说明或验收文档上。

通用小流程建模 v1 目标：

- 普通空白项目不再进入或自动匹配 `Mixer-Flash` / `Heater-Flash` 小案例状态。
- 运行前检查按当前 `Flowsheet` 的真实建模输入判断，不再按某个案例步骤阻断；物性包选择仍由正式 run package resolution 判断，避免 shell 误拦可由本地唯一缓存包解析的旧示例项目。
- 在受控范围内支持用户自行组合 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum`。
- 缺项目组分、缺 Feed composition、Feed source stream 状态缺口、必要单元参数缺失和组成未归一等建模输入问题应由 readiness 定位到具体 stream / unit；未连接 material port、缺失 stream reference、重复 source / sink、orphan stream 和 cycle 等结构性问题继续进入正式 Run Panel 诊断 / recovery；缺物性包继续走正式运行命令的 package 解析与 Run Panel 诊断。
- 保存 / 重开 / rerun 仍必须稳定；不引入自由连线编辑器、自动布线、完整拖拽布局器或完整报表系统。

当前进展摘要：

- 空白项目需显式选择 `binary-hydrocarbon-lite-v1` 与 methane / ethane，Feed composition 和 Unit 参数均走正式 Inspector draft / commit / undo / save / reopen 路径。
- 通用 readiness 已按真实 `Flowsheet` 检查 Feed source stream T/P/F/z、项目组分、composition 归一和必要单元参数；结构性连接 / 拓扑 / 求解阶段参数失败继续走正式 Run Panel 诊断 / recovery。
- `Run` 与 `Resume` 两个用户运行入口已统一使用同一 readiness 判断；shell、AppHost、StudioGuiDriver 与 command registry 分发不再各自维护不同建模输入口径；在 Hold 且存在 pending reason 的空白项目中，Resume 保留 pending reason 并引导用户先补建模输入。
- 普通空白项目已覆盖 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum` 的显式输入、保存 / 重开 / rerun、结果审阅和关键结果合理性。
- 结果审阅当前包括 Result Inspector、底部结果表、Results commands、轻量导出、case-level `review_summary`、单相 Flash 零流量出口缺席语义和失败态定位。
- 编辑后重跑一致性已覆盖 Feed composition、unit 参数和连接状态变更；旧快照只作为 stale notice 来源，不再驱动结果审阅或导出入口。
- 下一步建议继续观察普通空白项目真实建模缺口与 readiness / Run Panel 诊断边界：只在发现主路径 blocker、结果判断缺口或文档事实源冲突时继续推进。

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
