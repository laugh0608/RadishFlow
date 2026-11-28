# 项目 v3 工程区字段与结构样本

更新时间：2026-11-28

本篇供工程文件读写器和设备任务维护者核对 G0/S1 的字段、绑定、来源引用与严格解码规则。保存确认、只读检查及 GS-01—10 验收见[项目生命周期专题](../topics/project-lifecycle-storage.md#v3-结构样本与版本行为)，后续校核任务扩展见 [S2 存储契约](../topics/project-lifecycle-storage.md#s2-校核任务保存扩展方案2026-10-20已确认)。

## v3 字段与兼容契约（2026-10-11）

以下为已认可 GE-D04 内的存储契约。旧 `StoredProjectFile` 常量仍为 2；Studio 通过独立 v3 DTO 分派，不扩大旧 CLI 能力。采用 UTF-8 JSON、camelCase，保持扩展名及 `kind=radishflow.project-file`；外层 `schemaVersion=3`，`document.metadata.schemaVersion=1`。先按外层版本分派，再交相应 DTO 验证，不能先把 v3 强行读进 v2 严格结构。读取始终不写盘。

| 位置 | 必需字段 / 规则 |
| --- | --- |
| 外层 | `kind`、`schemaVersion`、`document`、`presentation` 必需；`engineering` 可缺省，缺省表示空工程区；显式 null 拒绝。所有外层未知 / 重复键拒绝 |
| `document` | 原 v2 语义和字段保持；无设备专用虚构 unit / stream，不改变流程 revision 或 metadata 版本 |
| `presentation` | 只含 `displayUnits`，每项 `quantityId/unitId`；完整 15 类，各恰一次，写出按固定目录顺序。v2 始终只有原 7 类，详见 [单位契约](../topics/units/catalog-and-conversion.md#g0s1-量与单位契约2026-10-11) |
| `engineering` | 出现时必须包含 `basisEntries`、`sources`、`tasks` 三个数组，可为空；已知结构严格拒绝未知字段，不再增加同义版本号 |
| `basisEntries[]` | `id`、`revision`、`name`、`quantityId`、`valueSi`、`sourceId` 必需；量种来自统一目录，valueSi 有限；来源与适用条件从 source 引用 |
| `sources[]` | `id`、`revision`、`kind`、`title`、`reference`、`locator`、`publicationVersion`、`accessedOn`、`adoptionNote`、`review`、`applicability`、`observations` 必需；未取得的文本可为空，不能当作依据充分 |
| `tasks[]` | `id`、`revision`、`tag`、`name`、`kind`、`method`、`case`、`inputs`、可选 `axialSections`；S1 的 kind 为 `verticalGasLiquidSeparatorSizing`，不含流程 unitId |
| `method` | `id=radishflow.separator.vertical-sb-working-volume`、`version=1`；ID 和版本共同标识全部输入语义、公式及适用域，不按名称猜版本 |
| `case` | `id`、`name`、`purpose`、`fluidSystem`、`applicabilitySourceIds`、`applicability` 必需；文本可空，适用性声明按下表逐项保存 |
| `inputs` | 仅接受本节列明的十个字段键；字段缺省表示未填写，可保存但计算阻挡；存在时必须是完整绑定，不接受 null 或数字字符串 |
| `axialSections` | 可缺省；出现时是下列区段键到 length 绑定的映射，允许不完整且总高度未确定；不保存派生 h_work 或 H_TT |

全部实体 ID 非空、大小写敏感、项目内按记录种类唯一；case ID 在项目任务间唯一，数组顺序不构成身份。revision 为非负整数，保存记录版本而非执行凭据，变更单调推进。task/tag 唯一性按 G0；重复键、重复 ID、悬空引用、绑定量种不兼容或引用环拒绝装载，不丢掉坏项后打开。错误提供 JSON 路径及原因。有限但不符合设备方法范围的值可作为有问题的正式输入保留并定位，禁止产生候选；NaN/Inf、非法 JSON 和不完整绑定是格式错误。

十个输入键固定为 `operatingTemperature`、`operatingPressure`、`gasActualVolumeFlow`、`liquidActualVolumeFlow`、`gasMassDensity`、`liquidMassDensity`、`effectiveK`、`gasUtilization`、`workingDuration`、`maximumWorkingLiquidHeight`，映射见单位契约。六个可选轴向区段为 `bottomTangentToLLL`、`nllToHLL`、`hllToInletCenter`、`inletCenterToInternalsBottom`、`internalsThickness`、`internalsTopToTopTangent`，按 S0 定义解释，不按字段顺序重复加工作液层。具体内件和有效面积适用性说明必须由 case 的依据及 K 来源承载，不能只凭一个总括标签放行。

`case.applicability` 的六项枚举字段均必需；显式 `null` 表示未确认，缺字段是格式错误。已声明但不支持的值必须原样保存，不退化为“已确认 / 未确认”：

| 字段 | 允许的非 null 值 |
| --- | --- |
| `orientation` | `vertical`、`horizontal` |
| `gasDirection` | `upward`、`other` |
| `service` | `cleanNonFoamingTwoPhase`、`singlePhase`、`foaming`、`solids`、`slugging`、`threePhase` |
| `geometry` | `fullEqualDiameterCylinder`、`reducedEffectiveArea`、`nonCylindrical` |
| `kBasis` | `effectiveWholeVesselForDeclaredInternals`、`mistEliminatorOnly`、`unmatchedInternals` |
| `flowBasis` | `actualAtOperatingConditions`、`standardConditions` |

另有必需字符串 `internalsDescription`，可空。10 月 11 日设计草案中的 `twoPhasesDeclared` / `configuration` 不能表达完整领域声明，未作为设备产品写出；检测到旧 `case.configuration` 结构时整份只读，不推断或自动迁移。方法 ID / 版本和数值算法保持不变。

字段绑定采用带标签的联合，字段集合严格如下；quantityId 在 Direct/Override 值处必需，不从单位猜量种：

```json
[
  {"kind":"direct","quantityId":"duration","valueSi":120.0,"sourceId":"src-assumption"},
  {"kind":"basisRef","basisEntryId":"basis-working-time"},
  {"kind":"override","basisEntryId":"basis-working-time","quantityId":"duration","valueSi":180.0,"sourceId":"src-override","reason":"独立工况明确覆盖"}
]
```

上述仅演示三种编码，不是可计算工程样本。BasisRef 只指基础条目，不能指其他绑定；Override 必须保留同量种的原基础引用、本地来源和非空理由，不改原基础值。来源日期 accessedOn 为空或 ISO 8601 日期（YYYY-MM-DD），publicationVersion 保留来源发布日期或版本原文；review 变更须推进源 revision。来源 kind 枚举 `measured/vendor/literature/estimated/assumed/unknown`；review 枚举 `unreviewed/adopted/reviewed`。分类与 review 不自动证明可信度；unknown 可保存但不允许计算。

`applicability` 包含必需 `fluidSystem`、`phase`、`operatingConditions`、`scope`，phase 为 `gas/liquid/both/notApplicable`；operatingConditions 为空对象表示未记录，或者含 `temperatureK` 与 `pressurePa` 两个有限正数，必须成对出现，不从 task 自动复制证明匹配。流量 / 密度要求同一物系与工况；准则用 scope 记录适用范围、内件与折减，无法机器确认的条件必须保留审阅说明。

`observations` 是原始量记录数组，每项含 `id`、`quantityId`、`originalValue`（有限数）、`originalUnit`（来源原文）、`canonicalValue`（有限 SI）、`conversionNote` 及 `dependsOn`。dependsOn 为 `{sourceId, observationId}` 引用数组，支持有限的可追溯派生链，禁止环；不是可执行公式或第二套自动换算引擎。originalUnit 可保留不支持的外部单位，但只有统一目录可换算的值或注明外部方法并显式采用的 canonicalValue 才能使用，不从文本执行表达式。缺 observation 不等于缺 source：假设可仅以 adoptionNote 说明，分类始终为 assumed。引用绑定以 sourceId 关联当前来源，计算时捕获具体源 revision 与观察记录，不把捕获写回成输入。

不保存计算 / 检查结果、候选、输入草稿、session ID、计算代次或执行 token。任务 / 来源持久 revision 不充当跨会话执行资格；重开建立新会话和运行时代次，设备候选 missing，显式重算。保存脏标记比较正式 document、presentation、engineering 各自已保存基线；仅设备变更不增加流程 revision。

## v3 结构样本与版本行为

以下是可直接放入 v3 外层的 **engineering 子树**，示范可保存的不完整独立任务；没有 T/P、流量、密度或 K，必须返回缺项而不能计算。它不属于 S0-B04，也不替代有来源的工程方法验证。构造完整文件时复制任一合法旧样本的 document，外层版本改为 3，presentation 保留原 7 类再添加单位契约中 8 类的 SI 项；这是测试构造说明，不授权改写现有用户工程。

```json
{
  "basisEntries": [],
  "sources": [],
  "tasks": [{
    "id": "equipment-1",
    "revision": 0,
    "tag": "V-101",
    "name": "待补依据的独立任务",
    "kind": "verticalGasLiquidSeparatorSizing",
    "method": {"id":"radishflow.separator.vertical-sb-working-volume","version":1},
    "case": {
      "id":"case-1", "name":"工况一", "purpose":"初步尺寸",
      "fluidSystem":"", "applicabilitySourceIds":[],
      "applicability": {
        "orientation":null, "gasDirection":null, "service":null,
        "geometry":null, "kBasis":null, "flowBasis":null,
        "internalsDescription":""
      }
    },
    "inputs": {}
  }]
}
```

未知外层版本拒绝打开且不写盘。已知 v3 内未知 task kind / method ID / method version 不按版本 1 解码其 payload：保留原始任务数据，提供只读检查并禁止计算。首批对此文件采用整个工程只读模式，避免不知道依赖关系却更改其基础 / 来源；原文件始终不覆写，不提供剥离未知任务的“兼容保存”。这属于未知能力保护，不能显示已恢复可编辑任务。已知方法的未知字段则为格式错误，防止拼写错误被当作扩展吞掉。

旧 CLI v1/v2 对所有 v3 文件在建模、求解或写入前返回不支持项目版本 / 能力，沿用其结构化错误通道；即使工程区缺省或为空也不降格消费。新设备协议另行审定。无工程区且新增 8 类均为 SI 的旧文件保持原 v1/v2 保存资格；首次有工程内容或新增非 SI 呈现时，显式确认 v3 升级 / 另存为 / 取消。已有 v3 即使清空任务也不自动降级。
