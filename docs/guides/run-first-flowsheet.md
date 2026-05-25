# Run First Flowsheet

更新时间：2026-05-25

## 目的

本文档面向第一次实际运行 `RadishFlow Studio` 的读者，目标是用仓库内置示例走通一次最小求解闭环。

推荐示例：

- `examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json`

这个示例的好处是结构简单，同时已经覆盖：

- 流股输入
- 单元串联
- `Flash Drum` 两相分离
- 结果、步骤和诊断审阅

## 0. 前提

如果从历史内部便携 staging 体验，进入 staging 目录后直接启动：

```powershell
.\radishflow-studio.exe
```

包内示例位于 `examples/flowsheets`。当前 staging 是 Windows 便携目录形态，不是安装器、正式 demo 或 release 节点；不会执行 COM 注册、PME 自动化或第三方 CAPE-OPEN 模型加载。

如果从仓库开发态体验，建议当前工作区至少可通过最小构建检查：

```powershell
cargo check --workspace
```

如需完整仓库级验证，执行：

```powershell
pwsh ./scripts/check-repo.ps1
```

然后启动 Studio：

```powershell
cargo run -p radishflow-studio
```

## 1. 打开示例项目

启动后默认进入 Home Dashboard。第一次运行建议先在首页打开内置示例：

- 左侧 `开始` 区域的 `打开示例项目`
- 中央 `示例项目` 列表中的示例行；单击选择，双击整行打开
- 有最近项目时，也可以从 `最近项目` 选择或双击打开已记录的项目

进入工作台后，仍可通过顶部主路径打开项目：

- `打开示例`：内置示例入口
- `打开项目...`：Windows 原生文件选择器
- `视图` / 命令面板中的打开项目相关命令

如果当前工作区有未保存变更，打开示例、打开项目或新建项目都会先进入确认流程。选择继续后才丢弃当前未保存内容；取消则留在原项目。

第一次建议直接打开：

```text
examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json
```

如果你更想先看更复杂一点的路径，也可以改用：

```text
examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json
```

## 2. 确认项目内容

成功打开后，当前项目应至少包含：

- `Feed`
- `Heater`
- `Flash Drum`
- `stream-feed`
- `stream-heated`
- `stream-liquid`
- `stream-vapor`

此时你通常可以在 Canvas 和检查器中先确认：

- `Feed.outlet -> Heater.inlet`
- `Heater.outlet -> Flash Drum.inlet`
- `Flash Drum.liquid / vapor` 两个出口流股已存在

## 3. 触发运行

优先点击顶部主路径的 `运行`。

`运行` 会派发正式 `run_panel.run_manual` 命令。若按钮不可用，先看 hover 说明、右侧 `运行` 摘要和底部 `诊断`，而不是反复点击。需要从完整命令面执行时，可通过 `视图` 打开命令面板后查找运行命令。

成功时，当前工作流应从“项目已打开”推进到“已有一份最新 `SolveSnapshot`”。  
失败时，也应至少得到结构化诊断，而不是只有无上下文报错文本。

## 4. 查看结果

第一次运行后，Studio 预期会自动把右侧切到 `结果`、底部切到 `结果表`。优先看这四个位置：

1. 右侧 `结果` tab 的流股结果选择区
2. 右侧 `结果` tab 的单元结果和流股对比区
3. 底部 `结果表` 与 `诊断`
4. 右侧 `检查器` 中当前对象的关联结果

当前你应该特别注意以下结果字段：

- `T / P / F / H`
- `phase_region`
- `bubble_dew_window`
- `liquid / vapor / overall` 相结果

其中：

- `T` = `temperature_k`
- `P` = `pressure_pa`
- `F` = `total_molar_flow_mol_s`
- `H` = `molar_enthalpy_j_per_mol`

按对象读时，建议再多看一层：

- 先看 source stream，例如 `stream-feed`
- 再看 non-flash intermediate，例如 `stream-heated`
- 再看 step 输入/输出是否与全局流股结果保持同一份 DTO 语义
- 最后再看 `stream-liquid / stream-vapor` 这类 flash outlet 的窗口边界和缺席语义

如果你想按这条顺序系统地读一遍结果，继续阅读：

- `docs/guides/review-solve-results.md`

## 5. 保存并重新打开

如果只是验证运行链路，建议再顺手做一次保存与重开：

- 点击顶部主路径的 `保存`
- 或通过命令面板执行 `file.save`
- 或直接点击顶部 `另存为...` 到新的 `*.rfproj.json` 路径

