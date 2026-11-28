# 固定温压求解计划的 CLI v4–v6 契约

更新时间：2026-11-28

本地脚本与适配器可用本篇核对分块回流、仿射联立与物料联立方程三种计划的请求、结果及版本保护。通用动作、项目指纹和退出码见 [CLI 总参考](headless-cli.md)，混合分区见 [CLI v7](headless-cli-v7.md)。

## QS1-C CLI v4（2026-10-31，已实现）

`query` / `execute` 根据请求 `schema_version` 显式分派 3 / 4；请求版本与工程版本独立。v4 读取已知工程 v1—v4，未知 v4 能力只给安全元数据与只读原因，不进入可编辑工作区。v3 在任何编辑 / 资源读取 / 求解前拒绝工程 v4；inspect / run v1/v2 和现有 FFI 保持原能力。v4 不增加保存、项目 JSON 导出、sidecar / 偏好或缓存写入；路径、expected_project 原始文件 SHA-256、请求局部步骤引用和退出码延续 v3。

新增严格词汇：

| 入口 | v4 增量 |
| --- | --- |
| `results` / `actions` 的 `scope: "flow_plan"` | 正式 configuration、plan_revision、计划拓扑 ready / not_ready、simulation_dirty、方法与动作；ready 不替代输入 / 物性 / 授权检查 |
| `configure_flow_plan` | 必填 expected_plan_revision 与整份 plan，原子应用；允许保存边界尚未就绪但形状合法的配置，不修改物理 revision |
| `run_flow` | 同时必填 expected_revision 和 expected_plan_revision；只运行已应用计划，实际 Run 仍要求 flow_resources |
| `create_unit` | unit_kind 新增 splitter；原六类保留 |
| `write` | Splitter 输入 field 新增 outlet_a_split_fraction，Number / SI / DimensionlessRatio / [0,1]，无缺省比例 |
| `connect_exact` | 必填 expected_revision、source / sink `{unit,port}` 和 stream 身份；允许显式闭环，不替换已占用端口；旧 connect 仍使用原无环建议规则 |

计划 DTO 独立于文件 DTO。method 是 `radishflow.acyclic-sm/1` 或 `radishflow.partitioned-sm/1`；无环计划只有 method。分块计划示例：

```json
{
  "kind": "configure_flow_plan",
  "expected_plan_revision": 0,
  "plan": {
    "method": "radishflow.partitioned-sm/1",
    "acceptance_contract_version": 1,
    "temperature_k": 300,
    "pressure_pa": 650000,
    "max_iteration_evaluations": 500,
    "tears": [{
      "stream": {"id": "s-recycle"},
      "source": {"unit": {"id": "splitter-1"}, "port": "outlet_a"},
      "sink": {"unit": {"id": "mixer-2"}, "port": "inlet_b"},
      "initialization": {"kind": "zero_component_flows"},
      "residual_scaling": {"kind": "evaluated_fresh_total"}
    }]
  }
}
```

初值也可为 `{"kind":"component_flows","values":[{"component_id":"methane","value_mol_s":0}]}`；尺度可为 `component_values`，同样使用 values。运行就绪时显式向量必须完整绑定项目组分，初值非负、尺度严格正，重复条目和非有限数拒绝。上述单条仅说明形状，不是二元工程的完整向量。撕裂流股和端点支持 v3 身份引用；plan revision 的前序引用使用 `{"step":"configure","output":"plan"}`，只接受已经成功且确实输出计划修订的步骤。预算为 1—500 整数，独立最终复核另计一次。

入口 response.project 保留实际读取的 project_schema_version / file_sha256；另有 in_memory_required_project_schema_version，表示当前内存状态需要的保存版本，不声称已写盘。结果 `execution_kind=acyclic_sm|partitioned_sm`：Current 分块载荷包含生产者 streams、按执行顺序的 blocks / members、最终真实局部 I/O、assumed / returned、逐组分残差 / 尺度 / 固定容差、独立复核与实际 costs。来源包含实际方法 / 验收契约 / 规范配置指纹和已加载包证据；运行身份继续使用请求内 step 引用。Stale / Missing / Failed 不给正式 fields；失败试算值、残差和调用数只在 diagnostic_evidence。

