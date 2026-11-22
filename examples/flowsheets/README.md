# Flowsheets

该目录存放示例流程。

当前示例包含以下类型：

- official hydrocarbon 示例：`feed-heater-flash-binary-hydrocarbon.rfproj.json`、`feed-cooler-flash-binary-hydrocarbon.rfproj.json`、`feed-valve-flash-binary-hydrocarbon.rfproj.json`、`feed-mixer-flash-binary-hydrocarbon.rfproj.json`
- synthetic demo 示例：`feed-heater-flash-synthetic-demo.rfproj.json`、`feed-cooler-flash-synthetic-demo.rfproj.json`、`feed-valve-flash-synthetic-demo.rfproj.json`、`feed-mixer-flash-synthetic-demo.rfproj.json`、`feed-mixer-heater-flash-synthetic-demo.rfproj.json`
- QS1 固定温压回流：[`fixed-tp-recycle-binary-hydrocarbon.rfproj.json`](fixed-tp-recycle-binary-hydrocarbon.rfproj.json)，工程 v4，0.6 回流、显式撕裂与独立最终复核；[CLI v4 查询](../automation/query-recycle-v4.request.json)无需物性缓存。非进料模板故意偏离解，结果只来自真实局部求值。
- QS4 首个混合 DAG：[`fixed-tp-mixed-material-binary-hydrocarbon.rfproj.json`](fixed-tp-mixed-material-binary-hydrocarbon.rfproj.json)，Local Feed → 四单元正向 EO → Local Heater；4 块 / 3 条真实边界，EO 为 24×24，最终产品总量 5 mol/s。工程 v4 新能力；[CLI v7 查询](../automation/query-mixed-material-v7.request.json)、[运行](../automation/run-mixed-material-v7.request.template.json)与[建图运行](../automation/build-mixed-material-v7.request.template.json)。
- [QS4 M3 串联回流区的整图对照](fixed-tp-serial-recycle-eo-reference.rfproj.json)使用现有 v4 正向物料 EO，含两个单向串联的回流区、13 股物流和 46×46 方程。独立分数基准和 CLI v6 重放见 [M3 算例](../../docs/topics/modeling/mixed-serial-recycle.md#独立基准与现有边界)。CLI v15 读取应用保存的 M3 两区计划，重放方式见 [保存产物与验收表](../../docs/topics/modeling/mixed-serial-recycle.md#实际保存产物与验收表)。
- QS3 固定温压物料 EO：[`fixed-tp-material-eo-binary-hydrocarbon.rfproj.json`](fixed-tp-material-eo-binary-hydrocarbon.rfproj.json)，两股新鲜进料 / 交叉回流、释放一个分流比例、产品目标 1.5 mol/s。工程 v4 新能力；[CLI v6 查询](../automation/query-material-eo-v6.request.json)与[采纳后改正向重算](../automation/adopt-material-eo-v6.request.template.json)复用正式动作，运行方式见 CLI 参考。
- [QS3 终端产品组成反算](fixed-tp-product-composition-binary-hydrocarbon.rfproj.json) 是上述双进料交叉回流工程另存的 v4 工况。目标为产品 B 中甲烷 0.35 mol/mol，A 侧已保存的分流比例为 0.6。使用支持 EO 能力版本 2 的工程读取入口重开，结果为 Missing。
- [单相显热双进料](two-feed-single-vapor-sensible.rfproj.json) 使用 v4 和工程自带的合成气相热容。绝热混合后加热到 380 K，带符号热负荷为 +7800 W。可用 Studio 或 CLI v11 运行并审阅能量残差。
- H3 同心套管算例在各自 v4 工程中保存冷热侧纯水流股关联。冷侧 [40 mol/s](h3-linked-water-40.rfproj.json) 三项通过，[50 mol/s](h3-linked-water-50.rfproj.json) 仅环隙压降超限。两档均可用 CLI v14 或 Studio 复核。 [请求与基准](../../docs/topics/equipment/heat-exchanger-design-and-rating.md#h3-流程关联-cli-v14)
- PME 验证示例：`feed-heater-flash-water-ethanol.rfproj.json`

其中，official hydrocarbon 示例使用 methane / ethane，并在项目文件的 `flowsheet.thermo.property_package_id` 中持久化 `binary-hydrocarbon-lite-v1`。Studio 的 Home、独立物性页、左侧项目输入和运行路径读取同一物性包状态；界面主展示会把该稳定 id 映射为可读 label `二元烃 Lite`，项目文件和命令边界仍保留稳定 id。

synthetic demo 族继续使用 `component-a/component-b` 与 `binary-hydrocarbon-synthetic-demo-v1` 语义，主要服务 solver / integration / interop 回归，不再与 official methane/ethane 示例共用泛化命名。

`binary-hydrocarbon-basis.rfproj.json` 仅含 methane / ethane 与物性包选择，不含设备和流股，供 [B3-5 建模请求](../automation/build-feed-heater-flash.request.json) 创建流程。使用方式及实际缓存配置见 [CLI 参考](../../docs/reference/headless-cli.md)。这些示例验证软件行为，不构成独立工程准确性证明。

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
