# RadishFlow

RadishFlow 是一个以 Rust 为核心、以 Rust UI 为主界面、以 `.NET 10` 负责 CAPE-OPEN / COM 适配的稳态流程模拟软件。

## 当前定位

当前第一阶段保持以下稳定边界：

- Rust 实现稳态模拟核心
- Rust 实现桌面 UI
- `.NET 10` 暴露自有 CAPE-OPEN Unit Operation PMC
- 当前不加载第三方 CAPE-OPEN 模型
- Rust 不直接处理 COM；Rust 与 `.NET` 边界只允许句柄、基础数值、UTF-8 字符串和 JSON

更具体的阶段目标、冻结范围和非目标，以 `docs/status/current.md`、`docs/mvp/scope.md` 和 `docs/capeopen/boundary.md` 为准。

## 当前状态入口

- 当前阶段、当前重点、当前验证基线和下一步建议：`docs/status/current.md`
- 文档总索引：`docs/README.md`
- 协作规则入口：`AGENTS.md`、`CLAUDE.md`

根 `README.md` 只保留稳定入口信息，不再重复维护易过期的阶段进度。

## 快速开始

默认仓库级验证入口：

```powershell
pwsh ./scripts/check-repo.ps1
```

```bash
./scripts/check-repo.sh
```

只检查文本编码与换行格式时：

```powershell
pwsh ./scripts/check-text-files.ps1
pwsh ./scripts/normalize-text-files.ps1
```

```bash
./scripts/check-text-files.sh
```

`check-repo` 会统一执行文本门禁、`cargo fmt --all --check`、`cargo check --workspace`、`cargo test --workspace` 与 `cargo clippy --workspace --all-targets -- -D warnings`。

## 文本与格式约束

仓库自有文本资产默认采用：

- `UTF-8` 无 BOM
- `LF` 换行
- 文件末尾保留换行

这些约束由 `.editorconfig`、`.gitattributes` 和 `xtask check-text-files` 共同保证。`adapters/reference/` 下的外部参考资料允许保留上游编码、BOM 和换行格式，不纳入严格文本门禁。

## 仓库结构

- `apps/radishflow-studio/`: Rust 桌面应用
- `crates/`: Rust 核心、UI、求解与 FFI crates
- `adapters/dotnet-capeopen/`: `.NET 10` CAPE-OPEN / COM 适配层
- `docs/`: 架构、MVP、边界、周志与迁移文档
- `examples/`: 示例流程与 PME 验证样例
- `tests/`: 数值回归与互操作测试
- `scripts/`: 仓库检查、绑定生成、注册和打包脚本
- `assets/`: 图标、主题与示例数据占位目录

## 关键文档

- `docs/status/current.md`: 当前阶段、当前重点、验证基线和下一步建议
- `docs/README.md`: 文档总索引
- `docs/architecture/overview.md`: 当前仓库分层与模块边界
- `docs/architecture/app-architecture.md`: 桌面 App 架构规划
- `docs/architecture/auth-entitlement-architecture.md`: 桌面登录、授权与远端物性资产架构
- `docs/mvp/scope.md`: MVP 范围、非目标与近期开发节奏
- `docs/thermo/mvp-model.md`: 热力学与闪蒸的当前契约
- `docs/capeopen/boundary.md`: Rust 与 `.NET 10` 的 CAPE-OPEN 边界
- `docs/adr/0001-branch-and-pr-governance.md`: 分支与 PR 治理策略
- `docs/architecture/versioning.md`: 项目版本命名、tag 与发布轨道约定
- `docs/devlogs/README.md`: 周志规范与索引

## 许可

当前仓库采用自定义的 source-available 许可。默认允许阅读源码用于个人参考和学习，但不默认授予复制、分发、修改、衍生开发或商业使用权利。

完整条款见 `LICENSE` 文件。

## 参考仓库

- [CapeOpenCore](https://github.com/laugh0608/CapeOpenCore): 当前用于提取 CAPE-OPEN 接口、GUID、异常语义与注册语义的参考仓库
- [Radish](https://github.com/laugh0608/Radish): 当前用于参考 OIDC / Auth / Client Registration 能力的统一平台仓库
- [DWSIM](https://github.com/DanWBR/dwsim): 当前用于参考模块拆分、求解组织和自动化入口的工程经验，不直接迁移源码
