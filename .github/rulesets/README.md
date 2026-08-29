# GitHub Rulesets

本目录存放 RadishFlow 的仓库规则模板。  
当前维护默认分支 `master` / `main` 的保护规则，`dev` 作为常态开发分支，不启用强制保护，也不自动触发 CI。

## 建议流程

1. 日常开发提交到 `dev` 或功能分支
2. 串行普通维护直接在 `dev` 推进；只有外部贡献、并行写入、风险隔离或明确评审需求才通过主题分支合入 `dev`
3. 阶段性稳定后，再从 `dev` 发起到默认分支（当前为 `master`，如切换可适配 `main`）的 Pull Request
4. 默认分支 PR 必须通过仓库检查和 staging package 验证
5. 合并到默认分支后，先把最新默认分支回灌并推送到 `dev`；可快进时优先 fast-forward，否则使用普通 merge
6. 完成回灌后，再独立决定是否创建 tag、发布 artifact、GitHub Release 或执行部署
7. 管理员如需绕过规则，也只能通过 Pull Request，不开放直接 push

## 默认分支规则说明

- 禁止直接推送到受保护的默认分支（`master` / `main`）
- 禁止 force push
- 禁止删除分支
- 仅允许通过 Pull Request 合并
- 单人维护阶段不要求额外审批，但仍要求已解决会话
- 仅要求聚合检查 `Candidate Quality` 通过；它会汇总 `Repo Hygiene`、Linux `Rust Baseline`、`Rust Baseline (macOS)`、`Rust Baseline (Windows)`、`.NET Adapter Baseline` 与 `Windows Staging Package` 六个组件结果
- `PR Checks` 保留六个独立组件便于定位失败，并由稳定的 `Candidate Quality` 统一收口；任一组件失败、取消或跳过都会使聚合检查失败
- `PR Checks` 响应 `pull_request -> dev/master/main`；目标为 `dev` 的 PR 为其他开发者提供合并前反馈，目标为默认分支的 PR 承担阶段稳定化门禁，普通 `dev` push 不触发
- `Release Checks` 当前只保留 `workflow_dispatch` 手动 staging 入口；tag push 不自动触发 CI/CD，避免普通内部 staging 或历史 tag 造成误发布信号
- 允许 `merge` 与 `rebase` 两种合并方式，禁用 `squash`；`dev -> master/main` 优先使用 merge commit，以便稳定主线直接 fast-forward 回灌 `dev`
- 管理员仅可通过 Pull Request 方式绕过规则，不开放直接 push

## dev 策略说明

- `dev` 是当前常态开发分支
- 当前阶段不启用 branch protection
- 当前默认不要求 push 到 `dev` 时自动触发仓库检查
- 目标为 `dev` 的 Pull Request 自动运行 `PR Checks`，但 `dev` 当前不启用 required checks 或 branch protection；直接进入共享 `dev` 的连续开发仍按改动风险执行本地验证
- Agent 不因默认流程自动创建 `codex/*` 主题分支或额外 worktree
- `dev` 接受稳定主线合并结果的回灌；回灌完成前不开始下一轮集成开发
- 如后续进入多人并行开发，再评估是否对 `dev` 追加保护

## 检查入口

- `scripts/check-repo.ps1` 与 `scripts/check-repo.sh` 当前复用同一套 Rust `xtask` 实现；除 Rust workspace 基线外，还执行必需治理文件、Markdown 相对链接、JSON、协作文件同步、GitHub 配置契约和 diff whitespace 检查
- `PR Checks` 的 `Repo Hygiene` 会针对 PR base ref 单独执行治理检查，确保 `git diff --check` 覆盖完整 PR 差异；`Release Checks` 在手动 staging 前执行同一治理基线
- `scripts/check-dotnet-capeopen.ps1` 当前作为 Windows `.NET 10` CAPE-OPEN baseline 入口，负责 `rf-ffi` native build、`.NET` solution build、contract tests 和 smoke tests；不执行 COM 注册、反注册或注册表写入
- `scripts/package.ps1` 当前只产出 Windows portable staging package；CI 上传 workflow artifact，不创建 GitHub Release，也不发布安装包
- CI 当前在 Linux / macOS runner 上使用 `.sh` 入口，在 Windows runner 上使用 `.ps1`

## 应用方式

如果仓库还没有对应 ruleset，可以使用 GitHub CLI 或 REST API 导入：

```bash
gh api repos/<owner>/<repo>/rulesets --method POST --input .github/rulesets/master-protection.json
```

如果仓库中已存在旧 ruleset，建议改用 `PUT /repos/{owner}/{repo}/rulesets/{ruleset_id}` 更新。

本目录模板还包含 Conventional Commits 的远端校验规则。当前远端 ruleset 未启用该规则；仅调整 Actions 触发策略、required checks 或合并方式时，应基于远端现状构造精确更新，不直接导入完整模板扩大门禁范围。

`master-protection.json` 中的 `actor_id: 5` 按“RepositoryRole = Admin”模板生成，表示管理员只能通过 PR 绕过规则。

## 配套仓库设置

- 仓库 Merge options 中启用 `Rebase merging`
- 仓库 Merge options 中启用 `Merge commits`
- 关闭 `Squash merging`
- 如后续增加 `CODEOWNERS`，再决定是否开启 code owner review
- 如果后续形成稳定的多人评审安排，再提高 `required_approving_review_count`；单人阶段不应把管理员 bypass 当作每次合并的常规路径
