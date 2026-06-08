# ADR 0001: Branch And PR Governance

更新时间：2026-06-08

## 状态

Accepted

## 背景

当前仓库已经从“空骨架”进入“逐步可开发”的阶段，但分支治理、PR 流程和自动化检查仍未建立。  
如果继续在 `master` 上直接累积提交，后续很难形成稳定的发布基线，也不利于多人或多代理协作。

## 决策

仓库采用以下分支与 PR 治理策略：

### 分支角色

- `master` / `main`: 稳定主线，只接受 Pull Request 合并；当前仓库仍以 `master` 为主，若后续切换默认分支则同一规则迁移到 `main`
- `dev`: 日常集成分支，功能、文档、规范类分支默认合并到这里
- `feature/*`: 功能开发分支
- `docs/*`: 文档与规范分支
- `chore/*`: 基础设施、脚本、CI、仓库治理分支

### 合并策略

- 默认开发流程为 `feature/*` -> `dev`
- 阶段性稳定后，再通过 PR 将 `dev` 合并到 `master` / `main`
- 仅在必须修复主线问题时，才允许 `hotfix/*` 直接向 `master` / `main` 发 PR

### `master` / `main` 规则

- 禁止直接 push
- 必须通过 PR 合并
- 必须通过仓库检查和 staging package 验证
- 当前允许 `merge commit` 与 `rebase merge`，禁用 `squash merge`
- 管理员仅可通过 PR 方式绕过规则
- 允许在单人开发阶段保留管理员 PR 直过能力

### `dev` 规则

- 允许作为当前阶段默认目标分支
- 当前阶段不启用分支保护
- 作为日常开发分支，不自动触发 CI/CD
- 本地提交前按改动风险自行执行 focused test 或仓库级验证；CI/CD 统一放到 `dev -> master/main` 稳定化 PR

## 需要在 GitHub 仓库设置中完成的动作

以下规则不能仅靠仓库文件完全强制，需要仓库管理员在 GitHub Settings 中启用：

1. 创建远端 `dev` 分支
2. 将默认分支切换为 `dev`，或至少把开发 PR 默认目标改为 `dev`
3. 对 `master` / `main` 启用 branch protection
4. 要求 `master` / `main` 通过 `Repo Hygiene`、三平台 `Rust Baseline`、`.NET Adapter Baseline` 与 `Windows Staging Package` 状态检查
5. 对 `master` / `main` 开启 “Require a pull request before merging”
6. 仓库 Merge options 中启用 `Merge commits` 与 `Rebase merging`，关闭 `Squash merging`
7. 配置管理员仅通过 PR 绕过，不开放直接 push
8. `dev` 当前不配置 branch protection

## 仓库内已落地的支撑项

为配合该决策，仓库内已同步增加：

- PR 模板
- GitHub Actions PR 检查工作流
  - `PR Checks` 当前默认只在目标分支为 `master` / `main` 的 Pull Request 上自动触发，用于 `dev -> master/main` 稳定化合并
  - 当前拆分为 `Repo Hygiene`、三平台 `Rust Baseline`、`.NET Adapter Baseline` 与 `Windows Staging Package`，保留拆分式门禁，但不引入当前仓库并不存在的 `Frontend Lint`
  - `master` / `main` required checks 当前按 job 名 `Repo Hygiene` / `Rust Baseline` / `Rust Baseline (macOS)` / `Rust Baseline (Windows)` / `.NET Adapter Baseline` / `Windows Staging Package` 配置，不使用 workflow 前缀
  - 规范 tag push 暂不自动触发 CI/CD；`Release Checks` 仅保留 `workflow_dispatch` 手动 staging 入口，避免普通内部 staging 或历史 tag 造成误发布信号
- 文本编码与文件格式检查脚本
  - 正式实现源收口到 Rust `xtask`，`.ps1` 与 `.sh` 仅作为平台包装层
- Rust workspace 基础校验入口
- Windows `.NET 10` CAPE-OPEN baseline 入口：`scripts/check-dotnet-capeopen.ps1` 负责 native build、`.NET` solution build、contract tests 和 smoke tests；不执行 COM 注册、反注册或注册表写入
- Windows portable staging package 入口：`scripts/package.ps1` 只产出 workflow artifact，不创建 GitHub Release，也不发布安装包
- `master` / `main` ruleset 模板

## 影响

正面影响：

- `master` / `main` 可以保持稳定
- `dev` 可以作为当前阶段的真实日常开发面，不被日常 CI 噪声阻塞
- 文档、规范、脚本和代码都能纳入统一 PR 检查
- `.NET` CAPE-OPEN 适配层和 Windows staging package 不再游离于默认分支合并门禁之外
- 单人开发阶段仍保留必要的管理员 PR 绕过能力

代价：

- 需要维护远端 `master` / `main` 保护设置
- 开发节奏从“直接提交”切换为“分支 + PR”
- 默认分支 PR 检查时间会增加，尤其是 Windows `.NET` baseline 和 staging package job
