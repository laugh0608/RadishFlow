# Auth And Entitlement Architecture

更新时间：2026-04-03

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

认证、授权与远端资产保护属于 **外部控制面**，不是 RadishFlow 桌面进程内部的新业务层；当前正式推荐该控制面采用 `ASP.NET Core / .NET 10` 实现。

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

截至 2026-03-30，以下口径正式冻结。

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
- `RadishFlow Control Plane` 当前正式推荐采用 `ASP.NET Core / .NET 10`

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

### 2.1 控制面实现建议

当前进一步冻结以下技术口径：

- 控制面 API 优先采用 `ASP.NET Core / .NET 10`
- `Radish.Auth` 继续提供 OIDC / OAuth 2.0 身份源，控制面只消费已认证主体与 claims，不重做第二套登录系统
- 物性派生包下载优先采用对象存储 / CDN / 下载网关，由控制面签发短时票据或签名 URL
- 密钥、证书和签名材料优先托管在平台密钥设施中，不自创加密协议或自管长期明文密钥
- 当前不额外引入 Go 作为新的控制面主语言，避免让桌面端、桥接层和控制面形成 Rust / .NET / Go 三套并行技术面

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
        │    └─ RadishFlow Control Plane (ASP.NET Core / .NET 10)
        │
        ├─ Derived Property Package Download
        │    └─ Object Storage / CDN / Download Gateway
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

### 目标部署口径

当前进一步明确以下部署边界：

- 桌面端继续作为“压缩包展开后即可直接启动”的原生客户端交付，不把服务端进程混入桌面安装产物
- `RadishFlow Control Plane` 与 `Asset Delivery` 属于远端独立部署服务，不要求与当前仓库共存于同一个 Monorepo
- 当前仓库继续只维护桌面端所需的 DTO、缓存、下载与协议映射，不把服务端业务逻辑倒灌进 `rf-ui`、`rf-store` 或 `rf-thermo`

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

- 实现形态建议为 `ASP.NET Core / .NET 10` REST API

- `GET /api/radishflow/entitlements/current`
  - 返回当前用户的 `EntitlementSnapshot`
- `GET /api/radishflow/property-packages/manifest`
  - 返回当前授权可见的 `PropertyPackageManifest[]`
- `POST /api/radishflow/property-packages/{packageId}/lease`
  - 返回某个派生资产包的短时下载租约或访问票据
- `POST /api/radishflow/offline-leases/refresh`
  - 刷新桌面端离线租约，并返回新的授权快照
- `POST /api/radishflow/audit/usage`
  - 上传关键受控资产使用审计

当前进一步冻结以下接口语义：

- `GET /api/radishflow/entitlements/current`
  - 桌面端默认授权请求包含 `openid profile offline_access`
  - 控制面 Bearer token 至少具备 `radishflow.control.read`
  - `200 OK` 响应体就是单个 `EntitlementSnapshot`
  - 响应只表达“当前用户此刻可做什么”，不夹带物性包二进制或本地缓存路径
- `GET /api/radishflow/property-packages/manifest`
  - Bearer token 必须具备 `radishflow.assets.read`
  - `200 OK` 返回带 `generatedAt` 的清单容器，其中 `packages` 为 `PropertyPackageManifest[]`
  - manifest 只提供展示、校验和下载前置元信息，不直接提供长期下载 URL
- `POST /api/radishflow/property-packages/{packageId}/lease`
  - Bearer token 必须具备 `radishflow.assets.lease`
  - 请求体至少携带 `version`，用于避免客户端租到错误版本
  - 返回值只允许包含短时票据、到期时间、摘要校验和下载入口，不直接回传物性包内容
- `POST /api/radishflow/offline-leases/refresh`
  - 刷新后返回新的 `EntitlementSnapshot` 和新的 `offlineLeaseExpiresAt`
  - 桌面端用它更新本地授权缓存索引，但仍不把 token 明文写入 JSON
