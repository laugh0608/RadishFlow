# 消息系统需求追踪表

更新时间：2026-09-23

## 用途与维护

用途：保存消息大专题的需求、切片、验收、未决事项及实现证据之间的关联，防止跨阶段遗漏。
读者：规划、实现、评审与测试维护者。
不包含：规则正文的副本、冻结接口或本轮产品能力交付声明。

父专题：[运行、消息与诊断](../runtime-messages-and-diagnostics.md)。本表为配套追踪，不是第七个功能子专题；场景正文见 [验收矩阵](scenarios-and-acceptance.md)。

MSG 编号稳定且不复用；删除 / 替代要求保留去向，拆分保留来源。此处是实施 / 验证状态唯一真相，子专题维护规范，周志记录历史。新增代码或测试时关联要求与场景编号，可用测试名称 / 链接及提交证据，不要求在每个源文件散布编号。

- 设计状态分“方向已确认 / 细化待决策 / 细化已审定”；方向确认不等于接口已冻结。
- 实施状态分“未纳入切片 / 待实现 / 部分实现 / 已实现”；验证状态分“待验证 / 部分通过 / 已通过 / 失败”。三者分别记录。
- 当前所有 MSG 的设计方向已确认；带 D 依赖的细节仍待审定。所有 MSG 的新增范围均为待实现、待验证，代码 / 证据均为“—”。现有诊断等基础尚未逐项对齐，不能直接填已实现。
- 进入实施时给每项补充实施状态、验证状态、代码 / 测试路径、证据链接及日期。跨阶段逐段记录，不能用早期部分通过代表全项通过。代码行号变化应维护链接，不以周志中的旧结论替代当前状态。

## 要求与验收关联

下表“待实现 / 待验证；—”明确表示本次没有新增产品实现或验收证据；细化决策见后表。

| 编号 | 要求 / 规范归属 | 切片 | 场景 | 实施 / 验证；代码与证据 |
| --- | --- | --- | --- | --- |
| MSG-01 | [消息分类与来源](message-model-and-lifecycle.md) | N0 / N1 | S-01 | 待实现 / 待验证；— |
| MSG-02 | [身份、关联与本地化](message-model-and-lifecycle.md) | N1 / N2 | S-02 | 待实现 / 待验证；— |
| MSG-03 | [失效、重检与生命周期](message-model-and-lifecycle.md) | N1 | S-03 | 待实现 / 待验证；— |
| MSG-04 | [恢复与终止错误](message-model-and-lifecycle.md) | N1 / N2 | S-04 | 待实现 / 待验证；— |
| MSG-05 | [过滤与连锁聚合](message-model-and-lifecycle.md) | N1 | S-05 | 待实现 / 待验证；— |
| MSG-06 | [状态维度与结果资格](message-model-and-lifecycle.md) | N0 / N1 / N3 | S-06 | 待实现 / 待验证；— |
| MSG-07 | [真实步骤与失败证据](execution-trace-and-convergence.md) | N2 | S-07 | 待实现 / 待验证；— |
| MSG-08 | [层级、顺序与能力](execution-trace-and-convergence.md) | N2 / N4 | S-08 | 待实现 / 待验证；— |
| MSG-09 | [残差与观察值解释](execution-trace-and-convergence.md) | N4 | S-09 | 待实现 / 待验证；— |
| MSG-10 | [采集、容量与性能](execution-trace-and-convergence.md) | N2 | S-10 | 待实现 / 待验证；— |
| MSG-11 | [安全暂停与能力](run-control-and-watch-conditions.md) | N3 | S-11 | 待实现 / 待验证；— |
| MSG-12 | [单步、继续与取消](run-control-and-watch-conditions.md) | N3 | S-12 | 待实现 / 待验证；— |
| MSG-13 | [编辑、退出与迟到隔离](run-control-and-watch-conditions.md) | N3 | S-13 | 待实现 / 待验证；— |
| MSG-14 | [条件量与单位](run-control-and-watch-conditions.md) | N4 | S-14 | 待实现 / 待验证；— |
| MSG-15 | [无效条件与缺值](run-control-and-watch-conditions.md) | N4 | S-15 | 待实现 / 待验证；— |
| MSG-16 | [求值、重新武装与采样](run-control-and-watch-conditions.md) | N4 | S-16 | 待实现 / 待验证；— |
| MSG-17 | [动态、回放与检查点](run-control-and-watch-conditions.md) | N5 | S-17 | 待实现 / 待验证；— |
| MSG-18 | [双区与多处联动](diagnostic-workbench-and-navigation.md) | N0 / N1 / N2 | S-18 | 待实现 / 待验证；— |
| MSG-19 | [定位、修复与草稿](diagnostic-workbench-and-navigation.md) | N1 | S-19 | 待实现 / 待验证；— |
| MSG-20 | [滚动、焦点与密集更新](diagnostic-workbench-and-navigation.md) | N1 / N2 | S-20 | 待实现 / 待验证；— |
| MSG-21 | [可访问性与分层信息](diagnostic-workbench-and-navigation.md) | N0 / N1 | S-21 | 待实现 / 待验证；— |
| MSG-22 | [空状态与上下文](diagnostic-workbench-and-navigation.md) | N0 / N1 / N2 | S-22 | 待实现 / 待验证；— |
| MSG-23 | [记录、规则与偏好所有权](records-storage-and-consumers.md) | N2 / N3 / N4 | S-23 | 待实现 / 待验证；— |
| MSG-24 | [旧工程与未知规则](records-storage-and-consumers.md) | N3 / N4 | S-24 | 待实现 / 待验证；— |
| MSG-25 | [重开、保留与损坏记录](records-storage-and-consumers.md) | N2 / N3 | S-25 | 待实现 / 待验证；— |
| MSG-26 | [CLI 与跨端语义](records-storage-and-consumers.md) | N1 / N2 / 后续协议 | S-26 | 待实现 / 待验证；— |
| MSG-27 | [导出范围与复现依据](records-storage-and-consumers.md) | N2 / 后续导出 | S-27 | 待实现 / 待验证；— |

