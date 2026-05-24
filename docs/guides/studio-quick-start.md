# Studio Quick Start

更新时间：2026-05-24

## 目的

本文档面向第一次进入仓库、想直接体验 `RadishFlow Studio` 当前最小工作台闭环的读者。

它回答的是：

- 当前 Studio 已经能做什么
- 如何从内部便携包或开发态启动 Studio
- 第一次建议打开哪个示例
- 接下来应该看哪些文档

它不替代架构文档，也不展开未来规划。

## 当前能做什么

截至 2026-05-24，Studio 当前已经具备以下最小闭环：

- 启动后默认进入中文 Home Dashboard，可从 `开始 / 最近项目 / 示例项目 / 环境 / 消息` 分区判断从哪里开始
- 新建未命名空白项目，并用 MVP 默认 `methane / ethane` 二元体系进入最短建模路径
- 打开已有 `*.rfproj.json` 项目
- 通过首页 `新建项目`、`打开项目`、`打开示例项目` 或进入工作台后的顶部主路径切换项目
- 最近项目和示例项目列表行可选择，也可双击整行打开；文件缺失时只降级对应行状态，不阻断首页
- 进入项目后在顶部主路径直接使用 `Home / 打开示例 / 新建空白 / 打开项目... / 运行 / 保存 / 另存为... / 视图`
- 运行仓库内或便携包内的 official hydrocarbon 正向示例 flowsheet
- 在左侧 `项目 / 示例项目 / 放置`、中央 `Canvas`、右侧 `检查器 / 结果 / 运行 / 物性包` 和底部 `消息 / 运行日志 / 结果表 / 诊断` 中完成当前 MVP α 工作流
- 在当前 `SolveSnapshot` 内切换 stream-centric / unit-centric / comparison 三类结果审阅面
- 复制当前 `SolveSnapshot` 文本，或导出当前快照为轻量 `.txt`
- 通过 `检查`、`诊断目标`、结果选择项和命令入口在流股、单元、步骤和当前检查器之间定位同一份结果
- 在流股检查器中编辑流股基础字段与组成草稿，并显式提交、归一化或丢弃
- 在单元检查器中编辑首批关键单元参数：`Heater / Cooler` 的 outlet temperature / outlet pressure、`Mixer / Valve` 的 outlet pressure 和 `Flash Drum` 的 flash temperature / flash pressure
- 选中物料流股后，可通过 Canvas / Inspector 的 `Disconnect stream` 解除端口绑定并保留流股规格，或通过 `Delete stream` 解除绑定后删除错误流股；单端流股还可在唯一且不会成环的候选存在时执行受控 `Reconnect stream`
- 执行基础 `undo / redo`
- 保存当前项目，或通过顶部 `另存为...` / 未命名项目首次 `保存` 到新路径
- 保存并恢复 Canvas placement / viewport sidecar：`<project>.rfstudio-layout.json`；当前可拖动单元位置、平移 viewport，并用 `Fit to content` 重新居中
- 默认隐藏低频命令大全；需要完整命令列表时可从顶部 `视图` 或命令面板入口展开

当前最短可求解建模路径已经覆盖：

- `Feed -> Flash Drum`
- `Feed -> Heater/Cooler/Valve -> Flash Drum`
- `Feed + Feed -> Mixer -> Flash Drum`

## 当前明确还不是的东西

Studio 现在还不是完整产品说明书意义上的“成熟桌面软件”。以下能力当前仍不属于稳定范围：

- 完整自由连线编辑器
- 完整拖拽式布局编辑器
- 完整组件库和物性包浏览器
- 完整结果报表、模板和批量导出
- 跨快照历史对比系统
- 完整 CAPE-OPEN 第三方模型加载

因此，第一次体验应优先基于仓库自带示例，而不是把它当成已经收口的通用生产工具。

## 启动方式

如果你拿到的是内部便携包，先解压或进入 staging 目录：

```text
artifacts/packages/RadishFlow-v26.5.1-dev-windows-x64/
```

