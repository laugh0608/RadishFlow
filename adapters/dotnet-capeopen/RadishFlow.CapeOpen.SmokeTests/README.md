# RadishFlow.CapeOpen.SmokeTests

当前目录已建立第一版最小 smoke console 项目，用于验证：

- `.NET 10` Adapter 能创建 native engine
- 可加载 flowsheet json
- 可列出 property package registry
- 可触发 solve
- direct adapter 模式可读取 flowsheet / stream snapshot json
- `UnitOp.Mvp` 模式当前已切到“最小外部 host 驱动 + 结果消费样例”口径：样例侧通过 `UnitOperationSmokeHostDriver` 明确收口 `Initialize -> 配参数 -> 连端口 -> Validate -> Calculate -> 读结果 -> Terminate` 这一条最小宿主调用顺序，并在 `Calculate` 失败时先分类为 `InvocationOrder / Validation / Native`，再通过 `UnitOperationHostReportReader.Read(...)`、`UnitOperationHostReportPresenter.Present(...)` 和 `UnitOperationHostReportFormatter.Format(...)` 完成 `none / failure / success / none` 三态判断、稳定 detail 展示和 supplemental diagnostic lines 分区展示；当前 `unitop` smoke 已明确拆成两段：一段锁定细粒度边界矩阵，另一段输出 `Host Session Timeline`，按真实宿主多轮会话顺序覆盖“成功 -> native failure -> 恢复成功 -> validation failure -> 恢复成功 -> 终止”这类反复驱动路径；不再把 `LastCalculationResult` / `LastCalculationFailure` 视为外部宿主主消费面

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
- `--package <id>`
- `--manifest <path>`
- `--payload <path>`
- `--stream <id>`
- `--native-lib-dir <dir>`
- `--help`
