# 当前状态

更新时间：2026-05-26

## 用途

用途：为新会话恢复上下文、判断“今天做什么”提供入口。  
读者：开发者、用户、AI / Agent。  
不包含：完整历史流水、详细设计推演、测试日志和长期说明书。

默认先读本文档。只有当任务需要具体实现细节、历史依据或专题边界时，再读取下方“按需阅读”列表。`AGENTS.md` / `CLAUDE.md` 只保留长期协作规则，不承载当前阶段流水。

## 阶段结论

- 产品定位：以 Rust Core + Rust UI + `.NET 10` CAPE-OPEN / COM 适配层构建稳态流程模拟软件。
- MVP 第一阶段 M1-M5、MVP α 内部验收和首版 demo 前硬化期已经阶段性收口。
- **MVP β 第一刀：小案例作者体验 v0 已通过。**
- 当前尚未进入正式 tag / release 节点；历史 `v26.5.1-dev` 只作为内部 staging 草案和验证记录保留。

β 第一刀通过依据：

- Studio 可从 Home 进入 `Feed + Feed -> Mixer -> Flash Drum` 与 `Feed -> Heater -> Flash Drum` 两条空白项目作者路径。
- 两条路径均复用正式 placement / suggestion / Unit Inspector / run / save / reopen / result review / snapshot export 边界，不是项目向导、自由连线或自动建模系统。
- 运行成功后可在右侧 `结果`、底部 `结果表` 和 `复制快照` / `导出文本` 中审阅同一份 `SolveSnapshot`。
- 2026-05-26 仓库级验证 `pwsh ./scripts/check-repo.ps1` 已在真实环境通过。

## 当前开发策略

当前项目由个人开发者推进，后续不再用“持续补细颗粒度体验缺口”的方式消耗主线节奏。已经通过的阶段只修真实 blocker：

- 无法完成主路径建模
- 无法运行或运行结果明显错误
- 保存 / 重开破坏项目
- 文档事实源与代码能力明显冲突
- 仓库级验证或核心 focused test 失败

不再主动追逐 hover、提示、按钮文案、局部 selector、presentation 小瑕疵或更多同构作者入口。`Feed -> Valve -> Flash Drum` 作者路径、更多 UI 小补口和完整结果表增强先降级为 backlog。

## 下阶段目标

**MVP β 第二刀：建模输入能力 v0。**

目标是让用户不再只能跑固定样例，而是能在受控范围内配置组分、选择内置物性方法、输入 Feed 组成和单元参数，并运行一个自己定义的小流程。演示案例用于验收这组输入能力，不再作为主目标本身。

优先级：

1. **组分输入 / 组分选择 v0**：先做内置小型组分目录，不做完整组分数据库；允许项目在受控范围内选择组分，并复用现有 Feed composition draft / normalize / commit 机制。
2. **物性方法 / 物性包选择 v0**：先支持内置物性方法或内置 package 的显式选择，不加载第三方 Property Package；选择结果必须保存 / 重开后保持，并能影响求解路径或结果。
3. **建模输入工作流 UI**：围绕“项目组分 / 物性方法 / Feed composition / Unit 参数 / 运行结果”串成可理解的工作流，不做独立大规模视觉精修。
4. **demo case 验收**：用新的输入能力复现 2 个 official demo case，并写清参数、预期结果和核对点。

阶段退出标准：

- 用户可在受控范围内选择项目组分，并能编辑 Feed 组成后运行。
- 用户可选择一个内置物性方法 / package；该选择进入项目持久化，保存 / 重开后仍可运行。
- 至少 2 个 official demo case 由这套输入能力支撑，而不是只依赖硬编码固定样例。
- 每个 demo case 有明确的结果核对点，能解释温度、压力、流量、相态、焓值或 flash 分割中的关键变化。
- 相关核心路径有 focused 自动化验证；阶段收口时跑 `pwsh ./scripts/check-repo.ps1`。
- 不引入完整组分数据库、完整物性包系统、第三方 Property Package 加载、完整报表、自由连线、自动布线或完整参数表。

当前推进切片：

- 已开始落地“项目级物性包选择”主路径：`Flowsheet` 承载热力学配置，项目 JSON 可保存 / 重开该选择，`Preferred` 运行解析优先使用项目中保存的 package。
- 下一步接 UI 控制：从只读项目树展示推进到受控内置 package 选择，并与后续项目组分选择保持同一输入工作流。

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
