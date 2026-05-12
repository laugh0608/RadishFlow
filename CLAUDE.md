# RadishFlow 协作约定

本文件为 RadishFlow 仓库中的 AI 协作者与人工协作者提供统一协作规范。  
它只约束本仓库的工作方式，不代表其他项目，也不复用其他项目的实现边界。

## 语言规范

- 默认使用中文进行讨论、说明、提交总结和开发日志记录
- 代码、命令、路径、配置键、类型名、接口名保留原文
- 新增文档默认使用中文，除非该文件天然要求英文

## 协作流程

- 开始任何任务前，先检查仓库状态，并阅读与当前任务直接相关的文档
- 若用户没有明确要求直接修改，编写任何代码之前，必须先说明方案并等待批准
- 若用户明确要求直接修改，且范围清晰、风险可控，则直接实施，不必先停在纯建议阶段
- 若需求不明确，或改动会影响架构、阶段边界、接口口径、验证基线，则先说明判断并做必要澄清
- 每次新增/修改功能、修复 bug 或处理其他任务时，优先从根因、长期维护性和系统一致性出发，选择更完整、更稳妥的治理方案；不要把“最小修复”当作默认优先级，也不要无节制地层层增加兜底来掩盖问题
- 修改规则、架构、接口、目录职责、MVP 范围或协作文档时，优先保持与 `docs/` 中现有正式文档一致
- 每做完一个可分割子步骤，都应进行最小验证；默认以 `docs/status/current.md` 中的当前验证基线为准，若该文档未覆盖，再执行与改动最相关的最小验证
- 重要阶段性决策除了改代码，还应同步更新对应文档；如果属于本周重要推进，追加到周志

## 文档真相源

`docs/` 是本仓库的正式文档源。新会话默认只优先读取最小入口文档，避免为获取背景而消耗过多上下文：

1. `docs/status/current.md`
2. `docs/README.md`

当用户询问“根据项目规划和开发进度，今天要做什么”这类问题时，优先读取 `docs/status/current.md`，再按该文档的“按需阅读”列表补读专题文档，不默认展开 `overview`、`scope` 或整篇周志。

涉及具体领域时，再按需读取对应专题文档，例如 `docs/capeopen/boundary.md`、`docs/thermo/mvp-model.md`、`docs/architecture/app-architecture.md`、`docs/architecture/auth-entitlement-architecture.md`、`docs/radishflow-mvp-roadmap.md`。

涉及实现风格争议、命名争议或抽象边界判断时，读取 `docs/development/code-style.md`。

`AGENTS.md` / `CLAUDE.md` 只保留长期稳定的协作规则；当前阶段、当前重点、当前验证基线、暂不推进项和其他易过期口径，以 `docs/status/current.md` 和对应专题文档为准，不在本文件重复展开。

规则：

- 若代码与文档冲突，先判断是代码偏离文档，还是文档已过期，再统一修正
- 优先更新已有文档，不为一次性讨论创建大量散文档
- `docs/` 的关键入口文档必须尽可能简约，只描述当前阶段、最近进度、稳定边界和下一步；历史背景、详细过程和长篇推演应放入专题文档或周志
- 更新关键入口文档时，应优先删减过期背景和重复叙述，避免让 AI/Agent 在新会话中读取大量低价值上下文
- 协作入口文件只保留长期稳定规则；阶段性口径优先写入 `docs/status/current.md` 与对应专题文档
- 文档篇幅按“默认阅读成本”和“职责单一性”治理，不照搬源码行数上限；中文 Markdown 优先看字符数、默认入口权重和是否混入历史流水
- `docs/status/current.md` 目标上限为 8k 字符，`docs/README.md` 目标上限为 10k 字符，`AGENTS.md` / `CLAUDE.md` 目标上限为 14k 字符；超过上限时应先删减重复背景、历史流水和低价值细节
- Guide / Runbook 单篇目标上限为 15k 字符；Reference / Architecture / Boundary 单篇目标上限为 25k-30k 字符；ADR 单篇目标上限为 12k 字符
- 超过目标上限的专题文档应优先拆成“入口摘要 + 专题正文”，或降级为历史材料并从默认阅读链移除；周志和历史草案可更长，但必须有顶部摘要和清晰索引
- 每篇新增或大改文档应在开头说明用途、读者和不包含内容，避免把使用说明、字段参考、架构推演与历史流水混写进同一篇文档
- 周志按 `docs/devlogs/YYYY-Www.md` 命名
- 许可条款以仓库根 `LICENSE` 文件为准

