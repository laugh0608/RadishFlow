# RadishFlow 架构草案

更新时间：2026-05-01

## 文档目的

本文档用于定义一个面向未来的目标架构：在保留 CAPE-OPEN 互操作能力的前提下，构建一个以 Rust 为核心、以 Rust UI 为主界面、以 `.NET 10` 负责 COM/CAPE-OPEN 适配、以 `ASP.NET Core / .NET 10` 负责远端授权与受控资产控制面的新一代稳态流程模拟软件。

本文档描述的是**目标仓库与目标系统结构**，不是对当前 `CapeOpenCore` 仓库的立即目录改造说明。当前仓库可以作为 CAPE-OPEN 接口参考与适配层演进基础，但不建议直接原地演化为最终产品结构。

## 名称方案

## 推荐主名称

推荐软件名称：`RadishFlow`

推荐原因：

- 保留已有项目 `Radish` 的品牌识别
- `Flow` 明确指向流程模拟、流股、流程图与稳态求解
- 名称简洁，适合作为产品名、仓库名和命名空间前缀
- 后续容易扩展出子产品与子模块

## 推荐产品命名

| 对象 | 推荐名称 | 说明 |
| --- | --- | --- |
| 软件总名 | `RadishFlow` | 产品主标识 |
| 桌面应用 | `RadishFlow Studio` | Rust UI 桌面程序 |
| 核心引擎 | `RadishFlow Core` | Rust 模拟内核 |
| CAPE-OPEN 适配层 | `RadishFlow CapeBridge` | .NET 10 COM/CAPE-OPEN 桥接层 |
| 远端控制面 | `RadishFlow Control Plane` | ASP.NET Core / .NET 10 授权、租约与资产控制面 API |
| 仓库名 | `RadishFlow` | 目标 Monorepo 名称 |

## 产品目标

## 第一阶段目标

第一阶段只追求以下闭环：

- 使用 Rust 实现稳态模拟核心
- 使用 Rust 实现桌面 UI
- 使用 .NET 10 实现 CAPE-OPEN/COM 适配层
- 使用 `ASP.NET Core / .NET 10` 实现远端授权、租约与受控资产控制面
- 只导出自有 CAPE-OPEN 模型给通用 PME 使用
- 暂不支持加载第三方 CAPE-OPEN 模型
- 支持最小化 MVP 热力学与单元模块

## MVP 范围

建议锁定 MVP 范围如下：

- 二元体系
- 简化热力学模型
- `TP Flash`
- 物性与焓的最小可用实现
- 流股对象
- 单元模块：`Feed`、`Mixer`、`Heater/Cooler`、`Valve`、`Flash Drum`
- 无回路或极简回路的顺序模块法求解
- JSON 项目存储
- 至少一个可被外部 PME 识别和调用的 CAPE-OPEN Unit Operation PMC

## 总体架构

系统建议拆分为桌面进程内三层，加一个独立外部服务平面：

1. Rust Core
2. Rust Studio UI
3. .NET 10 CapeBridge
4. .NET 10 Control Plane (External)

### Rust Core

负责：

- 领域模型
- 物性与热力学
- 闪蒸算法
- 单元模块
- 流程图数据结构
- 稳态求解器
- 项目存储
- 对外 FFI

### Rust Studio UI

负责：

- 流程图画布
- 平面 / 立体视图模式切换
- 模型编辑
- 参数面板
- 运行控制
- 结果展示
- 日志与诊断
- 项目打开保存
- 受控物性包选择与工作区运行命令编排
- 基于规则与模型辅助的建模建议交互

### .NET 10 CapeBridge

负责：

- COM 暴露
- CAPE-OPEN 接口实现
- GUID/ProgID/注册语义
- PME 互操作
- ECape 异常映射
- 对 Rust Core 的句柄式调用封装

### .NET 10 Control Plane (External)

负责：

- 对接 `Radish.Auth` 与 OIDC / OAuth 2.0 身份体系
- 返回 `EntitlementSnapshot`、`PropertyPackageManifest` 与离线租约
- 为派生物性包签发短时下载票据或签名 URL
- 管理受控资产审计、撤销与租约刷新
- 保护 A 级原始物性资产，不让其默认完整下发到桌面端

不负责：

- 本地主求解循环
- CAPE-OPEN / COM 适配
- 用在线 RPC 替代本地 `rf-thermo` / `rf-flash` / `rf-solver`

## 技术选择

## Rust UI

第一阶段推荐使用：

- UI 框架：`egui/eframe`

推荐原因：

- 适合快速做工程化 MVP
- 适合流程图画布、属性面板、日志面板、节点编辑
- 与 Rust 内核同语言，前期迭代效率高
- 比一开始就上更重的桌面方案更适合 MVP

