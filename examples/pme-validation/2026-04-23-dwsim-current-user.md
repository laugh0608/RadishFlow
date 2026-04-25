Date: 2026-04-23
Validator: Codex（终端准备；DWSIM GUI 人工验证待继续）
RadishFlow commit: `61f597e251efaecfcdc091d401b7c37a45315483`
OS: `Microsoft Windows [Version 10.0.26200.8246]`
PME: `DWSIM`
PME version: `9.0.2`
PME bitness: `64-bit`
Registry scope: `current-user`
Comhost path: `D:\Code\RadishFlow\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp\bin\Debug\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll`
Dry-run command: `pwsh .\scripts\register-com.ps1 -Scope current-user -SkipBuild -ComHostPath .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp\bin\Debug\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll -Json`
Registration command: `pwsh .\scripts\register-com.ps1 -Scope current-user -Execute -ConfirmToken register-current-user-2F0E4C8F -SkipBuild -BackupDir .\artifacts\registration-validation\register-current-user -ComHostPath .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp\bin\Debug\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll -Json`
Unregistration command: `pwsh .\scripts\register-com.ps1 -Action unregister -Scope current-user -Execute -ConfirmToken unregister-current-user-2F0E4C8F -SkipBuild -BackupDir .\artifacts\registration-validation\unregister-current-user -ComHostPath .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp\bin\Debug\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll -Json`
Preflight result: `Pass`
Warnings accepted: `None`
Register post-check:
- `HKCU\Software\Classes\CLSID\{2F0E4C8F-7C89-4DA7-A5D3-5F8C987D6718}` exists
- `HKCU\Software\Classes\RadishFlow.CapeOpen.UnitOp.Mvp` exists
- `HKCU\Software\Classes\RadishFlow.CapeOpen.UnitOp.Mvp.1` exists
Discovery: `Pending`。DWSIM 已启动，待人工在 CAPE-OPEN Unit Operation 选择入口确认是否能看到 `RadishFlow Unit Operation` / `RadishFlow.CapeOpen.UnitOp.Mvp`。
Activation: `Pending`。待人工在 DWSIM 中实例化组件。
Identity: `Pending`。待人工确认 identification 名称与描述是否匹配 `UnitOperationComIdentity`。
Parameters: `Pending`。待人工确认最小参数集合是否可见且名称稳定。
Ports: `Pending`。待人工确认 `Feed` / `Product` 端口及方向语义。
Connection: `Pending`。待人工尝试连接 DWSIM material object。
Validate: `Pending`。待人工触发 PME 侧 `Validate`。
Calculate: `Pending`。待人工触发 PME 侧 `Calculate`。
Report: `Pending`。待人工确认结果摘要或失败诊断是否能被 DWSIM 稳定读取。
Unregister: `Pass`。已执行 `current-user` 反注册，`ExecutionSummary.Succeeded = true`。
Unregister post-check:
- `HKCU\Software\Classes\CLSID\{2F0E4C8F-7C89-4DA7-A5D3-5F8C987D6718}` absent
- `HKCU\Software\Classes\RadishFlow.CapeOpen.UnitOp.Mvp` absent
- `HKCU\Software\Classes\RadishFlow.CapeOpen.UnitOp.Mvp.1` absent
Logs:
- Registration backup: `D:\Code\RadishFlow\artifacts\registration-validation\register-current-user\registry-backup.json`
- Registration execution log: `D:\Code\RadishFlow\artifacts\registration-validation\register-current-user\execution-log.json`
- Unregistration backup: `D:\Code\RadishFlow\artifacts\registration-validation\unregister-current-user\registry-backup.json`
- Unregistration execution log: `D:\Code\RadishFlow\artifacts\registration-validation\unregister-current-user\execution-log.json`
- DWSIM executable: `D:\Program Files\DWSIM\DWSIM.exe`
- DWSIM process observed: `DWSIM (PID 26300)`
Decision: `Blocked`
Follow-up:
- 已确认注册发现问题已基本收敛；真实阻塞点改为 `TypeLib / TLB` 缺失。
- Windows PowerShell COM 探测结果：
- `Type.GetTypeFromProgID("RadishFlow.CapeOpen.UnitOp.Mvp")`：`Pass`
- `New-Object -ComObject "RadishFlow.CapeOpen.UnitOp.Mvp"`：`Pass`
- 首个晚绑定 `IDispatch` 调用：`Fail`，错误 `0x80131165 Type library is not registered`
- 等新的 `TLB` 生成/嵌入/注册链路落地后，再重新执行 `current-user register -> DWSIM/COFE discovery -> activation -> validate -> calculate`。
- 完成 GUI 验证后，补写本记录中的 `Discovery` 至 `Report` 字段。
- 若 DWSIM / COFE 在补齐 `TLB` 后仍无法实例化组件，优先按 `Activation / IDispatch / Parameters / Ports / Reporting` 分类补充现象、截图和宿主侧报错文本。

2026-04-25 update:
- `UnitOp.Mvp` 已补齐 `Interop` / `UnitOp.Mvp` 程序集级 `Guid / TypeLibVersion`，并为主对象、parameter/port collection、parameter/port placeholder 补出显式 `ComDefaultInterface`。
- `Windows PowerShell 5` 真实环境复验：`New-Object -ComObject`、`Initialize()`、`Parameters.Count()`、`Parameters.Item(1).Specification`、`Terminate()` 均为 `Pass`，`0x80131165` 不再复现。
- `ICapeUnit` `QueryInterface` 返回 `S_OK`；`Ports` 仍需在真实 PME 或强类型宿主路径中复验，不能用 PowerShell 默认 `ICapeUtilities` binder 代替。
- 已补入最小 `ICapeUnitReport` activation 兼容面与更新后的 `TLB`；下一轮注册后应额外复验 `QueryInterface(ICapeUnitReport)`、`reports`、`selectedReport` 与 `ProduceReport(ref string)`。
- 本轮终端侧补充验证：`cargo check`、`UnitOp.Mvp` build、`ContractTests` build、32 项 contract tests、`SampleHost` build/run 均通过；`Registration` dry-run 已显示 `ICapeUnitReport`，preflight 全部 `Pass`。
- 本轮终端侧注册注意事项：提权上下文可执行 register/unregister，但其 `HKCU` 与普通 DWSIM/COFE 用户上下文不同；非提权沙盒上下文执行 `RegisterTypeLibForUser` 命中 `TYPE_E_REGISTRYACCESS (0x8002801C)`，工具已 rollback，普通 HKCU 四棵目标树均 `exists=False`。
- 下一步重新执行 `current-user register -> DWSIM/COFE discovery -> activation -> validate -> calculate`，并补写本记录中的 `Discovery` 至 `Report` 字段。
