# RadishFlow.CapeOpen.SmokeTests

当前目录已建立第一版最小 smoke console 项目，用于验证：

- `.NET 10` Adapter 能创建 native engine
- 可加载 flowsheet json
- 可列出 property package registry
- 可触发 solve
- direct adapter 模式可读取 flowsheet / stream snapshot json
- `UnitOp.Mvp` 模式可校验最小成功结果契约 `status / summary / diagnostics`、失败路径的最小诊断摘要契约、统一只读查询面 `GetCalculationReport()`，以及建立在 report DTO 之上的最小文本导出面 `GetCalculationReportLineCount()` / `GetCalculationReportLine(int)` / `GetCalculationReportLines()` / `GetCalculationReportText()`

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
