# RadishFlow MVP 开发路线图

更新时间：2026-04-04

## 文档目的

本文档用于把 `RadishFlow` 的第一阶段目标拆成可执行的开发路线。

本文档只覆盖 MVP，不覆盖以下内容：

- 第三方 CAPE-OPEN 模型导入
- 完整 Thermo PMC
- recycle 全功能收敛
- 复杂 GUI 风格打磨
- 大规模组件数据库

## MVP 目标声明

MVP 的目标固定为：

构建一个以 Rust 为核心、以 Rust UI 为主界面、以 .NET 10 暴露 CAPE-OPEN Unit Operation PMC 的最小稳态流程模拟闭环，并让至少一个自有单元模型可被外部 PME 识别与调用。

补充系统口径：

- 桌面端主线固定为 Rust 客户端，负责 UI、求解、项目文件读写与本地授权缓存使用
- 认证、授权、离线租约、受控物性包清单与下载票据由外部 `ASP.NET Core / .NET 10` 控制面承担
- 派生资产下载优先走对象存储 / CDN / 下载网关，不把控制面 API 设计成长期大文件出口
- 桌面端最终交付形态默认为“压缩包展开后直接运行”，不以单文件可执行产物为阶段目标

## MVP 功能边界

MVP 建议只包含以下能力：

- 二元体系
- 最小物性参数集
- 简化热力学模型
- `TP Flash`
- 流股对象
- 单元模块：`Feed`、`Mixer`、`Heater/Cooler`、`Valve`、`Flash Drum`
- 无回路流程或极简回路
- JSON 项目格式
- 一个最小 Rust 桌面工作台
- 一个可注册的 CAPE-OPEN Unit Operation PMC

## 不做项

MVP 阶段明确不做：

- 加载外部 CAPE-OPEN 单元
- 加载外部 CAPE-OPEN 物性包
- 完整 Thermodynamics 1.0/1.1 主机兼容
- 完整物性数据库
- 动态模拟
- 复杂报表系统
- 多套热力学包切换

## 总体里程碑

建议把 MVP 分成 5 个里程碑：

1. `M1`：仓库与基础骨架初始化
2. `M2`：二元体系 `TP Flash` 核心跑通
3. `M3`：最小稳态流程闭环跑通
4. `M4`：Rust FFI 与 .NET 10 适配层打通
5. `M5`：外部 PME 识别并调用自有 PMC

## M1：仓库与基础骨架初始化

### 目标

建立 `RadishFlow` 新仓库与最小开发骨架。

### 任务

- 创建 `RadishFlow` 新仓库
- 初始化 Rust workspace
- 创建核心 crates
- 创建 `apps/radishflow-studio`
- 创建 `.NET 10` 适配层解决方案目录
- 冻结外部 `.NET 10` 控制面的职责边界与最小 API 契约
- 初始化根文档与 `docs/README.md`
- 建立基础脚本目录与测试目录

### 建议优先创建的 crate

- `rf-types`
- `rf-model`
- `rf-thermo`
- `rf-flash`
- `rf-unitops`
- `rf-flowsheet`
- `rf-solver`
- `rf-ffi`

### 退出标准

- 新仓库可正常克隆
- Rust workspace 可成功 `cargo check`
- .NET 适配层目录已存在并可打开
- 文档已说明 MVP 边界与目录职责
- 外部 `.NET 10` 控制面边界与桌面客户端职责已经冻结

## M2：二元体系 TP Flash 核心跑通

### 目标

先打通最小的热力学与闪蒸核心。

### 任务

- 定义组分与基础物性结构
- 支持二元体系输入
- 实现 Antoine 饱和蒸气压
- 实现简化 `K` 值逻辑
- 实现 Rachford-Rice
- 实现 `TP Flash`
- 输出相分率、汽液相组成、基础焓值
- 建立黄金样例测试

### 推荐 crate 分工

- `rf-types`：基础枚举、ID、相标签
- `rf-model`：流股状态对象
- `rf-thermo`：组分参数与热力学模型
- `rf-flash`：闪蒸算法

### 退出标准

- 至少一个二元样例可稳定计算
- 数值结果可回归测试
- API 已可供单元模块调用
- `tests/thermo-golden` 与 `tests/flash-golden` 已建立首批黄金样例

## M3：最小稳态流程闭环跑通

### 目标

构建最小 flowsheet 求解能力。

