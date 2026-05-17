# Review Solve Results

更新时间：2026-05-12

## 目的

本文档面向已经能打开并运行示例 flowsheet 的读者，解释当前应该如何在 Studio 中审阅 `SolveSnapshot` 结果。

它回答的是：

- 先看哪几类流股和步骤
- `stream selector`、`comparison`、`unit-centric` 三种结果面该怎么配合看
- `source stream`、非 flash 中间流股、flash outlet、unit step 输入/输出各自该怎么看
- `Inspect` / `DiagnosticTargets` / `Results` commands 应该怎样帮助你核对同一份结果
- `H`、`phase_region`、`bubble_dew_window` 在结果区里分别代表什么

它不是架构文档，也不展开测试或实现细节。

## 推荐示例

建议优先使用当前 official hydrocarbon 示例：

- `examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json`
- `examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json`
- `examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json`

如果只是第一次上手，先从第一条开始；如果你更想看 non-flash intermediate 的 `bubble_dew_window`，第二条更直观。

## 先看哪四处

一次运行成功后，先按下面顺序看：

1. `Result Inspector` 的 stream-centric 视图
2. `Result Inspector` 的 unit-centric 视图
3. `Related solve steps` / step 列表
4. `Active Inspector`

当前这四处都应该只读消费同一份 `SolveSnapshot` DTO；如果某个字段只在其中一处出现，通常应先怀疑消费层回归，而不是先猜数值层分叉。

## 先定 selector，再看 comparison / unit

当前结果区至少有三类选择态：

- `selected stream`
- `comparison stream`
- `selected unit`

稳定阅读方式是：

1. 先定一个 base stream
2. 再决定是否加 comparison
3. 最后再切 unit-centric 视图核对 step 输入/输出

有两条当前应视为正式行为的规则：

- `comparison` 只比较当前同一份 `SolveSnapshot` 里已经存在的两股流，不会触发第二次求解
- 如果把 base stream 切成当前 compared stream，comparison 会被清空；这表示 selector state 复位，不表示结果丢失

`selected unit` 也只是切换“看哪一个单元的结果面”，不应该改变任何 stream result 本身的数值语义。

## 1. 先看 source stream

第一次建议先选 `stream-feed`。

当前稳定预期是：

- source stream 本身就应带 `T / P / F / z`
- 若当前 solved path 已物化 `overall` phase，则应能看到 `H`
- 若当前 thermo path 已物化窗口，则应能看到 `phase_region` 与 `bubble_dew_window`

source stream 的这些字段不是给 downstream consumer 现算的临时值。  
当它被第一个单元消费时，consumer step 的输入流股应继续表达同一份结果语义。

## 2. 再看非 flash 中间流股

对 `Feed -> Heater/Cooler/Valve/Mixer -> Flash` 这类链路，第二步建议看：

- `stream-heated`
- `stream-cooled`
- `stream-throttled`
- `stream-mix-out`

当前这些 non-flash intermediate stream 也应能稳定展示：

- `T / P / F / H`
- `phase_region`
- `bubble_dew_window`

阅读原则：

- 先把它当作“当前流股自己的正式结果”
- 再去看 downstream flash inlet 的 consumed stream
- 两边应保持同一份 DTO 语义，而不是一边来自 unit output、一边来自 consumer 现算

## 3. 再看 unit step 输入/输出

切到 `Result Inspector` 的 unit-centric 视图后，重点看某个 step 的：

- `consumed_streams`
- `streams`

当前稳定语义是：

- `consumed_streams` 表示这个单元实际消费的输入流股快照
- `streams` 表示这个单元实际产出的输出流股快照

它们都应由 solver 直接物化，再沿 `rf-solver -> rf-ui::SolveSnapshot -> workspace run / Studio consumer` 透传。  
Studio 不应再通过全局 stream 列表按 id 回填、拼装或猜测第二套 step 输入/输出结果。

## 4. 最后看 flash outlet

`Flash Drum` outlet 是最容易误读的一类结果。

