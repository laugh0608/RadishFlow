# ADR 0001: Branch And PR Governance

更新时间：2026-08-29

## 状态

Accepted

## 背景

当前仓库已经从“空骨架”进入“逐步可开发”的阶段，但分支治理、PR 流程和自动化检查仍未建立。  
如果继续在 `master` 上直接累积提交，后续很难形成稳定的发布基线，也不利于多人或多代理协作。

## 决策

仓库采用以下分支与 PR 治理策略：

### 分支角色

- `master` / `main`: 稳定主线，只接受 Pull Request 合并；当前仓库仍以 `master` 为主，若后续切换默认分支则同一规则迁移到 `main`
- `dev`: 常态开发与集成分支，允许直接承载串行推进的维护提交
- `feature/*`: 需要独立隔离或评审时使用的功能开发分支
- `docs/*`: 需要独立隔离或评审时使用的文档与规范分支
- `chore/*`: 需要独立隔离或评审时使用的基础设施、脚本、CI、仓库治理分支

### 合并策略

- 串行推进的普通维护默认直接在 `dev` 完成，不为了流程形式自动创建主题分支、Pull Request 或额外 worktree
- 项目所有者明确要求、外部贡献、并行写入、风险隔离或明确评审需求时，使用主题分支并向 `dev` 发 Pull Request；Agent 不自动创建 `codex/*` 等临时分支
- 阶段性稳定后，再通过 PR 将 `dev` 合并到 `master` / `main`
- `dev -> master/main` 的阶段性 PR 优先使用 merge commit，使合并后的稳定主线可以直接 fast-forward 回灌 `dev`；仓库仍允许 rebase merge，但必须承担后续普通 merge 回灌
- 仅在必须修复主线问题时，才允许 `hotfix/*` 直接向 `master` / `main` 发 PR

### `master` / `main` -> `dev` 合并后回灌

- 任何 PR 合并到 `master` / `main` 后，都必须把最新 `origin/master` / `origin/main` 同步回 `dev`；这包括 `dev -> master/main` 阶段性集成和 `hotfix/* -> master/main` 紧急修复
- 回灌是稳定主线 PR 的必需收口动作；在回灌完成前，应暂停向 `dev` 追加下一轮提交，避免人为制造不必要的分叉
- 最新稳定主线是 `dev` 的后继且可快进时，优先使用 fast-forward；因 rebase merge 或并发提交无法快进时，使用普通 merge 并完成与差异相称的验证
- 禁止通过 rebase、reset、force push 或重写既有提交来伪造已同步状态
- 回灌完成后再推送 `dev`，并确认 `dev` 已包含最新稳定主线；该动作不代表创建 tag、发布 artifact、GitHub Release 或生产部署

### `master` / `main` 规则

- 禁止直接 push
- 必须通过 PR 合并
- 必须通过仓库检查和 staging package 验证
- 当前允许 `merge commit` 与 `rebase merge`，禁用 `squash merge`
- 管理员仅可通过 PR 方式绕过规则
- 单人维护阶段不要求额外审批，但仍要求已解决会话

### `dev` 规则

- 作为当前阶段常态开发与集成分支，允许串行推进的普通维护直接提交
- 当前阶段不启用分支保护
- 普通 `push -> dev` 不自动触发 CI/CD
- 目标为 `dev` 的 Pull Request 自动运行 `PR Checks`，为其他开发者提供合并前反馈；`dev` 当前不启用 required checks 或 branch protection
- 直接进入共享 `dev` 的连续开发仍在本地按改动风险执行 focused test 或仓库级验证；完整强制门禁统一放到 `dev -> master/main` 稳定化 PR
- 接受 `master` / `main` 合并结果的回灌；回灌完成前不开始下一轮集成开发

## 需要在 GitHub 仓库设置中完成的动作

以下规则不能仅靠仓库文件完全强制，需要仓库管理员在 GitHub Settings 中启用：

1. 创建远端 `dev` 分支
2. 将默认分支切换为 `dev`，或至少把开发 PR 默认目标改为 `dev`
3. 对 `master` / `main` 启用 branch protection
4. 要求 `master` / `main` 通过聚合状态检查 `Candidate Quality`
5. 对 `master` / `main` 开启 “Require a pull request before merging”
6. 仓库 Merge options 中启用 `Merge commits` 与 `Rebase merging`，关闭 `Squash merging`
7. 配置管理员仅通过 PR 绕过，不开放直接 push
8. `dev` 当前不配置 branch protection

