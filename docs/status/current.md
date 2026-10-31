# 当前状态

更新时间：2026-10-31

## 用途

用途：恢复当前状态、优先级、临时门禁及范围。

读者：仓库维护者、用户与 AI / Agent。

不包含：完整历史流水、详细功能设计、长期协作规则、命令级测试日志和发布说明。

先读本文档，再按“按需阅读”进入专题；`AGENTS.md` / `CLAUDE.md` 只保留跨任务、跨阶段且必须启动即生效的长期约束。

## 当前结论

- 自 2026-09-12 起恢复正常开发和迭代，取消此前业务停更限制；模拟核心、物性、单元、Studio、生命周期、互操作和发布准备按规划推进，外部贡献按贡献流程评审。
- MVP 第一阶段 M1-M5、MVP α / β 与通用小流程建模 v1 已阶段性收口，后续迭代以现有能力与验证证据为起点。
- 2026-10-23 再次确认基础架构优先：先贯通工程生命周期、模型装配、运行任务、应用命令与结果链路，再逐步充实物性、单元和算法；延续 2026-09-13 的基础闭环安排。独立数值研究不作为基础功能的统一前置门禁，物理能力仍独立验收。
- 长期覆盖稳态、动态、瞬态、间歇 / 半间歇，以及递归分块、序贯模块、联立模块和联立方程；统一 API 明确采用变量 / 动作浏览树，支持 COM 自动化和操作录制 / 回放。领域与术语边界见 [模拟平台长期规划](../architecture/simulation-platform.md)；目标不表示已实现或同时排期。
- 插件、分层许可与公共 API 授权 / 配额已纳入 [规划](../radishflow-mvp-roadmap.md#插件与公共-api-的实施切片)；运行时、许可政策、接口和部署待切片决策。
- [单位系统](../topics/units-and-quantity-system.md)、[规格与辅助](../topics/modeling-assistance-and-specifications.md)、[设备设计与校核](../topics/equipment-design-and-rating.md) 首批已有实现，进展见下文；设备工程不等待完整动态，未来 Agent 复用统一入口，当前不接入 AI。
- [工程基础、工况与来源](../topics/engineering-basis-and-cases.md) 已支撑当前设备链路；目标反算、工程检查、压力 / 公用工程和交付按 [配套切片](../radishflow-mvp-roadmap.md#工程工作流的配套切片) 扩展，近期顺序服从 AF。
- 旧 `.pen` 保留，新增分区按 [UI 计划](../architecture/studio-ui-topic-plan.md) 评审；U2 / V0 局部画板已获认可。
- 当前尚未进入正式 tag / release 节点；历史 `v26.5.1-dev` 只作为内部 staging 草案和验证记录保留。

## 当前开发范围与优先级

### 正常推进范围

- 按专题设计、实现和验证模拟功能、物性模型、单元模块及 Studio 用户路径。
- 修复项目保存恢复、原生句柄生命周期、跨平台兼容与结果诊断等已知问题。
- 按需求评审账户、控制面、发布与安装能力；进入实现前明确依赖、接口、兼容性和验收标准。
- 持续维护文档、CI、仓库治理、安全基线和固定工具链，为产品迭代提供验证基础。

### 当前优先级

1. 按 [代码链路审计](../architecture/overview.md#工程基础链路审计2026-10-23) 和 [AF0—AF6](../radishflow-mvp-roadmap.md#基础架构优先的实施切片) 推进。AF0 已完成；AF2a 内建目录 / 工厂、AF2b 单包上下文已接通。AF1 未定因，独立跟踪。
2. AF2—AF5 已贯通，AF6 审查材料完成；先审定有限组分衡算装配方案。
3. C2-A 后续方法对比方案已认可，排期服从基础主线；原精度失败不变。C2-B / C3 暂不实施，不以共享基础为名接入新 PR 产品链或流程设备物理关联。
4. 基础契约中提前审查 EO / 动态 / 瞬态的扩展边界，具体物性、单元、递归分块 / recycle 和高级算法按真实场景逐步填入；每项另做物理验证。
5. 保持可靠性、固定工具链及跨平台回归；I5、未验平台和发布前 GUI / PME 门禁不变。

2026-09-13 的 B0 / B1 主路径基线见 [建模专题](../topics/flowsheet-modeling-and-solve.md) 和 [生命周期专题](../topics/project-lifecycle-storage.md)。账户联合登录仍为待架构决策的 Draft。

### 最近进展（截至 2026-10-31）

- [AF6 审查](../topics/modeling/solver-strategies-and-assembly.md#af6-扩展边界复核2026-10-31)已形成代码证据、两个场景及最小方案，接口 / 模型实现待审定。

- [AF2a](../topics/modeling/model-catalog-and-runtime.md#af2a-内建目录与工厂)六类目录 / 工厂已接通，旧 kind、格式和诊断保持。

- G1 / S1、S2-A、G2-B 及 C1 采纳 UI / 固定证据 / 输出已完成，证据见 [W43](../devlogs/2026-10/2026-W43.md)。

- AF0 审计与排序已完成；C2-A 三轮材料已提交，原表审计待审、精度仍未达标。

- B3、U1—U3 首批已接通；证据见 [W38](../devlogs/2026-09/2026-W38.md) 与 [W41](../devlogs/2026-10/2026-W41.md)。

### 下一步

- [ ] 按已确认的 [I5 延期安排](../topics/units/requirements-traceability.md#i5-受控延期与-u3-开工安排) 保留未验项与补验节点；U / A 首批完成不表示 I5 通过或发布放行。
- [x] G0 / S1、S2-A、G2-B 与 [C1](../architecture/designs/studio-equipment-adoption-brief.md#接下来事项)的设备数据 / 事务 / 保存 / UI / 输出及代表验证完成，流程关联未实现。
- [x] AF0 代码审计、功能断点与依赖切片已整理，基础架构优先安排已同步；不代表后续切片实施完成。
- [ ] 按 [AF1](../topics/project-lifecycle-storage.md#af1-接下来事项2026-10-23) 捕获异常现场，定因后确认修复并验收；未复现不等于收口。
- [x] AF2a / b 装配、AF3 后台运行、AF4a / AF5a 查询与 AF4b 正式动作 / GUI 已完成。
- [x] [AF4c 契约](../topics/variable-and-object-browser.md#af4c-新版-cli-契约2026-10-26已确认)四片完成，v3 覆盖及磁盘边界已验证，旧协议保持。
- [x] AF5b 来源、导航、输出及代表验证完成。
- [x] AF6 链路 / 场景审查材料完成；按[接下来事项](../topics/variable-and-object-browser.md#接下来事项2026-10-31)审定后续实现。
- [ ] [C2-A 方法对比](../thermo/mvp-model.md#c2-a-后续方法对比与排期)已认可、尚未实施；第三轮来源审计仍待审，混合物气相仍缺，B / C3 不开工。
- [ ] 全局 R0 待评审；U / A / 设备局部稿通过不代表全局定稿。
- [ ] [可视化 V1—V4](../radishflow-mvp-roadmap.md#颜色与流程可视化的配套切片)方向已确认、均未实现：二维流向 / 数据着色随结果基础，局部三维依赖设备几何，动态回放依赖真实时间结果；整厂三维另评估。

### 相关规划

- [模型装配 MC0—MC3](../topics/modeling/model-catalog-and-runtime.md) 与 [多策略 QS0—QS4](../topics/modeling/solver-strategies-and-assembly.md) 方向已确认；现有模型装配先按 AF2 推进，算法按 P3 / P4 分期；第三方宿主未实现。

- [变量与对象浏览器 VB0—VB3](../topics/variable-and-object-browser.md) 的后续跨端覆盖衔接 AF4；已有变量读写不等于全部工程对象已覆盖。

- [装置工程与数字孪生](../architecture/simulation-platform.md#装置工程与数字孪生愿景) 按 DT0—DT4 分期，未因本轮架构梳理启动实施。

- [运行与诊断 N0—N5](../topics/runtime-messages-and-diagnostics.md) 的任务基础与结果联动衔接 AF3 / AF5；GUI 后台与取消已接通；暂停和开发工作台待续。

### 后续事项

- [ ] 按项目所有者 2026-09-15 要求，Windows / Linux 原生键盘、保存 / 输出等路径累积到后续阶段节点集中验证；远距离视口按实际缺口推进。未测项保留，P1 不宣称全平台收口。
- 真实窗口前先告知，UTM 串行且最多一台运行；未测平台保持待验证。
- 插件运行时、具体许可政策、公共 API 与完整报告继续按独立规划推进。

## 当前能力基线

- 受控流程可覆盖 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum`，以及显式输入、运行、保存、重开、rerun 和轻量结果审阅。
- Studio 已形成 Home、独立物性页、独立设备页、流程图工作台、模块设置 / 结果和运行状态区域的主路径；设备计算与流程 Run 分离，支持尺寸、逐行校核及有限矩阵 CSV / JSON。
- Rust Core 与 `.NET 10` 适配层保持隔离；CAPE-OPEN / COM 语义不进入 Rust Core。
- 数值实现仍使用演示物性、常热容显热和简化单元行为；golden、结果透传与用户路径验收不等于独立物理准确性证明，见 [热力学模型](../thermo/mvp-model.md)。
- 能力、命令和验收证据以 `docs/topics/`、`docs/mvp/`、`docs/reference/` 与历史周志为准。

## 已知差异与风险路由

- 架构差异：`rf-canvas` 占位与 Studio 执行边界见 [架构总览](../architecture/overview.md)。
- 原生句柄释放与并发调用契约及验证范围见 [CAPE-OPEN 专题](../topics/capeopen-pmc-adapter.md)。
- Windows 保存替换与回滚失败的恢复契约见 [项目生命周期专题](../topics/project-lifecycle-storage.md)。

可靠性、平台证据见周志，其他边界见专题；顺序见 [迭代路线图](../radishflow-mvp-roadmap.md#后续迭代顺序)。

## 工具链现状

workspace 最低支持版本声明为 Rust `1.96`，`rust-toolchain.toml`、`PR Checks` 与 `Release Checks` 固定使用 `1.96.0`。这是仓库选择维护的支持下限与验证基线，不表示更低版本必然无法编译；不承诺未验证的低版本兼容性。

2026-09-12 核对完整锁定依赖图共 469 个包，外部依赖已声明的最高 Rust 要求为 `1.88.0`（`image`、`time`、`time-core`、`time-macros`），另有 127 个包未声明最低版本。元数据不构成完整 workspace 在 Rust 1.88 上可构建的证据；原有 `1.86` 声明已撤回，`Cargo.lock` 保持不变。

正式 Rust 检查与 CI 构建入口使用 `--locked`；浮动 `stable` 由独立的 `Rust Compatibility` 手动工作流在三平台检查，不进入 `Candidate Quality`，不自动升级固定版本。维护步骤与复现范围见 [工具链维护规则](../development/agent-collaboration.md#rust-工具链与锁定依赖)。

macOS 现有图形依赖链中的 `block 0.1.6` 仍有未来 Rust 兼容性警告；固定工具链不等于完成依赖升级，也不保证未来编译器兼容。

## 当前验证基线

- 文档改动：运行 `xtask` 的 `check-repository-governance`、`check-text-files`，检查文档体量和 diff；仅验证工具 / 编排变化另跑 `cargo test --locked -p xtask`。
- 阶段收口或跨模块治理变化：在 macOS / Linux / CI 执行 `./scripts/check-repo.sh`；Windows 使用 `pwsh ./scripts/check-repo.ps1`。
- `check-repo` 统一执行治理、文本、格式、严格 Clippy 和测试并报告计时；CI 按 [ADR](../adr/0001-branch-and-pr-governance.md) 路由，稳定主线保持完整覆盖。
- `adapters/reference/` 下的外部参考资料保留上游编码、BOM 和换行格式，不为通过仓库文本门禁而批量改写。
- 重要验证若出现明显的沙盒权限、受限 restore、project reference 解析或 native 装载差异，先按 Agent 协作规则告知用户，再申请最小范围真实环境复验。

最近通过记录：

- 2026-10-31：本机 1,811 项及 PR #17 全部 CI 通过，merge 后已回灌 `dev`；原生实窗边界见 [W44](../devlogs/2026-10/2026-W44.md)。

- 既有功能与实窗证据见 [W38](../devlogs/2026-09/2026-W38.md)，跨平台、native 与 `.NET` 历史基线见 [W37](../devlogs/2026-09/2026-W37.md)，更早记录见 [W36](../devlogs/2026-09/2026-W36.md)；历史证据不替代新增路径或发布验收。

## 当前范围控制

- 动态、瞬态与多求解策略已进入长期目标，尚未进入当前基础功能实现批次；历史 MVP 非目标不构成永久排除。第三方 CAPE-OPEN、完整 Thermodynamics PMC 与通用 CFD 耦合按相应专题和需求评估。
- 控制面、账户登录、完整报表和安装器按实际需求与前置条件安排，不同时作为默认开发主线。设备工程已进入独立规划，尺寸 / 设计 / 校核不等于机械规范或制造认证；当前模型无对应工业精度承诺。
- 业务开发恢复不等于创建 tag、发布产物或部署服务；这些操作按相应验收与发布流程推进。
- Rust Core 不承载 COM 语义；新增抽象需服务真实领域职责与维护收益。

## 按需阅读

- 文档总索引与篇幅治理：`docs/README.md`
- Agent 协作、授权与验证选择：`docs/development/agent-collaboration.md`
- 开发专题与能力索引：`docs/topics/README.md`
- 未排期账户与联合登录规划：`docs/topics/platform/account-and-federated-login.md`；实现前完成 M0 范围与架构决策。
- 插件与许可边界：[插件系统](../architecture/simulation-platform.md#插件系统)、[授权分层](../architecture/auth-entitlement-architecture.md#软件许可产品授权与操作权限)；公共凭据、资源权限与计算限额见 [公共 API 专题](../topics/platform/public-api-and-access-control.md)。
- MVP 冻结范围与非目标：`docs/mvp/scope.md`
- 仓库分层与模块边界：`docs/architecture/overview.md`
- 长期能力地图、弱耦合边界与模拟 / 求解方式：`docs/architecture/simulation-platform.md`
- App / Canvas / UI 边界：`docs/architecture/app-architecture.md`、`docs/architecture/canvas-interaction-contract.md`
- 热力学与闪蒸契约：`docs/thermo/mvp-model.md`
- CAPE-OPEN / COM 边界：`docs/capeopen/boundary.md`
- 代码风格、命名和抽象判断：`docs/development/code-style.md`
- 分支与 PR 治理：`docs/adr/0001-branch-and-pr-governance.md`
- 单位 / 建模辅助 / 设备设计的详细规划见本文当前结论链接；分离器与换热器子专题从设备专题进入。UI 功能分区、旧稿状态和 R0—R2 见 UI 计划。
- 最新周志：[W44](../devlogs/2026-10/2026-W44.md)；AF 已实现证据见 W43，设备基础 / UI 见 W42，U / A 见 W41。

## 更新规则

- 本文档只维护当前状态、当前优先级、临时验证基线、当前不推进项和最近必要事实。
- 长期协作规则进入 Agent 协作指南；功能和领域细节进入专题；历史过程和命令级证据进入周志或记录。
- 完成重要开发或维护步骤后，先更新对应专题或治理文档，再同步本文档中仍需保留的当前摘要。
