# Rust Integration Tests

该目录现在作为独立 workspace crate，承载仓库级 Rust 集成测试。

当前主要覆盖：

- 示例 `*.rfproj.json` 的加载
- `rf-solver` 对示例流程的端到端求解
- 求解输出中的关键流股状态断言
- `Flash Drum` liquid / vapor outlet 自身 `bubble_dew_window` 边界语义在 raw solver example path、`solver_bridge` 和 `workspace_control` 中的仓库级回归
- source stream 与非 flash 中间流股进入后续 first consumer / flash inlet 时的 `phase_region` / `bubble_dew_window` near-boundary `±ΔP / ±ΔT` 稳定性回归，当前同时覆盖 `binary-hydrocarbon-lite-v1` three-composition two-phase 的 `feed-heater-flash-binary-hydrocarbon` / `feed-cooler-flash-binary-hydrocarbon` / `feed-valve-flash-binary-hydrocarbon` / `feed-mixer-flash-binary-hydrocarbon` 链路，以及 synthetic `liquid-only / vapor-only` 单相样例的 `feed-heater-flash-synthetic-demo` / `feed-cooler-flash-synthetic-demo` / `feed-valve-flash-synthetic-demo` / `feed-mixer-flash-synthetic-demo` 链路；同一批 dedicated near-boundary case 现在也会在 raw solver path、`solver_bridge` 和 `workspace_control` 中继续锁定 source/intermediate stream 与 downstream consumed stream 的同一份 DTO 语义，以及 `Flash Drum` outlet 的零流量窗口缺席语义和 two-phase split 时 liquid/vapor outlet 的饱和边界语义
- `rf-ui::AppState` 对 `rf-solver::SolveSnapshot` 的仓库级映射
- `radishflow-studio::solver_bridge` 在成功、单元执行失败和缺包失败下的诊断写回与 `RunPanel` 通知基线
- `radishflow-studio::workspace_control` / `run_panel_driver` 对成功运行、包选择阻塞、near-boundary workspace run 和失败恢复动作的仓库级闭环
