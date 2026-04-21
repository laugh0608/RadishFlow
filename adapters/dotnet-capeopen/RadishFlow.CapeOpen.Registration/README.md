# RadishFlow.CapeOpen.Registration

该目录当前提供第一版 `net10.0` 注册前置 / dry-run console。

当前职责：

- 输出自有 MVP Unit Operation PMC 的注册前置描述
- 冻结当前组件的 `CLSID / ProgID / Versioned ProgID`
- 列出将来注册时需要关联的 CAPE-OPEN component categories
- 列出当前最小实现承诺的 CAPE-OPEN 接口面

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

当前冻结的组件标识：

- `CLSID`: `2F0E4C8F-7C89-4DA7-A5D3-5F8C987D6718`
- `ProgID`: `RadishFlow.CapeOpen.UnitOp.Mvp`
- `Versioned ProgID`: `RadishFlow.CapeOpen.UnitOp.Mvp.1`
