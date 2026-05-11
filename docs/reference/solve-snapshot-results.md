# Solve Snapshot Results Reference

更新时间：2026-05-10

## 目的

本文档用于说明当前 `rf-ui::SolveSnapshot` 及其相关结果 DTO 的稳定语义。

它回答的是：

- `SolveSnapshot.streams` 和 `StepSnapshot` 里的流股各自代表什么
- source stream、非 flash 中间流股、flash outlet、step 输入/输出的正式边界是什么
- 结果消费层不应该做哪些“二次组装”

它不是运行指南，也不展开 UI 操作步骤。

## 结果对象总览

当前与 Studio 结果审阅直接相关的主要对象是：

- `SolveSnapshot`
- `StreamStateSnapshot`
- `StepSnapshot`
- `BubbleDewWindowSnapshot`
- `PhaseStateSnapshot`

当前 `StepSnapshot` 至少包含：

- `index`
- `unit_id`
- `summary`
- `execution`
- `consumed_streams`
- `streams`

## `SolveSnapshot` 的稳定语义

`SolveSnapshot` 表示“某个文档修订号上的一次不可变求解结果”。

当前稳定边界：

- `streams` 表示这次求解后已物化的全局流股结果集合
- `steps` 表示按执行顺序记录的单元求解步骤
- consumer 应把它当成正式结果真相源，而不是中间缓存

## `StreamStateSnapshot` 的稳定语义

单条 `StreamStateSnapshot` 当前稳定包含：

- `stream_id`
- `label`
- `temperature_k`
- `pressure_pa`
- `total_molar_flow_mol_s`
- `overall_mole_fractions`
- `phases`
- `bubble_dew_window`

其中：

- `phases` 表示当前已物化的相结果
- `bubble_dew_window` 表示当前 overall composition 在给定 `T / P` 下的平衡边界窗口

如果 `bubble_dew_window` 缺席，consumer 应按“当前没有窗口 DTO”处理，而不是猜测默认边界。

## `StepSnapshot` 的稳定语义

当前 step 级结果要分成两类看：

- `consumed_streams`：该单元实际消费的输入流股快照
- `streams`：该单元实际产出的输出流股快照

它们都应由 solver step 直接物化。  
当前 workspace run path、Studio window-model、Result Inspector 和 Active Inspector 都应继续只读消费这两组结构化 DTO。

当前不应做的事：

- 不按 stream id 从全局 `SolveSnapshot.streams` 反查并重建 step 输入
- 不按 stream id 从全局 `SolveSnapshot.streams` 反查并重建 step 输出
- 不在 UI / shell 中为 step stream 合成 placeholder 结果

## 四类常见流股的正式边界

### 1. Source stream

source stream 当前不是“只有文档态输入，没有结果态”的对象。  
在 solved path 上，它应能按正式数值链路物化：

- `T / P / F / z`
- 已物化的相结果
- `overall` phase 上的 `H`（若已物化）
- `bubble_dew_window`（若已物化）

当 source stream 被 first consumer 消费时，consumer step 的对应输入流股应继续保持同一份 DTO 语义。

### 2. 非 flash 中间流股

`Heater`、`Cooler`、`Valve`、`Mixer` 的 flowing outlet 当前都属于正式结果对象，而不是给 downstream flash 准备的临时桥接值。

当前这类流股应继续沿同一条正式链路透传：

- upstream unit output
- `SolveSnapshot.streams`
- downstream step `consumed_streams`
- Studio consumer

这几处不应各自维护第二套 `H / phase_region / bubble_dew_window` 口径。

### 3. Flash outlet

`Flash Drum` outlet 需要区分 flowing outlet 和零流量对侧 outlet。

flowing outlet：

- 可带 phase rows
- 可带 `H`
- 可带 `bubble_dew_window`

零流量对侧 outlet：

- 当前允许 `bubble_dew_window` 缺席
- 当前允许 phase rows 缺席
- 当前允许 `H` 缺席

consumer 不应为了“显示完整”而补造这些缺席字段。

### 4. Step 输入/输出流股

step 输入/输出当前的职责是表达“这一步执行时实际看到的流股快照”，不是全局 stream map 的另一种索引视图。

因此：

- 若上游 output 与下游 consumed 指向同一股流，它们应保持同一份结果语义
- step DTO 的作用是锁定这个一致性，不是给 consumer 再做一轮拼装

## `H`、`phase_region` 和 `bubble_dew_window`

### `H`

当前结果区里说的 `H` 指：

- `molar_enthalpy_j_per_mol`

它来自已物化的相结果或 `overall` phase。  
当前 shell / UI 不应自行重算焓值。

### `phase_region`

`phase_region` 当前使用以下稳定词汇：

- `liquid-only`
- `two-phase`
- `vapor-only`

它属于 `bubble_dew_window` 的窗口语义，不是 phase rows 的直接替身。

### `bubble_dew_window`

窗口内至少包含：

- `phase_region`
- `bubble_pressure_pa`
- `dew_pressure_pa`
- `bubble_temperature_k`
- `dew_temperature_k`

对 two-phase flash outlet，当前允许窗口继续落在饱和边界附近；这时窗口语义和“实际只显示哪几行 phase rows”不必强行做成同一个概念。

## 对结果消费层的约束

当前稳定规则是：

- 只读消费 solver 已物化 DTO
- 不在集成层重算 `H`
- 不在集成层重算 `bubble/dew`
- 不在 UI / shell 中再组装第二套 step input/output
- 缺失字段按缺失处理，不伪造“更完整”的显示值

如果需要排查漂移，优先先比对：

1. `SolveSnapshot.streams`
2. step `consumed_streams`
3. step `streams`
4. 最终 Studio 展示

## 相关文档

- `docs/guides/review-solve-results.md`
- `docs/reference/units-and-conventions.md`
- `docs/architecture/app-architecture.md`
- `docs/thermo/mvp-model.md`
