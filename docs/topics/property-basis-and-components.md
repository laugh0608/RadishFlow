# 项目物性基础与组分选择

更新时间：2026-10-24

> 本文定义该专题的能力、开发范围与验收要求；具体迭代切片和优先级以 [当前状态](../status/current.md) 为准。

## 用途

用途：定义项目级物性 package、项目组分选择、保存 / 重开和运行请求之间的事实源与开发边界。
读者：负责物性页、项目文件、运行 package resolution、Home / 项目输入展示和相关测试的开发者、用户、AI / Agent。
不包含：完整组分数据库、第三方 Property Package 加载、完整 Thermodynamics PMC、物性参数拟合和高级物性分析实现。

## 专题目标

- 项目物性状态只有一个事实源：项目文档中的稳定 package id、项目组分和当前可选内置 package / component choices。
- UI 展示可读 label，项目文件、command id 和运行请求保留稳定 id。
- 空白项目必须显式选择物性 package 和项目组分后再进入建模主路径；official 示例必须持久化自己的 package 选择。

## 子专题关系

- 当前内置 package 与项目组分选择留在本专题。
- 远端授权、派生资产清单、下载票据和离线租约属于 `platform/control-plane-service.md`。
- 后端管理台页面属于 `platform/control-plane-web-ui.md`。
- 若未来推进完整组分数据库、物性分析或第三方 package 加载，应新增独立二级专题，不继续扩写本文。

## 规划衔接（尚未实现）

2026-09-16 的 [设备工程](equipment-design-and-rating.md) 规划按任务要求补可靠密度、焓及输运物性，不把现有演示数据用于工程尺寸承诺。[单位系统](units-and-quantity-system.md) 的质量 / 摩尔与体积基准转换需要本领域提供显式数据和状态，物性参考态不能当普通单位系数。物性页在 [UI 重设计](../architecture/studio-ui-topic-plan.md) 中保留独立任务入口。