然后直接启动包内入口：

```powershell
.\radishflow-studio.exe
```

包内示例目录位于 exe 同目录的：

```text
examples/flowsheets
```

当前 Studio 会优先从 exe 同目录发现内置示例；如果不是从便携包启动，则回退到仓库内 `examples/flowsheets`。

说明：

- 这是 Windows 内部便携包 / staging 形态，不是安装器
- 不会执行 COM 注册、PME 自动化、Windows Registry 写入或第三方 CAPE-OPEN 模型加载
- 包内 `PACKAGE-MANIFEST.txt` 应记录 `version=v26.5.1-dev`、`gitCommit=7479e82`、`gitDirty=false`

开发态启动方式如下。

如需先做仓库级验证，执行：

```powershell
pwsh ./scripts/check-repo.ps1
```

开发态启动 Studio：

```powershell
cargo run -p radishflow-studio
```

开发态说明：

- 这是长时间运行的桌面 UI 命令
- Windows 当前已接入原生打开/另存为选择器；其他平台的文件工作流暂不承诺同等完成度

## 第一次建议体验什么

第一次建议直接打开以下 official hydrocarbon 正向示例之一：

- `examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json`
- `examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json`

如果只想先走最短路径，优先第一个示例。它覆盖：

- 一个 `Feed`
- 一个 `Heater`
- 一个 `Flash Drum`
- 一条单入口、双产物流股的最小求解链路

详细操作步骤见：

- `docs/guides/run-first-flowsheet.md`

## 启动首页

启动后，第一视野是 Home Dashboard，而不是直接进入某个项目的画布。首页的稳定分区如下：

- `开始`：`新建项目`、`打开项目`、`打开示例项目`
- `最近项目`：显示最近项目、路径摘要、物性包和就绪 / 缺失状态；整行可选择，双击可打开
- `示例项目`：显示内置示例、流程摘要、组分和物性包；整行可选择，双击可打开
- `环境`：显示客户端、服务端和设备三组摘要
- `消息`：显示登录、示例目录、物性包缓存等可行动摘要

工程术语、文件名、包名和路径会保留原文；用户动作、状态和环境字段默认使用中文。首页不承载流程图编辑，打开项目或示例后才进入工作台。

首版 demo 前，Home / Workbench 的默认示例选择器只暴露四条 official hydrocarbon 演示路径：`Feed -> Heater/Cooler/Valve -> Flash Drum` 与 `Feed + Feed -> Mixer -> Flash Drum`。仓库和便携包内仍可能附带 synthetic 或 PME 验证样例文件，但这些文件主要服务回归或外部验证，不作为首页高频演示入口。

若当前工作区存在未保存变更，首页的 `新建项目`、`打开项目`、`打开示例项目` 以及工作台顶部的项目切换入口都会先进入显式确认流程；继续后才丢弃当前未保存内容，取消则保持当前项目不变。

## 工作台主路径

进入项目后，顶部第一行展示应用、当前项目和状态 chip；第二行提供当前主路径：

- `Home`：返回启动首页
- `打开示例`：打开仓库或便携包内置正向示例
- `新建空白`：新建未命名空白项目；不会立刻弹出保存对话框
- `打开项目...`：从磁盘选择已有 `*.rfproj.json`
- `运行`：对当前工作区执行一次手动运行；不可用时 hover 会说明原因
- `保存`：保存当前项目
- `另存为...`：把当前项目另存到新的 `*.rfproj.json` 路径；未命名空白项目首次 `保存` 也会进入这条选择器路径
- `视图`：收纳低频视图入口、语言切换、命令面板和开发诊断入口

命令面板默认不占据第一视野。它不是功能移除，而是把低频和调试型入口从主工作流移开。

## 从空白项目开始

如果不想先打开示例，可以直接走当前最小空白建模路径：

