# 设备计算表格式与输出资格

更新时间：2026-10-26

用途：维护 Sizing / Rating / Matrix CSV、JSON 的字段、版本及输出资格。

读者：输出消费者、Studio 与设备工程维护者。

不包含：数值方法推导、项目存储格式和原生选择器实现；领域范围见[分离器专题](../topics/equipment/separator-sizing-and-rating.md)。


## Sizing 计算表

CSV / JSON 仅导出独立分离器的当前候选，不扩展项目 v3、CLI、计算方法或依赖；文件为计算证据，不是可导入工程或自动化协议。

| 内容 | 字段与语义 |
| --- | --- |
| 身份 | 工程 ID、任务 / 工况 ID、发起时名称和位号、当前与计算捕获的记录 revision、输入 / 证据代次、方法 ID / 版本；内存计算身份仅用于资格重验，不作为可恢复令牌写出 |
| 工况与适用性 | 工况用途、物系、显式结构 / 相态 / 流量 / K 基准、内件说明和适用性来源 ID |
| 输入 | 十项正式输入、六个轴向区段；有效值与原始绑定分开，保留 Direct / BasisRef / Override、基础与来源 ID、覆盖理由及原始基础值 |
| 结果 | 最小直径、工作容积 / 液层高度、气相 / 液体 / 必需面积、允许气速、Souders–Brown 气速、气相负荷比例、控制约束、总高及缓冲容积；附方法公式 |
| 证据 | 捕获的基础条目和来源版本、定位、访问日期、采用理由、审阅 / 假设状态、适用条件、原始观测值 / 单位、规范值和转换依赖；不读取未引用来源 |
| 限制 | 条件式初步工艺尺寸、缺失或无效区段、总高问题、假设 / 待复核来源及全部未核查项；未知量用显式 `unknown` 与空数值表示，不补零 |

CSV 与 JSON 消费同一计算表行模型，按 `section / recordId / field` 定位事实。固定列为 `section,recordId,field,status,quantityId,valueSi,siUnitId,displayValue,displayUnitId,displayUnitSymbol,text`；JSON 包含格式标识、版本 `2`（AF5b）及 `rows`。CSV 首部的元数据行同样记录格式与版本，使用 UTF-8、逗号、CRLF 和双引号转义。JSON 使用 UTF-8；结构键和枚举值不随界面语言变化，JSON 保留用户来源原文；CSV 对可能被表格软件解释为公式的文本加前导单引号，原有前导单引号同样转义，数值列不改。

规范值 `valueSi` 保持浮点有效精度；展示值 `displayValue` 由统一单位目录和 6 位有效数字格式生成，点号小数，不用于无损回放。两个格式都包含量种、规范单位 ID、展示单位 ID / 符号。导出在打开保存选择器之前捕获项目单位，忽略临时视图覆盖；随后切换单位不修改已捕获的输出。来源原始值 / 单位作为原始证据另列。转换失败拒绝整份输出，不回退为错标单位的 SI。

仅通过 `current_candidate` 取得的成功候选可准备输出；当前任务及其依赖的未提交草稿阻挡导出，不相关草稿不阻挡。写文件前再次检查候选资格、同一计算身份及相关草稿：输入 / 证据改变、Undo / Redo、重新计算、同 ID 重开或切换工程均不得复用旧输出。名称变更不使数值过期，文件明确区分发起时标签和计算捕获标签；等待期间的标签变化不重建捕获数据。

入口在既有设备动作区提供 CSV / JSON 按钮及“使用项目单位”说明。按所选格式补扩展名；选择 `.rfproj.json` 时追加 `.calculation.csv` / `.calculation.json`，避免导出覆盖工程。已有计算表显式确认覆盖；取消、过期或写入失败保留原文件、输入 / 草稿、历史和工程脏状态。沿用既有文件写入策略，不将导出当作工程保存。自动回归覆盖两格式一致性、单位 / 精度 / 来源、未知与部分高度、资格失效、草稿、覆盖 / 取消 / 失败；macOS 补原生保存及设备页代表路径，其他平台与 I5 按原节点处理。

## Rating 校核表

