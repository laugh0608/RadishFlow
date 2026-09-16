# 本地无界面运行接口

更新时间：2026-09-16

用途：说明 Studio B3-4 命令、JSON 请求 / 响应和错误语义。

读者：使用本地脚本执行参数化工况的用户及适配器开发者。

不包含：公共网络 API、COM、录制脚本、后台任务与项目保存。

## 规划兼容边界

B3-5 将按现有受控动作扩展创建 / 连接，当前尚未实现。2026-09-16 确认的 [单位系统](../topics/units-and-quantity-system.md) 和 [建模辅助](../topics/modeling-assistance-and-specifications.md) 后续复用本接口的身份、修订、命令与诊断；本文 v1 的 SI 数值、路径与错误契约不变。未来带单位请求须显式版本化或定义兼容扩展，不受 GUI 显示偏好影响。

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

## 响应与退出码

响应始终包含 `schema_version`、`status`、`document`、`package_id`、`writes`、`variables`、`diagnostic`、`error`。尚未到达的单值阶段为 null，未产生的列表为空数组。

| 退出码 | status | 意义 |
| --- | --- | --- |
| 0 | `ok` | inspect 完成，或运行生成当前修订的收敛快照且全部查询完成 |
| 2 | `error` | 命令 / JSON / 文件加载 / 身份修订 / 写入 / 读取失败；见 `error.stage`、`code`、`message`、可选 `index` |
| 3 | `blocked` | 正式 readiness 或包选择阻塞，或运行跳过；没有当前成功结果 |
| 4 | `failed` | 缓存资产不可用、求解失败，或未形成当前修订的收敛快照 |
| 5 | 无可靠响应 | 标准输出写入、flush 或 JSON 序列化失败，不能把部分输出当成功 |

- `writes`：每条含原 target、实际修订与 `changed`。相同有效输入返回 `changed: false`，不增加修订。
- `variables`：每条含 target、label、unit、value、state、source、writable。`state` 为 `valid` / `invalid` / `unspecified` / `missing` / `stale`；缺失和旧化不冒充当前数值。`source.kind` 区分 `document_input`、`stream_template`、`solve_result`、`no_result`，求解来源另含 snapshot / revision / sequence。
- `diagnostic`：沿用运行诊断的 code、message、unit_ids、stream_ids、ports，可定位具体对象及端口；无正式诊断时为 null，仍可从 `error` 获取阻塞原因。原有诊断代码不由 CLI 改写。
- 运行阻塞 / 失败时不执行 reads。成功运行后的某个查询失败时退出码为 2、stage 为 `read`，保留运行诊断与文档修订，整个 `variables` 为空，避免误用部分查询值。

当前是同版本本地工具接口，尚不承诺跨版本公共传输兼容性。实现和阶段边界见 [Studio B3-4](../topics/studio-main-workflow.md#b3-4首个无界面参数化运行消费者)。
