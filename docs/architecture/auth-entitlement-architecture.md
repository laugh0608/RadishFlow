# Auth And Entitlement Architecture

更新时间：2026-03-29

## 目标

本文档用于冻结 RadishFlow 桌面端的登录、授权、远端物性资产保护与本地求解边界。

这里讨论的不是“通用账号系统怎么做”，而是以下问题在 RadishFlow 中应该如何分层：

- 桌面端如何登录
- 桌面端如何获取授权
- 高价值物性数据如何避免完整落到客户端
- 哪些能力必须本地运行，哪些能力可以放在远端控制面
- `rf-ui`、`rf-store`、`rf-thermo`、`rf-solver` 与外部服务如何解耦

本文档的结论与现有三层产品架构并不冲突：

1. Rust Core
2. Rust Studio UI
3. .NET 10 CAPE-OPEN Bridge

认证、授权与远端资产保护属于 **外部控制面**，不是 RadishFlow 桌面进程内部的新业务层。

## 背景

RadishFlow 虽然是桌面客户端，但后续会承载一部分高价值、不可完全公开的物性资产。  
如果这些资产以“完整原始数据 + 本地永久明文缓存”的方式下发，最终仍会被提取。

因此当前正式采用以下判断：

- 桌面客户端可以做登录和授权，但不能被当成可以长期保密的运行环境
- 对真正高价值的原始资产，不应默认完整下发到客户端
- 求解热路径仍应以本地执行为主，不能把主求解循环设计成在线 RPC
- 远端服务应优先承担“控制面”职责，而不是吞掉整个本地求解器

## 参考来源

本方案优先参考现有 `Radish` 平台的统一身份能力，而不是另起一套账号体系。

当前参考资产包括：

- `D:\Code\Radish\Docs\guide\authentication.md`
- `D:\Code\Radish\Docs\architecture\overview.md`
- `D:\Code\Radish\Frontend\radish.client\src\services\auth.ts`
- `D:\Code\Radish\Frontend\radish.client\src\services\tokenService.ts`

从 `Radish` 当前实现中，RadishFlow 直接继承以下方向：

- 统一使用 OIDC / OAuth 2.0
- 已存在的 `/connect/authorize`、`/connect/token`、`/connect/userinfo` 等标准端点
- `openid profile offline_access` 这一类标准 scope 组合
- Access Token + Refresh Token 的标准续期思路
- 以独立 Auth 服务作为统一身份源，而不是把认证逻辑塞回业务 API

但需要明确一条关键差异：

- `Radish` 当前的前端示例包含浏览器 `localStorage` 的 token 管理逻辑，那是 Web 客户端方案
- RadishFlow 是桌面原生客户端，**不能**照搬浏览器 token 存储策略

## 当前正式决策

截至 2026-03-29，以下口径正式冻结。

### 1. 身份协议

- RadishFlow 桌面端统一采用 `OIDC Authorization Code + PKCE`
- 桌面端被视为 `public client`，不配置长期 `client_secret`
- 登录流程必须使用 **系统浏览器**，不采用内嵌 WebView 作为默认方案
- 回调优先采用 **loopback redirect**，即：
  - `http://127.0.0.1:{ephemeral-port}/oidc/callback`
  - `http://localhost:{ephemeral-port}/oidc/callback`
- 仅在 loopback 方案确实受限时，才评估自定义 URI scheme

### 2. 身份源与控制面分工

- `Radish.Auth` 继续作为统一身份源与 OIDC 服务器
- RadishFlow 不自建第二套用户系统
- RadishFlow 另建自己的 **授权与资产控制面 API**，但可以部署在 Radish 平台内
- 当前建议把这部分能力称为 `RadishFlow Control Plane`

控制面职责：

- 读取当前用户身份
- 计算并返回 RadishFlow 专属授权
- 返回可访问的物性资产清单
- 下发派生数据包下载信息
- 管理离线租约
- 记录审计日志

不承担：

- 主求解循环
- 全量 TP Flash 在线求解
- 对每一次单元迭代进行远端同步调用

### 3. 数据分级

RadishFlow 相关数据当前正式划分为三类：

#### A 级：绝对核心资产

示例：

- 原始物性数据库
- 高价值专有参数源数据
- 不希望被完整提取的实验或商业数据

处理原则：

- 默认不完整下发到客户端
- 尽量保留在服务端
- 若桌面端必须使用，则优先转化为裁剪后的派生资产或远端求值接口

#### B 级：授权后可下发的派生资产

示例：

- 面向特定物系、温压范围或模型版本裁剪后的参数包
- 本地求解所需的衍生系数包
- 限时、限版本、限授权的加密缓存包

处理原则：

- 允许授权后下载到客户端
- 允许本地缓存
- 必须接受“只要客户端最终要用明文参与计算，就不能承诺绝对防提取”
- 主要防线是授权、裁剪、限时、审计和撤销，而不是幻想客户端绝对保密

