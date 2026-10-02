# 单位系统需求追踪表

更新时间：2026-10-02

## 用途与维护

用途：关联单位需求、职责真相源、验收场景、待决策与实现证据。
读者：规划、实现、评审与测试维护者。
不包含：规范正文副本、冻结接口或本轮产品验收声明。

父专题：[单位系统](../units-and-quantity-system.md)。UNIT、UA、UD 编号各自稳定且不复用；替代或拆分保留去向。每项 UNIT 有规范、切片和至少一个 UA 场景，每个 UD 有负责领域与最迟时机；实施后在本表补代码 / 测试及带日期证据。

设计、实施与验证分开记录：U1 已实现；U2 行为 / 兼容范围已接受、I1 / I2 项目层与 I3 会话 / 事务已接通，I4 首批原生控件、统一离开确认与设置 / 保存角色已接通；I5 首轮 macOS 抽样及 13 类字段提交 / 保存重开通过，完整交互与可访问性矩阵待续；U3 方向已确认且未实现；U2 完整用户路径仍待实现 / 待验证；U4 按需审定。所有权设计不等于持久化实现。跨阶段要求逐段记录，不用早期基础代替全部交付。

U1 证据为 [W38 的 U1 记录](../../devlogs/2026-09/2026-W38.md) 与现有代码 / 测试，历史提交 `8ab9b98f`；2026-09-16 macOS 基线已验证，不代表本轮重跑、非 SI UI 或全平台验收。本轮静态核对代码与测试位置，文档检查写入 W39。

## 要求与场景

