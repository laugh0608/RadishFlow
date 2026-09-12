# Thermo MVP Model

更新时间：2026-09-12

## 用途与维护状态

用途：说明已有热力学与闪蒸的公式、简化假设、样例解释和验证证据边界。
读者：维护计算内核、物性样例、单元或结果消费链路的开发者，以及需要判断结果含义的使用者。
不包含：完整物性数据库、工业工况准确性承诺、详细 UI 交互和新增功能排期。

本文描述已有模型与未验证项；新增数值能力须明确适用工况与独立验证依据，当前迭代优先级见 [当前状态](../status/current.md)。

## 职责与实际实现

- `rf-thermo` 定义 `ThermoComponent / ThermoSystem / ThermoProvider`，计算关联式、K 值、相焓和泡露点。
- `rf-flash` 定义 `TpFlashInput / TpFlashResult / TpFlashSolver`，执行 TP Flash 与相组成求解。
- `rf-model` 承载 `MaterialStreamState / PhaseState / BubbleDewWindow`，供单元、solver 和 UI 传递。
- `rf-types` 是相区 pressure / temperature tolerance 的真相源；flash 复用该容差调和边界附近的分类差异，远离边界的真实冲突仍报错。

目标上计算与 provider 接口不承担文件缓存、授权索引或网络编排。当前 `rf-thermo::CachedPropertyPackageProvider` 实际直接依赖 `rf-store` 并执行缓存装载与过期过滤，尚不满足完整的纯计算隔离；详见 [架构总览](../architecture/overview.md#core-与持久化)。本次不移动代码，也不把既有偏移改写成新的目标边界。

现有具体实现仍命名为 `PlaceholderThermoProvider` 与 `PlaceholderTpFlashSolver`，但已执行以下算法；名称既不表示完全未实现，也不表示通用物性实现已经完成。

## 已实现公式与输入约定

| 部分 | 当前实现 | 解释限制 |
| --- | --- | --- |
| Antoine | `ln(P_sat / kPa) = A - B / (T[K] + C)` | 系数必须匹配该对数底与单位，不能直接套用其他单位制的 A/B/C |
| 理想 K 值 | `K_i = P_sat_i / P` | 理想溶液 / 理想气相近似；无逸度系数、活度系数和真实 EOS |
| TP Flash | Rachford-Rice、相区判别、相分率和相组成 | 限当前 MVP 样例假设 |
| 泡露点 | 定温 bubble/dew pressure，定压 bubble/dew temperature | 由同一关联式估算，不是完整 phase envelope tracing |
| 相摩尔焓 | `h_phase = sum_i(x_i,phase * Cp_i,phase) * (T - 298.15 K)` | 常热容显热近似，未建立完整相间参考态与相变潜热 |
| overall 摩尔焓 | liquid / vapor 相焓按 flash 相分率加权 | 是该简化模型的混合结果，不表示能量平衡已求解 |

表中 `x_i,phase` 是传入该相焓计算的组分摩尔分率。温度、压力和流量使用 K、Pa、mol/s；组成使用摩尔分率。直接 thermo / flash 数值 API 要求组成有限、非负并归一；TP Flash 总摩尔流量必须有限且非负，拒绝 NaN 和正负无穷，零流量保持允许；单元层会先归一化文档组成，Studio 运行 readiness 另行检查文档输入。

[AntoineCoefficients](../../crates/rf-thermo/src/lib.rs) 当前只保存 A/B/C，没有适用温区、临界性质或数据引用字段。数值有限、压力为正及求解成功，均不能替代关联式适用范围判断。

## 单元近似与能量解释

| 单元 | 当前行为 | 未完成的物理含义 |
| --- | --- | --- |
| Feed | 从已提交源流股物化状态与整体焓 | 不提供独立物性准确性证明 |
| Mixer | 合并流量和组成，按摩尔流量加权入口温度，限制出口压力 | 未通过总焓平衡求出口温度；热容、组成或相态变化时不保证绝热能量守恒 |
| Heater / Cooler | 使用指定出口 T/P，继承入口流量与组成，并物化焓与窗口 | 无指定热负荷反算温度的 PH 路径，不构成完整换热器模型 |
| Valve | 保持入口温度并降低压力，继承流量与组成 | 不等同于绝热等焓节流，未求解 PH Flash |
| Flash Drum | 在指定 T/P 下进行汽液分配，形成 liquid / vapor outlet | 固定 TP 分离不自动证明设备绝热能量平衡 |

当前相焓在 `298.15 K` 的液相与气相都回到零点，没有用相间参考焓差表达真实汽化潜热。因此 H 适合验证 DTO、单位和当前算法一致性，不能直接作为真实相变热负荷依据。

Mixer 的区别可对照 [IDAES Mixer 模型](https://idaes-pse.readthedocs.io/en/stable/reference_guides/model_libraries/generic/unit_models/mixer.html)：其能量混合使用总焓平衡。该引用用于说明物理模型差异，不表示本仓库采用该实现或与其完成对标。

## 样例身份与适用范围

`binary-hydrocarbon-lite-v1`、其 methane / ethane 组分名以及历史文档中的 `official hydrocarbon`，标识的是仓库标准演示与回归路径，不代表独立审核的真实物性数据。当前没有从样例追溯到独立参考输出、适用温区和工程误差的完整证据链；这些结果应按演示 / 合成模型解释。

例如 [300 K 黄金样例](../../tests/thermo-golden/binary-hydrocarbon-lite-v1-300k-650kpa.json) 把 methane 的 Antoine 计算结果记为饱和蒸气压。NIST 给出的甲烷临界温度约为 `190.6 K`，其 Antoine 数据也明确列出低于约 `190.5 K` 的适用温区；该样例在 `300 K` 的计算值不能作为真实甲烷纯组分汽液饱和蒸气压。这个判断针对纯组分关联式，不推导为含超临界组分的混合物一定不能形成两相。[NIST Chemistry WebBook: Methane](https://webbook.nist.gov/cgi/cbook.cgi?ID=C74828&Mask=4)

`water-ethanol-lite-v1` 的 PME 人工记录用于验证宿主识别、材料读写和调用闭环，也不构成独立物性准确性背书。

以后如获准引入真实数据，应先明确关联式形式与单位、组分身份、来源版本、适用温压区间、参考态、独立期望值和误差依据。当前 DTO 未实现这些元数据，本次不更改数据格式、样例名称或结果值。

## 结果传递契约

- TP Flash 输出携带 `phase_region` 与 `bubble_dew_window`，并物化 liquid / vapor 与 overall 摩尔焓。
- Flash Drum outlet 按各自组成重算边界窗口，不直接复用 feed 的窗口。
- Feed、Mixer、Heater / Cooler、Valve 的整体焓通过同一 TP Flash 路径物化；后续 consumer step 复用已物化流股。
- `rf-solver -> rf-ui -> Studio` 传递同一份 T/P/z/H/窗口结果；Result Inspector、Active Inspector 和导出不得分叉另一套数值求值逻辑。
- `Converged` 表示当前算法与执行契约成功，不包含物性适用域认证、能量守恒认证或工程精度等级。

结果状态、revision 与相缺席语义见 [SolveSnapshot 参考](../reference/solve-snapshot-results.md)。

## 验证证据与缺口

| 证据 | 当前已有内容 | 能证明的范围 |
| --- | --- | --- |
| 算法回归 | thermo / flash golden，三组 two-phase 组成，synthetic 单相和 exact / near-boundary ±ΔP / ±ΔT | 实现是否偏离已保存的期望值 |
| 输入契约 | 未归一、非有限、负组成等 focused tests | API 输入拒绝与容差行为 |
| 跨层一致性 | Feed、Heater / Cooler / Valve / Mixer 到 Flash 的集成和 workspace 路径 | 已物化状态是否一致传入 solver step 与 UI |
| 独立物理基准 | 当前样例未提供完整独立来源、适用工况和误差证据 | 不能从前三类测试通过推导工程准确性 |

golden 中的小数位数和数值容差属于实现回归条件，不是物性数据不确定度；大量边界变体也不能代替独立基准。

若未来获准推进数值验证，可评估以下缺口，不把它们记为已通过或当前新增门禁：

- 独立参考数据或独立实现的对照，记录同一模型、参考态、单位与容差依据。
- 组分与总物料守恒；完成能量模型后再验证相应单元的焓平衡残差。
- 组分排列不变性、流量缩放、近纯组分、零流量和适用温区边界。
- 将同一数值案例的跨层透传测试与物理准确性测试分开组织，避免复制同源期望值造成独立验证的假象。

## 尚未实现与后续决策

完整焓参考态、潜热、PH / PS Flash、真实 EOS、更复杂多组分与 phase envelope tracing 仍未实现。后续迭代按目标体系和工况选择模型，优先建立物性来源与能量闭环，再扩展 UI 或远期能力；具体顺序见 [迭代路线图](../radishflow-mvp-roadmap.md#后续迭代顺序)。

## 相关样例

- [Thermo golden](../../tests/thermo-golden/README.md)
- [Flash golden](../../tests/flash-golden/README.md)
- [本地物性样例](../../examples/sample-components/README.md)
- [单位与字段约定](../reference/units-and-conventions.md)
