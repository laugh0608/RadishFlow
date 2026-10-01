# MVP Scope

更新时间：2026-09-13

## 用途与维护状态

用途：记录当前 MVP 的能力范围、实现限制与验收含义。
读者：需要判断现有代码能做什么、不能证明什么的使用者和维护者。
不包含：逐周开发流水、详细接口定义、工具链命令和新的开发排期。

项目正常开发和迭代，本文的已有能力构成回归基线；当前切片与优先级以 [当前状态](../status/current.md) 为准。

## 已保留的 MVP 范围

第一阶段验证了 Rust Core + Rust Studio + `.NET 10` CAPE-OPEN Unit Operation PMC 的最小稳态流程闭环。

| 领域 | 已有范围 | 尚不具备的能力 |
| --- | --- | --- |
| 物性与闪蒸 | 二元样例、Antoine / 理想 K 值、Rachford-Rice TP Flash、泡露点窗口、常热容相焓 | 完整焓参考态与潜热、PH / PS Flash、真实 EOS 与完整组分数据库 |
| 单元与流程 | Feed、Mixer、Heater / Cooler、Valve、Flash Drum，无回路顺序模块法 | recycle 全功能收敛、严格塔器、复杂多组分或反应网络 |
| Studio | 物性选择、受控放置与连接、输入检查、运行、结果审阅和失败恢复 | 任意端口自由连线、自动布线、完整报表、完整多文档工作台 |
| 项目生命周期 | 新建、打开、保存、另存为、undo / redo、布局 sidecar、最近项目 | 云同步、多人协作、完整跨平台原生文件工作流 |
| CAPE-OPEN | 自有 PMC、受控注册、DWSIM / COFE 代表场景的历史人工验证 | 第三方模型加载、完整 Thermodynamics PMC、任意 PME 兼容承诺 |
| 控制面 | 客户端协议、HTTP transport、缓存与租约编排 | 本仓库内的服务端交付、真实登录到授权分发的完整联调 |

已覆盖的受控流程为：

- `Feed -> Flash Drum`
- `Feed -> Heater/Cooler/Valve -> Flash Drum`
- `Feed + Feed -> Mixer -> Flash Drum`

M1-M5、MVP α / β 和通用小流程建模的收口指上述范围内的契约与用户路径验收，不表示工业工况准确性认证或正式发布。证据见 [α 清单](alpha-acceptance-checklist.md)、[β 清单](beta-acceptance-checklist.md) 和 [历史路线图](../radishflow-mvp-roadmap.md)。

## 数值与数据解释边界

- `binary-hydrocarbon-lite-v1` 和相关 golden 是软件演示与回归样例；methane / ethane 名称以及历史材料中的 `official` 不代表参数已完成真实物性验证。
- Mixer 当前按摩尔流量加权温度；Valve 当前保持入口温度并调低压力；Heater / Cooler 使用指定出口 T/P。它们不构成完整能量闭环。
- 相焓采用以 `298.15 K` 为参考的常热容显热模型，不能据此解释真实相变潜热。
- `Converged` 表示当前模型的求解路径成功，不表示已验证物性适用范围、能量守恒或工程误差。
- golden 回归、跨层 DTO 一致性和独立物理基准是不同证据，不能互相替代。

公式、关联式温区、样例解释和数值证据要求统一见 [热力学 MVP 模型](../thermo/mvp-model.md)，不在各单元专题复制另一套假设。

## 保留的领域与应用约束

### Core

- 内部使用 SI：温度 K、压力 Pa、摩尔流量 mol/s；组成使用摩尔分率，相标签为 overall / liquid / vapor。
- `rf-model` 只承载对象模型；COM 语义留在 `.NET`，求解调度留在 solver。
- 单元围绕 `MaterialStreamState` 输入输出执行，连接使用 canonical material ports；一股一源一汇，终端产品流可只有 source。
- 当前求解器拒绝环路；早期“极简回路”设想不作为已实现能力。

### Studio 与项目

- 保持单文档工作区，普通空白项目通过独立物性页显式选择受控 package 与组分，不预写选中状态。
- readiness 检查真实建模输入；小案例作者清单仅用于导航，不是通用运行 gate，也不代替 solver 的连接、拓扑和执行诊断。
- Inspector 草稿只有经语义提交才进入文档；显示的 outlet 默认值不等同于已提交 unit parameter。
- 文档变更通过正式命令进入 revision 和 undo history；当前 undo / redo 使用 before / after flowsheet 快照。
- 结果按 document revision 失效；审阅、复制和轻量 `.txt` 导出使用当前快照，不在 UI 重算数值。
- 受控 suggestion 必须由用户显式接受；断开、删除和唯一合法候选重连继续走命令边界，不等于自由连线。
- 单元位置与 viewport offset 保存在 `<project>.rfstudio-layout.json`，不写入项目语义或文档历史。
- `Save / Save As`、覆盖确认、脏改关闭和退出共用项目生命周期；staged write 的平台恢复限制见 [存储专题](../topics/project-lifecycle-storage.md)。
- 项目格式是单文件 `*.rfproj.json`；授权缓存、token、snapshot history 和窗口布局不混入项目文件。
- 中文 / 英文切换属于 shell 偏好；当前 shell 已内嵌 Inter 与 SourceHanSansSC 字体。Windows 原生打开 / 保存选择器不构成跨平台文件工作流承诺。

详细边界见 [App Architecture](../architecture/app-architecture.md) 与 [专题索引](../topics/README.md)。

### 互操作与外部控制面

- Rust 与 `.NET` 之间只传递句柄、基础数值、UTF-8 和 JSON。
- 自有 PMC 的接口、GUID、IDL / TLB、异常、注册和 PME 消费路径由 [CAPE-OPEN 边界](../capeopen/boundary.md) 管理。
- 注册保持默认 dry-run、执行 preflight 和显式确认，不因历史验证通过而默认修改系统环境。
- 历史控制面目标采用 OIDC Authorization Code + PKCE；public client 不内置长期 client_secret，token 目标存储为操作系统安全存储。
- 本地求解不依赖逐次远端计算；控制面与资产分发方案不代表已经部署。实现现状见 [控制面专题](../topics/platform/control-plane-service.md)。

## 非目标与后续决策

动态、瞬态与多种求解策略不属于已交付 MVP，但已纳入 [长期模拟平台目标](../architecture/simulation-platform.md)；历史非目标不作为永久排除。通用 CFD、第三方 CAPE-OPEN 模型加载与完整 Thermodynamics PMC 仍需独立评估范围、依赖和验收。

下一阶段优先完善基础功能与使用闭环，持续验证基本正确性和可靠性；新数值模型实施前再明确对应体系、工况与独立验收依据。迭代顺序见 [路线图](../radishflow-mvp-roadmap.md#后续迭代顺序)，具体切片由当前状态统一管理。

## 文档路由

- 当前维护范围与验证入口：[current.md](../status/current.md)
- 模块实际职责：[架构总览](../architecture/overview.md)
- 数值假设与准确性边界：[热力学模型](../thermo/mvp-model.md)
- 功能、单元和存储契约：[专题索引](../topics/README.md)
- 历史 M1-M5 与计划对齐：[路线图](../radishflow-mvp-roadmap.md)
- 逐周过程、历史命令及实现收口：[周志索引](../devlogs/README.md)
