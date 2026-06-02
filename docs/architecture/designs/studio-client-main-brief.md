# Studio Client Main Design Brief

更新时间：2026-06-02

## 用途

用途：为 P0 `studio-client-main.pen` 信息架构稿提供可评审的文字 brief，确保 Pencil 设计稿创建与评审时已经明确页面、状态来源、主工作流和暂不纳入项。  
读者：准备绘制或评审 `docs/architecture/designs/studio-client-main.pen` 的设计协作者。  
不包含：`.pen` 设计稿本体、视觉 token、实现代码、完整控件规格或完整交互动画。

本 brief 对应 `docs/architecture/studio-ui-topic-plan.md` 中的 P0 Studio 客户端本体端点。`.pen` 文件仍应通过 Pencil MCP 工具创建、维护和验证；本文件只作为设计前置材料和评审索引。

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

## 全局线框规格

首版线框稿优先表达信息架构，采用低保真浅色框架，不做最终视觉：

- Frame 背景：`#F5F7FA`。
- 主要面板背景：`#FFFFFF`。
- 分隔线：`#D8DEE8`。
- 主强调：克制蓝色，仅用于主动作和运行状态。
- 字体：系统 sans-serif；中文标签可用现有 Studio 中文文案。
- 圆角：常规面板 6-8 px，按钮 6 px。
- 阴影：不在主面板使用，弹层或菜单后续再定。

桌面 frame 使用 `1440 x 960`，建议固定区域：

| 区域 | X | Y | W | H | 说明 |
| --- | ---: | ---: | ---: | ---: | --- |
| App Bar | 0 | 0 | 1440 | 56 | 全局标题、项目状态、运行主命令 |
| Left Rail | 0 | 56 | 288 | 704 | Home start / Workbench project and palette |
| Main Stage | 288 | 56 | 816 | 704 | Home 列表或 Workbench Canvas |
| Right Rail | 1104 | 56 | 336 | 704 | Environment / Inspector / Result |
| Bottom Drawer | 0 | 760 | 1440 | 200 | Messages / Diagnostics / Results Table |

线框稿中应保留 frame 名称、区域名和状态 chip，避免放长篇说明。具体字段解释进入 brief 或后续评审记录，不塞进画布。

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

### 线框内容

| 区域 | 内容 |
| --- | --- |
| App Bar | `RadishFlow Studio`、`development build`、`Local ready`、`Signed out`、`Settings` |
| Left Rail | 主按钮 `新建项目`；次级按钮 `打开项目`、`打开示例项目`；低权重链接 `创建 Mixer-Flash 小案例`、`创建 Heater-Flash 小案例` |
| Main Stage | 上半区 `最近项目` 列表；下半区 `示例项目` 列表；每行包含名称、路径 / 来源、package、状态 chip |
| Right Rail | `Client`、`Server`、`Device` 三个状态 section；每个 section 2-3 行摘要 |
| Bottom Drawer | Messages 列表，包含 severity、domain、summary、action |

示例行建议：

- Recent：`Blank Project` / `binary-hydrocarbon-lite-v1` / `Modified`。
- Example：`Feed -> Cooler -> Flash Drum` / `methane, ethane` / `Ready`。
- Message：`PACKAGE` / `内置物性包缓存可用` / `View packages`。

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

### 线框内容

| 区域 | 内容 |
| --- | --- |
| App Bar | 项目名 `Blank Project`、保存状态 `Unsaved`、`运行`、`保存`、run status `Ready to run` |
| Left Rail | Tabs：`项目` / `放置`；项目 tab 显示 package、components；放置 tab 显示 Feed、Mixer、Cooler、Valve、Flash Drum |
| Main Stage | Canvas 网格；单元块 Feed、Cooler、Flash Drum；流股线 `stream-feed-1-outlet`、`stream-cooler-1-outlet`、liquid / vapor outlets；suggestion chip |
| Right Rail | Inspector target：`Unit cooler-1`；字段 `outlet_temperature_k`、`outlet_pressure_pa`；Ports；Actions |
| Bottom Drawer | Messages tab active；短行显示最近提交、suggestion 接受、保存状态 |

Canvas 示例结构：

```text
Feed ── stream-feed-1-outlet ── Cooler ── stream-cooler-1-outlet ── Flash Drum
                                                           ├─ stream-flash-1-liquid
                                                           └─ stream-flash-1-vapor
```

Inspector 字段行应包含：

| 字段 | 显示 |
| --- | --- |
| `outlet_temperature_k` | label、input、unit `K`、state `Draft` / `Synced`、commit icon |
| `outlet_pressure_pa` | label、input、unit `Pa`、constraint `<= inlet pressure`、commit icon |
| `Ports` | inlet connected、outlet connected |
| `Related` | latest diagnostics empty、result unavailable before run |

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

### 线框内容

| 区域 | 内容 |
| --- | --- |
| App Bar | run status `Blocked`；primary action 保持 `运行`，状态摘要说明 `Modeling inputs not ready` |
| Left Rail | 保持当前项目 / 放置上下文，不跳转到 checklist |
| Main Stage | Canvas 中目标对象显示 attention outline，例如 `flash-1` |
| Right Rail | Inspector 聚焦 `Unit flash-1`；缺失字段 `outlet_temperature_k` 显示可提交 displayed default |
| Bottom Drawer | Messages active；notice 标题 `模型输入未完成`；detail 指向 `Flash Drum 出口温度` |

Readiness notice 线框文案只保留短句：

```text
模型输入未完成
Flash Drum flash-1 需要提交出口温度。
```

字段提交后，frame 注释应说明下一状态推进到 `outlet_pressure_pa`，但不需要再画一个额外 frame。

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

### 线框内容

| 区域 | 内容 |
| --- | --- |
| App Bar | run status `Converged`、snapshot id short、`Run again`、`Save` |
| Left Rail | 项目对象列表可继续导航；选中 stream / unit 同步 Canvas focus |
| Main Stage | Canvas 显示结果 badge：source、intermediate、terminal；不把完整结果数字塞进画布 |
| Right Rail | Result Inspector；stream selector、unit selector、summary rows、phase rows |
| Bottom Drawer | Results Table active；Streams / Units / Diagnostics tabs；Export current snapshot |

Result Inspector 示例内容：

| 区块 | 内容 |
| --- | --- |
| Selected stream | `stream-cooler-1-outlet` |
| Summary | `T K`、`P Pa`、`F mol/s`、`H J/mol` |
| Composition | methane、ethane |
| Phase | overall、liquid / vapor when present |
| Unit references | consumed by `flash-1`、produced by `cooler-1` |

Stale notice 可作为 Results frame 内的右栏局部 variant 标注，不需要新增主 frame：

```text
结果已过期
当前项目已编辑，请重新运行以查看最新结果。
```

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
