# Studio Quick Start

更新时间：2026-09-12

> 本文用于运行与验证当前 MVP 用户路径。内置样例与历史 `official` 称谓表示仓库演示 / 回归案例，运行成功不构成工程精度验证；解释结果前请阅读 [模型与样例边界](../thermo/mvp-model.md)。

## 目的

本文档面向第一次进入仓库、想直接体验 `RadishFlow Studio` 当前主工作台流程的读者。

它回答的是：

- 当前 Studio 已经能做什么
- 如何从历史便携 staging 或开发态启动 Studio
- 第一次建议打开哪个示例
- 接下来应该看哪些文档

它不替代架构文档，也不展开未来规划。

## 当前能做什么

截至 2026-06-10，Studio 当前已经具备以下主路径能力：

- 启动后默认进入中文 Home Dashboard，可从 `开始 / 最近项目 / 示例项目 / 环境 / 消息` 分区判断从哪里开始
- 新建未命名空白项目后先进入独立 `物性` 页，并从受控内置列表显式选择 `二元烃 Lite` 与 `methane / ethane`，再进入最短建模路径；项目文件仍保存稳定 id `binary-hydrocarbon-lite-v1`
- 打开已有 `*.rfproj.json` 项目
- 通过首页 `新建项目`、`打开项目`、`打开示例项目` 或进入工作台后的 `文件` 菜单切换项目；本次会话已有当前工作区时，Home 左侧开始区会显示 `返回工作区`
- 通过首页 `创建 Mixer-Flash 小案例` 或 `创建 Heater-Flash 小案例` 从空白项目进入对应作者路径，并在左侧 `模块` 面板查看任务清单
- 最近项目、当前工作区和示例项目使用统一 case tile 展示：tile 包含轻量流程缩影、路径 / 来源、可读物性包 label、组分摘要和 `Ready / Current / Missing file` 状态；当前工作区 tile 只从当前 `workspace_document` 派生，不把未保存项目写入 recent projects；单击选择，双击打开，文件缺失时只降级对应 tile 状态，不阻断首页
- 进入项目后顶部导航收敛为 `文件 / 主页 / 物性 / 流程图 / 运行 / 结果 / 工具 / 设置`；新建、打开、保存和另存为进入 `文件`，命令面板、Commands 面板和逻辑窗口进入 `工具`，语言进入 `设置`
- 运行仓库内或便携 staging 内的 official hydrocarbon 正向示例 flowsheet
- 在独立 `物性` 页维护当前受控物性包和项目组分；进入 `流程图` 后使用左侧 `模块 / 项目`、中央 `Canvas`、右侧 `检查器 / 模块设置 / 模块结果` 和底部 `消息 / 运行日志 / 收敛 / 建议 / 诊断 / 结果表` 完成当前 MVP 建模与结果核对工作流；`模块` 面板已按 `流股源 / 调节单元 / 汇合与分离` 组织受控单元并支持本地筛选，`项目` 面板已按 `项目输入 / 示例入口 / 对象树 / 审阅状态` 分区，中央 Canvas 保留 `画布状态`、工具条、选择、视口、图例、画布实体和受控建议，不再重复项目对象树
- `物性 / 流程图 / 运行 / 结果` screen 会在顶部导航下方显示上下文工具栏，只渲染已有 presentation / command / state 中可用的入口和状态；`物性` 工具栏会用同一 Property page DTO 显示可读 package label、项目组分和进入流程图建模 readiness
- 在当前 `SolveSnapshot` 内切换 stream-centric / unit-centric / comparison 三类结果审阅面
- 选中单元时，右侧 `检查器` 的单元区域窄口径消费 `Module Settings` presentation：只显示正式 active inspector 来源的参数字段、端口、连接动作、诊断动作和帮助空状态
- 选中单元并存在当前 revision 的最新结果时，右侧 `模块结果` 窄口径消费 `Module Results` presentation：显示该单元 latest result、consumed / produced stream、关联步骤和诊断；若结果过期，不继续渲染旧单元结果
- 复制当前 `SolveSnapshot` 文本，或导出当前快照为轻量 `.txt`；导出文本包含 `Streams / Review / Units / Steps / Diagnostics`
- 通过 `检查`、`诊断目标`、结果选择项和命令入口在流股、单元、步骤和当前检查器之间定位同一份结果
- 在流股检查器中编辑流股基础字段与组成草稿，并显式提交、归一化或丢弃
- 在单元检查器 / Module Settings 中编辑首批关键单元参数：`Feed` 的 source temperature / pressure、`Heater / Cooler` 的 outlet temperature / outlet pressure、`Mixer / Valve` 的 outlet pressure 和 `Flash Drum` 的 flash temperature / flash pressure
- `流程图` / `运行` 上下文工具栏、Run Panel `Resume`、`F5 / Shift+F5`、命令面板、AppHost / StudioGuiDriver / StudioGuiHost command registry 等正式入口共享同一层建模输入 readiness；建模输入缺失不会因入口不同绕过诊断
- 成功运行后若项目文档继续编辑，旧 `SolveSnapshot` 会标为过期；Result Inspector、底部结果表、Results commands、复制 / 导出和 `Review` 摘要只继续消费当前 revision 的最新快照
- 选中物料流股后，可通过 Canvas / Inspector 的 `Disconnect stream` 解除端口绑定并保留流股规格，或通过 `Delete stream` 解除绑定后删除错误流股；单端流股还可在唯一且不会成环的候选存在时执行受控 `Reconnect stream`
- 执行基础 `undo / redo`
- 保存当前项目，或通过顶部 `文件` 菜单中的 `另存为...` / 未命名项目首次 `保存` 到新路径
- 关闭窗口或在 macOS 使用 `Cmd+Q` 退出时，若当前工作区有未保存变更，会进入与窗口关闭按钮相同的保存 / 舍弃 / 取消确认路径
- 保存并恢复 Canvas placement / viewport sidecar：`<project>.rfstudio-layout.json`；当前可拖动单元位置、平移 viewport，并用 `Fit to content` 重新居中
- Studio shell 使用随应用打包的 `InterVariable` 与 `SourceHanSansSC` 字体资源显示中文 UI，不依赖操作系统中某个固定 CJK 字体名称
- 默认隐藏低频命令大全；需要完整命令列表时可从顶部 `工具` 或命令面板入口展开

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