#### C 级：普通非敏感资产

示例：

- UI 配置
- 非敏感元数据
- 一般项目文件

处理原则：

- 正常本地保存即可

### 4. 求解热路径

当前正式冻结以下边界：

- `rf-solver`、`rf-thermo`、`rf-flash` 的主调用链仍然以本地执行为主
- 不把远端 API 设计成主求解循环中的硬依赖
- 桌面端断网后，只要离线租约与本地派生资产仍有效，已授权的本地求解能力就应继续工作
- 远端服务只允许承担控制面、分发面或极少数高价值特性的补充求值能力

这条边界的目的，是避免以下问题提前压入 MVP：

- 求解延迟不可控
- 服务端成本随着迭代次数爆炸
- 离线能力消失
- CAPE-OPEN / PME 集成场景下稳定性大幅下降

### 5. 离线租约

为了兼顾桌面可用性与授权控制，当前建议引入 **离线租约** 概念。

最小行为：

- 用户在线登录后，控制面返回授权快照与离线可用截止时间
- 只要离线租约仍有效，桌面端可以继续使用已缓存的派生物性包
- 租约过期后，桌面端不得继续加载受控资产，直到重新联机刷新授权

当前不冻结：

- 是否做设备绑定
- 绑定采用机器证书、系统 SID、TPM 还是更轻量的安装实例 ID
- 是否允许管理员手工吊销单设备租约

这些问题后续再细化，但不影响当前控制面边界先冻结。

## 目标拓扑

当前建议的运行拓扑如下：

```text
RadishFlow Studio (Desktop)
        │
        ├─ OIDC Login
        │    └─ Radish.Auth
        │
        ├─ Entitlement / Lease / Manifest
        │    └─ RadishFlow Control Plane
        │
        ├─ Derived Property Package Download
        │    └─ RadishFlow Asset Delivery
        │
        └─ Local Solver / Local Thermo / Local UI
             └─ 使用本地已授权的派生资产
```

可选扩展：

```text
RadishFlow Studio
        │
        └─ Premium Remote Property Evaluation
             └─ 仅面向极少数绝不下发的高价值能力
```

## 桌面端认证流程

当前建议流程：

1. 桌面端启动登录动作
2. 本地生成 `code_verifier` / `code_challenge`
3. 拉起系统浏览器，跳转到 `Radish.Auth /connect/authorize`
4. 用户在浏览器中完成登录
5. Auth Server 回调到本地 loopback 地址
6. 桌面端用授权码 + `code_verifier` 调用 `/connect/token`
7. 获得 `access_token` / `refresh_token`
8. 桌面端调用 `/connect/userinfo` 与 RadishFlow Control Plane
9. 获取用户信息、授权快照、派生资产清单和离线租约

当前冻结补充：

- 桌面端不保存用户密码
- 桌面端不保存长期客户端密钥
- Access Token / Refresh Token 只允许存入操作系统安全存储

建议的本地存储口径：

- Windows：DPAPI 或 Credential Locker
- macOS：Keychain
- Linux：Secret Service / keyring

## 控制面接口边界

当前建议最小 API 集合如下。

### OIDC 相关

由 `Radish.Auth` 提供：

- `/.well-known/openid-configuration`
- `/connect/authorize`
- `/connect/token`
- `/connect/userinfo`
- `/connect/logout` 或等价 end-session 端点

### RadishFlow Control Plane

建议最小资源：

- `GET /api/radishflow/entitlements/current`
  - 返回当前用户的授权快照
- `GET /api/radishflow/property-packages/manifest`
  - 返回当前授权可见的派生资产清单
- `POST /api/radishflow/property-packages/{packageId}/lease`
  - 返回某个派生资产包的下载租约或访问票据
- `POST /api/radishflow/offline-leases/refresh`
  - 刷新桌面端离线租约
- `POST /api/radishflow/audit/usage`
  - 上传关键受控资产使用审计

### 可选高级接口

仅在确有必要时再引入：

- `POST /api/radishflow/property-eval`
  - 对极少数绝不下发的高价值物性能力做远端求值

## 核心对象草案

### `EntitlementSnapshot`

建议最小字段：

```json
{
  "subjectId": "user-123",
  "tenantId": "tenant-001",
  "issuedAt": "2026-03-29T10:00:00Z",
  "expiresAt": "2026-03-29T11:00:00Z",
  "offlineLeaseExpiresAt": "2026-04-05T10:00:00Z",
  "features": [
    "desktop-login",
    "local-thermo-packages",
    "capeopen-export"
  ],
  "allowedPackageIds": [
    "thermo-basic-v1",
    "binary-hydrocarbon-lite-v1"
  ]
}
```

语义边界：

- 描述当前用户“被允许做什么”
- 不直接携带完整物性数据
- 不承担本地缓存文件索引

