# Architecture Overview

更新时间：2026-03-31

## 目标

RadishFlow 的目标架构已经冻结为“桌面端三层 + 外部控制面”：

1. Rust Core
2. Rust Studio UI
3. .NET 10 CAPE-OPEN Bridge

第一阶段只要求桌面三层边界清晰，不要求三层都立即进入完整实现。

同时，当前还补充冻结一个 **外部控制面**：

4. RadishFlow Control Plane (`ASP.NET Core / .NET 10`, External)

这不是桌面进程内部的新层，而是产品外部依赖的服务平面，用于承担：

- OIDC 登录
- RadishFlow 专属授权
- 受控物性资产清单与租约
- 派生数据包分发
- 审计与撤销入口

不承担：

- 本地主求解循环
- CAPE-OPEN/COM 适配

## 当前仓库分层

### Rust Core

当前优先建设的 Rust Core 由以下 crate 构成：

| crate | 当前职责 | 当前状态 |
| --- | --- | --- |
| `rf-types` | 基础 ID、枚举、错误类型 | 已建立第一批基础类型 |
| `rf-model` | 组分、流股、单元、流程图对象模型 | 已建立第一批领域数据结构 |
| `rf-thermo` | 热力学数据结构与热力学接口 | 已建立最小 API、内存 provider、基于本地缓存目录/授权缓存索引的 `PropertyPackageProvider` 实现，并用首个真实样例包覆盖装载测试 |
| `rf-flash` | `TP Flash` 输入输出契约与求解器接口 | 已建立最小 API，并已实现最小二元 `TP Flash`、Rachford-Rice 与黄金样例 |
| `rf-unitops` | 单元模块行为抽象 | 已建立内建单元规范、统一流股输入输出接口，并实现 `Feed`、`Mixer`、`Heater/Cooler`、`Flash Drum` 的最小行为边界 |
| `rf-flowsheet` | 连接关系与图结构校验 | 已建立首轮材料端口连接校验，覆盖 canonical port signature、流股存在性与“一股一源一汇”约束 |
| `rf-solver` | 顺序模块法求解器 | 已建立首轮无回路顺序模块法，可执行 `Feed + Mixer + Flash Drum` 与 `Feed -> Heater -> Flash Drum` 闭环，并产出最小 `SolveSnapshot` |
| `rf-store` | JSON 存储与授权缓存索引 | 已建立项目文件 / 授权缓存 / 本地包 `manifest.json` / `payload.rfpkg` 的 JSON 读写、迁移分发、版本校验与相对路径布局 |
| `rf-ffi` | Rust 与 .NET 的 C ABI 边界 | 仍为占位 |

### Rust Studio UI

当前 UI 相关 crate 已进入“状态骨架与应用边界冻结”阶段，但仍未进入具体界面实现主线：

| crate | 当前职责 | 当前状态 |
| --- | --- | --- |
| `rf-ui` | UI 状态与行为逻辑 | 已建立 `AppState`、授权态、求解态与控制面 DTO 骨架 |
| `rf-canvas` | 流程图画布能力 | 占位 |
| `apps/radishflow-studio` | 桌面入口程序 | 已建立 auth cache sync 桥接、控制面 HTTP client、entitlement / manifest / lease / offline refresh 编排、下载获取抽象、基于 `reqwest + rustls` 的真实 HTTP transport、HTTP 请求/响应适配层、可重试/不可重试失败分类、下载 JSON 到本地 payload DTO 的协议映射、摘要校验、失败回滚与测试 |

原因很直接：在 `M2/M3` 之前过早推进 UI，会掩盖内核尚未定型的问题。

不过 App 架构层面的关键口径已经开始冻结，当前包括：

- 单文档工作区优先
- 字段级草稿提交
- `SimulationMode` / `RunStatus` 分离
- 独立 `SolveSnapshot`
- OIDC / 授权 / 远端资产保护作为外部控制面，而不是塞进 Rust Core

这些决定的目的是先把 UI 和求解层之间的长期接口边界定清楚，再决定具体控件和交互实现。