后续如果需要更强的原生桌面风格，可再评估 Slint 或其他方案，但不建议第一阶段更换。

## 外部控制面

第一阶段推荐使用：

- 服务框架：`ASP.NET Core`
- 运行时：`.NET 10`
- 资产分发：对象存储 / CDN / 下载网关 + 短时票据或签名 URL

推荐原因：

- 与现有 `Radish.Auth` 的 OIDC / Claims / Policy 体系更容易对齐
- 与当前已经冻结的 `.NET 10` CAPE-OPEN 适配层处在同一语言生态，避免形成 Rust / .NET / Go 三线并行
- 控制面本质上是认证、授权、租约与审计 API，不是数值热路径服务
- 桌面端最终交付形态是“压缩包展开后直接运行”的原生客户端，不以服务端是否也能做单文件产物为决策中心

## Rust 与 .NET 边界

推荐边界形式：

- Rust 导出稳定的 `extern "C"` ABI
- .NET 10 用 `LibraryImport` / PInvoke 调用
- 边界上传递句柄、数组、基础数值、UTF-8 字符串、JSON

不建议：

- 让 Rust 直接处理 COM
- 让 Rust 直接处理 `IDispatch`、`VARIANT`、`SAFEARRAY`
- 在边界上传递复杂对象图

## CAPE-OPEN 规范实现策略

RadishFlow 对 CAPE-OPEN 的实现原则不是“复制某个官方示例工程”，而是“把 CAPE-OPEN 视为外部契约并独立实现自己的桥接层”。

规范真相源按以下优先级冻结：

- 官方 PDF 规格书与 errata / clarifications 负责定义行为语义
- 官方 IDL、Type Libraries、Primary Interop Assemblies 负责定义二进制接口形状
- 官方安装包和接口分发版本用于本地校准与互操作验证
- 官方示例代码与历史参考仓库只作参考，不直接充当实现蓝本

落到工程分层上，应坚持：

- CAPE-OPEN / COM 只存在于 `.NET 10 CapeBridge`
- Rust Core 保持领域模型和求解模型自主，不按 COM 对象形状建模
- `.NET 10 CapeBridge` 对外严格贴标准接口、GUID、HRESULT、ECape 语义和注册约定
- `.NET 10 CapeBridge` 对内只调用 RadishFlow 自己的稳定 ABI，而不是把 CAPE-OPEN 语义压回 Rust

因此，RadishFlow 的路线应是：

- 外部兼容 CAPE-OPEN
- 内部保持 RadishFlow 自主接口
- 示例代码只帮助理解标准，不直接决定内部架构
- 当前阶段先完成 COM-compatible Unit Operation PMC 主线，不把完整 Thermo PMC、第三方模型加载或 COBIA 主线化一起提前带入

关于 COBIA：

- COBIA 可作为后续互操作技术选项持续跟踪
- 但在当前阶段，它不应替代既定的 COM-compatible CAPE-OPEN 主线
- 除非目标 PME 与验证路径明确要求，否则不为引入 COBIA 而改写当前边界

## 推荐仓库结构

建议目标仓库采用 Monorepo：

```text
RadishFlow/
├─ Cargo.toml
├─ rust-toolchain.toml
├─ README.md
├─ LICENSE
├─ .gitignore
├─ docs/
│  ├─ architecture/
│  ├─ mvp/
│  ├─ capeopen/
│  ├─ thermo/
│  └─ adr/
├─ apps/
│  └─ radishflow-studio/
│     ├─ Cargo.toml
│     └─ src/
├─ crates/
│  ├─ rf-types/
│  ├─ rf-model/
│  ├─ rf-thermo/
│  ├─ rf-flash/
│  ├─ rf-unitops/
│  ├─ rf-flowsheet/
│  ├─ rf-solver/
│  ├─ rf-store/
│  ├─ rf-ui/
│  ├─ rf-canvas/
│  └─ rf-ffi/
├─ adapters/
│  └─ dotnet-capeopen/
│     ├─ RadishFlow.CapeOpen.sln
│     ├─ Directory.Build.props
│     ├─ RadishFlow.CapeOpen.Interop/
│     ├─ RadishFlow.CapeOpen.Adapter/
│     ├─ RadishFlow.CapeOpen.UnitOp.Mvp/
│     ├─ RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests/
│     ├─ RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost/
│     ├─ RadishFlow.CapeOpen.Registration/
│     └─ RadishFlow.CapeOpen.SmokeTests/
├─ bindings/
│  └─ c/
│     └─ radishflow.h
├─ examples/
│  ├─ flowsheets/
│  ├─ sample-components/
│  └─ pme-validation/
├─ tests/
│  ├─ rust-integration/
│  ├─ thermo-golden/
│  ├─ flash-golden/
│  └─ capeopen-smoke/
├─ scripts/
│  ├─ gen-bindings.ps1
│  ├─ register-com.ps1
│  ├─ package.ps1
│  └─ smoke-test.ps1
└─ assets/
   ├─ icons/
   ├─ themes/
   └─ sample-data/
```

