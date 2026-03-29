# RadishFlow CAPE-OPEN 资产提取清单

更新时间：2026-03-29

## 文档目的

本文档用于指导如何从当前 `CapeOpenCore` 仓库中提取 CAPE-OPEN 相关资产，服务未来 `RadishFlow` 新仓库中的 `.NET 10` 适配层，尤其是：

- `RadishFlow.CapeOpen.Interop`
- `RadishFlow.CapeOpen.Adapter`
- `RadishFlow.CapeOpen.UnitOp.Mvp`
- `RadishFlow.CapeOpen.Registration`

这份清单强调的是**资产提取策略**，不是直接复制代码的建议。

## 使用原则

提取时遵循以下原则：

1. 优先迁移语义，不优先迁移旧实现。
2. 优先迁移接口定义、GUID、异常语义和注册语义。
3. 不把 `.NET Framework 4.8`、WinForms、`RegistrationServices` 等历史包袱直接带入新仓库。
4. 第一阶段只服务自有 Unit Operation PMC，不为“加载第三方 CO 模型”预先过度建设。

## 目标映射

建议把旧仓库的 CAPE-OPEN 资产映射到新仓库如下：

| 新仓库目标项目 | 主要来源 | 迁移方式 |
| --- | --- | --- |
| `RadishFlow.CapeOpen.Interop` | `*IDL.cs`、`COGuids.cs`、异常接口与属性定义 | 以整理、裁剪、现代化为主 |
| `RadishFlow.CapeOpen.Adapter` | 端口/参数/单元抽象经验 | 重新设计实现，不直接复制 |
| `RadishFlow.CapeOpen.UnitOp.Mvp` | `MixerExample.cs`、`CapeUnitBase.cs` 语义 | 仅借鉴行为，不直接照搬 |
| `RadishFlow.CapeOpen.Registration` | `Program.cs`、`CapeObjectBase.cs`、`CapeUnitBase.cs` 注册经验 | 改写为 .NET 10 COM host 路径 |

## 第一阶段必须提取的资产

这些内容建议作为 `RadishFlow` 启动时就要整理的最小 CAPE-OPEN 资产。

### 1. GUID 与常量

优先来源：

- `CapeOpenCore.Class/COGuids.cs`

作用：

- 保留 CAPE-OPEN 接口、Category 和相关 COM 标识语义
- 统一新仓库中的 GUID 来源

建议：

- 迁移语义和常量定义
- 在新仓库中重新命名和整理命名空间
- 不要在多个项目中重复定义同一组 GUID

### 2. 通用 CAPE-OPEN 接口定义

优先来源：

- `CapeOpenCore.Class/CommonIdl.cs`
- `CapeOpenCore.Class/CoseIDL.cs`
- `CapeOpenCore.Class/UnitIDL.cs`
- `CapeOpenCore.Class/ErrorIdl.cs`

作用：

- 构成 `RadishFlow.CapeOpen.Interop` 的最小接口面
- 为 Unit Operation PMC 提供 `ICapeIdentification`、`ICapeUtilities`、`ICapeUnit` 及错误接口语义

建议：

- 先只提取 MVP 需要的接口
- 不要一次性把全部 IDL 映射无差别搬过去
- 优先按“最小可运行 PMC”组织命名空间

### 3. ECape 异常语义

优先来源：

- `CapeOpenCore.Class/ErrorIdl.cs`
- `CapeOpenCore.Class/Exceptions01.cs`
- `CapeOpenCore.Class/Exceptions02.cs`

作用：

- 为 `.NET 10` 适配层提供与 CAPE-OPEN 规范一致的错误语义
- 确保 PME 接收到可识别的错误类别

建议：

- 保留异常分类语义
- 重写实现，清理旧式序列化构造与 `.NET Framework` 遗留
- 第一阶段先覆盖 Unit PMC 需要的异常类型

### 4. 属性与元数据语义

优先来源：

- `CapeOpenCore.Class/CapeOpen.cs`

作用：

