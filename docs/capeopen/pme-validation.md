# CAPE-OPEN PME 人工验证说明

更新时间：2026-04-26

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
.\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests\bin\Debug\net10.0-windows7.0\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests.exe --native-lib-dir <repo>\target\debug
.\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost\bin\Debug\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost.exe --native-lib-dir <repo>\target\debug
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
pwsh .\scripts\pme-register-latest.ps1
pwsh .\scripts\pme-unregister.ps1
```

上述两个脚本是日常 `DWSIM / COFE` 人工调试入口：`pme-register-latest.ps1` 会先构建最新 `rf-ffi`，再通过底层受控注册入口执行 `current-user` 注册；`pme-unregister.ps1` 通过同一底层入口执行反注册。若需要显式查看或复现底层门控参数，可直接使用：

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
   pwsh .\scripts\pme-register-latest.ps1
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
   pwsh .\scripts\pme-unregister.ps1
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

默认会为 `DWSIM.exe` 与 `COFE.exe` 写入进程级 `LocalDumps` 配置，dump 输出目录为 `<repo>\artifacts\pme-dumps`，dump 类型为 full dump。人工复验后可查看：

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

## PME 调用 trace

`UnitOp.Mvp` 仍保留轻量 COM 调用 trace 钩子，用于未来定位真实 PME 调用顺序、COM marshalling 或 material object 读写问题。

该 trace 默认关闭，不再固定写入仓库 `artifacts/pme-trace`。需要临时启用时，先在启动 PME 的同一用户环境中设置：

```powershell
$env:RADISHFLOW_CAPEOPEN_TRACE_DIR = "<repo>\artifacts\pme-trace"
$env:RADISHFLOW_CAPEOPEN_TRACE_FILE = "radishflow-unitop-trace.log"
```

`RADISHFLOW_CAPEOPEN_TRACE_FILE` 可省略，默认文件名为 `radishflow-unitop-trace.log`。验证结束后应清理这两个环境变量；trace 文件只作为诊断附件，不作为正式产品输出。

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

截至 2026-04-27，`SampleHost` 的 PME-like 薄宿主入口、`Registration` execute 门控、脚本化安装/反安装运行手册，以及真实 `DWSIM / COFE` 人工复验已经把 discovery、activation、placement、端口连接与最小 calculate 主路径推进到阶段性闭环。

当前已确认：

- `IDL -> TLB -> ComHostTypeLibrary -> TypeLib 注册` 链路已具备执行路径，并已补齐 `Interop` / `UnitOp.Mvp` 程序集级 TLB identity 与主要 COM-visible class 的默认 interface 口径。
- 真实 Windows PowerShell 5 探测已确认 `New-Object -ComObject`、默认 `ICapeUtilities.Initialize()`、`Parameters.Count()`、`Parameters.Item(1).Specification` 与 `Terminate()` 均通过，先前 `0x80131165 Type library is not registered` 不再复现。
- `DWSIM / COFE` 均已能发现、实例化并放置当前 PMC，也能连接 `Feed / Product` material streams；water/ethanol 复验样例下均已能完成收敛。
- COFE 关闭 case 时的 material object release warning 已消失；端口连接会在连接期间保留 live PME material object 引用，并在断开/终止时释放本 UnitOp 持有的 RCW。
- 最新 COFE trace 已确认 `Validate()` 能在参数配置后返回 valid，`Calculate()` 能完成 native solve、Product material 写回和 CAPE-OPEN 1.1 `CalcEquilibrium(TP)`；后续 mass balance 红字已通过计算前读取 connected `Feed` material 并临时覆盖 native boundary input 解决。
- DWSIM 已确认会按 `InitNew -> Initialize -> SimulationContext set -> identification set -> Ports -> Parameters` 消费 UnitOp；`ICapeUtilities` 前序 vtable 需要保持 DWSIM setter-only PIA 兼容顺序，同时保留 COFE late-bound `SimulationContext` getter。
- DWSIM `GetParams()` 会直接把 `myparms.Item(i)` 返回对象 cast 成 `ICapeIdentification`、`ICapeParameterSpec`、type-specific spec 与 `ICapeParameter`；因此 parameter placeholder 本身必须继续实现 `ICapeParameterSpec` 与 `ICapeOptionParameterSpec`，不能只依赖 `ICapeParameter.Specification`。
- 当前 COFE trace 中 `Validate()` 返回 "Required parameter `Flowsheet Json` is not configured." 属于 MVP 必填参数未配置时的预期 invalid 结果，不再归类为 placement 或 connection 失败。
- DWSIM 日志中的 `AutomaticTranslation.AutomaticTranslator.SetMainWindow(...)` `NullReferenceException` 来自 DWSIM 主窗口 extender 初始化路径，发生在 RadishFlow UnitOp activation 之前；当前仅作为宿主侧启动噪声记录，不作为 RadishFlow CAPE-OPEN blocker。

本次真实 `DWSIM / COFE` 成功复验已经沉淀为正式记录：

- `examples/pme-validation/2026-04-27-dwsim-current-user.md`
- `examples/pme-validation/2026-04-27-cofe-current-user.md`

先前固定写入 `artifacts/pme-trace` 的临时诊断输出也已收口为显式环境变量开关；后续只有在新的 PME 失败需要调用顺序证据时才临时启用。
