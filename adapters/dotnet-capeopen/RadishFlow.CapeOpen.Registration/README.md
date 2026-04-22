# RadishFlow.CapeOpen.Registration

该目录当前提供第一版 `net10.0` 注册前置 / 执行门控 console。

当前职责：

- 输出自有 MVP Unit Operation PMC 的注册前置描述和 dry-run registry plan
- 冻结当前组件的 `CLSID / ProgID / Versioned ProgID`
- 列出将来注册时需要关联的 CAPE-OPEN component categories
- 列出当前最小实现承诺的 CAPE-OPEN 接口面
- 按 `register / unregister` 和 `current-user / local-machine` 生成将来会写入或删除的 registry key 计划
- 解析并检查 `RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll`
- 只读检查目标 registry key 是否已存在，并输出备份范围
- 在显式 `--execute` + `--confirm` 下执行 register / unregister
- 在执行前导出三棵 registry tree 的 JSON 备份，并写出 execution log
- 若执行中途失败，按刚捕获的备份自动回滚 `CLSID / ProgID / Versioned ProgID` 三棵树

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

当前 dry-run 计划口径：

- 默认 action 为 `register`
- 默认 scope 为 `current-user`，即计划输出 `HKCU\Software\Classes\...`
- `local-machine` 只输出 `HKLM\Software\Classes\...` 计划，不实际写入
- `unregister` 只输出待删除的 `CLSID / ProgID / Versioned ProgID` 树，不实际删除
- 输出中的 `.NET COM hosting` 键值仍是前置规划；工具会先解析并校验 `RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll` 路径，当前不会退回旧 `.NET Framework` 的 `mscoree.dll` 注册口径
- preflight checks 只读检查 comhost 文件、comhost PE 机器类型、当前进程位数、scope 权限口径和目标 registry key 现状
- backup plan 只列出真实执行前应备份的 registry tree，不导出、不删除、不写入
- 默认 comhost 路径从当前加载到 `Registration` 进程中的 `RadishFlow.CapeOpen.UnitOp.Mvp` assembly 目录推导；开发构建下通常是 `Registration\bin\Debug\net10.0` 中被复制的项目引用产物。发布/安装工具后应显式传入最终安装目录中的 `--comhost <path>`

当前执行门控口径：

- 默认行为仍保持 dry-run
- 真实写入必须显式传入 `--execute`
- `--confirm` 必须精确匹配当前 descriptor 暴露的 confirmation token
- 当前固定 token 形状为 `<action>-<scope>-<classid前8位>`，例如 `register-current-user-2F0E4C8F`
- 同一参数下 preflight 不存在 `Fail`
- `local-machine` scope 在 execute 模式下必须通过 elevation 检查
- 执行前会把 `CLSID / ProgID / Versioned ProgID` 三棵 registry tree 备份到 JSON
- 执行后会把 descriptor、执行结果、备份文件路径与回滚状态写入 execution log
- 若执行任一步骤失败，工具会尝试用刚捕获的备份恢复三棵树
- 实际写入范围仍严格限制在 dry-run registry plan 列出的 key/value 内

补充说明：

- `--backup-dir <path>` 可显式指定备份与执行日志目录
- 若未指定 `--backup-dir`，工具会在 `%LOCALAPPDATA%\RadishFlow\CapeOpen\RegistrationBackups\...` 下生成时间戳目录
- 当前 rollback 是“失败时自动恢复刚捕获的三棵树”，不是面向用户公开的独立 restore CLI
- 当前 contract tests 已锁住默认 dry-run、confirmation token 门控、preflight fail 阻断，以及 register / unregister 计划与备份范围

本工具即使进入执行型注册阶段，也不应负责启动 PME、自动操作 PME UI、加载第三方 CAPE-OPEN 模型或生成安装包。目标 PME 人工验证路径见 `docs/capeopen/pme-validation.md`。