如果当前是 `新建空白` 创建的未命名空白项目，首次点击 `保存` 会进入和 `另存为...` 相同的 Windows 保存选择器；它不应阻断后续 `新建空白 / 打开项目... / 打开示例`。

当前项目保存涉及两类文件：

- 项目真相源：`*.rfproj.json`
- Canvas 布局 sidecar：`*.rfstudio-layout.json`

其中：

- `*.rfproj.json` 保存流程语义、参数、连接和文档元信息
- `*.rfstudio-layout.json` 保存 Canvas placement 等 shell / layout 相关状态

如果当前项目已有 Canvas placement 或 viewport offset，保存并重开后应能恢复这份 sidecar 状态。单元拖动、空白画布平移和 `Fit to content` 只更新这份 sidecar，不改变项目流程语义。

## 6. 从空白项目建模

如果想验证“不是只会打开示例”，可以用当前 MVP α 支持的最短空白路径：

1. 在首页点击 `新建项目`，或进入工作台后点击顶部 `新建空白`，进入未命名空白项目。
2. 左侧切到 `放置`，放置 `进料` 和 `闪蒸罐`；需要中间设备时可加 `加热器 / 冷却器 / 阀门`，需要双入口时可加第二个 `进料` 和 `混合器`。按钮文案当前是 `放置进料`、`放置闪蒸罐` 等，项目对象名仍可显示为 `Feed`、`Flash Drum` 等领域名。
3. 每次放置单元后，在 Canvas 中点击落点提交；这只提交当前放置意图，不是完整拖拽布局编辑器。
4. 使用 Canvas suggestion 中的 `Connect stream` / `连接流股` 或 `Create stream` / `创建流股` 动作补齐 `source -> sink` 端口绑定和必要 outlet stream。`Heater / Cooler / Valve`、`Mixer` 和 `Flash Drum` 的 outlet stream 建议会等必要 inlet 绑定后才出现。
5. 从左侧 `项目` 或 Canvas 对象列表选择 stream / unit，右侧 `检查器` 会显示当前对象。
6. 流股检查器当前可编辑 `name / temperature_k / pressure_pa / total_molar_flow_mol_s` 和已有 flowsheet component catalog 中的组成条目；组成修改需要显式提交、归一化或丢弃。
7. 单元检查器当前已暴露首批高频参数：Feed 的 source temperature / pressure、Heater / Cooler 的 outlet temperature / outlet pressure、Mixer / Valve 的 outlet pressure 与 Flash Drum 的 flash temperature / flash pressure；字段使用 SI 单位并显示约束提示，Mixer outlet pressure 不能高于两股已连接 inlet pressure 的较低值，Heater / Cooler / Valve outlet pressure 若高于已连接 inlet pressure 会停留在无效草稿态。其余内容仍以端口、关联步骤、关联诊断和最新 `SolveSnapshot` 中的单元结果为主，不等同于完整单元参数表。
8. 点击 `运行`，成功后右侧会自动切到 `结果`，底部会自动切到 `结果表`；失败时会切到右侧 `运行` 和底部 `消息`，方便先看诊断。

如果误接或漏接了流股，可以选中对应 material stream 后使用右侧检查器或 Canvas 操作区中的受控恢复动作：

- `Disconnect stream`：解除所有 material port 绑定并保留流股规格。
- `Disconnect source` / `Disconnect sink`：只解除唯一 upstream source 或 downstream sink 端点。
- `Reconnect stream`：只补齐 source-only 或 sink-only 流股的唯一缺失端点；候选必须唯一、未绑定且不会形成 unit dependency cycle。
- `Delete stream`：解除绑定后删除流股。

这些动作都进入 undo/redo 历史，但仍不是自由连线、任意端口选择或自动布线工具。

当前仍不支持自由拉线、任意端口点击创建、任意端口重连、自动布线、完整组件库、完整物性包浏览/切换或完整单元参数表。这些缺口若影响验证，应记录为 MVP α 后续任务，而不是用 shell 私有状态绕过。

## 7. MVP β 小案例作者路径

如果你已经能跑通内置示例，下一步建议从空白项目手工复现一个小案例。当前推荐先用 `Feed + Feed -> Mixer -> Flash Drum`，因为它同时覆盖双入口连接、单元参数、保存重开和结果导出；也可以从首页选择 `创建 Heater-Flash 小案例`，用同一任务清单机制复现 `Feed -> Heater -> Flash Drum`。

建议步骤：