| 编号 | 要求 / 规范归属 | 切片 | 场景 | 实施 / 验证；证据 |
| --- | --- | --- | --- | --- |
| UNIT-01 | [稳定量与单位身份](catalog-and-conversion.md) | U1 | [UA-01](catalog-and-conversion.md#目录与转换验收场景) | 已实现 / 已有历史验证；[目录](../../../crates/rf-types/src/units/catalog.rs)、[目录测试](../../../crates/rf-types/src/units/tests.rs)；W38 |
| UNIT-02 | [共享转换与明确拒绝](catalog-and-conversion.md) | U1 | [UA-02](catalog-and-conversion.md#目录与转换验收场景) | 已实现 / 已有历史验证；[转换](../../../crates/rf-types/src/units/mod.rs)、[转换测试](../../../crates/rf-types/src/units/tests.rs)；W38 |
| UNIT-03 | [转换精度与物理约束分离](catalog-and-conversion.md) | U1 | [UA-03](catalog-and-conversion.md#目录与转换验收场景) | 已实现 / 已有历史验证；[转换测试](../../../crates/rf-types/src/units/tests.rs)、[变量写入测试](../../../crates/rf-ui/src/variable_browser/tests.rs)；W38 |
| UNIT-04 | [变量量语义与规范标签](catalog-and-conversion.md) | U1 | [UA-04](catalog-and-conversion.md#目录与转换验收场景) | 已实现 / 已有历史验证；[变量元数据](../../../crates/rf-ui/src/variable_browser/variables.rs)、[浏览测试](../../../crates/rf-ui/src/variable_browser/tests.rs)；W38 |
| UNIT-05 | [单位集与选择优先级](unit-sets-and-persistence.md) | U2 | [UA-05](unit-sets-and-persistence.md#单位集与保存验收场景) | 部分实现 / I1 单位集已验证；[实现与证据](unit-sets-and-persistence.md#i1-代码与验证范围)；I2 个人默认已接通，[I4 覆盖 / 生命周期已接](unit-sets-and-persistence.md#i4-视图覆盖与历史生命周期)，实窗待验；[I2 证据](unit-sets-and-persistence.md#i2-代码与验证范围) |
| UNIT-06 | [独立保存脏标记与安全保存](unit-sets-and-persistence.md) | U2 | [UA-06](unit-sets-and-persistence.md#单位集与保存验收场景) | I2 已接通独立脏状态、呈现历史和失败基线保护；[证据](unit-sets-and-persistence.md#i2-代码与验证范围) |
| UNIT-07 | [旧工程与未知单位兼容](unit-sets-and-persistence.md) | U2 | [UA-07](unit-sets-and-persistence.md#单位集与保存验收场景) | 部分实现 / I1 文件兼容、拒读与原件保护已验证；[证据](unit-sets-and-persistence.md#i1-代码与验证范围)；I2 已接通升级确认及取消 / 失败保护 |
| UNIT-08 | [输入单位事件与拒绝保护](input-drafts-and-interactions.md) | U2 | [UA-08](input-drafts-and-interactions.md#交互验收场景) | I3 解析 / 换算事件已验证；[代码与范围](input-drafts-and-interactions.md#i3-代码与验证范围)；[I4 首批控件 / 合成事件证据](input-drafts-and-interactions.md#i4-首批控件与验证范围)，实窗待验 |
| UNIT-09 | [显示单位事件与草稿解释](input-drafts-and-interactions.md) | U2 | [UA-09](input-drafts-and-interactions.md#交互验收场景) | I3 显示切换保留会话已验证；[代码与范围](input-drafts-and-interactions.md#i3-代码与验证范围)；[I4 首批控件 / 合成事件证据](input-drafts-and-interactions.md#i4-首批控件与验证范围)，实窗待验 |
| UNIT-10 | [提交、取消与正式事务](input-drafts-and-interactions.md) | U2 | [UA-10](input-drafts-and-interactions.md#交互验收场景) | I3 提交 / 取消 / 来源重验已验证；[代码与范围](input-drafts-and-interactions.md#i3-代码与验证范围)；[I4 首批控件 / 合成事件证据](input-drafts-and-interactions.md#i4-首批控件与验证范围)，实窗待验 |
| UNIT-11 | [撤销、保存与焦点路由](input-drafts-and-interactions.md) | U2 | [UA-11](input-drafts-and-interactions.md#交互验收场景) | I2 已接项目呈现独立历史 / 保存保留草稿；I4 数值焦点与会话历史已接，视图历史已接，[统一离开确认及应用回归已接通](input-drafts-and-interactions.md#i4-统一离开确认与失败保护)，实窗待续 |
| UNIT-12 | [未编辑值、候选精度与等价输入](input-drafts-and-interactions.md) | U2 | [UA-12](input-drafts-and-interactions.md#交互验收场景) | I3 精度直通 / 微小编辑 / 继承采用已验证；[代码与范围](input-drafts-and-interactions.md#i3-代码与验证范围)；[I4 首批控件 / 合成事件证据](input-drafts-and-interactions.md#i4-首批控件与验证范围)，实窗待验 |
| UNIT-13 | [现有结果与输出一致性](output-and-consumer-consistency.md) | U3 | [UA-13](output-and-consumer-consistency.md#输出验收场景) | 待实现 / 待验证；— |
| UNIT-14 | [输出资格与失败隔离](output-and-consumer-consistency.md) | U3 | [UA-14](output-and-consumer-consistency.md#输出验收场景) | 待实现 / 待验证；—；沿用结果门禁，新增转换路径未验证 |
| UNIT-15 | [SI 协议兼容与后续消费者](output-and-consumer-consistency.md) | U3 / 后续 API、录制、N4 | [UA-15](output-and-consumer-consistency.md#输出验收场景) | 新增接入待实现 / 待验证；—；现有 SI 基础见 UNIT-04 |
| UNIT-16 | [上下文与基准转换扩展](catalog-and-conversion.md) | U4 | [UA-16](catalog-and-conversion.md#目录与转换验收场景) | 待实现 / 待验证；—；U1 仅识别并拒绝，见 UNIT-02 |

### 2026-10-02 I5 原生抽样证据

上表 I4 的原生待验范围已取得以下增量证据，完整场景尚未全部通过。详情见 [交互专题](input-drafts-and-interactions.md#i5-首轮-macos-原生验证) 与 [W40](../../devlogs/2026-09/2026-W40.md#2026-10-02-u2-i5-首轮原生验收与修复)。

| 场景 | 本轮 macOS 证据 | 仍需补充 |
| --- | --- | --- |
| UA-05—07 | 工程集应用 / 保存 / 重开、v2 升级确认、独立保存摘要与呈现撤销；未应用设置阻止保存 / 关闭，损坏 JSON 打开保留原现场；SI 文件读回 | 个人默认恢复、未知单位拒读等完整原生路径 |
| UA-08 / 10 | Feed 压力 bar ↔ kPa、本地历史、未完成 / 拼音非法输入保护；13 类非 SI 提交及四例保存重开 / 重跑；显式 kPa 与 bar 后缀冲突保护 | 全字段交互组合、更多后缀及输入法时序 |
| UA-09 | 项目显示切换与撤销不改物理值或当前结果资格；双对象草稿跨 Inspector / 模块设置及项目 SI / 视图 MPa，提交一项保留另一项 | 更多语言 / 视图组合及跨逻辑窗口原生样本 |
| UA-11 | 工程撤销取消 / 放弃、流股删除范围、创建出口修复、保存保留草稿及三类离开状态；打开失败 / 取消保留，成功替换清理；菜单键盘、⌘Q / ⌘K；关闭确认默认取消、Tab 循环 / Escape 保留 | 其余对象 / 修复入口、迟到确认 / 失败事务的完整原生矩阵及跨平台 |
| UA-12 | 13 类非 SI 字段的参数 / 出口模板与磁盘 SI 值一致；继承 Feed 温度同值确认采用显式来源 | 高精度反复往返、真实微小编辑等原生样本；现有自动回归保持 |

后续字段批次另有 13 类 × 中英文共 26 个自动回归场景，覆盖提交、工程历史、保存重开及求解；原生语言仅按字段分组抽样。约束提示和继承字段草稿计数的修复与证据见 [交互专题](input-drafts-and-interactions.md#i5-首批字段矩阵与呈现一致性) 及 [W40](../../devlogs/2026-09/2026-W40.md#2026-10-02-u2-i5-字段矩阵与呈现一致性)，不等于完整 UA 通过。

单位菜单焦点越界修复、三类菜单键盘回归和 48 个中英文 / 窄栏 / 字体组件场景见 [后续验证](input-drafts-and-interactions.md#i5-单位菜单键盘与窄栏验证)；原生宽度与组件宽度分别记录。

同日补齐首批数值错误及项目单位 / 个人默认弹窗中英文，类型化拒绝载荷交由壳层呈现；20 个错误组件场景、默认保存 / 恢复失败回归和 macOS 实窗抽样见 [本地化证据](input-drafts-and-interactions.md#i5-数值错误与单位设置本地化)。UA-05 / 06 / 10 / 11 仅增加局部证据，完整语言 / 原生组合、系统缩放和读屏仍待验。

多对象草稿、显示作用域、删除 / 修复 / 工程撤销及保存 / 打开 / 关闭的组合实窗证据见 [离开保护验证](input-drafts-and-interactions.md#i5-多对象草稿与离开保护原生验证)。本批无产品代码变更，另通过 14 项既有定向回归；自动与原生证据分别记录，未将代表路径扩为完整 I5 收口。

关闭确认的键盘焦点修复、`1512×982` / `1024×665` 系统工作区抽样及实绘色对回归见 [可访问性增量](input-drafts-and-interactions.md#i5-键盘焦点系统缩放与实绘对比)。VoiceOver 自身窗口读取超时，未确认实际朗读；精确缩放及灰度 / 色觉矩阵仍待验。

## 待决策清单

2026-09-23 项目所有者接受 UD-01—06 的候选及五张 Pencil 局部画板；以下区分已接受内容与剩余决策。行为已回写所属规范；I1 按提案获授权并实现，U2 整体产品验收仍待完成；[局部画板与静态证据](../../architecture/designs/studio-client-main-brief.md#本轮画板与静态复核) 不等于接口冻结。UD-07 / UD-08 保持后续切片决策。

| 编号 | 决策 / 规范归属 | 关联要求 | 最迟时机 / 负责领域 | 本轮结论与未完成项 |
| --- | --- | --- | --- | --- |
| UD-01 | [内置集、自定义范围、格式、视图覆盖生命周期与目录扩充](unit-sets-and-persistence.md#首版单位集与适用范围) | UNIT-01、05、08、09 | U2 单位集 / 菜单实施前；单位与 Studio | 首版范围已接受；扩展精度 / 本地化格式及新增目录后续另审 |
| UD-02 | [呈现 DTO、版本、未知 ID 保留、旧版本往返与草稿保存范围](unit-sets-and-persistence.md#u2-存储接口) | UNIT-06、07、11 | U2 文件格式变更前；存储与单位 | 兼容方向已接受；外层 2 / metadata 1、完整 DTO 与严格读取已批准并实现；固定旧版本门禁证据已补；I2 独立个人默认 / 显式备份恢复与升级提示已授权并实现 |
| UD-03 | [显示切换期间的草稿策略、失焦 / 跨对象 / 保存触发及事务重验](input-drafts-and-interactions.md#u2-编辑接口与接线边界) | UNIT-09、10、11 | U2 局部交互评审与接线前；Studio 与应用事务 | 显示切换、显式提交、导航 / 保存 / 离开策略已接受；I3 会话 / 重验及失败保护已接通；I4 统一原生离开确认已接通，原生组合验收待 I5 |
| UD-04 | [文本 / 工程 / 呈现历史的快捷键路由与分组](input-drafts-and-interactions.md#u2-编辑接口与接线边界) | UNIT-06、11 | U2 撤销交互实施前；编辑历史与 Studio | 三类历史路由已接受；I3 会话历史快照已实现；I4 首批连续输入分组 / 焦点路由已接，实窗待验 |
| UD-05 | [数值语法、本地化、粘贴单位、空值 / 清除与校验时机](input-drafts-and-interactions.md#解析与精度规则) | UNIT-08、10、11 | U2 字段解析实施前；字段与单位 | 首版语法、后缀冲突与空白分类已接受；I3 解析结果 / 单位来源已实现；I4 首批原生非 SI 已接，实窗待验 |
| UD-06 | [实际编辑后的等价值判定、容差及独立数值 / 平台用例](input-drafts-and-interactions.md#解析与精度规则) | UNIT-03、10、12 | U2 精度与提交实施前；事务、单位与验证 | 精确同值与未编辑 SI 保留已接受；I3 精度来源与独立数值回归已补；原生 / 平台证据待补 |
| UD-07 | [输出捕获时点、格式、覆盖选择和对外协议兼容](output-and-consumer-consistency.md) | UNIT-13、14、15 | U3 输出接入前；新协议 / 回放 / N4 各自实施前另审；输出与 API | 待 U3 及后续消费者审定 |
| UD-08 | [上下文来源 / 有效性、定义版本与独立工程依据](catalog-and-conversion.md) | UNIT-16 | U4 各基准转换实施前；单位、物性与工程基础 | 待 U4 具体转换审定 |

## 维护检查

逐项核对 UNIT / UA 双向覆盖、UD 关联、链接、事件表 UE-01—13 的场景覆盖；新增失败模式补到所属子专题。U2 评审同时核对 [V0 叠加](../color-state/semantics-and-composition.md)，不复制颜色规则或消息生命周期。U2→U3→规格辅助→设备工程的顺序保持。
