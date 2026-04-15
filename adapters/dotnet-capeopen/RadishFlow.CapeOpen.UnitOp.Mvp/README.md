# RadishFlow.CapeOpen.UnitOp.Mvp

当前目录已从纯占位推进为第一版最小 `net10.0` PMC 骨架项目，职责只限于：

- 提供一个最小 `ICapeIdentification` + `ICapeUtilities` + `ICapeUnit` 实现类
- 为后续真正的 CAPE-OPEN Unit Operation PMC 留出项目边界和最小状态机
- 提供最小内部 flowsheet/package 配置入口，并通过 `RadishFlow.CapeOpen.Adapter` 接入 `rf-ffi` 求解闭环

当前已包含的最小公共面：

- `RadishFlowCapeOpenUnitOperation`
- `UnitOperationPortPlaceholder` / `UnitOperationParameterPlaceholder`
- `UnitOperationPlaceholderCollection<T>`
- `Initialize / Validate / Calculate / Terminate / Edit` 的第一版状态骨架
- 内部 `LoadFlowsheetJson(...)`、`LoadPropertyPackageFiles(...)`、`SelectPropertyPackage(...)` 配置入口
- `SetPortConnected(...)` 这一类最小端口状态入口
- `ConfigureNativeLibraryDirectory(...)` 与 `LastFlowsheetSnapshotJson`
- `Calculate()` 对未满足前置条件的最小 ECape 语义抛错，以及经由 `rf-ffi` 的最小真实求解接线
- `ICapeCollection` / `ICapeParameter` / `ICapeUnitPort` 的第一版最小对象运行时
- placeholder 对象对 unit owner 生命周期的最小访问守卫，以及 `Terminate()` 时的端口连接释放

当前明确不包含：

- COM 注册 / 反注册
- 稳定 CLSID / ProgID 策略
- 报告接口的正式实现
- PME 生命周期集成
- 完整 CAPE-OPEN PMC 运行时

说明：

- 当前 `Ports` / `Parameters` 已返回带 `Item(object)` 和 `Count()` 的最小 `ICapeCollection` 风格对象，并支持按 `ComponentName` 或 1-based 索引取项
- 当前参数对象已提供最小 `ICapeParameter` + `ICapeParameterSpec` 语义，端口对象已提供最小 `ICapeUnitPort` 语义，但仍只覆盖 MVP 所需的字符串参数和占位连接对象
- 当前 placeholder 对象已开始收口到宿主生命周期边界：`Terminate()` 后不再允许继续通过集合/参数/端口对象做 CAPE-OPEN 风格访问，并会释放端口上的已连接对象引用
- `Calculate()` 当前已能在最小前置条件满足后调用 `rf-ffi` 完成求解，并把 flowsheet snapshot JSON 缓存在实例内；但这仍不等于完整 PMC 生命周期、正式端口/参数运行时或 PME 互调已经完成
- Rust/.NET 边界仍保持为句柄 + UTF-8 + JSON + 状态码，没有在这里提前引入 COM 注册或更宽的跨边界对象传递
