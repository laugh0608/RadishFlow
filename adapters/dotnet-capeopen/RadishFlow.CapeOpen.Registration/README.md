# RadishFlow.CapeOpen.Registration

该目录当前提供第一版 `net10.0` 注册前置 / 执行门控 console。

当前职责：

- 输出自有 MVP Unit Operation PMC 的注册前置描述和 dry-run registry plan
- 冻结当前组件的 `CLSID / ProgID / Versioned ProgID`
- 冻结当前 `TypeLib ID / TypeLib Version / TLB file name`
- 列出将来注册时需要关联的 CAPE-OPEN component categories
- 列出当前最小实现承诺的 CAPE-OPEN 接口面
- 按 `register / unregister` 和 `current-user / local-machine` 生成将来会写入、删除或调用 `TypeLib` 注册 API 的执行计划
- 解析并检查 `RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll`
- 解析并检查 `RadishFlow.CapeOpen.UnitOp.Mvp.tlb`
- 只读检查目标 registry key 是否已存在，并输出备份范围
- 在显式 `--execute` + `--confirm` 下执行 register / unregister
- 在执行前导出四棵 registration tree 的 JSON 备份，并写出 execution log
- 若执行中途失败，按刚捕获的备份自动回滚 `CLSID / ProgID / Versioned ProgID / TypeLib` 四棵树

默认行为当前仍然是 dry-run。未显式传入 `--execute` 时，本工具不会写 Windows Registry。

当前仍明确不做：

- 写 Windows Registry
- 注册或反注册 COM class
- 启动或自动化外部 PME
- 加载第三方 CAPE-OPEN 模型
- 安装证书、写系统目录或修改本机环境

运行示例：

```powershell
dotnet run --project .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.Registration\RadishFlow.CapeOpen.Registration.csproj
```

推荐仓库脚本入口：

```powershell
pwsh .\scripts\register-com.ps1
```

JSON 输出：

```powershell
dotnet run --project .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.Registration\RadishFlow.CapeOpen.Registration.csproj -- --json
```

反注册计划：

```powershell
dotnet run --project .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.Registration\RadishFlow.CapeOpen.Registration.csproj -- --action unregister
```

机器级注册计划：

```powershell
dotnet run --project .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.Registration\RadishFlow.CapeOpen.Registration.csproj -- --scope local-machine
```

显式指定 comhost：

```powershell
dotnet run --project .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.Registration\RadishFlow.CapeOpen.Registration.csproj -- --comhost .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp\bin\Debug\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll
```

显式指定 type library：

```powershell
dotnet run --project .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.Registration\RadishFlow.CapeOpen.Registration.csproj -- --typelib .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp\typelib\RadishFlow.CapeOpen.UnitOp.Mvp.tlb
```

执行型 register：

```powershell
dotnet run --project .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.Registration\RadishFlow.CapeOpen.Registration.csproj -- --execute --confirm register-current-user-2F0E4C8F
```

通过仓库脚本执行：

```powershell
pwsh .\scripts\register-com.ps1 -Execute -ConfirmToken register-current-user-2F0E4C8F
```

执行型 unregister：

```powershell
dotnet run --project .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.Registration\RadishFlow.CapeOpen.Registration.csproj -- --action unregister --execute --confirm unregister-current-user-2F0E4C8F
```

通过仓库脚本执行：

```powershell
pwsh .\scripts\register-com.ps1 -Action unregister -Execute -ConfirmToken unregister-current-user-2F0E4C8F
```

当前冻结的组件标识：

- `CLSID`: `2F0E4C8F-7C89-4DA7-A5D3-5F8C987D6718`
- `ProgID`: `RadishFlow.CapeOpen.UnitOp.Mvp`
- `Versioned ProgID`: `RadishFlow.CapeOpen.UnitOp.Mvp.1`
- `TypeLib ID`: `9D9E5F0D-5E28-4A45-9E2A-70A39D4C8D11`
- `TypeLib Version`: `1.0`

当前 dry-run 计划口径：

