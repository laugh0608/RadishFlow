# CAPE-OPEN PMC 适配层

更新时间：2026-09-06

> 本文保留已有能力与历史设计；自 2026-06-12 起业务功能开发停止。下文阶段、验证计划和历史状态不构成当前排期或操作授权，维护范围以 [当前状态](../status/current.md) 为准。

## 用途

用途：定义 `.NET 10` CAPE-OPEN / COM 适配层、PMC 注册、PME 人工验证和 Rust FFI 边界。
读者：负责 `adapters/dotnet-capeopen/`、`rf-ffi`、COM 注册脚本、DWSIM / COFE 验证和互操作测试的开发者、用户、AI / Agent。
不包含：Rust Core 内部 COM 语义、第三方 CAPE-OPEN 模型加载、完整 Thermodynamics PMC、PME 自动化产品化和安装器发布。

## 专题目标

- `.NET 10` 适配层继续作为 CAPE-OPEN / COM 语义唯一承载层。
- Rust Core 只通过 `rf-ffi` 暴露稳定 C ABI / JSON / error 边界。
- 当前阶段保持 DWSIM / COFE 关键 PME 兼容基线，只修真实 blocker。

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

## Native engine 生命周期静态风险

2026-09-06 核对 [RadishFlowNativeEngine](../../adapters/dotnet-capeopen/RadishFlow.CapeOpen.Adapter/RadishFlowNativeEngine.cs) 与 [P/Invoke 声明](../../adapters/dotnet-capeopen/RadishFlow.CapeOpen.Adapter/RfNativeMethods.cs)：调用通过 `DangerousGetHandle()` 取得裸句柄，`Dispose()` 直接释放底层 engine；该封装未统一检查释放状态，也未提供调用期引用保护或同一 engine 的调用串行化。

释放后再次调用、调用与 Dispose 并发或同一 engine 并发进入 native，存在失效指针或违反 Rust 可变访问约束的风险。PMC 外层已有生命周期守卫，不能据此推导独立公开 Adapter 的所有消费方式均已安全。[Microsoft SafeHandle 文档](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.safehandle.dangerousgethandle?view=net-10.0) 说明裸句柄可能失效，需正确保护其生命周期。

本次结论来自静态代码审阅，不是 Windows 崩溃复现或完整互操作审计。若后续获准修复，可评估让 P/Invoke 消费 SafeHandle 或成对引用保护，并明确已释放调用、调用期间 Dispose 和同一 engine 串行访问的契约与测试；只加 disposed 布尔值不足以解决调用与释放竞态。

已有 DWSIM / COFE smoke 继续只证明记录中的版本、场景和调用顺序，不扩张为任意线程或任意 PME 兼容承诺。

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
| M2 | Blocker 修复 | 仅修验证暴露的 discovery / activation / validate / calculate blocker |
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

## 历史组织记录

- 历史状态：Frozen / Blocker-only
- 最近更新：2026-06-14 从路线图和 current 状态中拆出 CAPE-OPEN 适配层专题入口。
- 历史下一步（未激活）：只在 `.NET` baseline、注册脚本或 PME 人工验证暴露真实 blocker 时推进。
