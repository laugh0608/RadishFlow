# GitHub Rulesets

本目录存放 RadishFlow 的仓库规则模板。  
当前只维护 `master` 分支保护规则，`dev` 作为常态开发分支，不启用强制保护。

## 建议流程

1. 日常开发提交到 `dev` 或功能分支
2. 功能、文档、规范类变更默认先合并到 `dev`
3. 阶段性稳定后，再从 `dev` 发起到 `master` 的 Pull Request
4. `master` PR 必须通过仓库检查
5. 管理员如需绕过规则，也只能通过 Pull Request，不开放直接 push

## master 规则说明

- 禁止直接推送到 `master`
- 禁止 force push
- 禁止删除分支
- 仅允许通过 Pull Request 合并
- 要求 `Repo Hygiene` 与 `Rust Baseline` 检查通过
- `PR Checks` 当前拆分为 `Repo Hygiene` 与 `Rust Baseline` 两个 job，保留拆分式门禁，但不引入当前仓库并不存在的 `Frontend Lint`
- GitHub 对 Actions required status checks 当前按 job 名匹配，不看 workflow 前缀或事件后缀，因此 ruleset 中固定写 job 名
- `PR Checks` 只响应 `pull_request -> master`，避免与 tag / 手动检查共用同一个 workflow 名称后产生状态名漂移
- 限制合并方式为 `squash` / `rebase`
- 管理员仅可通过 Pull Request 方式绕过规则，不开放直接 push

## dev 策略说明

- `dev` 是当前常态开发分支
- 当前阶段不启用 branch protection
- 当前默认不要求 push 到 `dev` 时自动触发仓库检查
- 当前也不仿照 `Radish` 对 `pull_request -> dev` 强制收口；tag 与手动补跑改由独立的 `Release Checks` workflow 承担
- 如后续进入多人并行开发，再评估是否对 `dev` 追加保护

## 检查入口

- `scripts/check-repo.ps1` 与 `scripts/check-repo.sh` 当前复用同一套 Rust `xtask` 实现
- CI 当前默认在 Linux runner 上使用 `.sh` 入口，本地 Windows 仍可继续使用 `.ps1`

## 应用方式

如果仓库还没有对应 ruleset，可以使用 GitHub CLI 或 REST API 导入：

```bash
gh api repos/<owner>/<repo>/rulesets --method POST --input .github/rulesets/master-protection.json
```

如果仓库中已存在旧 ruleset，建议改用 `PUT /repos/{owner}/{repo}/rulesets/{ruleset_id}` 更新。

`master-protection.json` 中的 `actor_id: 5` 按“RepositoryRole = Admin”模板生成，表示管理员只能通过 PR 绕过规则。

## 配套仓库设置

- 仓库 Merge options 中启用 `Squash merging` 与 `Rebase merging`
- 关闭 `Merge commits`
- 如后续增加 `CODEOWNERS`，再决定是否开启 code owner review

