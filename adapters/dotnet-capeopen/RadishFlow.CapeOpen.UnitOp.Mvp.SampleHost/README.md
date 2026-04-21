# RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost

该目录提供一个独立的最小外部 host console，用来验证：

- 外部宿主可以不依赖 `SmokeTests` 私有 driver
- 外部宿主可以直接复用 `UnitOp.Mvp` 的正式 `host view / request planner / round outcome` 消费面
- 外部宿主可以通过 `supplemental mutation phase` 写入非 blocking、但仍希望显式提供的配置，例如 property package `manifest/payload`

样例固定演示的最小顺序为：

1. 创建 `RadishFlowCapeOpenUnitOperation`
2. 读取 constructed / initialized host views
3. 构造 `UnitOperationHostActionExecutionInputSet`
4. 用 `UnitOperationHostActionExecutionRequestPlanner.Plan(...)` 查看 ready request / missing input
5. 用 `UnitOperationHostRoundOrchestrator.Execute(...)` 一次执行 `ready actions -> supplemental mutations -> Validate -> Calculate`
6. 读取 `session / execution / port-material / report` 正式结果面
7. `Terminate()`

默认行为：

- 默认读取 `examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json`
- 默认读取 `examples/sample-components/property-packages/binary-hydrocarbon-lite-v1`
- 默认从 `target/debug` 查找 `rf_ffi.dll`

运行示例：

```powershell
dotnet run --project .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost\RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost.csproj -- --native-lib-dir D:\Code\RadishFlow\target\debug
```

可选参数：

- `--project <path>`
- `--package <id>`
- `--manifest <path>`
- `--payload <path>`
- `--native-lib-dir <dir>`
- `--help`