如果你拿到的是历史内部便携 staging，先解压或进入 staging 目录。下面路径只作为历史示例，不代表当前正式版本节点：

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

当前 Studio 会优先从 exe 同目录发现内置示例；如果不是从便携 staging 启动，则回退到仓库内 `examples/flowsheets`。

说明：

- 这是 Windows 内部便携 staging 形态，不是安装器、正式 demo 或 release 节点
- 不会执行 COM 注册、PME 自动化、Windows Registry 写入或第三方 CAPE-OPEN 模型加载
- 历史 staging 的 `PACKAGE-MANIFEST.txt` 可能记录 `version=v26.5.1-dev`、`gitCommit=7479e82`、`gitDirty=false`；这些字段只说明当时 staging 的构建信息，不等同于当前正式 tag

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
- macOS 开发态使用 `eframe/wgpu` + Metal backend，避免系统 OpenGL loader 探测较新 `gl*` 符号时刷出无关 `dlsym` 日志。
- macOS 直接运行 `target/debug/radishflow-studio` 或从 RustRover debug 裸二进制时，终端可能输出 `com.apple.linkd.autoShortcut`、App Intents、`[WindowTab] Cannot index window tabs due to missing main bundle identifier` 或 task name port right 相关系统日志。只要窗口正常打开、退出码正常、功能路径可用，当前将其视为 macOS 非 `.app` bundle 开发态噪声；真正收口点放到未来 macOS `.app`、`Info.plist`、bundle identifier、签名或打包流程。

## 第一次建议体验什么

第一次建议直接打开以下 official hydrocarbon 正向示例之一：

- `examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json`
- `examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json`

如果只想先走最短路径，优先第一个示例。它覆盖：

- 一个 `Feed`
- 一个 `Heater`
- 一个 `Flash Drum`
- 一条单入口、双产物流股的短链路求解流程

详细操作步骤见：

- `docs/guides/run-first-flowsheet.md`

如果内置示例已经跑通，下一步建议从首页点击 `创建 Mixer-Flash 小案例` 或 `创建 Heater-Flash 小案例`，按 `docs/guides/author-small-cases.md` 从空白项目复现小案例：放置单元、接受 suggestion、提交单元参数、运行、保存重开、重跑并导出当前结果。这些作者入口只打开空白项目和任务清单，不是自由连线、自动布线或完整项目向导。

## 启动首页

启动后，第一视野是 Home Dashboard，而不是直接进入某个项目的画布。首页的稳定分区如下：

