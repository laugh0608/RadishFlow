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

2026-04-25 update 7:
- 已确认先前 `scripts/configure-pme-dumps.ps1` 写入的是 `HKCU`，但 Microsoft WER `LocalDumps` 的有效配置位于 `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps`，因此先前 `current-user` enable 状态不会产出 dump。
- 脚本已改为默认使用 `local-machine/HKLM`，`enable/disable` 需要管理员 PowerShell；`current-user/HKCU` 仅保留用于清理旧的无效配置。

2026-04-25 update 8:
- 用户侧已成功采集 full dump：`COFE.exe.28544.dmp`（约 196 MB）与 `DWSIM.exe.39524.dmp`（约 782 MB）。
- `COFE` dump：异常为 `0xc0000005`，故障地址 `COFE.exe+0xbc5ffe`，访问地址 `0x10`；trace 最后一行仍是 `RadishFlowCapeOpenUnitOperation constructor-exit`。当前判断为 COFE native 侧在 COM activation 返回后空指针解引用，尚无证据进入 RadishFlow 托管成员或新增 OLE/CAPE-OPEN 成员。
- `DWSIM` dump：异常为 `0xc0000005`，故障地址 `coreclr.dll+0x3b745`，访问地址 `0x8`；同进程同时加载 `.NET Framework 4.x clr.dll/mscoree/mscoreei` 与 `.NET 10 coreclr/hostfxr/hostpolicy/comhost`，trace 最后一行仍是 `IPersistStreamInit.InitNew exit`。当前判断为 `.NET 10 in-proc comhost/CoreCLR` 在 DWSIM 进程内的承载风险已成为首要嫌疑。
- 用户安装 `dotnet-dump` 后复读 DWSIM dump：崩溃线程存在 `System.ExecutionEngineException (0x80131506)`，托管栈为 `System.StubHelpers.InterfaceMarshaler.ConvertToManaged(IntPtr ByRef, IntPtr, IntPtr, Int32) -> ILStubClass.IL_STUB_COMtoCLR(IntPtr) -> ComMethodFrame`。这说明 DWSIM 在 COM-to-CLR stub 把接口参数转成 managed interface 时崩溃，尚未进入对应 C# 方法体。
- 基于该结果，当前把 `IPersistStreamInit.Load / Save` managed 签名从 `IStream?` 改成 raw `IntPtr`，因为 MVP no-op persistence 当前不消费 stream；目标是避开 `IStream` interface marshaler，让方法体能稳定记录 trace 并返回 `S_OK`。
- DWSIM error log 中的 `AutomaticTranslation.AutomaticTranslator.SetMainWindow` `NullReferenceException` 出现在 Extender Initialization 阶段，早于本组件添加到画布的 crash trace；暂记录为宿主启动噪声候选，不作为 RadishFlow COM activation 主因。
- 下一步先带回 DWSIM / COFE 复验该 raw pointer persistence 修正；若仍停在同一阶段，再评估 out-of-proc COM shim / local server 或更保守的 PME in-proc shim + 外部 worker 承载策略，把 `.NET 10` runtime 从 PME 进程中隔离出去。

2026-04-25 update 9:
- 用户侧复验 raw pointer persistence 修正后仍崩溃：`DWSIM` trace 仍停在 `IPersistStreamInit.InitNew exit`，`COFE` trace 仍停在 constructor exit。
- 新 dump：`DWSIM.exe.34144.dmp` 与 `COFE.exe.35284.dmp`。`dotnet-dump` 复读 DWSIM 新 dump 仍显示 `System.ExecutionEngineException (0x80131506)`，托管栈仍为 `System.StubHelpers.InterfaceMarshaler.ConvertToManaged -> IL_STUB_COMtoCLR -> ComMethodFrame`。
- 当前判断：`IPersistStreamInit.Load / Save` 的 `IStream` 参数不是本轮触发点；更高概率触发面是 PME 在添加组件后设置 `ICapeUtilities.SimulationContext`，或早绑定调用 CAPE-OPEN dual interface 时遇到 C# `InterfaceIsIDispatch` 与 IDL/TLB `dual` 不一致。
- 本轮修正：C# 侧主要 CAPE-OPEN automation 接口已改为 `InterfaceIsDual`，并将 `ICapeUtilities.SimulationContext` managed 签名改为 raw `IntPtr`，不再让 CLR 在进入 setter 前创建 managed interface/RCW。
- 本轮终端侧补充验证：`cargo check`、`UnitOp.Mvp` build（真实环境）、`ContractTests` build（真实环境）、34 项 contract tests 均通过。