### 任务

- 定义端口与连接关系
- 实现 `Feed`
- 实现 `Mixer`
- 实现 `Heater/Cooler`
- 实现 `Valve`
- 实现 `Flash Drum`
- 实现流程图完整性校验
- 实现顺序模块法求解
- 建立至少一个最小流程示例
- 形成至少一个可从 `*.rfproj.json` 加载到求解输出的端到端闭环样例

### 推荐示例流程

- `Feed -> Heater -> Flash Drum`
- `Feed -> Valve -> Flash Drum`
- `Feed1 + Feed2 -> Mixer -> Flash Drum`

### 推荐 crate 分工

- `rf-unitops`
- `rf-flowsheet`
- `rf-solver`
- `rf-store`

### 当前收口口径

- `rf-unitops` 第一轮统一接口先围绕标准 `MaterialStreamState` 输入输出与必要热力学服务注入
- `Mixer` 当前先固定为两进一出，`Flash Drum` 当前先固定为一进两出，优先建立第一条可求解闭环
- `Heater/Cooler` 与 `Valve` 当前先沿用“一进一出”最小接口，并以 outlet 流股模板承载当前阶段目标状态设定
- `rf-flowsheet` 第一轮只做 canonical material 端口签名、流股存在性与“一股一源一汇”校验，拓扑排序和顺序模块调度继续留给 `rf-solver`
- `rf-solver` 第一轮先只支持无回路、内建单元和标准材料流股执行，当前已覆盖 `Feed1 + Feed2 -> Mixer -> Flash Drum`、`Feed -> Heater -> Flash Drum` 与 `Feed -> Valve -> Flash Drum`
- `rf-solver` 当前已补最小求解诊断层：`SolveSnapshot` 至少包含 summary、仓库级 diagnostics 和逐步执行 step 明细；失败路径至少带 step 序号、unit id / kind 与 inlet stream 上下文
- `examples/flowsheets` 当前应维护至少三条可直接从 `*.rfproj.json` 载入并求解的示例项目，作为内核闭环回归基线
- `tests/rust-integration` 当前应作为仓库级 Rust 集成测试入口，覆盖“加载项目 -> 求解 -> 读取结果”的示例流程回归
- `rf-ui` 当前已具备把 `rf-solver::SolveSnapshot` 回写为 UI 层结果快照的稳定映射
- `rf-ui` 当前已补出 `RunPanelState`，并由 `AppState` 在文档提交、快照写入、模式切换、失败记录和日志追加后自动刷新运行栏摘要
- `rf-ui` 当前已把运行栏按钮模型冻结为 `RunPanelCommandModel`，让主按钮选择和 `Run/Resume/Hold/Active` 的启用逻辑正式留在 UI 层
- `apps/radishflow-studio` 当前已具备 `StudioAppFacade -> WorkspaceRunCommand -> WorkspaceSolveService -> solver_bridge` 四级应用层入口，可基于 `PropertyPackageProvider` 或本地 `StoredAuthCacheIndex` 执行真实 solve
- Studio 当前把运行触发先区分为 `Manual` / `Automatic`，并把 `SimulationMode`、`pending_reason` 与默认 `snapshot_id` / `sequence` 生成收口到应用层
- Studio 当前默认包选择采取保守策略：只有在唯一候选包明确时才自动选中，多包场景必须显式指定 package

### 退出标准

- 能打开一个 JSON 示例流程
- 能完成一次完整求解
- 能输出每股流体的基本状态结果
- 已具备最小集成测试，能覆盖“加载项目 -> 求解 -> 读取结果”的端到端数据流
- 仓库级验证入口 `scripts/check-repo.ps1` / `scripts/check-repo.sh` 已自动覆盖这些集成测试
- Studio 应用层已具备显式运行命令 / 服务边界，不要求最终桌面按钮已接通，但不再由 UI 直接拼接底层 provider/solver 调用

## M4：Rust FFI 与 .NET 10 适配层打通

### 目标

让 Rust 核心可被 .NET 10 适配层调用。

### 任务

- 定义稳定的 C ABI
- 设计句柄式对象生命周期
- 处理字符串与错误码返回
- 定义最小 JSON 快照接口
- 在 `.NET 10` 中建立 `RadishFlow.CapeOpen.Adapter`
- 完成 PInvoke 封装
- 建立最小互调测试

