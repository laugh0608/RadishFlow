# Control Plane 后端服务专题

更新时间：2026-06-14

## 用途

用途：定义 RadishFlow 远端控制面后端服务的目标职责、边界和进入实现前的准备口径。  
读者：负责身份、授权、物性资产分发、离线租约、控制面 API 和部署边界的开发者、用户、AI / Agent。  
不包含：当前桌面求解热路径、完整在线求解服务、Radish.Auth 本体实现、后端运营 UI 和安装包发布。

## 专题层级

- 层级：二级功能
- 父专题：`docs/topics/property-basis-and-components.md`、`docs/topics/project-lifecycle-storage.md`
- 关联专题：`docs/architecture/auth-entitlement-architecture.md`

## 专题目标

- Control Plane 只承担身份消费、授权、租约、资产清单、派生包分发和审计，不吞掉本地求解器。
- 当前正式推荐采用 `ASP.NET Core / .NET 10`，并复用 `Radish.Auth` 作为 OIDC 身份源。
- 在进入实现前，先冻结 API、DTO、权限、部署和本地缓存边界。

## 当前实现快照

已完成 / 已有边界：

- 架构文档已冻结 OIDC Authorization Code + PKCE、Radish.Auth、离线租约和派生资产分发方向。
- Studio 已有 entitlement / auth cache / control plane client 相关应用层编排历史。
- 当前仓库不要求服务端与桌面端共存于同一个 monorepo。

已知缺口：

- 当前没有把 Control Plane 后端服务作为近期实现主线。
- API schema、数据库、部署、审计和管理台仍需独立设计。

## 范围

本专题纳入：

- 当前用户授权查询。
- RadishFlow 专属 entitlement。
- 物性资产 manifest。
- 派生 package 下载票据或签名 URL。
- 离线租约刷新。
- 审计日志。

本专题不纳入：

- 主求解循环在线化。
- 全量 TP Flash 在线 RPC。
- Radish.Auth 本体重写。
- 后端 Web 管理台实现。
- 第三方 Property Package 加载。

## 设计边界

### 数据与状态

- 服务端保存授权、租约、资产清单和审计。
- 桌面端只保存必要 auth cache、派生包缓存和离线租约摘要。
- 高价值原始物性资产默认不完整下发到客户端。

### 命令与接口

- 身份协议使用 OIDC / OAuth 2.0。
- Control Plane API 只消费已认证主体和 claims。
- 派生包下载优先走对象存储 / CDN / 下载网关短时票据。

### UI 与交互

- Studio 只展示用户可判断的授权 / 资产状态摘要。
- 后端管理 UI 另见 `control-plane-web-ui.md`。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | API 与 DTO 设计 | entitlement、manifest、lease、download ticket 和 audit 边界明确 |
| M2 | 最小服务实现 | 可在测试环境返回当前用户授权与可下载派生包清单 |
| M3 | 桌面端联调 | Studio 能刷新授权、下载派生包并处理离线租约 |

## 验收标准

- 不把求解热路径变成远端硬依赖。
- 桌面端断网且租约有效时仍可使用已授权本地派生资产。
- 过期租约不能继续加载受控资产。
- 原始高价值物性资产不默认完整下发。

## 验证计划

- 后端实现前先补 API contract test 计划。
- 桌面联调需要 focused test 覆盖 entitlement / manifest / lease 状态映射。
- 涉及网络或部署时需单独说明真实环境验证。

## 状态记录

- 当前状态：Backlog
- 最近更新：2026-06-14 建立二级功能专题。
- 下一步：只有当控制面进入近期主线时，先补 API / DTO 设计，不直接写服务代码。
