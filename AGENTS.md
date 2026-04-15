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
- 修改规则、架构、接口、目录职责、MVP 范围或协作文档时，优先保持与 `docs/` 中现有正式文档一致
- 每做完一个可分割子步骤，都应进行最小验证；当前阶段的默认验证基线是 `cargo check`
- 重要阶段性决策除了改代码，还应同步更新对应文档；如果属于本周重要推进，追加到周志

## 文档真相源

`docs/` 是本仓库的正式文档源，当前优先级最高的文档如下：

1. `docs/radishflow-architecture-draft.md`
2. `docs/radishflow-startup-checklist.md`
3. `docs/radishflow-mvp-roadmap.md`
4. `docs/radishflow-capeopen-asset-checklist.md`
5. `docs/architecture/overview.md`
6. `docs/mvp/scope.md`
7. `docs/capeopen/boundary.md`
8. `docs/thermo/mvp-model.md`
9. `docs/devlogs/README.md`

规则：

- 若代码与文档冲突，先判断是代码偏离文档，还是文档已过期，再统一修正
- 优先更新已有文档，不为一次性讨论创建大量散文档
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
- 当前阶段：`M1/M2` 之间，重点是把骨架推进为真正可开发的基础结构
- 当前工作区：只在 `D:\Code\RadishFlow` 内工作
- 外部参考：`CapeOpenCore` 仅作为 CAPE-OPEN / COM 语义和历史经验参考；`DWSIM` 仅作为模块拆分、自动化入口和 flowsheet solver 组织经验参考；二者都不在本仓库协作中跨工作区修改，也不直接迁移实现代码

## 当前阶段产品边界

- Rust 负责模拟核心
- Rust 负责主界面
- `.NET 10` 负责 CAPE-OPEN/COM 适配
- 第一阶段只导出自有 CAPE-OPEN Unit Operation PMC
- 第一阶段不支持加载第三方 CAPE-OPEN 模型
- Rust 不直接处理 COM
- Rust 与 `.NET` 边界只允许句柄、基础数值、UTF-8 字符串和 JSON

## 当前阶段优先项

当前阶段先不以主线功能推进为最高优先级，而以“地基建设”类工作为最高优先级：

- 仓库规范
- 代码与文档格式规范
- 分支与 PR 规则
- CI 基线
- App 架构规划
- 设计文档与进度文档完善

只有当这些基础项达到可持续协作标准后，再恢复主线功能推进节奏。

## 当前分支约定

- 当前常态开发分支为 `dev`
- `master` 仅作为稳定主线
- 非特殊情况不直接在 `master` 上开发
- `master` 只通过 Pull Request 合并
- 当前阶段不要求保护 `dev`
- 管理员如需绕过规则，也应通过 PR 合并，而不是直接 push 到 `master`

## 当前推荐开发顺序

按以下顺序推进，不要跳步扩张范围：

1. `rf-types`
2. `rf-model`
3. `rf-thermo`
4. `rf-flash`
5. `rf-unitops`
6. `rf-flowsheet`
7. `rf-solver`
8. `rf-store`
9. `rf-ffi`
10. `.NET 10` 适配层
11. Rust UI 深化

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

### 当前默认不做

- 跨工作区编辑其他项目
- 把旧仓库代码整包迁入当前仓库
- 在 `M4` 前展开复杂 `.NET 10` 运行时实现
- 在 Rust 中直接引入 COM 语义和类型
- 未经明确要求执行破坏性 Git 操作

## 当前验证基线

当前阶段以 Rust workspace 为主，验证入口按以下优先级执行：

1. `cargo check`
2. `cargo test`
3. `cargo fmt --check`
4. `cargo clippy --workspace --all-targets`

补充说明：

- `scripts/check-repo.ps1` 与 `scripts/check-repo.sh` 当前是正式仓库级验证入口，复用同一套 Rust `xtask` 实现
- `.NET 10` 解决方案当前仍是骨架，不应作为现阶段主验证基线
- 如果某一步改动只涉及文档，仍应至少确认工作区未引入额外脏改动
- 仓库治理或 CI 改动优先执行 `pwsh ./scripts/check-repo.ps1`；在 Linux/macOS/CI 中执行 `./scripts/check-repo.sh`
- 如果重要构建、测试或 smoke 验证在沙盒环境中失败，且失败现象明显带有沙盒限制、受限 restore / project reference 解析、native 加载路径差异或其他环境隔离特征，应允许申请提权到真实环境复验，而不是把这类失败直接归因到代码
- 对当前仓库的 `.NET 10` CAPE-OPEN 解决方案、`dotnet build`、`dotnet run --project ... --no-build` smoke 与 `rf-ffi` native 装载路径，若沙盒结果与代码现状明显不符，应优先在获得授权后用真实环境完成最终验证

## 当前实现约定

- 单位统一使用 SI 基本单位
- 温度使用 K
- 压力使用 Pa
- 摩尔流量使用 mol/s
- 第一阶段流股组成统一使用摩尔分率
- 相标签当前只保留 `overall`、`liquid`、`vapor`
- `rf-model` 只承载对象模型，不提前承载求解策略或 COM 语义
- `rf-thermo` 与 `rf-flash` 先稳定接口，再补 Antoine、Raoult 和 Rachford-Rice
- MVP 保持单文档工作区，但源码仍按职责拆分，不以单文件承载全部状态
- 单个源码文件原则上不超过 1000 行；若文件已接近或超过该阈值，后续新增实现应优先拆分职责、提取子模块或测试 helper，而不是继续膨胀原文件
- `src/` 下源码应按职责做浅层目录分组，优先使用 1 层子目录收纳同域模块，避免长期把所有模块平铺在 `src/` 根下，也避免为了“整齐”堆出过深目录树
- 属性编辑采用字段级草稿态，语义提交后才写回文档并形成命令
- 求解控制采用 `SimulationMode(Active/Hold)` 与 `RunStatus` 分离模型
- 求解结果采用独立 `SolveSnapshot`，不直接覆盖 `FlowsheetDocument`
- 每次新增/修改功能、修复 bug 或完成其他任务时，不应优先追求“最小修复方案”，而应优先考虑能否做出完善、稳妥的根治性修改
- 避免连续叠加治标不治本的兜底逻辑；如果问题的根因已可定位，应优先修正根因，而不是无止境地继续包裹一层又一层 fallback

## 常见偏航点

- 不要跳过 `M2/M3` 直接去做 CAPE-OPEN 外壳
- 不要让 UI 设计工作阻塞内核与求解主线
- 不要把第三方 CAPE-OPEN 模型加载需求提前带入第一阶段
- 不要把旧 `.NET Framework`、WinForms、`RegistrationServices` 包袱迁入新仓库
- 不要在尚未建立测试样例前频繁改动热力学和闪蒸接口

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

## 当前阶段判断标准

如果一个改动同时满足以下条件，则方向通常是正确的：

- 边界更清晰
- workspace 仍稳定可编译
- 文档、代码、阶段目标三者一致
- 没有把后续阶段复杂度提前压进当前阶段
