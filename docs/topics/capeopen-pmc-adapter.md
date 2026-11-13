# CAPE-OPEN PMC 适配层

更新时间：2026-11-13

> 本文定义该专题的能力、开发范围与验收要求；具体迭代切片和优先级以 [当前状态](../status/current.md) 为准。

## 用途

用途：定义 `.NET 10` CAPE-OPEN / COM 适配层、PMC 注册、PME 人工验证、Rust FFI 边界及未来 COBIA 接入计划。
读者：负责 `adapters/dotnet-capeopen/`、`rf-ffi`、COM 注册脚本、DWSIM / COFE 验证和互操作测试的开发者、用户、AI / Agent。
不包含：Rust Core 内部 COM 语义、第三方 CAPE-OPEN 模型加载、完整 Thermodynamics PMC、PME 自动化产品化和安装器发布。

变量浏览树、模块创建与参数读写等通用 COM Automation 已进入 [长期自动化目标](../architecture/simulation-platform.md#统一-api-的目标形态)，未来建立独立应用自动化适配；它不等同于 CAPE-OPEN 接口，也不把本专题的既有 PMC 测试视为通用自动化已交付。

## 专题目标

- 当前 `.NET 10` 适配层继续作为 CAPE-OPEN / COM 语义承载层；未来 COBIA 候选独立适配，COM 语义仍只留在 `.NET`。
- Rust Core 只通过 `rf-ffi` 暴露稳定 C ABI / JSON / error 边界。
- 保持 DWSIM / COFE 关键 PME 兼容基线，推进 native 生命周期、调用可靠性与接口契约的正常迭代。

## 当前实现快照

已完成：

- `rf-ffi` JSON/error 和 native 装载路径已有回归基线。
- `.NET 10` Unit Operation PMC 骨架已形成。
- DWSIM / COFE discovery、activation、placement、端口连接和最小 `Validate / Calculate` 主路径有历史验证；不覆盖下述收紧后的物料契约。
- 注册脚本默认 dry-run，并要求显式 `--execute` + `--confirm` 才执行。
- `docs/capeopen/pme-validation.md` 保存 PME 人工验证 runbook。

已知缺口：

- 不推进第三方模型加载。
- 不推进完整 Thermodynamics PMC。
- Windows `.NET` 基线可在对应 Windows runner 验证；真实 PME 验收使用目标厂商支持的 Windows x64 环境，runner 构建不等于 PME 已验收。
- 当前不做正式发布、安装器或自动 COM 注册。

## 物料边界与成功语义治理

- 方案：2026-11-13 已确认，严格单入口、单出口，取消隐式合并和失败兜底。
- 执行：实现及本机定向验证完成，**待 Windows / 真实 PME 验收**；这不是完整标准符合性或工业物理精度声明。
- 范围：`.NET UnitOp.Mvp` 物料访问及 Interop 标准相目录消费声明、组分映射、输入副本、出口回读、成功 / 失败及失效语义；ContractTests、SmokeTests、SampleHost 和样例随之更新。Rust Core、COM GUID / 接口、项目 schema、依赖不变。
- 稳定契约见 [物料边界](../capeopen/boundary.md#pmc-单入口单出口物料契约)，操作步骤见 [本轮 PME 矩阵](../capeopen/pme-validation.md#单入口单出口复验矩阵)。

| 验收面 | 通过标准 | 当前证据 / 缺口 |
| --- | --- | --- |
| 配置形状 | 单进单出；多进、多出、无出口、缺引用在运行前拒绝 | 本机契约覆盖，旧闪蒸样例为反向用例 |
| 组分与输入 | 重排保持各组分摩尔流量；缺失 / 额外（含零量）/ 歧义 / 猜测身份拒绝；读取错误无默认输入兜底 | 本机完整映射、数值、读取异常与输入副本测试 |
| 出口发布 | 写入、闪蒸、回读、相态或组分闭合失败均不成功；候选无效时尚未修改 Product | 本机故障注入及来源断言；COM 真实对象待验 |
| 零流量 | 有效组成与 TP，保持零流量仍执行平衡；模型限制明确报错 | 发布器及 native Feed→Heater 零流量已验；真实宿主待补 |
| 协议 | 1.0 / 1.1 分别成功、失败传播；双接口对象不因错误跨版本重试 | 托管替身已验，真实 marshalling 待 Windows |
| 生命周期 | 成功→失败→恢复、改输入重算、断连重连、终止；旧结果不能作为当前值 | 本机 contract 与 UnitOp smoke 已验；真实 COM 引用释放待验 |
| 目标 PME | DWSIM / COFE 单出口正向、双出口拒绝及关闭释放 | 未执行，历史双出口通过记录不沿用 |

验证明细见 [W46](../devlogs/2026-11/2026-W46.md#2026-11-13cape-open-物料边界治理)。下一步在告知所有者后补 Windows `.NET` 完整基线和 Windows x64 上的 DWSIM / COFE 真实矩阵；进入该环境前不宣布本切片验收完成。

## COBIA 分阶段接入计划

- 方案：2026-11-13 已完成只读评估，所有者确认将结论和分阶段计划归档；Mac 为主开发，Windows x64 承担真实 PME 验收。
- 执行：仅文档规划，**暂不接入 COBIA、不安装 SDK**；不增加依赖、注册组件或启动虚拟机。下表均为后续候选工作，未排期、未启动，不因归档自动获得实施授权。
- 依据与边界：[平台 / 工具链评估及适配方向](../capeopen/boundary.md#平台与工具链评估2026-11-13)；操作和证据口径见 [平台分工](../capeopen/pme-validation.md#开发平台与验收平台)。

| 阶段 | 工作范围与启动条件 | 退出依据 | 状态 |
| --- | --- | --- | --- |
| 接入条件核实 | 明确首个目标 PMC / PME；向上游核实 macOS ARM64 SDK / Runtime、源码获取、构建与分发条件 | 得到可复现构建所需材料与支持范围；无法满足时保留缺口，不扩大为移植工程 | 待启动 |
| 独立可行性验证 | 条件明确并确认实施范围后，在 macOS ARM64 验证初始化、发现 / 创建、一次调用、错误传播与释放；不接 Studio 或编辑 UI | 原生调用闭环可重复运行，版本、架构、失败与释放证据齐全 | 待启动 |
| 有限 PMC 接入 | 前阶段通过，且共享执行契约明确后，复用单入口单出口案例建立独立 COBIA 适配；优先评估 Rust 绑定 | 与 COM 路径的受支持输入、结果和失败语义一致；目标平台 native 链路通过 | 待启动 |
| 产品化与平台验收 | 有明确用户 / 宿主需求后，补 Windows x64 真实 PME、COMBIA、持久化、线程 / 回调和安装分发 | 每个声明支持的系统 / 架构及目标 PME 有独立证据；未测平台继续列待验 | 待启动 |

本计划不替换当前 COM 物料治理验收，不作为 Rust Core / Studio 主线开发前置条件。Linux 按明确组件需求落实工具链与验证，不由 macOS 或 Windows 结果推导支持。第三方 Unit Operation / Property Package 宿主能力仍按下节与 MC3 另行规划；完整 Thermodynamics PMC 不随本计划自动进入范围。

## 宿主侧加载的规划关系

当前链路是外部 PME 发现和调用 RadishFlow 自有 PMC；未来 RadishFlow 发现、激活并使用第三方单元 / 物性包是反向宿主能力，尚未实现。`SampleHost` 和既有 DWSIM / COFE 验证不能替代该方向的验收，也不能证明完整 Thermodynamics PMC 已交付。

2026-09-29 将模型发现与装配归入 [MC0—MC3](modeling/model-catalog-and-runtime.md#分阶段切片)；若首个外部模型选择 CAPE-OPEN，先明确 Windows 位数、接口版本、Material Object 服务、组分 / 相态映射、焓基准、线程、引用释放和保存恢复，再建立独立宿主实施专题。对应概念与三类符合性证据见 [边界说明](../capeopen/boundary.md#互操作方向与符合性证据)。这不扩大本专题的当前 PMC 实现范围或默认注册权限。

## Native engine 生命周期

2026-09-12 修复 [RadishFlowNativeEngine](../../adapters/dotnet-capeopen/RadishFlow.CapeOpen.Adapter/RadishFlowNativeEngine.cs) 的公开调用边界：同一 engine 的完整操作与 `Dispose()` 共用实例锁；P/Invoke 直接接收 `RfNativeEngineHandle`，由 SafeHandle marshalling 保护 native 调用期间的句柄引用。Rust C ABI 和 COM 接口形状保持不变。

- 已进入的调用完成后，等待中的 Dispose 才能释放句柄；同一 engine 的调用串行执行。
- 释放后使用有效参数调用任何 native 操作都会抛出 `ObjectDisposedException`，重复 Dispose 可安全返回；无效参数仍按既有参数校验抛错。
- native 错误消息和 JSON 在同一操作锁内读取，避免被另一线程的调用覆盖。
- 锁以单次 Adapter 操作为边界；调用方若需要跨 Load / Solve / Get 多次调用的事务一致性，仍应管理自己的会话编排。
- Rust C ABI 的其他直接消费者仍须遵守有效句柄、唯一所有权和串行访问要求；本次改动不让任意裸指针调用自动安全。

Adapter smoke 新增释放后调用、调用期间 Dispose、同一 engine 串行化和并发错误隔离四个场景。验证状态见当前周志；DWSIM / COFE 历史 smoke 仍只证明所记录宿主与场景，不扩张为任意 PME 兼容承诺。

## 用户路径

1. 构建 Rust native 产物和 `.NET 10` 适配层。
2. 通过注册脚本 dry-run 审查注册计划。
3. 在显式确认下注册 / 反注册当前 PMC。
4. 在 DWSIM / COFE 中发现、放置、连接、Validate / Calculate。
5. 若验证失败，按 PME runbook 归类为注册、discovery、activation、material object、validate 或 calculate 问题。

## 范围

本专题纳入：

- `rf-ffi` ABI、JSON request / response、error contract。
- `.NET 10` CAPE-OPEN Unit Operation PMC。
- COM 注册 / 反注册脚本和 dry-run / execute 门控。
- DWSIM / COFE 人工验证基线。
- Windows `.NET` contract / smoke 验证入口。

本专题不纳入：

- Rust Core 直接处理 COM。
- 第三方 CAPE-OPEN 单元加载。
- 第三方 Thermo / Property Package 加载。
- 完整 Thermodynamics PMC。
- 自动驱动外部 PME 的产品化测试平台。
- 安装器、发布包和默认本机注册。

## 设计边界

### 数据与状态

- Rust Core 不保存 CAPE-OPEN / COM 宿主状态。
- `.NET` 适配层把宿主语义收口成只读模型、显式请求规划模型和 outcome 模型。
- PME 验证记录写入 `docs/capeopen/` 或 `examples/pme-validation/`，不写入 `status/current.md` 长流水。

### 命令与接口

- Rust 与 `.NET` 之间只通过 `rf-ffi` 边界通信。
- 注册脚本必须保留 dry-run、preflight、显式确认和失败 rollback。
- 真实 COM 注册、反注册和 PME 操作必须先告知用户。

### UI 与交互

- Studio UI 不承载 CAPE-OPEN / COM 语义。
- 只有需要向用户解释 PMC 验证状态时，才在文档或 runbook 中展示摘要。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 基线冻结 | DWSIM / COFE 关键人工验证路径记录完整 |
| M2 | 可靠性完善 | 生命周期、释放与并发调用契约有回归覆盖；discovery / activation / validate / calculate 基线保持通过 |
| M3 | 发布前复验 | 进入正式 release 节点前，按 runbook 重新执行 Windows / PME 验证 |

## 验收标准

- `.NET` build 和 contract tests 通过。
- PME 能发现、放置并执行最小 validate / calculate。
- 注册脚本不会默认修改本机 COM 环境。
- Rust Core 没有 COM 语义倒灌。
- 验证失败能按 runbook 分类定位。

## 验证计划

- macOS / Linux：可做脚本语法、Rust 验证及关闭 COM hosting 的托管 contract / native smoke；不能替代 Windows COM 验证。
- Windows：执行 `.NET` build、contract tests、smoke tests；真实 PME 使用 Windows x64，Windows ARM 仿真只作补充证据，详见 [平台分工](../capeopen/pme-validation.md#开发平台与验收平台)。
- 仓库级：涉及适配层关键变更时优先执行 `pwsh ./scripts/check-repo.ps1` 或 Windows CI 基线。

## 状态记录

- 当前状态：Active
- 历史更新：2026-09-12，native 生命周期保护通过 macOS Adapter smoke 与 Windows ARM64 构建、35 项 contract、Adapter smoke；运行环境和验证边界见 [周志](../devlogs/2026-09/2026-W37.md#2026-09-12-第一轮可靠性修复)。
- 当前切片及下一步：见上文[物料边界与成功语义治理](#物料边界与成功语义治理)，本机通过、Windows / PME 待验。
