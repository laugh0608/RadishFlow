# Versioning And Release

更新时间：2026-03-29

## 目标

本文档用于冻结 RadishFlow 当前阶段的项目版本命名、Git tag 规则与最小发布轨道口径。

这里的目标不是立刻把所有 crate 内部版本都切到发布口径，而是先明确：

- RadishFlow 对外版本如何命名
- 哪些 tag 视为规范发布 tag
- CI 当前对哪些 tag 自动响应
- 当前阶段如何处理“项目发布版本”和“workspace 内部 crate 版本”的关系

## 参考来源

当前版本命名规则参考 [Radish](https://github.com/laugh0608/Radish) 中已经落地的日历版本号规范，并保留对 RadishFlow 当前阶段更保守的收口：

- 继续沿用 **Calendar Versioning**
- 继续沿用 `-dev` / `-test` / `-release` 轨道后缀
- 继续把规范 tag 作为自动化发布和验收的主要入口

但当前也明确保留一条差异：

- RadishFlow 仍处于地基建设阶段，Rust workspace 内部 crate 的 `Cargo.toml version` 当前先允许维持统一占位版本，不要求每次规范 tag 都同步 bump

## 当前正式规则

截至 2026-03-29，RadishFlow 当前正式采用以下版本规则。

### 1. 基础版本号

项目对外版本号采用：

```text
vYY.M.RELEASE
```

字段含义：

- `YY`
  - 两位年份，例如 `26` 代表 `2026`
- `M`
  - 月份数字，使用 `1-12`，不补零
- `RELEASE`
  - 当月发版序号，从 `1` 开始递增，每月重置

示例：

- `v26.3.1`
  - 2026 年 3 月第 1 版
- `v26.3.2`
  - 2026 年 3 月第 2 版
- `v26.4.1`
  - 2026 年 4 月第 1 版

### 2. 轨道后缀

当前规范发布 tag 必须追加环境/轨道后缀：

| 后缀 | 含义 | 示例 |
| --- | --- | --- |
| `-dev` | 开发轨道 / 内部集成验收 | `v26.3.1-dev` |
| `-test` | 测试轨道 / 测试部署验收 | `v26.3.1-test` |
| `-release` | 正式发布轨道 | `v26.3.1-release` |

当前补充口径：

- 不带轨道后缀的 `vYY.M.RELEASE` 版本号可以作为发布计划或文档描述使用
- 真正进入 CI 自动触发和对外交付时，优先使用带后缀的规范 tag

### 3. 热更新/阶段性构建

如需区分同一基线版本上的热更新或阶段性构建，允许使用扩展格式：

```text
vYY.M.RELEASE.DDXX
```

字段含义：

- `DD`
  - 两位日期，`01-31`
- `XX`
  - 当日构建序号，`01-99`

示例：

- `v26.3.1.2901`
  - `v26.3.1` 在 29 日的第 1 次阶段性构建
- `v26.3.1.2901-test`
  - `v26.3.1` 在 29 日第 1 次测试轨道构建
- `v26.3.1.2902-release`
  - `v26.3.1` 在 29 日第 2 次正式发布轨道构建

## 当前 CI 响应规则

当前仓库自动化口径冻结为：

- `pull_request -> master`
  - 由 `PR Checks` workflow 执行 `Repo Hygiene` 与 `Rust Baseline`
- `push tag -> v*-dev`
  - 由 `Release Checks` workflow 执行 `Repo Hygiene` 与 `Rust Baseline`
- `push tag -> v*-test`
  - 由 `Release Checks` workflow 执行 `Repo Hygiene` 与 `Rust Baseline`
- `push tag -> v*-release`
  - 由 `Release Checks` workflow 执行 `Repo Hygiene` 与 `Rust Baseline`
- `workflow_dispatch`
  - 允许手动触发 `Release Checks` 补跑同一组检查

当前补充口径：

- `master` ruleset 要求的状态检查固定为 `Repo Hygiene` 与 `Rust Baseline`
- GitHub 对 Actions required status checks 当前按 job 名匹配，不看 workflow 前缀或事件后缀
- 当前向 Radish 对齐为拆分式门禁，但不引入当前仓库暂不存在的 Frontend Lint 或 pull_request -> dev 默认门禁
- 仓库检查正式由 Rust `xtask` 实现，`.ps1` 与 `.sh` 只保留为调用包装层
- 不再让 PR 检查与 tag / 手动检查共用同一个 workflow 名称，避免 required check 名称与实际上报名漂移

当前明确不做：

- 不对普通 `push -> dev` 自动执行该工作流
- 不对未带轨道后缀的普通 `v*` tag 自动触发

## 当前阶段的版本边界

当前需要明确区分两类版本概念：

### 项目发布版本

这是面向仓库发布、Release 记录、部署验收和 CI tag 的版本。

当前正式采用本文档中的 CalVer + 轨道后缀规则。

### workspace 内部 crate 版本

这是 Rust workspace 内部 `Cargo.toml` 的 crate 元数据版本。

当前阶段先冻结为：

- 可以继续保持统一占位版本
- 不要求每次规范 tag 都同步 bump
- 在真正进入对外发布、打包或 crates 级分发前，再统一决定是否把 crate 元数据版本切到与项目发布版本对齐

这样做的原因：

- 当前阶段重点仍是仓库地基建设和边界冻结
- 过早把每个 crate 都拉进发布版本同步，会制造额外维护噪声
- 当前更重要的是先把“对外怎么标记版本”和“自动化对哪些 tag 响应”固定下来

## 推荐使用方式

### 日常开发

- 继续在 `dev` 或功能分支推进
- 不为普通开发提交创建发布 tag

### 需要一轮内部验收

- 创建 `vYY.M.RELEASE-dev`
- 例如：`v26.3.1-dev`

### 需要一轮测试部署/测试验收

- 创建 `vYY.M.RELEASE-test`
- 例如：`v26.3.1-test`

### 需要正式发布

- 创建 `vYY.M.RELEASE-release`
- 例如：`v26.3.1-release`

## 当前后续事项

以下内容后续仍需继续细化，但不再属于“方向未定”：

1. 是否在首个对外版本前统一把 workspace crate version 从 `0.1.0` 切到发布口径
2. 是否需要补一份 RadishFlow 的发布操作清单，覆盖 version 更新、tag、Release Notes 和验收记录
3. tag push 后除仓库检查外，后续是否需要增加打包、工件归档或安装包产出流程
