# RadishFlow.CapeOpen.Adapter

当前目录已建立第一版最小 `.NET 10` 薄适配项目，职责只限于：

- 用 `LibraryImport` 调用 `rf-ffi`
- 管理 native engine 句柄生命周期
- 统一 UTF-8 输入/输出字符串分配释放
- 基于 `RadishFlow.CapeOpen.Interop` 把 `RfFfiStatus + last_error_message/json` 收口为可复用的 ECape 语义异常类型

当前已覆盖的最小调用面：

- `LoadFlowsheetJson(...)`
- `LoadPropertyPackageFiles(...)`
- `GetPropertyPackageListJson()`
- `SolveFlowsheet(...)`
- `GetFlowsheetSnapshotJson()`
- `GetStreamSnapshotJson(...)`
- `TryGetLastErrorMessage()`
- `TryGetLastErrorJson()`

当前明确不包含：

- COM / CAPE-OPEN 接口实现
- 完整 ECape 接口实现
- 完整 ECape 异常实现
- 注册与反注册
- PME 冒烟测试

当前错误映射口径：

- `InvalidEngineState` 会优先映射到 `ECapeBadInvOrder`，并在可识别时补 `RequestedOperation`
- `InvalidInput` / `MissingEntity` / `NullPointer` / `InvalidUtf8` 先映射到 `ECapeInvalidArgument`
- `Thermo` / `Flash` / `InvalidConnection` 先映射到 `ECapeSolvingError`
- Rust/.NET 边界仍只交换句柄、UTF-8、JSON 与状态码，不在这里提前引入 COM 注册或 PME 生命周期
