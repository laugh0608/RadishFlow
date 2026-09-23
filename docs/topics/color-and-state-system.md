# 颜色与状态系统

更新时间：2026-09-23

## 用途与状态

用途：作为一级专题统筹状态呈现、组合、控件 / 画布映射、主题及迁移验收。
读者：工程领域、应用状态、Studio、视觉设计与测试维护者。
不包含：第二套领域状态机、求解 / 输出门禁、具体色值、已实现的统一主题或产品验收声明。

- 层级：一级轨道；关联 [平台](../architecture/simulation-platform.md)、[单位](units-and-quantity-system.md)、[消息](runtime-messages-and-diagnostics.md)、[结果](results-review-diagnostics.md) 与 [UI 计划](../architecture/studio-ui-topic-plan.md)。
- 状态：Draft / V0 方向已确认，统一映射、控件迁移与可访问性验收待实施；现有控件与诊断只作为复用基础，不冒充 V0 已验收。
- 时机：V0 随 U2 / N0 / R0—R1 设计，消费者随 U2 / U3 / 规格辅助逐步迁移；不改变 U2→U3→规格辅助→设备工程的顺序。

## 目标与职责

用户应能分辨值从哪里来、是否可编辑、是否已提交、结果是否当前、哪里有问题以及哪些检查尚未进行。相同事实跨视图保持同义，状态叠加不隐藏信息，非彩色和键盘路径仍可使用。

状态事实由变量 / 规格、草稿事务、readiness、求解执行、结果资格、消息生命周期和项目保存等所属领域提供。颜色层只将这些事实映射为角色、图标、文字、线型与主题 token；不根据数值是否存在、日志最高级别或像素颜色推断状态，不新增独立的“可运行 / 可导出”判定。

| 子专题 | 唯一维护内容 |
| --- | --- |
| [状态语义与组合](color-state/semantics-and-composition.md) | 事实所有者、独立维度、V0 叠加规则与冲突呈现 |
| [控件与画布映射](color-state/control-and-canvas-mapping.md) | 字段、表格、节点 / 流股 / 端口、消息与应用通知的呈现角色 |
| [主题与可访问性](color-state/theme-and-accessibility.md) | 语义 token、默认颜色方向、非颜色表达与主题验证 |
| [迁移与验收](color-state/migration-and-acceptance.md) | 分批接入、代表场景、设计与产品退出证据 |

[需求追踪表](color-state/requirements-traceability.md) 维护 VIS 要求、VA 场景、VD 待决策及实施 / 验证状态。规则只在所属子专题维护；[Studio 视觉系统](../architecture/studio-visual-system.md) 继续拥有视觉定位、排版、密度和绘制风格，UI 计划拥有布局与画板。

## 阶段与退出

V0 的设计退出要求首批温压流量字段、跨视图代表组合、主题与非颜色表达完成局部评审，并明确适用 VD 决策。消费者的实现退出要求按已审定范围逐项测试 / 实窗验证，设计文档完成不是迁移完成。2026-09-23 首批方案已获接受，[局部画板](../architecture/designs/studio-client-main-brief.md#本轮画板与静态复核) 已静态复核并获项目所有者认可；[呈现接口提案](color-state/control-and-canvas-mapping.md#v0-呈现接口提案待审定) 待审定，产品仍未实施。

V1 流向、V2 数据着色、V3 设备三维和 V4 真实动态回放继续归 [结果可视化](results-review-diagnostics.md#流程可视化与回放规划尚未实现)，阶段顺序见 [路线图](../radishflow-mvp-roadmap.md#颜色与流程可视化的配套切片)。后续图层消费本专题的状态叠加与主题规则，不在本轮增设图形引擎、依赖或 schema。