1. 在首页点击 `新建项目`，或进入工作台后点击顶部 `新建空白`。
2. 在左侧切到 `放置`，用 `放置进料`、`放置闪蒸罐` 或 `放置加热器 / 放置冷却器 / 放置阀门 / 放置混合器` 开始放置单元。
3. 在 Canvas 中点击落点提交当前放置意图。
4. 使用 Canvas 上的 `Connect stream` / `连接流股` 或 `Create stream` / `创建流股` suggestion 补齐端口绑定和必要 outlet stream。`Heater / Cooler / Valve`、`Mixer` 和 `Flash Drum` 的 outlet stream 建议会等必要 inlet 绑定后才出现。
5. 在左侧 `项目` 或 Canvas 对象列表中选择 stream / unit，右侧 `检查器` 会切到对应对象。
6. 在流股检查器中编辑 `T / P / F` 和组成草稿；字段提交、全部应用、组成归一化都是显式动作。
7. 选中 `Heater / Cooler / Mixer / Valve / Flash Drum` 时，可在单元检查器中编辑已暴露的 outlet temperature、outlet pressure、flash temperature 或 flash pressure 字段；字段会显示 SI 单位和约束提示，提交后写回项目参数，并同步对应 outlet stream 模板。
8. 若流股错连或漏连，先选中该 material stream，再使用 `Disconnect stream`、`Disconnect source`、`Disconnect sink`、`Reconnect stream` 或 `Delete stream` 这组受控恢复动作；`Reconnect stream` 只在单端唯一候选且不会形成 unit dependency cycle 时可用。
9. 点击顶部 `运行`，结果只从最新 `SolveSnapshot` 展示到右侧 `结果` 和底部 `结果表`。

当前连接仍通过本地 suggestion 和正式 `DocumentCommand` 完成，不是自由拉线编辑器；单元参数编辑也仍限制在上述 MVP 已暴露字段，不等同于完整单元参数表。

## 单元参数与连接诊断

当前首批单元参数字段只覆盖最短建模路径中的高频项：

- `Heater / Cooler`：`outlet temperature`，单位 K
- `Heater / Cooler`：`outlet pressure`，单位 Pa；不能高于已连接 inlet pressure
- `Mixer`：`outlet pressure`，单位 Pa；不能高于两股已连接 inlet pressure 的较低值
- `Valve`：`outlet pressure`，单位 Pa；不能高于已连接 inlet pressure
- `Flash Drum`：`flash temperature`，单位 K；`flash pressure`，单位 Pa；提交后同步 liquid / vapor 两个出口流股模板，并作为 TP Flash 输入参与求解

`Mixer / Heater / Cooler / Valve` 的 outlet pressure 若高于已连接 inlet pressure 约束，会在单元检查器草稿态直接标记为无效，保持草稿、不写回项目文档，也不会同步 outlet stream 模板。Flash Drum 的 flash temperature / flash pressure 当前只要求正有限 SI 值，不施加 inlet pressure 上限。若旧项目或外部编辑已经把越界参数写入文档，运行会产生 `solver.step.parameter` 诊断，并把 failure detail、端口 attention 和 recovery action 指向相关 unit / port / stream。

连接类失败同样会尽量携带可修复目标：例如缺失 upstream source、未绑定 outlet port、cycle、自环、坏 stream 引用、重复 source / sink 或 orphan stream。Run Panel 中的 recovery action 可能只是聚焦相关 unit / port / stream，也可能执行明确的局部修复动作；按钮文案应区分这两类行为。

如果在当前无自由拉线阶段误接了流股，可以先选中这条物料流股，再使用右侧检查器或 Canvas 操作区中的恢复动作：

- `Disconnect stream`：解除该流股绑定到的所有 material ports，流股规格保留在项目中，便于后续重新按 suggestion 补连。
- `Disconnect source` / `Disconnect sink`：只解除唯一 upstream source 或 downstream sink 端点，适合保留另一端连接后继续补齐。
- `Reconnect stream`：只补齐 source-only 或 sink-only 流股的唯一缺失端点；若无候选、候选不唯一、已双端连接或候选会形成 unit dependency cycle，则不可用。
- `Delete stream`：解除 material port 绑定后删除该流股，适合清理误创建或错误连接的流股。