### 推荐暴露接口

- `engine_create`
- `engine_destroy`
- `flowsheet_load_json`
- `flowsheet_solve`
- `stream_get_snapshot_json`
- `unit_run`

### 退出标准

- .NET 可成功调用 Rust 动态库
- 能从 .NET 获取计算结果
- 有最小自动化测试覆盖对象创建与求解调用

## M5：外部 PME 识别并调用自有 PMC

### 目标

交付第一版可互操作的 CAPE-OPEN Unit Operation PMC。

### 任务

- 建立 `RadishFlow.CapeOpen.Interop`
- 建立 `RadishFlow.CapeOpen.UnitOp.Mvp`
- 设计最小参数与端口映射
- 实现 `ICapeIdentification`、`ICapeUtilities`、`ICapeUnit` 相关能力
- 建立注册工具 `RadishFlow.CapeOpen.Registration`
- 完成 COM host 注册流程
- 在目标 PME 中做人工验证

### 建议验证内容

- PME 能发现组件
- PME 能实例化组件
- PME 能读取参数
- PME 能连接端口
- PME 能触发 `Calculate`
- PME 能得到有效结果或可诊断错误

### 退出标准

- 至少一个目标 PME 成功识别并调用 PMC
- 注册/反注册流程可重复执行
- 文档中已写明验证路径

## 推荐开发顺序

建议严格按以下顺序推进：

1. `M1`
2. `M2`
3. `M3`
4. `M4`
5. `M5`

不要跳过 `M2` 和 `M3` 直接做 CAPE-OPEN 外壳。  
如果内核没有先稳定，后面在 PME 里出现错误时会很难定位到底是内核问题还是 COM 适配问题。

## 当前计划对齐

截至 2026-03-29，M1 阶段中与桌面控制面接线相关的地基项已进一步前推，当前已完成：

- 控制面 `entitlement / manifest / lease / offline refresh` HTTP client
- 控制面 JSON 契约到运行时 DTO 的应用层协议映射
- 按 manifest 申请 lease、下载派生包并回写本地缓存索引的应用层编排

截至 2026-03-30，系统技术口径进一步补充冻结为：

- 远端控制面优先采用 `ASP.NET Core / .NET 10`
- 资产分发优先采用对象存储 / CDN / 下载网关 + 短时票据
- 当前仓库继续只保留客户端侧接线、DTO 与缓存编排，不把服务端代码强行并入 Rust workspace

基于当前状态，下一阶段计划进一步对齐为：

- 今天（2026-03-30）优先细化授权刷新后的 UI 事件流、联网失败提示和离线刷新触发策略
- 随后恢复 `rf-thermo` / `rf-flash` 数值主线，避免地基建设继续挤占核心算法推进
- 在数值主线恢复后，再进入 `rf-solver` 无回路顺序模块法和首个可求解 flowsheet 示例

截至 2026-04-02，M3 与 Studio 应用层又进一步前推，当前已完成：