- 默认 action 为 `register`
- 默认 scope 为 `current-user`，即计划输出 `HKCU\Software\Classes\...`
- `local-machine` 只输出 `HKLM\Software\Classes\...` 计划，不实际写入
- `unregister` 当前会先计划 `TypeLib` 反注册，再输出待删除的 `CLSID / ProgID / Versioned ProgID / TypeLib` 树
- 输出中的 `.NET COM hosting` 键值仍是前置规划；工具会先解析并校验 `RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll` 路径，当前不会退回旧 `.NET Framework` 的 `mscoree.dll` 注册口径
- 当前默认也会解析 `RadishFlow.CapeOpen.UnitOp.Mvp.tlb` 路径；开发构建下该文件会随 `UnitOp.Mvp` 与 `Registration` 输出目录一起复制，必要时也可显式传入 `--typelib <path>`
- preflight checks 只读检查 comhost 文件、comhost PE 机器类型、`UnitOp.Mvp.runtimeconfig/deps` sidecar、`TLB` 路径、`TypeLib GUID/version`、当前进程位数、scope 权限口径和目标 registry key 现状
- backup plan 只列出真实执行前应备份的 registry tree，不导出、不删除、不写入
- 默认 comhost 路径当前会优先回到仓库内真实 `UnitOp.Mvp\bin\<Configuration>\<TFM>\` 输出目录，并要求该目录同时包含 `RadishFlow.CapeOpen.UnitOp.Mvp.runtimeconfig.json` 与 `RadishFlow.CapeOpen.UnitOp.Mvp.deps.json`；若只找到被复制到 `Registration\bin\...` 或 `ContractTests\bin\...` 的 comhost，但缺少 runtime sidecar，则不会被优先选中
- 仓库脚本 `scripts/register-com.ps1` 当前也会默认显式传入 `UnitOp.Mvp\bin\<Configuration>\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll`
- 默认 type library 路径优先从已解析的 comhost 同目录或其 `typelib\` 子目录推导；开发构建下当前会直接解析到 `UnitOp.Mvp\bin\Debug\net10.0\typelib\RadishFlow.CapeOpen.UnitOp.Mvp.tlb`

当前执行门控口径：

- 默认行为仍保持 dry-run
- 真实写入必须显式传入 `--execute`
- `--confirm` 必须精确匹配当前 descriptor 暴露的 confirmation token
- 当前固定 token 形状为 `<action>-<scope>-<classid前8位>`，例如 `register-current-user-2F0E4C8F`
- 同一参数下 preflight 不存在 `Fail`
- `local-machine` scope 在 execute 模式下必须通过 elevation 检查
- execute `register` 当前会在写 `CLSID / ProgID / Versioned ProgID` 前先调用标准 `TypeLib` 注册 API：`current-user` 走 `RegisterTypeLibForUser(...)`，`local-machine` 走 `RegisterTypeLib(...)`
- execute `register` 当前也会在 `CLSID\{...}` 下写入 classic COM 所需的 `TypeLib` 关联值，避免只注册 `TypeLib` 树却不把 CLSID 回链到 typelib GUID
- execute `unregister` 当前会先调用标准 `TypeLib` 反注册 API，再清理 `CLSID / ProgID / Versioned ProgID / TypeLib` 四棵树
- 执行前会把 `CLSID / ProgID / Versioned ProgID / TypeLib` 四棵 registry tree 备份到 JSON
- 执行后会把 descriptor、执行结果、备份文件路径与回滚状态写入 execution log
- 若执行任一步骤失败，工具会尝试用刚捕获的备份恢复四棵树
- 实际写入范围仍严格限制在 dry-run registry plan 列出的 key/value 内

补充说明：

- `--backup-dir <path>` 可显式指定备份与执行日志目录
- 若未指定 `--backup-dir`，工具会在 `%LOCALAPPDATA%\RadishFlow\CapeOpen\RegistrationBackups\...` 下生成时间戳目录
- 当前 rollback 是“失败时自动恢复刚捕获的四棵树”，不是面向用户公开的独立 restore CLI
- 当前 contract tests 已锁住默认 dry-run、confirmation token 门控、preflight fail 阻断，以及 register / unregister 计划与备份范围
- 截至 2026-04-25，本工具的 dry-run JSON 已可直接输出 `ResolvedTypeLibraryPath`、`type library identity` 与 `comhost runtime layout` 预检结果，并包含显式 `RegisterTypeLibrary / UnregisterTypeLibrary` 计划步骤；真实 Windows PowerShell 5 复验已确认默认 `ICapeUtilities` 与 parameter collection 晚绑定调用不再触发 `0x80131165`，但是否已满足 `DWSIM / COFE` 仍需通过目标 PME 人工复验确认

本工具即使进入执行型注册阶段，也不应负责启动 PME、自动操作 PME UI、加载第三方 CAPE-OPEN 模型或生成安装包。目标 PME 人工验证路径见 `docs/capeopen/pme-validation.md`。
