# Review Solve Results

更新时间：2026-05-31

## 目的

本文档面向已经能打开并运行示例 flowsheet 的读者，解释当前应该如何在 Studio 中审阅 `SolveSnapshot` 结果。

它回答的是：

- 先看哪几类流股和步骤
- `流股选择`、`流股对比`、`单元结果` 三种结果面该怎么配合看
- `source stream`、非 flash 中间流股、flash outlet、unit step 输入/输出各自该怎么看
- `检查` / `诊断目标` / `结果` commands 应该怎样帮助你核对同一份结果
- `H`、`phase_region`、`bubble_dew_window` 在结果区里分别代表什么
- 当前快照复制 / 导出应该怎样理解
- 从小案例作者路径运行后应该先核对哪些结果

它不是架构文档，也不展开测试或实现细节。

## 推荐示例

建议优先使用当前 official hydrocarbon 示例：

- `examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json`
- `examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json`
- `examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json`

如果只是第一次上手，先从第一条开始；如果你更想看 non-flash intermediate 的 `bubble_dew_window`，第二条更直观。

如果你是从 Home 的 `创建 Mixer-Flash 小案例` 或 `创建 Heater-Flash 小案例` 进入，先按 `docs/guides/author-small-cases.md` 完成放置、连接和参数提交，再回到本文档审阅结果。

## 先看哪四处

一次运行成功后，先按下面顺序看：

1. 右侧 `结果检查器` 的流股结果视图
2. 右侧 `结果检查器` 的单元结果视图
3. `关联求解步骤` / step 列表
4. 当前对象 `检查器`

当前这四处都应该只读消费同一份 `SolveSnapshot` DTO；如果某个字段只在其中一处出现，通常应先怀疑消费层回归，而不是先猜数值层分叉。

当前中文 UI 中，这四处通常对应右侧 `结果` tab、右侧 `检查器` tab 中的关联结果、底部 `结果表 / 诊断`，以及命令入口中的结果定位项。英文术语在本文档中只用于指代内部结果组织方式，不表示默认界面必须显示英文。

## 小案例作者路径的结果核对

从 `Mixer-Flash` 作者路径运行后，先按下面链路核对：

- 输入流股：两股 Feed outlet 的 `T / P / F / composition / H / bubble_dew_window`
- 中间流股：mixer outlet 的总摩尔流量应为两股 Feed outlet 之和，composition 应为摩尔流量加权结果
- `Mixer` 单元结果：输入流股应包含两个 Feed outlet，产出流股应是 mixer outlet
- `Flash Drum` 单元结果：输入流股应是 mixer outlet，产出流股应包含 liquid / vapor
- Flash 分割：liquid / vapor 两股 outlet 的总摩尔流量之和应等于 flash inlet
- 相态 / 焓值：flowing outlet 应能看到 phase row 和 `H`；two-phase case 中 liquid / vapor outlet 的窗口分别落在 bubble / dew 边界
- 右侧 `结果` 区的复制 / 导出文本应来自同一份最新 `SolveSnapshot`

从 `Heater-Flash` 作者路径运行后，先按下面链路核对：

- 输入流股：Feed outlet 的 `T / P / F / composition / H / bubble_dew_window`
- 中间流股：heater outlet 的温度和压力应反映已提交的 `Heater` outlet temperature / outlet pressure
- `Heater` 单元结果：输入流股应是 Feed outlet，产出流股应是 heater outlet
- `Flash Drum` 单元结果：输入流股应是 heater outlet，不应直接消费 Feed outlet
- Flash 分割：liquid / vapor 两股 outlet 的总摩尔流量之和应等于 flash inlet；若是 vapor-only 条件，零流量 liquid outlet 允许缺席 phase rows、`H` 和窗口
- 相态 / 焓值：flowing vapor outlet 应能看到 Vapor phase row、`H` 和 vapor-only 窗口
- flash liquid / vapor outlet：应能在 stream result 和 unit result 中互相定位

这些核对只读消费运行后的 `SolveSnapshot`。如果结果不符合预期，先回到 `检查器` 查看单元参数是否已经提交，再检查 Canvas suggestion 是否已经把对应 source / sink 端点补齐。

official hydrocarbon demo 的稳定数值口径详见 `docs/guides/author-small-cases.md` 中的“结果核对与案例说明 v0”。本文只说明阅读顺序，不复制每个示例的输入表。

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

