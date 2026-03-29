# RadishFlow

RadishFlow 是一个以 Rust 为核心、以 Rust UI 为主界面、以 .NET 10 负责 CAPE-OPEN/COM 适配的稳态流程模拟软件。

## 当前目标

第一阶段只追求最小可运行闭环：

- Rust 实现稳态模拟核心
- Rust 实现桌面 UI
- .NET 10 暴露自有 CAPE-OPEN Unit Operation PMC
- 外部 PME 能识别并调用至少一个自有模型

## 仓库结构

- `apps/radishflow-studio/`: Rust 桌面应用
- `crates/`: Rust 核心、UI、求解与 FFI
- `adapters/dotnet-capeopen/`: .NET 10 CAPE-OPEN/COM 适配层
- `docs/`: 架构、MVP 和迁移文档
- `examples/`: 示例流程与 PME 验证样例
- `tests/`: 数值回归与互操作测试

## 参考文档

- `docs/radishflow-architecture-draft.md`
- `docs/radishflow-startup-checklist.md`
- `docs/radishflow-mvp-roadmap.md`
- `docs/radishflow-capeopen-asset-checklist.md`

## 参考仓库

- [CapeOpenCore](https://github.com/laugh0608/CapeOpenCore): 当前阶段用于提取 CAPE-OPEN 接口、GUID、异常语义与注册语义的参考仓库。

