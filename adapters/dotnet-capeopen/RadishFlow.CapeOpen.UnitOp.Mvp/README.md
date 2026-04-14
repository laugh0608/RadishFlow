# RadishFlow.CapeOpen.UnitOp.Mvp

当前目录已从纯占位推进为第一版最小 `net10.0` PMC 骨架项目，职责只限于：

- 提供一个最小 `ICapeIdentification` + `ICapeUtilities` + `ICapeUnit` 实现类
- 为后续真正的 CAPE-OPEN Unit Operation PMC 留出项目边界和最小状态机
- 预留内部 flowsheet/package 配置入口，但不提前引入完整 native 运行时编排

当前已包含的最小公共面：

- `RadishFlowCapeOpenUnitOperation`
- `Initialize / Validate / Calculate / Terminate / Edit` 的第一版状态骨架
- 内部 `LoadFlowsheetJson(...)`、`LoadPropertyPackageFiles(...)`、`SelectPropertyPackage(...)` 配置入口
- `Calculate()` 对未满足前置条件的最小 ECape 语义抛错骨架

当前明确不包含：

- COM 注册 / 反注册
- 稳定 CLSID / ProgID 策略
- 端口集合、参数集合、报告接口的正式实现
- PME 生命周期集成
- 完整 CAPE-OPEN PMC 运行时

说明：

- 当前 `Ports` 和 `Parameters` 仍返回空占位，目的是先把 `UnitOp.Mvp` 的接口实现与状态边界固定下来
- `Calculate()` 当前只负责前置校验和最小 ECape 语义抛错，不代表最终 PMC 的真实求解入口已经完成
- Rust/.NET 边界仍保持为句柄 + UTF-8 + JSON + 状态码，没有在这里提前引入 COM 注册或更宽的跨边界对象传递
