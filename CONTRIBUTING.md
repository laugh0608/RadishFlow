# 参与 RadishFlow

感谢你关注 RadishFlow。自 2026-06-12 起，本项目停止公开业务功能开发、产品问题处理、外部 Pull Request 合并、发布打包和对外推广。本文件用于说明当前维护边界，不构成恢复外部协作或继续开发产品的邀请。

## 当前贡献状态

- 不接受新增模拟功能、物性模型、单元模块、CAPE-OPEN / COM 能力、商业化能力、发布包或产品路线扩张。
- 不处理常规产品问题，也不保证审阅外部 Issue、Pull Request、补丁或功能建议；未经邀请的 Pull Request 可能直接关闭。
- 仓库所有者仍可按需维护文档治理、CI、ruleset、安全基线、仓库元数据和工具链兼容性；这不表示业务功能恢复维护。
- 安全漏洞不要创建公开 Issue 或 Pull Request，请按[安全策略](SECURITY.md)私下报告。
- 参与任何项目交流时均须遵守[社区行为准则](CODE_OF_CONDUCT.md)。

若未来恢复某类外部贡献，将通过仓库公告重新说明范围、流程和支持承诺。在此之前，请不要以本文件推定维护者会接受或合并贡献。

## 合规与材料边界

- 不提交任职单位或其他第三方的源代码、设计、测试数据、流程数据、文档、商业秘密或其他保密材料。
- 不根据无法确认授权的内部实现进行复制、改写式移植、逆向还原或衍生开发。
- 第三方代码、数据、模型、字体、图标和其他资产必须具有清晰来源，并遵守各自许可证；仅有公开可见性不等于获得再利用授权。
- 不提交真实密钥、凭据、个人数据、专有项目文件或未经脱敏的日志。
- 对代码或资料的来源、权利边界存在疑问时，不应提交。

## 许可证与贡献授权

本仓库采用 [RadishFlow Source-Available License 1.0](LICENSE)，不是开放源码许可证。除 `LICENSE` 明确允许的个人阅读和学习外，不默认授予复制、修改、分发、衍生开发或商业使用权利。

如果你提交 Pull Request、补丁、Issue 附件或其他贡献，即表示你有权提供相关内容，并接受 `LICENSE` 第 4 节规定的贡献授权。贡献授权不代表维护者承诺审阅、合并、发布或持续维护该内容；许可条款冲突时始终以 `LICENSE` 为准。

## 维护者工作流

经仓库所有者确认、且属于当前允许范围的维护，应先阅读：

1. [当前状态](docs/status/current.md)
2. [文档入口](docs/README.md)
3. [协作规则](AGENTS.md)
4. [分支与 PR 治理](docs/adr/0001-branch-and-pr-governance.md)
5. 与改动直接相关的专题文档

仓库所有者或已授权维护者串行推进普通维护时直接在 `dev` 开发和提交；项目所有者明确要求、外部贡献、并行写入、确有隔离价值的高风险改动或明确需要评审时，才从 `docs/*`、`chore/*`、`fix/*` 等主题分支向 `dev` 发起 Pull Request。Agent 不因默认流程自动创建 `codex/*` 分支或额外 worktree。`master` / `main` 只接收阶段性稳定化合并或明确的 `hotfix/*`；禁止直接 push 或 force push 共享分支。具体规则以 ADR 0001 为准。

稳定化 `dev -> master/main` PR 优先使用 merge commit。任何 PR 合并到 `master` / `main` 后，开始下一轮开发前必须把最新 `origin/master` / `origin/main` 回灌并推送到 `dev`；可快进时优先 fast-forward，否则使用普通 merge，禁止使用 rebase、reset、force push 或重写既有提交伪造同步。回灌只关闭分支拓扑，不代表创建 tag、发布 artifact 或执行部署。

提交信息遵循 Conventional Commits，例如：

```text
docs(governance): 完善仓库安全策略
fix(repo): 修复文本格式检查
chore(ci): 更新工具链兼容性基线
```

提交使用贡献者自己的 Git 身份，不添加 AI 协作者署名。

## 验证与变更说明

默认仓库级验证入口为：

```bash
./scripts/check-repo.sh
git diff --check
```

Windows PowerShell：

```powershell
pwsh ./scripts/check-repo.ps1
git diff --check
```

应按改动风险执行最小且充分的验证。Pull Request 只记录真实执行过的命令，并明确列出未执行、受环境限制或需要人工完成的项目。

变更说明应覆盖适用的目标、范围、非目标、维护边界、兼容性、安全与合规影响、实际验证、未验证风险和回滚方式。涉及架构、接口、项目格式、验证基线或正式口径时，必须同步更新 `docs/` 中对应的真相源。
