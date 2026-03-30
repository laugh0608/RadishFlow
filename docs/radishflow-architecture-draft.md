# RadishFlow 架构草案

更新时间：2026-03-30

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
- 模型编辑
- 参数面板
- 运行控制
- 结果展示
- 日志与诊断
- 项目打开保存

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
│     └─ src/
│        ├─ RadishFlow.CapeOpen.Interop/
│        ├─ RadishFlow.CapeOpen.Adapter/
│        ├─ RadishFlow.CapeOpen.UnitOp.Mvp/
│        ├─ RadishFlow.CapeOpen.Registration/
│        └─ RadishFlow.CapeOpen.SmokeTests/
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

不建议在这里直接堆放热力学与求解细节。

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

## `crates/rf-flowsheet`

流程图结构层。

职责：

- 节点与连线
- 端口连接规则
- 图完整性校验
- 拓扑排序前置检查

不负责具体数值求解。

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

建议：

- 这里放“UI 行为逻辑”
- 不直接承载底层算法实现

## `crates/rf-canvas`

流程图画布专用库。

职责：

- 节点绘制
- 端口绘制
- 连线绘制
- 拖拽、缩放、平移、框选

说明：

流程图画布复杂度会持续增长，单独拆库有利于后续维护。

## `crates/rf-ffi`

Rust 与 .NET 的桥接层。

职责：

- 对外导出稳定 C ABI
- 提供句柄式调用
- 管理字符串与内存边界

建议导出接口形态：

- `engine_create`
- `engine_destroy`
- `stream_create`
- `stream_set_tpzf`
- `flash_tp`
- `unit_create_*`
- `unit_solve`
- `flowsheet_solve`
- `snapshot_to_json`

## `adapters/dotnet-capeopen`

这是 .NET 10 的 CAPE-OPEN/COM 适配层根目录。

它是外部互操作桥，不是模拟器主体。

### `RadishFlow.CapeOpen.Interop`

职责：

- CAPE-OPEN 接口定义
- GUID、属性、辅助类型
- ECape 异常基础设施
- 注册辅助逻辑

它可以吸收当前 `CapeOpenCore` 仓库中的接口定义与注册经验。

### `RadishFlow.CapeOpen.Adapter`

职责：

- PInvoke 调用 `rf-ffi`
- 将 Rust 句柄封装成 .NET 侧对象
- 完成数据与错误转换

这是 .NET 与 Rust 的唯一正式运行时边界。

### `RadishFlow.CapeOpen.UnitOp.Mvp`

职责：

- MVP 阶段的 CAPE-OPEN Unit Operation PMC
- 通过适配层调用 Rust 核心
- 作为外部 PME 验证对象

建议先以 Unit Operation 为导出重点，不在第一阶段同时做完整 Thermo PMC。

### `RadishFlow.CapeOpen.Registration`

职责：

- COM host 注册与反注册
- 管理员提权
- 冒烟验证辅助

### `RadishFlow.CapeOpen.SmokeTests`

职责：

- 最小冒烟测试
- PMC 创建、参数读写、计算流程验证

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
9. 再建设 `radishflow-studio` 的画布与属性编辑 UI

## 当前仓库与目标仓库的关系

建议把当前 CapeOpenCore 仓库视为以下资产来源：

参考链接：

- [CapeOpenCore](https://github.com/laugh0608/CapeOpenCore)

- CAPE-OPEN 接口定义参考
- COM 注册行为参考
- GUID 与属性语义参考
- 示例 Unit Operation 行为参考

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