## 仓库内已落地的支撑项

为配合该决策，仓库内已同步增加：

- PR 模板
  - 面向 `master` / `main` 时要求明确合并后回灌 `dev` 的执行人和时机
- 社区与安全治理入口
  - 根目录 `CONTRIBUTING.md`、`CODE_OF_CONDUCT.md` 与 `SECURITY.md` 明确停更、许可、行为和私密漏洞报告边界
  - `.github/ISSUE_TEMPLATE/config.yml` 关闭普通空白 Issue，并只把安全问题引导到已启用的 GitHub Private Vulnerability Reporting
- GitHub Actions PR 检查工作流
  - `PR Checks` 在目标分支为 `dev`、`master` 或 `main` 的 Pull Request 上自动触发；目标为 `dev` 的 PR 提供合并前反馈，默认分支 PR 用于阶段稳定化合并，普通 `dev` push 不触发
  - 当前拆分为 `Repo Hygiene`、三平台 `Rust Baseline`、`.NET Adapter Baseline` 与 `Windows Staging Package` 六个组件，并由 `Candidate Quality` 聚合收口
  - `Candidate Quality` 使用 `if: always()` 汇总六个组件，任一组件失败、取消或跳过都会失败；`master` / `main` ruleset 只绑定该稳定 context
  - 规范 tag push 暂不自动触发 CI/CD；`Release Checks` 仅保留 `workflow_dispatch` 手动 staging 入口，避免普通内部 staging 或历史 tag 造成误发布信号
- 文本编码与文件格式检查脚本
  - 正式实现源收口到 Rust `xtask`，`.ps1` 与 `.sh` 仅作为平台包装层
- 仓库治理检查
  - Rust `xtask` 统一检查必需治理文件、仓库自有 Markdown 相对链接、JSON、`AGENTS.md` / `CLAUDE.md` 同步、Issue / ruleset / workflow / PR 模板契约和 `git diff --check`
  - 本地 `check-repo` 检查工作区与暂存区差异；PR 的 `Repo Hygiene` 使用 base ref 检查完整 PR 差异
  - `adapters/reference/` 下的外部参考资料保留上游状态，不纳入 Markdown 链接与 JSON 治理检查
- Rust workspace 基础校验入口
- Windows `.NET 10` CAPE-OPEN baseline 入口：`scripts/check-dotnet-capeopen.ps1` 负责 native build、`.NET` solution build、contract tests 和 smoke tests；不执行 COM 注册、反注册或注册表写入
- Windows portable staging package 入口：`scripts/package.ps1` 只产出 workflow artifact，不创建 GitHub Release，也不发布安装包
- `master` / `main` ruleset 模板

## 影响

正面影响：

- `master` / `main` 可以保持稳定
- `dev` 可以作为当前阶段的真实日常开发面，不被日常 CI 噪声阻塞
- 每次稳定主线合并结果都会回到 `dev`，避免稳定主线 ancestry 与下一批开发长期分叉
- 文档、规范、脚本和代码都能纳入统一 PR 检查
- `.NET` CAPE-OPEN 适配层和 Windows staging package 不再游离于默认分支合并门禁之外
- 单人开发阶段不再依赖管理员 bypass 完成常规 PR 合并

代价：

- 需要维护远端 `master` / `main` 保护设置
- 直接在 `dev` 连续维护时，需要维护者按改动风险主动执行并如实记录本地验证
- 每次合入 `master` / `main` 后增加一次明确的稳定主线回灌动作
- 默认分支 PR 检查时间会增加，尤其是 Windows `.NET` baseline 和 staging package job

## 后续约束

- 若后续调整分支角色、PR 目标分支、required checks、回灌策略或稳定主线保护规则，应同步更新本 ADR、根协作入口、贡献指南、PR 模板和 ruleset 说明
- 稳定主线 PR 未完成回灌时，不把该批次视为拓扑收口完成；发现遗漏应先补回灌，再继续下一批开发
- 新增同类治理规则时，继续采用“仓库文件声明 + 自动契约检查 + GitHub 设置落地 + ADR 固化”的方式，避免只靠口头约定
