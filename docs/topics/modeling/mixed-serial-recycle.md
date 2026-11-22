# QS4 M3 串联回流区算例与核心方案

更新时间：2026-11-22

用途：定义串联回流的工程算例、独立基准、求解范围与验收条件。读者：solver、共享运行和工程验证维护者。不包含：同一区多撕裂、嵌套回流、热或相平衡方程、自动分区、持久化和消费者接入。

**核心、共享运行接收、v4 工程保存与 CLI v15 已完成本机验收。** Studio 接入安排在下一批。 [QS4 混合分区](mixed-partition-execution.md)

## 工程问题与连接

上游循环气的部分分出量可作下游补料。工程要区分上游放空、跨区转供和下游两股产品，并核对调节上游分流时下游实际收到的量和组成。此公开合成算例只考察固定温压物料守恒。两回流区固定为 300 K、650,000 Pa，组分为甲烷和乙烷，采用已有 Feed、Mixer 和非选择性 Splitter。A 新鲜料 `s-fa` 为 `(3, 2)` mol/s，B 补料 `s-fb` 为 `(1, 4)` mol/s。A 主分流器和回流分流器 `outlet_a` 比例分别为 `1/2`、`1/3`，B 分别为 `2/3`、`3/4`。此例中的规格与数据由解析物料衡算定义。

| 块 | 工程成员与物流 |
| --- | --- |
| Local `feed-a`、`feed-b` | 分别输出 `s-fa → mixer-a.inlet_a`、`s-fb → mixer-makeup-b.inlet_b` |
| EO `equations-a` | `mixer-a.outlet → splitter-primary-a.inlet` 使用 `s-ma`。`outlet_a` 经 `s-sa` 到 A 回流分流器，`outlet_b` 为终端放空 `s-pa` |
| Local `splitter-recycle-a` | `outlet_a` 经撕裂 `s-ra → mixer-a.inlet_b`。`outlet_b` 经 `s-ab → mixer-makeup-b.inlet_a` 转供下游 |
| Local `mixer-makeup-b` | 合并 `s-ab`、`s-fb`，经 `s-bf → mixer-b.inlet_a` 供给 B 区 |
| EO `equations-b` | `mixer-b.outlet → splitter-primary-b.inlet` 使用 `s-mb`。`outlet_a` 经 `s-sb` 到 B 回流分流器，`outlet_b` 为终端产品 `s-pb` |
| Local `splitter-recycle-b` | `outlet_a` 经撕裂 `s-rb → mixer-b.inlet_b`，`outlet_b` 为终端产品 `s-qb` |

九个单元归入七个执行块。每个回流区由一个 EO 块和一个 Local Splitter 块组成，A 到 B 单向连通。每个 EO 块有 12 个组分流量未知量和 12 条方程，包含 2 条 Mixer 衡算、4 条 Splitter 分流、2 条内部连接及 4 条外部边界方程。两区与转供 Mixer 的顺序由实际连接决定。

## 独立基准与现有边界

令 `f` 为区域外部输入分量，`p`、`q` 分别为该区主分流和回流分流比例。混合器入口总分量为 `m = f + r`，回流 `r = pq·m`，所以 `m = f/(1−pq)`。主分流出口为 `p·m` 和 `(1−p)·m`，回流分流器的非回流出口为 `(1−q)p·m`。先求 A，再用 `f_B = s-ab + s-fb` 独立求 B。两区反馈增益为 `1/6` 和 `1/2`。

| 流股 | 甲烷、乙烷分量（mol/s） |
| --- | --- |
| `s-fa`、`s-fb` | `(3, 2)`、`(1, 4)` |
| `s-ma`、`s-sa` | `(18/5, 12/5)`、`(9/5, 6/5)` |
| `s-ra`、`s-pa`、`s-ab` | `(3/5, 2/5)`、`(9/5, 6/5)`、`(6/5, 4/5)` |
| `s-bf`、`s-mb`、`s-sb` | `(11/5, 24/5)`、`(22/5, 48/5)`、`(44/15, 32/5)` |
| `s-rb` | `(11/5, 24/5)` |
| `s-pb`、`s-qb` | `(22/15, 16/5)`、`(11/15, 8/5)` |

逐组分独立核对 `s-fa − s-pa − s-ab = 0`、`s-bf − s-pb − s-qb = 0` 和 `s-fa + s-fb − s-pa − s-pb − s-qb = 0`。终端合计为 `(4, 6)` mol/s，总量 10 mol/s。两股撕裂和跨区转供均为内部物流。

