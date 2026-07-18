# GitHub Rulesets

本目录存放 RadishFlow 的仓库规则模板。  
当前维护默认分支 `master` / `main` 的保护规则，`dev` 作为常态开发分支，不启用强制保护，也不自动触发 CI。

## 建议流程

1. 日常开发提交到 `dev` 或功能分支
2. 功能、文档、规范类变更默认先合并到 `dev`
3. 阶段性稳定后，再从 `dev` 发起到默认分支（当前为 `master`，如切换可适配 `main`）的 Pull Request
4. 默认分支 PR 必须通过仓库检查和 staging package 验证
5. 管理员如需绕过规则，也只能通过 Pull Request，不开放直接 push

## 默认分支规则说明

- 禁止直接推送到受保护的默认分支（`master` / `main`）
- 禁止 force push
- 禁止删除分支
- 仅允许通过 Pull Request 合并
- 要求 `Repo Hygiene`、Linux `Rust Baseline`、`Rust Baseline (macOS)`、`Rust Baseline (Windows)`、`.NET Adapter Baseline` 与 `Windows Staging Package` 检查通过
- `PR Checks` 当前拆分为 `Repo Hygiene`、三平台 Rust baseline、Windows `.NET` 适配层 baseline 和 Windows staging package job，保留拆分式门禁，但不引入当前仓库并不存在的 `Frontend Lint`
- GitHub 对 Actions required status checks 当前按 job 名匹配，不看 workflow 前缀或事件后缀，因此 ruleset 中固定写 job 名
- `PR Checks` 响应 `pull_request -> dev/master/main`；目标为 `dev` 的 PR 为其他开发者提供合并前反馈，目标为默认分支的 PR 承担阶段稳定化门禁，普通 `dev` push 不触发
- `Release Checks` 当前只保留 `workflow_dispatch` 手动 staging 入口；tag push 不自动触发 CI/CD，避免普通内部 staging 或历史 tag 造成误发布信号
- 允许 `merge` 与 `rebase` 两种合并方式，禁用 `squash`
- 管理员仅可通过 Pull Request 方式绕过规则，不开放直接 push

## dev 策略说明

- `dev` 是当前常态开发分支
- 当前阶段不启用 branch protection
- 当前默认不要求 push 到 `dev` 时自动触发仓库检查
- 目标为 `dev` 的 Pull Request 自动运行 `PR Checks`，但 `dev` 当前不启用 required checks 或 branch protection；直接进入共享 `dev` 的连续开发仍按改动风险执行本地验证
- 如后续进入多人并行开发，再评估是否对 `dev` 追加保护

## 检查入口

- `scripts/check-repo.ps1` 与 `scripts/check-repo.sh` 当前复用同一套 Rust `xtask` 实现
- `scripts/check-dotnet-capeopen.ps1` 当前作为 Windows `.NET 10` CAPE-OPEN baseline 入口，负责 `rf-ffi` native build、`.NET` solution build、contract tests 和 smoke tests；不执行 COM 注册、反注册或注册表写入
- `scripts/package.ps1` 当前只产出 Windows portable staging package；CI 上传 workflow artifact，不创建 GitHub Release，也不发布安装包
- CI 当前在 Linux / macOS runner 上使用 `.sh` 入口，在 Windows runner 上使用 `.ps1`

## 应用方式

如果仓库还没有对应 ruleset，可以使用 GitHub CLI 或 REST API 导入：

```bash
gh api repos/<owner>/<repo>/rulesets --method POST --input .github/rulesets/master-protection.json
```

如果仓库中已存在旧 ruleset，建议改用 `PUT /repos/{owner}/{repo}/rulesets/{ruleset_id}` 更新。

`master-protection.json` 中的 `actor_id: 5` 按“RepositoryRole = Admin”模板生成，表示管理员只能通过 PR 绕过规则。

## 配套仓库设置

- 仓库 Merge options 中启用 `Rebase merging`
- 仓库 Merge options 中启用 `Merge commits`
- 关闭 `Squash merging`
- 如后续增加 `CODEOWNERS`，再决定是否开启 code owner review