- `开始`：`新建项目`、`创建 Mixer-Flash 小案例`、`创建 Heater-Flash 小案例`、`打开项目`、`打开示例项目`
- `最近项目`：显示当前工作区和最近项目 tile、流程缩影、路径摘要、物性包 label、组分和就绪 / 当前 / 缺失状态；单击选择，双击打开。当前工作区 tile 是返回当前文档的入口，未保存项目不会因此写入 recent projects，保存过的当前项目也不会通过 recent path 重新打开自身
- `示例项目`：显示内置示例 tile、流程缩影、流程摘要、组分和物性包 label；单击选择，双击打开
- `环境`：显示客户端、服务端和设备三组摘要
- `消息`：显示登录、示例目录、物性包缓存等可行动摘要

工程术语、文件名、稳定包 id 和路径会保留原文；用户动作、状态和环境字段默认使用中文。首页不承载流程图编辑，打开项目或示例后才进入工作台。

Home 的 recent / current / example case tile 显示的是从当前 document / builtin package choice 映射出的可读 label，例如中文界面显示 `二元烃 Lite`；空白项目仍显示 `未选择`。`binary-hydrocarbon-lite-v1` 这类稳定 id 继续只用于项目文件、command id、运行请求和内部状态边界，不作为首页主要展示文本。

首版 demo 前，Home / Workbench 的默认示例选择器只暴露四条 official hydrocarbon 演示路径：`Feed -> Heater/Cooler/Valve -> Flash Drum` 与 `Feed + Feed -> Mixer -> Flash Drum`。仓库和便携 staging 内仍可能附带 synthetic 或 PME 验证样例文件，但这些文件主要服务回归或外部验证，不作为首页高频演示入口。

若当前工作区存在未保存变更，首页的 `新建项目`、两个小案例作者入口、`打开项目`、`打开示例项目`、`返回工作区`、case tile 双击以及工作台顶部的项目切换入口都会先进入显式确认流程；继续后才丢弃当前未保存内容，取消则保持当前项目不变。

小案例作者入口当前支持：

- `Mixer-Flash`：`Feed + Feed -> Mixer -> Flash Drum`
- `Heater-Flash`：`Feed -> Heater -> Flash Drum`

入口只决定左侧 `模块` 面板展示哪一条任务清单。清单状态从当前 canvas unit / stream / solve snapshot 推导，不会自动补单元、自动连线或修改项目文档。

## 工作台顶部导航

进入项目后，顶部第一行展示应用、当前项目和状态 chip；第二行固定为八个主入口：

- `文件`：`新建空白`、`打开示例`、`打开项目...`、`保存`、`另存为...`
- `主页`：返回启动首页
- `物性`：进入独立物性页，查看并选择当前受控内置物性包和项目组分
- `流程图`：进入建模工作台，查看 Canvas、放置入口、检查器、模块设置、模块结果和底部运行信息
- `运行`：进入运行 screen，查看运行控制、运行日志、收敛、suggestion 和诊断相关入口
- `结果`：进入结果 screen，查看当前 / 过期 / 缺失 `SolveSnapshot` 状态、结果聚焦入口、右侧模块结果和底部结果表入口
- `工具`：命令面板、Commands 面板显示 / 隐藏、逻辑窗口入口
- `设置`：当前只放语言切换

当前 screen 是 `物性 / 流程图 / 运行 / 结果` 时，顶部导航下方会显示对应上下文工具栏。它们只消费已有 DTO、command registry、Run Panel state、Module Results、结果表状态和 shell layout state，不新增第二套项目、物性、运行、结果或诊断真相源。普通空白项目在 package 和至少一个项目组分选齐前，顶部 `流程图` 入口会保持不可用；可用性和 hover 说明来自同一个 `StudioGuiWindowPropertyPageModel`。

新建普通空白项目后会先进入独立 `物性` 页；打开项目或示例后，左侧 `项目` 面板会显示当前 `物性包` 和 `项目组分` 摘要。独立 `物性` 页提供同一组受控项目级输入入口、本地包摘要和 `进入流程图建模` 状态。MVP β 建模输入 v0 只提供受控内置 `二元烃 Lite` 物性包和 methane / ethane 组分目录；空白项目初始不预选，选择后分别写入稳定 id `Flowsheet.thermo.property_package_id = "binary-hydrocarbon-lite-v1"` 与 `Flowsheet.components`，并决定 Stream Inspector 中 Feed composition 可添加的组分。

