# 当前状态

更新时间：2026-05-27

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

## 当前开发策略

当前项目由个人开发者推进，后续不再用“持续补细颗粒度体验缺口”的方式消耗主线节奏。已经通过的阶段只修真实 blocker：

- 无法完成主路径建模
- 无法运行或运行结果明显错误
- 保存 / 重开破坏项目
- 文档事实源与代码能力明显冲突
- 仓库级验证或核心 focused test 失败

不再主动追逐 hover、提示、按钮文案、局部 selector、presentation 小瑕疵或更多同构作者入口。`Feed -> Valve -> Flash Drum` 作者路径、更多 UI 小补口和完整结果表增强先降级为 backlog。

## 下阶段目标

**MVP β 后续能力包：先定题，再实施。**

建模输入能力 v0 已收口。下一步不继续补零散 UI 小项，优先在下面两类方向中选一个成组推进：

- **结果核对与案例说明 v0**：把 official demo case 的输入、关键中间流股、flash 分割、焓值 / 相态等结果解释成用户可核对的说明和轻量导出能力。
- **下一组高频建模能力**：在不引入完整组件库、完整物性包系统或自由连线编辑器的前提下，选择一个真实建模 blocker 成组推进。

候选未定前，不推进 tag、release notes、便携包刷新或对外发布自动化。

后续能力包判断标准：

- 能服务用户真实建模或结果判断路径。
- 有明确退出标准和 focused 验证。
- 不引入当前暂不推进项中的复杂度。

当前推进切片：

- 已落地“项目级物性包选择”主路径：`Flowsheet` 承载热力学配置，项目 JSON 可保存 / 重开该选择，`Preferred` 运行解析优先使用项目中保存的 package。
- Studio 已从只读展示推进到受控内置物性包选择 UI：右侧 `物性包` 页展示内置 package 选项，选择写入 `Flowsheet.thermo.property_package_id`，保存 / 重开保持，并由 `Preferred` 运行使用。
- 已落地“项目组分选择 v0”：Studio 左侧 `项目` 面板和右侧 `物性包` 页暴露受控内置 methane / ethane 组分目录，选择写入 `Flowsheet.components`，保存 / 重开保持；删除只允许未被任何 stream composition 引用的组件，Feed composition 的受控添加项继续从项目组件列表派生。
- 已修正空白项目主路径：新建未命名空白项目不再预写默认物性包和默认组分；用户需从受控内置列表显式选择 `binary-hydrocarbon-lite-v1` 与 methane / ethane 后，再进入 Feed composition、单元参数、运行、保存 / 重开路径。
- 已修正 Unit Inspector 参数输入页 blocker：单元参数不再用窄表格挤压长字段说明，改为本地化短标签、输入框、单位、状态和操作的紧凑行布局；约束提示缩短为辅助说明，不再把英文长句挤成竖排。
- 已补 Feed composition 输入主路径回归：从 official Heater-Flash 示例复制临时项目，走真实 Stream Inspector draft update / normalize / save / reopen / Preferred run 路径，把 `stream-feed` 组成从草稿归一到 methane 0.25 / ethane 0.75，并在保存项目和求解结果中核对。
- 已推进 official demo case 复现验收：`docs/guides/author-small-cases.md` 写清 `Heater-Flash` 与 `Mixer-Flash` 两条输入表和结果核对点；空白项目 focused 回归覆盖项目组分、内置物性包、Feed composition、Unit 参数、保存 / 重开、Preferred run 与 `SolveSnapshot` 核对。
- 下一步为 β 后续能力包定题：优先评估“结果核对与案例说明 v0”是否作为下一刀；不回到已通过阶段的零散 UI 打磨。

## 验证节奏

- 核心数据、求解、保存和项目格式：必须测试。
- 新能力主路径：至少覆盖一条 happy path focused test。
- UI 展示细节：除非曾经造成 blocker，否则不为单个小展示点新增测试。
- 阶段收口：执行 `pwsh ./scripts/check-repo.ps1`。
- 若仓库级验证在沙盒中出现明显环境性失败，可按协作规则申请真实环境复验。

## 暂不推进

- 不继续在 β 第一刀上追加 `Valve-Flash` 作者入口或同类 checklist。
- 不推进 tag、release notes、便携包刷新或对外发布自动化；恢复版本节点前必须先定义验收标准。
- 不做自由连线编辑器、自动布线系统、完整拖拽布局编辑器、完整报表系统、完整参数表。
- 不引入第三方 CAPE-OPEN 模型、第三方物性包加载、完整组分数据库或完整 Thermodynamics PMC。
- 不把 CAPE-OPEN / COM 语义倒灌到 Rust Core。
- 不为未来可能需求预先堆叠不明意义的 helper / manager / orchestrator / context / adapter。

## 按需阅读

- 最新流水和决策依据：`docs/devlogs/2026-05/2026-W22.md`
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
