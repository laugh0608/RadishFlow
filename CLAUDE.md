# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 提供在 RadishFlow 仓库中工作时的指导。

## 项目定位

RadishFlow 是一个以 Rust 为核心、以 Rust UI 为主界面、以 .NET 10 负责 CAPE-OPEN/COM 适配的稳态流程模拟软件。当前处于 M1/M2 过渡阶段，优先建设仓库地基而非推进功能主线。

语言规范：

- 默认使用中文进行讨论、说明、提交总结和开发日志记录
- 代码、命令、路径、配置键、类型名、接口名保留原文

外部参考：

- `CapeOpenCore` 仅作为 CAPE-OPEN / COM 语义和历史经验参考
- `DWSIM` 仅作为模块拆分、自动化入口和 flowsheet solver 组织经验参考
- 两者都不在本仓库协作中跨工作区修改，也不直接迁移实现代码

## 常用命令

### 基础验证

```bash
# 最基本的验证，每次修改后至少执行
$env:RUST_BACKTRACE=1; cargo check

# 格式化检查
cargo fmt --all --check

# 运行测试
cargo test --workspace

# 运行 clippy
cargo clippy --workspace --all-targets -- -D warnings
```

### 仓库级验证

Windows 环境：

```powershell
pwsh ./scripts/check-repo.ps1
```

Linux/macOS 环境：

```bash
./scripts/check-repo.sh
```

### xtask 验证入口

```bash
# 检查文本文件（UTF-8、LF 换行、无 BOM）
cargo run -p xtask -- check-text-files

# 验证工作空间（fmt、check、test、clippy）
cargo run -p xtask -- validate-workspace

# 完整仓库检查
cargo run -p xtask -- check-repo

# 跳过 clippy（更快）
cargo run -p xtask -- check-repo --skip-clippy

# 跳过文本文件检查
cargo run -p xtask -- check-repo --skip-text-files
```

### 单测试执行

```bash
# 运行特定测试
cargo test --workspace -- <test_name>

# 运行特定 crate 的测试
cargo test -p rf-thermo
```

## 代码架构

### Workspace 结构

**核心求解 crates**（按此顺序实现）：

- `crates/rf-types`：基础 ID、枚举、错误类型
- `crates/rf-model`：组分、流股、相态、单元、流程图对象模型
- `crates/rf-thermo`：热力学数据结构与接口
- `crates/rf-flash`：TP Flash 输入输出契约与求解器接口
- `crates/rf-unitops`：单元模块行为（Feed、Mixer、Heater/Cooler、Valve、Flash Drum）
- `crates/rf-flowsheet`：图结构与连接校验
- `crates/rf-solver`：顺序模块法求解器

**存储与 FFI**：

- `crates/rf-store`：JSON 存储、授权缓存索引、物性包缓存
- `crates/rf-ffi`：Rust 与 .NET 的 C ABI 边界

**UI crates**：

- `crates/rf-ui`：UI 状态（AppState）、授权态、求解态、控制面 DTO
- `crates/rf-canvas`：流程图画布（占位）

**应用程序**：

- `apps/radishflow-studio`：桌面应用入口，含 auth cache sync、控制面 HTTP client、entitlement/lease/offline refresh 编排

**适配层**：

- `adapters/dotnet-capeopen/`：.NET 10 CAPE-OPEN/COM 适配层（当前仅占位）

### 关键架构边界

1. **Rust 不直接处理 COM** —— 所有 CAPE-OPEN/COM 适配放在 .NET 10 中
2. **第一阶段只导出自有 Unit Operation PMC** —— 不支持加载第三方 CAPE-OPEN 模型
3. **Rust 与 .NET 边界** —— 只允许句柄、基础数值、UTF-8 字符串和 JSON
4. **SI 基本单位**：温度 K、压力 Pa、摩尔流量 mol/s、流股组成摩尔分率
5. **相标签**：仅保留 `overall`、`liquid`、`vapor`
6. **单文档工作区** —— 源码按职责拆分，但 UI 工作区为单文档

### App 状态架构

- **属性编辑采用字段级草稿态** —— 语义提交后才写回文档
- **求解控制采用 `SimulationMode` (Active/Hold) 与 `RunStatus` 分离模型**
- **求解结果采用独立 `SolveSnapshot`** —— 不直接覆盖 `FlowsheetDocument`
- **`DocumentMetadata`** 只保存文档身份与序列化元信息
- **`UserPreferences`** 只保存应用级偏好与快照窗口策略
- **`CommandHistory`** 只记录语义化文档命令
- **`SolveSessionState`** 绑定当前观察的文档修订号，`SolveSnapshot` 由工作区持有有界历史窗口

## 开发流程

### 开始任务前