独立 `物性` 页、Property toolbar 和左侧 `项目输入` 都以可读 label 展示当前选择。若需要排查文件内容，再到 `*.rfproj.json` 中查看稳定 id；不要把运行结果中的收敛状态当作第二套物性包状态来源。

命令面板默认不占据第一视野。它不是功能移除，而是把低频、调试型入口和窗口工具从主工作流移到 `工具`。

## 从空白项目开始

如果不想先打开示例，可以直接走当前最小空白建模路径：

1. 在首页点击 `新建项目`，或进入工作区后从顶部 `文件` 菜单点击 `新建空白`。
2. Studio 会进入未命名空白项目的独立 `物性` 页。选择 `二元烃 Lite`，再选择 methane / ethane 项目组分；保存到项目文件时对应稳定 id 是 `binary-hydrocarbon-lite-v1`。
3. 当 `物性` 上下文工具栏中的 `进入流程图建模` 变为可用时，点击它，或点击顶部 `流程图`，进入建模工作台。空 flowsheet 会把建模入口聚焦到左侧 `模块` 面板；左侧 `项目` 面板继续扫读项目输入、对象树和审阅状态。
4. 在左侧 `模块` 中用 `放置进料`、`放置闪蒸罐` 或 `放置加热器 / 放置冷却器 / 放置阀门 / 放置混合器` 开始放置单元。
5. 在 Canvas 中点击落点提交当前放置意图。
6. 使用 Canvas 上的 `Connect stream` / `连接流股` 或 `Create stream` / `创建流股` suggestion 补齐端口绑定和必要 outlet stream。`Heater / Cooler / Valve`、`Mixer` 和 `Flash Drum` 的 outlet stream 建议会等必要 inlet 绑定后才出现。
7. 在左侧 `项目` 的 `对象树` 中选择 stream / unit，或直接点击 Canvas 中的单元 / 物料线，右侧 `检查器` 会切到对应对象。
8. 在流股检查器中编辑 `T / P / F` 和组成草稿；字段提交、全部应用、组成归一化都是显式动作。
9. 选中 `Feed / Heater / Cooler / Mixer / Valve / Flash Drum` 时，可在单元检查器 / Module Settings 中编辑已暴露的 source temperature、source pressure、outlet temperature、outlet pressure、flash temperature 或 flash pressure 字段；字段会显示 SI 单位和约束提示，提交后写回项目参数，并同步对应 outlet stream 模板。若字段当前显示的是 outlet stream 模板值或内置默认值，但对应 unit parameter 尚未显式提交，即使输入值与显示值相同，也应提交一次，让正式 `SetUnitParameter` 写入项目文档。字段来源与 readiness 关系见 `docs/reference/units-and-conventions.md`。
10. 若流股错连或漏连，先选中该 material stream，再使用 `Disconnect stream`、`Disconnect source`、`Disconnect sink`、`Reconnect stream` 或 `Delete stream` 这组受控恢复动作；`Reconnect stream` 只在单端唯一候选且不会形成 unit dependency cycle 时可用。
11. 在 `流程图` 或 `运行` 上下文工具栏点击 `运行当前流程`。若 Feed source stream 的 `T / P / F / z`、项目组分引用、composition 归一或必要单元参数尚未就绪，Studio 会先显示“模型输入未完成”并聚焦到对应 stream / unit；输入补齐后，结果只从当前 revision 的最新 `SolveSnapshot` 展示到顶部 `结果` screen、右侧 `模块结果`、底部 `结果表`、Results commands 和轻量导出的 `Review` 摘要。正式物性包解析失败、结构性连接错误、拓扑错误和求解阶段参数失败继续由 Run Panel 诊断 / recovery 承载。

当前连接仍通过本地 suggestion 和正式 `DocumentCommand` 完成，不是自由拉线编辑器；单元参数编辑也仍限制在上述 MVP 已暴露字段，不等同于完整单元参数表。Module Settings 的帮助区目前只表达“暂无正式模块帮助命令”，不会临时伪造 help action。

## 单元参数与连接诊断

当前首批单元参数字段只覆盖最短建模路径中的高频项：

