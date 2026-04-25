# CAPE-OPEN PME 人工验证说明

更新时间：2026-04-23

## 文档目的

本文档用于冻结自有 MVP Unit Operation PMC 进入真实 PME 前的人工验证路径。

当前文档只描述验证计划、执行前门控和记录口径，不代表仓库已经允许默认写 Windows Registry、自动化启动 PME，或加载第三方 CAPE-OPEN 模型。

## 当前边界

当前允许：

- 使用 `RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost` 验证正式 host-facing 消费路径
- 使用 `RadishFlow.CapeOpen.Registration` 输出 dry-run / preflight，并在获准后通过 execute 门控执行真实 register / unregister
- 检查 `RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll` 是否存在、位数是否匹配、目标 registry key 是否已有冲突
- 记录将来 register / unregister 会涉及的 registry key、CAPE-OPEN categories 与备份范围
- 编写目标 PME 人工验证记录

当前不允许：

- 默认写 Windows Registry
- 默认注册或反注册 COM class
- 自动化启动、控制或脚本化外部 PME
- 加载第三方 CAPE-OPEN Unit Operation 或 Thermo/Property Package
- 将 PME / COM 对象语义倒灌到 Rust core 或 `rf-ffi`

## 目标 PME 选择口径

第一轮目标 PME 只需要选定一个可人工安装、可手动打开、可显示 CAPE-OPEN Unit Operation PMC 列表并能实例化 Unit Operation 的宿主。

目标 PME 记录应至少包含：

- PME 名称
- PME 版本
- 进程位数
- 操作系统版本
- RadishFlow commit
- `RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll` 路径
- registry scope：`current-user` 或 `local-machine`
- 验证人员与验证日期

若目标 PME 只能读取机器级 COM 注册，则本仓库仍不应因此默认走 HKLM；只能在当前 execute 门控满足显式确认、权限检查、备份与回滚后再单独执行。

## 执行前验证基线

进入真实 PME 前必须先通过以下本地基线：

```powershell
cargo check
dotnet build .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp\RadishFlow.CapeOpen.UnitOp.Mvp.csproj -v minimal
dotnet build .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.Registration\RadishFlow.CapeOpen.Registration.csproj -v minimal
dotnet build .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests.csproj -v minimal
dotnet build .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost\RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost.csproj -v minimal
```

真实 PME 前还应先运行最小 contract / sample host：

```powershell
.\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests\bin\Debug\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests.exe --native-lib-dir D:\Code\RadishFlow\target\debug
.\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost\bin\Debug\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost.exe --native-lib-dir D:\Code\RadishFlow\target\debug
```

如果 `.NET 10` first-time use、restore、native path 或沙盒隔离导致结果与代码现状明显不符，应按仓库约定申请真实环境复验，而不是直接把失败归因到实现。

## 注册前 dry-run

真实写入前必须先执行 dry-run：

```powershell
pwsh .\scripts\register-com.ps1 -Scope current-user -ComHostPath .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp\bin\Debug\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll
```

dry-run 输出必须人工确认：

- `CLSID`、`ProgID` 与 `Versioned ProgID` 与文档一致
- CAPE-OPEN categories 至少包含 CAPE-OPEN Object 与 Unit Operation
- implemented interfaces 至少包含 `ICapeIdentification`、`ICapeUtilities`、`ICapeUnit`、`ICapeUnitReport`、`IPersistStreamInit`、`IPersistStorage` 与 `IOleObject`
- `comhost path` 为 `Pass`
- `comhost architecture` 与目标 PME 进程位数一致
- `comhost runtime layout` 为 `Pass`
- `process architecture` 与预期 registry view 一致
- `registry conflict` 没有未解释的冲突
- `backup plan` 覆盖 `CLSID / ProgID / Versioned ProgID` 三棵树
- `Writes registry` 仍为 `no`

若 dry-run 出现 `Fail`，不得进入真实注册。若出现 `Warning`，必须在验证记录里解释是否接受。

补充探测约束：

- 需要做 PowerShell COM 晚绑定复验时，优先使用 `C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe`
- 不要默认用 `pwsh` 作为 native COM 探测宿主；若 `pwsh` 已预加载其它版本的 `.NET` runtime，可能会出现与注册树无关的 `0x800080A5` 假阴性

## 执行型注册工具门控

当前真实写入路径仍必须默认保持 dry-run。执行型注册只能在额外显式参数下启用，并至少满足以下门控：

