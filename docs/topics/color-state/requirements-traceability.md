# 颜色与状态系统需求追踪表

更新时间：2026-10-02

## 用途与维护

用途：关联 VIS 需求、VA 验收、VD 决策与迁移证据，避免把规划或已有局部样式当统一交付。
读者：规划、设计、实现与验证维护者。
不包含：重复规范、冻结接口或产品验收声明。

父专题：[颜色与状态系统](../color-and-state-system.md)。VIS / VA / VD 编号稳定且不复用，替代保留去向。需求正文只在规范归属维护，状态与证据在本表维护，历史记录归周志。

首批设计、映射与测量方向已于 2026-09-23 获接受，局部画板亦获认可；2026-09-28 首批数值字段已接通有限类型化接口、浅色角色与可访问性信息输出，合成事件回归通过。2026-10-02 设置 / 保存已接通共享角色，I5 首轮已补 macOS 键盘与原生可访问性树抽样，完整 VA 仍待验。VIS-12 的文档冲突已修正，但全局新布局 / 产品仍待评审和验证，不能据此标全项完成。后续按消费者和阶段分别记录实施、验证、代码 / 测试、日期与证据。

## 要求与场景

| 编号 | 要求 / 规范归属 | 切片 | 场景 | 实施 / 验证；证据 |
| --- | --- | --- | --- | --- |
| VIS-01 | [状态事实与所有权](semantics-and-composition.md) | V0 / 随领域 | [VA-01](migration-and-acceptance.md#验收场景) | 部分接入；首批数值来源为指定 / 继承 / 缺失，其他来源与消费者未覆盖；见下方证据 |
| VIS-02 | [完整性、草稿与独立保存](semantics-and-composition.md) | V0 / U2 | [VA-02](migration-and-acceptance.md#验收场景) | 部分接入；草稿 / 未完成与保存摘要独立；保存区 V0 角色已接通，完整原生场景待续；见下方证据 |
| VIS-03 | [运行、结果与检查的作用域](semantics-and-composition.md) | V0 / 随运行能力 | [VA-03](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |
| VIS-04 | [诊断、来源与焦点叠加](semantics-and-composition.md) | V0 | [VA-04](migration-and-acceptance.md#验收场景) | 部分接入；首批字段错误 / 来源 / 焦点叠加已补实绘对比与边界分离，完整组合待验；见下方证据 |
| VIS-05 | [字段与表格映射](control-and-canvas-mapping.md) | U2 / U3 | [VA-05](migration-and-acceptance.md#验收场景) | 首批输入字段已接入；变量 / 结果表待 U3；见下方证据 |
| VIS-06 | [画布映射与数据着色分离](control-and-canvas-mapping.md) | V0；后续 V2 | [VA-06](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |
| VIS-07 | [消息身份与应用通知映射](control-and-canvas-mapping.md) | V0 / N1 | [VA-07](migration-and-acceptance.md#验收场景) | 应用通知已接入类型化角色；全局问题身份 / 计数仍待 N1；见下方证据 |
| VIS-08 | [token 与主题一致性](theme-and-accessibility.md) | V0 / 随主题 | [VA-08](migration-and-acceptance.md#验收场景) | 局部浅色 token 已接入；全局主题一致性待验；见下方证据 |
| VIS-09 | [非颜色表达与对比](theme-and-accessibility.md) | V0 | [VA-09](migration-and-acceptance.md#验收场景) | 来源 / 草稿 / 错误及组合禁用说明已接入；首批实绘色对、双语灰度 / 三类色觉样例已补，完整消费者待验；见下方证据 |
| VIS-10 | [键盘、文本与减少动画](theme-and-accessibility.md) | V0；后续动画 | [VA-10](migration-and-acceptance.md#验收场景) | macOS 单位菜单、关闭焦点 / Escape、原生名称和系统工作区抽样通过；完整矩阵、读屏 / 精确缩放待验；见下方证据 |
| VIS-11 | [分批迁移与门禁保持](migration-and-acceptance.md) | U2 / U3 / 辅助 | [VA-11](migration-and-acceptance.md#验收场景) | 已记录首批数值迁移与未迁移范围；完整消费者门禁验收待续；见下方证据 |
| VIS-12 | [按工程任务分配主区域](migration-and-acceptance.md) | R0 / R1 | [VA-12](migration-and-acceptance.md#验收场景) | 待实现 / 待验证；— |

### 2026-09-28 首批实施证据

- [共享数值投影](../../../crates/rf-ui/src/state/numeric_edit/presentation.rs) 与 [原生数值控件](../../../apps/radishflow-studio/src/studio_gui_shell/numeric_input.rs) 支撑上述部分接入；名称 / 组成、设置 / 保存角色、结果 / 变量表、消息与画布未完成 V0 迁移。
- [领域回归](../../../crates/rf-ui/src/state/numeric_edit/tests.rs)、[控件合成事件](../../../apps/radishflow-studio/src/studio_gui_shell/tests/numeric_edits.rs) 与 [视图回归](../../../apps/radishflow-studio/src/studio_gui_shell/tests/view_units.rs) 覆盖单位、会话、历史和作用域；全仓 1,271 项通过记录见 [W40](../../devlogs/2026-09/2026-W40.md)。这些测试未证明实际颜色对比、原生可访问性桥接或读屏，全部完整 VA 仍待验收。

### 2026-10-02 设置 / 保存实施证据

- [共享角色与保存摘要](../../../apps/radishflow-studio/src/studio_gui_shell/state_presentation.rs) 消费 `ProjectSaveState` 和类型化通知，覆盖项目 / 视图单位设置、保存操作提示与关闭摘要。草稿、呈现保存和工程保存分别表达，警告 / 错误不由翻译后的标签推断。
- [离开确认回归](../../../apps/radishflow-studio/src/studio_gui_shell/tests/input_departure.rs) 和 [保存 / 默认回归](../../../apps/radishflow-studio/src/studio_gui_shell/tests/unit_presentation.rs) 覆盖失败保留、保存不清草稿及默认恢复重试；证据归 W40。实窗控制连接超时，未新增实际对比、读屏或系统缩放通过声明。

### 2026-10-02 I5 原生抽样证据

- 恢复原生控制后，中文浅色 Feed 压力样本验证未完成 / 错误 / 焦点组合、三类保存摘要、关闭保护及三类历史；项目显示变更保持当前结果，物理提交使结果过期。只覆盖 VA-02 / 04 / 05 / 10 / 11 的部分组合。
- 修复输入、项目及检查器单位菜单的匿名原生节点，现暴露作用域、量、当前值与组合期间禁用状态；[AccessKit 回归](../../../apps/radishflow-studio/src/studio_gui_shell/tests/unit_accessibility.rs) 与 macOS 原生树复核一致。快捷键修饰键时序另有 [回归](../../../apps/radishflow-studio/src/studio_gui_shell/tests/keyboard_events.rs)，实窗 ⌘Q / ⌘K、Tab → Enter、拼音组合通过抽样。
- 随后补齐首批 13 类字段原生提交 / 保存重开，修复继承值误计草稿与温压约束提示，覆盖 VA-02 / 04 / 05 / 11 的增量样本；[呈现回归](../../../apps/radishflow-studio/src/studio_gui_shell/tests/runtime/numeric_field_presentation.rs) 与 [字段矩阵证据](../units/input-drafts-and-interactions.md#i5-首批字段矩阵与呈现一致性) 区分 26 个中英文组件场景与原生语言抽样。
- 单位菜单越界焦点修复后，macOS 输入 / 项目 / 视图菜单键盘选择与应用通过；字段无障碍名称明确分隔单位、待提交与问题。另有 48 个组件宽度 / 字体 / 语言 / 状态场景，提供 VA-04 / 10 增量证据；见 [范围与缺口](../units/input-drafts-and-interactions.md#i5-单位菜单键盘与窄栏验证)。
- 首批数值错误改由类型化原因生成中英文，项目单位 / 个人默认弹窗补齐英文，20 个错误组件场景覆盖可访问名称及状态不变量；原生窗口验证提示换行、语言切换保留草稿、领域诊断可展开。作为 VA-02 / 04 / 10 / 11 的局部增量，[本地化证据](../units/input-drafts-and-interactions.md#i5-数值错误与单位设置本地化) 保留整应用与完整原生矩阵边界。
- 后续修复关闭确认焦点、数值选区和单位边框，补充实绘色对及两档系统工作区抽样，见 [主题证据](theme-and-accessibility.md#i5-实绘对比与原生验证)。VoiceOver 字幕窗口读取超时，实际播报未确认；完整缩放 / 原生组合、其余状态色对及运行态灰度 / 色觉矩阵待验，不声明完整 VA 通过。
- 本批再修复数值错误框 / 焦点环的内外框偏差，补双语输入法组合禁用说明及 60 组布局回归；12 组状态实绘及 50 个非彩色对照视图见 [主题样例](theme-and-accessibility.md#i5-状态组合与非彩色样例)。原生中英文拼音组合通过；VoiceOver 开关开启后应用清单仍显示未运行，实际读屏待验。

## 待决策清单

2026-09-23 项目所有者接受首批映射、主题与测量、迁移样本方向及五张 Pencil 局部画板。以下保留编号，区分接受范围与待完成项；[画板与静态证据](../../architecture/designs/studio-client-main-brief.md#本轮画板与静态复核) 不表示接口或产品验收通过。领域状态仍由所属领域决定，不能以 VD 决策另建求解 / 消息状态机。

| 编号 | 决策 / 规范归属 | 关联要求 | 最迟时机 / 负责领域 | 本轮结论与未完成项 |
| --- | --- | --- | --- | --- |
| VD-01 | [状态映射接口、身份 / 修订一致性、未知 / 不适用与不一致反馈](control-and-canvas-mapping.md#u2-首批映射方案已接受) | VIS-01、02、03、04、07 | V0 首批接入前；领域状态与 Studio；后续运行能力接入再复核 | 领域供给事实、共享映射方向已接受；[类型与身份 / 修订投影提案](control-and-canvas-mapping.md#v0-呈现接口提案待审定) 的通用扩展待审；首批数值有限接口已实施 |
| VD-02 | [角色槽位、图标 / 线型、窄布局压缩及图层叠加](control-and-canvas-mapping.md#u2-首批映射方案已接受) | VIS-04、05、06、12 | R1 对应画板与消费者实施前；Studio / Canvas / 设计 | 槽位与叠加方向已接受；五张局部画板已获认可，原生运行态仍待验证 |
| VD-03 | [token 色值、支持主题、对比目标、测量及原生可访问性映射](theme-and-accessibility.md#v0-首批主题方案已接受) | VIS-08、09、10 | R1 主题与首批控件实施前；主题 / 可访问性 / 验证 | 浅色评审色值与对比目标已接受；静态检查完成，[token / 原生映射提案](control-and-canvas-mapping.md#v0-呈现接口提案待审定) 的通用扩展待审；首批数值有限接口已实施 |
| VD-04 | [消费者盘点、共享映射边界、旧新并存与完成判据](control-and-canvas-mapping.md#u2-首批映射方案已接受) | VIS-05、06、07、11 | U2 首批迁移前；U3 / 辅助批次复核；各消费者维护方 | 首批字段、单位设置与保存提示范围已接受；首批数值字段已接通，2026-10-02 单位设置 / 保存角色已接通，其他消费者待迁移 |
| VD-05 | [样例数据、测量记录、主题 / 语言 / 窗口 / 平台样本](migration-and-acceptance.md#退出与证据) | VIS-01—12 | 各批次实施 / 验收前；设计与验证 | 样本范围已接受；静态样例与首批双语控件实绘灰度 / 色觉样例已补，完整消费者及原生矩阵待验 |

## 维护检查

每个 VIS 至少有一个 VA，场景反向注明要求；每个 VD 有规范归属、受影响范围及最后时机。V0 与 [单位事件](../units/input-drafts-and-interactions.md#u2-事件表)、[消息状态](../runtime-messages/message-model-and-lifecycle.md) 交叉核对，但不复制其事务或生命周期。文档检查证据不能替代画板、对比测量和原生运行验证。