2026-04-25 update 10:
- 用户侧复验 dual/raw `SimulationContext` 修正后，`DWSIM` 不再闪退，但组件未添加到画布；trace 已推进到 `SimulationContext set-enter`，随后 setter 记录 `NullReferenceException` 并退出。
- 同轮 `COFE` 仍闪退；trace 已推进到 `SimulationContext get-enter/get-result context=null/get-exit`，随后 COFE native 侧崩溃；新 dump `COFE.exe.43200.dmp` 无当前托管异常。
- 当前判断：DWSIM 的下一阻塞是 `SimulationContext` setter 不应持有或释放宿主指针；COFE 的下一阻塞是 getter 不能返回 null `IDispatch*`。
- 本轮修正：`SimulationContext` setter 改为 no-op 记录，只保存“宿主提供过 context”的布尔状态；getter 在尚无可消费 PME context 时返回非空 `ICapeIdentification` placeholder 指针，避免 COFE native 侧空指针解引用。
- 本轮终端侧补充验证：`cargo check`、`UnitOp.Mvp` build（真实环境）、`ContractTests` build（真实环境）、34 项 contract tests 均通过。

2026-04-25 update 11:
- 用户侧复验 no-op `SimulationContext` setter 与非空 placeholder getter 后，`DWSIM` 已继续进入 `ComponentName set`、`ComponentDescription set`、`Ports get` 与 `Parameters get`，且没有产生 DWSIM dump；这说明 DWSIM 添加路径已越过原先的 simulation context 阻塞点。
- 同轮 `COFE` 仍闪退；trace 最后一段为 `SimulationContext get-enter -> get-result fallback=provided; hostContext=missing -> get-exit`，新 dump `COFE.exe.32048.dmp` 仍无当前托管异常。
- 当前判断：COFE 不只是要求 `SimulationContext` getter 返回非空对象，它还会继续按 COSE context 相关接口消费该对象。参考接口显示 `ICapeSimulationContext` 是无方法 marker，常见可选消费面包括 `ICapeCOSEUtilities` 和 `ICapeDiagnostic`。
- 本轮修正：新增最小 `ICapeSimulationContext`、`ICapeCOSEUtilities` 与 `ICapeDiagnostic` 接口定义；simulation context placeholder 同时实现这三个接口，`NamedValueList` 返回空字符串数组，`NamedValue(...)` 返回空字符串，diagnostic 方法只写 trace。

## Update 16 - 2026-04-25

- 用户侧复验新增 COSE context 接口后，`DWSIM` 仍推进到 `SimulationContext set/get`、`ComponentName/Description set`、`Ports get` 与 `Parameters get`，未产生新的 DWSIM dump。
- 同轮 `COFE` 仍在 `SimulationContext get-result fallback=provided; hostContext=missing -> get-exit` 后 native 崩溃，新 dump 为 `COFE.exe.42720.dmp`，仍未进入 `NamedValueList / NamedValue / LogMessage / PopUpMessage`。
- 当前判断：COFE 更可能在 getter 返回后做早期 `QueryInterface` 或 typeinfo 探测；若缺少 `ICapeMaterialTemplateSystem` 或 placeholder 没有公开 coclass typeinfo，COFE native 侧可能仍沿空指针路径崩溃。
- 本轮修正：补入最小 `ICapeMaterialTemplateSystem`，并将 simulation context placeholder 提升为公开 COM-visible coclass；IDL/TLB 已同步，下一轮重点看 COFE 是否越过 `SimulationContext get-exit`。

## Update 17 - 2026-04-25

- 用户侧复验公开 placeholder coclass 与 `ICapeMaterialTemplateSystem` 后，`COFE` 仍在 `SimulationContext get-result fallback=provided; hostContext=missing -> get-exit` 后 native 崩溃，新 dump 为 `COFE.exe.11352.dmp`。
- 同轮 `DWSIM` 仍推进到 `SimulationContext set/get`、`ComponentName/Description set`、`Ports get` 与 `Parameters get`，未产生新的 DWSIM dump。
- 当前判断：COFE 未进入 `MaterialTemplates / NamedValueList / NamedValue / LogMessage / PopUpMessage`，因此继续补普通 context optional interface 的收益已经很低；更可能是 `SimulationContext` getter 的 native COM 签名与 managed raw pointer 形状存在不一致。
- 本轮修正：`ICapeUtilities.SimulationContext` 继续在 managed 侧使用 raw `IntPtr`，但 getter/setter 显式标注 `IDispatch` marshalling，用于把 native 侧签名重新压回 IDL 的 `IDispatch** / IDispatch*`。
- 本轮已同步 `typelib/RadishFlow.CapeOpen.UnitOp.Mvp.idl` 并用 Windows SDK MIDL 重新生成 TLB；MIDL 仅报告 IDL 中文注释 code page warning，生成成功。
- 本轮终端侧补充验证：`cargo check`、`UnitOp.Mvp` build（真实环境）、`ContractTests` build（真实环境）、34 项 contract tests 均通过。