- `POST /api/radishflow/audit/usage`
  - 当前只记关键受控资产访问与加载事件
  - 当前不把每一步单元迭代都上报到控制面

### 可选高级接口

仅在确有必要时再引入：

- `POST /api/radishflow/property-eval`
  - 对极少数绝不下发的高价值物性能力做远端求值

## 核心对象草案

### `EntitlementSnapshot`

建议最小字段：

```json
{
  "schemaVersion": 1,
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

- `schemaVersion` 当前冻结为 `1`，用于控制面 JSON 向后兼容
- 描述当前用户“被允许做什么”
- 不直接携带完整物性数据
- 不承担本地缓存文件索引
- `issuedAt` / `expiresAt` 由服务端生成，不由客户端自行推断
- `allowedPackageIds` 只表达授权白名单，不反向替代 manifest 明细

### `PropertyPackageManifest`

建议最小字段：

```json
{
  "schemaVersion": 1,
  "packageId": "binary-hydrocarbon-lite-v1",
  "version": "2026.03.1",
  "classification": "derived",
  "source": "download",
  "hash": "sha256:...",
  "sizeBytes": 123456,
  "componentIds": ["methane", "ethane"],
  "leaseRequired": true,
  "expiresAt": "2026-04-05T10:00:00Z"
}
```

语义边界：

- `schemaVersion` 当前冻结为 `1`
- `classification` 当前建议只允许 `derived` / `remote-only`
- `source` 当前建议只允许 `bundled` / `download` / `remote-eval`
- 描述可见资产，不直接下发资产内容
- `componentIds` 只描述包覆盖范围，不直接替代包内数据
- `leaseRequired` 用于区分“可直接使用的本地 bundled 资产”和“需要额外租约的受控下载包”
- 用于让桌面端知道“哪些包可下、版本是什么、有效期多久”

### `PropertyPackageManifestList`

`GET /api/radishflow/property-packages/manifest` 当前建议返回以下容器，而不是裸数组：

```json
{
  "schemaVersion": 1,
  "generatedAt": "2026-03-29T10:00:05Z",
  "packages": [
    {
      "schemaVersion": 1,
      "packageId": "binary-hydrocarbon-lite-v1",
      "version": "2026.03.1",
      "classification": "derived",
      "source": "download",
      "hash": "sha256:...",
      "sizeBytes": 123456,
      "componentIds": ["methane", "ethane"],
      "leaseRequired": true,
      "expiresAt": "2026-04-05T10:00:00Z"
    }
  ]
}
```

这样做的原因：

- 后续可以在容器层增加分页、生成时间和服务端游标
- 不必为 manifest 清单额外引入第二个“不透明响应头协议”
- 客户端可以用 `generatedAt` 和本地 `lastSyncedAt` 做最小对账

### `PropertyPackageLeaseRequest`

`POST /api/radishflow/property-packages/{packageId}/lease` 当前建议最小请求体：

```json
{
  "version": "2026.03.1",
  "currentHash": "sha256:...",
  "installationId": "studio-installation-001"
}
```

冻结边界：

- `version` 当前为必填，用于避免客户端租到错误版本
- `currentHash` 当前为可选，用于服务端决定是否允许复用本地缓存
- `installationId` 当前先保留为可选字段，为后续设备绑定或实例级审计留口

### `PropertyPackageLeaseGrant`

`POST /api/radishflow/property-packages/{packageId}/lease` 当前建议最小响应体：

```json
{
  "packageId": "binary-hydrocarbon-lite-v1",
  "version": "2026.03.1",
  "leaseId": "lease-001",
  "downloadUrl": "https://assets.radish.local/leases/lease-001/download",
  "hash": "sha256:...",
  "sizeBytes": 123456,
  "expiresAt": "2026-03-29T10:30:00Z"
}
```

冻结边界：

- 返回值只允许包含短时下载租约，不直接回传包体内容
- `leaseId` 用于审计和问题追溯，不等于长期凭据
- `expiresAt` 明确租约有效期，避免客户端把短时 URL 当长期缓存入口
- 该字段当前只表达下载 URL / 下载租约的有效期，不直接写入 `StoredPropertyPackageRecord.expiresAt`

### `PropertyPackageDownload`

桌面端通过 `downloadUrl` 拉取包体后，当前第一版建议响应体直接采用 JSON 下载 DTO：

```json
{
  "kind": "radishflow.property-package-download",
  "schemaVersion": 1,
  "packageId": "binary-hydrocarbon-lite-v1",
  "version": "2026.03.1",
  "components": [
    {
      "id": "methane",
      "name": "Methane",
      "antoine": {
        "a": 8.987,
        "b": 659.7,
        "c": -16.7
      },
      "liquidHeatCapacityJPerMolK": 35.0,
      "vaporHeatCapacityJPerMolK": 36.5
    }
  ],
  "method": {
    "liquidPhaseModel": "ideal-solution",
    "vaporPhaseModel": "ideal-gas"
  }
}
```

冻结边界：

- 下载 DTO 当前固定为 `kind = radishflow.property-package-download`、`schemaVersion = 1`
- 下载 DTO 是控制面 / 资产分发协议对象，不直接等同于本地 `StoredPropertyPackagePayload`
- 当前协议映射层正式收口到 `apps/radishflow-studio`，由应用层把下载 JSON 映射为 `StoredPropertyPackagePayload` 并继续走本地缓存落盘
- `PropertyPackageLeaseGrant.hash` / `PropertyPackageManifest.hash` 与 `sizeBytes` 当前正式绑定“协议映射后的 `StoredPropertyPackagePayload` 规范化 JSON 字节”，不绑定原始 HTTP 响应外壳
- 桌面端当前先在 `apps/radishflow-studio` 收口下载获取抽象；后续真实 HTTP 客户端只负责实现 fetcher，不再改动本地缓存 DTO 或 `rf-thermo` provider 口径
- 当前也已补 `PropertyPackageDownloadHttpRequest` / `Response` 与 transport 端口，把原始 HTTP 请求、响应头/状态码与下载协议映射层显式分开
- 当前仓库已补首个基于 `reqwest + rustls` 的 blocking HTTP transport，作为 `PropertyPackageDownloadHttpTransport` 的默认真实实现
- 下载抓取错误当前正式区分 `timeout`、连接失败、限流/服务不可用、鉴权失败、找不到资源、无效响应和其他瞬时/永久错误，由 `apps/radishflow-studio` 负责收口
- 默认只对明确可重试的抓取错误做有限次重试：`timeout`、连接失败、限流、服务不可用和其他瞬时错误；授权失败、资源不存在、无效响应、摘要不匹配和本地落盘失败都不重试
- 当前 HTTP 状态码映射正式收口为：`401 -> unauthorized`、`403 -> forbidden`、`404 -> not-found`、`408/504 -> timeout`、`429 -> rate-limited`、`5xx -> service-unavailable`、其余非 2xx 状态与错误 content-type 归为 `invalid-response`
- 当前下载 transport 默认直接对 `downloadUrl` 发起 `GET` 请求，并发送 `Accept: application/json`；不额外叠加长期 bearer token 到下载通道
- 下载落盘当前正式要求失败回滚：若 `payload.rfpkg`、`manifest.json` 或 `index.json` 任一步写入失败，必须恢复或清理已写半成品，避免损坏现有缓存
- `rf-store` 继续只理解本地持久化 DTO，不直接理解控制面下载协议
- `rf-thermo` 继续只理解本地缓存结果，不直接解析下载响应 JSON

### `OfflineLeaseRefreshRequest`

`POST /api/radishflow/offline-leases/refresh` 当前建议最小请求体：

```json
{
  "packageIds": ["binary-hydrocarbon-lite-v1"],
  "currentOfflineLeaseExpiresAt": "2026-04-05T10:00:00Z",
  "installationId": "studio-installation-001"
}
```

冻结边界：

- `packageIds` 用于让控制面知道当前客户端仍依赖哪些包
- `currentOfflineLeaseExpiresAt` 用于服务端判断是否应续期或收紧授权
- `installationId` 当前继续保留为后续设备绑定扩展口

### `OfflineLeaseRefreshResponse`

`POST /api/radishflow/offline-leases/refresh` 当前建议最小响应体：

```json
{
  "refreshedAt": "2026-03-29T10:10:00Z",
  "snapshot": {
    "schemaVersion": 1,
    "subjectId": "user-123",
    "tenantId": "tenant-001",
    "issuedAt": "2026-03-29T10:10:00Z",
    "expiresAt": "2026-03-29T11:10:00Z",
    "offlineLeaseExpiresAt": "2026-04-06T10:10:00Z",
    "features": ["desktop-login", "local-thermo-packages"],
    "allowedPackageIds": ["binary-hydrocarbon-lite-v1"]
  },
  "manifestList": {
    "schemaVersion": 1,
    "generatedAt": "2026-03-29T10:10:00Z",
    "packages": []
  }
}
```

冻结边界：

- 响应必须同时返回新的 `EntitlementSnapshot`
- `manifestList` 当前允许为空数组，但不省略容器对象
- 桌面端用该响应同时刷新授权态和本地 manifest 展示态

### `AuditUsageRequest`

`POST /api/radishflow/audit/usage` 当前建议最小请求体：

```json
{
  "events": [
    {
      "packageId": "binary-hydrocarbon-lite-v1",
      "version": "2026.03.1",
      "eventKind": "package-loaded",
      "occurredAt": "2026-03-29T10:12:00Z"
    }
  ]
}
```

冻结边界：

- 当前只要求“批量事件上报”容器，不为每类事件发明独立资源路径
- `eventKind` 当前先建议收口为 `package-loaded`、`lease-requested`、`remote-evaluation-requested`
- 当前不要求把文档内容、求解详情或明文参数打进审计事件

### `PropertyAssetSource`

建议先冻结为三类：

- `LocalBundled`
  - 随安装包提供的非敏感或开发资产
- `RemoteDerivedPackage`
  - 授权后下载的派生包
- `RemoteEvaluationService`
  - 不下发数据，只允许远端求值

## RadishFlow 内部模块边界

### `apps/radishflow-studio`

职责：

- 作为应用组合根装配 `rf-ui`、`rf-store`、`rf-thermo` 与后续控制面客户端
- 承接 `AuthSessionState` / `EntitlementState` 与 `StoredAuthCacheIndex` 之间的桥接和同步
- 负责把离线刷新、下载完成和缓存索引更新收口成单一路径
- 负责 entitlement panel driver、启动预检、会话内调度策略与 session event 宿主

不应承担：

- 重新发明独立授权模型
- 把控制面 DTO 直接塞回 `FlowsheetDocument`
- 让桥接逻辑反向扩散成 `rf-ui -> rf-store` 或 `rf-store -> rf-ui` 直接依赖

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

- `AuthSessionState` 与 `EntitlementState` 当前已经作为 `AppState` 外层状态骨架落入 `rf-ui`
- 登录态和授权态继续保持在 `AppState` 外层应用状态，不混进 `FlowsheetDocument`
- `rf-ui` 当前只承载运行时授权状态和控制面 DTO，不直接持久化 `StoredAuthCacheIndex`
- `apps/radishflow-studio` 当前已经补上“下载完成 -> 写入 `<cache-root>/packages/.../manifest.json` / `payload.rfpkg` -> 写回 `<cache-root>/auth/.../index.json`”的单一路径
- `apps/radishflow-studio` 当前也已补上 `PropertyPackageDownload` JSON 到 `StoredPropertyPackagePayload` 的首版协议映射，不让该协议直接扩散到 `rf-store` 或 `rf-thermo`
- `apps/radishflow-studio` 当前也已补上下载获取抽象、基于规范化 payload 的摘要校验和失败回滚，不让真实 HTTP 细节反向污染存储与热路径边界
- `apps/radishflow-studio` 当前也已补上下载抓取失败分类和有限次重试策略，为后续真实 HTTP adapter 预留固定语义边界
- `apps/radishflow-studio` 当前也已补上控制面 `entitlement` / `manifest` / `lease` / `offline refresh` 的 HTTP client、JSON 协议映射和 `reqwest + rustls` 真实 transport
- `apps/radishflow-studio` 当前也已补上 entitlement 同步、离线租约刷新、按 manifest 申请 lease 并下载入缓存的应用层编排，不让控制面调用路径继续散落
- `apps/radishflow-studio` 当前也已补出 entitlement panel driver，把 `rf-ui` 的 entitlement widget event 正式桥接回 `StudioAppFacade` 的 control plane command
- `apps/radishflow-studio` 当前也已补出 entitlement startup preflight 与 session scheduler，最小规则收口为“缺失快照先 sync、离线租约临近过期先 refresh、entitlement snapshot 临近过期补 sync”
- `apps/radishflow-studio` 当前也已补出 `entitlement_session_driver`，把 `SessionStarted`、`LoginCompleted`、`TimerElapsed` 与 `EntitlementCommandCompleted` 这类会话事件收口为统一应用层入口，并把失败退避、下一次建议检查时机和 session state 回写固定在 Studio 侧
- `apps/radishflow-studio` 当前也已补出 `entitlement_session_host`，把 session event、entitlement panel 主动作和指定动作进一步收口为统一宿主触发入口，并显式补出 `NetworkRestored` / `WindowForegrounded` 这类 GUI 生命周期事件到现有 session tick 语义的映射；同时 host 当前已可把 `next_check_at` 收口为可直接 arm 的 timer 请求对象，并进一步产出 `Schedule / Reschedule / Keep / Clear` 这类宿主侧定时器决策；当 runtime 自身没有 entitlement notice 时，宿主可把这类提示覆盖到 panel 视图层，但不回写 `EntitlementState`；当前又进一步补出 `EntitlementSessionHostSnapshot`，把 schedule、timer arm、timer command、host notice 与 panel driver state 聚合成单一宿主消费对象，并补出 `EntitlementSessionHostContext` 负责保留当前已挂 timer 与上一份 snapshot，让 bootstrap 或后续 GUI 宿主不必再各自重复实现这套上下文推进逻辑
- `apps/radishflow-studio` 当前又已补出 `EntitlementSessionHostPresentation / TextView`，把 host snapshot 的 schedule/timer/timer command/host notice 文本消费正式收口回宿主模块，`main.rs` 这类最小入口不再继续手写 entitlement host 输出拼装

### `rf-store`

职责：

- 持久化普通项目文件
- 持久化本地资产清单索引、授权缓存元信息和下载包元信息
- 区分“项目文件真相源”和“运行态授权缓存”

不应承担：

- 保存用户密码
- 把 Access Token 明文写进项目文件
- 把授权租约和项目文档混成同一个 JSON 根对象

当前第一版骨架建议分成两类根索引对象 + 两类本地包实体 DTO：

- `StoredProjectFile`
  - 只表示项目文件真相源
- `StoredAuthCacheIndex`
  - 只表示授权缓存、派生包缓存索引与安全凭据引用
- `StoredPropertyPackageManifest`
  - 只表示本地 `manifest.json` 的持久化元信息真相源
- `StoredPropertyPackagePayload`
  - 只表示本地 `payload.rfpkg` 的持久化 thermo payload DTO

当前进一步冻结以下 JSON DTO 与文件布局约定：

- 项目文件继续采用用户选择路径下的单文件 `*.rfproj.json`
- `StoredProjectFile.kind` 固定为 `radishflow.project-file`
- `StoredProjectFile.document.metadata.documentId` 作为文档稳定身份，不依赖文件路径
- `StoredAuthCacheIndex.kind` 固定为 `radishflow.auth-cache-index`
- `StoredPropertyPackageManifest.kind` 固定为 `radishflow.property-package-manifest`
- `StoredPropertyPackagePayload.kind` 固定为 `radishflow.property-package-payload`
- 授权缓存索引和派生包缓存继续放在应用私有缓存根目录，不与项目文件同目录混放
- 授权缓存索引只保存相对缓存路径和安全凭据引用，不保存绝对缓存路径与明文 token

建议的项目文件 JSON 轮廓：

```json
{
  "kind": "radishflow.project-file",
  "schemaVersion": 1,
  "document": {
    "revision": 12,
    "flowsheet": {
      "...": "..."
    },
    "metadata": {
      "documentId": "doc-1",
      "title": "Demo Project",
      "schemaVersion": 1,
      "createdAt": "2026-03-29T09:00:00Z",
      "updatedAt": "2026-03-29T10:10:00Z"
    }
  }
}
```

建议的授权缓存索引 JSON 轮廓：

```json
{
  "kind": "radishflow.auth-cache-index",
  "schemaVersion": 1,
  "authorityUrl": "https://id.radish.local",
  "subjectId": "user-123",
  "credential": {
    "service": "radishflow-studio",
    "account": "user-123-primary"
  },
  "entitlement": {
    "subjectId": "user-123",
    "tenantId": "tenant-001",
    "syncedAt": "2026-03-29T10:05:00Z",
    "issuedAt": "2026-03-29T10:00:00Z",
    "expiresAt": "2026-03-29T11:00:00Z",
    "offlineLeaseExpiresAt": "2026-04-05T10:00:00Z",
    "featureKeys": ["desktop-login", "local-thermo-packages"],
    "allowedPackageIds": ["binary-hydrocarbon-lite-v1"]
  },
  "propertyPackages": [
    {
      "packageId": "binary-hydrocarbon-lite-v1",
      "version": "2026.03.1",
      "source": "remote-derived-package",
      "manifestRelativePath": "packages/binary-hydrocarbon-lite-v1/2026.03.1/manifest.json",
      "payloadRelativePath": "packages/binary-hydrocarbon-lite-v1/2026.03.1/payload.rfpkg",
      "hash": "sha256:...",
      "sizeBytes": 123456,
      "downloadedAt": "2026-03-29T10:05:10Z",
      "expiresAt": "2026-04-05T10:00:00Z"
    }
  ],
  "lastSyncedAt": "2026-03-29T10:05:00Z"
}
```

建议的文件布局：

```text
<chosen-project-path>/<name>.rfproj.json
<app-private-cache-root>/auth/<sanitized-authority>/<subject-id>/index.json
<app-private-cache-root>/packages/<package-id>/<version>/manifest.json
<app-private-cache-root>/packages/<package-id>/<version>/payload.rfpkg
```

补充边界：

- `payload.rfpkg` 只在 `LocalBundled` / `RemoteDerivedPackage` 下出现；`RemoteEvaluationService` 不要求本地 payload 文件
- `StoredPropertyPackageRecord` 当前正式只记录相对路径，方便缓存根目录迁移与跨设备导入时做路径重定位
- `StoredPropertyPackageRecord.expiresAt` 当前正式跟随授权快照中的离线租约/授权过期时间，不直接复用 `PropertyPackageLeaseGrant.expiresAt` 的短时下载 URL 过期时间
- 本地 `manifest.json` 当前固定为带 `kind` / `schemaVersion` 的 camelCase JSON DTO，并显式校验 `source`、`classification` 与 `leaseRequired` 一致性
- 本地 `payload.rfpkg` 当前第一版内部仍采用带 `kind` / `schemaVersion` 的 JSON DTO，承载 `ThermoSystem` 所需的组分、Antoine 参数占位和相模型信息；后续若改为压缩包或二进制格式，必须通过显式迁移切换
- 授权缓存索引是“运行态缓存真相源”，项目文件仍然只描述用户编辑的流程图语义

### `rf-thermo`

职责：

- 通过稳定接口读取本地已授权派生物性包
- 对上层隐藏包格式细节

不应承担：

- 直接发 OIDC 请求
- 直接管理刷新 token
- 在热路径中自己决定联网鉴权

当前第一版接口方向已经冻结为：

- `ThermoProvider`
  - 面向求解热路径的本地热力学接口
- `PropertyPackageProvider`
  - 面向受控派生物性包的加载与清单接口

当前进一步冻结以下本地缓存 provider 行为：

- `PropertyPackageProvider` 的本地缓存实现必须显式接收应用私有缓存根目录和 `StoredAuthCacheIndex`，而不是自己推断登录态或联网刷新授权
- provider 构造阶段就应校验 `StoredAuthCacheIndex`、本地 `manifest.json` 和本地 `payload.rfpkg` 三者是否一致，不把索引漂移问题拖进求解热路径
- provider 当前只暴露“未过期、存在本地 payload、manifest/payload 与索引一致”的本地包；`RemoteEvaluationService` 记录不进入本地 provider 列表
- 当前仓库已在 `examples/sample-components/property-packages/` 提供首个真实样例包和对应下载 JSON，用于校验上述本地缓存链路、协议映射和 provider 装载行为

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

当前建议把客户端注册进一步冻结为：

- `application_type = native`
- `grant_types = authorization_code refresh_token`
- `response_types = code`
- `token_endpoint_auth_method = none`
- `require_pkce = true`
- `redirect_uris`
  - `http://127.0.0.1:{ephemeral-port}/oidc/callback`
  - `http://localhost:{ephemeral-port}/oidc/callback`