- 必须存在 `--execute`
- 必须存在与本次 action / scope / CLSID 绑定的确认 token，例如 `--confirm register-current-user-2F0E4C8F`
- 必须先运行同一参数下的 preflight，并且不存在 `Fail`
- `local-machine` scope 必须检测 elevation，不满足时直接拒绝写入
- 必须显式提供备份输出路径，或由工具生成带时间戳的备份记录路径
- 必须把实际写入范围限制在 dry-run registry plan 列出的 key/value 内
- 必须在写入前记录已有 key 的存在状态和待覆盖值
- 必须支持 `unregister`，并要求同等级确认门控
- 必须输出可附加到人工验证记录的执行日志
- 若执行过程中任一步骤失败，必须尝试用本次刚捕获的备份恢复三棵 registry tree，并把 rollback 结果写入 execution log

建议优先通过仓库脚本入口执行，而不是直接手写底层 exe 命令：

```powershell
pwsh .\scripts\register-com.ps1 -Execute -ConfirmToken register-current-user-2F0E4C8F -BackupDir .\artifacts\registration-validation\register-current-user
pwsh .\scripts\register-com.ps1 -Action unregister -Execute -ConfirmToken unregister-current-user-2F0E4C8F -BackupDir .\artifacts\registration-validation\unregister-current-user
```

## 安装/反安装运行手册

推荐把真实注册、人工 PME 验证与反注册收口为以下顺序；除非目标 PME 明确只能读取 HKLM，否则默认优先走 `current-user`。

1. 先执行验证基线，并确认 `SampleHost` 与 contract tests 结果稳定。
2. 运行一次 dry-run，确认 `PreflightChecks` 中不存在 `Fail`，并把 `comhost path / architecture / registry conflict / backup plan` 结果抄入验证记录。
3. 执行 register：

   ```powershell
   pwsh .\scripts\register-com.ps1 -Execute -ConfirmToken register-current-user-2F0E4C8F -BackupDir .\artifacts\registration-validation\register-current-user
   ```

4. register 成功后顺序复查三棵 registry tree 已落地，不要与注册命令并行执行：

   ```powershell
   Get-Item Registry::HKEY_CURRENT_USER\Software\Classes\CLSID\{2F0E4C8F-7C89-4DA7-A5D3-5F8C987D6718}
   Get-Item Registry::HKEY_CURRENT_USER\Software\Classes\RadishFlow.CapeOpen.UnitOp.Mvp
   Get-Item Registry::HKEY_CURRENT_USER\Software\Classes\RadishFlow.CapeOpen.UnitOp.Mvp.1
   ```

5. 按下文人工 PME 验证路径完成 discovery / activation / validate / calculate 记录，并把执行日志、备份路径和 PME 观察结果一起落到 `examples/pme-validation/` 对应记录文件。
6. 验证完成后执行 unregister：

   ```powershell
   pwsh .\scripts\register-com.ps1 -Action unregister -Execute -ConfirmToken unregister-current-user-2F0E4C8F -BackupDir .\artifacts\registration-validation\unregister-current-user
   ```

7. unregister 成功后再次顺序复查三棵 registry tree 已删除；若任一键残留，应先记录为 `Unregister` 失败，再检查 execution log 与 rollback 状态。

`local-machine` 只在目标 PME 明确要求 HKLM、且验证人员具备 elevation、备份路径和回滚窗口时才允许进入；当前仓库不建议把它作为默认安装路径。

执行型注册工具不应顺手承担以下职责：

- 启动 PME
- 操作 PME UI
- 加载第三方 CAPE-OPEN 模型
- 生成安装包
- 扩张 `UnitOp.Mvp` 的 host round 或 smoke driver DSL

## 人工 PME 验证路径

注册完成后，目标 PME 验证仍采用人工路径：

1. 手动启动目标 PME。
2. 打开 PME 的 CAPE-OPEN Unit Operation 或自定义单元选择入口。
3. 确认能看到 `RadishFlow MVP Unit Operation` 或对应 `ProgID`。
4. 实例化该 Unit Operation。
5. 读取 identification，确认名称和描述符合 `UnitOperationComIdentity`。
6. 读取 parameters，确认最小参数集合存在且名称稳定。
7. 读取 ports，确认 `Feed` 与 `Product` 端口存在且方向语义符合预期。
8. 尝试连接 PME 提供的 material object。
9. 尝试写入 flowsheet JSON、package id、manifest/payload 路径等 MVP 参数。
10. 触发 PME 侧 `Validate`。
11. 触发 PME 侧 `Calculate`。
12. 读取报告或 PME 日志，确认成功结果或可诊断失败被稳定暴露。
13. 关闭 PME 后执行 unregister，并确认组件不再出现在 PME discovery 入口中。

