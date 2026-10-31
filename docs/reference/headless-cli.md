# 本地无界面运行接口

更新时间：2026-10-31

用途：说明 Studio 旧 v1 / v2 与 AF4c v3 命令、JSON 请求 / 响应和错误语义。

读者：使用本地脚本执行参数化工况的用户及适配器开发者。

不包含：公共网络 API、COM、录制脚本、后台任务与项目保存。

## 版本与兼容边界

2026-10-26 已确认 [AF4c 新版 CLI 契约](../topics/variable-and-object-browser.md#af4c-新版-cli-契约2026-10-26已确认)，AF4c-1—4 已完成。新版 query / execute 的查询、正式输入 / 确认、同步运行和采纳已接通，并通过真实进程与正式仓库基线。以下旧协议契约继续有效；新版参考见文末。

B3-5 已通过 v2 请求接通受控创建 / 连接和步骤身份引用，v1 请求与响应字段保持兼容。2026-09-16 确认的 [单位系统](../topics/units-and-quantity-system.md) 和 [建模辅助](../topics/modeling-assistance-and-specifications.md) 后续复用本接口的身份、修订、命令与诊断；本文 v1 的 SI 数值、路径与错误契约不变。U1 已让响应的 unit 标签从公共目录派生，v1 / v2 仍只读写 SI，不接收额外单位字段。未来带单位请求须显式版本化或定义兼容扩展，不受 GUI 显示偏好影响。

U2-I1 已让本接口通过共享存储层读取项目文件 v1 / v2；工程集呈现配置不改变 CLI 的 SI 数值与标签，非法呈现配置会使装载失败，不能丢弃配置后继续运行。项目外层版本与 CLI 请求版本独立，文件兼容规则见 [单位存储接口](../topics/units/unit-sets-and-persistence.md#u2-存储接口)。读取与运行仍不改写原工程，尚未提供 U3 的输出单位选择。

Studio GUI 已支持已知设备项目 v3，旧 CLI 的 `inspect`、v1 `run` 与 v2 workflow 仍在任何建模 / 求解前拒绝所有外层 v3 文件，包括工程区缺省或为空的文件。设备计算、保存及 CSV / JSON 导出不属于旧 CLI 协议；不能删除工程区后伪装为旧版继续运行。详见 [v3 生命周期](../topics/project-lifecycle-storage.md#完整设备保存与最小-studio-生命周期2026-10-15)。

## 旧协议命令

使用已构建的 `radishflow-studio` 可执行文件：

```sh
radishflow-studio --headless inspect project.rfproj.json
radishflow-studio --headless run request.json
radishflow-studio --headless --help
```

开发工作区可执行 `cargo build --locked -p radishflow-studio`，随后使用 `target/debug/radishflow-studio`（Windows 为 `.exe`）。无界面分支在 GUI 初始化前执行，不打开窗口、访问最近项目或写入偏好。当前仍构建和链接 Studio 的既有桌面依赖，不承诺图形依赖已从产物中移除。其他平台运行态按阶段集中验证。

每次命令创建独立内存工作区，加载已有 `*.rfproj.json`，结束时丢弃本次修改。标准输出是一个 UTF-8 JSON 响应，帮助命令除外；输出故障信息写入标准错误。保存响应可由调用方重定向到独立文件，不能把目标设为输入项目或缓存文件。编译日志属于 Cargo 输出，不属于响应协议。

## 查询后运行

1. `inspect` 获取 `document.id`、`document.revision` 与 `variables[].target`。查询不需要物性缓存，也不运行求解；工程文件不存储求解快照，结果变量可能是 `missing`。
2. 将实际身份、修订与变量 target 放入运行请求，指定所用本地缓存。示例：[Heater 330 K 请求](../../examples/automation/heater-330k.request.json)。示例中的 `cache_root` / `auth_cache_index` 需要改成已有缓存位置；它不携带凭据或缓存，不会自动生成演示物性。
3. 执行 `run`，检查退出码和 `status` 后读取变量。示例结果应为出口 330 K；输入模板流量仍为 0 mol/s，而当前求解流量为 5 mol/s。这验证软件调用链，不构成工业物理精度依据。

## 请求版本 1

字段使用 snake_case；除 `package_id` 外全部必填。未知字段、重复字段、未知版本或枚举、数值类型错误均拒绝。`writes` / `reads` 可为空数组。

| 字段 | 含义 |
| --- | --- |
| `schema_version` | 固定为整数 `1`，仅指本地 CLI DTO，与工程 schema 独立 |
| `project` | 已有工程文件路径 |
| `document_id` / `expected_revision` | 必须匹配磁盘加载的文档，防止请求作用于错误工程或已变化输入 |
| `cache_root` / `auth_cache_index` | 派生物性文件根目录和正式缓存索引文件；不猜测默认路径，不登录或下载 |
| `package_id` | 可省略或为 null，沿用 `Preferred`；字符串沿用正式显式包选择规则 |
| `writes` | 有序的 `{ "target": ..., "value": ... }`；数值为 JSON number，名称为 string，不隐式转换 |
| `reads` | 本次求解成功后要读取的变量 target 数组 |

请求中的相对文件路径全部相对于 **请求文件目录**；命令行参数本身相对于进程工作目录。空路径拒绝；绝对路径可用。索引内的资产相对路径沿用既有缓存验证规则。

### 变量 target

```json
{
  "object": { "unit": "heater-1" },
  "section": "inputs",
  "field": "outlet_temperature"
}
```

`object` 为 `{ "unit": "<UnitId>" }` 或 `{ "stream": "<StreamId>" }`，使用身份而非显示名称；`section` 为 `inputs` 或 `results`。请求文档身份限定其中所有 target；每次成功写入后的实际修订用于下一次统一写入与运行调用。

| field 形式 | 含义 |
| --- | --- |
| `"name"` | 对象名称 |
| `"outlet_temperature"` / `"outlet_pressure"` | 单元出口参数，K / Pa |
| `"temperature"` / `"pressure"` / `"molar_flow"` | 流股温度、压力、摩尔流量，K / Pa / mol/s |
| `{ "mole_fraction": "methane" }` | 指定组分摩尔分数，mol/mol |
| `{ "phase_fraction": "vapor" }` | 指定相摩尔分率，mol/mol |
| `{ "phase_mole_fraction": { "phase": "vapor", "component": "methane" } }` | 相内指定组分摩尔分数，mol/mol |
| `{ "phase_molar_enthalpy": "vapor" }` | 相摩尔焓，J/mol |

具体字段是否存在、可写以及数值约束由统一变量入口决定；相字段是结果，不保证所有相在每份快照中存在。优先复用查询得到的 target。结果分区不可写，不把结果作为输入模板。组成修改必须显式给足目标项，不自动补齐或归一化。

写入逐条执行；若第 N 条拒绝，响应保留此前在本次内存中生效的写入 receipt、修订及从 0 开始的 `error.index`，后续写入和求解不执行。所有磁盘输入保持不变；这不是跨请求持久化事务或录制回放格式。

## 请求版本 2：建模后运行

使用同一 `--headless run` 命令。完整示例：[建模请求](../../examples/automation/build-feed-heater-flash.request.json) 与 [已配置组分 / 物性的空流程](../../examples/flowsheets/binary-hydrocarbon-basis.rfproj.json)。先通过 `inspect` 核对工程身份和修订，并将缓存路径改为实际本地资产；不需要预先猜测新对象 ID。

v2 使用 `schema_version: 2`，保留 v1 的 project、身份 / 修订、缓存路径及 package_id 字段，以必填 `steps` 代替 `writes`；两者不能混用。`steps` 和 `reads` 均可为空。全部步骤按顺序成功后，执行一次同步求解，再读取 `reads`；不在步骤中嵌入运行、循环、条件分支或任意脚本。

每步为 `{ "id": "步骤名", "action": { ... } }`。步骤名必须非空白、唯一、大小写敏感，按原文匹配，不自动 trim。它只在本次请求内标识操作，不能代替工程稳定身份。

| action.kind | 字段 | 结果 |
| --- | --- | --- |
| `create_unit` | 必填 `unit_kind`：feed / heater / cooler / valve / mixer / flash_drum | 返回新 unit_id，身份由应用分配 |
| `connect` | 必填 source，选填 sink / stream | 返回实际 stream_id 和端点；只接受唯一实时规则候选 |
| `write` | 必填 target / value，字段和值类型沿用 v1 | 返回解析后的 target、changed；同值写入不推进修订 |

对象引用分别为 `{"id":"已有稳定ID"}` 或 `{"step":"此前成功步骤名"}`；两种不能混写。创建步骤只返回设备身份，连接步骤只返回流股身份，写入步骤不返回可引用对象。不存在 / 前向引用返回 `unknown_step_reference`；设备 / 流股错用或引用写入步骤返回 `reference_kind_mismatch`。更改名称不影响步骤绑定。

v2 target 的 object 使用 `{"unit":{"step":"heater"}}` 或 `{"stream":{"id":"stream-feed"}}` 等形式；section / field 沿用 v1。`reads` 也使用此格式；返回的变量 target 始终是已解析的稳定 ID，不保留临时步骤名。

```json
{
  "id": "feed_to_heater",
  "action": {
    "kind": "connect",
    "source": { "unit": { "step": "feed" }, "port": "outlet" },
    "sink": { "unit": { "step": "heater" }, "port": "inlet" },
    "stream": { "step": "feed_stream" }
  }
}
```

source / sink 为设备引用和精确端口名。sink 省略或 null 表示创建规则允许的未接下游出口，不会自动挑选目标设备；stream 省略或 null 时按端点匹配唯一候选，指定时还必须匹配流股身份，不能借此创建任意流股。连接前重新枚举候选，正式动作执行时再次校验；无候选返回 `connection_unavailable`，多个候选返回 `ambiguous_connection`。现有规则拒绝多上游歧义时，即使请求指定目标，也不能绕过。

示例先建立 Feed 的出口流股，再写入流量 / 组成并逐步连接 Heater 和 Flash。新流股初值沿用现有应用规则，工程参数仍需调用方明确写入和检查；示例演示软件闭环，不提供工业物性精度承诺。

### v2 回执与失败

- 成功解析的 v2 请求返回 `schema_version: 2` 和 `steps` 数组。每个成功回执含 id、从 0 开始的 index、实际 revision 及 effect；effect.kind 为 created / connected / written，携带上表的实际身份或写入结果。v2 的 `writes` 保持空数组，写入回执只在 steps 出现。
- 第 N 步失败时，`error.stage` 为 `step`，index 为 N；保留前 N 步的成功回执和最新内存修订，失败步骤不改文档，后续步骤及求解不执行。重复步骤名、空白步骤名分别返回 `duplicate_step_id` / `empty_step_id`。领域参数错误沿用正式错误码。
- 每步复用原有输入事务、修订、历史与结果旧化规则；整个 steps 不是全有或全无事务，返回成功回执也不表示已经保存到磁盘。
- 全部步骤成功而求解阻塞 / 失败时，保留 steps，沿用退出码 3 / 4 和运行诊断。读取失败沿用退出码 2，stage 为 read，index 定位 reads；不发布部分变量列表。
- 未知字段、重复字段、混用版本或类型错误在整个 JSON 解析时拒绝，任何步骤都不执行。请求读取 / 解析 / 未支持版本错误尚未建立 v2 上下文，使用原 v1 错误响应；inspect 也保持 v1。消费者应先检查退出码和响应 schema_version。

不新增工程保存、组分 / 物性配置协议、任意连接、删除 / 断开、录制器、COM 或网络服务；输入工程及缓存仍只读。本地请求格式与项目文件 schema 独立。

## 响应与退出码

响应始终包含 `schema_version`、`status`、`document`、`package_id`、`writes`、`variables`、`diagnostic`、`error`。尚未到达的单值阶段为 null，未产生的列表为空数组。

| 退出码 | status | 意义 |
| --- | --- | --- |
| 0 | `ok` | inspect 完成，或运行生成当前修订的收敛快照且全部查询完成 |
| 2 | `error` | 命令 / JSON / 文件加载 / 身份修订 / 步骤 / 写入 / 读取失败；见 `error.stage`、`code`、`message`、可选 `index` |
| 3 | `blocked` | 正式 readiness 或包选择阻塞，或运行跳过；没有当前成功结果 |
| 4 | `failed` | 缓存资产不可用、求解失败，或未形成当前修订的收敛快照 |
| 5 | 无可靠响应 | 标准输出写入、flush 或 JSON 序列化失败，不能把部分输出当成功 |

- `writes`：每条含原 target、实际修订与 `changed`。相同有效输入返回 `changed: false`，不增加修订。
- `variables`：每条含 target、label、unit、value、state、source、writable。`state` 为 `valid` / `invalid` / `unspecified` / `missing` / `stale`；缺失和旧化不冒充当前数值。`source.kind` 区分 `document_input`、`stream_template`、`solve_result`、`no_result`，求解来源另含 snapshot / revision / sequence。
- `diagnostic`：沿用运行诊断的 code、message、unit_ids、stream_ids、ports，可定位具体对象及端口；无正式诊断时为 null，仍可从 `error` 获取阻塞原因。原有诊断代码不由 CLI 改写。
- 运行阻塞 / 失败时不执行 reads。成功运行后的某个查询失败时退出码为 2、stage 为 `read`，保留运行诊断与文档修订，整个 `variables` 为空，避免误用部分查询值。

当前是同版本本地工具接口，尚不承诺跨版本公共传输兼容性。实现和阶段边界见 [Studio B3-4](../topics/studio-main-workflow.md#b3-4首个无界面参数化运行消费者) 与 [B3-5](../topics/studio-main-workflow.md#b3-5无界面创建连接与步骤身份引用)。

## 新版 v3：查询

AF4c-1 已提供 `radishflow-studio --headless query request.json`；不需要物性缓存，不修改磁盘或触发计算。工程 v1 / v2 / 已知 v3 使用相同入口，未知设备能力保留整份只读保护。以下是独立请求，不与旧 `run` 混用：

```json
{
  "schema_version": 3,
  "project": "project.rfproj.json",
  "queries": [
    {"id": "objects", "query": {"kind": "objects"}},
    {"id": "flow_results", "query": {"kind": "results", "scope": "flow"}}
  ]
}
```

三个顶层字段必填，queries 可为空；id 非空白且唯一。未知 / 重复字段、错误类型和非有限数拒绝。可选字段用省略表达，不接受 null；路径相对请求文件目录。

| query.kind | 参数与输出 |
| --- | --- |
| objects | 无其他参数；返回对象列表，含 target、document_id、parent、kind、label |
| fields | target 必填；返回流程变量或设备输入字段、规范单位、值状态及来源；未指定不补零 |
| record | target 必填；返回对象正式记录、版本和专用 data；方法、身份、固定采纳证据是只读元数据 |
| actions | scope 必填；返回领域动作描述及适配器 callable；已覆盖本批正式输入、检查、同步计算和 C1 采纳 |
| results | scope 为 flow，或精确 task / case 对象；返回 Current / Stale / Missing / Failed 资格与实际来源，仅 Current 返回数值 |

对象 target 为单一枚举对象：`{"unit":{"id":"u"}}`、`{"stream":{"id":"s"}}`、`{"equipment":{"id":"t"}}`、`{"case":{"task":{"id":"t"},"case":{"id":"c"}}}`、`{"geometry":{"id":"t"}}`、`{"basis":{"id":"b"}}`、`{"source":{"id":"src"}}`。scope 是 `"project"`（根动作发现）、`"flow"` 或 `{"object": target}`；单独 query 不存在前序动作输出，步骤引用会明确失败。

响应固定包含 schema_version、status、project、steps、queries 和 error。project 包含 document_id、project_schema_version、flow_revision、file_sha256、read_only、equipment_task_count；只读工程另有 read_only_reason。file_sha256 为同次读取原始字节的 64 位小写十六进制 SHA-256，空白变化也会改变指纹，不是签名。步骤尚未执行时 steps 为空。每项查询分别返回 value 或结构化 error；一项失败不阻止后续只读查询，顶层保留首个错误。

未知能力工程的 objects 只列可安全读取的流程对象，数量与只读原因在 project 返回；fields / record 只允许流程输入，writable 为 false，设备记录和结果返回 unsupported_capability。源文件不被裁剪或重写。

本阶段退出码 0 表示查询完成，2 表示请求 / 装载 / 查询错误，5 表示响应输出失败；Missing 等结果状态是有效查询。已保存工程不会恢复运行缓存，因此初次结果通常为 Missing；固定采纳证据仍是历史记录，不等同当前计算结果。


## 新版 v3：正式输入与影响确认

AF4c-2 提供 `radishflow-studio --headless execute request.json`。请求必填 schema_version、project、expected_project、steps、queries；expected_project 从 query 的 project 中提取 document_id 和 file_sha256。每步为 `{id, action, confirmation?}`，确认字段通常省略。所有步骤及嵌套数据先严格解析，再检查原文件身份和指纹；流程、设备输入与检查不加载物性缓存。

```json
{
  "schema_version": 3,
  "project": "project.rfproj.json",
  "expected_project": {"document_id": "从 query 复制", "file_sha256": "从 query 复制"},
  "steps": [
    {"id": "rating", "action": {"kind": "create_rating_task"}},
    {"id": "labels", "action": {
      "kind": "update_rating_labels",
      "task": {"step": "rating", "output": "task"},
      "expected_revision": {"step": "rating", "output": "task"},
      "data": {"tag": "V-101", "name": "分离器校核"}
    }}
  ],
  "queries": [
    {"id": "record", "query": {"kind": "record", "target": {"equipment": {"step": "rating", "output": "task"}}}}
  ]
}
```

对象引用为 `{id}` 或 `{step, output}`，output 限 document / unit / stream / task / case / geometry / basis / source；每步只提供实际可用且不歧义的输出。geometry 的对象身份沿 task，geometry 输出用于几何版本引用。expected_revision 为 u64 或 `{step, output}`，引用必须属于实际目标记录；不能引用另一个同版本对象。流程动作要求 document 版本，设备更新要求被更新记录版本；Sizing case 没有独立版本。

| action.kind | 必填字段；可选字段注明 |
| --- | --- |
| write | expected_revision、target（object / section / field）、value；字段沿流程目录 |
| create_unit | expected_revision、unit_kind（六类内建模型） |
| connect | expected_revision、source（unit 引用 / port）；sink、stream 可省略 |
| disconnect | expected_revision、port（unit 引用 / port） |
| disconnect_stream | expected_revision、stream；direction 可省略，或 inlet / outlet |
| create_sizing_task / create_rating_task / create_source | 无其他字段；建立正式空记录，输入由后续 update 完成 |
| create_basis | data：name、value、source |
| update_sizing_task | task、expected_revision、data：tag、name、case、axial_sections |
| update_source | source、expected_revision、完整 SourceData |
| update_basis | basis、expected_revision、data：name、value、source |
| update_rating_labels | task、expected_revision、data：tag、name |
| update_rating_geometry | task、expected_revision（geometry）、完整 GeometryData |
| update_rating_case | task、case、expected_revision（case）、完整 RatingCaseData |
| add_rating_case | task、expected_revision（task）；copy_from 可省略，必须为同任务 case |
| reorder_rating_cases | task、expected_revision（task）、cases（完整有序引用数组） |
| delete_equipment_record | target、expected_revision；只允许领域可删除的记录，共同几何不可单独删除 |
| check_sizing / check_rating | task；check_rating 另需 case |

完整 data 可从对应 record 查询复制，修改后提交；不能夹带 id、revision、method、adoption 或其他只读元数据。主要数据结构如下，所有字段除明确可选项外必填：

- 工程数值为 `{quantity, value_si}`。绑定为 `{kind:"direct", value, source}`、`{kind:"basis", basis}` 或 `{kind:"override", basis, value, source, reason}`；覆盖理由不得为空。
- SourceData：kind、title、reference、locator、publication_version、adoption_note、review、applicability、observations；accessed_on 为可选 ISO 日期。applicability 包含 fluid_system、phase、scope，可选 operating_conditions（temperature_k / pressure_pa）。observation 包含 id、quantity、original_value、original_unit、canonical_value、conversion_note、depends_on；依赖为 source 引用与 observation_id。
- Sizing case：name、purpose、fluid_system、geometry_applicability、case_applicability、applicability_sources、inputs。GeometryData：applicability、applicability_sources、inputs、axial_sections。RatingCaseData：name、purpose、fluid_system、applicability、applicability_sources、inputs。
- geometry_applicability 包含 internals_description，可选 orientation / gas_direction / geometry；case_applicability 的 service / k_basis / flow_basis 均可省略。数值 inputs 和 axial_sections 是显式字段枚举到绑定的映射；字段 / quantity 由 fields 查询和[类型目录](../../apps/radishflow-studio/src/headless/v3/enums.rs)确定，不接受任意字段或单位。

每步回执包含 outcome、实际身份 / 版本 outputs、影响；无变化为 unchanged，不推进版本。检查为 checked 并返回结构化 issues，有问题仍是成功检查。第一处失败或待确认停止后续动作；queries 对停止时正式状态逐项执行，不能覆盖原动作错误。原 project 快照仍表示装载输入，各步版本见回执。

触发删除、依据影响或旧 Rating 方法升级时，退出码为 6，error.details 提供 preview 与 confirmation_digest。预览包含旧 / 新输入、记录 / 方法版本及影响；该步未提交。审阅后把 `{ "digest": "af4c-confirm-v1:…" }` 放入原步骤的 confirmation，重新执行同一请求。前序步骤会重新执行，摘要匹配才提交本次新准备的事务；摘要错误返回 confirmation_mismatch 与新预览。无需确认的动作带 confirmation 会被拒绝且不留下该步变更。

退出码 0 / 2 / 3 / 5 / 6 分别为完成、请求或版本 / 查询错误、执行门禁拒绝、输出失败、待确认或确认内容变化。run_flow / size / rate / adopt_geometry 见下节；不执行隐式计算。任何成功输入回执都只作用于本次内存，不写回工程。


## 新版 v3：显式计算、采纳与结果

AF4c-3 已接通 run_flow、size、rate 和 adopt_geometry。每个计算步骤同步返回实际终态，步骤之间可编辑或再次计算；不提供后台轮询、取消、条件或循环。size / rate 完全不读取流程缓存。

仅当 steps 含 run_flow 时，execute 顶层必须提供 flow_resources；其他请求携带它会在执行前拒绝。flow_resources 必填 cache_root、auth_cache_index 和 package，路径相对请求文件。package 为 `{ "kind": "preferred" }` 或 `{ "kind": "explicit", "package_id": "本地包ID" }`；保留正式选包、授权、readiness 和摘要校验。缓存索引在实际 run_flow 步骤才读取，加载失败不会抹掉前序编辑回执。

| action.kind | 必填字段与版本 |
| --- | --- |
| run_flow | expected_revision：document |
| size | task、expected_revision：Sizing task |
| rate | task、case、expected_case_revision：精确行；expected_geometry_revision：共同几何 |
| adopt_geometry | source：Sizing task、expected_revision：源 task、target、adjustments |

adopt_geometry.target 为 `{kind:"new", tag, name, copy_case}` 或 `{kind:"existing", task, expected_revision}`，后者校验目标 geometry 版本。adjustments 必填，可为 `{}`；可选 diameter / working_liquid_height，每项为 `{value_m, reason}`，调整值必须为正且理由非空。只能采纳本次工作区的当前成功候选；先执行 size，输入或来源变化导致旧化后必须重新 size，不能导入旧候选或恢复旧运行 key。

采纳始终先返回退出码 6 的审阅提案。preview 还包含本次候选尺寸、输入、来源、方法、限制、旧 / 新几何和完整固定证据。摘要排除执行身份和耗时，包含资源选择及调整内容；重放重新计算、重新验证当前资格。确认后输出 task / geometry、唯一 case（适用时）和 source（本次设计来源），created_sources 列出复制来源映射。复制来源在摘要中按原来源身份命名，不依赖生成字符串；原工程归档证据不套用当前身份归一化。

运行回执的 outcome 区分 completed / start_failed / blocked / skipped；completed 另含 terminal（succeeded / failed / cancelled / infrastructure_failed）、reception（applied / discarded）、execution（request 范围的 step）、诊断及当时的 result。执行失败退出 4；开始前的正式门禁拒绝退出 3；版本错误仍退出 2。只有成功且接收的计算可继续后续动作，最终查询依然独立执行并保留原失败优先级。

result 为 Current / Stale / Missing / Failed 的小写标记，仅 current 含数值和规范 SI 单位。设备结果 origin 包含任务 / case / geometry 版本、输入及证据 generation、方法和 basis / source 版本；流程 origin 包含真实 solver、package、payload 指纹及数据来源。所有运行引用只属于本次请求；重新启动不会恢复结果。显式 rate 算法成功但 conclusion 为 exceeded 是有效校核结果，退出 0 不表示满足工程准则或验证了物理精度。

### v3 有限物料计算扩展（2026-10-31，已确认）

显式增加 `run_material_balance` 动作和 `material_balance` 查询作用域；原 v1 / v2 与已有 v3 请求、回执形状保持。物料任务仅支持两股固定同温同压 Feed 连接终端 Mixer，不产生完整流程热状态，不读取物性缓存。

动作必填 `expected_revision`（document）、`mixer`（Unit 身份引用）与 `settings`。settings 必填 `algorithm: "lower_triangular_substitution"`、`scales_mol_s: {inlet_a, inlet_b, outlet}`、`absolute_tolerance_mol_s`、`scaled_tolerance`，尺度 / 容差须正且有限。可选 `outlet_total_molar_flow_mol_s` 用于明确规格检查：固定进料已确定总量，重复和冲突均拒绝。未知字段 / 算法或显式 null 仍严格拒绝。

物料动作不要求 flow_resources；只有同一请求另含 run_flow 时才提供它。沿用现有步骤引用、expected_project / SHA-256 检查、一次性执行身份与磁盘只读。准备前的非法计划返回 blocked / 3，装配或数值失败返回 failed / 4 与当次独立诊断；最终查询仍执行。

查询示例为 `{ "kind": "results", "scope": "material_balance" }`，动作发现使用相同 scope 的 actions 查询。Current 返回变量 / 方程的 unit、typed port、stream、component 领域绑定、规范 SI 数值、原始与缩放残差、出口物料量和实际算法 / 修订、冻结设置及成本统计。没有初值迭代时不输出虚构迭代数。Stale 仅给来源，不输出旧值；新进程为 Missing，不从文件恢复结果。物料结果不会改变 flow 结果作用域或设备计算资格。

[公开工程](../../examples/flowsheets/feed-mixer-material-balance.rfproj.json)与[执行请求](../../examples/automation/material-balance-v3.request.json)可直接复现：

```sh
target/debug/radishflow-studio --headless execute examples/automation/material-balance-v3.request.json
```

预期出口 5 mol/s，Methane / Ethane 摩尔分率 0.44 / 0.56。请求的退出码 / 回执不表示热力学准确性或完整 EO 流程验收；真实进程测试同时检查整个临时目录及原工程字节不变。

## v3 公开示例与复现

- [流程建模与显式运行请求](../../examples/automation/build-feed-heater-flash-v3.request.json)：从已有组分 / 包选择的空工程创建 Feed、Heater、Flash，建立受控连接，写入 SI 输入，再执行 run_flow。先把 flow_resources 改为已有本地缓存根目录和索引；CLI 不下载或生成替代资产。
- [独立设备空工程](../../examples/flowsheets/standalone-separator-basis.rfproj.json)与[查询请求](../../examples/automation/query-separator-v3.request.json)：已知 v3，设备区为空，保留完整显示单位配置。CLI 仍使用 SI，不受摄氏度、bar 等显示偏好影响。
- [尺寸计算、采纳与校核请求](../../examples/automation/size-adopt-rate-v3.request.json)：创建来源和依据，填入公开合成 S0-B01 算术输入，检查、size、adopt_geometry、check_rating、rate，最后读取结果。无需缓存；首次执行在 adopt 停止并返回 6，审阅 preview 后仅为该步补 confirmation，再重放整个请求。

在仓库根目录使用已构建程序：

```sh
target/debug/radishflow-studio --headless query examples/automation/query-separator-v3.request.json
target/debug/radishflow-studio --headless execute examples/automation/size-adopt-rate-v3.request.json
```

示例的 expected_project 对应当前提交中的原始工程字节。若修改工程、转换换行或从别处复制后字节改变，先重新 query 并核对 / 更新身份和指纹，不能自动忽略冲突。确认步骤不会修改原请求文件；由调用方审阅后填入摘要。新进程重新装载时不会看到上次创建的设备或计算结果。

本批自动验收在 macOS 执行；真实进程测试逐字节比较工程、受控合成本地缓存、已有 sidecar 和整个测试目录清单，覆盖确认重放、来源改变、连续确认、失败终态和旧协议拒绝 v3。Windows / Linux 原生运行、桌面实窗、COM、物理准确性与发布验收不由这些 CLI 测试替代。


## QS1-C CLI v4 提案（待确认、不可调用）

当前 query / execute 仍为 schema_version=3。拟定的 [QS1-C06](../topics/modeling/recycle-project-and-consumers.md#qs1-c06cli-v4-与兼容消费)使用新 schema_version=4 接入计划查询 / 应用、Splitter / 显式连接和分块结果；请求与工程版本分别判定，旧 v3 明确拒绝 v4 工程。仍只操作本次内存副本，不提供工程保存或文件导出。此处仅作方案导航，不改变以上已实现的词汇、请求 / 回应或退出码。