1. 检查仓库状态
2. 阅读与当前任务直接相关的 `docs/` 文档
3. 若任务未明确定义或影响架构，先说明方案并等待批准

### 当前阶段优先项

当前阶段先不以主线功能推进为最高优先级，而以"地基建设"类工作为最高优先级：

- 仓库规范与协作约定
- 代码与文档格式约束
- 分支、PR 和 CI 规则
- App 架构规划
- 设计文档与进度文档完善

### 推荐开发顺序

按以下顺序推进，不要跳步：

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

### 代码质量

- 每做完一个可分割子步骤，进行最小验证；默认基线是 `cargo check`
- 仓库治理或 CI 改动优先执行 `pwsh ./scripts/check-repo.ps1`
- 优先修正根因，而不是无止境地包裹 fallback 逻辑

### 实现约定

- 单位统一使用 SI 基本单位
- 温度使用 K，压力使用 Pa，摩尔流量使用 mol/s
- 第一阶段流股组成统一使用摩尔分率
- 相标签当前只保留 `overall`、`liquid`、`vapor`
- `rf-model` 只承载对象模型，不提前承载求解策略或 COM 语义
- `rf-thermo` 与 `rf-flash` 先稳定接口，再补 Antoine、Raoult 和 Rachford-Rice
- MVP 保持单文档工作区，但源码仍按职责拆分，不以单文件承载全部状态
- 单个源码文件原则上不超过 1000 行；若文件已接近或超过该阈值，后续新增实现应优先拆分职责、提取子模块或测试 helper，而不是继续膨胀原文件
- `src/` 下源码应按职责做浅层目录分组，优先使用 1 层子目录收纳同域模块，避免长期把所有模块平铺在 `src/` 根下，也避免为了“整齐”堆出过深目录树

### Git 提交规范

使用简洁明确的 Conventional Commits 风格，示例：

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

规则：

- 优先把代码改动和文档改动按主题拆分
- 不添加 AI 协作者署名
- 小修改保持一条简洁说明
- 大修改补充 3~6 条简短说明概括主要变更点
- 提交前至少确认最小验证已执行

### 文档更新要求

- 架构、边界、阶段目标变化时，同步更新 `docs/`
- 影响协作方式或工作流的变更，同步更新当前入口对应的协作文件
- 若该协作规则同时存在于多个 Agent 入口协作文件中，也应一并同步更新
- 每周重要推进记录到 `docs/devlogs/YYYY-Www.md`
- 周志包含：本周目标、完成情况、关键决策、风险与未完成项、下周建议

### 文本文件约束

所有文本文件必须：

- UTF-8 编码（无 BOM）
- LF 换行（无 CRLF）
- 末尾带换行符

由仓库检查脚本强制执行。

## 分支与 PR

- 当前常态开发分支为 `dev`
- `master` 仅作为稳定主线，只通过 Pull Request 合并
- PR 检查执行 `Repo Hygiene`（文本文件）和 `Rust Baseline`（fmt、check、test、clippy）
- 非特殊情况不直接在 `master` 上开发

## 执行边界

### 可直接执行

- 代码读取、代码修改、文档修改
- `cargo check`、`cargo test`、`cargo fmt --check`
- `cargo clippy --workspace --all-targets`
- 只读 Git 操作（`git status`、`git diff`、`git log`）
- 简洁明确的提交操作

### 需要先告知用户再执行

- 长时间运行或需要人工交互的命令（`cargo run`、桌面 UI 启动）
- 可能修改本机环境的命令（COM 注册、证书安装、写注册表）
- 依赖网络或可能引入依赖变更的命令（`cargo add`、`cargo update`、`dotnet add package`）
- 打包、发布、安装类命令

### 当前默认不做

- 跨工作区编辑其他项目
- 把旧仓库代码整包迁入当前仓库
- 在 M4 前展开复杂 .NET 10 运行时实现
- 在 Rust 中直接引入 COM 语义和类型
- 未经明确要求执行破坏性 Git 操作

## 文档真相源

`docs/` 是正式文档源，优先级最高：

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

- 若代码与文档冲突，先判断是代码偏离还是文档过期，再统一修正
- 优先更新已有文档，不为一次性讨论创建大量散文档
- 许可条款以仓库根 `LICENSE` 文件为准

## Agent 协同文件

- 仓库中面向不同 Agent 入口名的协作文件，应保持“基本复制”和长期同步
- 这些同类协作文件不应演化出彼此冲突的规则口径
- 若某个同类协作文件更新了通用协作规则、执行边界、验证基线或阶段约束，其余同类文件也应尽快同步
- 同类协作文件只允许保留极少量与入口名称直接相关的表述差异，不应借此分叉实际协作规范
