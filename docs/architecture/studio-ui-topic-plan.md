# Studio UI 专题计划

更新时间：2026-06-02

## 用途

用途：为 Studio UI 专题阶段提供设计前置边界，明确端点清单、优先级、信息架构、主工作流、状态模型和 `.pen` 设计稿规则。  
读者：负责 RadishFlow Studio、单元模块 UI、控制面 UI、移动端或只读视图设计与实现的开发者和设计协作者。  
不包含：具体视觉稿、实现代码、视觉 token 细则、完整主题系统、自由连线编辑器、完整报表系统或发布计划。

本文是专题计划文档。长期视觉规范仍以 `docs/architecture/studio-ui-design-guidelines.md` 和 `docs/architecture/studio-visual-system.md` 为准；外部设计参考仍以 `docs/architecture/ui-inspiration-reference.md` 为准。

## 阶段目标

Studio UI 专题不再处理零散按钮、文案、hover、局部 selector 或一次性 smoke 面板。当前阶段先完成设计前置：

- 明确哪些端点进入首批设计稿，哪些端点只保留边界。
- 明确每个端点的主任务、页面职责、状态来源和禁止提前实现的能力。
- 明确 `.pen` 设计稿目录、命名、评审和同步规则。
- 明确设计稿到代码实现的边界：实现必须继续沿用正式 presentation / command / state 模型。

阶段完成标准：

- 本文档列出的首批端点、暂不纳入项和设计稿规则已经稳定。
- 首批 `.pen` 设计稿可以按同一目录和命名规则创建。
- 每个设计稿开始前都能回答主任务、状态来源、可操作命令和不包含内容。
- 不需要通过新增 UI 代码来证明本阶段完成。

## 端点清单

### 首批设计端点

首批只覆盖会直接影响当前 Studio 主工作流的端点：

| 端点 | 设计稿建议名 | 优先级 | 目标 |
| --- | --- | --- | --- |
| Studio 客户端本体 | `studio-client-main.pen` | P0 | 统一 Home、Workbench、Canvas、Inspector、Result、Package / Auth 的信息架构和主状态表达 |
| 单元模块 UI | `unit-module-panel.pen` | P1 | 统一 Feed、Mixer、Heater / Cooler、Valve、Flash Drum 的参数、端口、结果和诊断界面规则 |
| 控制面 / 服务端管理 UI | `control-plane-admin.pen` | P2 | 梳理身份、授权、物性资产、版本、租约和分发状态的管理型页面边界 |
| 移动端或只读视图 | `readonly-mobile-view.pen` | P2 | 梳理项目浏览、运行状态、结果审阅和诊断摘要的只读体验 |

### 本阶段重点

P0 先行，P1 跟随 P0 的 Inspector / Result 规则推进。P2 只做边界和页面职责，不进入完整设计稿细化，除非 P0 / P1 已经评审通过。

当前不把端点拆到更细，例如单独的 Home 设计稿、Canvas 设计稿或 Result 设计稿。原因是 Studio 客户端本体必须先统一主工作流和区域职责，再决定是否拆分子稿。若后续 P0 设计稿过大，再按 Home / Workbench / Result 等域拆分，并保留同一命名前缀。

## 暂不纳入项

以下内容不进入本专题第一轮设计，也不应作为设计稿中的隐藏未来入口：

- 自由连线编辑器、任意端口选择器、自动布线系统、完整拖拽布局编辑器。
- 完整报表系统、跨快照报表、模板导出、打印、批量导出。
- 完整参数表、完整组分数据库、第三方物性包加载、完整 Thermodynamics PMC。
- tag、release notes、便携包刷新、安装器、发布自动化。
- CAPE-OPEN / COM 语义倒灌到 Rust Core 或通用 Studio UI。
- 为尚未进入范围的移动端编辑、多人协作、控制面运营后台提前设计完整功能矩阵。

设计稿可以为这些能力保留架构空间，但不能把它们画成当前可用入口。

## Studio 客户端本体

Studio 客户端本体设计稿覆盖当前用户从启动到结果审阅的主路径。

### 页面职责

| 区域 | 职责 | 不承担 |
| --- | --- | --- |
| Home | 开始项目、打开示例、最近项目、环境状态、登录入口 | 营销页、完整控制台、完整命令面板 |
| Workbench App Bar | 项目标题、保存状态、运行主命令、全局状态 | 完整 ribbon、调试计数、所有低频命令 |
| Project / Palette | 项目物性包、项目组分、对象导航、放置入口 | 检查器字段编辑、结果审阅、运行日志 |
| Canvas | 流程图主舞台、单元和流股扫读、受控 suggestion、当前选择 | 自由连线、完整拖拽布局、长说明文本 |
| Inspector | 当前对象的输入、端口、关联诊断、关联结果入口 | 完整参数表、第二套结果解释、全局运行日志 |
| Result | 当前 revision 的最新 `SolveSnapshot` 审阅 | 历史快照报表、跨快照比较、shell 私有缓存 |
| Package / Auth | 内置 package、项目组分、登录 / 授权状态 | 第三方包市场、完整控制面后台 |
| Bottom Drawer | 结果表、诊断、消息和必要 activity | 原始 trace 常驻、开发态内部状态墙 |

### 主工作流

P0 设计稿必须覆盖这些用户路径：

