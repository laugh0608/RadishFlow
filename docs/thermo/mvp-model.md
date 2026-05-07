# Thermo MVP Model

更新时间：2026-05-07

该目录用于沉淀第一阶段热力学模型范围与样例数据。

## 当前最小边界

为配合 `M1/M2` 的基础结构建设，当前代码中已经冻结以下最小职责边界：

- `rf-thermo` 负责纯组分热力学数据结构和热力学接口，不负责 COM、UI 或 flowsheet 连接逻辑。
- `rf-flash` 负责 `TP Flash` 的输入输出契约和求解器接口，不直接持有 CAPE-OPEN 适配语义。
- `rf-model` 负责流股和相态结果对象，供 `rf-flash` 产出和后续 unit operation 复用。

补充冻结以下分层约束：

- `rf-thermo` 只负责纯计算和 provider 接口，不直接承担文件系统缓存、授权缓存索引或网络下载逻辑。
- 本地物性包缓存、缓存索引和路径布局属于 `rf-store` 职责，不反向污染热力学接口。
- 控制面交互、授权编排和派生包下载属于桌面应用层职责，不直接进入 `rf-thermo` / `rf-flash`。

## 已落地的数据契约

当前最小 API 已包含：

- `ThermoComponent`
- `AntoineCoefficients`
- `ThermoSystem`
- `ThermoProvider`
- `TpFlashInput`
- `TpFlashResult`
- `TpFlashSolver`

## 当前已实现内容

- `rf-thermo` 已实现基于 Antoine 相关式的饱和蒸气压计算
- `rf-thermo` 已实现基于理想体系假设的 `K` 值估算
- `rf-thermo` 已实现固定温度下的 bubble/dew pressure 边界估算
- `rf-thermo` 已实现 fixed-pressure 的 bubble/dew temperature 边界估算
- `rf-thermo` 已实现基于 property package 中 liquid/vapor 常热容的 MVP 相 molar enthalpy，参考温度固定为 `298.15 K`
- `rf-flash` 已实现 Rachford-Rice 求解
- `rf-flash` 已实现最小二元汽液两相 `TP Flash`
- `rf-flash` 当前已可产出带 `overall` / `liquid` / `vapor` 相态结果的 `MaterialStreamState`，并把 liquid/vapor 与按相分率加权的 overall molar enthalpy 写入相态结果
- `rf-flash` 当前会在 `TP Flash` 结果中显式携带 `liquid-only / two-phase / vapor-only` phase region 与 bubble/dew pressure / temperature
- `rf-model::MaterialStreamState` 当前已为相平衡边界正式携带结构化 `bubble_dew_window`
- `Flash Drum` liquid / vapor outlet、`rf-solver::SolveSnapshot` 与 `rf-ui::SolveSnapshot` 当前已能透传这组窗口；其中 flash outlet 会按各自 outlet composition 重算窗口，而不是直接复用 overall flash feed 的边界
- `rf-thermo` 当前要求传入热力学状态和相态焓计算的 mole fractions 在有限、非负之外必须归一到 1；`rf-flash` 直接调用入口会继承该契约，unit operation 层仍负责先把文档流股组成归一化后再调用 flash

## 当前刻意未实现的内容

为了避免在第一轮就把范围扩散到完整热力学求解，以下内容仍保持为后续任务：

- 完整焓参考态、相变潜热与更真实物性模型
- `PH Flash`
- `PS Flash`
- 更完整 phase envelope tracing
- 多物性模型切换与更复杂 EOS
- 超出当前 MVP 的复杂多组分与更大数据库能力

## 下一步建议

当前数值主线已经从“补第一版算法”切换为“围绕已实现算法建立更稳定的闭环与回归基线”，优先顺序建议保持为：

1. 继续补更稳定的黄金样例与边界条件测试
2. 让 Result Inspector / Active Inspector 开始只读消费已结构化的 `bubble_dew_window`
3. 在 `Flash Drum` 之外，评估是否把同一组窗口扩到更多中间流股结果
4. 待 MVP 闭环更稳后，再评估更真实 EOS 或更复杂物性模型

## 测试样例要求

为避免数值接口在后续迭代中漂移，当前阶段应持续维护以下最小验证材料：

- `tests/thermo-golden` 中的热力学黄金样例
- `tests/flash-golden` 中的 `TP Flash` 黄金样例
- bubble/dew pressure / temperature 边界与 phase region 的 focused tests
- 与 flowsheet 闭环样例联动的端到端回归样例
- 未归一、非有限、负组成等输入契约边界测试，避免数值 API 静默接受无效 mole fractions
- 黄金样例进入版本控制，数值变更应能够被回归测试直接发现
