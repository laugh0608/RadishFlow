# Sample Components

该目录存放样例组件与参数集。

当前已提供首个本地物性包样例：

- `property-packages/binary-hydrocarbon-lite-v1/download.json`
- `property-packages/binary-hydrocarbon-lite-v1/manifest.json`
- `property-packages/binary-hydrocarbon-lite-v1/payload.rfpkg`

用途：

- 作为 `rf-store` / `rf-thermo` 当前本地缓存包 DTO 的真实样例
- 作为 `apps/radishflow-studio` 当前“控制面下载响应 -> 本地 payload DTO”映射的真实样例
- 作为应用私有缓存根目录 `<cache-root>/packages/<package-id>/<version>/` 的参考布局
- 作为后续下载落盘、provider 装载与样例 flowsheet 接线前的最小数据入口
