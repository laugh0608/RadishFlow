# Units And Conventions

更新时间：2026-05-10

## 目的

本文档用于收口当前仓库中最常用、最稳定的物理单位、字段后缀和表示约定。

它回答的是：

- 物理量默认用什么单位
- 流股组成怎么表示
- 相标签和相区间怎么表示
- 常见字段名中的后缀各自是什么意思

它不替代更完整的对象格式说明；项目文件和结果 DTO 的详细字段见独立 reference 文档。

## 物理单位

当前 MVP 默认统一使用 SI 基本单位或与之直接兼容的工程单位。

| 量 | 当前单位 | 典型字段 |
| --- | --- | --- |
| 温度 | `K` | `temperature_k` |
| 压力 | `Pa` | `pressure_pa` |
| 总摩尔流量 | `mol/s` | `total_molar_flow_mol_s` |
| 摩尔焓 | `J/mol` | `molar_enthalpy_j_per_mol` |
| 摩尔热容 | `J/(mol*K)` | `liquid_heat_capacity_j_per_mol_k`、`vapor_heat_capacity_j_per_mol_k` |
| 泡点压力 | `Pa` | `bubble_pressure_pa` |
| 露点压力 | `Pa` | `dew_pressure_pa` |
| 泡点温度 | `K` | `bubble_temperature_k` |
| 露点温度 | `K` | `dew_temperature_k` |

补充约束：

- 压力默认按绝压理解
- 温度默认按绝对温标理解
- 当前文档、代码和测试都不应混入 `degC`、`bar`、`kmol/h` 之类第二套默认口径

## 流股组成约定

当前流股组成统一表示为摩尔分率：

- 字段名：`overall_mole_fractions`
- 语义：各组分在整体流股中的摩尔分率
- 当前阶段不引入质量分率或体积分率切换

当前稳定规则：

- 分率必须有限
- 分率必须非负
- 分率和必须归一到 `1`

对不同层的约束再补充一条：

- `rf-thermo` / `rf-flash` 的直接数值 API 会拒绝未归一组成
- UI 草稿态允许用户先编辑，再显式归一或提交
- 运行前若文档流股总体组成未归一，Studio 当前会阻塞运行

## 相标签约定

当前阶段只保留三类正式相标签：

- `overall`
- `liquid`
- `vapor`

它们分别用于：

- `overall`：整体流股或整体结果摘要
- `liquid`：液相结果
- `vapor`：气相结果

当前不引入：

- 多液相标签
- 固相标签
- 自定义相名体系

## 相区间约定

当前 `phase_region` 使用以下稳定词汇：

- `liquid-only`
- `two-phase`
- `vapor-only`

它表示的是在当前 `T / P / z` 条件下，体系落在的最小相区间判断。

与之配套的结构化窗口为：

- `bubble_dew_window`

当前窗口内至少包含：

- `phase_region`
- `bubble_pressure_pa`
- `dew_pressure_pa`
- `bubble_temperature_k`
- `dew_temperature_k`

当前 UI 和结果审阅层应继续只读消费这些 DTO，不在 shell 或展示层再分叉第二套相平衡判断。

再补三条稳定约束：

- `phase_region` 属于窗口语义，不直接等价于最终会显示几行 phase rows
- zero-flow 的单相 flash 对侧 outlet 当前允许 `bubble_dew_window` 缺席
- 结果消费层看到缺席窗口时应按缺席处理，不补造默认边界

## 结果字段读取约定

当前结果区里最容易混淆的是 `H`、`phase rows` 和 `bubble_dew_window`。

稳定规则：

- `H` 只表示已物化的 `molar_enthalpy_j_per_mol`
- 若某股流没有 `H`，当前 consumer 应按缺失处理，不在 shell 中补算
- `bubble_dew_window` 表示当前 overall composition 的平衡边界窗口
- source stream、非 flash 中间流股、flash outlet，以及 step 输入/输出都应继续沿同一份结果 DTO 被消费

更完整的结果 DTO 语义见：

- `docs/reference/solve-snapshot-results.md`

## 字段后缀约定

当前仓库中的字段命名会尽量把单位直接写进后缀。

常见后缀如下：

| 后缀 | 含义 | 示例 |
| --- | --- | --- |
| `_k` | Kelvin 温度 | `temperature_k` |
| `_pa` | Pascal 压力 | `pressure_pa` |
| `_mol_s` | 每秒摩尔流量 | `total_molar_flow_mol_s` |
| `_j_per_mol` | 摩尔量上的能量 | `molar_enthalpy_j_per_mol` |
| `_j_per_mol_k` | 摩尔热容 | `liquid_heat_capacity_j_per_mol_k` |

规则：

- 字段名应优先表达真实物理量和单位
- 不应把单位只留在注释或调用约定里
- 新增字段若属于稳定边界，优先沿用这一套后缀风格

## 当前 MVP 默认假设

以下约定当前已经进入正式口径：

- 温度单位：`K`
- 压力单位：`Pa`
- 流量单位：`mol/s`
- 组成表示：摩尔分率
- 相标签：`overall / liquid / vapor`
- 焓值参考温度：当前 MVP 基线固定为 `298.15 K`

最后一条尤其重要：当前热力学/闪蒸的焓值能力仍是 MVP 显热基线，不应被误读为已经实现完整参考态、潜热和更真实物性模型。

## 相关文档

- `docs/guides/studio-quick-start.md`
- `docs/guides/run-first-flowsheet.md`
- `docs/guides/review-solve-results.md`
- `docs/reference/solve-snapshot-results.md`
- `docs/thermo/mvp-model.md`
- `docs/mvp/scope.md`
