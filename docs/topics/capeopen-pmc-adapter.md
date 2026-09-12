# CAPE-OPEN PMC 适配层

更新时间：2026-09-12

> 本文定义该专题的能力、开发范围与验收要求；具体迭代切片和优先级以 [当前状态](../status/current.md) 为准。

## 用途

用途：定义 `.NET 10` CAPE-OPEN / COM 适配层、PMC 注册、PME 人工验证和 Rust FFI 边界。
读者：负责 `adapters/dotnet-capeopen/`、`rf-ffi`、COM 注册脚本、DWSIM / COFE 验证和互操作测试的开发者、用户、AI / Agent。
不包含：Rust Core 内部 COM 语义、第三方 CAPE-OPEN 模型加载、完整 Thermodynamics PMC、PME 自动化产品化和安装器发布。

## 专题目标

- `.NET 10` 适配层继续作为 CAPE-OPEN / COM 语义唯一承载层。
- Rust Core 只通过 `rf-ffi` 暴露稳定 C ABI / JSON / error 边界。
- 保持 DWSIM / COFE 关键 PME 兼容基线，推进 native 生命周期、调用可靠性与接口契约的正常迭代。

## 当前实现快照

已完成：

- `rf-ffi` JSON/error 和 native 装载路径已有回归基线。
- `.NET 10` Unit Operation PMC 骨架已形成。
- DWSIM / COFE discovery、activation、placement、端口连接和最小 `Validate / Calculate` 主路径已阶段性验证。
- 注册脚本默认 dry-run，并要求显式 `--execute` + `--confirm` 才执行。
- `docs/capeopen/pme-validation.md` 保存 PME 人工验证 runbook。

已知缺口：

- 不推进第三方模型加载。
- 不推进完整 Thermodynamics PMC。
- Windows `.NET` / PME 验证需在真实 Windows 或 GitHub Windows runner 完成。
- 当前不做正式发布、安装器或自动 COM 注册。

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

- macOS / Linux：可做脚本语法和 Rust 仓库级验证。
- Windows：执行 `.NET` build、contract tests、smoke tests 和必要 PME 人工验证。
- 仓库级：涉及适配层关键变更时优先执行 `pwsh ./scripts/check-repo.ps1` 或 Windows CI 基线。

## 状态记录

- 当前状态：Active
- 最近更新：2026-09-12，native 生命周期保护通过 macOS Adapter smoke 与 Windows ARM64 构建、35 项 contract、Adapter smoke；运行环境和验证边界见 [周志](../devlogs/2026-09/2026-W37.md#2026-09-12-第一轮可靠性修复)。
- 下一步：按选定功能切片维护上述回归；后续涉及宿主行为或发布时执行对应 PME 复验。