若目标 PME 的调用顺序与上述顺序不同，应记录实际顺序、触发的 `UnitOp.Mvp` 状态、异常类型和 report/follow-up 输出；不要为了某个 PME 行为立即把临时兼容逻辑塞进 Rust core。

## 通过标准

第一轮人工 PME 验证通过标准：

- PME 能发现组件
- PME 能实例化组件
- PME 能读取 identification
- PME 能读取最小 parameter collection
- PME 能读取最小 port collection
- PME 能连接必需端口或给出可解释失败
- PME 能触发 `Validate`
- PME 能触发 `Calculate`
- 成功时能读取最小结果摘要
- 失败时能读取稳定 diagnostic / report 文本
- unregister 后同一 scope 下的组件发现结果被移除

## 失败分类

验证失败应先按以下分类记录：

- `Discovery`：PME 没有发现组件
- `Activation`：PME 发现但无法实例化 COM class
- `Identity`：实例化后 identification 读取失败或字段漂移
- `Collection`：parameters / ports collection 形状不符合 PME 期待
- `Connection`：PME material object 连接失败
- `Validation`：`Validate` 失败且无法诊断
- `Calculation`：`Calculate` 失败且无法诊断
- `Reporting`：结果已产生但 PME 无法读取报告或日志
- `Unregister`：反注册后 discovery 仍残留

失败修复优先级：

- 先判断是否是注册信息、位数、权限或 comhost 路径问题
- 再判断是否是 `.NET` COM-visible / marshalling / interface shape 问题
- 再判断是否是 `UnitOp.Mvp` 对象运行时、collection、parameter 或 port 语义问题
- 最后才考虑 host round / follow-up 层是否需要补正式 helper
- 不把单个 PME 的非标准行为直接下沉到 Rust core

## PME 崩溃 dump 采集

若 `DWSIM` / `COFE` 能发现组件，但在选择组件并添加到 flowsheet 画布时直接闪退，应先收集 native 崩溃现场，再继续判断是否需要新增 COM 接口面或调整承载策略。

仓库提供单行脚本配置 Windows Error Reporting `LocalDumps`。Microsoft 文档要求该功能写入 `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\LocalDumps`，因此启用/清理有效 dump 配置需要管理员 PowerShell：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\configure-pme-dumps.ps1 -Action enable -Scope local-machine
```

默认会为 `DWSIM.exe` 与 `COFE.exe` 写入进程级 `LocalDumps` 配置，dump 输出目录为 `D:\Code\RadishFlow\artifacts\pme-dumps`，dump 类型为 full dump。人工复验后可查看：

```powershell
Get-ChildItem .\artifacts\pme-dumps
```

若崩溃后目录为空，先确认 LocalDumps 配置仍然启用，并检查是否存在 WER 禁用策略：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\configure-pme-dumps.ps1 -Action status
```

`current-user/HKCU` 下的 `LocalDumps` 配置不会被 WER 用于真实 dump 采集；如果曾经用旧脚本写入过 HKCU，可只用于清理：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\configure-pme-dumps.ps1 -Action disable -Scope current-user
```

验证结束后应清理该 WER 配置：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\configure-pme-dumps.ps1 -Action disable -Scope local-machine
```

这一路径只用于捕捉 PME 进程崩溃栈，不替代 `current-user register / unregister` 的安装清理要求。

## 验证记录模板

建议每次人工验证记录以下内容：

- 正式模板已放到 `examples/pme-validation/pme-validation-record-template.md`
- 建议每次记录按 `examples/pme-validation/YYYY-MM-DD-<pme>-<scope>.md` 命名，例如 `examples/pme-validation/2026-04-22-dwsim-current-user.md`

```text
Date:
Validator:
RadishFlow commit:
OS:
PME:
PME version:
PME bitness:
Registry scope:
Comhost path:
Dry-run command:
Registration command:
Unregistration command:
Preflight result:
Warnings accepted:
Register post-check:
Discovery:
Activation:
Identity:
Parameters:
Ports:
Connection:
Validate:
Calculate:
Report:
Unregister:
Unregister post-check:
Logs:
Decision:
Follow-up:
```

## 当前判断

截至 2026-04-25，`SampleHost` 的 PME-like 薄宿主入口、`Registration` execute 门控以及脚本化安装/反安装运行手册，已足以结束“注册工具设计缺口”这条子任务。

当前已新增的明确判断是：

