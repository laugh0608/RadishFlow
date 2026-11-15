# VB-D02 稳定变量引用契约

更新时间：2026-11-15

## 用途与状态

2026-11-12 所有者认可 [变量与对象浏览器](../variable-and-object-browser.md#稳定身份与路径) 的引用解析规则、持久创建标识及旧工程显式初始化。Studio 首批流程标量的复制和跳转已接通。项目存储、迁移与兼容见 [存储契约](identity-storage.md)。持久关注归 VB-D03，外部协议归 VB3。

## 现有身份与生命周期核对

| 核对项 | 当前行为及对引用的影响 |
| --- | --- |
| 变量定位 | `VariableId` 由 `DocumentId`、`UnitId` / `StreamId`、输入或结果分区和 `VariableField` 组成。`ProjectQuery` 以 `ProjectObjectRef` 和 `ProjectField` 在指定工作区内读取。对象显示名可重复或修改。设备工况使用任务 ID 加工况 ID，共同几何归所属任务。见 [变量描述](../../../crates/rf-ui/src/variable_browser/variables.rs)、[工程查询](../../../crates/rf-ui/src/project_query/mod.rs)。 |
| 保存、另存与重开 | 现有保存链序列化同一文档 ID 和工程对象，另存成功后将工作区路径切到目标，保存失败保留原路径与基线。加载用文件内 ID 构造新的工作区及运行会话。已有 macOS 代表验收中，另存副本和原工程的完整 JSON 相等。见 [生命周期入口](../../../apps/radishflow-studio/src/document_lifecycle_driver.rs)、[另存实窗记录](../../devlogs/2026-11/2026-W46.md#2026-11-11af3-关闭与原生另存补验)。 |
| 删除、恢复与重建 | 流程单元、流股的已分配 ID 在当前工作区保留，撤销恢复原对象。重开只用文件中仍存在的对象重建已分配集合。单元分配器从 `1` 开始寻找可用编号，已删除且未保存进文件的旧 ID 因此可能被重用。见 [工作区](../../../crates/rf-ui/src/state.rs)、[单元分配](../../../crates/rf-ui/src/state/actions.rs)。 |
| 查询上下文 | 会话、修订和快照资格约束读取时的输入与结果。新会话会拒绝旧 `ProjectInputContext`，却仍可按当前对象身份查询。AF4c 的 `file_sha256` 绑定整份磁盘快照，内容变化即失效。见 [上下文回归](../../../crates/rf-ui/src/project_query/tests.rs)、[CLI 快照条件](../variable-and-object-browser.md#输入身份与请求结构)。 |

## 解析规则

**以用户当前打开的工程为解析范围。** 粘贴前后均显示当前工程的名称和文件位置。同 ID 副本由用户通过原工程打开流程主动选择。解析只查询当前可编辑工作区，项目切换会取消待完成的定位。解析器不扫描磁盘、最近项目或其他工作区。

| 场景 | 解析结果 |
| --- | --- |
| 原工程普通编辑、重命名、移动、保存重开 | 文档 ID、对象类型、对象 ID、创建标识与字段一致时定位。展示当前名称、值、单位、来源和结果资格。 |
| 另存成功或打开操作系统复制的文件 | 副本保留文件中已有的身份。在副本内可定位继承的对象。另存切换工作区路径后，展示该副本的当前值。取消或保存失败时继续在原工作区解析。 |
| 原件与副本分叉 | 共同继承的对象仍可在当前打开的分支定位。分叉后独立创建的对象分配不同创建标识，即使两边使用了同一对象 ID。旧引用在缺对象的一侧报缺失，在复用 ID 的一侧报身份不匹配。 |
| 删除、撤销及重建 | 删除时对象缺失。撤销或重做若恢复同一对象则恢复其创建标识。重新创建始终分配新标识，同一会话及保存重开后均拒绝旧引用。 |
| 打开不同 `DocumentId` 的工程 | 报工程不匹配，即使存在同名、同类型、同 ID 的对象。禁止按名称、序号、文件名或其他字段推测。 |
| 模型对象复制或未来导入 | 新建对象归目标工程并分配新创建标识。原引用仍指向来源对象。跨工程映射随显式导入或回放功能审定。 |
| 未建立身份记录的旧工程、未知格式只读页 | 旧工程继续使用现有浏览路径，稳定引用功能提示身份未建立。未知格式只读页提示工作区不可用。两种情况都不尝试读取上一个工作区。 |

### 持久身份前置条件

工程保存类型化对象 ID 与不可变的**创建标识**。创建时分配，删除和撤销纳入同一输入事务，保存和恢复随对象处理。各副本独立分配。连接和求解计划继续使用原对象 ID。复制稳定引用只对已在当前目标成功保存身份的对象开放，保存失败时保留既有基线。

旧工程由用户显式初始化并保存身份。共同继承的副本需要从已保存身份的版本复制。两份历史副本分别初始化会得到独立创建标识，缺少原始标识的旧副本无法证明对象相同。存储格式、兼容保护与失败恢复见 [存储契约](identity-storage.md)。

### 文本格式与读取

```text
rfvar:1/<document-id>/<unit|stream>/<object-id>/<creation-id>/<inputs|results>/<field>
rfvar:1/doc-demo/unit/heater-1/b91c614d-4a7a-4d0e-82ba-09c82da255ba/inputs/outlet_temperature
```

- 首批字段为现有目录已描述的单元输入 `name`、`outlet_temperature`、`outlet_pressure`、`outlet_a_split_fraction`，以及流股输入 `name` 与输入 / 结果中的 `temperature`、`pressure`、`molar_flow`。输入和结果使用不同分区。组成、相、设备记录和动作留待各自增量。端口定位到已连接流股的同一引用。
- 固定段和字段令牌显式登记并区分大小写。文档、对象和创建标识按原始字符串逐段编码，ASCII 字母、数字及 `-._~` 原样保留，其余 UTF-8 字节使用大写 `%HH`。先分段，再严格解码一次。拒绝空段、非法 UTF-8、非法转义和非规范编码。ID 区分大小写，保持原有 Unicode 字节序列。v1 暂无别名，未知版本单独拒绝。显示名称、单位、文件路径、会话、修订与快照不进入引用。
- 解析错误区分格式错误、版本不支持、当前工作区不可用、工程不匹配、字段类型尚未纳入、对象缺失、创建标识不匹配和对象上无该字段。结构和版本通过后校验工作区与文档，再查对象、创建标识和字段。格式有效但超出首批字段表时提示覆盖范围。字段存在但未指定、无结果、过期或失败时照常定位，数值按现有资格展示。
- 入口为浏览器中的复制引用与粘贴跳转。输入定位到原编辑器，结果定位到浏览器中的精确字段。解析复用 `ProjectQuery`，草稿确认后再次核对会话与目标。跳转只改变浏览或定位状态。见 [浏览器定位](../../../apps/radishflow-studio/src/studio_gui_shell/variable_browser/navigation.rs)。

## 实施验收

| 场景 | 通过条件 |
| --- | --- |
| 文本与字段 | 编解码往返，覆盖中文、空格、`/`、`%`、大小写和不同 Unicode 字节序列。非法编码、未知版本、非法分区组合、未纳入字段及对象上缺字段得到明确拒绝。 |
| 原工程 | 复制引用后修改显示名，保存重开仍跳到原字段。删除后定位失败，Undo 恢复。保存删除并重开，重建同 ID 对象后旧引用报身份不匹配。 |
| 副本与其他工程 | 原件另存和系统复制后均能找到继承对象。原件、副本各自改值，只显示当前打开的一份。两侧独立创建同 ID 对象不能互相定位。不同文档和未初始化的历史副本拒绝。另存失败后仍在原工程解析。 |
| 资格与副作用 | 结果引用覆盖 Current、Stale、Missing、Failed。旧快照、模板和草稿不被当作当前值。草稿确认取消保持原选择，切换会话取消旧导航。验证工程修订、未提交输入、文件内容和运行状态不因复制或跳转变化。 |
| 存储兼容 | 旧格式正常打开，显式升级取消或失败保留原文件。保存重开恢复身份，旧写入器对新能力按现有只读和写入保护处理。 |

阶段验证见 [W46](../../devlogs/2026-11/2026-W46.md)。

## VB3 CLI 只读查询首片

2026-11-15 项目所有者确认后实施。调用场景为维护者复核 Studio 保存后的正式输入。沿用 VB2 双进料实窗验收工程 `VB2-Batch.rfproj.json` 的 r3 副本，外层 v4，已保存对象身份和关注清单。复核 `stream-feed-a` 的输入总摩尔流量 `10 mol/s`，以及 `stream-liquid` 的结果总摩尔流量 `Missing`。用公开算例复现时，在 Studio 对临时副本启用并保存稳定引用，将进料 A 保存为 `10 mol/s`，重开后复制两条引用。自动验收复用同一 Studio 生命周期准备样本。 [公开算例](../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json) · [VB2 实窗证据](../../devlogs/2026-11/2026-W46.md#2026-11-14-vb2-进料流量批量编辑首片)

### 请求与响应

`--headless query <request.json>` 的 `schema_version: 8` 只接受 `stable_variable` 查询。请求字段为 `schema_version`、`project` 和 `queries`。相对路径以请求文件为基准，查询 ID 非空白且唯一，可用空列表读取工程头。保持严格 JSON 校验。复用样例时，将 `project` 和 `reference` 换成自己的保存文件与 Studio 复制的完整引用。

```json
{
  "schema_version": 8,
  "project": "VB2-Batch.rfproj.json",
  "queries": [
    {
      "id": "feed-a-flow",
      "query": {
        "kind": "stable_variable",
        "reference": "rfvar:1/vb2-batch-native-review/stream/stream-feed-a/1d8a0644-26f9-430c-96e1-2943f578a703/inputs/molar_flow"
      }
    }
  ]
}
```

- 复用 v7 的计划读取覆盖，为 v8 `query` 单独启用已知对象身份及关注清单存储能力。工程外层仍为 v4。可识别但无已保存身份的工程按 `object_identities_unavailable` 返回查询错误；未知必需能力仍整份保护，工程头返回只读原因，查询项返回 `unsupported_capability`。
- 用现有 `StableVariableReference` 解析并核对已加载工程，再由 `ProjectQuery` 读取字段。沿用首批标量表及现有流字段序列化形状，在成功项 `value` 中附上规范 `reference`，`variable.writable` 为 `false`。数字沿用 SI，`variable.source` 复用字段来源。工程头的 `read_only` 仍说明文件能力是否触发保护，`file_sha256` 来自本次装载的原字节。
- 复用响应外层及逐项顺序。成功读取进料返回 `state: "valid"`、`value: 10.0`、`unit: "mol/s"`、`source.kind: "document_input"`。新进程从保存文件读取结果字段时返回 `state: "missing"`、`value: null`、`source.kind: "no_result"`，退出 `0`。字段未指定继续使用 `unspecified`。首次查询不恢复求解快照、不启动计算、不读物性缓存、不写工程及用户状态。

### 拒绝顺序与验收

先校验请求版本、结构与路径，再装载工程并执行整份能力保护。可用工作区中的单项错误沿用稳定引用解析顺序。格式错误用 `invalid_variable_reference`，未知引用版本用 `unsupported_reference_version`，未纳入的字段或分区用 `unsupported_field`。解析后按工程 ID、身份区、对象存在性、创建标识、对象字段顺序检查，分别返回 `different_document`、`object_identities_unavailable`、`object_missing`、`object_recreated`、`field_missing`。查询项失败带 `query_id`，后续只读项继续，顶层保留首错，退出 `2`。

`execute` 收到 v8 请求后，在读取工程前返回请求级 `unsupported_version`，沿用未建立协议上下文时的 v3 错误响应，退出 `2`。

| VB-A07 验收 | 通过条件 |
| --- | --- |
| 共享读取 | Studio 保存后重开与 CLI 读取同一文件，两条引用的字段身份、SI 值、来源和结果资格一致。修改显示单位后 CLI 仍为 SI。重命名或另存继承对象身份后按显式选择的文件读取。 |
| 身份与错误 | 覆盖非法文本、未知版本、未纳入字段、合法字段在对象上缺失、不同文档、对象删除、同 ID 重建、旧文件未启用身份、未知必需能力、已知身份区损坏及混合成功与失败的查询。失败项无数值。 |
| 协议兼容 | 旧 `inspect` / `run` 及 CLI v3 保留 v4 拒绝；v4–v7 `query` 仍只给安全元数据，`execute` 对有效请求仍依次核对工程 ID、文件指纹和只读保护。v3–v7 严格拒绝 `stable_variable`。v8 `execute` 在工程 IO 前按现有请求级 `unsupported_version` 拒绝。 |
| 真实进程与磁盘边界 | 复用 CLI 子进程矩阵，验证单份 JSON、退出码、错误阶段和逐项回执。查询成功与失败后逐字节比较工程、现有 sidecar 与目录清单。按影响面跑共享查询回归及正式仓库基线。 |

首片止于本地只读标量复核。CLI 写入、关注清单枚举、运行、多工况、COM 和录制各按对应消费者另行评审。 [CLI 现行版本契约](../../reference/headless-cli.md#版本与兼容边界)

macOS 真实 CLI 进程验收通过，另以 VB2 实窗保存的 r3 文件字节复跑代表场景。 [W46 验收证据](../../devlogs/2026-11/2026-W46.md#2026-11-15-vb3-cli-v8-实现与验收)
