# RadishFlow.CapeOpen.SmokeTests

当前目录已建立第一版最小 smoke console 项目，用于验证：

- `.NET 10` Adapter 能创建 native engine
- 可加载 flowsheet json
- 可列出 property package registry
- 可触发 solve
- direct adapter 模式可读取 flowsheet / stream snapshot json
- `UnitOp.Mvp` 模式当前已切到“最小外部 host 驱动 + 配置/结果五读取样例”口径：样例侧通过 `UnitOperationSmokeHostDriver` 明确收口 `Initialize -> 读 configuration snapshot / action plan / port-material snapshot / execution snapshot / session snapshot -> 通过 action execution 应用阻塞配置动作 -> 读 configuration snapshot / action plan / port-material snapshot / execution snapshot / session snapshot -> Validate -> Calculate -> 读 report / port-material snapshot / execution snapshot / session snapshot -> Terminate` 这一条最小宿主调用顺序，并在 `Calculate` 失败时先分类为 `InvocationOrder / Validation / Native`；对配置面，当前已优先通过 `UnitOperationHostConfigurationReader.Read(...)` 读取 readiness 和 blocking issues，再通过 `UnitOperationHostActionPlanReader.Read(...)` 读取分组 action checklist，并用 `UnitOperationHostActionExecutionDispatcher` 执行 parameter/port 类阻塞动作，非阻塞的可选 package file 输入仍作为 smoke 输入准备显式应用；对结果面，当前除 `UnitOperationHostReportReader.Read(...)`、`UnitOperationHostReportPresenter.Present(...)` 和 `UnitOperationHostReportFormatter.Format(...)` 之外，又新增 `UnitOperationHostPortMaterialReader.Read(...)`、`UnitOperationHostExecutionReader.Read(...)` 与 `UnitOperationHostSessionReader.Read(...)`，分别收口 host port 的 boundary stream/material 状态、本次求解的 summary/diagnostics/steps，以及带 canonical session state 的统一宿主整体状态摘要；当前 `unitop` smoke 已进一步分成十层：`driver` 负责最小宿主编排，`UnitOperationSmokeConfigurationAssertions` 负责共享配置/动作断言，`UnitOperationSmokePortMaterialAssertions` 负责共享 port/material 断言，`UnitOperationSmokeExecutionAssertions` 负责共享 execution 断言，`UnitOperationSmokeHostSessionAssertions` 负责共享 session 摘要断言，`UnitOperationSmokeReportAssertions` 负责共享报告断言，`UnitOperationSmokeBoundarySuite` 锁定细粒度边界矩阵，`UnitOperationSmokeSession` 负责多轮宿主会话 DSL，`UnitOperationSmokeScenarioCatalog` 负责场景模板目录与 runner，`Program.cs` 只保留最顶层调度；边界矩阵中仍保留少量直接 parameter/port 改写，用于刻意制造非法或 stale 状态；在此之上，当前已至少覆盖三条不同会话变体：`Host Session Timeline` 侧重 invocation/native/validation 混合回合，且每一轮都会显式输出 `sessionState`，`Host Recovery Timeline` 侧重 validation 恢复、feed 端口恢复与 native 恢复顺序，`Host Shutdown Timeline` 侧重 success 后只读报告消费、`Terminate()` 收尾与终止后阻断；后续继续补场景时，应优先向 catalog 增量追加定义，而不是回退到 `Program.cs` 内手写流程；不再把 `LastCalculationResult` / `LastCalculationFailure` 视为外部宿主主消费面
- `UnitOp.Mvp` 模式当前还支持 `--unitop-scenario <all|session|recovery|shutdown>`，默认值为 `all`；当只想复验某一类宿主路径时，可直接按场景过滤，避免每次都跑完整套 unitop smoke

默认行为：

- 默认读取 `examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json`
- 默认读取 `examples/sample-components/property-packages/binary-hydrocarbon-lite-v1`
- 默认从 `target/debug` 查找 `rf_ffi.dll`

默认样例中的 flowsheet 组件 ID 已与 `binary-hydrocarbon-lite-v1` 对齐，因此在先完成 `cargo build -p rf-ffi` 后，可直接用该 console 跑通：

```powershell
dotnet run --project .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.SmokeTests\RadishFlow.CapeOpen.SmokeTests.csproj --no-build -- --native-lib-dir D:\Code\RadishFlow\target\debug
```

可选参数：

- `--project <path>`
- `--unitop-scenario <all|session|recovery|shutdown>`
- `--package <id>`
- `--manifest <path>`
- `--payload <path>`
- `--stream <id>`
- `--native-lib-dir <dir>`
- `--help`