- `Feed`：source outlet temperature，单位 K；source outlet pressure，单位 Pa；提交后同步 Feed outlet stream 模板
- `Heater / Cooler`：`outlet temperature`，单位 K
- `Heater / Cooler`：`outlet pressure`，单位 Pa；不能高于已连接 inlet pressure
- `Mixer`：`outlet pressure`，单位 Pa；不能高于两股已连接 inlet pressure 的较低值
- `Valve`：`outlet pressure`，单位 Pa；不能高于已连接 inlet pressure
- `Flash Drum`：`flash temperature`，单位 K；`flash pressure`，单位 Pa；提交后同步 liquid / vapor 两个出口流股模板，并作为 TP Flash 输入参与求解

`Mixer / Heater / Cooler / Valve` 的 outlet pressure 若高于已连接 inlet pressure 约束，会在单元检查器草稿态直接标记为无效，保持草稿、不写回项目文档，也不会同步 outlet stream 模板。Flash Drum 的 flash temperature / flash pressure 当前只要求正有限 SI 值，不施加 inlet pressure 上限。若旧项目或外部编辑已经把越界参数写入文档，运行会产生 `solver.step.parameter` 诊断，并把 failure detail、端口 attention 和 recovery action 指向相关 unit / port / stream。

运行前 readiness 不再使用 `Mixer-Flash` 或 `Heater-Flash` 小案例清单作为 gate。普通空白项目按当前 `Flowsheet` 判断：Feed source stream 必须有正有限温度、压力和摩尔流量，composition 必须引用已选择项目组分并归一；Heater / Cooler / Flash Drum 必须提交出口温度和压力，Mixer / Valve 必须提交出口压力。缺 material port 绑定、坏 stream reference、重复 source / sink、orphan stream、cycle 等结构性问题仍交给正式 Run Panel 诊断 / recovery；物性包缺失也继续交给正式运行命令的 package resolution 诊断。

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

- 顶部导航中的 `文件 / 主页 / 物性 / 流程图 / 运行 / 结果 / 工具 / 设置` 八个主入口
- `物性 / 流程图 / 运行 / 结果` screen 下方对应的上下文工具栏；这些工具栏只显示当前已有可用命令和状态
- 顶部当前项目标题、运行状态、pending 状态和未保存提示；完整路径优先进入摘要、tooltip 或详情区域
- 左侧 `模块 / 项目`，分别用于按分类筛选和放置 MVP 内建单元、查看作者任务清单，以及扫读 `项目输入 / 示例入口 / 对象树 / 审阅状态`
- Canvas 上的 `画布状态`、工具条、单元、物料线和当前关注对象；项目对象树仍在左侧 `项目`
- 右侧 `检查器 / 模块设置 / 模块结果` tabs，其中 `检查器` 负责当前对象输入、端口和关联诊断；`模块设置` 窄口径消费正式 Module Settings DTO，不渲染 latest-result 内容；`模块结果` 负责只读结果审阅和选中单元的 Module Results 摘要
- 底部 `消息 / 运行日志 / 收敛 / 建议 / 诊断 / 结果表` drawer，其中结果表只读消费当前 `SolveSnapshot`，默认消息区比结果 / 诊断页更紧凑
- `诊断目标` 中可直接定位的流股 / 单元结果目标
- 当前快照导出中的 `Review` 摘要：按 source / intermediate / terminal streams 与 latest unit results 快速核对同一条结果链路

如果运行成功，Studio 会把结果入口聚焦到顶部 `结果` screen、右侧 `模块结果` 和底部 `结果表`。选中单元时，`模块结果` 会先展示当前单元的 Module Results 摘要，再展示同一份快照的流股 / 步骤 / 诊断结果。`Flash Drum` 相关结果当前应能进一步展示：

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
- 运行前若文档组成缺失、引用未选择项目组分、数值无效或未归一化，应先阻断并显示诊断，不做隐式差值补偿

## 运行反馈和退出

开发态启动时，Studio 会向 stderr 输出带 `[radishflow-studio]` 前缀的用户操作与求解审计线。这些输出服务 smoke 和排查，不代表正式 telemetry 或长期审计接口。

如果 GUI 回调发生内部 panic，当前壳层会降级到错误页，并提示查看 stderr。若只是关闭最后一个 Studio 窗口，当前预期是自然退出进程，不应短暂闪回默认 Commands 左栏，也不应留下黑屏但进程不退出的状态。macOS `Cmd+Q` 与窗口关闭按钮走同一条退出确认路径：脏工作区先选择保存并关闭、舍弃并关闭或取消关闭；取消、保存失败或另存为取消都会让当前窗口保持打开。

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
