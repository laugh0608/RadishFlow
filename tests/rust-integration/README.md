# Rust Integration Tests

该目录现在作为独立 workspace crate，承载仓库级 Rust 集成测试。

当前主要覆盖：

- 示例 `*.rfproj.json` 的加载
- `rf-solver` 对示例流程的端到端求解
- 求解输出中的关键流股状态断言
- `rf-ui::AppState` 对 `rf-solver::SolveSnapshot` 的仓库级映射
- `radishflow-studio::solver_bridge` 在成功、单元执行失败和缺包失败下的诊断写回与 `RunPanel` 通知基线
- `radishflow-studio::workspace_control` / `run_panel_driver` 对成功运行、包选择阻塞和失败恢复动作的仓库级闭环