这些动作会进入文档历史，可用 `undo / redo` 回退；它们只覆盖当前 MVP material stream，不提供任意端口重连、自动布线或完整画布编辑。

## Canvas 布局与视口

Canvas 中的单元位置和 viewport offset 保存到项目同目录的 `<project>.rfstudio-layout.json` sidecar，不写入 `*.rfproj.json`，也不参与求解。

当前可用的布局 / 视口动作包括：

- 放置单元时点击落点，写入 sidecar 中的单元位置。
- 选中单元后在空白处点击，可把该单元定位到点击位置。
- 直接拖动单元块，释放后保存最终位置。
- 在空白 Canvas 上拖拽可平移 viewport，并保存 offset。
- `Fit to content` 会按当前内容重新居中 viewport。

这些动作都不进入 `CommandHistory`，不改变流程语义，也不代表完整拖拽布局编辑器、自动布线或完整 camera state 持久化。

## 启动后应该看到什么

成功打开示例后，当前 Studio 应至少能让你扫读到以下信息：

- 顶部主路径中的 Home、打开示例、新建空白、打开项目、运行、保存、另存为和视图入口
- 顶部当前项目标题、运行状态、pending 状态和未保存提示；完整路径优先进入摘要、tooltip 或详情区域
- 左侧 `项目 / 示例项目 / 放置`，分别用于项目树扫读、示例入口和放置 MVP 内建单元
- Canvas 上的单元、物流线和当前关注对象
- 右侧 `检查器 / 结果 / 运行 / 物性包` tabs，其中 `检查器` 负责当前对象参数、组成、端口和关联结果，`结果` 负责只读结果审阅，`物性包` 负责本地包和同步状态摘要
- 底部 `消息 / 运行日志 / 结果表 / 诊断` drawer，其中结果表只读消费当前 `SolveSnapshot`，默认消息区比结果 / 诊断页更紧凑
- `诊断目标` 中可直接定位的流股 / 单元结果目标

如果运行成功，Studio 会自动把右侧切到 `结果`、底部切到 `结果表`。`Flash Drum` 相关结果当前应能进一步展示：

- `phase_region`
- `bubble_dew_window`
- `liquid / vapor / overall` 相结果
- 各相摩尔流量与 molar enthalpy

## 流股检查器组成编辑

当前流股检查器的组成编辑遵循显式提交原则：

- `Draft` 表示有未提交的组成草稿
- `Unnormalized` 表示组成已经进入项目文档，但总和不是 1
- `Normalize composition` 会按当前组成显式归一化
- 组分添加 / 删除只从当前 flowsheet component catalog 派生，不创建完整组件库
- 运行前若仍有未提交草稿或未归一化文档组成，应先阻断并显示诊断，不做隐式差值补偿

## 运行反馈和退出

开发态启动时，Studio 会向 stderr 输出带 `[radishflow-studio]` 前缀的用户操作与求解审计线。这些输出服务 smoke 和排查，不代表正式 telemetry 或长期审计接口。

如果 GUI 回调发生内部 panic，当前壳层会降级到错误页，并提示查看 stderr。若只是关闭最后一个 Studio 窗口，当前预期是自然退出进程，不应短暂闪回默认 Commands 左栏，也不应留下黑屏但进程不退出的状态。

如果你接下来更关心“这些结果分别代表什么”，而不是只看字段名字，直接继续读：

- `docs/guides/review-solve-results.md`
- `docs/reference/solve-snapshot-results.md`

## 下一步应该读什么

按使用顺序，建议继续阅读：

1. `docs/guides/run-first-flowsheet.md`
2. `docs/guides/review-solve-results.md`
3. `docs/reference/units-and-conventions.md`
4. `docs/reference/solve-snapshot-results.md`
5. `docs/architecture/overview.md`
6. `docs/thermo/mvp-model.md`

如果你关心的是“当前阶段做到哪了”，再读：

- `docs/status/current.md`