1. 在首页点击 `创建 Mixer-Flash 小案例`，或手动新建空白项目后切到左侧 `放置`。该入口只负责打开作者任务清单，不会自动生成 flowsheet。
2. 放置两个 `Feed`、一个 `Mixer` 和一个 `Flash Drum`。
3. 依次接受本地 suggestion，形成：
   - `Feed 1 -> Mixer.inlet_a`
   - `Feed 2 -> Mixer.inlet_b`
   - `Mixer.outlet -> Flash Drum.inlet`
   - `Flash Drum.liquid / vapor` 两个出口流股
4. 在单元检查器中提交一组 SI 参数：
   - `Feed 1` source temperature = `305 K`，source pressure = `130000 Pa`
   - `Feed 2` source temperature = `315 K`，source pressure = `120000 Pa`
   - `Mixer` outlet pressure = `90000 Pa`
   - `Flash Drum` flash temperature = `300 K`，flash pressure = `85000 Pa`
5. 点击顶部 `运行`，确认运行收敛，`stream-mixer-1-outlet` 总摩尔流量为两股入口之和。
6. 保存项目，关闭或重新打开该项目，再次运行。
7. 在右侧 `结果` 区复制当前 `SolveSnapshot`，或导出为轻量 `.txt`。

这条路径的目标不是新增项目向导，而是验证用户能按现有 `放置 -> suggestion -> 单元参数 -> 运行 -> 保存重开 -> 结果导出` 工作流复现一个小案例。连接仍通过正式 suggestion 和 `DocumentCommand` 完成；参数仍只覆盖当前已暴露的 MVP 高频字段；导出只消费当前结果 DTO，不写项目、不进入 undo。

## 8. 常见阻塞点

如果当前示例没有直接跑通，优先检查以下几类问题：

- 流股检查器中是否还有未提交或无效草稿
- 文档态流股组成是否未归一到 1
- 当前运行环境下是否出现多包可选且未显式指定 package
- 项目是否被改成了不完整连接或不一致端口绑定
- Feed source temperature / pressure 或 Flash Drum flash temperature / flash pressure 是否为正有限 SI 值；Mixer / Heater / Cooler / Valve outlet pressure 是否高于已连接 inlet pressure 约束。若越界值已存在于项目文档，运行诊断会归类为 `solver.step.parameter`
- 顶部 `运行` 是否处于 disabled 状态，以及 hover 文案给出的原因
- 启动 Studio 的终端 stderr 是否有 `[radishflow-studio]` 审计线或 GUI panic 提示

当前系统的默认处理原则是：

- 不做隐式自动补偿
- 不在运行前偷偷改写文档
- 优先通过结构化诊断暴露问题
- Run Panel recovery action 只执行明确的局部恢复；不能自动修复时，应聚焦到相关 unit / port / stream，帮助用户进入可编辑位置
- 用户主动修正错连或漏连时，优先使用选中 stream 后的 `Disconnect stream`、`Disconnect source`、`Disconnect sink`、`Reconnect stream` 或 `Delete stream`；不要把它理解为自由连线或任意重连工具
- 开发态 stderr 只作为 smoke 和排查辅助，不替代右侧 `运行` 或底部 `诊断` 中的用户可见诊断

连接类诊断当前应按下面理解：

- `missing_upstream_source`、`unbound_outlet_port`、cycle / self-loop 等失败会尽量在 Run Panel、failure detail、Canvas attention 和端口 attention 中保留相关 unit / port / stream target
- 坏 stream 引用、orphan stream、重复 source / sink 等失败若存在明确 mutation，recovery action 可以执行断开、删除、创建或绑定这类局部修复
- 若诊断里引用的 stream 已不存在，界面不应渲染一个不可达 stream target，而应指向持有坏引用的 unit / port

组成相关提示可按下面理解：

- `Draft`：当前输入值还只是草稿，尚未写入项目文档
- `Unnormalized`：组成已经写入文档，但总和不是 1
- `Normalize composition`：显式把当前组成归一化；它不会代替用户猜测新增或删除组分
- `Remove` / add component：只在当前 flowsheet 已有组件目录内操作，不触发项目级组件迁移

## 9. 下一步建议

如果这次运行已经走通，下一步建议按下面顺序继续：

1. 修改一个流股字段并重新运行
2. 在结果检查器对比不同流股结果
3. 阅读 `docs/guides/review-solve-results.md`
4. 阅读 `docs/reference/units-and-conventions.md`
5. 阅读 `docs/reference/solve-snapshot-results.md`
6. 阅读 `docs/thermo/mvp-model.md`

如果你接下来更关心系统边界，而不是继续点 UI，则直接转到：

- `docs/architecture/overview.md`
