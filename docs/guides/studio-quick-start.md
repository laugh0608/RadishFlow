# Studio Quick Start

更新时间：2026-05-16

## 目的

本文档面向第一次进入仓库、想直接体验 `RadishFlow Studio` 当前最小工作台闭环的读者。

它回答的是：

- 当前 Studio 已经能做什么
- 如何在开发态启动 Studio
- 第一次建议打开哪个示例
- 接下来应该看哪些文档

它不替代架构文档，也不展开未来规划。

## 当前能做什么

截至 2026-05-16，Studio 当前已经具备以下最小闭环：

- 启动后默认进入中文 Home Dashboard，可从 `开始 / 最近 Case / 示例 Case / 环境 / 消息` 分区判断从哪里开始
- 新建未命名空白项目，并用 MVP 默认 `methane / ethane` 二元体系进入最短建模路径
- 打开已有 `*.rfproj.json` 项目
- 通过首页 `打开示例 Case`、`打开 Case`、`继续上次 Case` 或进入工作台后的顶部主路径切换项目
- 进入 case 后在顶部主路径直接使用 `Home / 打开示例 / 新建空白 / 打开项目... / 运行 / 保存 / 另存为... / 视图`
- 运行仓库内的最小正向示例 flowsheet
- 在左侧 `项目 / 示例项目 / 放置`、中央 `Canvas`、右侧 `检查器 / 结果 / 运行 / 物性包` 和底部 `消息 / 运行日志 / 结果表 / 诊断` 中完成当前 MVP α 工作流
- 在当前 `SolveSnapshot` 内切换 stream-centric / unit-centric / comparison 三类结果审阅面
- 通过 `Inspect` / `DiagnosticTargets` 在 stream、unit、step 和 Active Inspector 之间定位同一份结果
- 在 Stream Inspector 中编辑流股基础字段与组成草稿，并显式提交、归一化或丢弃
- 执行基础 `undo / redo`
- 保存当前项目，或通过顶部 `另存为...` / 未命名项目首次 `保存` 到新路径
- 保存并恢复 Canvas placement sidecar：`<project>.rfstudio-layout.json`
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
- 结果报表导出
- 跨快照历史对比系统
- 完整 CAPE-OPEN 第三方模型加载

因此，第一次体验应优先基于仓库自带示例，而不是把它当成已经收口的通用生产工具。

## 启动方式

如需先做仓库级验证，执行：

```powershell
pwsh ./scripts/check-repo.ps1
```

开发态启动 Studio：

```powershell
cargo run -p radishflow-studio
```

说明：

- 这是长时间运行的桌面 UI 命令
- 当前文档只描述开发态启动方式，不代表已经存在正式安装包
- Windows 当前已接入原生打开/另存为选择器；其他平台的文件工作流暂不承诺同等完成度

## 第一次建议体验什么

第一次建议直接打开以下正向示例之一：

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

启动后，第一视野是 Home Dashboard，而不是直接进入某个 case 的画布。首页的稳定分区如下：

- `开始`：`继续上次 Case`、`新建空白 Case`、`打开 Case`、`打开示例 Case`
- `最近 Case`：显示最近项目、路径摘要、物性包和就绪 / 缺失状态
- `示例 Case`：显示内置示例、流程摘要、组分、物性包和打开入口
- `环境`：显示客户端、服务端和设备三组摘要
- `消息`：显示登录、示例目录、物性包缓存等可行动摘要

工程术语、文件名、包名和路径会保留原文；用户动作、状态和环境字段默认使用中文。首页不承载流程图编辑，打开 case 或示例后才进入工作台。

## 工作台主路径

进入 case 后，顶部第一行展示应用、当前项目和状态 chip；第二行提供当前主路径：

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

1. 在首页点击 `新建空白 Case`，或进入工作台后点击顶部 `新建空白`。
2. 在左侧切到 `放置`，用 `放置 Feed`、`放置 Flash Drum` 或 `放置 Heater / Cooler / Valve / Mixer` 开始放置单元。
3. 在 Canvas 中点击落点提交当前放置意图。
4. 使用 Canvas 上的 `Connect` / `连接` suggestion 补齐端口绑定和必要 outlet stream。
5. 在左侧 `项目` 或 Canvas 对象列表中选择 stream / unit，右侧 `检查器` 会切到对应对象。
6. 在 Stream Inspector 中编辑 `T / P / F` 和组成草稿；字段提交、全部应用、组成归一化都是显式动作。
7. 点击顶部 `运行`，结果只从最新 `SolveSnapshot` 展示到右侧 `结果` 和底部 `结果表`。

当前连接仍通过本地 suggestion 和正式 `DocumentCommand::ConnectPorts` 完成，不是自由拉线编辑器；单元参数编辑也仍限制在 MVP 已暴露的 Inspector 字段和端口/结果只读信息内。

## 启动后应该看到什么

成功打开示例后，当前 Studio 应至少能让你扫读到以下信息：

- 顶部主路径中的 Home、打开示例、新建空白、打开项目、运行、保存、另存为和视图入口
- 顶部当前项目标题、运行状态、pending 状态和未保存提示；完整路径在第二行摘要中显示
- 左侧 `项目 / 示例项目 / 放置`，分别用于项目树扫读、示例入口和放置 MVP 内建单元
- Canvas 上的单元、物流线和当前关注对象
- 右侧 `检查器 / 结果 / 运行 / 物性包` tabs，其中 `检查器` 负责当前对象参数、组成、端口和关联结果，`结果` 负责只读结果审阅，`物性包` 负责本地包和同步状态摘要
- 底部 `消息 / 运行日志 / 结果表 / 诊断` drawer，其中结果表只读消费当前 `SolveSnapshot`，默认消息区比结果 / 诊断页更紧凑
- `DiagnosticTargets` 中可直接定位的 stream / unit 结果目标

如果运行成功，`Flash Drum` 相关结果当前应能进一步展示：

- `phase_region`
- `bubble_dew_window`
- `liquid / vapor / overall` 相结果
- 各相摩尔流量与 molar enthalpy

## Stream Inspector 组成编辑

当前 Stream Inspector 的组成编辑遵循显式提交原则：

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