当前建议分别看：

- `stream-liquid`
- `stream-vapor`

### Two-phase 场景

在 two-phase 样例里，flowing liquid/vapor outlet 当前通常都应有：

- `bubble_dew_window`
- phase rows
- `H` 摘要

并且有两条专门的边界语义：

- liquid outlet 的 `bubble_pressure/temperature` 应与该 outlet 当前 `P/T` 对齐
- vapor outlet 的 `dew_pressure/temperature` 应与该 outlet 当前 `P/T` 对齐

### Single-phase 场景

在 single-phase 场景里，零流量对侧 outlet 当前应保持缺席语义，而不是伪造完整结果。

常见表现是：

- `bubble_dew_window` 缺席
- phase rows 缺席
- `H` 摘要缺席

这不是 UI 漏显示，而是当前稳定边界的一部分。

## 5. 用 `Inspect`、`DiagnosticTargets` 和 `Results` commands 交叉核对

当前结果审阅不只靠静态字段，还可以借助两类动作面：

- `Inspect`
- `DiagnosticTargets`
- command palette / menu / command list 里的 `Results` commands

推荐用法：

1. 在 stream comparison 里用 `Inspect` 从 `stream-liquid / stream-vapor` 跳到对应对象详情
2. 在 unit-centric 视图里用输入/输出流股的 `Inspect`，核对 `Flash Drum` inlet/outlet 和 step stream 是否还是同一份结果
3. 在 `DiagnosticTargets` 里再跳一次 flash inlet 或 flash unit，确认 `Result Inspector -> Active Inspector` 没有分叉成第二套 consumer 语义
4. 在 command palette 或菜单中搜索 `result` / `snapshot` / stream label，确认 `Results` command 也定位到同一份当前快照结果

这里要注意：

- `Inspect` 只是定位到当前已有 stream/unit 结果，不会重新求解
- `DiagnosticTargets` 只汇总当前 `SolveSnapshot`、相关 step 和相关 diagnostic 已经存在的目标，不是 shell 私造的第三套导航模型
- `Results` commands 也只派发既有 `inspector.focus_stream:*` / `inspector.focus_unit:*`，不会创建第二套结果缓存
- 如果某个 section 没有 `DiagnosticTargets`，应先理解为“当前没有已物化目标”，而不是默认它被隐藏或漏显示

## 如何读关键字段

### `H`

- `H` 指 `molar_enthalpy_j_per_mol`
- 当前来源是已物化的相结果或 `overall` phase
- Studio 只展示已有值，不在 shell 中补算

如果某股流没有 `H`，当前应该按“未物化”处理，而不是再猜一个默认焓值。

### `phase_region`

- `phase_region` 表示当前 `T / P / z` 下的最小相区间判断
- 它属于 `bubble_dew_window` 的一部分

它不等于“当前一定会显示几行 phase rows”。  
例如 flash outlet 在边界态上可以合法地携带 `two-phase` 窗口，同时实际只显示一股 flowing phase 结果。

### `bubble_dew_window`

当前它至少应包含：

- `phase_region`
- `bubble_pressure_pa`
- `dew_pressure_pa`
- `bubble_temperature_k`
- `dew_temperature_k`

阅读时优先把它当成“当前流股 overall composition 的平衡边界窗口”，不要把它误当作另一个独立求解器或 UI 私有推断结果。

## 调试时的最小核对顺序

如果你怀疑某条结果链路有漂移，建议按这个顺序核对：

1. 全局 `stream` 结果
2. upstream unit step 的输出流股
3. downstream unit step 的输入流股
4. Result Inspector 与 Active Inspector 的展示

如果 1 到 3 已经不一致，先查 solver / snapshot。  
如果 1 到 3 一致、但 4 不一致，优先查 Studio consumer。

## 相关文档

- `docs/guides/run-first-flowsheet.md`
- `docs/reference/units-and-conventions.md`
- `docs/reference/solve-snapshot-results.md`
- `docs/thermo/mvp-model.md`
