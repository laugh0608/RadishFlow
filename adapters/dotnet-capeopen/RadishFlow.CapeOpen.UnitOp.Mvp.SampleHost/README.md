# RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost

该目录提供一个独立的最小外部 host console，并在 console 壳内新增一层更接近 PME host 的薄适配入口，用来验证：

- 外部宿主可以不依赖 `SmokeTests` 私有 driver
- 外部宿主可以直接复用 `UnitOp.Mvp` 的正式 `host view / request planner / round outcome` 消费面
- 外部宿主可以通过 `supplemental mutation phase` 写入非 blocking、但仍希望显式提供的配置，例如 property package `manifest/payload`
- PME-like 入口可以把“创建组件、初始化、读取视图、提交参数/端口对象、执行 validate/calculate round、读取正式结果面、终止”收口为薄宿主 session，而不引入 COM 注册、PME 自动化互调或第三方模型加载

样例固定演示的最小顺序为：

1. 创建 `RadishFlowCapeOpenUnitOperation`
2. 读取 constructed / initialized host views
3. 构造 `UnitOperationHostActionExecutionInputSet`
4. 用 `UnitOperationHostActionExecutionRequestPlanner.Plan(...)` 查看 ready request / missing input
5. 用 `UnitOperationHostRoundOrchestrator.Execute(...)` 一次执行 `ready actions -> supplemental mutations -> Validate -> Calculate`
6. 读取 `session / execution / port-material / report` 正式结果面
7. `Terminate()`

当前薄宿主入口：

- `PmeLikeUnitOperationHost`：负责创建 `RadishFlowCapeOpenUnitOperation`、配置 native library 目录并打开 session
- `PmeLikeUnitOperationSession`：负责持有单个 unit operation 实例、读取正式 host views、规划输入应用、执行 host round 和终止
- `PmeLikeUnitOperationInput`：负责承载宿主显式提供的 flowsheet JSON、package id、可选 manifest/payload 与 feed/product material object
- `PmeLikeUnitOperationRoundResult`：负责把 request plan、supplemental mutation commands 与正式 round outcome 一并返回给 console 展示层

这层入口刻意保持很薄：它不复用 `RadishFlow.CapeOpen.SmokeTests` 的 `UnitOperationSmokeHostDriver`，也不自行定义新的结果语义；所有核心判断仍来自 `UnitOp.Mvp` 的正式 reader / planner / round outcome。

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
