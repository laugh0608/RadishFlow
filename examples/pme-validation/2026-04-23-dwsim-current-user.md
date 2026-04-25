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
- 用户复验：补入 `ICapeUnitReport` 后，`DWSIM / COFE` 仍在“选择模块后加入 flowsheet 画布”时崩溃。WER 显示 DWSIM 加载 `RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll` 后继续加载 `.NET 10 hostfxr/hostpolicy/coreclr`，并在 `coreclr.dll` 上以 `c0000005 / 0x80131506` 终止；COFE 也在加载同一 comhost/CoreCLR 链路后崩溃。
- 下一轮诊断：已加入临时文件 trace，路径为 `D:\Code\RadishFlow\artifacts\pme-trace\radishflow-unitop-trace.log`。若崩溃后无该文件，说明尚未进入 `RadishFlowCapeOpenUnitOperation` managed 成员；若有文件，最后一行即 PME 崩溃前最后进入/退出的 COM 成员。
- 下一步重新执行 `current-user register -> DWSIM/COFE discovery -> activation -> validate -> calculate`，并补写本记录中的 `Discovery` 至 `Report` 字段。

2026-04-25 update 2:
- 用户侧 trace 复验：`COFE` 与 `DWSIM` 均只记录 `static-init -> constructor-enter -> constructor-exit`，未进入 `ComponentName / Initialize / Parameters / Ports / reports / ProduceReport` 等成员。
- 当前判断：崩溃点位于对象构造完成后、正式 automation 调用前，优先怀疑 PME 添加到 flowsheet 画布时的 `QueryInterface` / OLE canvas persistence 探测面，或 `.NET 10 in-proc comhost` 与宿主进程 runtime 承载冲突。
- 本轮已补入最小 `IPersistStreamInit`，并重新生成 `TLB`；下一轮 trace 应重点观察 `GetClassID / IsDirty / InitNew / Load / Save / GetSizeMax` 是否出现在崩溃前。
- 本轮终端侧补充验证：`cargo check`、`UnitOp.Mvp` build（真实环境）、`ContractTests` build、33 项 contract tests 均通过。

2026-04-25 update 3:
- 用户侧 trace 复验：`DWSIM` 已记录 `IPersistStreamInit.InitNew enter/exit`，随后在下一个 managed 成员调用前崩溃；`COFE` 仍只记录到 constructor exit。
- 当前判断：DWSIM 已确认走到 OLE canvas initialization，崩溃点从“constructor 后”进一步收窄到 `IPersistStreamInit.InitNew()` 返回后；下一优先排查面是相邻 `IPersistStorage` 或更大的 OLE embedding 接口。
- 本轮已补入最小 `IPersistStorage`，并重新生成 `TLB`；下一轮 trace 应重点观察 `IPersistStorage.InitNew / Load / Save / SaveCompleted / HandsOffStorage` 是否出现在崩溃前。
- 本轮终端侧补充验证：`cargo check`、`UnitOp.Mvp` build（真实环境）、`ContractTests` build、33 项 contract tests 均通过。

2026-04-25 update 4:
- 用户侧 trace 复验：`DWSIM` 仍记录 `IPersistStreamInit.InitNew enter/exit` 后崩溃，未进入 `IPersistStorage`；`COFE` 仍只记录到 constructor exit。
- 当前判断：DWSIM 的下一步更可能是 OLE container embedding 探测，例如 `QueryInterface(IOleObject)`，而不是继续调用 `IPersistStorage`。
- 本轮已补入最小 `IOleObject`，并重新生成 `TLB`；下一轮 trace 应重点观察 `SetClientSite / SetHostNames / DoVerb / GetUserClassID / GetUserType / SetExtent / GetExtent / GetMiscStatus / Close` 是否出现在崩溃前。
- 本轮终端侧补充验证：`cargo check`、`UnitOp.Mvp` build（真实环境）、`ContractTests` build、34 项 contract tests 均通过。

2026-04-25 update 5:
- 用户侧 trace 复验：`DWSIM` 仍只记录 `IPersistStreamInit.InitNew enter/exit` 后崩溃，未进入 `IOleObject` 任一成员；`COFE` 仍只记录到 constructor exit。
- 当前判断：`ICapeUnitReport`、`IPersistStreamInit`、`IPersistStorage`、`IOleObject` 均未解决 PME 添加到 flowsheet 画布时的硬崩。DWSIM 崩溃点已收窄到 `IPersistStreamInit.InitNew()` 返回后的 native/COM 过渡；COFE 崩溃点仍在 constructor 返回后的 native/COM 过渡。
- 下一轮不再优先盲补普通 COM/OLE 接口；应先通过 WER `LocalDumps` 或等价 native crash dump 拿到崩溃栈，确认是否为 `.NET 10 in-proc comhost/CoreCLR` 与 PME 宿主进程承载冲突，或宿主侧 `QueryInterface` / HRESULT / interface pointer 处理路径崩溃。
- 已新增仓库脚本 `scripts/configure-pme-dumps.ps1`，用于单行启用/清理当前用户的 `DWSIM.exe` / `COFE.exe` WER dump 配置，默认 dump 输出目录为 `D:\Code\RadishFlow\artifacts\pme-dumps`。

2026-04-25 update 6:
- 用户侧 trace 复验仍与 update 5 一致：`DWSIM` 停在 `IPersistStreamInit.InitNew()` exit，`COFE` 停在 constructor exit。
- 用户侧 `Get-ChildItem .\artifacts\pme-dumps` 未返回 dump 文件，说明 WER LocalDumps 路径没有产物；下一步先用 `scripts/configure-pme-dumps.ps1 -Action status` 确认 LocalDumps 仍处于 enable 状态，并检查 WER 禁用策略。
