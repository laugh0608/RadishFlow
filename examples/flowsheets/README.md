# Flowsheets

该目录存放示例流程。

当前示例分两类：

- official hydrocarbon 示例：`feed-heater-flash-binary-hydrocarbon.rfproj.json`、`feed-cooler-flash-binary-hydrocarbon.rfproj.json`、`feed-valve-flash-binary-hydrocarbon.rfproj.json`、`feed-mixer-flash-binary-hydrocarbon.rfproj.json`
- synthetic demo 示例：`feed-heater-flash-synthetic-demo.rfproj.json`、`feed-cooler-flash-synthetic-demo.rfproj.json`、`feed-valve-flash-synthetic-demo.rfproj.json`、`feed-mixer-flash-synthetic-demo.rfproj.json`、`feed-mixer-heater-flash-synthetic-demo.rfproj.json`
- PME 验证示例：`feed-heater-flash-water-ethanol.rfproj.json`

其中，synthetic demo 族继续使用 `component-a/component-b` 与 `binary-hydrocarbon-synthetic-demo-v1` 语义，主要服务 solver / integration / interop 回归，不再与 official methane/ethane 示例共用泛化命名。

同时，`failures/` 子目录当前开始承载仓库级负向回归夹具：

- `failures/valve-execution-failure.rfproj.json`: 覆盖 `solver.step.execution`
- `failures/unsupported-unit-kind.rfproj.json`: 覆盖 `solver.connection_validation`
- `failures/self-loop-cycle.rfproj.json`: 覆盖 `solver.topological_ordering`
- `failures/multi-unit-cycle.rfproj.json`: 覆盖多单元循环的 `solver.topological_ordering`
- `failures/missing-upstream-source.rfproj.json`: 覆盖缺失上游 source 的 `solver.connection_validation`
- `failures/missing-stream-reference.rfproj.json`: 覆盖缺失 stream 引用的 `solver.connection_validation`
- `failures/invalid-port-signature.rfproj.json`: 覆盖 canonical port signature 不匹配的 `solver.connection_validation`
- `failures/duplicate-downstream-sink.rfproj.json`: 覆盖重复下游 sink 的 `solver.connection_validation`
- `failures/orphan-stream.rfproj.json`: 覆盖孤立 stream 的 `solver.connection_validation`
- `failures/unbound-inlet-port.rfproj.json`: 覆盖未绑定 stream 的 inlet port `solver.connection_validation`
- `failures/unbound-outlet-port.rfproj.json`: 覆盖未绑定 stream 的 outlet port `solver.connection_validation`
