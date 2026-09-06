# Sample Components

该目录存放用于软件演示、装载与互操作回归的样例组件和参数集。

`binary-hydrocarbon-lite-v1` 的 methane / ethane 名称及 `water-ethanol-lite-v1` 的 PME 记录不代表真实物性准确性已验证；`official` 仅表示仓库标准示例。关联式适用范围、能量近似和独立数据证据缺口见 [热力学模型](../../docs/thermo/mvp-model.md#样例身份与适用范围)。

当前已提供首个本地物性包样例：

- `property-packages/binary-hydrocarbon-lite-v1/download.json`
- `property-packages/binary-hydrocarbon-lite-v1/manifest.json`
- `property-packages/binary-hydrocarbon-lite-v1/payload.rfpkg`

当前还提供一套面向 `COFE / DWSIM` 人工 PME 复验的常见二元体系样例：

- `property-packages/water-ethanol-lite-v1/download.json`
- `property-packages/water-ethanol-lite-v1/manifest.json`
- `property-packages/water-ethanol-lite-v1/payload.rfpkg`

配套 flowsheet：

- `examples/flowsheets/feed-heater-flash-water-ethanol.rfproj.json`

PME 人工复验时，宿主 property package / material streams 应包含 `water` 与 `ethanol` 两个 compounds，并使用以下 UnitOp 参数：

- `Flowsheet Json`: `examples/flowsheets/feed-heater-flash-water-ethanol.rfproj.json` 的压缩 JSON 文本
- `Property Package Id`: `water-ethanol-lite-v1`
- `Property Package Manifest Path`: `D:\Code\RadishFlow\examples\sample-components\property-packages\water-ethanol-lite-v1\manifest.json`
- `Property Package Payload Path`: `D:\Code\RadishFlow\examples\sample-components\property-packages\water-ethanol-lite-v1\payload.rfpkg`

用途：

- 作为 `rf-store` / `rf-thermo` 当前本地缓存包 DTO 的可装载样例
- 作为 `apps/radishflow-studio` 当前“控制面下载响应 -> 本地 payload DTO”映射的协议样例
- 作为应用私有缓存根目录 `<cache-root>/packages/<package-id>/<version>/` 的参考布局
- 作为后续下载落盘、provider 装载与样例 flowsheet 接线前的最小数据入口
