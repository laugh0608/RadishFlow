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
- 要求 `PR Checks / validate` 检查通过
- 限制合并方式为 `squash` / `rebase`
- 管理员仅可通过 Pull Request 方式绕过规则，不开放直接 push

## dev 策略说明

- `dev` 是当前常态开发分支
- 当前阶段不启用 branch protection
- 当前默认不要求 push 到 `dev` 时自动触发仓库检查
- 仓库检查默认收口在指向 `master` 的 Pull Request 上；如需额外检查，可手动触发 workflow
- 如后续进入多人并行开发，再评估是否对 `dev` 追加保护

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
