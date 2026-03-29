# Architecture Overview

更新时间：2026-03-29

## 目标

RadishFlow 的目标架构已经冻结为三层：

1. Rust Core
2. Rust Studio UI
3. .NET 10 CAPE-OPEN Bridge

第一阶段只要求三层边界清晰，不要求三层都立即进入完整实现。

## 当前仓库分层

### Rust Core

当前优先建设的 Rust Core 由以下 crate 构成：

| crate | 当前职责 | 当前状态 |
| --- | --- | --- |
| `rf-types` | 基础 ID、枚举、错误类型 | 已建立第一批基础类型 |
| `rf-model` | 组分、流股、单元、流程图对象模型 | 已建立第一批领域数据结构 |
| `rf-thermo` | 热力学数据结构与热力学接口 | 已建立最小 API，占位实现 |
| `rf-flash` | `TP Flash` 输入输出契约与求解器接口 | 已建立最小 API，占位实现 |
| `rf-unitops` | 单元模块行为抽象 | 仍为占位 |
| `rf-flowsheet` | 连接关系与图结构校验 | 仍为占位 |
| `rf-solver` | 顺序模块法求解器 | 仍为占位 |
| `rf-store` | JSON 存储与快照 | 仍为占位 |
| `rf-ffi` | Rust 与 .NET 的 C ABI 边界 | 仍为占位 |

### Rust Studio UI

当前 UI 相关 crate 只保留目录骨架，不进入主线：

| crate | 当前职责 | 当前状态 |
| --- | --- | --- |
| `rf-ui` | UI 状态与行为逻辑 | 占位 |
| `rf-canvas` | 流程图画布能力 | 占位 |
| `apps/radishflow-studio` | 桌面入口程序 | 最小壳程序 |

原因很直接：在 `M2/M3` 之前过早推进 UI，会掩盖内核尚未定型的问题。

### .NET 10 CAPE-OPEN Bridge

当前 `.NET 10` 目录只冻结项目边界，不进入复杂实现：

| 目录 | 当前职责 | 当前状态 |
| --- | --- | --- |
| `RadishFlow.CapeOpen.Interop` | 接口、GUID、异常语义 | 目录占位 |
| `RadishFlow.CapeOpen.Adapter` | PInvoke 与句柄封装 | 目录占位 |
| `RadishFlow.CapeOpen.UnitOp.Mvp` | 第一版自有 PMC | 目录占位 |
| `RadishFlow.CapeOpen.Registration` | 注册与反注册工具 | 目录占位 |
| `RadishFlow.CapeOpen.SmokeTests` | 冒烟测试 | 目录占位 |

## 当前关键边界

第一阶段必须严格遵守以下边界：

- Rust 不直接处理 COM、`IDispatch`、`VARIANT`、`SAFEARRAY`
- CAPE-OPEN/COM 适配全部放在 `.NET 10` 中
- 第一阶段只导出自有 Unit Operation PMC，不支持加载第三方 CAPE-OPEN 模型
- Rust 与 .NET 边界只允许句柄、基础数值、UTF-8 字符串和 JSON

## 当前开发策略

当前开发顺序不是“谁都做一点”，而是明显偏向内核优先：

1. 先稳定 `rf-types`、`rf-model`、`rf-thermo`、`rf-flash`
2. 再进入 `rf-unitops`、`rf-flowsheet`、`rf-solver`
3. 再做 `rf-ffi`
4. 最后才让 `.NET 10` 适配层真正接入运行时

这个顺序的目的，是把数值问题和 COM 互操作问题分开定位，避免后期排错混杂。

## 初始化阶段结论

截至 2026-03-29，仓库初始化阶段已经从“纯目录骨架”进入“可继续开发的基础结构”阶段。  
接下来的重点不再是增加目录，而是补充算法、测试和最小闭环。