- `solver_bridge`：把 `PropertyPackageProvider` / 本地 auth cache 与 `rf-solver`、`rf-ui::AppState` 接通
- `WorkspaceSolveService`：把默认 request、手动/自动触发门控与工作区求解分发收口为应用服务入口
- `WorkspaceRunCommand`：把“触发类型 + package 选择”抽成更接近桌面命令层的对象，并冻结当前默认包选择策略
- `StudioAppFacade`：把 auth cache 上下文、运行命令、结果派发摘要和后续异步执行边界收口为明确的桌面应用入口
- `run_studio_bootstrap` + `apps/radishflow-studio/src/main.rs`：把样例项目加载、本地样例 auth cache 构造和 `StudioBootstrapTrigger -> RunPanelWidgetEvent / RunPanelIntent -> run_panel_driver / WorkspaceControlAction -> StudioAppFacade` 链路接成最小桌面进程触发点
- Studio 当前已把 Automatic skip 原因补进 `AppLogFeed`，避免“入口触发但未运行”完全静默
- Studio 当前已把 `StudioWorkspaceRunDispatch` 扩成结构化摘要，统一派发最新 snapshot 摘要和最新日志摘要给入口层消费
- Studio 当前已把 `StudioAppCommand` 扩成显式运行控制命令，覆盖 `RunWorkspace`、`ResumeWorkspace` 与 `SetWorkspaceSimulationMode`
- Studio 当前已补出 `ResumeWorkspace` 这一条 `Hold -> Active` 恢复路径，并把 `simulation_mode` / `pending_reason` 一并纳入结果派发
- Studio 当前已补出 `WorkspaceControlAction` / `WorkspaceControlState` 这一层更接近运行栏 / 状态栏的 UI 入口契约，并在 bootstrap 入口中直接消费
- Studio 当前已把 `WorkspaceControlState` 映射回 `rf_ui::RunPanelState`，从而让 `rf-ui` 正式消费最小运行栏状态对象，而不反向依赖 Studio 类型
- `rf-ui` 当前已补出 `RunPanelIntent` / `RunPanelPackageSelection`，Studio 当前已提供对应映射与分发入口，为后续按钮、菜单或快捷键接线保留稳定接口
- `rf-ui` 当前已不只拥有运行意图类型，还拥有最小按钮模型、文本渲染 DTO、`RunPanelPresentation` 与 `RunPanelWidgetModel`；后续真正视图层接线时不必再重新发明运行栏启用判断、按钮槽位组织、文本布局或按钮激活语义
- 当前最小桌面入口 `run_studio_bootstrap` / `main.rs` 已开始直接消费 `RunPanelWidgetModel`，作为真正的最小运行栏组件/消费层
- Studio 当前也已补出 `RunPanelWidgetEvent -> WorkspaceControlAction` 的分发桥，使最小 widget 交互结果能够回到既有应用命令链路
- Studio 当前也已补出 `run_panel_driver`，把最小运行栏组件驱动流程收口成单独应用层入口
- `run_studio_bootstrap` 当前也已补出 `StudioBootstrapTrigger::{Intent, WidgetPrimaryAction, WidgetAction}`，默认通过 `WidgetPrimaryAction` 驱动这条链路，而不再只验证裸 `RunPanelIntent`
- Studio 当前也已补出 entitlement control / panel / startup preflight / session scheduler / session driver，使 entitlement panel 动作、启动预检、失败退避与下一次检查时机正式留在 Studio 应用层
- Studio 当前也已补出 `EntitlementSessionEvent::{SessionStarted, LoginCompleted, TimerElapsed, EntitlementCommandCompleted}`，让 bootstrap 自动预检和 entitlement 命令完成后的 session state 推进共享统一事件入口
- Studio 当前也已补出 `entitlement_session_host`，把 entitlement session event 与 entitlement panel 动作统一收口为宿主触发入口，并补出 `NetworkRestored` / `WindowForegrounded` 到既有 session tick 语义的映射；host 当前也已能把 `next_check_at` 收口为 timer arm 摘要、宿主建议 notice 与 `Schedule / Reschedule / Keep / Clear` 定时器决策，并进一步聚合成单一 `EntitlementSessionHostSnapshot`；当前又补出 `EntitlementSessionHostContext`，把当前已挂 timer 与上一份 host snapshot 的推进逻辑收回 host 模块，使 bootstrap / `main.rs` 与后续 GUI 宿主不再手写 control plane runtime 分发、多字段拼装或上下文记忆；bootstrap 最小入口当前也已能直接演练 `Reschedule / Keep` 语义而不是每次都从 `Schedule` 起步
- Studio 当前也已补出 `EntitlementSessionHostPresentation / TextView`，把 `main.rs` 仍在分散拼装的 entitlement host schedule/timer/notice 文本输出收回宿主模块，最小入口直接消费正式 host presentation
- Studio 当前也已补出 `EntitlementSessionHostRuntimeOutput / TimerEffect / Runtime`，把 `Schedule / Reschedule / Keep / Clear` 继续收成更贴近桌面宿主的 timer effect，并让 bootstrap 不再直接解释低层 host context
- Studio 当前也已把 bootstrap 最小入口继续收成可复用 session，并开始覆盖多宿主事件序列测试，验证共享 runtime 下 `TimerElapsed / NetworkRestored / WindowForegrounded` 的连续推进语义
- Studio 当前也已补出 `StudioRuntime`，把 bootstrap 内部可复用 session 上提为共享顶层入口，让 `run_studio_bootstrap(...)` 与 `main.rs` 统一走同一条 `StudioBootstrapTrigger -> entitlement host runtime / app facade` 分发链
- Studio 当前也已在 `StudioRuntime` 顶层补出 `StudioRuntimeOutput / StudioRuntimeEffect`，把 entitlement timer effect 正式作为运行时宿主输出暴露出来，后续真实桌面框架不必继续从 bootstrap 报告里翻字段取 effect
- Studio 当前也已给顶层 runtime 补出正式 `StudioRuntimeConfig / Trigger / Report / Dispatch` 命名，让真实宿主可先脱离 `StudioBootstrap*` 命名耦合，再继续推进具体 GUI 接线
- Studio 当前也已给顶层宿主输出补出最小 apply/ack 协议：`StudioRuntimeHostEffect` 现带稳定 `id`、`follow_up trigger` 与 `ack` 状态回报，先把 entitlement timer 接线闭环固定下来

