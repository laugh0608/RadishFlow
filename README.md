# RadishFlow

RadishFlow 是一个以 Rust 为核心、以 Rust UI 为主界面、以 .NET 10 负责 CAPE-OPEN/COM 适配的稳态流程模拟软件。

## 项目定位

第一阶段只追求最小可运行闭环：

- Rust 实现稳态模拟核心
- Rust 实现桌面 UI
- .NET 10 暴露自有 CAPE-OPEN Unit Operation PMC
- 外部 PME 能识别并调用至少一个自有模型

## 当前状态

截至 2026-03-29，仓库已经完成以下初始化工作：

- Rust workspace 骨架已建立并可 `cargo check`
- 第一批基础 crate 已从空壳推进到可继续开发的边界结构
- `.NET 10` 适配层目录与解决方案骨架已初始化
- MVP 边界、迁移边界和协作约定已在 `docs/` 与 `AGENTS.md` 中冻结
- 仓库治理、PR 检查、文本格式约束和基础协作规则正在补齐

当前仍未开始的内容：

- 真正的二元 `TP Flash` 数值实现
- 单元模块与 flowsheet 求解闭环
- Rust FFI 与 `.NET 10` 运行时联通
- 外部 PME 冒烟验证

## 当前阶段优先项

现阶段优先项暂时从“推进功能主线”切换为“先夯地基”：

- 完善仓库规范与协作约定
- 建立分支、PR 和 CI 规则
- 完善代码与文档格式约束
- 完善 App 架构规划与功能边界文档
- 建立稳定的阶段目标、进度记录和设计口径

## 当前分支策略

- `dev` 是当前常态开发分支
- `master` 是稳定主线，只接受 PR 合并
- 当前阶段只要求保护 `master`
- 管理员可通过 PR 方式绕过 `master` 规则，但不应直接 push 到 `master`
## 快速开始

本仓库当前以 Rust workspace 为主工作入口：

```powershell
cargo check
```

`.NET 10` 适配层目前仍处于目录与职责冻结阶段，当前阶段不作为主开发入口。

## 仓库结构

- `apps/radishflow-studio/`: Rust 桌面应用
- `crates/`: Rust 核心、UI、求解与 FFI crates
- `adapters/dotnet-capeopen/`: .NET 10 CAPE-OPEN/COM 适配层
- `docs/`: 架构、MVP、边界、周志与迁移文档
- `examples/`: 示例流程与 PME 验证样例
- `tests/`: 数值回归与互操作测试
- `assets/`: 图标、主题与示例数据占位目录

## 文档入口

- `docs/README.md`: 文档总索引
- `docs/architecture/overview.md`: 当前仓库分层与模块边界
- `docs/architecture/app-architecture.md`: 桌面 App 架构规划
- `docs/mvp/scope.md`: MVP 范围、非目标与近期开发节奏
- `docs/thermo/mvp-model.md`: 热力学与闪蒸的当前契约
- `docs/capeopen/boundary.md`: Rust 与 .NET 10 的 CAPE-OPEN 边界
- `docs/adr/0001-branch-and-pr-governance.md`: 分支与 PR 治理策略
- `.github/rulesets/master-protection.json`: `master` 分支保护规则模板
- `docs/devlogs/README.md`: 周志规范与索引

## 协作入口

- `AGENTS.md`: 仓库协作约定、阶段边界与工作流
- `docs/devlogs/2026-W13.md`: 当前阶段首份开发日志

## 参考仓库

- [CapeOpenCore](https://github.com/laugh0608/CapeOpenCore): 当前阶段用于提取 CAPE-OPEN 接口、GUID、异常语义与注册语义的参考仓库。

