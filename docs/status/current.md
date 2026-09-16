# 当前状态

更新时间：2026-09-16

## 用途

用途：为新会话恢复当前维护状态、优先级、临时门禁和“当前不做”提供最小入口。

读者：仓库维护者、用户与 AI / Agent。

不包含：完整历史流水、详细功能设计、长期协作规则、命令级测试日志和发布说明。

默认先读本文档。需要处理具体领域时，再按“按需阅读”进入对应专题；`AGENTS.md` / `CLAUDE.md` 只保留跨任务、跨阶段且必须启动即生效的长期约束。

## 当前结论

- 自 2026-09-12 起，项目所有者明确恢复 RadishFlow 正常开发和迭代；此前的业务停更、仅限外围维护、拒绝产品问题与外部业务贡献等限制取消。
- 模拟核心、物性与单元模型、Studio、项目生命周期、CAPE-OPEN / COM 适配、测试和发布准备均可按项目规划推进；产品问题、功能建议和外部贡献按贡献流程评审。
- MVP 第一阶段 M1-M5、MVP α / β 与通用小流程建模 v1 已阶段性收口，后续迭代以现有能力与验证证据为起点。
- 2026-09-13 起，近期主线调整为基础功能与使用闭环完善；独立数值研究不作为编辑、保存等基础功能的统一前置门禁，基本正确性与可靠性仍随功能验证。
- 长期覆盖稳态、动态、瞬态、间歇 / 半间歇，以及递归分块、序贯模块、联立模块和联立方程；统一 API 明确采用变量 / 动作浏览树，支持 COM 自动化和操作录制 / 回放。领域与术语边界见 [模拟平台长期规划](../architecture/simulation-platform.md)；目标不表示已实现或同时排期。
- 2026-09-14 确认补入插件系统、参考 RadishNexus 的分层许可与公共 API 授权 / 配额规划；基础功能优先级不变，插件运行时、具体许可政策、接口和部署仍待切片决策，见 [路线图](../radishflow-mvp-roadmap.md#插件与公共-api-的实施切片)。
- 2026-09-16 确认 [单位系统](../topics/units-and-quantity-system.md)、[规格分析与确定性辅助](../topics/modeling-assistance-and-specifications.md)、[设备设计与校核](../topics/equipment-design-and-rating.md) 规划。B3-5 后优先单位与规格基础；设备工程不等待完整动态；未来 Agent 复用统一入口，当前不接入 AI。
- 同日补充 [工程基础、工况与来源](../topics/engineering-basis-and-cases.md)、目标反算、工程检查、压力 / 公用工程、候选权衡与交付。G0 优先纳入近期设计，G1 / G2 随单位、规格和设备推进，详见 [配套切片](../radishflow-mvp-roadmap.md#工程工作流的配套切片)。
- Studio 旧 `.pen` 保留，新增功能分区需按 [UI 计划](../architecture/studio-ui-topic-plan.md) 重新评审；本轮没有修改设计文件或完成新稿验收。
- 当前尚未进入正式 tag / release 节点；历史 `v26.5.1-dev` 只作为内部 staging 草案和验证记录保留。

## 当前开发范围与优先级

### 正常推进范围

- 按专题设计、实现和验证模拟功能、物性模型、单元模块及 Studio 用户路径。
- 修复项目保存恢复、原生句柄生命周期、跨平台兼容与结果诊断等已知问题。
- 按需求评审账户、控制面、发布与安装能力；进入实现前明确依赖、接口、兼容性和验收标准。
- 持续维护文档、CI、仓库治理、安全基线和固定工具链，为产品迭代提供验证基础。

### 当前优先级

1. 完善基础功能：核对组分 / 物性选择、对象与连接编辑、参数、运行反馈、结果审阅、undo / redo、保存 / 重开。按 [Studio 基础功能切片](../topics/studio-main-workflow.md#基础功能完善切片) 建立实现与验收清单，先补普通用户操作的实际缺口。
2. B3-5 已接通，下一步推进单位元数据 / 转换、输入显示和现有输出，再完善现有单元的规格检查与确定性推荐；切片见 [路线图](../radishflow-mvp-roadmap.md#单位辅助设备与-ui-的实施顺序)。UI 信息架构与局部交互评审随切片衔接，纯领域基础不等待整套重绘。
3. 随主路径需求完善运行与扩展基础：逐步建立变量 / 动作元数据与浏览树、统一命令、COM / 无界面调用、操作录制与回放，并明确任务、取消、进度和过期结果处理。接口服务实际操作，不预建空框架。
4. 基础闭环稳定后集中推进数值与工程能力，先按递归分块与循环块扩展序贯求解；具体模型明确工况、独立基准与鲁棒性要求；按设备需求补物性，先评估分离器初步尺寸和换热器热工闭环。联立模块 / 联立方程、动态、瞬态、间歇与 APC 研究按依赖逐步实施。
5. 持续处理可靠性与跨平台问题。第一轮 Flash、保存恢复与 native 生命周期修复已收口；保持对应回归、固定工具链和 CI，发布前另行完成平台、GUI 与 PME 验收。

2026-09-13 的 B0 / B1 主路径基线见 [建模专题](../topics/flowsheet-modeling-and-solve.md) 和 [生命周期专题](../topics/project-lifecycle-storage.md)。账户联合登录仍为待架构决策的 Draft。

### 最近实现进展（2026-09-16）

- B2-5 [Cooler 空白建模与温压结果闭环](../topics/unitops/heater-cooler.md#b2-5cooler-空白建模与温压结果闭环) 已完成 macOS 实窗与文件读回：显式温压、非法草稿与上游联动、Undo / Redo、失败定位修复、保存重开重跑及当前结果输出通过。

- B3-1 只读变量 / 动作浏览和 B3-2 统一变量写入已完成，支持稳定身份、修订检查、共享输入事务及 current / stale / missing 查询；对应实窗与保存重开回归见 [W38](../devlogs/2026-09/2026-W38.md)。

- B3-3 受控创建 / 连接 / 运行和 B3-4 [首个无界面消费者](../reference/headless-cli.md) 已实现：统一事务、实时候选和运行诊断供应用调用；CLI 支持查询已有工程、写入 SI 参数、同步求解与读取当前结果，保留原磁盘输入。macOS 离线 `check-repo` 通过 1,199 项测试与严格 clippy，详见 [W38](../devlogs/2026-09/2026-W38.md#2026-09-15-b3-4-首个无界面参数化运行消费者)。

- B3-5 无界面创建 / 连接已接通：v2 有序步骤复用受控动作与变量写入，返回对象身份供后续引用；步骤失败保留回执并停止，成功后求解 / 读取，v1 和磁盘输入保持兼容。验证见 [W38](../devlogs/2026-09/2026-W38.md#2026-09-16-b3-5-无界面建模闭环)。

### 下一步

- [ ] 优先 U1：明确量 / 单位目录、统一转换、现有元数据迁移和显示偏好所有权，保持 SI 请求及项目兼容；先验证温压流量、温差及非法单位。
- [ ] 再按 U2—U3、A1—A2 推进输入 / 输出与规格基础；E0 / S0 / H0 明确首个设备工况与独立基准。具体接口、存储及依赖按切片审定。
- [ ] G0 优先明确设计基础、工况矩阵和输入来源，并纳入 R0 的项目 / 工况 / 设备 / 结果关系；代码切片顺序保持。
- [ ] R0 先评审 UI 功能分区，R1 优先单位 / 规格交互，再设备工作区；新 `.pen` 和相关代码尚未完成。

### 今日规划整理（2026-09-16）

- 已贯通平台目标、路线图、三个一级专题和分离器 / 换热器子专题，更新当前消费者与 UI 重设计边界。现有 P1 / P2 实现及 P3 数值限制不变；验证与文件范围见 [W38](../devlogs/2026-09/2026-W38.md#2026-09-16-单位建模辅助设备工程与-ui-规划整理)。

### 后续事项

- [ ] 按项目所有者 2026-09-15 要求，Windows / Linux 原生键盘、保存 / 输出等路径累积到后续阶段节点集中验证；远距离视口按实际缺口推进。未测项保留，P1 不宣称全平台收口。
- 真实窗口前先告知，UTM 串行且最多一台运行；未测平台保持待验证。
- 插件运行时、具体许可政策、公共 API 与完整报告继续按独立规划推进。

## 当前能力基线

以下能力构成后续迭代的回归基线：

- 受控流程可覆盖 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum`，以及显式输入、运行、保存、重开、rerun 和轻量结果审阅。
- Studio 已形成 Home、独立物性页、流程图工作台、模块设置 / 结果和运行状态区域的主路径。
- Rust Core 与 `.NET 10` 适配层保持隔离；CAPE-OPEN / COM 语义不进入 Rust Core。
- 数值实现仍使用演示物性、常热容显热和简化单元行为；golden、结果透传与用户路径验收不等于独立物理准确性证明，见 [热力学模型](../thermo/mvp-model.md)。
- 更详细的能力、状态命令和验收证据以 `docs/topics/`、`docs/mvp/`、`docs/reference/` 与历史周志为准。

## 已知差异与风险路由

- 架构目标与实际实现差异：`rf-thermo` 缓存装载依赖、`rf-canvas` 占位和 Studio 同步求解见 [架构总览](../architecture/overview.md)。
- 原生句柄释放与并发调用契约及验证范围见 [CAPE-OPEN 专题](../topics/capeopen-pmc-adapter.md)。
- Windows 保存替换与回滚失败的恢复契约见 [项目生命周期专题](../topics/project-lifecycle-storage.md)。

本轮可靠性修复及平台验证状态见最新周志；模型、架构和发布的其他已知边界继续按各专题管理。后续实现与验收顺序见 [迭代路线图](../radishflow-mvp-roadmap.md#后续迭代顺序)。

## 工具链现状

workspace 最低支持版本声明为 Rust `1.96`，`rust-toolchain.toml`、`PR Checks` 与 `Release Checks` 固定使用 `1.96.0`。这是仓库选择维护的支持下限与验证基线，不表示更低版本必然无法编译；不承诺未验证的低版本兼容性。

2026-09-12 核对完整锁定依赖图共 469 个包，外部依赖已声明的最高 Rust 要求为 `1.88.0`（`image`、`time`、`time-core`、`time-macros`），另有 127 个包未声明最低版本。元数据不构成完整 workspace 在 Rust 1.88 上可构建的证据；原有 `1.86` 声明已撤回，`Cargo.lock` 保持不变。

正式 Rust 检查与 CI 构建入口使用 `--locked`；浮动 `stable` 由独立的 `Rust Compatibility` 手动工作流在三平台检查，不进入 `Candidate Quality`，不自动升级固定版本。维护步骤与复现范围见 [工具链维护规则](../development/agent-collaboration.md#rust-工具链与锁定依赖)。

macOS 现有图形依赖链中的 `block 0.1.6` 仍有未来 Rust 兼容性警告；固定工具链不等于完成依赖升级，也不保证未来编译器兼容。

## 当前验证基线

- 文档、协作入口或仓库治理改动：执行 `cargo test --locked -p xtask` 与 `cargo run --locked --quiet -p xtask -- check-repository-governance`，并检查文档体量和工作区差异。
- 阶段收口或跨模块治理变化：在 macOS / Linux / CI 执行 `./scripts/check-repo.sh`；Windows 使用 `pwsh ./scripts/check-repo.ps1`。
- `check-repo` 是正式仓库级入口，统一执行治理与文本门禁、Rust workspace 格式、构建、测试和 clippy 基线。
- `adapters/reference/` 下的外部参考资料保留上游编码、BOM 和换行格式，不为通过仓库文本门禁而批量改写。
- 重要验证若出现明显的沙盒权限、受限 restore、project reference 解析或 native 装载差异，先按 Agent 协作规则告知用户，再申请最小范围真实环境复验。

最近通过记录：

- 2026-09-16：B3-5 通过 macOS 离线全仓基线，1,209 项测试与严格 clippy；含 16 项 CLI 进程回归。使用进程级 Command Line Tools；本地 HTTP 测试经沙盒外复验通过，详见 W38。

- 2026-09-15：B2-5、B3-1 至 B3-4 分别通过 macOS 离线全仓基线，最终 1,199 项测试与严格 clippy；Cooler / 变量浏览有实窗证据，B3-4 有 8 项 CLI 进程回归。其他平台原生未测，见 [W38](../devlogs/2026-09/2026-W38.md)。
- 先前固定工具链、macOS / UTM Windows ARM64 / Linux 基线、第一轮 native 可靠性及 `.NET` 构建 / contract / smoke 证据见 [W37](../devlogs/2026-09/2026-W37.md)，更早全仓记录见 [W36](../devlogs/2026-09/2026-W36.md)。它们不替代本轮新增路径的跨平台、COM / PME 或发布验收。

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
- 最新历史流水：`docs/devlogs/2026-09/2026-W38.md`；基础编辑与实窗证据继续见 `2026-W37.md`。

## 更新规则

- 本文档只维护当前状态、当前优先级、临时验证基线、当前不推进项和最近必要事实。
- 长期协作规则进入 Agent 协作指南；功能和领域细节进入专题；历史过程和命令级证据进入周志或记录。
- 完成重要开发或维护步骤后，先更新对应专题或治理文档，再同步本文档中仍需保留的当前摘要。
