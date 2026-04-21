# RadishFlow.CapeOpen.Registration

该目录当前提供第一版 `net10.0` 注册前置 / dry-run console。

当前职责：

- 输出自有 MVP Unit Operation PMC 的注册前置描述和 dry-run registry plan
- 冻结当前组件的 `CLSID / ProgID / Versioned ProgID`
- 列出将来注册时需要关联的 CAPE-OPEN component categories
- 列出当前最小实现承诺的 CAPE-OPEN 接口面
- 按 `register / unregister` 和 `current-user / local-machine` 生成将来会写入或删除的 registry key 计划

当前明确不做：

- 写 Windows Registry
- 注册或反注册 COM class
- 启动或自动化外部 PME
- 加载第三方 CAPE-OPEN 模型
- 安装证书、写系统目录或修改本机环境

运行示例：

```powershell
dotnet run --project .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.Registration\RadishFlow.CapeOpen.Registration.csproj
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

当前冻结的组件标识：

- `CLSID`: `2F0E4C8F-7C89-4DA7-A5D3-5F8C987D6718`
- `ProgID`: `RadishFlow.CapeOpen.UnitOp.Mvp`
- `Versioned ProgID`: `RadishFlow.CapeOpen.UnitOp.Mvp.1`

当前 dry-run 计划口径：

- 默认 action 为 `register`
- 默认 scope 为 `current-user`，即计划输出 `HKCU\Software\Classes\...`
- `local-machine` 只输出 `HKLM\Software\Classes\...` 计划，不实际写入
- `unregister` 只输出待删除的 `CLSID / ProgID / Versioned ProgID` 树，不实际删除
- 输出中的 `.NET COM hosting` 键值仍是前置规划；真实执行前必须先解析并校验 `RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll` 路径，当前不会退回旧 `.NET Framework` 的 `mscoree.dll` 注册口径