- `post_logout_redirect_uris`
  - `http://127.0.0.1:{ephemeral-port}/oidc/logout-callback`
  - `http://localhost:{ephemeral-port}/oidc/logout-callback`

scope 当前建议按“产品.资源.动作”命名，而不是继续使用过宽泛的单块 scope：

- 标准 OIDC scope：
  - `openid`
  - `profile`
  - `offline_access`
- RadishFlow 资源 scope：
  - `radishflow.control.read`
  - `radishflow.assets.read`
  - `radishflow.assets.lease`
  - `radishflow.audit.write`

当前建议：

- 登录与授权同步默认请求 `openid profile offline_access radishflow.control.read radishflow.assets.read radishflow.assets.lease`
- `radishflow.audit.write` 允许后续单独补到审计上报通道，不强制绑在首版登录请求里
- 不再建议使用 `radishflow-control`、`radishflow-assets` 这种边界过宽的 scope 名称

## MVP 范围建议

当前建议把 RadishFlow 的认证与授权后端能力拆成两个阶段。

### M1：控制面最小闭环

- 桌面端 OIDC 登录
- `userinfo` 获取
- EntitlementSnapshot 返回
- 派生物性包 manifest 返回
- 受控资产下载租约
- 本地安全存储 token
- `StoredProjectFile` / `StoredAuthCacheIndex` JSON DTO 与文件布局冻结

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

在继续深化后端与桌面接入代码之前，当前更值得优先收口以下事项：

1. 在已补出的 `entitlement_session_driver + entitlement_session_host + EntitlementSessionEvent` 基础上，把网络恢复、窗口恢复前台和真实 GUI 定时器宿主接到统一会话入口
2. 在已接通的控制面 HTTP client、下载通道和 session scheduler 之上，细化联网失败后的用户提示、刷新节奏和重试触发口径
3. 在已接通的 `PropertyPackageProvider` 本地缓存实现之上，补更多实际包样例和加载/替换场景测试
4. 继续保持其余控制面 JSON 契约到运行时 DTO 的协议映射层统一收口到应用层
5. 决定离线租约后续是否需要设备绑定键模型
