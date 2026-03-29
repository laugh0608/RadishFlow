# ADR 0001: Branch And PR Governance

更新时间：2026-03-29

## 状态

Accepted

## 背景

当前仓库已经从“空骨架”进入“逐步可开发”的阶段，但分支治理、PR 流程和自动化检查仍未建立。  
如果继续在 `master` 上直接累积提交，后续很难形成稳定的发布基线，也不利于多人或多代理协作。

## 决策

仓库采用以下分支与 PR 治理策略：

### 分支角色

- `master`: 稳定主线，只接受 Pull Request 合并
- `dev`: 日常集成分支，功能、文档、规范类分支默认合并到这里
- `feature/*`: 功能开发分支
- `docs/*`: 文档与规范分支
- `chore/*`: 基础设施、脚本、CI、仓库治理分支

### 合并策略

- 默认开发流程为 `feature/*` -> `dev`
- 阶段性稳定后，再通过 PR 将 `dev` 合并到 `master`
- 仅在必须修复主线问题时，才允许 `hotfix/*` 直接向 `master` 发 PR

### `master` 规则

- 禁止直接 push
- 必须通过 PR 合并
- 必须通过仓库检查
- 管理员仅可通过 PR 方式绕过规则
- 允许在单人开发阶段保留管理员 PR 直过能力

### `dev` 规则

- 允许作为当前阶段默认目标分支
- 当前阶段不启用分支保护
- 仍建议保留 CI 检查和 PR 习惯，但不作为强制规则

## 需要在 GitHub 仓库设置中完成的动作

以下规则不能仅靠仓库文件完全强制，需要仓库管理员在 GitHub Settings 中启用：

1. 创建远端 `dev` 分支
2. 将默认分支切换为 `dev`，或至少把开发 PR 默认目标改为 `dev`
3. 对 `master` 启用 branch protection
4. 要求 `master` 通过 `PR Checks / validate` 状态检查
5. 对 `master` 开启 “Require a pull request before merging”
6. 配置管理员仅通过 PR 绕过，不开放直接 push
7. `dev` 当前不配置 branch protection

## 仓库内已落地的支撑项

为配合该决策，仓库内已同步增加：

- PR 模板
- GitHub Actions PR 检查工作流
- 文本编码与文件格式检查脚本
- Rust workspace 基础校验入口
- `master` ruleset 模板

## 影响

正面影响：

- `master` 可以保持稳定
- `dev` 可以作为当前阶段的真实集成面
- 文档、规范、脚本和代码都能纳入统一 PR 检查
- 单人开发阶段仍保留必要的管理员 PR 绕过能力

代价：

- 需要维护远端 `master` 保护设置
- 开发节奏从“直接提交”切换为“分支 + PR”