## 待决策清单

以下均为“细化待决策”；须在标注范围进入实现前给出结论并回写所属子专题，不能以这个清单代替具体设计。负责方指领域职责，不指未获指派的人员。

| 编号 | 决策 / 规范归属 | 受影响要求 | 最迟进入时机 / 负责方 |
| --- | --- | --- | --- |
| D-01 | [问题键、失效、重检与阻断](message-model-and-lifecycle.md#切片待决策与最小验证) | MSG-01—06 | N1 前 / 检查与应用状态 |
| D-02 | [事件协议、顺序、采集及容量](execution-trace-and-convergence.md#切片待决策与最小验证) | MSG-07—10 | N2 前 / 求解执行与运行记录 |
| D-03 | [残差及收敛观察判据](execution-trace-and-convergence.md#切片待决策与最小验证) | MSG-08—09、16 | N4 前 / 数值求解 |
| D-04 | [控制协议与编辑 / 关闭竞争](run-control-and-watch-conditions.md#切片待决策与最小验证) | MSG-11—13 | N3 前 / 应用任务与执行者 |
| D-05 | [条件有效性、精度与触发](run-control-and-watch-conditions.md#切片待决策与最小验证) | MSG-14—16 | N4 前 / 运行控制与单位 |
| D-06 | [时间、回放和检查点](run-control-and-watch-conditions.md#切片待决策与最小验证) | MSG-17 | N5 前 / 动态与运行恢复 |
| D-07 | [布局、空状态和操作路径](diagnostic-workbench-and-navigation.md#切片待决策与最小验证) | MSG-18—22 | N0 局部画板实施前 / Studio 设计；后续实现再复核 |
| D-08 | [记录持久化与规则兼容](records-storage-and-consumers.md#切片待决策与最小验证) | MSG-23—25、27 | N2 持久化前，N3 / N4 规则保存前 / 存储与运行记录 |
| D-09 | [外部协议与能力版本](records-storage-and-consumers.md#切片待决策与最小验证) | MSG-26—27 | 新协议实施前 / API 与适配层 |
| D-10 | [验证载荷、指标与平台](scenarios-and-acceptance.md#切片评审与退出) | 全部，尤其 MSG-10、20—21 | 各切片实施前 / 实现与验证 |

## 覆盖维护与检查

- 每项 MSG 必须有一个规范归属、实施切片和至少一个 S 场景；新失败模式先补场景，再决定是否需要新增要求。
- 每个待决策 D 都应有负责领域、受影响要求和最迟进入时机；结论形成后保留编号与出处，不删去决定过程的索引。
- 提交前核对编号唯一、场景双向对应、规范及证据链接有效，并检查父专题和路线图的阶段一致性。
- 当前 N0 对应的语义与设计场景为 S-01、S-06、S-18、S-21、S-22；这些仅做设计评审，产品行为仍在所属阶段验证。
