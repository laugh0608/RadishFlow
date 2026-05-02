# COFE PME Validation Record

Date: `2026-04-27`
Validator: `User-side manual PME validation; Codex documentation consolidation`
RadishFlow commit: `10f19101cfe28e7a67f5b0e532a37433fade1071`
OS: `Microsoft Windows 10.0.26200.8246`
PME: `COFE`
PME version: `COCO 3.9 / COFE`
PME bitness: `64-bit`
Registry scope: `current-user`
Comhost path: `D:\Code\RadishFlow\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp\bin\Debug\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll`
Dry-run command: `pwsh .\scripts\register-com.ps1 -Scope current-user -SkipBuild -ComHostPath .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp\bin\Debug\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll -Json`
Registration command: `pwsh .\scripts\pme-register-latest.ps1`
Unregistration command: `pwsh .\scripts\pme-unregister.ps1`
Preflight result: `Pass`
Warnings accepted: `None`
Register post-check:
- `HKCU\Software\Classes\CLSID\{2F0E4C8F-7C89-4DA7-A5D3-5F8C987D6718}` exists after register.
- `HKCU\Software\Classes\RadishFlow.CapeOpen.UnitOp.Mvp` exists after register.
- `HKCU\Software\Classes\RadishFlow.CapeOpen.UnitOp.Mvp.1` exists after register.

Discovery: `Pass`。COFE 能发现 `RadishFlow MVP Unit Operation` / `RadishFlow.CapeOpen.UnitOp.Mvp`。
Activation: `Pass`。COFE 能实例化并放置当前 PMC。
Identity: `Pass`。COFE 能读取 UnitOp identification。
Parameters: `Pass`。COFE 能读取最小参数集合，并能在配置 MVP 参数后驱动 validation / calculation。
Ports: `Pass`。COFE 能读取 `Feed` / `Product` material ports。
Connection: `Pass`。COFE 能连接 inlet / outlet material streams；关闭 case 时先前的 material object release warning 已消失。
Validate: `Pass`。配置 water/ethanol MVP 参数后，COFE 可触发 `Validate()` 并返回 valid。未配置 `Flowsheet Json` 时返回 invalid 属于预期必填参数校验。
Calculate: `Pass`。water/ethanol 样例可触发 `Calculate()`，完成 native solve、Product material 写回和 CAPE-OPEN 1.1 `CalcEquilibrium(TP)`。
Report: `Pass`。成功路径可读取最小报告 / 诊断摘要；失败路径仍通过既有 report / ECape 语义暴露诊断。
Unregister: `Pass`。通过受控脚本执行 `current-user` 反注册。
Unregister post-check:
- `HKCU\Software\Classes\CLSID\{2F0E4C8F-7C89-4DA7-A5D3-5F8C987D6718}` absent after unregister.
- `HKCU\Software\Classes\RadishFlow.CapeOpen.UnitOp.Mvp` absent after unregister.
- `HKCU\Software\Classes\RadishFlow.CapeOpen.UnitOp.Mvp.1` absent after unregister.

Logs:
- Registration backup / execution log: `D:\Code\RadishFlow\artifacts\registration-validation\register-current-user\`
- Unregistration backup / execution log: `D:\Code\RadishFlow\artifacts\registration-validation\unregister-current-user\`
- Earlier diagnostic trace: `D:\Code\RadishFlow\artifacts\pme-trace\radishflow-unitop-trace.log`
- Earlier dump analysis artifacts: `D:\Code\RadishFlow\artifacts\pme-dumps\`

Sample inputs:
- Flowsheet JSON: `D:\Code\RadishFlow\examples\flowsheets\feed-heater-flash-water-ethanol.rfproj.json`
- Property package id: `water-ethanol-lite-v1`
- Manifest: `D:\Code\RadishFlow\examples\sample-components\property-packages\water-ethanol-lite-v1\manifest.json`
- Payload: `D:\Code\RadishFlow\examples\sample-components\property-packages\water-ethanol-lite-v1\payload.rfpkg`

Key observations:
- COFE requires `SimulationContext` getter to return a non-null dispatch-compatible object.
- COFE can complete material object publication through CAPE-OPEN 1.1 material / equilibrium interfaces.
- The previous `one or more output streams were not flashed` and mass balance warnings were resolved by Product material publication and Feed material overlay.
- UnitOp currently keeps a live connected PME material object reference during connection and releases the UnitOp-held RCW on disconnect / terminate.

Decision: `Pass`
Follow-up:
- Keep the current COFE path as a manual PME regression baseline.
- Treat connected PME material object access as a short-lived interop boundary; do not promote live PME material objects into durable product state.
- COM trace is now an explicit diagnostic opt-in and should only be enabled when a future PME failure needs call-order evidence.
