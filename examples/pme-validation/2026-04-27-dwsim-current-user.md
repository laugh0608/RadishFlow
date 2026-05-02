# DWSIM PME Validation Record

Date: `2026-04-27`
Validator: `User-side manual PME validation; Codex documentation consolidation`
RadishFlow commit: `10f19101cfe28e7a67f5b0e532a37433fade1071`
OS: `Microsoft Windows 10.0.26200.8246`
PME: `DWSIM`
PME version: `9.0.2`
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

Discovery: `Pass`。DWSIM 能发现 `RadishFlow MVP Unit Operation` / `RadishFlow.CapeOpen.UnitOp.Mvp`。
Activation: `Pass`。DWSIM 能实例化当前 PMC 并放置到 flowsheet 画布。
Identity: `Pass`。DWSIM 能读取并设置 `ComponentName` / `ComponentDescription`，字段语义与 `UnitOperationComIdentity` 一致。
Parameters: `Pass`。DWSIM 能枚举最小参数集合，并按 `ICapeIdentification`、`ICapeParameterSpec`、`ICapeOptionParameterSpec` 与 `ICapeParameter` 消费 parameter placeholder。
Ports: `Pass`。DWSIM 能读取 `Feed` / `Product` 端口，方向分别为 inlet / outlet，类型为 material。
Connection: `Pass`。DWSIM 能连接 `Feed` 与 `Product` material streams。
Validate: `Pass`。配置 water/ethanol MVP 参数后，DWSIM 可触发 `Validate()` 并进入 valid 主路径。未配置必填参数时返回 invalid 属于预期行为。
Calculate: `Pass`。water/ethanol 样例可触发 `Calculate()`，并完成 native solve、Feed material overlay 与 Product material publication。
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
- DWSIM consumes the unit operation through `InitNew -> Initialize -> SimulationContext set -> identification set -> Ports -> Parameters`.
- The `ICapeUtilities` front slot order must remain compatible with DWSIM's setter-only PIA shape.
- DWSIM parameter enumeration expects `Parameters.Item(i)` to return an object that directly supports identification, specification and parameter interfaces.
- The earlier `AutomaticTranslation.AutomaticTranslator.SetMainWindow(...)` `NullReferenceException` is treated as DWSIM startup noise because it occurs before RadishFlow UnitOp activation.

Decision: `Pass`
Follow-up:
- Keep the current DWSIM path as a manual PME regression baseline.
- Do not treat the thermodynamics registration categories as a promise to implement full Thermo PMC in MVP.
- COM trace is now an explicit diagnostic opt-in and should only be enabled when a future PME failure needs call-order evidence.
