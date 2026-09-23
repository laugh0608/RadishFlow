# 颜色与状态系统需求追踪表

更新时间：2026-09-23

## 用途与维护

用途：关联 VIS 需求、VA 验收、VD 决策与迁移证据，避免把规划或已有局部样式当统一交付。
读者：规划、设计、实现与验证维护者。
不包含：重复规范、冻结接口或产品验收声明。

父专题：[颜色与状态系统](../color-and-state-system.md)。VIS / VA / VD 编号稳定且不复用，替代保留去向。需求正文只在规范归属维护，状态与证据在本表维护，历史记录归周志。

设计方向已确认，关联 VD 的细节待决策；全部新增 V0 / 跨消费者范围待实现、待验证，现有控件仅为复用基础。VIS-12 的文档冲突已修正，但新布局画板 / 产品仍待评审和验证，不能据此标全项完成。后续按消费者和阶段分别记录实施、验证、代码 / 测试、日期与证据。

## 要求与场景

| 编号 | 要求 / 规范归属 | 切片 | 场景 | 实施 / 验证；证据 |
| --- | --- | --- | --- | --- |
| VIS-01 | [状态事实与所有权](semantics-and-composition.md) | V0 / 随领域 | [VA-01](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |
| VIS-02 | [完整性、草稿与独立保存](semantics-and-composition.md) | V0 / U2 | [VA-02](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |
| VIS-03 | [运行、结果与检查的作用域](semantics-and-composition.md) | V0 / 随运行能力 | [VA-03](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |
| VIS-04 | [诊断、来源与焦点叠加](semantics-and-composition.md) | V0 | [VA-04](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |
| VIS-05 | [字段与表格映射](control-and-canvas-mapping.md) | U2 / U3 | [VA-05](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |
| VIS-06 | [画布映射与数据着色分离](control-and-canvas-mapping.md) | V0；后续 V2 | [VA-06](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |
| VIS-07 | [消息身份与应用通知映射](control-and-canvas-mapping.md) | V0 / N1 | [VA-07](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |
| VIS-08 | [token 与主题一致性](theme-and-accessibility.md) | V0 / 随主题 | [VA-08](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |
| VIS-09 | [非颜色表达与对比](theme-and-accessibility.md) | V0 | [VA-09](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |
| VIS-10 | [键盘、文本与减少动画](theme-and-accessibility.md) | V0；后续动画 | [VA-10](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |
| VIS-11 | [分批迁移与门禁保持](migration-and-acceptance.md) | U2 / U3 / 辅助 | [VA-11](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |
| VIS-12 | [按工程任务分配主区域](migration-and-acceptance.md) | R0 / R1 | [VA-12](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |

## 待决策清单

以下均为细化待决策；2026-09-23 已形成首批映射、主题与测量、迁移样本候选，接口、画板及原生验证未审定。确认结论回写规范，保留编号和依据。领域状态仍由所属领域决定，不能以 VD 决策另建求解 / 消息状态机。

| 编号 | 决策 / 规范归属 | 关联要求 | 最迟时机 / 负责领域 |
| --- | --- | --- | --- |
| VD-01 | [状态映射接口、身份 / 修订一致性、未知 / 不适用与不一致反馈](control-and-canvas-mapping.md#u2-首批映射候选待审定) | VIS-01、02、03、04、07 | V0 首批接入前；领域状态与 Studio；后续运行能力接入再复核 |
| VD-02 | [角色槽位、图标 / 线型、窄布局压缩及图层叠加](control-and-canvas-mapping.md#u2-首批映射候选待审定) | VIS-04、05、06、12 | R1 对应画板与消费者实施前；Studio / Canvas / 设计 |
| VD-03 | [token 色值、支持主题、对比目标、测量及原生可访问性映射](theme-and-accessibility.md#v0-首批主题候选待审定) | VIS-08、09、10 | R1 主题与首批控件实施前；主题 / 可访问性 / 验证 |
| VD-04 | [消费者盘点、共享映射边界、旧新并存与完成判据](control-and-canvas-mapping.md#u2-首批映射候选待审定) | VIS-05、06、07、11 | U2 首批迁移前；U3 / 辅助批次复核；各消费者维护方 |
| VD-05 | [样例数据、测量记录、主题 / 语言 / 窗口 / 平台样本](migration-and-acceptance.md#退出与证据) | VIS-01—12 | 各批次实施 / 验收前；设计与验证 |

## 维护检查

每个 VIS 至少有一个 VA，场景反向注明要求；每个 VD 有规范归属、受影响范围及最后时机。V0 与 [单位事件](../units/input-drafts-and-interactions.md#u2-事件表)、[消息状态](../runtime-messages/message-model-and-lifecycle.md) 交叉核对，但不复制其事务或生命周期。文档检查证据不能替代画板、对比测量和原生运行验证。