## Agent 协同文件

- 仓库中面向不同 Agent 入口名的协作文件，应保持“基本复制”和长期同步
- 这些同类协作文件不应演化出彼此冲突的规则口径
- 若某个同类协作文件更新了通用协作规则、执行边界、验证基线或阶段约束，其余同类文件也应尽快同步
- 同类协作文件只允许保留极少量与入口名称直接相关的表述差异，不应借此分叉实际协作规范

## 快速认知

- 产品定位：稳态流程模拟软件
- 核心技术栈：Rust Core + Rust UI + `.NET 10` CAPE-OPEN/COM 适配层
- 当前阶段、当前主线、当前重点、当前验证基线与暂不推进项：`docs/status/current.md`
- MVP 范围、冻结实现细节、非目标与近期节奏：`docs/mvp/scope.md`
- 架构边界与模块职责：`docs/architecture/overview.md`、`docs/architecture/app-architecture.md`
- CAPE-OPEN / COM 边界：`docs/capeopen/boundary.md`
- 分支与 PR 治理：`docs/adr/0001-branch-and-pr-governance.md`
- 外部参考：`CapeOpenCore` 仅作为 CAPE-OPEN / COM 语义和历史经验参考；`DWSIM` 仅作为模块拆分、自动化入口和 flowsheet solver 组织经验参考；二者都不在本仓库协作中跨工作区修改，也不直接迁移实现代码

## 仓库结构速记

### Rust 主线

- `apps/radishflow-studio/`: Rust 桌面应用入口
- `crates/rf-types`: 基础 ID、枚举、错误类型
- `crates/rf-model`: 组分、流股、相态、单元和 flowsheet 对象模型
- `crates/rf-thermo`: 热力学数据结构与接口
- `crates/rf-flash`: `TP Flash` 输入输出契约与求解器接口
- `crates/rf-unitops`: 单元模块
- `crates/rf-flowsheet`: 图结构与连接校验
- `crates/rf-solver`: 顺序模块法求解器
- `crates/rf-store`: 项目存储
- `crates/rf-ffi`: Rust 与 `.NET` 的 C ABI 边界

### 适配层与辅助目录

- `adapters/dotnet-capeopen/`: `.NET 10` CAPE-OPEN 适配层骨架
- `bindings/c/`: C 头文件与绑定产物落点
- `examples/`: 示例流程、样例数据、PME 验证样例
- `tests/`: Rust 集成测试、黄金样例、互操作测试
- `scripts/`: 仓库检查与自动化脚本

## AI 执行边界

### 可直接执行

