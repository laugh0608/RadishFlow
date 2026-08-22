# RadishFlow

> [!IMPORTANT]
> ## 项目业务功能停止公开维护公告
>
> 自 2026-06-12 起，RadishFlow 停止公开业务功能开发、产品问题处理、外部 PR 合并、发布打包和对外推广。本仓库继续作为历史代码与个人学习记录保留；现有内容不再代表持续维护的产品路线，也不构成任何交付、支持或继续开发承诺。作者仍可按需维护不扩张产品能力的仓库外围基础设施，例如文档治理、CI、ruleset、安全基线、仓库元数据和工具链兼容性。
>
> 停更原因是作者基于当前职业合规、保密义务和知识产权边界作出的审慎决定。作者近期需要遵守任职单位关于保密资料、职务开发成果、非职务开发成果及相关知识产权归属的更明确要求。本项目虽为个人业余项目，技术栈、实现路径和代码来源均与任职单位内部实现不同，但其功能方向与化工工艺流程模拟软件高度接近。继续公开迭代、接受外部贡献或推进产品化，可能造成外界对利益冲突、保密义务、职务成果归属或不当竞争边界的误解，也可能给作者本人、任职单位和潜在使用者带来不必要的合规风险。
>
> 为尊重任职单位的制度要求，并尽量避免任何潜在争议，作者决定主动停止本项目的后续业务功能维护。后续不会继续添加模拟功能、物性模型、CAPE-OPEN / COM 适配、商业化能力、发布包或路线图内容；仓库外围基础设施维护不得被解释为恢复产品开发。也不建议任何人基于本仓库开展与作者任职单位业务存在直接竞争关系的公开协作。
>
> 本公告仅说明项目维护状态和作者的风险回避立场，不用于披露、确认或评价任何第三方商业秘密、技术秘密、产品实现或权利归属；也不构成法律意见或对既有代码权属的扩张性声明。既有代码的阅读和使用边界仍以仓库根目录 `LICENSE` 文件为准，本公告不新增任何授权。

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

- [当前阶段、当前重点、当前验证基线和下一步建议](docs/status/current.md)
- [文档总索引](docs/README.md)
- 协作规则入口：[AGENTS.md](AGENTS.md)、[CLAUDE.md](CLAUDE.md)
- [维护与潜在贡献边界](CONTRIBUTING.md)
- [社区交流规范](CODE_OF_CONDUCT.md)
- [漏洞私下报告与处理边界](SECURITY.md)

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

`check-repo` 会统一执行仓库治理与文本门禁、`cargo fmt --all --check`、`cargo check --workspace`、`cargo test --workspace` 与 `cargo clippy --workspace --all-targets -- -D warnings`。治理门禁覆盖必需文件、Markdown 相对链接、JSON、协作文件同步、GitHub 配置契约和 `git diff --check`。

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

完整条款见 [LICENSE](LICENSE) 文件。

项目当前不接受外部业务功能贡献；维护状态、潜在贡献授权和合规边界见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 参考仓库

- [CapeOpenCore](https://github.com/laugh0608/CapeOpenCore): 当前用于提取 CAPE-OPEN 接口、GUID、异常语义与注册语义的参考仓库
- [Radish](https://github.com/laugh0608/Radish): 当前用于参考 OIDC / Auth / Client Registration 能力的统一平台仓库
- [DWSIM](https://github.com/DanWBR/dwsim): 当前用于参考模块拆分、求解组织和自动化入口的工程经验，不直接迁移源码