校核表使用 `radishflow.separator-rating-table`、当前 version 3（AF5b），与 S1 共用 11 列行模型、单位目录、原生选择器及写入策略，列结构保持。包含任务 / 工况身份、标签与捕获 revision、给定几何、九项有效输入、轴向段、六项适用声明、绑定 / 基础 / 覆盖与传递来源、公式、条件限值、两项负荷 / 带符号余量、未舍入判定阈值及未核查项。缺高度 / 缓冲量使用 `unknown` 与空数值；约束键为 `withinCriteria` / `boundaryReview` / `exceeded`，不以显示舍入决定。

仅 Current 且无相关草稿可输出，包括超限与边界结果。路径选择前捕获项目单位；写前重验行身份、不可复用的计算身份、资格、草稿及项目单位，期间变更须重新导出。标签更改不使数值过期，输出区分发起时和计算时标签。取消、覆盖失败、过期拒绝不改输入 / 历史 / 脏状态；工程后缀保护沿用 S1。

## C1 固定采纳证据

C1 初版中，含采纳记录的单行表 `radishflow.separator-rating-table` 与矩阵 `radishflow.separator-rating-matrix` 使用 version `2`；未采纳任务和 S1 表使用 version `1`。AF5b 已将 Sizing 升为 `2`、Rating / Matrix 统一升为 `3`，下述采纳行内容保持。CSV 列数仍为单行 11 / 矩阵 12，JSON 保持相同行模型；CSV 的 `metadata/version` 与 JSON 的 `version` 一致。该版本独立于项目外层 v3 和 Rating 能力 v3。

- `adoption`：源 document / task / case、标签、用途 / 物系、方法 / 修订、适用声明来源及固定证据范围；不表示当前源执行或实测几何。
- `adoptionDimension`：内径、工作液层的 `originalValue`、`adoptedValue`、`adjustmentReason`、`currentValue` 和 `comparison`。
- `adoptionAxialSection`：全部轴向段的采用值、当前值与差异；未知值留空 / null，状态 `unknown`，不补零。`adoptionOriginalAxialInput` 保留原绑定，包括未被采用的段。
- `adoptionInput`、`adoptionBasis`、`adoptionSource`、`adoptionApplicability`：采纳时输入有效值、原绑定 / 覆盖、基础与来源闭包及审阅状态；与可变当前记录隔离，ID 只在各自证据范围内解释。

`comparison` 为 `unchanged` / `changed` / `addedSinceAdoption` / `currentUnknown` / `bothUnknown`，按未舍入 SI 比较；历史副本不因手改、源删除或重算而改变。所有量保留 SI 原值并按捕获的项目单位呈现，调整理由保留原文。

单行仍只输出 Current；矩阵共同证据只输出一次、`caseId` 为空，即使零 Current 也保留。非 Current 行仍无旧计算数值，成员与原因不丢失。写前沿用正式输入、草稿、结果身份和单位重验；固定证据不回读可变源任务。自动读回和原生代表验证见 [W43](../devlogs/2026-10/2026-W43.md)。

## AF5b 执行来源

Sizing 表 version `2`、Rating / Matrix 表 version `3` 新增 `executionOrigin` section。单行 `recordId` 为执行时工况 ID；矩阵额外以外层 `caseId` 限定成员。Current / Stale / Failed 使用各自冻结来源；Missing 的 `availability=unrecorded`、`recordId` 为空，仍由矩阵外层工况限定。非 Current 行不携带历史计算数值，C1 历史采纳副本独立保留。

| 字段 | 内容 |
| --- | --- |
| `documentId`、`taskId`、`caseId` | 执行所属工程 / 任务 / 工况 |
| `executionIdentity` | `recorded; session-local, not serialized`；不透明计算身份只用于会话内守卫，不输出内存地址 |
| `methodId`、`methodVersion` | 执行采用的方法与版本 |
| `taskRevision`、`caseRevision`、`geometryRevision` | 捕获的记录修订；Sizing 后两项为 `notApplicable` |
| `inputGeneration`、`evidenceGeneration` | 本工况执行捕获的输入 / 证据代次 |
| `taskTag`、`taskName`、`caseName` | 执行时标签；与既有发起时当前标签分开 |
| `basisRevision.<id>`、`sourceRevision.<id>` | 实际引用的基础及来源记录修订，不使用当前记录回填 |

CSV / JSON 仍共享同一行模型和版本值，列数保持 11 / 12；旧文件不改写，不增加旧版本导出选项。项目外层格式、能力版本、CLI / FFI 与方法数值保持。仅输出来源补充不改变既有草稿、计算身份、覆盖前单位和写入重验规则。
