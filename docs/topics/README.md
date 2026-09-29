# 开发专题索引

更新时间：2026-09-29

## 用途

用途：作为 RadishFlow “总进度 + 一级轨道专题 + 二级功能专题”开发节奏的入口。

读者：需要判断下一步做什么、某个功能是否已进入范围、以及实现前应读哪些边界文档的开发者、用户、AI / Agent。

不包含：完整历史流水、逐日提交记录、具体代码实现细节和一次性讨论。

> [!IMPORTANT]
> 自 2026-09-12 起恢复正常开发和迭代。下表按专题组织已有能力与后续工作；Active 专题按范围持续迭代，Draft / Backlog 按优先级纳入计划，具体切片以 `docs/status/current.md` 为准。

2026-09-13 确立基础功能优先的近期顺序。[模拟平台长期规划](../architecture/simulation-platform.md) 统一维护组分、物性分析、画布、单元 / 反应、稳态 / 动态 / 瞬态、算法、报告和 API 的能力地图；未来能力进入实施时再建立对应专题，不把长期清单全部标为 Active。近期从 [Studio 基础功能切片](studio-main-workflow.md#基础功能完善切片) 推进。

后续明确的递归分块、间歇 / 半间歇、变量树式 API、COM 自动化及操作录制 / 回放同样由长期规划承载。gPROMS 为方程建模重点参考；APC 暂按先进过程控制领域规划，尚未指定同名产品。

2026-09-14 补充 [插件系统](../architecture/simulation-platform.md#插件系统)、[许可与授权分层](../architecture/auth-entitlement-architecture.md#软件许可产品授权与操作权限) 和下表公共 API 专题。方向已确认，运行时、具体许可政策与接口仍待实施切片决策，不改变基础功能优先级。

2026-09-16 确认单位系统、规格分析与确定性辅助、设备设计与校核三条主线，单位专题的 U1 已实现并进入 Active；其余新专题为 Draft / 规划已确认。UI 旧主稿需重审功能分区，见 [UI 重设计计划](../architecture/studio-ui-topic-plan.md)。B3-5 无界面建模已接通；近期顺序为 U2 / U3 单位输入输出→规格 / 推荐→首个可信设备任务。

本轮后续补充 [工程设计基础、工况与数据来源](engineering-basis-and-cases.md)。目标反算归建模求解，压力 / 公用工程与候选权衡归设备专题，工程检查 / 交付归结果专题；各切片见路线图，均未改变已实现能力声明。

2026-09-26 确认 [装置工程与数字孪生愿景](../architecture/simulation-platform.md#装置工程与数字孪生愿景)：同一工厂 / 装置工程关联多模型、工况与运行任务。目录 / 交付包归 [生命周期规划](project-lifecycle-storage.md#装置工程容器规划)，配置及场景归 [工程基础](engineering-basis-and-cases.md#装置配置与运行场景规划)，未来 DT0—DT4 依赖与退出依据归 [路线图](../radishflow-mvp-roadmap.md#装置工程与数字孪生的演进切片)。均为规划，现场接入与在线优化未实现，近期顺序保持。

## 组织原则

- `docs/status/current.md` 只回答当前维护状态、优先级、临时门禁、验证基线和下一步。
- 一级轨道专题回答一条开发主线的边界，例如 Studio 主路径、流程图建模、结果审阅、项目生命周期、CAPE-OPEN 适配层。
- 二级功能专题回答一个具体功能、单元、服务或页面怎么设计、分阶段推进、验收和验证，例如换热器、闪蒸罐、后端服务、后端 Web UI。
- `docs/devlogs/` 记录历史推进，不再承担“下一步怎么做”的职责。
- `docs/architecture/`、`docs/reference/`、`docs/capeopen/`、`docs/thermo/` 仍保存长期架构、字段语义和边界。
- 一个专题只有在范围、非目标、验收标准和最小验证都写清后，才进入代码实现。

## 一级轨道专题

| 专题 | 当前状态 | 专题目标 | 入口 |
| --- | --- | --- | --- |
| Studio 主工作台与空白项目建模主路径 | Active | 把普通空白项目从物性配置、建模、运行、结果审阅和保存 / 重开收束成可复现主路径 | `studio-main-workflow.md` |
| 项目物性基础与组分选择 | Active | 稳定内置 package、项目组分、保存 / 重开和运行请求之间的同一事实源 | `property-basis-and-components.md` |
| 流程图建模与求解闭环 | Active | 维护受控单元、连接、readiness、Run Panel 和 solver 之间的边界 | `flowsheet-modeling-and-solve.md` |
| 结果审阅、诊断与恢复 | Active | 结果与恢复保持同源；V1—V4 流向、着色、设备三维和回放为新增规划 | [结果与可视化](results-review-diagnostics.md) |
| 运行、消息与诊断 | Draft | 当前问题、执行事件、状态联动及 N0—N5 运行控制规划；已有恢复保持 | [运行与诊断](runtime-messages-and-diagnostics.md) |
| 项目生命周期与存储 | Active | 明确打开、保存、另存为、最近项目、sidecar 和脏改确认的长期边界 | `project-lifecycle-storage.md` |
| CAPE-OPEN PMC 适配层 | Active | 完善 `.NET 10` PMC 生命周期与调用可靠性，保持 COM 注册和 PME 验证基线 | `capeopen-pmc-adapter.md` |
| 工程设计基础、工况与数据来源 | Draft | 共享工程条件、来源 / 假设、工况矩阵、位号与复现依据 | [工程基础](engineering-basis-and-cases.md) |
| 单位系统与输入显示 | Active | U1、U2 保存 / 会话及首批控件 / 视图已接通；剩余交互、验收与 U3 待续 | [单位系统](units-and-quantity-system.md) |
| 颜色与状态系统 | Active | V0 首批数值字段已接通；设置 / 保存及其他消费者和运行态验收待续 | [颜色与状态](color-and-state-system.md) |
| 规格分析与建模辅助 | Draft | 分层规格 / 自由度检查、可解释规则与固定工作流、未来 Agent 入口 | [建模辅助](modeling-assistance-and-specifications.md) |
| 设备设计与性能校核 | Draft | 独立设备及流程关联，初步选型、设计、校核与采纳 | [设备工程](equipment-design-and-rating.md) |

UI 跨专题设计入口为 [Studio UI 计划](../architecture/studio-ui-topic-plan.md)，承载 R0—R2 功能分区重评与 Pencil 设计验收，不以画稿完成冒充代码实现。

## 二级功能专题

### Studio 与自动化

| 专题 | 当前状态 | 父专题与范围 | 入口 |
| --- | --- | --- | --- |
| 统一变量与对象浏览器 | Draft / 2026-09-28 方向已确认 | Studio 主工作流；工程视图、变量树、稳定引用、受控编辑和跨端消费，复用已有 B3 基础；VB0—VB3 待实施 | [变量与对象浏览器](variable-and-object-browser.md) |

### 运行、消息与诊断

父专题为 [运行、消息与诊断](runtime-messages-and-diagnostics.md)，下列均为 Draft / 待实施。详细规则按职责归位，需求与验证状态只在 [追踪表](runtime-messages/requirements-traceability.md) 维护。

| 二级专题 | 范围 | 入口 |
| --- | --- | --- |
| 消息模型与问题生命周期 | 分类、身份、重检 / 恢复、状态维度 | [消息模型](runtime-messages/message-model-and-lifecycle.md) |
| 执行追踪与收敛诊断 | 真实步骤、循环、观察与采集 | [执行追踪](runtime-messages/execution-trace-and-convergence.md) |
| 运行控制与条件监视 | 暂停、单步、取消、条件求值 | [运行控制](runtime-messages/run-control-and-watch-conditions.md) |
| 诊断工作台与多处联动 | 双区、定位、草稿、焦点与可访问性 | [工作台](runtime-messages/diagnostic-workbench-and-navigation.md) |
| 记录存储与跨端消费 | 证据、规则、偏好及协议兼容 | [记录与消费者](runtime-messages/records-storage-and-consumers.md) |
| 开发者诊断与模型验证 | 算法 / 模块 / 物性证据、对照实验与诊断包 | [开发者诊断](runtime-messages/developer-diagnostics-and-model-validation.md) |
| 场景与验收矩阵 | 可观察预期与分阶段验证 | [验收矩阵](runtime-messages/scenarios-and-acceptance.md) |

### 单位与颜色状态

[单位总纲](units-and-quantity-system.md) 下按四项职责维护，U1 / U2-I1 已实现，其余按追踪表标注；[颜色总纲](color-and-state-system.md) 下四个子专题均为 Draft。需求 / 场景 / 决策与证据分别由 [单位追踪](units/requirements-traceability.md) 和 [颜色追踪](color-state/requirements-traceability.md) 维护。

| 一级专题 | 二级职责入口 |
| --- | --- |
| 单位 | [目录与转换](units/catalog-and-conversion.md)、[单位集与保存](units/unit-sets-and-persistence.md)、[输入草稿与交互](units/input-drafts-and-interactions.md)、[输出与跨端一致性](units/output-and-consumer-consistency.md) |
| 颜色 / 状态 | [语义与组合](color-state/semantics-and-composition.md)、[控件 / 画布映射](color-state/control-and-canvas-mapping.md)、[主题与可访问性](color-state/theme-and-accessibility.md)、[迁移与验收](color-state/migration-and-acceptance.md) |

### 单元模块

| 专题 | 当前状态 | 父专题 | 入口 |
| --- | --- | --- | --- |
| Feed / 进料源 | Active | 流程图建模与求解闭环 | `unitops/feed-source.md` |
| Heater / Cooler 温压调节单元 | Active | 流程图建模与求解闭环 | `unitops/heater-cooler.md` |
| Flash Drum 闪蒸罐 | Active | 流程图建模与求解闭环 | `unitops/flash-drum.md` |
| Mixer 混合器 | Active | 流程图建模与求解闭环 | `unitops/mixer.md` |
| Valve 阀门 | Active | 流程图建模与求解闭环 | `unitops/valve.md` |

### 设备工程

| 专题 | 当前状态 | 父专题 | 入口 |
| --- | --- | --- | --- |
| 气液分离器尺寸与能力校核 | Draft | 设备设计与性能校核 | [分离器](equipment/separator-sizing-and-rating.md) |
| 换热器设计与性能校核 | Draft | 设备设计与性能校核 | [换热器](equipment/heat-exchanger-design-and-rating.md) |

### 建模对象

| 专题 | 当前状态 | 父专题 | 入口 |
| --- | --- | --- | --- |
| Material Stream 物流股 | Active | 流程图建模与求解闭环 / 结果审阅、诊断与恢复 | `modeling/material-stream.md` |
| 模型目录、发现与运行实例 | Draft / 2026-09-29 方向已确认 | 流程图建模与求解闭环 / 项目物性基础；MC0—MC3 待实施 | [模型装配](modeling/model-catalog-and-runtime.md) |

### 平台与服务

| 专题 | 当前状态 | 父专题 | 入口 |
| --- | --- | --- | --- |
| Control Plane 后端服务 | Backlog | 项目物性基础与组分选择 / 项目生命周期与存储 | `platform/control-plane-service.md` |
| Control Plane Web UI 后端管理台 | Backlog | Control Plane 后端服务 | `platform/control-plane-web-ui.md` |

### 未来规划（未激活）

| 专题 | 当前文档状态 | 父专题 | 入口 |
| --- | --- | --- | --- |
| 账户、登录与 Radish 联合身份 | Draft / 未排期 / 待架构决策 | Control Plane 后端服务 | [账户与联合登录](platform/account-and-federated-login.md) |
| 公共 API 与访问控制 | Draft / 规划方向已确认 / 未排期 | Control Plane 后端服务（远端访问控制） | [公共 API 与访问控制](platform/public-api-and-access-control.md) |

此处记录仍待范围与架构决策的规划；进入实施前先完成对应专题的前置决策。

## 专题状态定义

| 状态 | 含义 |
| --- | --- |
| Draft | 已建文档，具体切片尚未进入实现；可注明方向已确认，须完成切片设计与验收定义 |
| Active | 当前持续迭代的专题，具体切片由当前优先级确定 |
| Blocked | 已确认阻塞，等待决策、外部环境或前置专题 |
| Done | 约定范围内验收完成，后续扩展需更新范围与验收标准 |
| Frozen | 阶段性冻结，只修 blocker，不扩范围 |
| Backlog | 已记录但暂不推进 |

## 新增专题规则

已有问题优先更新所属专题；新增专题按当前优先级组织，不以文档建立代替实现或验收。

新增专题时优先复制 `topic-template.md`，并至少写清：

- 专题层级、父专题和子专题关系。
- 专题目标和用户路径。
- 当前实现快照和已知缺口。
- 本专题纳入和不纳入的范围。
- 数据 / 状态 / 命令 / UI 边界。
- 分阶段切片和退出标准。
- 最小验证计划。

不要把同一类内容同时写进多个专题。若一个改动跨专题，先在当前激活专题写主决策，再在相关专题补引用。

## 总进度关系

- 当前阶段总进度：`docs/status/current.md`
- MVP 总边界：`docs/mvp/scope.md`
- 第一阶段路线图：`docs/radishflow-mvp-roadmap.md`
- 周志索引：`docs/devlogs/README.md`