设备提案沿用相同领域预览与规范摘要编码；v4 使用 `headless-v4-confirm-v1:`，覆盖原文件、前序动作、正式计划及内存能力变化，旧 v3 摘要不能代用。计划改变或改回后，既有结果 / 在途任务 / 已准备输出的资格都需要重验；计划资格不并入 QS0 或普通工程编辑身份。

可直接查询[公开回流工程](../../examples/flowsheets/fixed-tp-recycle-binary-hydrocarbon.rfproj.json)：

```sh
target/debug/radishflow-studio --headless query examples/automation/query-recycle-v4.request.json
```

[运行请求模板](../../examples/automation/run-recycle-v4.request.template.json)给出完整期望与显式资源位置；先将 flow_resources 指向已有、未到期的 binary-hydrocarbon-lite-v1 缓存 / 索引，工程内容改变后重新 query 获取入口 SHA。CLI 不创建替代物性包。本机代表得到 48 次迭代 + 1 次复核、102 次局部调用；0.99 达预算失败和 1 无有限稳态的证据不发布为 Current。测试、实窗与未测平台边界见 [QS1-C](../topics/modeling/recycle-project-and-consumers.md)及 [W44](../devlogs/2026-10/2026-W44.md)。

## QS2-C CLI v5（2026-10-31，已实现）

[D06-06](../topics/modeling/simultaneous-project-and-consumers.md)已批准。query / execute 的请求与响应 `schema_version=5`，工程外层仍为 v4；v5 支持原 Acyclic / Partitioned 和新增固定温压仿射联立计划。旧 v4 在创建可编辑工作区前按固定能力范围将 QS2 工程保护为整份只读，执行在资源读取与步骤之前拒绝；v1—v3 原边界保持。

配置请求沿 `configure_flow_plan` 的 `expected_plan_revision`，新 Plan 形状为：

```json
{
  "method": "radishflow.fixed-tp-affine-modular-lu/1",
  "acceptance_contract_version": 1,
  "response_contract_version": 1,
  "temperature_k": 300.0,
  "pressure_pa": 650000.0,
  "scaling": {"kind": "evaluated_fresh_total"}
}
```

- 显式尺度使用 `scaling={kind:"explicit_couplings",values:[...]}`，每项含 `stream`、`source`、`sink`、`component_id`、`variable_mol_s` 与 `residual_mol_s`。前两类端口 / 流股引用沿 IdentityRef / Port，支持前序 receipt；必须完整覆盖所有联立块，重复 / 缺项 / 错绑定不自动补齐。形状合法但图未就绪可配置，Run 明确拒绝。
- `run_flow` 同时检查物理与计划修订；`flow_plan` 查询输出规范配置、就绪诊断、耦合预览和内存所需能力。v5 `connect_exact` 可省略 `sink` 创建产品出口，显式 null 拒绝；旧 v4 仍要求 `sink`。
- 成功结果为 `execution_kind="simultaneous_modular"`，含真实生产者 `streams`、`blocks`、`final_units`、`coupling` / `linear` 和 `costs`，不伪造 tears 或迭代次数。失败只有 `diagnostic_evidence`（含底层 `error_code` 和可选诊断 `code`），不提供正式 fields / streams；Stale / Missing 同样不提供值。未取得数值为 null；失败证据中的 NaN / ±Infinity 为 `{non_finite:"nan"|"positive_infinity"|"negative_infinity"}`，不折叠成 null。字段明细归 D06-04。
- `origin.execution.plan` 包含冻结 method、capability、response / acceptance 版本、规范配置 SHA-256 及实际 `core_method`；包加载失败时 core_method 为 null，implementation 为 `not-invoked`。实际包来源只取本次已加载事实。
- Project 响应的 `project_schema_version` / `file_sha256` 继续表示入口原文件；另列 `in_memory_required_project_schema_version` / `in_memory_required_capabilities`，不暗示内存已保存。影响确认前缀为 `headless-v5-confirm-v1:`，不接受 v3 / v4 摘要；普通配置 / Run 无额外确认。

