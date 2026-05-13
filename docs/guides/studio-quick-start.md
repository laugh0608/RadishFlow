# Studio Quick Start

更新时间：2026-05-13

## 目的

本文档面向第一次进入仓库、想直接体验 `RadishFlow Studio` 当前最小工作台闭环的读者。

它回答的是：

- 当前 Studio 已经能做什么
- 如何在开发态启动 Studio
- 第一次建议打开哪个示例
- 接下来应该看哪些文档

它不替代架构文档，也不展开未来规划。

## 当前能做什么

截至 2026-05-13，Studio 当前已经具备以下最小闭环：

- 打开已有 `*.rfproj.json` 项目
- 通过内置示例入口、最近项目列表、路径输入或 Windows 原生文件选择器切换项目
- 启动后在顶部快速操作区直接使用 `Open Example / Open Project / Run / Save / Commands / Command Palette`
- 运行仓库内的最小正向示例 flowsheet
- 在 Runtime / Result Inspector / Active Inspector 中查看结构化结果、步骤和诊断
- 在当前 `SolveSnapshot` 内切换 stream-centric / unit-centric / comparison 三类结果审阅面
- 通过 `Inspect` / `DiagnosticTargets` 在 stream、unit、step 和 Active Inspector 之间定位同一份结果
- 在 Stream Inspector 中编辑流股基础字段与组成草稿，并显式提交、归一化或丢弃
- 执行基础 `undo / redo`
- 保存当前项目，或 `Save As` 到新路径
- 保存并恢复 Canvas placement sidecar：`<project>.rfstudio-layout.json`
- 默认隐藏低频 Commands 面板；需要完整命令列表时可从顶部 `Commands` 或 `Command Palette` 展开

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

## 顶部快速操作

启动后，第一视野优先看顶部快速操作区，而不是先找完整菜单或 Commands 面板：

- `Open Example`：打开仓库内置正向示例
- `Open Project`：从磁盘选择已有 `*.rfproj.json`
- `Run`：对当前工作区执行一次手动运行；不可用时 hover 会说明原因
- `Save`：保存当前项目
- `Commands`：显示或隐藏低频命令面板
- `Command Palette`：搜索并执行当前可用命令

Commands 面板默认隐藏是当前 MVP α 体验口径的一部分。它不是功能移除，而是把低频和调试型入口从首屏主路径移开。

## 启动后应该看到什么

成功打开示例后，当前 Studio 应至少能让你扫读到以下信息：

- 顶部快速操作区中的打开、运行、保存和命令入口
- 顶部当前项目标题、路径和未保存提示
- Runtime 区域中的运行状态和最近一次结果摘要
- Canvas 上的单元、物流线和当前关注对象
- Result Inspector 中的 stream-centric / unit-centric 结果审阅
- Result Inspector 中当前快照内的 stream comparison 与 `Inspect` 跳转
- Active Inspector 中的对象详情、端口和关联结果
- `DiagnosticTargets` 中可直接定位的 stream / unit 结果目标
- Diagnostics / steps / logs 等运行辅助信息

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