基于上述进展，当前下一阶段计划调整为：

- 在已接通的 `main.rs` 最小 bootstrap 入口、`RunPanelWidgetModel`、`run_panel_driver`、entitlement session driver 与 `StudioBootstrapTrigger` 契约基础上，继续把 `StudioAppFacade + WorkspaceRunCommand + WorkspaceSolveService` 和 entitlement session event 接到更完整的 UI 运行命令、登录完成事件与定时调度入口
- 在已补出的 `StudioRuntimeConfig + Trigger + Report + Output` 共享入口基础上，继续决定真实桌面框架里的 timer 句柄、窗口生命周期事件与后台任务宿主如何接到这条统一 runtime 链路
- 在已补出的 host effect `id + follow_up + ack` 协议基础上，继续决定真实桌面框架里的 timer 句柄生命周期、apply 时机与 ack 回写时机
- 继续冻结运行结果派发、日志入口与后续异步执行边界
- 在不打乱当前边界的前提下，再恢复更完整的 Studio 交互流、联网提示位置与内核主线推进

## 推荐工作流

### 内核优先

先保证以下三层完全可本地跑通：

- `rf-thermo`
- `rf-flash`
- `rf-solver`

### FFI 第二

等 Rust 内核最小流程闭环稳定后，再导出 FFI。

### CAPE-OPEN 第三

等 FFI 稳定后，再实现 `.NET 10` 的 CAPE-OPEN 外壳。

### Studio 第四

Rust Studio UI 可以在 `M2` 后并行开始，但不要阻塞 `M4/M5` 主线。

## 建议的首批任务拆分

### Sprint A：基础骨架

- 初始化仓库
- 初始化 workspace
- 建立核心 crates
- 建立文档骨架

### Sprint B：热力学与闪蒸

- 二元组分参数结构
- Antoine
- Rachford-Rice
- TP Flash
- 黄金样例测试

### Sprint C：流程求解

- 流股对象
- 单元模块
- 流程连接
- 顺序模块法
- JSON 示例流程
- 最小集成测试与端到端闭环样例

### Sprint D：互操作边界

- Rust FFI
- .NET Adapter
- 基础互调测试

### Sprint E：PMC 暴露

- Unit Operation PMC
- 注册工具
- PME 冒烟验证

## 风险清单

| 风险 | 说明 | 应对策略 |
| --- | --- | --- |
| 范围膨胀 | 同时想做 PME、Thermo PMC、外部模型加载 | 严格锁死 MVP 不做项 |
| 热力学模型漂移 | 计算结果无法稳定复现 | 建立黄金样例测试 |
| FFI 设计过早复杂化 | 边界接口难维护 | 第一版只做句柄 + JSON |
| UI 干扰主线 | 过早投入画布与视觉细节 | UI 不阻塞内核与 CAPE-OPEN 主线 |
| PME 兼容不确定 | 外部软件实际行为偏标准之外 | 先选一个目标 PME 做主验证 |

## 建议的首批交付物

MVP 第一轮完成时，建议至少交付：

- 一个可运行的 Rust workspace
- 一个可运行的最小桌面程序壳
- 一个可求解的二元流程示例
- 一个 Rust FFI 动态库
- 一个 .NET 10 CAPE-OPEN Unit Operation PMC
- 一份目标 PME 验证说明

## Definition of Done

以下条件同时满足时，可认为 MVP 完成：

- Rust 内核可完成最小稳态流程求解
- Rust UI 可打开、编辑并运行至少一个示例流程
- .NET 10 适配层可成功调用 Rust 内核
- 自有 Unit Operation PMC 可被目标 PME 识别并调用
- 有最小自动化测试与人工验证记录
