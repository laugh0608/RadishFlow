# RadishFlow.CapeOpen.Interop

当前目录已从纯占位升级为第一版最小 `net10.0` 互操作语义项目，职责收口为：

- 放置后续 PMC 与注册层会复用的 CAPE-OPEN 接口骨架
- 集中维护已确认的 CAPE-OPEN GUID / Category 常量来源
- 提供第一版 ECape 异常语义基类、错误代码与公共上下文

当前已包含的最小公共面：

- `ICapeIdentification`
- `ICapeUtilities`
- `ICapeUnit`
- `CapeValidationStatus`
- `CapeOpenInterfaceIds`
- `CapeOpenCategoryIds`
- `CapeOpenErrorHResults`
- `CapeOpenException` 与若干语义化派生异常

当前明确不包含：

- COM 注册 / 反注册
- ProgID / CLSID 分配策略
- 完整 CAPE-OPEN 端口、参数、集合实现
- 完整 PMC 生命周期与 PME 集成

说明：

- `ICapeIdentification`、`ICapeUtilities`、`ICapeUnit` 的 IID 当前依据 `adapters/reference/CapeOpenMixerExample_CSharp/CapeOpen/COGuids1.cs` 校准
- 已确认的 GUID 与 HRESULT 统一放在本项目内，避免后续 `Adapter`、`UnitOp.Mvp`、`Registration` 重复定义
