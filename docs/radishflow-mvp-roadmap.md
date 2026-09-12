# RadishFlow MVP Roadmap

更新时间：2026-09-12

## 用途

用途：保留第一阶段里程碑、验收含义和历史规划入口，并记录未排期的后续决策参考。
读者：需要理解现有成果、历史目标和可能的投资顺序的项目所有者与维护者。
不包含：当前任务清单、逐日验证流水、完整 UI 设计和新的产品交付承诺。

## 维护状态

自 2026-06-12 起业务功能开发保持停止，当前没有激活的产品主线或功能排期。所有旧专题中的 Active / Backlog、旧 Sprint 和“下一步”均为历史组织记录；执行边界以 [当前状态](status/current.md) 为准。

本次校准不恢复产品路线、不批准代码重构，也不触发 tag、打包、发布或服务端部署。

## 第一阶段目标与成果

第一阶段目标是验证 Rust Core、Rust Studio 与 `.NET 10` CAPE-OPEN PMC 能在最小稳态流程中协作。保留的能力见 [MVP 范围](mvp/scope.md)。

| 里程碑 | 历史目标 | 已有证据与限制 |
| --- | --- | --- |
| M1 | 仓库与基础骨架 | workspace、分层和治理检查已形成 |
| M2 | 二元 TP Flash | 算法、golden 与相边界回归已形成；不等于独立物性准确性验证 |
| M3 | 小流程与 Studio 闭环 | 受控建模、运行、结果审阅、保存 / 重开 / rerun 已有回归 |
| M4 | Rust FFI 与 .NET 适配 | JSON / error、native 装载与调用路径已有基线；生命周期静态风险仍需处理 |
| M5 | PME 识别并调用 PMC | DWSIM / COFE 代表场景完成历史人工验证，不推导为任意宿主支持 |

MVP α、β 人工 smoke、失败恢复和通用小流程建模已在停更前阶段性收口。已通过范围不包含完整自由连线、真实 EOS、recycle 收敛、完整能量闭环、第三方模型加载或正式发布。历史 `v26.5.1-dev` 仅为内部 staging 记录。

- [MVP α 验收](mvp/alpha-acceptance-checklist.md)
- [MVP β 验收](mvp/beta-acceptance-checklist.md)
- [PME 验证入口](capeopen/pme-validation.md)

## 后续决策参考（未排期）

以下是 2026-09-06 审阅后的建议顺序，仅供项目所有者以后重新决策；任何业务实现都仍需先明确改变停更边界并确认范围。

| 顺序 | 需要回答的问题 | 可评估的工作 | 验收依据 |
| --- | --- | --- | --- |
| 1 | 用户、体系与工况是什么 | 选择一个有限用途，定义物性来源、适用温压范围和误差目标 | 场景与非目标明确，样例不冒充真实数据 |
| 2 | 计算是否有物理依据 | 一致焓模型、所需物性方法、PH Flash 与 Mixer / Valve 能量闭环 | 独立基准、守恒残差、边界行为；模型选择服从工况 |
| 3 | 原有项目与宿主调用是否可靠 | native 句柄生命周期、保存故障恢复、结果来源记录 | 释放与异常路径验证、可恢复项目和可追溯结果 |
| 4 | 用户能否独立完成流程 | 基于同一输入 / 命令 / snapshot 的建模、运行、诊断和重开 | 用户主路径可复现，不依赖开发者解释 |
| 5 | 复杂度是否有实际收益 | 审查 Studio 转发层、同步求解、重复投影与大型测试组织 | 真实负载与维护成本测量，按职责调整而非机械拆文件 |

前两项不会因 UI、CAPE-OPEN 或测试数量增加而自动完成。现有 golden 回归与跨层一致性应保留，同时增加独立证据；具体限制见 [热力学模型](thermo/mvp-model.md)。

Control Plane、完整报表、复杂画布、智能辅助、动态模拟与 CFD 不作为上述工作的并行默认主线。只有已有用途得到验证、外部依赖确有必要且资源允许时，才重新评估对应历史方案；不预先增加 schema、crate 或平台。

## 已发现问题的归属

| 事项 | 记录位置 | 当前性质 |
| --- | --- | --- |
| 工具链最低版本、可复现性 | [当前状态](status/current.md) | 已对齐固定基线与支持声明；属于外围维护，后续兼容性按需验证 |
| 物性样例、近似与数值证据 | [热力学模型](thermo/mvp-model.md) | 实现能力说明与未完成验证 |
| 缓存进入 thermo、Canvas 实际位置 | [架构总览](architecture/overview.md) | 目标与实现差异 |
| native 句柄释放保护 | [适配层专题](topics/capeopen-pmc-adapter.md) | 静态风险，未复现 Windows 崩溃 |
| Windows 保存回滚错误 | [存储专题](topics/project-lifecycle-storage.md) | 静态风险，未执行故障注入 |
| Studio 调用层次与同步执行 | [App 架构](architecture/app-architecture.md) | 维护性建议，无性能测量结论 |

## 历史规划与详细入口

下列文件保留当时的任务拆分和决策过程，其时间性建议不作为现行指令：

| 文档 | 内容 |
| --- | --- |
| [milestones.md](mvp/roadmap/milestones.md) | M1-M5 原始任务和退出标准 |
| [plan-alignment-studio.md](mvp/roadmap/plan-alignment-studio.md) | Studio / 控制面 / GUI 宿主历史计划对齐 |
| [plan-alignment-capeopen.md](mvp/roadmap/plan-alignment-capeopen.md) | .NET / COM / PME 历史计划对齐 |
| [workflow-and-risks.md](mvp/roadmap/workflow-and-risks.md) | 原始 Sprint、风险和阶段复盘 |
| [周志索引](devlogs/README.md) | 逐周推进与验证流水 |

具体能力见 [专题索引](topics/README.md)；Studio 设计方案见 [UI 专题](architecture/studio-ui-topic-plan.md)。架构目标、设计稿和已实现代码分别保留其证据边界，不以设计完成代替实现完成。