1. 启动 Studio，判断本地环境、登录状态和最近项目。
2. 新建普通空白项目，显式选择内置 package 和项目组分。
3. 放置 `Feed -> Flash Drum`，补齐输入，运行并查看结果。
4. 放置 `Feed -> Cooler / Valve -> Flash Drum`，补齐单元参数，运行并查看中间流股结果。
5. 放置 `Feed + Feed -> Mixer -> Flash Drum`，补齐双入口、Mixer 参数，运行并查看 Mixer outlet 和 Flash split。
6. 保存、重开、rerun，并确认旧结果失效提示和最新结果入口。
7. 在 readiness 阻断和 Run Panel 正式诊断之间区分建模输入缺口、结构连接问题和求解阶段失败。

### 状态来源

设计稿中的状态必须能映射到既有模型，不新增第二套真相源：

| UI 状态 | 来源 |
| --- | --- |
| 项目标题、脏状态、revision | `WorkspaceDocument` / document metadata |
| 运行状态、pending reason、failure | Run Panel / workspace control model |
| 建模输入缺口 | shared modeling readiness |
| 结构性连接、拓扑、求解失败 | formal solve diagnostics / Run Panel recovery |
| 当前结果 | latest current-revision `SolveSnapshot` |
| 旧结果失效提示 | stale snapshot presentation |
| Canvas unit / stream / suggestion | canvas presentation / local rules suggestions |
| Inspector 字段、草稿、提交命令 | inspector draft / command surface |
| package / components | flowsheet thermo and component catalog |
| 登录 / 授权 | auth / entitlement state |

## 单元模块 UI

单元模块 UI 设计稿重点不是做每个单元的完整高级配置，而是统一现有可编辑单元的界面规则。

### 覆盖对象

| 单元 | 当前重点 |
| --- | --- |
| Feed | source temperature、source pressure、molar flow、composition、outlet stream |
| Mixer | inlet_a / inlet_b、outlet pressure、weighted outlet result |
| Heater / Cooler | inlet、outlet temperature、outlet pressure、intermediate stream result |
| Valve | inlet、outlet pressure、throttled stream result |
| Flash Drum | inlet、flash temperature、flash pressure、liquid / vapor outlets |

### 统一规则

- 参数字段必须显示单位、当前值来源、草稿状态、提交动作和约束提示。
- 端口区必须区分已连接、未连接、suggested、diagnostic attention。
- 结果区只读消费当前 `SolveSnapshot`，并清楚区分 consumed streams、produced streams 和 terminal outlets。
- 诊断区必须能跳转到相关 stream、unit、port 或 recovery action。
- 帮助入口只解释当前字段和主路径，不放长篇教程或未来功能说明。

## 控制面与只读端点

P2 端点当前只做边界，不进入功能扩张。

控制面 / 服务端管理 UI 可以借鉴 Cloudflare、1Panel 和 Discourse Admin 的管理型页面组织，但第一轮只明确：

- 身份和授权状态如何展示。
- 物性资产、版本、租约和分发状态是否需要同一页面。
- 哪些状态只属于服务端，不能放进 Studio 客户端主工作台。

移动端或只读视图第一轮只明确：

- 只读项目浏览和运行状态摘要。
- 最新结果审阅和诊断摘要。
- 不提供建模编辑、运行提交、保存或包管理操作。

## `.pen` 设计稿规则

设计稿目录：

```text
docs/architecture/designs/
```

首批命名：

```text
docs/architecture/designs/studio-client-main.pen
docs/architecture/designs/unit-module-panel.pen
docs/architecture/designs/control-plane-admin.pen
docs/architecture/designs/readonly-mobile-view.pen
```

设计 brief：

```text
docs/architecture/designs/studio-client-main-brief.md
```

规则：

- `.pen` 文件只通过 Pencil MCP 工具读取、生成、验证和导出，不用普通文本工具读取或改写。
- 若 `.pen` 创建前需要先评审信息架构，可在同目录维护对应 Markdown brief；brief 只记录设计输入和评审问题，不替代 `.pen` 设计稿。
- 设计稿必须随仓库保存和同步，不只保留在个人本地或聊天记录里。
- 每个设计稿必须在评审记录中说明参考了哪些原则，不能复制外部产品品牌、图标、具体配色或页面结构。
- P0 设计稿先产出信息架构和主工作流，再细化视觉。
- 若设计稿拆分，使用稳定前缀，例如 `studio-client-home.pen`、`studio-client-workbench.pen`。
- 设计稿进入代码实现前，必须记录评审状态、实现范围和允许偏离点。

## 评审顺序

1. 评审本文档的端点边界和暂不纳入项。
2. 创建并评审 `studio-client-main.pen` 的信息架构稿。
3. 对照 `docs/architecture/studio-ui-design-guidelines.md`、`docs/architecture/assets/studio-ui/baseline/` 和 `docs/architecture/ui-inspiration-reference.md`，确认主工作流、状态来源和 Home / Workbench 视觉方向没有冲突。
4. 决定是否拆分 P0 子稿。
5. 创建并评审 `unit-module-panel.pen`。
6. 仅在 P0 / P1 稳定后，评估 P2 是否需要进入设计稿细化。

## 实现前检查

任何 UI 代码实现前，必须先确认：

- 设计稿对应的 `.pen` 文件已存在且评审通过。
- 改动不会引入自由连线、完整拖拽布局、自动布线、完整参数表或完整报表。
- UI 状态能映射到既有 presentation / command / state 模型。
- 需要新增 presentation 字段时，字段职责明确，不是 shell 私有补丁。
- 需要新增命令时，命令进入正式 command surface，不只藏在单个面板里。
- 结果审阅仍只消费当前 revision 的 latest `SolveSnapshot`。
- 文档、测试和周志会随实现同步更新。
