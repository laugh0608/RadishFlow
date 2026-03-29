# RadishFlow 启动清单

更新时间：2026-03-29

## 文档目的

本文档用于回答两个实际问题：

1. `RadishFlow` 新仓库应该如何初始化
2. 从当前 `CapeOpenCore` 启动 `RadishFlow` 时，哪些内容应该迁移，哪些内容不应该迁移

本文档默认前提：

- 新产品仓库名采用 `RadishFlow`
- 使用独立新仓库开发，不继续在当前仓库主线中生长
- 当前 `CapeOpenCore` 仓库保留为参考仓库与过渡资产来源

配套文档：

- `radishflow-architecture-draft.md`
- `radishflow-capeopen-asset-checklist.md`
- `radishflow-mvp-roadmap.md`

## 仓库命名建议

推荐仓库名：`RadishFlow`

说明：

- 采用大小写结合，符合你的品牌偏好
- 与软件名保持一致，识别度更强
- 在 GitHub 等平台上通常可保留显示大小写

建议约定：

- GitHub 仓库显示名：`RadishFlow`
- 本地克隆目录名：`RadishFlow`
- Rust crate 仍保持全小写加短前缀，例如 `rf-thermo`
- .NET 项目保持 PascalCase 命名，例如 `RadishFlow.CapeOpen.Adapter`

## 初始仓库骨架清单

下面这份清单是建议你在新仓库创建后的第一批目录与文件。

```text
RadishFlow/
├─ .gitignore
├─ README.md
├─ LICENSE
├─ Cargo.toml
├─ rust-toolchain.toml
├─ docs/
│  ├─ README.md
│  ├─ architecture/
│  ├─ mvp/
│  ├─ capeopen/
│  ├─ thermo/
│  └─ adr/
├─ apps/
│  └─ radishflow-studio/
│     ├─ Cargo.toml
│     └─ src/
├─ crates/
│  ├─ rf-types/
│  ├─ rf-model/
│  ├─ rf-thermo/
│  ├─ rf-flash/
│  ├─ rf-unitops/
│  ├─ rf-flowsheet/
│  ├─ rf-solver/
│  ├─ rf-store/
│  ├─ rf-ui/
│  ├─ rf-canvas/
│  └─ rf-ffi/
├─ adapters/
│  └─ dotnet-capeopen/
│     ├─ RadishFlow.CapeOpen.sln
│     ├─ Directory.Build.props
│     └─ src/
│        ├─ RadishFlow.CapeOpen.Interop/
│        ├─ RadishFlow.CapeOpen.Adapter/
│        ├─ RadishFlow.CapeOpen.UnitOp.Mvp/
│        ├─ RadishFlow.CapeOpen.Registration/
│        └─ RadishFlow.CapeOpen.SmokeTests/
├─ bindings/
│  └─ c/
├─ examples/
│  ├─ flowsheets/
│  ├─ sample-components/
│  └─ pme-validation/
├─ tests/
│  ├─ rust-integration/
│  ├─ thermo-golden/
│  ├─ flash-golden/
│  └─ capeopen-smoke/
├─ scripts/
│  ├─ gen-bindings.ps1
│  ├─ register-com.ps1
│  ├─ package.ps1
│  └─ smoke-test.ps1
└─ assets/
   ├─ icons/
   ├─ themes/
   └─ sample-data/
```

## 第一批必须创建的文件

建议新仓库初始化时就创建以下文件：

- 根 `README.md`
- 根 `.gitignore`
- 根 `Cargo.toml`
- `rust-toolchain.toml`
- `docs/README.md`
- `apps/radishflow-studio/Cargo.toml`
- `crates/rf-types/Cargo.toml`
- `crates/rf-model/Cargo.toml`
- `crates/rf-thermo/Cargo.toml`
- `crates/rf-flash/Cargo.toml`
- `crates/rf-unitops/Cargo.toml`
- `crates/rf-flowsheet/Cargo.toml`
- `crates/rf-solver/Cargo.toml`
- `crates/rf-ffi/Cargo.toml`
- `adapters/dotnet-capeopen/RadishFlow.CapeOpen.sln`

## 第一批可以先空着的目录

这些目录建议先建出来，但内容可以晚一点再补：

- `crates/rf-ui`
- `crates/rf-canvas`
- `crates/rf-store`
- `bindings/c`
- `examples/pme-validation`
- `tests/capeopen-smoke`
- `assets/themes`

这样做的好处是：

- 提前冻结整体形态
- 避免后期目录频繁搬迁
- 让文档、脚本和 CI 的路径尽早稳定

## 第一天要完成的最小初始化

如果你要把启动动作压缩到最小，我建议第一天只做这些：

1. 建立 `RadishFlow` 仓库
2. 建立 Rust workspace
3. 建立 `apps/radishflow-studio`
4. 建立 `rf-types`、`rf-model`、`rf-thermo`、`rf-flash`、`rf-unitops`、`rf-flowsheet`、`rf-solver`、`rf-ffi`
5. 建立 `adapters/dotnet-capeopen`
6. 写根 `README.md`
7. 写 `docs/README.md`