说明：

- `Radish.Auth`、`RadishFlow Control Plane` 与资产分发基础设施属于系统外部依赖，当前不作为本仓库 Monorepo 的必备目录
- 本仓库继续聚焦 Rust 客户端与 `.NET 10` CAPE-OPEN 适配层，不把远端服务端代码强行塞进 Rust workspace

## 仓库分层说明

## `apps/radishflow-studio`

这是 Rust 桌面应用入口。

职责：

- 应用启动
- 主窗口布局
- 菜单、工具栏、状态栏
- 文档生命周期管理
- 将 `rf-ui`、`rf-canvas`、`rf-solver` 等能力组装为产品
- 负责 auth cache、本地物性包选择、工作区运行命令与结果回写的应用层编排

不建议在这里直接堆放热力学与求解细节。

当前对齐：

- 已建立控制面 client、auth cache sync 与本地派生包下载落盘编排
- 已建立 `solver_bridge`，把 `PropertyPackageProvider` / 本地 auth cache 与 `rf-solver`、`rf-ui::AppState` 接通
- 已建立 `WorkspaceSolveService`，收口默认 request、手动/自动运行门控与工作区求解分发
- 已建立 `WorkspaceRunCommand`，把“触发类型 + package 选择”抽成更接近桌面命令层的对象
- 已建立 `StudioAppFacade`，把 auth cache 上下文、运行命令、结果派发摘要和后续异步执行边界收口为当前明确的桌面应用入口
- 已建立 `workspace_control`，把运行栏 / 状态栏动作入口与状态摘要收口为 `WorkspaceControlAction` / `WorkspaceControlState`
- 已建立 `run_panel_driver`，把运行栏 widget 的构建、激活和事件分发回收为单独应用层入口
- 已建立 `entitlement_control`、`entitlement_panel_driver`、`entitlement_preflight` 与 `entitlement_session_driver`，把 entitlement panel 动作、启动预检、会话内调度和显式 session event 宿主收口为 Studio 应用层入口
- 当前最小桌面入口 `run_studio_bootstrap` / `main.rs` 已改为默认通过 `StudioBootstrapTrigger::WidgetPrimaryAction -> RunPanelWidgetEvent -> run_panel_driver -> WorkspaceControlAction -> StudioAppFacade` 触发运行链路，同时仍保留显式 `RunPanelIntent` 兼容入口
- 当前 entitlement 会话调度也已通过 `EntitlementSessionEvent::{SessionStarted, LoginCompleted, TimerElapsed, EntitlementCommandCompleted}` 形成统一事件语义，并由 Studio 侧维护失败退避与下一次建议检查时机
- 当前 GUI / app host 命令面也已继续收口：`run_panel.recover_failure`、`entitlement.sync` 与 `entitlement.refresh_offline_lease` 等正式动作已统一走稳定 `command_id -> UiAction/trigger` 主通路，不再继续保留 entitlement 或 foreground recovery 的历史包装旁路
- 当前 Studio 文档生命周期已接入正式 runtime 边界：`Save / Save As` 通过 `StudioRuntimeTrigger::DocumentLifecycle -> document_lifecycle_driver -> rf-store::write_project_file` 写回项目文件，保存态由 `last_saved_revision / has_unsaved_changes` 表达，不进入 `CommandHistory`
- 当前 GUI 快捷键策略已把文本输入焦点、文档历史和项目保存边界分清：`Ctrl+S` 继续走 `file.save`，`Ctrl+Z / Ctrl+Y` 在文本输入焦点下交还输入框自身编辑历史，普通焦点下才派发 `edit.undo / edit.redo`
- 当前结果审阅继续通过 `StudioGuiWindowModel` 暴露只读 presentation：选中流股时可查看最新流股状态与组成，选中已运行单元时可查看最新 `SolveSnapshot` 中的执行状态、step 序号、summary 和产出流股跳转；求解步骤自身也携带单元与产出流股 command action，供 Runtime、Active Inspector 和 Result Inspector 复用；失败摘要也携带 recovery action 与 recovery target action，继续复用运行栏修复命令和 Inspector target 命令；这不回写项目文档，也不等同于结果导出或报表系统
- 当前 Studio Canvas 已从 pending edit 前置状态推进到最小可见、只读扫读层和多单元放置反馈闭环：单元块、物流线、Inspector 焦点反馈、viewport focus anchor、对象列表导航、端口 marker、端口 hover、运行/诊断 badge、状态 legend、对象列表临时筛选、`Feed / Mixer / Heater / Cooler / Valve / Flash Drum` placement palette 与最近一次 command result 摘要均消费 GUI-facing presentation；begin-place 只创建 transient pending edit，commit 成功和失败都会进入统一 `StudioGuiCanvasCommandResultViewModel`，并复用 object command target / focus anchor 驱动新建提示、Inspector 焦点、Canvas 一次性定位、GUI activity 与命令面只读反馈；当前已通过多单元 placement 提交端回归矩阵锁定 `CreateUnit kind`、canonical ports、Inspector 焦点、Canvas focus anchor 与 command result 反馈；2026-05-04 又补出本地 Canvas suggestions，让 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum` 和 `Feed + Feed -> Mixer -> Flash Drum` 可通过正式 `ConnectPorts` / outlet stream 创建与手动运行走到求解收敛；同日继续补齐空白项目前置基线，无组件项目打开时会初始化 MVP 默认二元组件与本地 `binary-hydrocarbon-lite-v1` 物性包缓存，保存后重新打开仍可运行 `Feed -> Flash Drum`；随后又补出逐条 suggestion Apply，让用户可显式接受指定本地连接或 outlet stream 创建建议，而不依赖当前 focused suggestion；同日还补出 placement 坐标最小持久化，已提交单元落点保存到 `<project>.rfstudio-layout.json` sidecar，重开后 Canvas 优先按 sidecar 坐标渲染；随后继续把离散布局移动前推为 `canvas.move_selected_unit.left/right/up/down` widget action / command surface，缺少 sidecar 坐标时先按 transient grid slot pin 出初始位置，只写 Studio layout sidecar，不写项目文档、不触发 dirty；这一层视为最小建模路径收口，当前仍不做端口点击编辑、自由连线创建、拖拽布局编辑器或视口持久化，也不做完整组件库或完整物性包选择器
- 当前默认包选择策略保持保守，只在唯一候选时自动选中，多包场景要求显式指定 package
- Automatic 运行当前先根据 `SimulationMode` / `pending_reason` 决定是否 skip，再决定是否需要 preferred package 解析

## `crates/rf-types`

最底层共享类型库。

职责：

- 基础 ID 类型
- 枚举与错误码
- 单位、相、端口方向等公共概念

目标：

- 稳定
- 轻量
- 尽量避免依赖其他 crate

## `crates/rf-model`

领域模型库。

职责：

- 组分
- 流股状态
- 单元定义
- 端口结构
- 流程图对象模型

## `crates/rf-thermo`

物性与热力学能力库。

MVP 建议内容：

- Antoine 饱和蒸气压
- 理想液相/气相假设
- Raoult 定律
- 简化焓模型
- 基础物性数据查询

说明：

第一阶段只求“能算、可验证、可用于流程闭环”，不追求工业级完整性。

## `crates/rf-flash`

闪蒸算法库。

MVP 建议内容：

- `TP Flash`
- Rachford-Rice 求解
- 汽液相摩尔分率与组成计算

后续再考虑：

- `PH Flash`
- `PS Flash`
- 泡点/露点

## `crates/rf-unitops`

单元模块库。

MVP 建议内容：

- `Feed`
- `Mixer`
- `Heater/Cooler`
- `Valve`
- `Flash Drum`

建议每个单元都实现统一的求解接口，输入输出尽量使用标准化流股与参数对象。

当前对齐：

- 已建立首轮内建单元规范
- 已实现 `Feed`、`Mixer`、`Flash Drum` 的最小行为边界
- 当前统一围绕标准 `MaterialStreamState` 输入输出与必要热力学服务注入

## `crates/rf-flowsheet`

流程图结构层。

职责：

- 节点与连线
- 端口连接规则
- 图完整性校验
- 拓扑排序前置检查

不负责具体数值求解。

当前对齐：

- 已建立首轮材料端口连接校验
- 当前先冻结为 canonical material ports、流股存在性与“一股一源一汇”约束
- 拓扑排序与执行调度继续留给 `rf-solver`

## `crates/rf-solver`

稳态求解器层。

MVP 建议内容：

- 无回路流程的顺序模块法
- 简单依赖图执行顺序
- 基础错误与诊断输出

后续扩展：

- recycle 收敛
- tear stream
- 更复杂的求解策略

当前对齐：

- 已建立首轮无回路顺序模块法
- 当前支持内建 `Feed`、`Mixer`、`Flash Drum` 的标准材料流股执行
- 已形成第一个可从 `*.rfproj.json` 载入到求解输出的最小闭环样例

## `crates/rf-store`

项目存储层。

职责：

- JSON 序列化/反序列化
- 项目版本兼容
- 示例流程读写

建议：

- 模型状态与 UI 状态分离保存

## `crates/rf-ui`

Rust UI 逻辑层。

职责：

- 面板状态
- 命令分发
- 选择集与属性编辑逻辑
- 与求解结果的展示绑定
- 智能建议、灰态占位与显式接受/拒绝交互的状态管理

建议：

- 这里放“UI 行为逻辑”
- 不直接承载底层算法实现

当前对齐：

- 已冻结 `AppState`、`WorkspaceState`、`SolveSessionState` 与 `SolveSnapshot` 的最小边界
- 已具备从 `rf-solver::SolveSnapshot` 映射到 UI 层快照并写回 `AppState` 的桥接
- 已补 `RunPanelState`，用于承接运行栏摘要
- 已补 `RunPanelIntent` / `RunPanelPackageSelection`，用于表达 UI 自有运行意图
- 已补 `RunPanelCommandModel`，把 `Run/Resume/Hold/Active` 的主动作、可见性与可用性冻结到 UI 层
- 已补 `RunPanelViewModel` / `RunPanelTextView` / `RunPanelPresentation`，把最小渲染与文本展示组织收回 UI 层
- 已补 `RunPanelWidgetModel` / `RunPanelWidgetEvent`，把最小 widget 激活语义冻结到 UI 层
- 已补 Stream Inspector 字段级草稿、单字段提交、多字段批量提交和 `CommandHistory` 基础 undo/redo；只有 valid dirty 草稿在语义提交时写回 `FlowsheetDocument`，无效中间态继续停留在 UI 草稿状态

后续规划：

- 后续可把 RadishMind 或本地规则产出的“候选补全”统一收口为 UI 层建议状态，而不是让画布渲染层或远端模型直接改写 flowsheet 文档
- 对标准单元的端口补全、常见连线建议与 Tab 接受语义，应优先沉淀为 UI 契约，再决定是否接入更强的模型推断

## `crates/rf-canvas`

流程图画布专用库。

职责：

- 节点绘制
- 端口绘制
- 连线绘制
- 视图投影与 2D / 立体模式切换
- 物流 / 能量流 / 信号流的静态与动态可视化
- 收敛、未收敛、待补全与告警等叠加态呈现
- 拖拽、缩放、平移、框选

说明：

流程图画布复杂度会持续增长，单独拆库有利于后续维护。

补充规划：

- 底层 flowsheet 语义模型应保持单一真相源，2D 与立体视图只是同一数据的不同投影与渲染模式，不引入第二套项目文件语义
- 流线可视化建议把“流线类型”和“运行状态”拆成两套正交编码：类型优先用主色区分，状态优先用饱和度、透明度、线型、箭头动效或徽标区分，避免只靠单一换色表达全部信息
- 对标准单元放置后的待补全入口/出口，可在画布中先显示灰态 ghost 端口或 ghost 连线，再通过 `Tab` 或显式确认动作接受；未接受前不写回正式文档

## `crates/rf-ffi`

Rust 与 .NET 的桥接层。

职责：

- 对外导出稳定 C ABI
- 提供句柄式调用
- 管理字符串与内存边界

建议导出接口形态：

- `engine_create`
- `engine_destroy`
- `engine_last_error_message`
- `engine_last_error_json`
- `rf_string_free`
- `flowsheet_load_json`
- `property_package_load_from_files`
- `property_package_list_json`
- `flowsheet_solve`
- `flowsheet_get_snapshot_json`
- `stream_get_snapshot_json`

## `adapters/dotnet-capeopen`

这是 .NET 10 的 CAPE-OPEN/COM 适配层根目录。

它是外部互操作桥，不是模拟器主体。

### `RadishFlow.CapeOpen.Interop`

职责：

- CAPE-OPEN 接口定义
- GUID、属性、辅助类型
- ECape 异常基础设施
- 最小 `IDispatch` marshalling 形状

它可以吸收当前 `CapeOpenCore` 仓库中的接口定义与注册经验。

当前对齐：

- 已形成 `ICapeIdentification`、`ICapeUtilities`、`ICapeUnit` 的最小接口骨架
- 已收口第一版 CAPE-OPEN interface/category GUID、HRESULT 与 `ECapeRoot` / `ECapeUser` / `ECapeBadInvOrder` / `ECapeBadCOParameter` 等最小异常语义

### `RadishFlow.CapeOpen.Adapter`

职责：

- PInvoke 调用 `rf-ffi`
- 将 Rust 句柄封装成 .NET 侧对象
- 完成数据与错误转换

这是 .NET 与 Rust 的唯一正式运行时边界。

当前对齐：

- 当前已形成 `LibraryImport` 薄适配与 native engine 句柄生命周期封装
- 当前已把 `RfFfiStatus + last_error_message/json` 收口到更细粒度 ECape 语义异常

### `RadishFlow.CapeOpen.UnitOp.Mvp`

职责：

- MVP 阶段的 CAPE-OPEN Unit Operation PMC 骨架
- 先冻结最小 `ICapeIdentification` / `ICapeUtilities` / `ICapeUnit` 状态机与对象边界
- 后续再通过适配层调用 Rust 核心并作为外部 PME 验证对象

建议先以 Unit Operation 为导出重点，不在第一阶段同时做完整 Thermo PMC。

当前对齐：

- 当前已建立最小 PMC 类、`Initialize/Validate/Calculate/Terminate/Edit` 状态机与内部配置入口
- 当前已把 `Ports` / `Parameters` 推进为最小占位对象集合，并让 `Validate()` 先基于对象状态做必填参数与必连端口检查
- 当前已进入受控 COM 注册与真实 `DWSIM / COFE` 人工复验阶段性收敛阶段，但仍未进入 PME 自动化互调或完整 PME 生命周期框架；最小 native 求解接线已打通，且 `Calculate()` 对外结果面当前已收口为稳定的“成功结果 + 失败摘要”双契约，并进一步提供统一只读查询面 `GetCalculationReport()`、其上的标量元数据入口 `GetCalculationReportState()/GetCalculationReportHeadline()`、可枚举 detail 键值入口 `GetCalculationReportDetailKeyCount()/GetCalculationReportDetailKey(int)/GetCalculationReportDetailValue(string)`、最小文本导出面 `GetCalculationReportLines()/GetCalculationReportText()`，以及更接近宿主逐行消费习惯的 `GetCalculationReportLineCount()/GetCalculationReportLine(int)`，而不是继续直接暴露完整 snapshot JSON 或 native error JSON
- 当前又已把上述 stable detail key 清单冻结为公开 catalog `UnitOperationCalculationReportDetailCatalog`，明确 success / failure 两条路径的 canonical key 顺序，避免宿主侧再依赖散落字符串常量或文档口径
- 在最小结果面阶段性冻结后，当前又已把内部状态推进显式收口为 `UnitOperationLifecycleState`、分段 `EvaluateValidation()` guard 链、分段 `Calculate()` 执行链，以及统一的 validation/calculation/report transition helper，避免后续宿主主线继续推进时在同一个 PMC 类里堆叠隐式状态分支
- 在公开 report API 之上，当前又已补出 `UnitOperationHostReportReader -> UnitOperationHostReportPresenter -> UnitOperationHostReportFormatter` 三级宿主消费 helper；这条链路的定位是冻结最小宿主读取/展示口径，而不是继续给 PMC 主类追加更多 convenience accessor
- 当前 host-facing 主路径又已继续收口为 `UnitOperationHostViewReader`、`UnitOperationHostActionExecutionRequestPlanner`、`UnitOperationHostActionExecutionOrchestrator`、`UnitOperationHostValidationRunner`、`UnitOperationHostCalculationRunner`、`UnitOperationHostRoundOrchestrator` 与统一 `FollowUp` / `StopKind` 模型；这些 helper 负责正式消费面和窄边界 orchestration，不把 `SmokeTests` 的完整 driver DSL 上移成库 API
- 当前又已通过 `UnitOperationComIdentity` 冻结 MVP PMC 的 `CLSID / ProgID / Versioned ProgID / DisplayName / Description`，并在 `RadishFlowCapeOpenUnitOperation` 上固定 `ComVisible / Guid / ProgId / ClassInterface(None)` 注册前置元数据；真实注册只能经由带 execute/token/backup/rollback 门控的 `Registration` 与仓库脚本入口执行

### `RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests`

职责：

- 锁定 `UnitOp.Mvp` 的库侧行为契约
- 在不依赖外部 NuGet 测试框架的前提下验证最小 PMC 状态机
- 为 `SmokeTests` 之外提供更贴近库内语义的回归入口

当前对齐：

- 当前已建立自举式 `Exe` runner，并已加入 `RadishFlow.CapeOpen.sln`
- 当前已覆盖 `Validate before Initialize`、validation failure report、native failure report、success report、配置变更 invalidation 与 `Terminate()` 后阻断共 6 条核心 contract case
- 当前已显式锁住 validation/native 两类 failure report 在 detail 字段上的缺省规则，避免这部分行为只停留在 smoke 输出或 README 约定里

### `RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost`

职责：

- 演示外部宿主如何不依赖 `SmokeTests` driver DSL 消费 `UnitOp.Mvp` 正式 host-facing 模型
- 提供更接近 PME host 的薄 session 样板
- 验证 `view -> request planning -> host round -> session/execution/port-material/report` 的正式消费路径

当前对齐：

- 当前已建立独立 `net10.0` console，并已加入 `RadishFlow.CapeOpen.sln`
- 当前 console 已默认通过 `PmeLikeUnitOperationHost / PmeLikeUnitOperationSession / PmeLikeUnitOperationInput` 执行“创建组件、初始化、读取视图、提交参数/端口对象、执行 validate/calculate round、读取正式结果面、终止”
- 该样例只复用 `UnitOp.Mvp` 正式 reader / planner / host round / session-execution-port-material-report 消费面，不复用 `SmokeTests` driver DSL
- 该样例不做 COM 注册、不驱动真实 PME、不加载第三方 CAPE-OPEN 模型，也不承诺完整 PME 生命周期框架

### `RadishFlow.CapeOpen.Registration`

职责：

- COM host 注册与反注册
- 管理员提权
- 冒烟验证辅助

当前对齐：

- 当前已建立第一版 `net10.0` 注册工具，并已加入 `RadishFlow.CapeOpen.sln`
- 当前可输出 MVP Unit Operation PMC 的 `CLSID / ProgID / Versioned ProgID`、CAPE-OPEN categories、最小已实现接口、当前 action / scope 与边界标志
- 当前可按 `register / unregister` 与 `current-user / local-machine` 生成 registry key plan，并把 `.NET comhost` 路径解析列为执行前 `Verify` 步骤
- 当前已启用 `UnitOp.Mvp` 的 `EnableComHosting`，并在 preflight 中只读检查真实 `UnitOp.Mvp` 输出目录中的 `RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll`、PE 机器类型、`UnitOp.Mvp.runtimeconfig/deps` sidecar、当前进程位数、scope 权限口径、目标 registry key 现状和备份范围
- 当前默认行为仍为 dry-run，但已支持在显式 `--execute`、匹配 confirmation token、无 `Fail` preflight、权限检查通过的前提下执行 `register / unregister`；执行边界已收口到 `CLSID / ProgID / Versioned ProgID / TypeLib` 四棵树备份、execution log 与失败 rollback
- 仓库根 `scripts/register-com.ps1` 当前已作为正式脚本入口，负责统一 build、token 提示、环境变量重定向和 `Registration.exe` 调用
- 当前仓库脚本默认会显式传入 `UnitOp.Mvp\bin\Debug\net10.0` 下的 comhost / typelib，不再依赖 `Registration.exe` 进程目录猜测默认路径
- 当前 execute `register` 还会补写 classic COM 所需的 `CLSID\{...}\TypeLib` 关联值，避免只注册 `TypeLib` 树却不把 CLSID 回链到 typelib GUID
- 当前 registration plan 又已补齐 `Consumes Thermodynamics`、`Supports Thermodynamics 1.0` 与 `Supports Thermodynamics 1.1` 三个 CAPE-OPEN implemented categories，用于 DWSIM 画布接受条件 probe；这不代表 MVP 已实现完整 Thermo PMC 或第三方 property package 加载
- 当前还已确认 DWSIM `CapeOpenUO.Instantiate()` 会按 `InitNew -> Initialize -> simulationContext set -> GetPorts -> GetParams` 消费 UnitOp；因此 `ICapeUtilities` vtable 必须保持 CAPE-OPEN PIA 兼容的 setter-only 前序槽位，同时把 COFE 需要的 `SimulationContext` getter 留作 late-bound 兼容面
- 当前用户侧 `DWSIM / COFE` 人工复验已确认 discovery、activation、placement、`Feed / Product` 端口连接与 water/ethanol 样例 `Validate / Calculate` 收敛主路径通过；DWSIM parameter enumeration 还要求 `Parameters.Item(i)` 返回对象本身直接支持 `ICapeIdentification / ICapeParameterSpec / ICapeOptionParameterSpec / ICapeParameter`
- 当前 `DWSIM / COFE` 成功复验已沉淀到 `examples/pme-validation/` 正式记录，临时 COM trace 也已清理为显式环境变量开关；未配置必填参数时 `Validate` 返回 invalid 属于预期行为
- 当前真实复验还确认了一个宿主侧假阴性：`pwsh` 若已预加载 `.NET 9.0.10`，会因与当前 PMC 目标 `.NET 10.0.0` runtime 不兼容而触发 `0x800080A5`；后续 native COM / PME 类探测应优先改用 `Windows PowerShell 5` 或其他非预加载 .NET 宿主
- 目标 PME 人工验证路径当前已单独落到 `docs/capeopen/pme-validation.md`，注册工具本身不承担 PME 自动化互调

### `RadishFlow.CapeOpen.SmokeTests`

职责：

- 最小冒烟测试
- `rf-ffi` 调用闭环与 .NET 项目可编译性验证

当前对齐：

- 当前可配置 native library 目录、加载示例 flowsheet 与本地 `manifest/payload` package
- 当前可列出 package registry，并覆盖 direct adapter 的 flowsheet / stream snapshot JSON 导出，以及 `UnitOp.Mvp` 的最小成功结果契约、失败摘要契约、统一只读 report access、标量元数据入口、可枚举 detail 键值入口、最小文本导出面与标量逐行读取面验证
- 当前 `unitop` 模式又已从“单条 console 冒烟脚本”收口为最小宿主验证骨架：`UnitOperationSmokeHostDriver` 固定真实宿主调用顺序，`UnitOperationSmokeBoundarySuite` 锁边界矩阵，`UnitOperationSmokeSession` 与 `UnitOperationSmokeScenarioCatalog` 负责时序变体编排；其中 action plan 到 execution request 的稳定公共部分已上移为 `UnitOperationHostActionExecutionRequestPlanner`，smoke driver 只继续负责准备样例输入、生命周期顺序和失败分类
- 在上述 request planning 之上，当前又补出 `UnitOperationHostActionExecutionOrchestrator` 与正式 `FollowUp` 模型，把“执行 ready requests 后刷新 configuration/action/session”以及“宿主下一步应补输入、做 validate、做 calculate，还是只剩 lifecycle/terminated”这层判断继续前推到库内；smoke session 当前只保留 timeline 组织与场景脚本，不再私有维护这层执行后摘要
- 当前已支持 `--unitop-scenario <all|session|recovery|shutdown>` 按宿主时序场景过滤运行，用于单独验证会话、恢复和收尾阶段，而不是每次都跑整套 timeline
- 当前暂不承担 PME/COM 注册路径的冒烟验证

## `bindings/c`

存放自动生成的 C 头文件。

建议：

- 使用 `cbindgen` 生成 `radishflow.h`

## `examples`

建议内容：

- 最小流程样例
- 外部 PME 验证样例
- Unit Operation 样例参数集

## `tests`

建议内容：

- Rust 集成测试
- 热力学基准测试
- 闪蒸基准测试
- CAPE-OPEN 冒烟测试

说明：

数值软件后期最容易出现“结果漂移但编译仍通过”的问题，因此黄金样例测试必须尽早建立。

## 设计原则

1. 模拟核心必须与 CAPE-OPEN 适配层解耦。
2. Rust 不直接处理 COM。
3. .NET 不直接实现重型热力学和闪蒸计算。
4. UI 层不直接控制 COM 注册与外部互操作逻辑。
5. 领域模型、求解器、画布和适配层必须边界清晰。

## 推荐开发顺序

1. 先建立 Rust workspace 和基础 crate 框架
2. 实现 `rf-types`、`rf-model`、`rf-thermo`、`rf-flash`
3. 做通一个最小二元 `TP Flash`
4. 实现 `rf-unitops` 中的 `Feed`、`Mixer`、`Flash Drum`
5. 实现 `rf-flowsheet` 与 `rf-solver` 的最小闭环
6. 实现 `rf-ffi`
7. 用 .NET 10 实现 `RadishFlow.CapeOpen.Adapter`
8. 导出第一个可被 PME 识别的 Unit Operation PMC
9. 在当前已建立的 `RunPanelWidgetModel + run_panel_driver + WorkspaceControlAction + StudioAppFacade + WorkspaceRunCommand + WorkspaceSolveService` 运行边界之上，再建设 `radishflow-studio` 的画布、属性编辑与最终桌面运行触发入口

## 当前仓库与目标仓库的关系

建议把当前外部参考资产视为以下来源：

参考链接：

- [CapeOpenCore](https://github.com/laugh0608/CapeOpenCore)
- [DWSIM](https://github.com/DanWBR/dwsim)

- CAPE-OPEN 接口定义参考
- COM 注册行为参考
- GUID 与属性语义参考
- 示例 Unit Operation 行为参考
- `Interfaces / FlowsheetBase / FlowsheetSolver / UnitOperations` 的职责拆分参考
- 自动化 API、自动化测试入口与 standalone thermo 暴露方式的工程组织参考
- 图形对象与求解对象分离、由连接关系决定求解顺序的实现思路参考

补充约束：

- `CapeOpenCore` 主要提供 CAPE-OPEN / COM 语义与互操作经验参考
- `DWSIM` 主要提供大型开源流程模拟器的模块拆分、自动化入口和 flowsheet solver 组织经验参考
- `DWSIM` 采用 GPL-3.0 许可，因此当前仓库只吸收架构和行为思路，不直接复制其实现代码

不建议：

- 把当前目录直接演化成最终 `RadishFlow` Monorepo
- 让当前 WinForms 代码成为未来主 UI
- 让当前 .NET 类库继续承担模拟核心职责

## 结论

`RadishFlow` 的合理形态应当是：

- Rust 做核心
- Rust 做 UI
- .NET 10 做 CAPE-OPEN/COM 桥
- 第一阶段只导出自有 Unit Operation PMC
- 第一阶段只完成最小稳态模拟闭环

这条路线既保留了 `Radish` 品牌，也最大程度降低了 COM 与 CAPE-OPEN 对核心架构的侵入。
