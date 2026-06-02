# Studio Client Main Design Brief

更新时间：2026-06-02

## 用途

用途：为 P0 `studio-client-main.pen` 信息架构稿提供可评审的文字 brief，确保 Pencil 设计稿创建前已经明确页面、状态来源、主工作流和暂不纳入项。  
读者：准备绘制或评审 `docs/architecture/designs/studio-client-main.pen` 的设计协作者。  
不包含：`.pen` 设计稿本体、视觉 token、实现代码、完整控件规格或完整交互动画。

本 brief 对应 `docs/architecture/studio-ui-topic-plan.md` 中的 P0 Studio 客户端本体端点。`.pen` 文件仍应通过 Pencil MCP 工具创建和维护；本文件只作为设计前置材料和评审索引。

## 设计目标

`studio-client-main.pen` 第一版只解决 Studio 客户端本体的信息架构，不做视觉精修：

- 用户能从 Home 判断环境是否可用，并进入普通空白项目或示例项目。
- 用户能在 Workbench 中看懂项目、画布、检查器、结果和诊断分别在哪里。
- 用户能完成当前已通过的四条普通空白项目主路径。
- readiness、Run Panel 诊断、旧结果失效和最新结果审阅有清晰但克制的状态表达。
- 所有状态都能映射到既有 presentation / command / state 模型。

## 设计画板建议

首版 `.pen` 建议包含 4 个主 frame：

| Frame | 建议尺寸 | 目的 |
| --- | ---: | --- |
| `Home - Ready` | 1440 x 960 | 启动首页，展示开始入口、最近项目、示例项目和环境状态 |
| `Workbench - Modeling` | 1440 x 960 | 普通空白项目建模中，展示 Project / Palette、Canvas、Inspector 和底部消息 |
| `Workbench - Readiness` | 1440 x 960 | 缺建模输入时的阻断状态，展示 readiness notice、目标对象聚焦和字段提交路径 |
| `Workbench - Results` | 1440 x 960 | 运行收敛后结果审阅，展示 Result Inspector、底部结果表、轻量导出和 stale notice 入口 |

P0 不拆单独移动端 frame，不画控制面后台 frame。

## Home - Ready

### 布局

- 顶部 App Bar：应用名、开发构建标识、登录状态、服务端状态、设置入口。
- 左侧 Start：`新建项目`、`打开项目`、`打开示例项目`，小案例作者入口降级为次级入口。
- 中央 Recent / Examples：最近项目列表和示例项目列表，以列表扫读为主，不使用营销卡片。
- 右侧 Environment：Client、Server、Device 三组状态摘要。
- 底部 Messages：最多 3-5 条可行动环境消息。

### 状态来源

| UI | 来源 |
| --- | --- |
| 最近项目 | app preferences / MRU |
| 示例项目 | `examples/flowsheets` discovery |
| 登录状态 | auth / entitlement state |
| local ready / cache ready | runtime environment probe |
| Messages | application event / environment summary |

### 不画

- 完整命令面板。
- 完整控制台导航。
- 完整小案例 checklist。
- 未来 release / package / installer 状态。

## Workbench - Modeling

### 布局

- 顶部命令带：项目标题、保存状态、`运行`、`保存`、`打开`、运行状态摘要。
- 左栏：`项目` 与 `放置` 两个主 tab。项目 tab 负责 package / components；放置 tab 负责当前内置单元。
- 中央 Canvas：主舞台，显示单元、流股、端口状态和受控 suggestion。
- 右栏 Inspector：当前 stream / unit 的字段、端口和关联动作。
- 底部 Drawer：Messages / Diagnostics / Results Table tabs，建模中默认显示 Messages。

### 必须覆盖的对象

- Feed source stream：T / P / F / composition。
- Cooler / Valve intermediate stream：outlet T / P 或 P。
- Mixer：两个 inlet、outlet pressure。
- Flash Drum：flash temperature / pressure、liquid / vapor outlets。

### 视觉和交互原则

- Canvas 占主面积，侧栏服务于建模，不抢主舞台。
- suggestion 用低噪声状态，不把长说明放在画布内。
- Inspector 字段必须显示单位、草稿状态、提交命令和约束提示。
- 运行状态和保存状态放在顶部，不散落到多个面板。

## Workbench - Readiness

### 场景

用户点击 `运行`，但缺少确定的建模输入：

- 缺项目组分。
- 缺 Feed composition。
- 缺 Feed source T / P / F。
- 缺必要单元参数。
- composition 未归一。

### 表达

- 顶部运行状态显示 pending / blocked。
- 右侧 Inspector 聚焦真实缺口对象，例如 `flash-1` 或 `stream-feed-1-outlet`。
- 底部 Messages 显示 “模型输入未完成” 的短 notice。
- 字段区显示可提交的 draft / displayed default，而不是只给文本提示。

### 边界

不把结构性连接、拓扑、非法旧项目或求解阶段参数失败画成 readiness。它们继续走正式 Run Panel 诊断 / recovery。

## Workbench - Results

### 运行成功

运行收敛后：

- 顶部运行状态显示 converged，pending reason 清空。
- 右栏切到 Result Inspector 或在 Inspector 中提供清晰结果 tab。
- 底部默认打开 Results Table。
- Results commands 能聚焦 stream / unit。
- 轻量导出入口只消费当前 revision 的 latest `SolveSnapshot`。

### 结果审阅对象

必须能扫读：

- source streams。
- intermediate streams。
- terminal streams。
- unit consumed / produced streams。
- Flash liquid / vapor outlet material balance。
- phase region、bubble / dew window、overall enthalpy。

### 旧结果失效

编辑文档后：

- latest result 区域不继续渲染旧结果。
- stale notice 显示旧 snapshot 和当前 document revision 已不一致。
- 主动作指向 rerun。
- 旧结果不驱动导出或 Result Inspector。

## 命令与状态映射

| 用户动作 | 命令 / 模型边界 |
| --- | --- |
| 新建项目 | app / shell project command，不依赖登录 |
| 放置单元 | canvas placement command |
| 接受 suggestion | local rules suggestion -> document command |
| 编辑字段 | inspector draft update |
| 提交字段 | inspector draft commit command |
| 运行 | `run_panel.run_manual` -> shared readiness -> solve dispatch |
| 保存 | file save command |
| 结果聚焦 | result command / inspector focus command |
| 导出当前快照 | latest current-revision `SolveSnapshot` export |

## 评审检查

创建 `.pen` 前先确认：

- Home、Modeling、Readiness、Results 四个 frame 是否覆盖当前 P0 主路径。
- 每个 visible 状态是否有明确来源。
- 是否避免画出当前暂不纳入项。
- 是否能从 frame 直接看出下一步动作。
- 是否没有把解释文字堆进 Canvas 主舞台。
- 是否保留足够空间给后续单元模块 UI 设计稿对齐。