- 代码读取、代码修改、文档修改
- `cargo check`
- `cargo test`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets`
- `git status`、`git diff`、`git log` 等只读 Git 操作
- 简洁明确的提交操作

### 需要先告知用户再执行

- 长时间运行或需要人工交互的命令，例如 `cargo run`、桌面 UI 启动
- 可能修改本机环境的命令，例如 COM 注册、反注册、安装证书、写系统注册表
- 依赖网络或可能引入依赖变更的命令，例如 `cargo add`、`cargo update`、`dotnet add package`、需要联网的 restore
- 打包、发布、安装类命令
- 如果关键构建、测试或 smoke 在沙盒环境里出现疑似环境性失败，可先告知用户并申请提权到真实环境执行同一验证命令，以确认问题是否真来自代码

### 默认不做

- 跨工作区编辑其他项目
- 把旧仓库代码整包迁入当前仓库
- 在 Rust 中直接引入 COM 语义和类型
- 未经明确要求执行破坏性 Git 操作

## 验证基线

默认先以 `docs/status/current.md` 中的当前验证基线为准；若当前任务对应的专题文档或用户要求另有明确口径，以更具体的口径为准。

补充说明：

- `scripts/check-repo.ps1` 与 `scripts/check-repo.sh` 当前是正式仓库级验证入口，复用同一套 Rust `xtask` 实现
- 如果某一步改动只涉及文档，仍应至少确认工作区未引入额外脏改动
- 仓库治理或 CI 改动优先执行 `pwsh ./scripts/check-repo.ps1`；在 Linux/macOS/CI 中执行 `./scripts/check-repo.sh`
- 仓库文本格式门禁只约束仓库自有文本资产；`adapters/reference/` 下的外部参考资料允许保留上游编码、BOM 与换行格式，不应为通过门禁而批量改写
- 如果重要构建、测试或 smoke 验证在沙盒环境中失败，且失败现象明显带有沙盒限制、受限 restore / project reference 解析、native 加载路径差异或其他环境隔离特征，应允许申请提权到真实环境复验，而不是把这类失败直接归因到代码
- 对当前仓库的 `.NET 10` CAPE-OPEN 解决方案、`dotnet build`、`dotnet run --project ... --no-build` smoke 与 `rf-ffi` native 装载路径，若沙盒结果与代码现状明显不符，应优先在获得授权后用真实环境完成最终验证

## 实现协作约束

- 单位统一使用 SI 基本单位
- 温度使用 K
- 压力使用 Pa
- 摩尔流量使用 mol/s
- Rust 不直接处理 COM；CAPE-OPEN / COM 语义只留在 `.NET` 适配层
- `rf-model` 只承载对象模型，不提前承载求解策略或 COM 语义
- 更具体的 MVP 实现约定，例如组成表达、相标签、工作区模型、结果模型和 UI 边界，以 `docs/mvp/scope.md`、`docs/architecture/app-architecture.md` 和 `docs/capeopen/boundary.md` 为准
- 单个源码文件原则上不超过 1500 行；若文件已接近或超过该阈值，后续新增实现应优先拆分职责、提取子模块或测试 helper，而不是继续膨胀原文件
- `src/` 下源码应按职责做浅层目录分组，优先使用 1 层子目录收纳同域模块，避免长期把所有模块平铺在 `src/` 根下，也避免为了“整齐”堆出过深目录树
- 每次新增/修改功能、修复 bug 或完成其他任务时，不应优先追求“最小修复方案”，而应优先考虑能否做出完善、稳妥的根治性修改
- 避免连续叠加治标不治本的兜底逻辑；如果问题的根因已可定位，应优先修正根因，而不是无止境地继续包裹一层又一层 fallback

## 语言与代码风格约束

- 详细规范见 `docs/development/code-style.md`；协作入口只保留必须始终遵守的高层约束
- 新增 Rust、C#、脚本或前端代码时，应遵循对应语言的主流、清晰、可维护实践，优先使用语言和标准库已有表达能力，而不是自造晦涩框架
- 命名必须表达真实领域职责；禁止新增含义不清的方法、类型或抽象层，例如只有技术包装意义、不能说明业务边界的泛化 helper、manager、orchestrator、context、adapter
- 若确需新增抽象，必须能减少真实重复、隔离稳定边界或表达明确领域概念，并应有清楚的调用面、错误语义和测试覆盖
- Rust 代码优先使用类型系统、`Result` / `Option`、所有权和小而明确的模块边界表达约束，避免通过字符串标记、全局状态或多层 fallback 掩盖模型问题
- C# / `.NET` 代码优先使用符合现代 .NET 的类型、异常和 interop 边界；COM / CAPE-OPEN 适配语义只能留在适配层，不向 Rust Core 或通用模型扩散
- 文档中描述代码约束时应落到可执行规则和边界，不写无法指导实现的抽象口号

## 范围偏航判断

- 当前阶段不做什么、近期优先什么，以及哪些复杂度不应提前压入主线，以 `docs/status/current.md`、`docs/mvp/scope.md` 和对应专题文档为准

## Git 提交规范

- 使用简洁明确的 Conventional Commits 风格
- 优先把代码改动和文档改动按主题拆分，而不是混成大提交
- 不添加 AI 协作者署名
- 小修改提交时，commit message 保持一条简洁说明即可
- 大修改提交时，除了首行 commit message 外，优先补充 3~6 条简短说明，概括本次主要变更点
- 提交前至少确认本次改动对应的最小验证已经执行

示例：

```text
docs: 更新了相关进度和协作文档

- 更新了 AGENTS.md 文档
- 为项目协作添加了相关约束规则
- 主要是对齐了项目现状代码与文档的进度
```

```text
ci(ruleset): add repository governance checks
```

```text
chore(PR): establish branch and pr conventions
```

## 文档与开发日志更新要求

- 架构、边界、阶段目标变化时，必须同步更新 `docs/`
- 影响协作方式或工作流的变更，应同步更新当前入口对应的协作文件
- 若该协作规则同时存在于多个 Agent 入口协作文件中，也应一并同步更新
- 每周重要推进应记录到对应周志
- 周志记录应包含：本周目标、完成情况、关键决策、风险与未完成项、下周建议

## 判断标准

如果一个改动同时满足以下条件，则方向通常是正确的：

- 边界更清晰
- workspace 仍稳定可编译
- 文档、代码和正式口径一致
- 没有把尚未进入当前范围的复杂度提前压进主线
