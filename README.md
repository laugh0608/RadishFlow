# RadishFlow

> RadishFlow 自 2026-09-12 起恢复正常开发和迭代。当前阶段与优先级见 [当前状态](docs/status/current.md)，具体范围与验收要求见对应开发专题。

RadishFlow 是一个以 Rust 为核心、以 Rust UI 为主界面、以 `.NET 10` 负责 CAPE-OPEN / COM 适配的可扩展流程模拟平台。当前从稳态流程起步，长期覆盖动态、瞬态及多种求解组织方式。

## 当前定位

项目以已完成的第一阶段 MVP 为基础继续迭代，保持以下架构边界：

- Rust 实现计算核心，当前为受控稳态流程，长期按领域能力扩展
- Rust 实现桌面 UI
- `.NET 10` 暴露自有 CAPE-OPEN Unit Operation PMC
- 当前不加载第三方 CAPE-OPEN 模型
- Rust 不直接处理 COM；Rust 与 `.NET` 边界只允许句柄、基础数值、UTF-8 字符串和 JSON

更具体的开发状态、已有能力与限制，见 [当前状态](docs/status/current.md)、[MVP 范围](docs/mvp/scope.md) 和 [CAPE-OPEN 边界](docs/capeopen/boundary.md)。

近期先完善基础功能与使用闭环；长期建设可独立组合的组分、物性与分析、画布、单元与反应、模拟、算法、报告及对外 API 系统，包含递归分块求解、间歇操作、变量浏览树、COM 自动化和脚本录制 / 回放。领域边界见 [模拟平台长期规划](docs/architecture/simulation-platform.md)，阶段与验收见 [开发路线图](docs/radishflow-mvp-roadmap.md)。目标能力按阶段实现，不代表现有产品已具备。

现有模型使用简化物性与单元假设，内置样例用于演示和软件回归；求解收敛、α / β 验收和跨层一致性不构成工程工况准确性证明。模型假设、样例来源限制及独立验证缺口统一见 [热力学 MVP 模型](docs/thermo/mvp-model.md)。

## 当前状态入口

- [当前阶段、当前重点、当前验证基线和下一步建议](docs/status/current.md)
- [文档总索引](docs/README.md)
- 协作规则入口：[AGENTS.md](AGENTS.md)、[CLAUDE.md](CLAUDE.md)
- [开发与贡献指南](CONTRIBUTING.md)
- [社区交流规范](CODE_OF_CONDUCT.md)
- [漏洞私下报告与处理边界](SECURITY.md)

根 `README.md` 只保留稳定入口信息，不再重复维护易过期的阶段进度。

## 快速开始

仓库通过 `rust-toolchain.toml` 固定 Rust 工具链，通过 `Cargo.lock` 与 `--locked` 保持正式检查的依赖组合；支持版本见 [当前状态](docs/status/current.md)，升级与兼容性检查方式见 [工具链维护规则](docs/development/agent-collaboration.md#rust-工具链与锁定依赖)。

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

`check-repo` 会统一执行仓库治理与文本门禁、`cargo fmt --all --check`、`cargo check --locked --workspace`、`cargo test --locked --workspace` 与 `cargo clippy --locked --workspace --all-targets -- -D warnings`。治理门禁覆盖必需文件、Markdown 相对链接、JSON、协作文件同步、GitHub 配置契约和 `git diff --check`。

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
- `docs/mvp/scope.md`: 已保留的 MVP 范围、模型限制与验收含义
- `docs/thermo/mvp-model.md`: 热力学与闪蒸的当前契约
- `docs/capeopen/boundary.md`: Rust 与 `.NET 10` 的 CAPE-OPEN 边界
- `docs/adr/0001-branch-and-pr-governance.md`: 分支与 PR 治理策略
- `docs/architecture/versioning.md`: 项目版本命名、tag 与发布轨道约定
- `docs/devlogs/README.md`: 周志规范与索引

## 许可

当前仓库采用自定义的 source-available 许可。默认允许阅读源码用于个人参考和学习，但不默认授予复制、分发、修改、衍生开发或商业使用权利。

完整条款见 [LICENSE](LICENSE) 文件。

产品问题、功能建议与代码贡献按项目流程评审；贡献授权和材料边界见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 参考仓库

- [CapeOpenCore](https://github.com/laugh0608/CapeOpenCore): 当前用于提取 CAPE-OPEN 接口、GUID、异常语义与注册语义的参考仓库
- [Radish](https://github.com/laugh0608/Radish): 当前用于参考 OIDC / Auth / Client Registration 能力的统一平台仓库
- [DWSIM](https://github.com/DanWBR/dwsim): 当前用于参考模块拆分、求解组织和自动化入口的工程经验，不直接迁移源码