- 为新仓库保留 `CapeNameAttribute`、`CapeDescriptionAttribute`、`CapeVersionAttribute` 等元数据语义
- 支撑注册表描述与组件自描述

建议：

- 将这些属性独立放入 `RadishFlow.CapeOpen.Interop` 或子命名空间
- 对文档注释与命名做现代化清理

### 5. CAPE-OPEN 注册语义

优先来源：

- `CapeOpenCore.Class/CapeObjectBase.cs`
- `CapeOpenCore.Class/CapeUnitBase.cs`
- `CapeOpenCore/Program.cs`

作用：

- 理解当前仓库如何写入 `CapeDescription`、`Implemented Categories` 等 CAPE-OPEN 相关注册信息
- 为新仓库的 `RadishFlow.CapeOpen.Registration` 提供行为依据

建议：

- 提取“应该写什么”的规则
- 不要直接复制“怎么写”的旧代码
- 在新仓库中按 `.NET 10 COM host` 方案重写注册器

## 第二阶段再提取的资产

这些内容对新仓库有价值，但不建议在 `RadishFlow` 仓库刚创建时就迁移。

### 1. 参数体系

主要来源：

- `CapeOpenCore.Class/ParameterIDL.cs`
- `ArrayParameter.cs`
- `BooleanParameter.cs`
- `IntegerParameter.cs`
- `OptionParameter.cs`
- `RealParameter.cs`
- `ParameterCollection.cs`

建议：

- 先提取接口语义
- 具体实现按新仓库需求重写
- 不建议把带 WinForms/UI 行为的参数实现直接迁过去

### 2. 端口体系

主要来源：

- `CapeOpenCore.Class/UnitPort.cs`
- `CapeOpenCore.Class/PortCollection.cs`

建议：

- 先抽象概念与行为
- 不直接复制旧对象实现
- 在新仓库中按 Rust 内核数据模型重新映射

### 3. 监控与扩展接口

主要来源：

- `CapeOpenCore.Class/MonitoringInterfaces.cs`
- `CapeOpenCore.Class/CofeIdl.cs`

建议：

- 第一阶段不作为阻塞项
- 等外部 PME 验证完成后，再按真实需求决定是否提取

### 4. Thermo 与 Material Object 接口面

主要来源：

- `CapeOpenCore.Class/ThrmIDL.cs`
- `CapeOpenCore.Class/PetroFractionsIDL.cs`
- `CapeOpenCore.Class/ReactionsIDL.cs`
- `CapeOpenCore.Class/PersistenceInterfacesIDL.cs`

建议：

- 第一阶段不要整包迁移
- 等你准备做 Thermo PMC 时再专项处理

## 只作为行为参考、不直接迁移的资产

这些内容有参考价值，但不建议直接进入新仓库的正式实现。

### 1. Unit Operation 基类实现

主要来源：

- `CapeOpenCore.Class/CapeObjectBase.cs`
- `CapeOpenCore.Class/CapeUnitBase.cs`

原因：

- 里面混合了注册逻辑、UI 行为、COM 行为、生命周期管理
- 还包含 `.NET Framework` 时代的运行时假设

使用方式：

- 参考它的接口行为
- 参考它的参数/端口/验证语义
- 不直接复制代码

### 2. 示例单元实现

主要来源：

- `CapeOpenCore.Test/MixerExample.cs`

原因：

- 这是很好的行为参考基线
- 但未来 `RadishFlow` 的实际单元应由 Rust 内核驱动

使用方式：

- 作为功能对照样例
- 用于定义新仓库中的最小单元输入输出行为

### 3. 可枚举系统与包装器

主要来源：

- `CapeOpenCore.Class/UnitOperationSystem.cs`
- `CapeOpenCore.Class/UnitOperationManager.cs`

原因：

- 当前实现与 COM 枚举、旧式程序集发现、`AppDomain`、`Assembly.LoadFrom` 强耦合

使用方式：

- 只参考“为什么需要这些能力”
- 不把当前实现迁到新仓库

