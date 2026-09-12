# 当前状态

更新时间：2026-09-12

## 用途

用途：为新会话恢复当前维护状态、优先级、临时门禁和“当前不做”提供最小入口。

读者：仓库维护者、用户与 AI / Agent。

不包含：完整历史流水、详细功能设计、长期协作规则、命令级测试日志和发布说明。

默认先读本文档。需要处理具体领域时，再按“按需阅读”进入对应专题；`AGENTS.md` / `CLAUDE.md` 只保留跨任务、跨阶段且必须启动即生效的长期约束。

## 当前结论

- 自 2026-06-12 起，RadishFlow 业务功能开发保持停止；不再推进模拟功能、物性模型、CAPE-OPEN / COM 适配、产品路线、发布能力或对外支持。
- 当前只按需维护不扩张产品能力的仓库外围基础设施，包括文档治理、CI、ruleset、安全基线、仓库元数据和工具链兼容性；外围维护不表示恢复产品开发。
- MVP 第一阶段 M1-M5、MVP α、MVP β 人工 smoke、失败修复闭环和通用小流程建模 v1 已在停更前阶段性收口，现有专题与验收材料作为能力和边界记录保留。
- `docs/topics/` 中原有 Active / Backlog 状态记录停更前的开发组织，不代表当前仍有激活开发主线；当前维护状态以本文档和根 `README.md` 为准。
- 当前尚未进入正式 tag / release 节点；历史 `v26.5.1-dev` 只作为内部 staging 草案和验证记录保留。

## 当前维护范围

### 可以按需推进

- 修正文档真相源、索引、篇幅、链接、编码和协作入口职责。
- 维护 CI、ruleset、PR 模板、社区健康文件、安全报告入口和仓库治理检查。
- 修复不改变产品能力的工具链兼容性、构建环境或仓库元数据问题。
- 处理仓库所有者明确授权、且不突破停止公开业务功能维护边界的其他外围事项。

### 当前优先级

1. 保持维护状态、实际能力、模型限制和验证含义一致；修正架构、专题与使用入口的过期描述，避免把历史规划解释为当前授权。
2. 保持固定工具链、最低支持版本声明与锁定依赖一致；按需手动检查新版 `stable` 兼容性，工具链说明见下文。
3. 保持仓库治理脚本、CI 契约、社区健康文件与分支规则一致；外围维护采用定向验证，阶段收口回到正式仓库级入口。

当前没有排期中的产品功能切片。若未来需要改变停更边界，应先由仓库所有者明确决策，并同步更新根 `README.md`、本文档和受影响专题；不得仅通过修改某篇专题的状态恢复开发。

## 已保留能力基线

以下内容只描述停更前已经形成的能力，用于理解代码和历史专题，不构成继续开发计划：

- 受控流程可覆盖 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum`，以及显式输入、运行、保存、重开、rerun 和轻量结果审阅。
- Studio 已形成 Home、独立物性页、流程图工作台、模块设置 / 结果和运行状态区域的主路径。
- Rust Core 与 `.NET 10` 适配层保持隔离；CAPE-OPEN / COM 语义不进入 Rust Core。
- 数值实现仍使用演示物性、常热容显热和简化单元行为；golden、结果透传与用户路径验收不等于独立物理准确性证明，见 [热力学模型](../thermo/mvp-model.md)。
- 更详细的能力、状态命令和验收证据以 `docs/topics/`、`docs/mvp/`、`docs/reference/` 与历史周志为准。

## 已知差异与风险路由

- 架构目标与实际实现差异：`rf-thermo` 缓存装载依赖、`rf-canvas` 占位和 Studio 同步求解见 [架构总览](../architecture/overview.md)。
- 原生句柄释放与并发调用静态风险见 [CAPE-OPEN 专题](../topics/capeopen-pmc-adapter.md)。
- Windows 保存替换与回滚双重失败的静态风险见 [项目生命周期专题](../topics/project-lifecycle-storage.md)。

上述业务实现风险尚未修复；本次文档审阅不等于 Windows 故障复现，也不授权突破停更边界。未排期的后续投资顺序只作为 [路线图决策参考](../radishflow-mvp-roadmap.md#后续决策参考未排期)。

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

## 当前不推进

- 不新增或扩张模拟功能、物性模型、单元模块、自由连线、完整报表或控制面能力。
- 不引入第三方 CAPE-OPEN 模型、第三方物性包、完整组分数据库或完整 Thermodynamics PMC。
- 不推进 tag、release notes、便携包刷新、安装器、发布自动化或对外推广。
- 不恢复公开 Bug、功能提案、产品问题处理、外部业务功能 PR 合并或支持承诺。
- 不把 CAPE-OPEN / COM 语义倒灌到 Rust Core。
- 不为未来可能需求预先堆叠不明意义的 helper、manager、orchestrator、context 或 adapter。

## 按需阅读

- 文档总索引与篇幅治理：`docs/README.md`
- Agent 协作、授权与验证选择：`docs/development/agent-collaboration.md`
- 停更前专题与能力索引：`docs/topics/README.md`
- 未排期账户与联合登录规划：`docs/topics/platform/account-and-federated-login.md`；待架构决策，不恢复业务实现。
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
- 完成重要外围维护后，先更新对应专题或治理文档，再同步本文档中仍需保留的当前摘要。
