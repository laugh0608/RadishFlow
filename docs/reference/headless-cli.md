# 本地无界面运行接口

更新时间：2026-09-16

用途：说明 Studio B3-4 / B3-5 命令、JSON 请求 / 响应和错误语义。

读者：使用本地脚本执行参数化工况的用户及适配器开发者。

不包含：公共网络 API、COM、录制脚本、后台任务与项目保存。

## 规划兼容边界

B3-5 已通过 v2 请求接通受控创建 / 连接和步骤身份引用，v1 请求与响应字段保持兼容。2026-09-16 确认的 [单位系统](../topics/units-and-quantity-system.md) 和 [建模辅助](../topics/modeling-assistance-and-specifications.md) 后续复用本接口的身份、修订、命令与诊断；本文 v1 的 SI 数值、路径与错误契约不变。U1 已让响应的 unit 标签从公共目录派生，v1 / v2 仍只读写 SI，不接收额外单位字段。未来带单位请求须显式版本化或定义兼容扩展，不受 GUI 显示偏好影响。

## 命令

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
