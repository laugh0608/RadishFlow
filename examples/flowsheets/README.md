# Flowsheets

该目录存放示例流程。

- `feed-mixer-flash.rfproj.json`: 当前最小可求解示例，覆盖 `Feed1 + Feed2 -> Mixer -> Flash Drum`
- `feed-mixer-heater-flash.rfproj.json`: 第二条组合示例，覆盖 `Feed1 + Feed2 -> Mixer -> Heater -> Flash Drum`
- `feed-heater-flash.rfproj.json`: 第三条最小可求解示例，覆盖 `Feed -> Heater -> Flash Drum`
- `feed-cooler-flash.rfproj.json`: 第四条最小可求解示例，覆盖 `Feed -> Cooler -> Flash Drum`
- `feed-valve-flash.rfproj.json`: 第五条最小可求解示例，覆盖 `Feed -> Valve -> Flash Drum`

同时，`failures/` 子目录当前开始承载仓库级负向回归夹具：

- `failures/valve-execution-failure.rfproj.json`: 覆盖 `solver.step.execution`
- `failures/unsupported-unit-kind.rfproj.json`: 覆盖 `solver.connection_validation`
- `failures/self-loop-cycle.rfproj.json`: 覆盖 `solver.topological_ordering`
- `failures/missing-upstream-source.rfproj.json`: 覆盖缺失上游 source 的 `solver.connection_validation`
- `failures/missing-stream-reference.rfproj.json`: 覆盖缺失 stream 引用的 `solver.connection_validation`
- `failures/invalid-port-signature.rfproj.json`: 覆盖 canonical port signature 不匹配的 `solver.connection_validation`
