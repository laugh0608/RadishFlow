# Rust Integration Tests

该目录现在作为独立 workspace crate，承载仓库级 Rust 集成测试。

当前主要覆盖：

- 示例 `*.rfproj.json` 的加载
- `rf-solver` 对示例流程的端到端求解
- 求解输出中的关键流股状态断言
