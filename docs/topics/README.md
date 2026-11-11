# 开发专题索引

更新时间：2026-11-11

## 用途

用途：连接整体规划、大专题（一级轨道）、子专题（二级功能）和任务 / 验收来源。
读者：需要了解能力边界、近期范围与状态来源的项目所有者、开发者及 AI / Agent。
不包含：任务状态副本、历史批次流水、完整设计正文或发布承诺。

## 组织原则

- 整体目标与能力地图见[模拟平台长期规划](../architecture/simulation-platform.md)，MVP 范围见[scope](../mvp/scope.md)，阶段、依赖与出口见[路线图](../radishflow-mvp-roadmap.md)。
- 大专题定义能力边界、权威设计、依赖、子专题清单和目标版本完成条件；子专题定义具体功能或工作包的场景、设计、失败行为与分期交付。
- 阶段与专题分别管理；AF、QS、U、N 等既有阶段 / 切片 / 要求编号保留，不因整理重新编号。每个子专题只有一个主要父专题，其余是协同关系。
- 子专题内部可分切片；复杂切片的附属文档继续保留，不另建递归专题层级。小子专题可由大专题内的稳定章节承载，远期只登记范围与启动条件。
- 日常先读[current](../status/current.md)，再读活跃子专题、任务状态来源与验收；全局阅读由整体规划进入本索引。设计正文仍在现有 topics、architecture、reference、capeopen、thermo 中维护。
- 统一规则、最小内容与收口要求见[协作规则](../development/agent-collaboration.md#三级规划与进度组织)，写作入口见[专题模板](topic-template.md)。

## 一级轨道专题

本表列能力与来源，不另维护 Active、Draft 或“首批已实现”快照。专题文档的完成声明须限定版本 / 切片；详细任务状态按下方归属表读取。

| 大专题 / 正式入口 | 能力边界 | 子专题或执行入口 |
| --- | --- | --- |
| [Studio 主工作台](studio-main-workflow.md) | 空白项目建模、运行、审阅、保存重开的用户路径 | 本文内基础功能切片；[变量与对象浏览器](variable-and-object-browser.md) |
| [物性基础与组分](property-basis-and-components.md) | package、组分与物性上下文 | 本文内切片；模型装配为协同 |
| [流程图建模与求解](flowsheet-modeling-and-solve.md) | 建模、连接、运行资格与求解 | 单元、物流股、模型装配、多策略求解 |
| [结果审阅与恢复](results-review-diagnostics.md) | 结果有效性、诊断恢复与可视化 | 本文内切片；与运行诊断及物流股协同 |
| [运行、消息与诊断](runtime-messages-and-diagnostics.md) | N0—N5 的运行控制、消息与诊断边界 | 下列七项职责子专题 |
| [项目生命周期与存储](project-lifecycle-storage.md) | 打开、保存、最近项目、脏改确认与工程容器 | 本文内切片；与控制面协同 |
| [CAPE-OPEN PMC](capeopen-pmc-adapter.md) | .NET / COM PMC 生命周期、宿主与调用边界 | 本文及现有 PME 验收入口 |
| [工程基础与工况](engineering-basis-and-cases.md) | 条件、来源、工况与工程证据 | 本文内 G0—G2 等切片；与设备工程协同 |
| [单位系统](units-and-quantity-system.md) | 量目录、单位集、输入输出一致性 | 下列四项职责子专题与 UNIT / UA 追踪 |
| [颜色与状态](color-and-state-system.md) | 语义、组合、消费者映射与迁移 | 下列四项职责子专题与 VIS / VA 追踪 |
| [规格与建模辅助](modeling-assistance-and-specifications.md) | 规格检查、可解释辅助与固定工作流 | 本文内 A1 / A2 等切片 |
| [设备设计与校核](equipment-design-and-rating.md) | 独立设备设计、校核与工程关联 | 分离器、换热器 |
| [Control Plane 后端服务](platform/control-plane-service.md) | 身份消费、授权、租约与资产分发 | Web 管理台、账户、公共 API；尚未进入近期实施 |

Control Plane 复用原文档作为大专题入口，原物性 / 生命周期关系明确为协同，其子专题范围与未排期边界不变。UI 跨专题设计仍由 [Studio UI 计划](../architecture/studio-ui-topic-plan.md)承载；设计验收不能替代功能验收。

## 二级功能专题

### Studio 与自动化

| 子专题 | 主要父专题 | 协同关系 / 状态来源 |
| --- | --- | --- |
| [变量与对象浏览器](variable-and-object-browser.md) | Studio 主工作台 | 物性、生命周期、结果与统一 API；AF4 / AF5a / VB 范围及证据在该专题；VB1 只读工作区首片已接通；路径、持久关注及跨平台原生保留 |

### 运行、消息与诊断

主要父专题为[运行、消息与诊断](runtime-messages-and-diagnostics.md)；下表规定设计归属，MSG 要求、S 场景、D 决策和执行 / 验证状态唯一维护在[追踪表](runtime-messages/requirements-traceability.md)。父专题给出 N0—N5 顺序与依赖。

| 子专题 | 范围 |
| --- | --- |
| [消息模型](runtime-messages/message-model-and-lifecycle.md) | 分类、身份、重检 / 恢复、状态维度 |
| [执行追踪](runtime-messages/execution-trace-and-convergence.md) | 真实步骤、循环、观察与采集 |
| [运行控制](runtime-messages/run-control-and-watch-conditions.md) | 暂停、单步、取消与条件求值 |
| [诊断工作台](runtime-messages/diagnostic-workbench-and-navigation.md) | 双区、定位、草稿、焦点与可访问性 |
| [记录与消费者](runtime-messages/records-storage-and-consumers.md) | 证据、规则、偏好及协议兼容 |
| [开发者诊断](runtime-messages/developer-diagnostics-and-model-validation.md) | 算法 / 模块 / 物性证据与诊断包 |
| [场景与验收](runtime-messages/scenarios-and-acceptance.md) | 可观察预期与分阶段标准；结果仍归追踪表 |

### 单位与颜色状态

| 主要父专题 | 子专题设计入口 | 唯一逐项状态与证据 |
| --- | --- | --- |
| [单位系统](units-and-quantity-system.md) | [目录](units/catalog-and-conversion.md)、[单位集](units/unit-sets-and-persistence.md)、[输入](units/input-drafts-and-interactions.md)、[输出](units/output-and-consumer-consistency.md) | [UNIT / UA / UD 追踪](units/requirements-traceability.md) |
| [颜色与状态](color-and-state-system.md) | [语义](color-state/semantics-and-composition.md)、[控件映射](color-state/control-and-canvas-mapping.md)、[主题](color-state/theme-and-accessibility.md)、[迁移](color-state/migration-and-acceptance.md) | [VIS / VA / VD 追踪](color-state/requirements-traceability.md) |

### 单元模块

| 子专题 | 主要父专题 | 状态与验收来源 |
| --- | --- | --- |
| [Feed](unitops/feed-source.md) | 流程图建模与求解 | 子专题的实现快照、切片与验收 |
| [Heater / Cooler](unitops/heater-cooler.md) | 流程图建模与求解 | 同上 |
| [Flash Drum](unitops/flash-drum.md) | 流程图建模与求解 | 同上 |
| [Mixer](unitops/mixer.md) | 流程图建模与求解 | 同上 |
| [Valve](unitops/valve.md) | 流程图建模与求解 | 同上 |

### 设备工程

| 子专题 | 主要父专题 | 协同关系 / 状态来源 |
| --- | --- | --- |
| [分离器尺寸与校核](equipment/separator-sizing-and-rating.md) | 设备设计与性能校核 | 工程基础、物性、单位；S / C 交付范围、待验项与证据在该专题 |
| [换热器设计与校核](equipment/heat-exchanger-design-and-rating.md) | 设备设计与性能校核 | 工程基础、物性、单位；范围、前置条件与验收在该专题 |

### 建模对象

| 子专题 | 主要父专题 | 协同关系 / 状态来源 |
| --- | --- | --- |
| [Material Stream](modeling/material-stream.md) | 流程图建模与求解 | 结果审阅、物性基础；实现与验收在该专题 |
| [模型目录与运行实例](modeling/model-catalog-and-runtime.md) | 流程图建模与求解 | 物性基础；AF2 / MC 范围与证据在该专题 |
| [多策略求解与装配](modeling/solver-strategies-and-assembly.md) | 流程图建模与求解 | 模型目录、物性、结果；QS 依赖和范围由本篇统筹 |

多策略求解内部的以下文档承载实施切片，不新增专题层级；其“父专题”指所属二级功能。切片逐项状态归各自文档，多策略总纲和路线图只引用有来源的摘要。

| 既有切片 | 设计、执行状态与验收来源 |
| --- | --- |
| QS0 | [多策略求解内的有限物料切片](modeling/solver-strategies-and-assembly.md) |
| QS1-A/B | [分块与回流核心](modeling/recycle-and-block-execution.md) |
| QS1-C | [回流工程与正式消费者](modeling/recycle-project-and-consumers.md) |
| QS2-A/B | [联立模块核心](modeling/simultaneous-modular-block.md) |
| QS2-C | [联立模块工程与正式消费者](modeling/simultaneous-project-and-consumers.md) |
| QS3 | [联立方程与规格反算](modeling/equation-oriented-block.md) |
| QS4 | [首个混合分区执行方案](modeling/mixed-partition-execution.md)；首片 A—D 已批准并完成本机验证，后续组合另审 |

### 平台与服务

| 子专题 | 主要父专题 | 协同关系 / 状态来源 |
| --- | --- | --- |
| [Web 管理台](platform/control-plane-web-ui.md) | Control Plane 后端服务 | UI、授权架构；范围、前置条件与验收在该专题 |

### 未来规划（未激活）

| 子专题 | 主要父专题 | 启动边界与状态来源 |
| --- | --- | --- |
| [账户与联合登录](platform/account-and-federated-login.md) | Control Plane 后端服务 | 未排期，架构决策及验收在该专题 |
| [公共 API 与访问控制](platform/public-api-and-access-control.md) | Control Plane 后端服务 | 方向已确认但未排期，协议 / 部署决策及验收在该专题 |

长期递归分块、动态 / 瞬态、间歇 / 半间歇、插件、COM 自动化、录制 / 回放和装置数字孪生继续由整体规划承载；不因索引整理提前激活。基础架构优先及当前停止线仍见 current。

## 专题状态定义

方案、执行、验证事实分别记录，统一语义见[协作规则](../development/agent-collaboration.md#三级规划与进度组织)。既有英文标记保留，不据此自动迁移完成状态：

| 标记 | 解释与完成边界 |
| --- | --- |
| Draft | 尚有待收敛设计；可能已有独立切片完成，须分别列出方案与执行事实 |
| Active | 持续迭代的能力主线，不表示其全部范围已实现 |
| Blocked | 指定范围受阻，记录原因与解除条件 |
| Done | 指定版本 / 切片满足验收，不代表未来扩展或全部平台完成 |
| Frozen | 已明确冻结，按原停止线处理，不能自动恢复实施 |
| Backlog | 已登记但未纳入近期执行；规划存在不代表已获实施授权 |

## 状态来源与验收归属

- 运行消息、单位、颜色的逐项任务 / 要求状态与证据归各自追踪表；子专题只拥有规则与验收场景，其他位置保留有链接的范围摘要。
- 其余专题的任务或切片状态归对应专题中的实施表 / 章节；独立附属切片文档已有状态时，以该文档为准，主专题不复制完整清单。
- 验收标准在对应正式专题 / 场景矩阵中维护，实际结果引用周志或验收记录；current 汇总活跃范围、阻塞与下一步，不另设任务表。
- 跨专题任务指定一个状态维护位置，其他参与方引用；同一要求与其实施任务可以不同编号，但应相互关联，不能把“任务结束”直接当作“要求验收通过”。
- 完成声明说明版本、切片、平台和未验证面。大专题按当前目标版本要求及跨专题主路径判断，不用文件数或任务勾选数计算完成率。

## 新增专题规则

优先沿用已有专题，按[模板](topic-template.md)补齐归属、场景、范围、设计依据、输入输出、正常 / 失败 / 恢复行为、切片、任务、状态来源与验收。大专题可引用既有设计，子专题可用稳定章节承载；不为远期清单创建空文件。

新增、调整或完成范围时，核对本索引、路线图对应关系、唯一状态来源、current 和实际证据；产品、接口或授权范围变化仍按既有协作规则处理。历史状态按时点解释，不把旧批次重新标成本次验收。

## 总进度关系

- 当前重点：[current](../status/current.md)。
- 总体能力：[模拟平台](../architecture/simulation-platform.md)；首版范围：[MVP](../mvp/scope.md)。
- 阶段与依赖：[路线图](../radishflow-mvp-roadmap.md)。
- 历史证据：[周志索引](../devlogs/README.md)。