[工程基础与数据来源](engineering-basis-and-cases.md) 统一记录实测、供应商、文献、估算和用户假设；物性领域提供数据 / 方法版本、量与基准、适用域和已知不确定度。未知来源 / 不确定度不填零，来源类别不自动决定可信等级；派生值保留依赖。物性选择提示可说明数据缺口和适用范围，但不因为某方法可运行就证明其适用当前体系。基准证据按 [模型验证](../thermo/mvp-model.md#工程基准与数据可信度规划) 管理。

## 当前实现快照

已完成：

- 普通空白项目初始不预选 package 或项目组分。
- 独立 `物性` 页消费 `StudioGuiWindowPropertyPageModel`。
- `binary-hydrocarbon-lite-v1` 和 methane / ethane 已作为当前受控内置路径。
- Home recent / current / example tile、左侧 `项目输入` 和独立 `物性` 页展示可读 package label。
- official hydrocarbon 示例项目文件已持久化 `binary-hydrocarbon-lite-v1`。

已知缺口：

- 完整物性页长期空间尚未拆成可实现专题。
- 物性分析、参数来源、完整组分查询仍只是未来空间。
- 后续若引入多个内置 package，默认选择策略和冲突提示需要单独补设计。

内置 `binary-hydrocarbon-lite-v1` 与 methane / ethane 名称用于演示和回归；现有参数尚无完整独立物性准确性证据。`official` 是仓库标准案例称谓，不能作为数据质量等级。样例身份、Antoine 适用域与验证限制统一见 [热力学模型](../thermo/mvp-model.md#样例身份与适用范围)。

## 物性配置与运行上下文规划

2026-09-29 确认按以下职责演进；除当前实现栏外均为目标，具体类型与存储随 [MC0—MC3](modeling/model-catalog-and-runtime.md#分阶段切片) 设计。

| 职责 | 当前实现 | 目标边界 |
| --- | --- | --- |
| 包发现与解析 | `rf-property-package::PropertyPackageProvider::list_packages / load_package`，受检的内存 / 缓存 / 文件数据装载 | 枚举描述、校验版本与来源、解析数据由加载职责拥有；计算层不处理授权缓存 IO |
| 工程物性配置 | 流程级 package ID 与项目组分 | 明确选用模型 / 数据版本、组分、允许相态、参数及参考态；变更经命令使旧结果失效 |
| 运行热力学服务 | Studio / FFI 共用 `prepare_local_context`；每次运行冻结一套现有物性 / TP Flash 服务 | 输入快照解析到明确实现与数据；不可变参数可共享，可变上下文按任务 / 实例拥有 |
| 多物性绑定 | 尚未实现 | 有真实工段 / 单元需求时再定义绑定优先级、映射和跨包衡算，不默认为每条流股任意选包 |

现有 manifest 与数据 payload 不等于可执行物性插件，也不等于 CAPE-OPEN Property Package Manager。当前换包不代表能任意替换物性算法；完整第三方加载仍由后续宿主 / 插件切片定义。

单元只消费所需物性和闪蒸能力，独立物性分析也可消费同一服务，不必依赖流程画布。物性实现需声明支持的组分 / 相态、规格、性质和适用域；缺失能力明确拒绝，不能隐式换模型或远端求值。

多包扩展前须明确组分身份和顺序、相标签、摩尔 / 质量基准与焓参考态的一致性，计算跨包连接的物料 / 能量残差。模型切换、参数修改和数据版本变化均可影响复现；不能只记录展示名，也不能把焓参考态差异当显示单位转换。

MC2 首先验证单包装配、缓存职责迁出、跨消费者一致性与任务隔离；增加多包时再验证上述映射、缺包恢复和跨包衡算。数值精度仍依 [热力学模型](../thermo/mvp-model.md) 的独立证据，不由目录加载成功推导。

## AF2b 物性装配与运行上下文

2026-10-24 所有者确认 MC2 的 AF2b 接口迁移与三项行为变化，已按下述边界实现。AF2a 已交付内建单元目录。范围为现有本地 / 内存数据包、单包装配、发现描述和 GUI / headless / FFI 消费者；AF1 保留跟踪。新 EOS、网络求值、完整插件、多包、后台任务和 C2-B / C3 不纳入。

### 迁移前问题与本轮目标

| 链路 | 迁移前事实（`de4d7980`） | 本轮已实现目标 |
| --- | --- | --- |
| 缓存 → 领域数据 | [`rf-thermo`](../../crates/rf-thermo/src/lib.rs) 直接依赖 `rf-store`；`CachedPropertyPackageProvider::new_at` 校验索引、跳过到期 / 远端求值条目，并预读其余所有 manifest / payload | 将缓存 IO、DTO 转换和包资格检查迁出数值 crate；运行只装载指定包，无关包损坏不阻断它 |
| manifest / payload 一致性 | [`rf-store`](../../crates/rf-store/src/package_cache.rs) 已检查包 ID、版本、组分顺序、可选 hash / size；hash 对应规范化 payload JSON，不是任意原始文件字节 | 复用既有校验及摘要口径，不重复实现解析或完整性算法 |
| 数据 → 计算服务 | [Studio 求解桥](../../apps/radishflow-studio/src/solver_bridge.rs) 与 [FFI Engine](../../crates/rf-ffi/src/engine.rs) 都固定构造 `PlaceholderThermoProvider` / `PlaceholderTpFlashSolver`；FFI 另有一套 stored 转换 | 共享本地数据转换与服务装配，保留调用方的结果接收及错误投影 |
| 选包与许可 | [运行选包](../../apps/radishflow-studio/src/workspace_run_command.rs) 先解析显式 / 工程选择和缓存候选，必要时与 entitlement manifest 求交；FFI 接收显式 ID | 应用入口保留选包与授权政策；包存在、数据有效、获准运行分别判断 |
| 组分发现 | [Studio 组分选择](../../apps/radishflow-studio/src/studio_project_component_selection.rs) 与包选择使用静态表；payload 含稳定 ID 和名称，未含分子式或物性适用域 | 提供来自已验证包的组分 / 能力描述；首批只替换现有内置条目的元数据来源，不放开任意缓存包供 GUI 建模 |

现有数值求值本身不读磁盘；问题是数值 crate 承担装载和资格职责。当前 manifest / payload 是数据包协议，不能因增加共享装配就宣称可加载任意算法。

### 职责与依赖方向

已新增 workspace crate **`rf-property-package`**。这是 Studio 和 FFI 两个实际消费者共享的包资源与运行装配边界；不把它放进 `rf-store` 或 `rf-thermo`，避免存储依赖数值实现，或数值层继续依赖缓存。只新增仓库内依赖，不新增外部库；`Cargo.lock` 仅记录 workspace 包和依赖关系变化。

| 归属 | 拥有的职责 | 不拥有的职责 |
| --- | --- | --- |
| `rf-store` | 持久 DTO、schema、路径、读写、manifest / record / payload 校验与规范化完整性计算 | runtime provider、应用授权与求解选择 |
| `rf-thermo` | `ThermoSystem`、`ThermoProvider`、当前物性方程与数值校验 | 文件路径、auth cache、时钟、租约、包注册；移除生产依赖 `rf-store` |
| `rf-flash` | TP Flash 算法，消费热力学服务 | 发现、包加载和许可策略 |
| `rf-property-package` | 纯包描述、内存 / 缓存 / 本地文件来源、共享 stored 转换、包资格与冻结数据、本地服务工厂 | 登录 / 下载 / 刷新租约、修改工程、选择用户的包、调度与发布结果 |
| Studio / headless / FFI | 入口自己的选包规则、授权和运行状态；持有运行上下文，组装 `SolverServices` 并接收结果 | 重复转换 stored 数据、各自硬编码数值服务构造 |
| `rf-ui` / 工程模型 | 正式物性配置命令、修订、草稿、结果旧化及显示投影 | IO、数值服务对象和外部来源句柄 |

依赖方向为 `Studio / rf-ffi → rf-property-package → rf-store + rf-thermo + rf-flash + rf-types`；`rf-flash → rf-thermo` 保持，`rf-property-package` 不依赖 solver、unitops 或 UI。因此它不会通过 `SolverServices` 形成反向依赖，数值 Core 不因装配重新引入 store。

现有 `PropertyPackageManifest / Source / Classification`、`PropertyPackageProvider` 及内存 / 缓存实现迁入新 crate；`ThermoSystem` 等数值类型不移动。仓库 Rust 消费者统一调整 import 和 trait 调用，不从 `rf-thermo` 反向 re-export 新 crate 来伪装兼容。C ABI、COM 及 JSON 协议另按下述兼容要求保持。

### 接口与所有权

当前 Rust 入口如下；类型字段按职责列明，不写入公共 wire schema：

```rust
trait PropertyPackageProvider {
    fn list_packages(&self, at: SystemTime) -> PackageResult<Vec<PackageListing>>;
    fn load_package(&self, package_id: &str, at: SystemTime)
        -> PackageResult<LoadedPropertyPackage>;
}

fn prepare_local_context(package: LoadedPropertyPackage)
    -> PackageResult<LocalThermoContext>;
```

- `PackageResult<T>` 为 `Result<T, PackageLoadError>`；错误保存有类型的原因、可定位的 package ID（全局索引错误可为空）与原始 `RfError`。应用按原因映射既有外层诊断，保留原错误文本和内部原因，不通过解析消息分类；数值方法仍使用原 `RfResult`。
- `PackageListing`：稳定包 ID 和有类型的可用 / 不可用状态，包含可读取的版本 / 来源及具体原因。列表可读索引和 manifest，不启动数值服务，不为列出一个包预读所有 payload；因此“可发现”不等于“数据完整可运行”。整体索引非法仍整次失败，单条资源不可用在该条目报告。
- `LoadedPropertyPackage`：字段私有，拥有已校验的 manifest、`ThermoSystem`、描述及来源证据；不持有路径读取回调或活动文档引用。由缓存、本地文件和内存来源共用校验 / 转换创建，不能通过公开字段绕过身份一致性。manifest 组分与 system 组分必须同序，重复 ID 拒绝。
- `PropertyPackageDescriptor`：包身份、数据版本、来源、计算方法、按 payload 顺序的组分 ID / 名称、能力及数据缺口。分子式、适用域、准确性等级等未提供信息保持未知；现有 methane / ethane 展示分子式可由应用已有标签映射补充，不假称来自 payload。
- `LocalThermoContext`：拥有当前 `PlaceholderThermoProvider` 和 `PlaceholderTpFlashSolver`，以及只读描述 / 来源证据；通过 `thermo()`、`tp_flash()` 的借用访问提供给既有 `SolverServices`。不自借用、不公开可变数据，不引入通用服务容器或插件工厂 trait。
- 来源证据记录实际 package ID / version、schema、规范化 payload 指纹及当前实现标识；已有非空声明摘要继续校验，空摘要 / 零 size 的旧包仍可接受，但不能称为已验证发行者来源。内存来源采用同一规范化数据投影生成指纹，注明来源；不把计算得到的摘要伪造成 manifest 的声明值。实现标识采用当前实现名称与 crate 版本，明确其不保证区分同版本开发构建。

每次运行重新装载或从明确的不可变注册快照取值，再创建独立上下文。冻结之后删除 / 替换缓存文件或修改注册项不影响这次计算；下一次运行重新检查资源和资格。首批不加跨运行可变缓存、`Arc<Mutex<_>>` 或全局注册；未来 AF3 再确定 `Send / Sync`、队列等待期间的资格复查、取消及结果身份。

“冻结”仅指本次物性数据与服务；工程输入快照、文档 / 运行身份和迟到结果接收仍归 AF3。来源证据先作为内部只读查询，不改既有快照 / CLI / FFI JSON；AF5 再决定正式结果与导出如何携带它。

### 资格、能力与错误顺序

装配链路为：**现有跳过 / readiness / 授权 → 解析选包 → 校验该来源的资源资格 → 读取并冻结该包 → 构造本地上下文 → 既有 solver 校验与计算 → 原入口发布结果**。

- 应用继续决定 entitlement、离线租约及是否允许某来源；包层只校验资源自身的到期、来源模式、结构与完整性。`LoadedPropertyPackage` 不是授权凭据，不能让“在磁盘上找到包”绕过入口许可。FFI 本地文件入口不被强加 Studio 的账户政策。
- 缓存索引先整体结构校验；选定条目的到期与远端求值属性在触盘前检查。将每次运行使用的时间显式传入，杜绝数值方法中读取系统时钟。已冻结的同步运行不因中途到期而改变结果；下一次运行拒绝到期资源。
- 能力描述区分“实现支持”与“所需数据齐全”：当前理想液相 / 理想气相、K 值、泡露点、TP Flash、液 / 气相常热容显热分别描述；Antoine 与各相热容按组分报告缺口。无密度、输运、PH Flash、EO / 导数、动态或工程准确性声明；`Overall` 物流焓由当前 Flash 组合得到，不能标成 `phase_molar_enthalpy(Overall)` 能力。
- 目录发现与工厂构造不试算物性。能力描述不为现有求解新增“必须提供全部性质”的总门禁；缺少 Antoine / 热容及工况数值失败仍由实际调用路径明确拒绝，保持旧 `solver.step.*` 诊断。未来新增能力请求接口时再依据明确请求做预检。
- `PackageLoadError` 至少区分 Missing、Expired、UnsupportedSource、MetadataMismatch、IntegrityMismatch、ResourceUnavailable、InvalidData；统一领域投影使用 `property_package.*` 诊断码。未支持的存储 schema 仍由 store 拒绝，归 InvalidData 并保留原错误；完整性与元数据错误不可按缺包 fallback。分类在校验点形成，不从 store 消息猜测错误类型。
- store 已增加有类型的校验阶段出口（身份 / 组分、完整性、schema / 数据），包层复用该结果。旧 `validate_* -> RfResult` 入口保留为兼容投影，原文本不变；不在新 crate 再复制一套存储校验，也不靠重复尝试来推断失败原因。
- Studio 保留既有 `workspace.run.*` 投影及恢复动作：缓存损坏仍归本地缓存不可用，到期 / 仅远端包沿用旧“物性包不可用”外层语义，内部保留具体包原因。FFI 文件缺失等已有 `ffi.property_package.*` 文本和代码保持；新增拒绝场景才新增对应码。现有回归保持旧映射，不通过字符串解析新原因。

### 兼容要求与显式行为变化

| 项目 | 必须保持或明确变化 |
| --- | --- |
| 工程与包格式 | 工程仅保存已有 package ID 和组分；不自动增加版本锁定、升级 manifest / payload schema 或重写缓存。记录版本 / 指纹只保证本次可追溯，不保证重开后自动恢复旧包版本 |
| 入口选包 | Studio 显式 ID、工程 ID 优先、无工程 ID 时既有单候选 / entitlement 交集和多候选拒绝保持；普通空白项目仍不自动选包。FFI 显式参数仍是实际运行选择，不擅自改成以工程字段覆盖 |
| 组分与数值 | manifest / payload 顺序严格一致；物流字典按包序转成向量，仍要求完整组分集合，不自动补零、取交集、排序包、裁剪数据包或把单组分编辑 readiness 当单组分可计算证明。SI、Antoine 约定、298.15 K 显热参考及旧数值结果保持 |
| UI / 命令 | 只替换现有内置元数据来源，保留 label、command ID、未选包时的现有组分候选和草稿 / Undo / Redo / 保存语义；不自动选择、删除或改写工程组分。新包身份先用于非 UI 目录测试，不扩大 GUI 可建模包集合 |
| FFI / headless | C 符号、调用参数、对象所有权、manifest / 结果 JSON、headless v1 / v2 与磁盘只读契约保持；FFI 原有相同 package ID 显式重新装入的替换语义保持，已准备上下文仍持有旧数据 |
| 指定包按需加载（已改变） | 原缓存构造会因任意未过期本地包损坏失败；当前运行只读取已选包。发现报告各条 manifest 的可读性，payload 损坏在该包加载时报告；无关包资源损坏不阻断已选健康包。不可为了避开坏包静默换选另一个包 |
| 来源与到期统一检查（已改变） | 原缓存会过滤到期 / 远端求值，而 FFI 文件入口和内存注册缺少同等资源检查。当前本地装配拒绝显式到期和 `RemoteEvaluationService / RemoteOnly`，保留原包和工程；这属于新增拒绝条件，不能作为纯搬迁隐藏 |
| 内存来源一致性（已改变） | 原内存构造可接受不匹配 manifest / system，重复 ID 后者覆盖。新增受检构造拒绝这些输入；FFI 的显式替换操作单独保留，不与重复批量登记混淆 |

首批界面可消费一次性加载的描述快照；不得在每帧渲染、Home 展示或普通工程打开时暗中读全部缓存 / 求值。新增不可用状态的呈现沿用既有 notice，若需新交互或画板，另按 UI 局部评审；本提案不授权全物性页重绘。

### 实施顺序与验收矩阵

1. **共享资源边界**：新增 `rf-property-package`，迁移纯包类型、来源与转换，移除 thermo 对 store 的生产依赖；冻结现有错误映射和数值 golden。
2. **指定包与运行上下文**：接入按需加载、资格检查、来源证据和独立上下文；Studio / headless 与 FFI 共用工厂，明确验证上表三项行为变化。
3. **发现描述与收口**：让现有内置展示消费同源组分 / 能力元数据；完成跨层与正式基线。若源数据不足，显示未知，不为填满目录增加假参数。

| 验证面 | 必须通过的案例与判据 |
| --- | --- |
| 依赖与纯计算 | 检查 Cargo 生产依赖图，`rf-thermo` 无 `rf-store` 依赖，包装配不依赖 UI / solver；冻结后删除临时包文件仍可重复求值，数值方法不触盘 / 读时钟 |
| 数据一致性 | ID / version / 顺序 / 重复组分、record / manifest 不符、payload hash / size 不符、损坏 JSON / 新 schema 全部明确拒绝；空摘要旧包及规范化 JSON 指纹口径保持 |
| 资格与隔离 | 注入时间验证到期边界；到期 / 远端来源不读取 payload、不网络 fallback；无关损坏包不阻断已选包，坏索引仍失败；无包和多候选仍遵循原选包规则 |
| 冻结与替换 | 两个不同参数包交替运行无串扰；准备后替换注册项 / 文件不影响旧上下文，下次运行看到新数据或拒绝；本次指纹 / 版本与实际 system 一致 |
| 描述真实性 | 相同数据经内存、缓存、FFI 文件入口的包身份 / 组分顺序 / 方法 / 能力 / 内容指纹与数值一致，来源证据按真实入口区分；零 / 单 / 多组分、乱序、缺 Antoine / 缺单相热容均可解释；缺数据不宣称齐全，未知适用域不变成全域有效 |
| 数值与诊断 | 六类单元、代表流程、纯相 / 两相及零流量回归保持；除上表明确变化外，既有输入结果不变。未知组分、缺参数、数值超域与多故障首错回归保持；三项新增拒绝 / 隔离行为单独列出预期 |
| 跨端与存储 | GUI 创建 / 改包 / 组分事务与旧化、Undo / Redo、保存重开；headless 只读请求及 golden、FFI JSON / ABI 回归通过；选中包不可用时保留原输入和文件 |
| 正式收口 | 按实际迁移运行 package / thermo / flash / unitops / solver / store / UI / FFI / Studio 定向测试及 `./scripts/check-repo.sh`；已有 UI 元数据接通后复核 macOS Home / Property / Workbench。Windows / Linux 与 PME 按既定节点保留未验项 |

实现定位：[包资源与发现](../../crates/rf-property-package/src/provider.rs)、[冻结数据与本地上下文](../../crates/rf-property-package/src/runtime.rs)、[数据缺口与来源证据](../../crates/rf-property-package/src/descriptor.rs)。`rf-store` 的详细校验出口保留阶段类型，原 `RfResult` 接口继续投影同一错误文本；`rf-thermo` 已移除 store 依赖。`InMemoryPropertyPackageProvider::try_new_at` 校验批量输入，`insert` 仅接受已校验包并保留显式替换。

Studio / headless 与 FFI 共用工厂；缓存准备失败仍保留旧运行状态，应用日志保留具体 `property_package.*` 原因。内建 UI 目录从 bootstrap 同源内嵌 payload 的不可变描述生成，只保留甲烷 / 乙烷既有展示标签；不做逐帧 IO 或自动放开新包。共享层测试覆盖来源、完整性、资格、按需隔离和冻结，跨端兼容与正式验收记录见 [W43](../devlogs/2026-10/2026-W43.md)。AF3 任务身份和物理准确性仍独立验收。

## B2 配置复核（2026-09-14）

macOS 普通“新建项目”进入物性页时 package / component 均未选择；只选二元烃 Lite 后建模仍禁用，选择 Methane 后可进入建模，移除后恢复禁用。本次确认现有配置门禁与文档说明一致，不把完整组分数据库或多物性包列为本轮缺陷。已有普通空白来源工程在重开后保留二元烃 Lite、Methane / Ethane，并可 rerun。

本次未完整复验带已有流股的组分移除限制、多包冲突或其他平台；具体证据进入 [W38](../devlogs/2026-09/2026-W38.md#2026-09-14-b1-4-平台快捷键与-b2-首轮复核)。

## 用户路径

1. 新建空白项目后进入 `物性`。
2. 选择内置 package。
3. 选择项目组分。
4. readiness 变为可进入流程图建模。
5. 保存并重开项目后仍显示同一 package label 和项目组分。
6. 运行时使用项目文件中的稳定 package id，而不是从 UI 文案或结果反推。

## 范围

本专题纳入：

- 内置 package / component 选择。
- 物性页 readiness。
- Home、左侧项目输入、Property toolbar 和 package 选择卡的同源展示。
- 保存 / 重开后的 package id 与 label 映射。
- official 示例项目的物性状态一致性。

本专题不纳入：

- 第三方 Property Package。
- 完整 Thermodynamics PMC。
- 完整组分数据库。
- 物性参数拟合、相图、Txy / Pxy 曲线和来源管理实现。
- 远端高价值物性资产分发，除非另开授权 / 资产专题。

## 设计边界

### 数据与状态

- 稳定事实源：`flowsheet.thermo.property_package_id` 与项目组分。
- 展示事实源：当前 document / builtin choices 映射出的 localized label。
- 不从运行结果、Home cache 或 UI 文案反推 package 状态。
- 空白项目显示“未选择”是合法状态，不自动伪造默认 package。

### 命令与接口

- package / component 修改必须走正式 command 或 GUI host dispatch。
- 进入流程图建模必须服从 `StudioGuiWindowPropertyPageModel::flowsheet_modeling_enabled`。
- 运行 package resolution 继续作为求解前正式校验，不被 UI readiness 替代。

### UI 与交互

- `物性` 是独立 screen。
- 当前 MVP 不渲染无操作价值的物性二级导航。
- UI 主文案显示可读 label；稳定 id 只用于项目文件、command id、运行请求和内部状态。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 当前口径冻结 | Home、Property、项目输入和 official 示例同源展示 package label |
| M2 | 多内置 package 准备 | 明确多个内置 package 时的显式选择、冲突提示和保存语义 |
| M3 | 物性页长期空间拆分 | 若推进组分查询、参数或分析，先新增独立专题 |

## 验收标准

- 空白项目新建后 package 和项目组分为未选择状态。
- 选择 package / component 后才能进入流程图建模。
- 保存 / 重开后 package id 与 UI label 一致。
- official 示例不会出现“运行已收敛但物性包未选择”的冲突。
- raw package id 不作为主展示文本外露。

## 验证计划

- focused test：Property readiness、package command dispatch、保存 / 重开、Home / Property / 项目输入展示映射。
- 仓库级：阶段收口执行 `./scripts/check-repo.sh`。
- 人工 smoke：当新增物性页 UI 或多个 package 选择行为时，复核 Home、Property、Workbench 三处展示。

## 状态记录

- 当前状态：Active
- 最近更新：2026-10-24，AF2b 共享包装载、单包上下文、来源证据及内置展示已接通，兼容和三项行为变化按本节验收。
- 下一步：按 AF3 / AF5 明确任务与结果来源契约；完整数据库、估算和物性分析按独立专题推进。