## 明确不迁移的内容

以下内容建议明确留在旧仓库，不进入 `RadishFlow` 主线。

### 1. 旧式项目系统与 net48 配置

包括：

- 旧式 `csproj`
- `App.config`
- `TargetFrameworkVersion=v4.8`
- binding redirects

### 2. WinForms UI 代码

包括：

- `CapeOpenCore.Class/CapeOpenUI/`
- `War.cs`
- 参数类中直接弹窗的逻辑

原因：

- 新产品 UI 主线已经明确为 Rust UI

### 3. RegistrationServices 方案

包括：

- `CapeOpenCore/Program.cs` 中基于 `RegistrationServices` 的注册器

原因：

- 新仓库应走 `.NET 10 COM host` 路径

### 4. 基于 AppDomain 的旧加载逻辑

包括：

- `CreateInstanceAndUnwrap`
- `AssemblyResolve`
- 基于 `LoadFrom` 的旧式扫描

原因：

- 新仓库不应该继续承袭旧 CLR 模型

### 5. 历史备份目录

包括：

- `CapeOpen.BackUp/`

原因：

- 只保留为外部参考来源
- 不进入新仓库主线

## 建议的提取顺序

建议按以下顺序提取：

1. `COGuids.cs`
2. `CapeOpen.cs`
3. `CommonIdl.cs`
4. `CoseIDL.cs`
5. `UnitIDL.cs`
6. `ErrorIdl.cs`
7. `Exceptions01.cs` / `Exceptions02.cs` 的异常分类语义
8. `MixerExample.cs` 的行为语义
9. 参数与端口体系的接口语义

这个顺序的好处是：

- 先建立 `Interop` 所需最小面
- 再建立 `UnitOp.Mvp` 所需行为层
- 最后才考虑更复杂的参数与 Thermo 面

## 建议的新仓库落点

建议在 `RadishFlow` 新仓库中的对应落点如下：

| 旧仓库来源 | 新仓库目标位置 |
| --- | --- |
| `COGuids.cs` | `adapters/dotnet-capeopen/src/RadishFlow.CapeOpen.Interop/Guids/` |
| `CapeOpen.cs` | `adapters/dotnet-capeopen/src/RadishFlow.CapeOpen.Interop/Metadata/` |
| `CommonIdl.cs` | `adapters/dotnet-capeopen/src/RadishFlow.CapeOpen.Interop/Common/` |
| `CoseIDL.cs` | `adapters/dotnet-capeopen/src/RadishFlow.CapeOpen.Interop/Cose/` |
| `UnitIDL.cs` | `adapters/dotnet-capeopen/src/RadishFlow.CapeOpen.Interop/Unit/` |
| `ErrorIdl.cs` | `adapters/dotnet-capeopen/src/RadishFlow.CapeOpen.Interop/Errors/` |
| 异常语义 | `adapters/dotnet-capeopen/src/RadishFlow.CapeOpen.Interop/Exceptions/` |
| 注册语义参考 | `adapters/dotnet-capeopen/src/RadishFlow.CapeOpen.Registration/` |
| 单元行为参考 | `adapters/dotnet-capeopen/src/RadishFlow.CapeOpen.UnitOp.Mvp/` |

## Definition of Ready

当以下条件满足时，可以认为旧仓库已经完成了对 `RadishFlow` 的 CAPE-OPEN 资产准备：

- 已明确哪些文件是第一阶段必须提取的
- 已明确哪些资产只做行为参考
- 已明确哪些内容绝不迁移
- 新仓库中的 `Interop` 目录边界已根据此清单设计完成

## 结论

对 `RadishFlow` 来说，旧仓库最有价值的不是“整包代码”，而是：

- 接口定义
- GUID
- 异常语义
- 注册语义
- 最小单元行为参考

只要把这些资产抽出来，新仓库就能在不继承旧技术债的前提下，建立起自己的 `.NET 10 CAPE-OPEN` 适配层。

