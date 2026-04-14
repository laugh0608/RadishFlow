# RadishFlow.CapeOpen.Adapter

当前目录已建立第一版最小 `.NET 10` 薄适配项目，职责只限于：

- 用 `LibraryImport` 调用 `rf-ffi`
- 管理 native engine 句柄生命周期
- 统一 UTF-8 输入/输出字符串分配释放
- 把 `RfFfiStatus + last_error_message/json` 收口为 .NET 异常

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
- ECape 异常映射
- 注册与反注册
- PME 冒烟测试
