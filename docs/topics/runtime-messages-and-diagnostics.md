# 运行、消息与诊断

更新时间：2026-09-23

## 用途与状态

用途：定义当前问题、计算事件、运行控制和多处状态展示的统一契约及分阶段验收。
读者：维护求解器、应用任务、Studio、自动化、设备工程与诊断的协作者。
不包含：已交付后台求解或调试器的声明、具体持久化 schema、任意脚本执行、工程算法、完整报告及远端审计服务。

- 层级：一级轨道；上位为 [平台规划](../architecture/simulation-platform.md)，阶段顺序见 [路线图](../radishflow-mvp-roadmap.md#运行消息与诊断的配套切片)。
- 状态：Draft / 2026-09-23 方案已确认；N0 设计衔接 U2 / V0 / R0—R1，新增运行能力尚未实现。
- 本专题拥有消息生命周期、执行追踪、停止语义与跨消费者一致性；[结果专题](results-review-diagnostics.md) 拥有结果有效性、工程检查与输出，[辅助专题](modeling-assistance-and-specifications.md) 拥有规则 / 建议，[颜色与状态专题](color-and-state-system.md) 拥有呈现映射，[UI 计划](../architecture/studio-ui-topic-plan.md) 拥有布局与画板。

## 目标与当前基础

用户在建模、运行和审阅时能回答：缺什么、正在算什么、哪里有问题、结果能否使用、下一步能做什么。通过同一问题记录连接全局摘要、画布图标、模块面板、问题列表与计算过程，不在各页面建立独立错误状态。

当前可复用：`rf-ui::diagnostics` 的严重程度 / 代码 / 对象定位，`RunPanel` 的 recovery action，`SolveSnapshot` / `StepSnapshot` 的步骤与修订，变量 / 动作元数据、单位目录及正式事务。现有 `AppLogEntry` 主要为级别与文本，尚非完整运行事件；`WorkspaceSolveService` 仍同步求解，现有 `Active / Hold` 不表示可暂停在途求解。现有失败恢复与快照门禁继续有效，不因本规划宣称已经有后台任务、循环 trace、实时断点或部分结果查询。

## 子专题与按任务阅读

本专题是大专题总纲；规则正文只在下表所属子专题维护，其他专题引用，避免多份状态机与单位规则。开始实现时先读所属子专题、对应 MSG 要求 / S 场景与未决 D 项，再确定接口；不能只读本总纲就进入编码。

| 二级专题 | 唯一维护内容 | 主要切片 |
| --- | --- | --- |
| [消息模型与问题生命周期](runtime-messages/message-model-and-lifecycle.md) | 分类、身份、生命周期、状态维度与聚合 | N0 / N1 |
| [执行追踪与收敛诊断](runtime-messages/execution-trace-and-convergence.md) | 步骤、循环、观察值、采集与容量 | N2 / N4 |
| [运行控制与条件监视](runtime-messages/run-control-and-watch-conditions.md) | 暂停、单步、取消、阈值与安全边界 | N3—N5 |
| [诊断工作台与多处联动](runtime-messages/diagnostic-workbench-and-navigation.md) | 用户路径、定位、草稿、滚动与可访问性 | N0—N4 随能力 |
| [记录存储与跨端消费](runtime-messages/records-storage-and-consumers.md) | 配置、证据、呈现偏好、保存与接口兼容 | N1—N5 随能力 |
| [开发者诊断与模型验证](runtime-messages/developer-diagnostics-and-model-validation.md) | 算法 / 模块 / 物性 / 插件证据、模型实验与诊断包 | N0 / N1 / N2 / N4 随模型能力 |
| [场景与验收矩阵](runtime-messages/scenarios-and-acceptance.md) | 独立预期、回归与验收方法 | 所有切片 |

[需求追踪表](runtime-messages/requirements-traceability.md) 是实施与验证状态的唯一清单，关联 MSG 要求、所属专题、N 切片、S 场景和证据；D 项标明细化决策的最后进入时机。它不另写一份规范正文。

用户主路径为：发现缺项→定位修正→重检运行→观察过程→审阅结果；未来增加条件命中→安全暂停→检查→继续 / 取消。当前问题与历史事件、暂停请求与已暂停、观察值与正式结果分别表达。监视只观察和请求控制，不暗改物理模型。

单位系数与量语义归单位专题；颜色 token 归颜色与状态专题；结果有效性归结果专题；规则算法归辅助与工程领域。运行层提供事实和能力，UI、CLI 和未来 Agent 共用语义。现有同步求解、CLI SI 与最终响应保持，Rust Core 不引入 COM。

## 切片与退出标准

| 切片 | 范围与时机 | 退出标准 |
| --- | --- | --- |
| N0 统一语义与局部设计 | 随 U2 / V0 / R0—R1 | 分类、身份、生命周期、状态组合及双区路径评审清楚；不要求完整调试器作为 U2 前置 |
| N1 当前问题闭环 | 与 A1 / A2 衔接，复用现有检查和恢复 | 全局 / 画布 / 模块 / 列表同源、去重、筛选定位、修订失效和动作重验可验证 |
| N2 运行过程记录 | 先真实顺序步骤的运行后审阅，再按执行能力完善实时事件 | 成功 / 失败均可追踪，有运行身份、分级采集和容量边界，旧记录不污染新状态 |
| N3 运行控制 | 后台任务、取消协议及安全边界建立后 | 请求 / 已生效分开，模块前后暂停、单步、继续 / 取消、输入变更与迟到结果处理通过 |
| N4 条件与数值观察 | 依赖 N2 / N3、变量及实际循环模型 | 条件时机 / 单位 / 无效数据清楚，嵌套 trace、残差判据与采样独立；运行比较有输入身份 |
| N5 动态与高级调试 | P5 / P6 提供真实时间、事件和相应恢复能力后 | 仿真时间、播放与运行控制分离，事件 / 检查点兼容验证，黑箱粒度明确 |

工程顺序保持 U2→U3→规格 / 推荐→可信设备；N0 当前纳入设计，N1 随规格基础，N2—N5 按运行与模型依赖安排。完整 N 轨道不阻塞单位与设备基础交付，也不以统一 UI 为理由提前改变求解器。

开发者诊断与工程视图共享事实；按任务展开模型、数值及技术详情，不因隐藏详情解除工程阻断。独立模型 / 物性实验和调用证据可先于后台调试，参数对照与导数检查随模型能力建设；MSG-28—33 的字段与载荷由 D-11 / D-12 衔接既有采集、数值及存储决策。

## 实施与验收入口

切片启动先完成追踪表中适用的 D 决策、范围和独立预期，实施后回填代码 / 测试位置与证据。新增方向均未获得运行态验收；原有能力仅作为复用基础。文档组织完成不等于 N0 UI 评审通过，也不改变 U2 优先级。

## 参考与适用范围

2026-09-23 调研公开资料，未安装实测商业软件；Aspen / HYSYS 部分为历史手册，借鉴概念而非声明最新版本布局或复制内部实现。

- [Aspen Plus 用户指南，第 11 章](https://web.tecnico.ulisboa.pt/mcasquilho/acad/Aspen/AspUserGuide10.pdf)：模块执行、前后停止点及结果正常 / 警告 / 错误 / 输入变化的区分。
- [HYSYS 用户指南，1.3 节](https://sites.ualberta.ca/CMENG/che312/F06ChE416/HysysDocs/AspenHYSYSUserGuide.pdf)：Object Status 与 Trace 的分工及对象定位。
- [CHEMCAD 用户指南](https://chemstations.com/hubfs/Datacor-Chemstations/PDF/CHEMCAD_User_Guide.pdf?hsLang=en)：消息分类与输入更改后的收敛状态更新。
- [COMSOL 求解日志设置](https://doc.comsol.com/6.3/doc/com.comsol.help.comsol/comsol_ref_solver.36.159.html)：日志详细度、嵌套求解信息与采样开销。
- [用户提供的 Aspen 案例](https://bbs.imbhj.com/t/topic/364)：警告时停止后查详细日志；数值阈值位于 Fortran 模型逻辑，不能据此认定任意阈值都是停止点菜单的原生功能。此案例只作场景依据，不采用无物理依据的数值截断作为通用修复。
- [NN/g 错误消息指南](https://www.nngroup.com/articles/error-message-guidelines/) 与 [W3C 状态消息说明](https://www.w3.org/WAI/WCAG21/Understanding/status-messages)：具体、可行动、不无故夺取焦点；Rust 桌面端通过原生可访问性映射，不照搬网页属性。