## 开新仓库前的迁移前准备

在正式创建 `RadishFlow` 仓库前，建议先在当前仓库完成以下准备：

- 冻结 `RadishFlow` 名称与命名规则
- 冻结 MVP 范围
- 冻结目标仓库骨架
- 冻结第一阶段只导出自有 Unit Operation PMC 的原则
- 整理当前仓库中可复用的 CAPE-OPEN 接口与注册语义清单
- 把关键文档先写完整，避免新仓库创建后还回头补方向性讨论

这些准备完成后，再开新仓库会明显顺很多。

## 从 CapeOpenCore 迁移时该带走什么

新仓库不应该“整仓复制”当前项目，而应按主题提炼资产。

建议迁移的内容：

- CAPE-OPEN 接口定义思路
- `Guid`、`ComVisible`、`ClassInterface`、`DispId` 等语义约定
- CAPE-OPEN 类别注册逻辑经验
- ECape 异常语义
- `MixerExample` 这类最小单元行为参考
- 参数、端口、单元抽象的设计经验
- 当前 `docs/` 中形成的迁移与架构文档

建议只作为参考、不直接复制的内容：

- 旧式 `.NET Framework 4.8` 项目文件
- WinForms UI 代码
- `RegistrationServices` 注册器实现
- 基于 `AppDomain` 的旧式加载逻辑
- 当前类库中与旧 COM 注册表结构强耦合的代码
- `CapeOpen.BackUp` 整包历史备份

## 从 CapeOpenCore 迁移时不该做什么

不建议做以下动作：

- 直接复制整个 `CapeOpenCore` 目录作为新仓库起点
- 在当前仓库根目录直接加入 Rust workspace 然后混合开发
- 让 Rust 直接接触 COM
- 第一阶段就尝试支持“加载别人的 CO 模型”
- 第一阶段就同时完成完整 Thermo PMC 和完整 PME 主机兼容

## 建议的迁移拆包方式

建议按“文档迁移、语义迁移、代码迁移”三类拆开：

### 文档迁移

先迁这些：

- `RadishFlow` 架构草案
- `RadishFlow` MVP 路线图
- MVP 范围说明
- 新仓库开发路线图
- CAPE-OPEN 适配边界说明

### 语义迁移

再迁这些：

- GUID 与接口语义约定
- CAPE-OPEN 类别与注册语义
- ECape 异常语义

详细提取边界见：

- `radishflow-capeopen-asset-checklist.md`

### 代码迁移

最后才迁这些：

- .NET 10 适配层里真正需要的接口声明
- 最小 Unit Operation PMC 外壳
- 注册工具的现代化实现

## 新仓库的第一批开发优先级

建议按以下优先级启动：

### 优先级 A

- `rf-types`
- `rf-model`
- `rf-thermo`
- `rf-flash`

目标：

- 先打通一个可验证的二元 `TP Flash`

### 优先级 B

- `rf-unitops`
- `rf-flowsheet`
- `rf-solver`

目标：

- 打通最小稳态流程闭环

### 优先级 C

- `rf-ffi`
- `RadishFlow.CapeOpen.Adapter`
- `RadishFlow.CapeOpen.UnitOp.Mvp`

目标：

- 让外部 PME 能识别并调用你的 Unit Operation PMC

### 优先级 D

- `rf-ui`
- `rf-canvas`
- `apps/radishflow-studio`

目标：

- 建立你自己的 Rust 桌面工作台

## 建议的首批里程碑

建议把新仓库第一阶段拆成以下 5 个里程碑：

1. `M1`：仓库与 workspace 初始化
2. `M2`：二元体系 `TP Flash` 跑通
3. `M3`：`Feed + Mixer + Flash Drum` 流程闭环跑通
4. `M4`：Rust FFI 与 .NET 10 适配层联通
5. `M5`：第一个 CAPE-OPEN Unit Operation PMC 被外部 PME 识别

详细拆分建议见：

- `radishflow-mvp-roadmap.md`

## 建议的首批文档

新仓库建立后，建议最先写这 5 份文档：

- `docs/README.md`
- `docs/architecture/overview.md`
- `docs/mvp/scope.md`
- `docs/capeopen/boundary.md`
- `docs/thermo/mvp-model.md`

## Definition of Ready

当以下条件满足时，可以视为 `RadishFlow` 仓库已经准备好正式开工：

- 新仓库已创建
- Rust workspace 已建好
- .NET 10 适配层目录已建好
- 初始文档已就位
- MVP 范围已冻结
- 已明确“第一阶段只导出自有 Unit Operation PMC”

## 结论

最稳妥的启动方式不是继续在 `CapeOpenCore` 上扩展，而是：

- 新开 `RadishFlow`
- 只迁移必要的语义与文档资产
- 先搭骨架
- 再做二元 `TP Flash`
- 最后把 Rust 核心通过 .NET 10 适配成 CAPE-OPEN Unit Operation PMC

这条启动路径最清晰，也最能避免当前仓库的历史包袱进入新产品主线。