[四成员公开工程](../../examples/flowsheets/fixed-tp-cross-recycle-binary-hydrocarbon.rfproj.json)采用 α=0.6、β=0.25，内部模板刻意不是正确解。先运行[查询模板](../../examples/automation/query-cross-recycle-v5.request.json)，再填写[运行模板](../../examples/automation/run-cross-recycle-v5.request.template.json)的既有有效缓存路径；也可用[正式动作建图模板](../../examples/automation/build-cross-recycle-v5.request.template.json)从空工况创建同一模型、配置并 Run。文件改动后重新查询 SHA；CLI 不创建物性缓存、不保存工程。

```sh
target/debug/radishflow-studio --headless query examples/automation/query-cross-recycle-v5.request.json
target/debug/radishflow-studio --headless execute examples/automation/run-cross-recycle-v5.request.template.json
```

本机真实子进程已验证两个运行模板、三块 / 八未知量、10 次局部调用 / 4 份响应 / 1 次分解 / 9 次三角求解、独立分数组分基准及失败 / Stale / 重开 Missing；整个临时目录（含工程、缓存、索引、sidecar）的字节和条目保持，未创建偏好文件。历史 `8c18f223` 存储库另编译验证只读 / 拒绝覆盖；GUI、Windows / Linux、COM 与远端 CI 未验证，证据归 [W44](../devlogs/2026-10/2026-W44.md)。


## QS3 CLI v6（2026-10-31，已实现）

[QS3-D08](../topics/modeling/equation-oriented-block.md#d08cli-v6验证矩阵与连续开发)使用 `schema_version=6`，工程仍为 v4。旧 v5 固定 Simultaneous 能力，EO 工程只读，execute 在步骤和资源加载前拒绝；v1—v4 不扩词汇。

`configure_flow_plan` 新增 `radishflow.fixed-tp-material-eo/1`。配置、IdentityRef 及尺度 wire 见模板与 QS3-D08。

查询 `results` 的 `flow_plan` scope 返回规范配置、ready、equation_preview（变量 / 方程、匹配与尺度）及能力范围，不加载物性；`flow` Current 返回 `execution_kind=equation_oriented`、真实流股 / 单元与 evidence。失败仅 diagnostic_evidence，不返回正式 fields / streams；保留缺失 / 有限 / tagged 非有限三态及实际 backend 来源。

`adopt_eo_split_fraction` 必填 expected_revision、expected_plan_revision、`execution:{"step":"本请求已接收的 run_flow 步骤"}` 与精确 target（unit object、inputs、outlet_a_split_fraction）。不接受调用者数值或跨请求执行引用。回执含来源、before / after revision、原值 / 采纳值和 persisted=false；单输入事务使结果 Stale、计划保留、不自动重算，相同值 unchanged。固定比例运行需显式改正向再 Run。

确认前缀 `headless-v6-confirm-v1:`；动作只影响请求内存，已验证目录及文件字节不变。

- [EO 工程](../../examples/flowsheets/fixed-tp-material-eo-binary-hydrocarbon.rfproj.json)：产品目标 1.5 mol/s，解 alpha=10/19；非进料模板不是结果。
- [查询](../../examples/automation/query-material-eo-v6.request.json)：无需物性资源。
- [反算运行](../../examples/automation/run-material-eo-v6.request.template.json)、[建图运行](../../examples/automation/build-material-eo-v6.request.template.json)：填写已有有效缓存路径，工程改变后重新查询 SHA。
- [反算 → 采纳 → 改正向 → 重算](../../examples/automation/adopt-material-eo-v6.request.template.json)：引用前序步骤的 document / plan revision。

```sh
target/debug/radishflow-studio --headless query examples/automation/query-material-eo-v6.request.json
target/debug/radishflow-studio --headless execute examples/automation/adopt-material-eo-v6.request.template.json
```
