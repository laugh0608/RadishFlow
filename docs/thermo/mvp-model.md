# Thermo MVP Model

该目录用于沉淀第一阶段热力学模型范围与样例数据。

## 当前最小边界

为配合 `M1/M2` 的基础结构建设，当前代码中已经冻结以下最小职责边界：

- `rf-thermo` 负责纯组分热力学数据结构和热力学接口，不负责 COM、UI 或 flowsheet 连接逻辑。
- `rf-flash` 负责 `TP Flash` 的输入输出契约和求解器接口，不直接持有 CAPE-OPEN 适配语义。
- `rf-model` 负责流股和相态结果对象，供 `rf-flash` 产出和后续 unit operation 复用。

## 已落地的数据契约

当前最小 API 以“先建边界、后补算法”为原则，已包含：

- `ThermoComponent`
- `AntoineCoefficients`
- `ThermoSystem`
- `ThermoProvider`
- `TpFlashInput`
- `TpFlashResult`
- `TpFlashSolver`

## 当前刻意未实现的内容

为了避免在第一轮就把范围扩散到完整热力学求解，以下内容仍保持为下一步任务：

- Antoine 饱和蒸气压计算
- Raoult 定律 `K` 值估算
- Rachford-Rice 求解
- 真正的汽液两相 `TP Flash`
- 焓模型的数值实现

## 下一步建议

下一步应在现有接口之上补真正的二元体系计算能力，优先顺序保持为：

1. Antoine 饱和蒸气压
2. 理想体系 `K` 值估算
3. Rachford-Rice
4. `TP Flash` 相分率和相组成