- `DWSIM` 与 `COFE` 的 discovery 基本已打通，不再是“完全看不到组件”
- `IDL -> TLB -> ComHostTypeLibrary -> TypeLib 注册` 链路当前已具备执行路径，并已补齐 `Interop` / `UnitOp.Mvp` 程序集级 TLB identity 与主要 COM-visible class 的默认 interface 口径
- 真实 Windows PowerShell 5 探测当前已确认 `New-Object -ComObject`、默认 `ICapeUtilities.Initialize()`、`Parameters.Count()`、`Parameters.Item(1).Specification` 与 `Terminate()` 均通过，先前 `0x80131165 Type library is not registered` 不再复现
- `ICapeUnit` 当前已通过 `QueryInterface` 探测返回 `S_OK`；但 PowerShell 默认 late binding 只代表默认 `ICapeUtilities` 面，`Ports` / `Validate` / `Calculate` 仍应在真实 PME 或强类型宿主路径中复验
- 当前已补入最小 `ICapeUnitReport` 接口与 TLB 描述，`ProduceReport(ref string)` 复用既有 canonical calculation report 文本，用于加固 PME 添加组件时可能读取报告接口的 activation 调用面
- 当前已补入最小 `IPersistStreamInit` 接口、主类实现与 TLB 描述，`GetClassID / IsDirty / InitNew / Load / Save / GetSizeMax` 当前均为无状态 no-op HRESULT 路径，用于加固 PME 把组件加入 flowsheet 画布时可能进行的 OLE 持久化探测
- 在用户侧 trace 复验确认 `DWSIM` 已调用并退出 `IPersistStreamInit.InitNew()` 后，当前又补入最小 `IPersistStorage` 接口、主类实现与 TLB 描述，继续覆盖 OLE canvas storage persistence 探测面
- 在用户侧 trace 继续确认 `DWSIM` 仍停在 `IPersistStreamInit.InitNew()` 返回后、`IPersistStorage` 未被调用后，当前又补入最小 `IOleObject` 接口、主类实现与 TLB 描述，覆盖 OLE container embedding 探测面；该实现不承诺真实可视 OLE 嵌入或 in-place activation
- 用户侧继续复验后，`DWSIM` 仍只到 `IPersistStreamInit.InitNew()` enter/exit，未进入 `IOleObject`；`COFE` 仍只到 constructor exit。这说明盲补常见 OLE/CAPE-OPEN 接口的收益已明显下降，下一轮应优先收集 WER LocalDumps / native 崩溃栈，确认崩溃是否来自 `.NET 10 in-proc comhost/CoreCLR` 承载冲突、宿主 native QI/返回值处理，或仍有具体 COM interface shape 问题。
- 用户侧已采集 `COFE.exe.28544.dmp` 与 `DWSIM.exe.39524.dmp`：`COFE` 故障地址为 `COFE.exe+0xbc5ffe`，访问地址 `0x10`；`DWSIM` 故障地址为 `coreclr.dll+0x3b745`，访问地址 `0x8`。结合 trace，当前首要风险已从“继续缺某个普通 optional interface”转为 `.NET 10 in-proc comhost/CoreCLR` 与 PME 宿主进程承载兼容性，或宿主 native 侧在 activation 返回后处理接口指针失败。
- `dotnet-dump` 复读 DWSIM dump 后确认托管栈停在 `System.StubHelpers.InterfaceMarshaler.ConvertToManaged -> IL_STUB_COMtoCLR -> ComMethodFrame`，因此当前又先做一处更具体的小修正：`IPersistStreamInit.Load / Save` 的 managed 签名使用 raw `IntPtr` stream，避免无状态 no-op persistence 在进入方法体前触发 `IStream` interface marshaling。
- 用户侧复验 raw stream 后 DWSIM 仍停在同一 `InterfaceMarshaler.ConvertToManaged` 栈；下一轮复验应重点验证 C# `InterfaceIsDual` 对齐与 `ICapeUtilities.SimulationContext` raw `IntPtr` setter 是否让 trace 推进到 `SimulationContext set-enter/exit` 或后续 CAPE-OPEN 成员。
- 本次终端验证中，非提权沙盒上下文执行 `RegisterTypeLibForUser` 会触发 `TYPE_E_REGISTRYACCESS (0x8002801C)` 并由注册工具自动 rollback；后续真实 PME 复验仍应以普通桌面用户上下文执行仓库脚本，避免把提权 `HKCU` 与 PME 用户 `HKCU` 混用

仍必须补齐的边界缺口不是新的 host round fallback，也不是继续盲补普通 optional interface。下一步先复验 dual interface 与 raw `SimulationContext` 修正；如果仍失败，再围绕 `DWSIM + COFE` 的 in-proc comhost 承载风险评估 out-of-proc COM shim / local server 或更保守的 PME in-proc shim + 外部 worker 策略。