### .NET 10 CAPE-OPEN Bridge

当前 `.NET 10` 目录只冻结项目边界，不进入复杂实现：

| 目录 | 当前职责 | 当前状态 |
| --- | --- | --- |
| `RadishFlow.CapeOpen.Interop` | 接口、GUID、异常语义 | 目录占位 |
| `RadishFlow.CapeOpen.Adapter` | PInvoke 与句柄封装 | 目录占位 |
| `RadishFlow.CapeOpen.UnitOp.Mvp` | 第一版自有 PMC | 目录占位 |
| `RadishFlow.CapeOpen.Registration` | 注册与反注册工具 | 目录占位 |
| `RadishFlow.CapeOpen.SmokeTests` | 冒烟测试 | 目录占位 |

### External .NET 10 Control Plane

外部控制面当前不在本仓库内实现，但系统级职责与技术口径已经冻结：

| 组件 | 当前职责 | 当前状态 |
| --- | --- | --- |
| `Radish.Auth` | OIDC 身份源与统一登录 | 外部平台依赖 |
| `RadishFlow Control Plane` | `ASP.NET Core / .NET 10` 授权、manifest、lease、offline refresh、audit API | 体系结构已冻结；当前仓库已有客户端 DTO 与 HTTP 接线 |
| `Asset Delivery` | 物性派生包下载入口，优先承载为对象存储 / CDN / 下载网关 | 体系结构已冻结；当前仓库只消费下载协议 |

这里的关键点不是“把服务端代码也搬进当前 Monorepo”，而是先把客户端、桥接层与控制面之间的长期契约固定住。

## 当前关键边界

第一阶段必须严格遵守以下边界：

- Rust 不直接处理 COM、`IDispatch`、`VARIANT`、`SAFEARRAY`
- CAPE-OPEN/COM 适配全部放在 `.NET 10` 中
- 第一阶段只导出自有 Unit Operation PMC，不支持加载第三方 CAPE-OPEN 模型
- Rust 与 .NET 边界只允许句柄、基础数值、UTF-8 字符串和 JSON
- 桌面端登录统一走 OIDC Authorization Code + PKCE，不内置长期 `client_secret`
- 高价值物性资产不默认完整下发到客户端
- 远端服务只承担控制面与资产分发面，不吞掉本地求解热路径
- 外部控制面建议采用 `ASP.NET Core / .NET 10`，不额外引入新的 Go 服务主线
- 资产分发优先采用对象存储 / CDN + 短时票据，而不是让控制面 API 长期直出大文件

## 当前开发策略

当前开发顺序不是“谁都做一点”，而是明显偏向内核优先：

1. 先稳定 `rf-types`、`rf-model`、`rf-thermo`、`rf-flash`
2. 再进入 `rf-unitops`、`rf-flowsheet`、`rf-solver`
3. 再做 `rf-ffi`
4. 最后才让 `.NET 10` 适配层真正接入运行时
5. 在桌面边界稳定后，按既有契约单独推进外部 `.NET 10` 控制面落地与部署

这个顺序的目的，是把数值问题和 COM 互操作问题分开定位，避免后期排错混杂。

## 当前阶段优先级调整

虽然主线顺序仍然保持不变，但当前短期优先级已调整为“地基建设优先”。

当前应优先推进的内容：

- 仓库治理
- 分支与 PR 规则
- 基础 CI
- 代码与文档格式规范
- App 架构规划
- 设计与进度文档完善

原因：

- 当前仓库仍处于早期演化阶段，过早推进主线功能，后续反而要回头返工协作规则和工程基础设施
- App 主界面、内核、适配层都还没有稳定的工程协作口径，先定规则更划算

这并不意味着放弃主线，而是先把主线开发赖以生存的仓库地基补完整。

## 初始化阶段结论

截至 2026-03-29，仓库初始化阶段已经从“纯目录骨架”进入“可继续开发的基础结构”阶段。  
接下来的重点不再是增加目录，而是继续补充算法、测试覆盖和最小闭环样例厚度。
