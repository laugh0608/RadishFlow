# 当前状态

更新时间：2026-09-12

## 用途

用途：为新会话恢复当前维护状态、优先级、临时门禁和“当前不做”提供最小入口。

读者：仓库维护者、用户与 AI / Agent。

不包含：完整历史流水、详细功能设计、长期协作规则、命令级测试日志和发布说明。

默认先读本文档。需要处理具体领域时，再按“按需阅读”进入对应专题；`AGENTS.md` / `CLAUDE.md` 只保留跨任务、跨阶段且必须启动即生效的长期约束。

## 当前结论

- 自 2026-09-12 起，项目所有者明确恢复 RadishFlow 正常开发和迭代；此前的业务停更、仅限外围维护、拒绝产品问题与外部业务贡献等限制取消。
- 模拟核心、物性与单元模型、Studio、项目生命周期、CAPE-OPEN / COM 适配、测试和发布准备均可按项目规划推进；产品问题、功能建议和外部贡献按贡献流程评审。
- MVP 第一阶段 M1-M5、MVP α / β 与通用小流程建模 v1 已阶段性收口，后续迭代以现有能力与验证证据为起点。
- 当前处于 MVP 后续迭代规划与可靠性完善阶段。专题范围、状态与验收见 `docs/topics/`；具体优先级统一由本文管理，历史 Sprint 不直接作为今日任务。
- 当前尚未进入正式 tag / release 节点；历史 `v26.5.1-dev` 只作为内部 staging 草案和验证记录保留。

## 当前开发范围与优先级

### 正常推进范围

- 按专题设计、实现和验证模拟功能、物性模型、单元模块及 Studio 用户路径。
- 修复项目保存恢复、原生句柄生命周期、跨平台兼容与结果诊断等已知问题。
- 按需求评审账户、控制面、发布与安装能力；进入实现前明确依赖、接口、兼容性和验收标准。
- 持续维护文档、CI、仓库治理、安全基线和固定工具链，为产品迭代提供验证基础。

### 当前优先级

1. 明确下一阶段的目标用户、体系与工况，定义物性来源、适用范围和数值验收标准；据此选择一个可独立交付的功能切片。
2. 优先复现并处理已知可靠性风险：Windows 保存失败后的恢复信息与 native 句柄生命周期；使用故障注入、释放与并发调用测试建立证据。
3. 围绕选定场景完善建模、求解、结果诊断和保存 / 重开的完整用户路径；数值能力按独立基准与守恒要求验收。
4. 保持跨平台工具链与 CI 基线，补充远端运行和按需 `stable` 兼容性验证；发布前另行完成平台、GUI 与 PME 验收。

下一轮从具体切片的范围、非目标和退出标准开始，按正常协作流程进入实现。账户联合登录仍为待架构决策的 Draft；恢复开发不代表所有 Backlog 已同时排期，也不表示任何缺失能力已经交付。

## 当前能力基线

以下能力构成后续迭代的回归基线：

- 受控流程可覆盖 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum`，以及显式输入、运行、保存、重开、rerun 和轻量结果审阅。
- Studio 已形成 Home、独立物性页、流程图工作台、模块设置 / 结果和运行状态区域的主路径。
- Rust Core 与 `.NET 10` 适配层保持隔离；CAPE-OPEN / COM 语义不进入 Rust Core。
- 数值实现仍使用演示物性、常热容显热和简化单元行为；golden、结果透传与用户路径验收不等于独立物理准确性证明，见 [热力学模型](../thermo/mvp-model.md)。
- 更详细的能力、状态命令和验收证据以 `docs/topics/`、`docs/mvp/`、`docs/reference/` 与历史周志为准。

## 已知差异与风险路由

- 架构目标与实际实现差异：`rf-thermo` 缓存装载依赖、`rf-canvas` 占位和 Studio 同步求解见 [架构总览](../architecture/overview.md)。
- 原生句柄释放与并发调用静态风险见 [CAPE-OPEN 专题](../topics/capeopen-pmc-adapter.md)。
- Windows 保存替换与回滚双重失败的静态风险见 [项目生命周期专题](../topics/project-lifecycle-storage.md)。

上述风险尚未修复，现有静态审阅不等于故障复现。后续实现与验收顺序见 [迭代路线图](../radishflow-mvp-roadmap.md#后续迭代顺序)。

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

- 2026-05-28：真实环境 `pwsh ./scripts/check-repo.ps1` 通过。
- 2026-06-10、2026-08-20、2026-08-22：`./scripts/check-repo.sh` 通过。
- 2026-09-06：macOS、Rust / Cargo 1.96.0，保持 Cargo 离线的 `./scripts/check-repo.sh` 通过，1,122 项 Rust 测试通过；未执行 GUI、Windows `.NET` / COM / PME 复验，详细记录见 [2026-W36](../devlogs/2026-09/2026-W36.md)。
- 2026-09-12：固定 Rust 1.96.0 后，macOS 真实环境离线 `check-repo` 全部通过（含 1,122 项测试与严格 clippy）；CI 配置完成静态复核，远端三平台运行尚未执行，详见 [2026-W37](../devlogs/2026-09/2026-W37.md)。
- 2026-09-12：提交 `a5f8d966` 在 UTM Debian 13.6 ARM64 与 Windows 11 ARM64 中串行完成 `check-repo`，两端各 1,120 项测试与严格 clippy 通过；Windows 补齐 Clang 19.1.5 后在现有 Windows PowerShell 5.1 中通过，不替代 CI 的 PowerShell 7 / x64 runner 或 `.NET` / COM / PME 验证，环境与证据见 [2026-W37](../devlogs/2026-09/2026-W37.md#2026-09-12-utm-跨平台复验)。

## 当前范围控制

- 第三方 CAPE-OPEN 模型、完整 Thermodynamics PMC、动态模拟、CFD 等仍属当前 MVP 范围之外；需要时按专题评审扩展，不自动并入本轮迭代。
- 控制面、账户登录、完整报表和安装器按实际需求与前置条件安排，不同时作为默认开发主线。
- 业务开发恢复不等于创建 tag、发布产物或部署服务；这些操作按相应验收与发布流程推进。
- Rust Core 不承载 COM 语义；新增抽象需服务真实领域职责与维护收益。

## 按需阅读

- 文档总索引与篇幅治理：`docs/README.md`
- Agent 协作、授权与验证选择：`docs/development/agent-collaboration.md`
- 开发专题与能力索引：`docs/topics/README.md`
- 未排期账户与联合登录规划：`docs/topics/platform/account-and-federated-login.md`；实现前完成 M0 范围与架构决策。
- MVP 冻结范围与非目标：`docs/mvp/scope.md`
- 仓库分层与模块边界：`docs/architecture/overview.md`
- App / Canvas / UI 边界：`docs/architecture/app-architecture.md`、`docs/architecture/canvas-interaction-contract.md`
- 热力学与闪蒸契约：`docs/thermo/mvp-model.md`
- CAPE-OPEN / COM 边界：`docs/capeopen/boundary.md`
- 代码风格、命名和抽象判断：`docs/development/code-style.md`
- 分支与 PR 治理：`docs/adr/0001-branch-and-pr-governance.md`
- 最新历史流水：`docs/devlogs/2026-09/2026-W37.md`

## 更新规则

- 本文档只维护当前状态、当前优先级、临时验证基线、当前不推进项和最近必要事实。
- 长期协作规则进入 Agent 协作指南；功能和领域细节进入专题；历史过程和命令级证据进入周志或记录。
- 完成重要开发或维护步骤后，先更新对应专题或治理文档，再同步本文档中仍需保留的当前摘要。
