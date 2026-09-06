# Flash Golden Tests

该目录用于保存 `rf-flash` 的黄金样例输入与期望输出。

当前样例：

- `binary-hydrocarbon-lite-v1-*.json`
  覆盖当前 three-composition two-phase flash 基线（`z=[0.195, 0.805] / [0.2, 0.8] / [0.23, 0.77]`）与 near-boundary `±ΔP / ±ΔT` 漂移监测。
- `binary-hydrocarbon-synthetic-liquid-only-*.json`
  覆盖 synthetic liquid-only flash 基线与跨 bubble/dew 边界的 near-boundary 漂移监测。
- `binary-hydrocarbon-synthetic-vapor-only-*.json`
  覆盖 synthetic vapor-only flash 基线与跨 bubble/dew 边界的 near-boundary 漂移监测。

## 期望值的解释边界

这些文件用于固定当前简化模型的输出与边界行为；`lite` / `synthetic` 样例均不构成独立物性准确性认证。`official` 的历史称谓表示仓库标准案例，不是数据质量等级。

当前样例缺少完整的独立来源、适用工况、期望值生成方法和物理误差依据；小数位数及测试容差仅服务软件回归。数值来源要求、单元近似与可评估的独立验证见 [热力学 MVP 模型](../../docs/thermo/mvp-model.md#验证证据与缺口)。本次仅补文档，不调整样例值或放宽容差。
