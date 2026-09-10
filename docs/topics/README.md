# 开发专题索引

更新时间：2026-09-10

## 用途

用途：作为 RadishFlow “总进度 + 一级轨道专题 + 二级功能专题”开发节奏的入口。

读者：需要判断下一步做什么、某个功能是否已进入范围、以及实现前应读哪些边界文档的开发者、用户、AI / Agent。

不包含：完整历史流水、逐日提交记录、具体代码实现细节和一次性讨论。

> [!IMPORTANT]
> 自 2026-06-12 起业务功能开发保持停止。下列表格中的 Active / Backlog 是停更前的专题组织记录，不表示当前仍有排期中的开发主线；专题当前只作为能力、边界、历史方案和未排期规划索引保留。当前维护范围以 `docs/status/current.md` 和根 `README.md` 为准。

## 组织原则

- `docs/status/current.md` 只回答当前维护状态、优先级、临时门禁、验证基线和下一步。
- 一级轨道专题回答一条开发主线的边界，例如 Studio 主路径、流程图建模、结果审阅、项目生命周期、CAPE-OPEN 适配层。
- 二级功能专题回答一个具体功能、单元、服务或页面怎么设计、分阶段推进、验收和验证，例如换热器、闪蒸罐、后端服务、后端 Web UI。
- `docs/devlogs/` 记录历史推进，不再承担“下一步怎么做”的职责。
- `docs/architecture/`、`docs/reference/`、`docs/capeopen/`、`docs/thermo/` 仍保存长期架构、字段语义和边界。
- 一个专题只有在范围、非目标、验收标准和最小验证都写清后，才进入代码实现。

## 一级轨道专题

| 专题 | 历史状态 | 停更前目标 | 入口 |
| --- | --- | --- | --- |
| Studio 主工作台与空白项目建模主路径 | Active | 把普通空白项目从物性配置、建模、运行、结果审阅和保存 / 重开收束成可复现主路径 | `studio-main-workflow.md` |
| 项目物性基础与组分选择 | Active | 稳定内置 package、项目组分、保存 / 重开和运行请求之间的同一事实源 | `property-basis-and-components.md` |
| 流程图建模与求解闭环 | Active | 维护受控单元、连接、readiness、Run Panel 和 solver 之间的边界 | `flowsheet-modeling-and-solve.md` |
| 结果审阅、诊断与恢复 | Active | 让 stream / unit 结果、诊断目标和 recovery action 使用同一套 snapshot / command 口径 | `results-review-diagnostics.md` |
| 项目生命周期与存储 | Backlog | 明确打开、保存、另存为、最近项目、sidecar 和脏改确认的长期边界 | `project-lifecycle-storage.md` |
| CAPE-OPEN PMC 适配层 | Frozen / Blocker-only | 保持 `.NET 10` PMC、COM 注册和 PME 验证基线，只修真实 blocker | `capeopen-pmc-adapter.md` |

## 二级功能专题

### 单元模块

| 专题 | 历史状态 | 父专题 | 入口 |
| --- | --- | --- | --- |
| Feed / 进料源 | Active | 流程图建模与求解闭环 | `unitops/feed-source.md` |
| Heater / Cooler 换热器 | Active | 流程图建模与求解闭环 | `unitops/heater-cooler.md` |
| Flash Drum 闪蒸罐 | Active | 流程图建模与求解闭环 | `unitops/flash-drum.md` |
| Mixer 混合器 | Active | 流程图建模与求解闭环 | `unitops/mixer.md` |
| Valve 阀门 | Active | 流程图建模与求解闭环 | `unitops/valve.md` |

### 建模对象

| 专题 | 历史状态 | 父专题 | 入口 |
| --- | --- | --- | --- |
| Material Stream 物流股 | Active | 流程图建模与求解闭环 / 结果审阅、诊断与恢复 | `modeling/material-stream.md` |

### 平台与服务

| 专题 | 历史状态 | 父专题 | 入口 |
| --- | --- | --- | --- |
| Control Plane 后端服务 | Backlog | 项目物性基础与组分选择 / 项目生命周期与存储 | `platform/control-plane-service.md` |
| Control Plane Web UI 后端管理台 | Backlog | Control Plane 后端服务 | `platform/control-plane-web-ui.md` |

### 未来规划（未激活）

| 专题 | 当前文档状态 | 父专题 | 入口 |
| --- | --- | --- | --- |
| 账户、登录与 Radish 联合身份 | Draft / 未排期 / 待架构决策 | Control Plane 后端服务 | [账户与联合登录](platform/account-and-federated-login.md) |

此处记录停更后按所有者要求补充的规划，不属于停更前的历史排期；建立专题不批准架构变更或恢复实现。

## 历史专题状态定义

| 状态 | 含义 |
| --- | --- |
| Draft | 已建文档但尚未作为近期实现依据 |
| Active | 当时的活跃专题；现在不表示获准恢复实现 |
| Blocked | 已确认阻塞，等待决策、外部环境或前置专题 |
| Done | 当时约定范围内验收完成，不代表工程准确性或当前支持承诺 |
| Frozen | 阶段性冻结，只修 blocker，不扩范围 |
| Backlog | 已记录但暂不推进 |

## 新增专题规则

当前新增文档不激活业务开发；已有问题优先更新所属专题。下列模板规则不替代当前维护边界。

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