右侧 `结果检查器` 的选择区应是紧凑可选项：流股、对比流股和单元以按钮 / chip 形式切换，不应为每个选项重复显示 `Inspect`。需要跳到对象详情时，使用当前选项、`检查` 动作、诊断目标或命令入口定位到同一份对象结果。

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

切到 `结果检查器` 的单元结果视图后，重点看某个 step 的：

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

## 5. 用 `检查`、`诊断目标` 和 `结果` commands 交叉核对

当前结果审阅不只靠静态字段，还可以借助两类动作面：

- `检查`
- `诊断目标`
- command palette / menu / command list 里的 `Results` commands
- 右侧 `结果` 区的当前快照轻量复制 / 导出

推荐用法：

1. 在流股对比里用 `检查` 或当前流股选项从 `stream-liquid / stream-vapor` 跳到对应对象详情
2. 在单元结果视图里用输入/输出流股的 `检查`，核对 `Flash Drum` inlet/outlet 和 step stream 是否还是同一份结果
3. 在 `诊断目标` 里再跳一次 flash inlet 或 flash unit，确认 `结果检查器 -> 当前检查器` 没有分叉成第二套 consumer 语义
4. 在 command palette 或菜单中搜索 `result` / `snapshot` / stream label，确认 `Results` command 也定位到同一份当前快照结果

这里要注意：

- `检查` 只是定位到当前已有 stream/unit 结果，不会重新求解
- `诊断目标` 只汇总当前 `SolveSnapshot`、相关 step 和相关 diagnostic 已经存在的目标，不是 shell 私造的第三套导航模型
- `Results` commands 也只派发既有 `inspector.focus_stream:*` / `inspector.focus_unit:*`，不会创建第二套结果缓存
- `复制快照` / `导出文本` 只把当前同一份 `SolveSnapshot` 格式化为纯文本，覆盖流股摘要、求解步骤和诊断；它们不写项目文件、不进入 undo，也不是完整报表、模板系统或批量导出入口
- 如果某个 section 没有 `诊断目标`，应先理解为“当前没有已物化目标”，而不是默认它被隐藏或漏显示

底部 `结果表` 当前采用中文 `流股 / 相态` 表头。`相态` 列应显示短摘要，例如 `总体 1.000`、`气相 1.000`、`无`，过长的原始相态明细只适合放进 tooltip、日志或开发诊断，不应撑开表格或裁切主要数值列。

## 底部结果表

底部 `结果表` 当前分两段展示同一份 `SolveSnapshot`：

- 上半段是流股表：按流股列出 `T / P / F / H / 相态`，点击流股会切到右侧 `结果` 对应流股。
- 下半段是单元表：按每个单元的最新求解步骤列出状态、step 序号、消费流股和产出流股，点击单元会切到右侧 `结果` 的单元结果面。

这张表只用于快速核对当前快照，不保存结果、不触发求解，也不是完整报表系统。小案例作者路径运行后，建议先在流股表确认关键 outlet，再在单元表确认 upstream / downstream 消费关系是否正确。

## 当前快照复制 / 导出

右侧 `结果` 区当前提供 `复制快照` 与 `导出文本...`。它们只把当前最新 `SolveSnapshot` 格式化成轻量纯文本，内容覆盖：

- 流股摘要
- Review 摘要：source / intermediate / terminal streams、latest unit results、diagnostics count
- 单元结果摘要
- 求解步骤
- 诊断条目

其中 `Review` 区按 case-level 审阅顺序汇总 source、intermediate、terminal stream，并列出 latest unit results 的状态、消费流股和产出流股；`Units` 区按单元列出最新 step、状态、summary、输入流股和输出流股，便于人工复核 unit-centric 结果。它们都从当前同一份 `SolveSnapshot` 派生，不是独立结果模型。

稳定边界：

- 不写 `*.rfproj.json`
- 不写 layout sidecar
- 不进入 undo / redo
- 不触发重新求解
- 不生成跨快照历史、模板化报表或批量导出任务

如果需要归档某次结果，这个文本可以作为轻量人工审阅材料；如果需要可复算的正式 case，仍应保存项目文件本身。

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
4. 结果检查器与当前对象检查器的展示

如果 1 到 3 已经不一致，先查 solver / snapshot。  
如果 1 到 3 一致、但 4 不一致，优先查 Studio consumer。

## 相关文档

- `docs/guides/run-first-flowsheet.md`
- `docs/reference/units-and-conventions.md`
- `docs/reference/solve-snapshot-results.md`
- `docs/thermo/mvp-model.md`
