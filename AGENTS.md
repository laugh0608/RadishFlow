# RadishFlow 协作约定

本文件为 RadishFlow 仓库中的 AI 协作者与人工协作者提供统一协作规范。  
它只约束本仓库的工作方式，不代表其他项目，也不复用其他项目的实现边界。

## 语言规范

- 默认使用中文进行讨论、说明、提交总结和开发日志记录
- 代码、命令、路径、配置键、类型名、接口名保留原文
- 新增文档默认使用中文，除非该文件天然要求英文

## 协作流程

- 开始任何任务前，先检查仓库状态，并阅读与当前任务直接相关的文档
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

## 快速认知

- 产品定位：稳态流程模拟软件
- 核心技术栈：Rust Core + Rust UI + `.NET 10` CAPE-OPEN/COM 适配层
- 当前阶段：`M1/M2` 之间，重点是把骨架推进为真正可开发的基础结构
- 当前工作区：只在 `D:\Code\RadishFlow` 内工作
- 外部参考：`CapeOpenCore` 仅作为语义和历史经验参考，不在本仓库协作中跨工作区修改

## 当前阶段产品边界

- Rust 负责模拟核心
- Rust 负责主界面
- `.NET 10` 负责 CAPE-OPEN/COM 适配
- 第一阶段只导出自有 CAPE-OPEN Unit Operation PMC
- 第一阶段不支持加载第三方 CAPE-OPEN 模型
- Rust 不直接处理 COM
- Rust 与 `.NET` 边界只允许句柄、基础数值、UTF-8 字符串和 JSON

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
- `scripts/`: 自动化脚本占位目录

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

- `scripts/` 目录下的 PowerShell 脚本当前仍是占位，不应视为正式验证入口
- `.NET 10` 解决方案当前仍是骨架，不应作为现阶段主验证基线
- 如果某一步改动只涉及文档，仍应至少确认工作区未引入额外脏改动

## 当前实现约定

- 单位统一使用 SI 基本单位
- 温度使用 K
- 压力使用 Pa
- 摩尔流量使用 mol/s
- 第一阶段流股组成统一使用摩尔分率
- 相标签当前只保留 `overall`、`liquid`、`vapor`
- `rf-model` 只承载对象模型，不提前承载求解策略或 COM 语义
- `rf-thermo` 与 `rf-flash` 先稳定接口，再补 Antoine、Raoult 和 Rachford-Rice

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
- 提交前至少确认本次改动对应的最小验证已经执行

示例：

```text
feat: implement binary tp flash solver
docs: refine mvp planning notes
chore: initialize workspace skeleton
```

## 文档与开发日志更新要求

- 架构、边界、阶段目标变化时，必须同步更新 `docs/`
- 影响协作方式或工作流的变更，应同步更新 `AGENTS.md`
- 每周重要推进应记录到对应周志
- 周志记录应包含：本周目标、完成情况、关键决策、风险与未完成项、下周建议

## 当前阶段判断标准

如果一个改动同时满足以下条件，则方向通常是正确的：

- 边界更清晰
- workspace 仍稳定可编译
- 文档、代码、阶段目标三者一致
- 没有把后续阶段复杂度提前压进当前阶段