### `PropertyPackageManifest`

建议最小字段：

```json
{
  "packageId": "binary-hydrocarbon-lite-v1",
  "version": "2026.03.1",
  "classification": "derived",
  "source": "download",
  "hash": "sha256:...",
  "sizeBytes": 123456,
  "expiresAt": "2026-04-05T10:00:00Z"
}
```

语义边界：

- 描述可见资产，不直接下发资产内容
- 用于让桌面端知道“哪些包可下、版本是什么、有效期多久”

### `PropertyAssetSource`

建议先冻结为三类：

- `LocalBundled`
  - 随安装包提供的非敏感或开发资产
- `RemoteDerivedPackage`
  - 授权后下载的派生包
- `RemoteEvaluationService`
  - 不下发数据，只允许远端求值

## RadishFlow 内部模块边界

### `rf-ui`

职责：

- 发起登录
- 持有登录态、授权态和资产目录展示态
- 提示用户租约过期、需要重新联机或需要重新拉取资产

不应承担：

- 直接在 UI 里写 OIDC 协议细节实现
- 直接处理加密资产解包细节
- 把授权异常硬编码进求解器逻辑

当前补充说明：

- 现有 `AppState` 文档尚未正式加入 `AuthSessionState`
- 后续若接入桌面登录，应把登录态和授权态放在 `AppState` 外层应用状态，而不是混进 `FlowsheetDocument`

### `rf-store`

职责：

- 持久化普通项目文件
- 持久化本地资产清单索引、授权缓存元信息和下载包元信息
- 区分“项目文件真相源”和“运行态授权缓存”

不应承担：

- 保存用户密码
- 把 Access Token 明文写进项目文件
- 把授权租约和项目文档混成同一个 JSON 根对象

### `rf-thermo`

职责：

- 通过稳定接口读取本地已授权派生物性包
- 对上层隐藏包格式细节

不应承担：

- 直接发 OIDC 请求
- 直接管理刷新 token
- 在热路径中自己决定联网鉴权

### `rf-solver`

职责：

- 只与本地 thermo/provider 接口交互
- 不感知 OIDC、HTTP、Refresh Token、租约刷新

不应承担：

- 直接调用控制面
- 在求解迭代内触发远端拉包

### `.NET 10` CAPE-OPEN Bridge

职责保持不变：

- 只负责 CAPE-OPEN/COM 适配

当前明确不做：

- 在 `.NET 10` 适配层内自行维护一套独立授权体系
- 让 Rust 通过 `.NET` 间接接入 OIDC

## 与 Radish 平台的对接建议

当前建议 RadishFlow 不复用 `Radish` Web 客户端的前端 token 存储代码，但可以复用以下服务能力：

- `Radish.Auth` 作为统一 OIDC 身份源
- 已有 client registration 模式
- `openid profile offline_access` scope 体系
- 统一用户、角色、租户身份语义
- Access Token / Refresh Token / 撤销 / 续期等后端能力

桌面端应新增一个独立客户端注册，例如：

- `client_id = radishflow-studio`

建议权限：

- Authorization Code
- Refresh Token
- PKCE required
- loopback redirect 列表
- RadishFlow 专属 scope，例如：
  - `radishflow-control`
  - `radishflow-assets`

## MVP 范围建议

当前建议把 RadishFlow 的认证与授权后端能力拆成两个阶段。

### M1：控制面最小闭环

- 桌面端 OIDC 登录
- `userinfo` 获取
- EntitlementSnapshot 返回
- 派生物性包 manifest 返回
- 受控资产下载租约
- 本地安全存储 token

### M2：离线与审计

- 离线租约刷新
- 本地派生包缓存
- 关键包访问审计
- 授权过期后的本地行为收口

### M3：高级保护

- 高价值远端求值能力
- 设备绑定
- 更细粒度包水印或定制裁剪
- 吊销与风控增强

## 当前明确不做

以下内容当前阶段明确不做：

- 自建第二套账号密码体系
- 在桌面端内置长期 `client_secret`
- 把主求解循环设计成在线 RPC
- 承诺“客户端绝对防逆向提取”
- 因安全焦虑把全部求解挪到服务端
- 在 Rust Core 中直接引入 OIDC/HTTP 细节

## 后续落地建议

在正式写后端与桌面接入代码之前，建议先做以下文档和类型收口：

1. 在 `rf-ui` 侧补 `AuthSessionState` / `EntitlementState` 草案
2. 在 `rf-store` 侧补“项目文件”与“授权缓存索引”分离模型
3. 在 `rf-thermo` 侧补 `PropertyPackageProvider` 接口草案
4. 在控制面文档中进一步细化 `EntitlementSnapshot` 和 `PropertyPackageManifest` JSON 契约
5. 明确 `radishflow-studio` 客户端注册信息与 scope 命名