补两组区分工况。将 B 补料改为 `(4, 1)` mol/s，A 全部流股不变，B 回流变为 `(26/5, 9/5)` mol/s。将 A 回流分流比改为 `2/3`，B 补料沿用主例，A 回流变为 `(3/2, 1)`，转供变为 `(3/4, 1/2)`，B 回流变为 `(7/4, 9/2)` mol/s。两组均以相同解析式逐股重新生成预期值。

[公开 v4 工程](../../../examples/flowsheets/fixed-tp-serial-recycle-eo-reference.rfproj.json)保存当前支持的 46×46 整图正向 EO 计划。CLI v6 三组实际运行的 13 股结果与分数基准一致。M1 同分区计划拒绝跨块环。M2 只切 `s-ra` 后仍有 B 环，只切 `s-rb` 则触发汇端 `inlet_a` 必须直连 Local Feed 的门禁。v9 工程读取拒绝在 M2 计划中声明两条撕裂。分数解是验收依据，整图 EO 是现有产品的交叉对照。 [本机回执记录](../../devlogs/2026-11/2026-W47.md#2026-11-22-qs4-m3-串联回流算例与方案核对)

### 重放整图对照

复用 [CLI v6 查询与执行契约](../../reference/headless-cli.md#qs3-cli-v62026-10-31已实现)与已有 `binary-hydrocarbon-lite-v1` 测试包缓存。将查询及运行模板的 `project` 改为上述工程，先查询再用其 `document_id`、`file_sha256` 填写 `expected_project`，使用查询返回的输入及计划修订。`run_flow` 后核对 `results(flow)` 的 13 股与来源。两组区分工况在副本中分别只编辑 `s-fb` 的进料组成和 `splitter-recycle-a` 的固定比例，修改后重新查询身份与指纹。

## 核心执行范围

1. **装配。** 按真实工程流股及源／汇端口绑定两条显式撕裂。以未切图的块强连通分量确定回流区，每个循环分量只接受一条连接 Local Splitter `outlet_a` 到 EO Mixer `inlet_b` 的撕裂。校验端口、归属、两区之间的单向依赖和切后 DAG，能力域限定为两组分正向物料及上述两区结构。正分流且小于 1 的固定比例保证两个区都有明确的非回流出口。无有效独立撕裂、两条均落在同一区、区外误切、切后仍成环、跨区反馈等在装配时定位拒绝。
2. **区域输入。** A 的 `inlet_a` 使用本次真实 Feed 输出。B 的 `inlet_a` 使用 `mixer-makeup-b` 的实际 `s-bf`，该 Mixer 必须收到 A 区独立复核后生产的 `s-ab` 以及本次补料。输入进首轮前冻结为正、有限、同温压、带生产者身份的本次边界。默认零试值继承同一区输入的已求组成，显式初值必须完整覆盖登记组分且非负有限。B 的资格来自已复核的上游和真实局部计算，工程模板及 A 的候选轮无权供料。
3. **计算与复核。** 每区独立复用 M2 的真实块求值和直接代入，每区候选预算默认 500、范围 `1..=500`，通过后另计一次独立复核。A 完成后转供 Mixer 运行一次，再启动 B。B 迭代期间不重算 A、补料或转供 Mixer。保留两区分别记录的末轮实际假设／返回值、闭合差与候选／复核次数。发布流股取生产者输出，汇端实际使用的试值在独立证据内保留。
4. **尺度与发布。** 各区撕裂、区内普通边界和区域衡算用该区冻结的外部输入总量缩放，主例 A、B 分别约 5、7 mol/s。各 EO 子问题仍以单次实际块输入装配自己的尺度。区外供料、转供边和整图按外部 Feed 合计 10 mol/s 缩放。各区、普通边界、单元和整图的逐组分绝对／缩放容限分别为 `1e-10 mol/s`、`1e-10`，整图总量残差 `≤ 2e-10 mol/s`。逐股对基准的分量误差 `≤ 1e-9 mol/s`、摩尔分率误差 `≤ 1e-10`，温压偏差 `≤ 1e-9 K`、`≤ 1e-6 Pa`。全部通过后才可一次发布 `Current`。失败保留本次诊断和已有复核证据，通过共享任务所有者处理取消、失效与迟到回执。

执行方法 `radishflow.fixed-tp-mixed-serial-recycle/1` 进入共享运行所有者。共享结果来源带实际执行方法和按稳定身份规范化的配置摘要，覆盖两条撕裂、两区初值、预算和尺度规则。工程保存通过显式 M3 应用入口。 [v4 工程保存](#v4-工程保存评审)

## 验收表

| 场景 | 通过条件 |
| --- | --- |
| 两组初值 | 主例分别用两区零初值和 A `(3, 1)`、B `(2, 3)` mol/s。13 股、两区与整图均满足上述门槛。分数参考分别需 A 14／15、B 36／35 次候选，每区另做一次复核。对照计数用于核对调度及报告，实际调用据实记录 |
| 扰动与顺序 | 跑 B 补料组成切换和 A 回流分流比调整。按稳定 ID 逐股对照分数期望，乱序传入块及撕裂的列表仍得到同一连接约束和 A→B 依赖。断言转供 Mixer 的真实输入来自 A 复核、B 看到该 Mixer 的生产者值，A 不随 B 候选重复执行 |
| 绑定及域拒绝 | 分别覆盖少一条、重复、错端口、切非循环边、新增反向跨区边、失去正的已求输入、组分或固定温压无效。前五类在装配阶段拒绝，运行才可确定的边界问题在相应阶段拒绝，诊断定位区域／流股／端口 |
| 失败传播 | 两区分别给候选预算 1，分别注入 A 最终复核漂移和 B 下游边界／全局不守恒。A 失败时 B 未执行，B 失败时可审阅 A 证据，正式结果均无当前数值。再验同一任务因所有局部回流合格而全局复核失败的情况 |
| 接收与回归 | 在 A 复核后、跨区交接和 B 复核中用可控门槛覆盖取消；旧任务迟到、输入／计划 ABA、切换或重开均不能发布新结果。重跑只接收新身份。并跑现有 M1 和两初值 M2，保留它们的原守恒、接收和消费者行为 |

核心以两套初值和两项扰动逐股验收。故障注入分别证明 A 撕裂复核失败、A 区域不守恒和跨区边界失效都会挡住 B 区。两区都闭合仍可能因累计漂移被整图门禁拒绝。共享运行用真实 M3 工作线程验证取消先于接收时拒收已计算成功和失败的载荷，回收后只接收新任务。 [实施与验证记录](../../devlogs/2026-11/2026-W47.md#2026-11-22-qs4-m3-核心与共享运行验收)

## v4 工程保存评审

**已批准并完成本机验收。** 本批交付 `rf-store` 的 v4 codec 与目标保护、应用层正式计划暂存保存和受支持重开，以及共享显式运行验收。通过 `prepare_mixed_serial_recycle_save` 准备写入，通过 `read_studio_project_with_support` 显式加载。默认 reader／writer 及既有消费者继续使用各自的能力上限。CLI 与 Studio 原生路径接入另审。

### 文件契约

外层仍为 `schemaVersion: 4`。M3 采用独立方法 `radishflow.fixed-tp-mixed-serial-recycle` v1 和必需能力 `radishflow.solver.fixed-tp-mixed-serial-recycle` v1，经显式 `FlowPlanSupport::MixedSerialRecycle` 入口读写。writer 由内容派生能力清单，reader 核对声明与实际计划。M1、M2 和正向 EO 保留各自的字段与版本，设备和对象身份能力按各自契约判定。

| `simulation.steadyStatePlan` 字段 | 保存约定 |
| --- | --- |
| `method`、`acceptanceContractVersion` | 两者均为 v1。冻结本批已验证的区域闭合、全局守恒与复核规则 |
| `fixedBoundary`、`blocks` | 保存正有限的 K、Pa 及完整 Local／EO 块分配。复用块 ID、成员 ID、EO 方程契约 v1 和现有 EO 块尺度。分流比例继续由工程 Splitter 参数提供 |
| `recycleRegions` | 必填两项。每项保存 `tear` 和该区独立的 `maxIterationEvaluations`，预算必填整数 `1…500`，最终独立复核另计。writer 按撕裂 `streamId` 排序。执行顺序由工程连接与装配决定，A／B 标签及数组位置无需持久化 |
| `recycleRegions[].tear` | 保存 `streamId` 及 `source`、`sink` 的单元 ID 和端口。`initialization` 为 `zero_component_flows` 或带 `valuesMolS` 的 `component_flows`。显式值为有限非负 SI 分量，组分 ID 唯一并按 ID 排序，运行时必须完整覆盖登记组分 |
| `scaling` | 固定 `{ "kind": "evaluated_region_inlet_and_global_feed_total" }`。各区按本次已求区域入口总流量确定尺度，整图按实际外部 Feed 总量确定尺度。各 EO 块仍按既有本次块输入规则装配 |

以下摘录计划的身份、边界、尺度和两区字段。完整计划中的 `blocks` 采用[已验证的七块分配](#工程问题与连接)。两区使用不同预算以便检查往返是否串位。

```json
{
  "method": { "id": "radishflow.fixed-tp-mixed-serial-recycle", "version": 1 },
  "acceptanceContractVersion": 1,
  "fixedBoundary": { "temperatureK": 300.0, "pressurePa": 650000.0 },
  "scaling": { "kind": "evaluated_region_inlet_and_global_feed_total" },
  "recycleRegions": [
    {
      "tear": {
        "streamId": "s-ra",
        "source": { "unitId": "splitter-recycle-a", "port": "outlet_a" },
        "sink": { "unitId": "mixer-a", "port": "inlet_b" },
        "initialization": { "kind": "zero_component_flows" }
      },
      "maxIterationEvaluations": 100
    },
    {
      "tear": {
        "streamId": "s-rb",
        "source": { "unitId": "splitter-recycle-b", "port": "outlet_a" },
        "sink": { "unitId": "mixer-b", "port": "inlet_b" },
        "initialization": {
          "kind": "component_flows",
          "valuesMolS": [
            { "componentId": "ethane", "value": 3.0 },
            { "componentId": "methane", "value": 2.0 }
          ]
        }
      },
      "maxIterationEvaluations": 240
    }
  ]
}
```

已知 v1 严格拒绝缺项、显式 `null`、未知键、重复 JSON 键、重复流股或源／汇端口绑定、非法预算、非有限数及负初值。M2 的计划级预算和 `tears` 字段仍由 M2 自己消费。图上对象被删除、连接改接、两个绑定落入同一区、切后仍有环或显式初值与现有组分不匹配时，保留正式配置和原绑定，允许可编辑重开并显示未就绪。共享运行在装载物性或启动求解前定位拒绝。仍被计划引用的 ID 继续保留给原对象，恢复或重绑需走正式输入和计划动作。 [v4 计划与身份保护](recycle-project-and-consumers.md#qs1-c01项目-v4-与能力保护)

工程文件只保存已应用计划及原有工程输入、呈现、受支持工程区和可选身份／关注清单。由运行计算的实际尺度、试值、回流和转供结果、诊断轨迹、快照、运行身份、计划代次、草稿和撤销栈留在会话内。沿用已验证的规范化运行配置摘要，独立于 JSON 字节指纹。改变任一区预算、初值、绑定或实际块设置会改变配置摘要，纯排列变化不会改变。

### 兼容与写入保护

1. **读取。** 能读 v4 但未启用 M3 的 reader 根据必需能力或方法探测进入整份只读。保留原 UTF-8 字节和安全元数据，编辑、运行、序列化、原路径保存及由只读对象另存一律拒绝。移除 M3 标记时，旧 reader 仍凭方法保护，新 reader 对已知 M3 v1 拒绝能力与内容不匹配。未知正版本的必需能力、方法、验收契约或 EO 块契约继续整份只读。只识别 v1–v3 的 reader 按原入口拒绝 v4。 [v4 能力探测](../../../crates/rf-store/src/project_v4/mod.rs)
2. **准备与确认。** 应用层以显式 M3 支持建立同一份待写工程和暂存保存记录。记录冻结来源会话、已应用计划代次、保存基线、正式项目内容、待声明能力和目标存在状态／原字节 SHA-256。首次加入 M3 的来源需要兼容确认，已有 v4 和另存新路径同样适用。覆盖低能力 v4 目标须确认新增能力。Save As 覆盖已有目标继续复用覆盖确认。已有 M3 同路径普通保存按既有规则处理。
3. **旧 writer 与降回其他方法。** 旧 writer 即使拿到调用方覆盖许可，也拒绝覆盖 M3 目标。支持 M3 的底层 writer 同样拒绝用其他方法覆盖已有 M3 目标，普通保存和另存覆盖共用此守卫。由 M3 切回 EO、M1 或 M2 时，可显式另存到新路径。该副本按实际方法派生能力，原 M3 文件逐字节保留。读仅元数据的入口、未知能力目标和已知坏格式目标均没有绕过写入保护的路径。
4. **提交与恢复。** 确认仅适用于准备记录捕获的内容。关闭重开、已应用计划改回、正式输入或目标在准备后变化，均须重新准备。新目标在写入前被占用时拒绝覆盖。复用 v4 指纹重验和同目录暂存写入，覆盖仍按既有平台替换／恢复契约处理。取消及失败均保留工作区路径、已保存配置基线和草稿。通常的写入失败须保留原字节，Windows 恢复失败保留可定位的原字节备份并报告错误。成功后才更新路径、版本、已保存计划及能力基线。并发验收到准备后目标变化和新路径抢占，沿用现有文件系统提交语义。 [应用保存事务](../../../apps/radishflow-studio/src/document_lifecycle_driver/plan_save.rs) · [v4 目标检查](../../../crates/rf-store/src/project_v4/file_io.rs) · [暂存写入](../../../crates/rf-store/src/staged_file.rs)

### 重开与结果资格

受支持的重开恢复正式输入、两区配置和保存基线，建立新的会话、计划代次和运行身份。两区配置摘要与保存前一致，从 `Missing` 起步。打开、保存、查询就绪性、编辑和切换 `Active` 均不自行请求 M3 求解。在同一会话保存已具备资格的 `Current` 时，结果仍留在该会话。重开后只响应共享运行所有者的显式运行命令。静态就绪性检查及 Ready 资格复核通过后才执行，A 复核后才向 B 交付。只有两区闭合、两区守恒和整图守恒均通过，且新任务回执仍具备资格时才发布 `Current`。旧会话迟到回执、取消后迟到回执及新会话未显式重跑期间均保持结果资格不变。 [共享调度门禁](../../../apps/radishflow-studio/src/workspace_solve_service.rs) · [结果资格](../../reference/solve-snapshot-results.md#当前资格来源与动作守卫)

### 保存验收表

| 场景 | 通过条件 |
| --- | --- |
| 正式往返与规范排序 | 以公开流程图创建 M3，覆盖两区全零、全显式及示例混合初值。使用不同预算验证分别绑定。将上游撕裂 ID 排在下游之后并打乱块、成员和组分条目，读写后规范文件字节和配置摘要相同，执行仍先 A 后 B。核对工程输入、单位、可选身份和关注清单往返 |
| 严格格式和未来能力 | 在真实保存产物上注入缺项、`null`、重复键／绑定、负数、非法预算、版本 0 及缺失／不匹配的能力标记，已支持 M3 v1 入口返回格式错误。正的未知版本、未知必需能力或块方法走整份只读 |
| 旧读写器 | 用冻结 `946a99fe` 的 Rust reader／writer 读新 writer 的产物，同时覆盖标记完备及移除 M3 标记的两个文件。比较原始字节和只读范围。序列化、Save As 及覆盖均拒绝，源／目标字节与不存在的另存路径状态保持。既有 v7／v9／v14 只读查询对照及 v9 execute 早期阻断继续通过 |
| 首存与覆盖 | 从 v4 EO 或 M2 来源应用 M3，分别验收原路径保存、新路径另存、覆盖兼容目标、取消及确认。取消后准备记录不能用于其他内容。已有 M3 同路径普通保存通过。未知能力、坏格式和任何已有 M3 目标的降级覆盖均拒绝。降回 M2 后另存新路径可由冻结构建打开，M3 原件原字节不变 |
| 变化与失败恢复 | 在准备后分别注入输入／计划改回、换会话、替换或删除目标、外部抢占新路径、暂存写入失败。对照源／目标原字节、保存基线、路径和草稿。重新准备后可重试，旧记录直接提交失败。覆盖恢复沿用既有 writer 契约 |
| 未就绪工程 | 保存含失效端口或组分表变更的已应用计划，重开保留绑定和当前工程，诊断定位流股、端口及组分且仍无当前数值。同名新对象不会取得旧绑定 |
| 有结果时保存与重开 | 原会话先有 `Current`，正式保存只写入输入。开新会话后确认 `Missing`、空结果历史、配置摘要一致。模拟 `Active` 或输入更改等自动触发，均不取得 worker。用门控让原会话成功／失败回执迟到，仍为 `Missing` |
| 显式重跑和回归 | 从同一真实保存产物发起共享手动运行，验证新身份、A→B 交接、13 股基准、两区及整图守恒，通过后发布 `Current`。将 A 预算改为 1 的另一个产物只返回失败诊断，B 零调用。补取消拒收，跑 M1／M2 的既有读写、结果和协议回归以及锁定离线全仓检查 |

实际保存产物及移除标记的副本在冻结 `946a99fe` 的 Rust 读写器中均整份只读，保留原字节并拒绝序列化、另存及覆盖。另存的 M2 可由同一旧读写器打开。实际 M3 文件重开后配置摘要一致且保持 `Missing`，显式运行通过两区及整图复核后发布新 `Current`。 [保存验收记录](../../devlogs/2026-11/2026-W47.md#2026-11-22-qs4-m3-v4-工程保存实现与验收)

## CLI v15 接入评审

**已批准并完成本机验收。** `schema_version: 15` 使用显式 M3 读者加载已保存的 v4 工程，经共享正式运行执行，通过 stdout 返回结果。工程外层、M3 方法／能力及验收契约沿用保存批的定义。已有各版本继续按各自的支持范围打开文件。 [现有协议分派](../../../apps/radishflow-studio/src/headless/v3/mod.rs) · [M2 v9 接入](mixed-partition-execution.md#m2-cli-首片评审)

### 请求与执行顺序

| 入口 | v15 词汇 |
| --- | --- |
| `--headless query` | `schema_version`、`project`、`queries`。允许 `queries: []` 查看工程头。`results` 只支持 `scope: flow_plan` 和 `scope: flow`。无需物性包或授权索引 |
| `--headless execute` | 同样带 `schema_version`、`project`、`queries`，另外必填 `expected_project`、`flow_resources` 和**恰好一个** `run_flow` 步骤。`expected_project` 取查询得到的 `document_id` 和原始 `file_sha256`。步骤必填 `expected_revision`、`expected_plan_revision`，均为无符号整数 |

一个执行请求产生一个明确的运行步骤来源。再次计算使用独立 `execute`。查询与执行沿用 `project`／`steps`／`queries`／`error` 响应外层。请求记录使用命名对象，位置数组、未知键、重复键、空白或重复查询 ID、空步骤 ID、空路径与不支持动作在请求校验时拒绝。各路径相对请求文件目录，`flow_resources` 使用既有 `cache_root`、`auth_cache_index` 和 `package` 语法，支持 `preferred` 或明确指定的已授权本地包。范围限定为已保存工程的发现、显式运行和结果复核，建模、编辑计划、写工程、后台取消和稳定变量读取保留在各自入口。

执行顺序为请求校验、同次读取工程与原字节 SHA-256、核对文档 ID 与指纹、整份只读门禁、M3 方法判定、两个修订、共享图就绪性、加载本地资源、手动运行。执行通过现有 `run_flow` 动作、结果发布与接收资格。就绪性命中真实静态诊断时在资源读取前阻断，并可在当前请求的末尾查询看到 `not_ready` 和 `missing`。保护中的工程只返回安全工程头与错误。已识别为 v15 的响应使用 `schema_version: 15`，版本识别前的错误沿用公共解析入口。 [共享读入与指纹](../../../apps/radishflow-studio/src/headless/v3/mod.rs) · [正式运行回执](../../../apps/radishflow-studio/src/headless/v3/runs.rs)

### 旧版边界

| 场景 | 读取与执行 |
| --- | --- |
| CLI v1／v2 `run` 与 v3 读取 M3 v4 | 现有外层版本门禁拒绝，不建立工作区。实际产物的 v1／v2 `run` 在 project 阶段退出 2，v3 query 返回 `unsupported_capability`／退出 2 |
| CLI v4—v14 读取已保存 M3 | 各版本支持的查询遵循原只读协议。工程头 `read_only: true`，指纹对应原文件。M3 能力标记缺失时仍由未知方法门禁保护。只读查询仅提供各协议允许的安全元数据。可执行版本的执行请求在项目只读门禁阻断，v8 沿用只提供稳定变量查询的范围 |
| v15 读取 M3 v4 | 支持 M3 的正式计划、已知对象身份和关注清单。须满足独立设备能力门禁。未知的必需能力、方法或正契约版本返回整份只读。`query` 仅提供安全元数据和原因，`execute` 返回 `read_only_project`／退出 3 |
| v15 读取其他已知可编辑计划 | 返回 `unsupported_capability`／退出 2，工程头按文件实际状态显示。已知 M3 v1 的字段坏损或缺少／错配 M3 能力声明返回 `invalid_project`／退出 2 |
| 实施前冻结构建收到 v15 | 请求阶段返回 `unsupported_version`／退出 2，工程尚未读取 |

### 查询和结果资格

- **计划。** `results(flow_plan)` 返回 `plan_revision`、`configuration_sha256`、`ready`、`not_ready` 与 `configuration`。配置含 M3 方法 `radishflow.fixed-tp-mixed-serial-recycle/1`、必需能力及验收契约、固定温压、块成员、尺度规则和 `recycle_regions`。计划列表按撕裂流股 ID 规范化。每区含独立预算、源／汇端口及初值。显式初值的列表项为 `component_id` 与 `value_mol_s`，单位和字段采用现有 CLI 规则。`not_ready` 复用共享诊断中的代码及单元、流股、端口和组分身份。配置摘要来自规范化运行计划，工程字节指纹来自入口文件。
- **打开与重复查询。** 每次独立 `query` 都建立新文件会话。已就绪的计划可以伴随 `results(flow)` 的 `{ "status": "missing", "reason": "no_result" }`，此时无 `fields`、`streams`、`origin.execution` 或计算证据。一次 `execute` 成功后，再启动 `query` 仍是 `missing`。
- **成功。** 只有显式 `run_flow` 获得已接收的成功回执，`steps[0].result` 与本请求末尾的 `results(flow)` 才均返回 `current`。`origin.execution` 带同一请求内 `step`、`scope: request`、实际 solver／物性包来源，以及该次执行方法、必需能力、验收版本和配置摘要。投影使用 `execution_kind: mixed_serial_recycle`，产出真实生产者 `streams`、`blocks`、`boundaries`、`global_material`、`costs` 和按实际依赖排序的 `recycle_regions`。先有 A 区独立复核及衡算，再传递实际产出给 B 区。两区和整图均通过后才可接收整个结果。
- **区域和边界。** 结果区域带 `equation_block_id`、`stream_id`、`inlet_stream_id`、候选预算、实际 `component_scales_mol_s`、`residual_trace`、`last_sweep` 和 `region_material`。候选和独立复核沿用 `iteration`／`verification` 标识，轨迹包含 `evaluation`、`max_absolute_mol_s` 和 `max_scaled`。最终撕裂汇端 `assumed` 与源端 `returned` 分别保留，逐组分残差为返回减试值。普通边界记录真实生产／消费两端。区域衡算的 `inlet_components_mol_s` 和 `outlet_components_mol_s` 包含本区外部物料，B 区的入口含转供。全局 `fresh_components_mol_s` 仅计两股外部 Feed，`product_components_mol_s` 仅计三股终端流，衡算残差为流出减流入。
- **失败。** 文档 ID、指纹或修订冲突各返回 `different_document`、`project_conflict` 或 `record_conflict`，退出 2。已知未就绪配置返回 `action_rejected`／退出 3，保持 `missing`。资源或运行准备失败沿用 `run_start_failed`／退出 4，已接收的求解失败使用 `run_failed`／退出 4。预算耗尽等数值失败返回 `failed`，已完成块、真实区域试值、残差及局部调用量只出现在 `diagnostic_evidence`，连同失败阶段和定位身份。只有完成了区域衡算的报告带该证据。成功投影须为有限数，诊断非有限值沿用 `non_finite` 标记。 [混合结果证据](../../../apps/radishflow-studio/src/mixed_result_projection.rs) · [CLI 基础结果资格](../../../apps/radishflow-studio/src/headless/v3/results.rs)

### 实际保存产物与验收表

基准使用保存批由应用写入的 `saved-m3.rfproj.json`，工程 ID 为 `example-qs4-serial-recycle-eo-reference`，外层 v4，输入修订 0。A 区 `s-ra` 保存零初值与预算 100，B 区 `s-rb` 保存 `(2, 3)` mol/s 的甲烷／乙烷初值与预算 240。实际文件 SHA-256 为 `d8b2c39a179205a607dc6645b8cc30f47dc5a7c0017bbec85ae865c119444808`，运行配置摘要为 `ccd5307a18c46753616ffb42116bf0eb7b8dfcf41668e5100f024dadc07dd4e7`。应用层以该文件的逐字节相同副本重开得到 `Missing`，显式运行后得到 13 股 `Current`，两区最大绝对闭合残差实测 `6.38e-12`、`2.62e-11 mol/s`。v15 的实际回执与该独立对照一致。 [已有保存与运行验收](../../devlogs/2026-11/2026-W47.md#2026-11-22-qs4-m3-v4-工程保存实现与验收) · [旧协议预检](../../devlogs/2026-11/2026-W47.md#2026-11-22-qs4-m3-cli-v15-评审预检)

以下 `query` 请求读取同目录的实际保存文件。

```json
{
  "schema_version": 15,
  "project": "saved-m3.rfproj.json",
  "queries": [
    { "id": "plan", "query": { "kind": "results", "scope": "flow_plan" } },
    { "id": "flow", "query": { "kind": "results", "scope": "flow" } }
  ]
}
```

以下 `execute` 请求使用同一文件的 ID、指纹和初始修订。本地缓存路径需对应已授权测试包。

```json
{
  "schema_version": 15,
  "project": "saved-m3.rfproj.json",
  "expected_project": {
    "document_id": "example-qs4-serial-recycle-eo-reference",
    "file_sha256": "d8b2c39a179205a607dc6645b8cc30f47dc5a7c0017bbec85ae865c119444808"
  },
  "flow_resources": {
    "cache_root": "./property-package-cache",
    "auth_cache_index": "./auth-cache-index.json",
    "package": { "kind": "preferred" }
  },
  "steps": [
    { "id": "solve", "action": { "kind": "run_flow", "expected_revision": 0, "expected_plan_revision": 0 } }
  ],
  "queries": [
    { "id": "plan", "query": { "kind": "results", "scope": "flow_plan" } },
    { "id": "flow", "query": { "kind": "results", "scope": "flow" } }
  ]
}
```

| 场景 | 通过条件 |
| --- | --- |
| 文件发现与 `Missing` | 对实际产物执行 v15 真进程 `query`，同时取 `flow_plan` 和 `flow`。工程头 v4、同一原始指纹且 `read_only: false`。配置摘要与应用层相同，两条撕裂、100／240 次预算和混合初值准确。`ready: true` 与 `missing` 同时成立。没有包缓存的环境中仍通过 |
| 显式重跑发布 | 用查询的 ID、指纹与两个修订提交单个步骤。步骤回执和最终查询均 `current`，`origin.execution.step` 相同，实际包来源和配置摘要相同。按独立基准核对 13 股、`s-ab` 与 `s-bf` 两端实际值、A→B 顺序、两区各一次最终复核。撕裂及区域和全局的逐组分绝对误差门槛为 `1e-10 mol/s`，缩放门槛为 `1e-10`，全局总量残差 `≤ 2e-10 mol/s`。终端产品合计 `(4, 6)` mol/s，逐股分量误差 `≤ 1e-9 mol/s`。再次开进程查询仍为 `missing` |
| 失败结果与诊断 | 由应用 writer 各另存一份 A 或 B 预算为 1 的 M3。首次查询均为 `missing`，显式执行为退出 4／`failed`。A 失败时 B 补料、转供和 B 块零调用。B 失败时能查看诊断里的已复核 A，但正式结果无 `streams` 或 `fields`。用核心既有故障用例加投影测试核对闭合通过而整图守恒失败、诊断的非有限标记以及拒收不发布 `current` |
| 早期拒绝与就绪检查 | 用真实格式的失效撕裂文件复核 `flow_plan.ready: false` 与同一定位诊断。用不存在的授权索引验证错文档 ID、改单字节、两个修订冲突和未就绪均先于资源读取。合法文件和修订搭配缺失资源则进入 `run_start_failed`，无当前数值 |
| 请求严格性与方法边界 | 对真实产物测试未知／重复键、位置数组替代对象、空或多步执行、禁止的动作和查询词汇。v15 读取实际 M2 另存产物时拒绝错方法。对 M3 移除标记后，支持 v15 的读者返回 `invalid_project`，v4—v14 仍走只读。未知正能力版本只给安全元数据 |
| 旧协议和磁盘 | 固定 `2212ad64` 与实施后构建分别重放实际 M3 及去标记副本的 v4—v14 查询，以及所有支持执行的旧版本的早期拒绝。v8 在请求版本门禁拒绝 `execute`。用实际 M2 副本跑 v9 查询及代表执行，保留原协议形状。对成功、失败、启动拒绝和连续独立查询逐字节核对工程、缓存、授权索引、偏好和布局，目录无新增结果文件。完成真实 CLI 进程定向测试及锁定离线全仓检查 |

macOS 本机重放通过。两份 M3 产物在冻结构建和当前构建的 v4–v14 查询均整份只读，每个构建覆盖 22 次查询、20 次项目级执行拒绝和两次 v8 请求级拒绝。v15 依次得到 `missing → current → missing`。A、B 区分别进行 14、35 次候选求值并各完成一次独立复核。13 股组分流量的最大解析偏差为 `5.58e-11 mol/s`，全局总流量残差为 `−3.97e-11 mol/s`。应用 writer 保存的 A／B 预算不足用例分别验证上游失败不执行下游，以及下游失败时已复核 A 仅见于诊断。 [实际回执与运行记录](../../devlogs/2026-11/2026-W47.md#2026-11-22-qs4-m3-cli-v15-实现与验收)
