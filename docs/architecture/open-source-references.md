# Open-Source References

更新时间：2026-11-14

## 文档目的

用途：维护 `RadishFlow` 可参考的开源流程模拟、热力学与工程计算项目及引入边界。
读者：选择外部参考、设计领域接口或评估第三方实现的维护者。
不包含：商业产品能力对标、已批准依赖清单和未来功能排期。

本文区分：

- 哪些项目可在许可证允许范围内借鉴实现代码
- 哪些项目只适合作为架构、行为或测试样例参考
- 哪些项目与当前功能切片或后续数值研究相关

本文档不是法律意见，但作为当前仓库的工程执行口径：

- 默认优先参考 `MIT`、`BSD-3-Clause`、`Apache-2.0` 项目
- `GPL` / `LGPL` 项目默认不直接迁移实现代码
- 公式、关联式与公开算法优先回到论文、标准或项目文档后自行重实现

## 当前引入规则

### 允许优先考虑直接借鉴实现的许可证

- `MIT`
- `BSD-3-Clause`
- `Apache-2.0`

当前建议：

- 若直接借鉴实现代码，必须在落地前逐项核对原项目 LICENSE
- 保留必要的版权和许可证文本
- 在提交说明或代码注释中记录来源仓库、文件和用途

### 默认只借鉴思路、不直接迁移实现的许可证

- `GPL-3.0`
- `GPL-2.0`
- `LGPL`
- 其他 copyleft 或许可边界不清晰的项目

当前建议：

- 只吸收架构、模块拆分、测试组织和公开算法思路
- 不复制源码、不做改写式移植
- 若后续确需引入，必须单独评估许可证兼容性

## 当前优先级判断

外部参考顺序服从 [当前状态](../status/current.md) 与 [开发路线图](../radishflow-mvp-roadmap.md)。2026-09-13 起先完善基础功能与使用闭环；计算内核参考价值高，不代表所有功能必须先等待数值研究。

1. 基础功能切片优先吸收组分组织、工程文档、流程图编辑、命令、运行诊断、报告与自动化边界的经验。
2. 数值切片再按具体模型选择 thermo / flash / unitops / solver 参考，同时建立独立验证来源。
3. 长期稳态、动态、瞬态及多求解方式的领域边界见 [模拟平台规划](simulation-platform.md)；不因参考项目庞大而提前引入无消费者的框架或整套实现。

下列项目条目中的 `M3/M4` 价值判断保留为原 MVP 选型背景，不作为现行优先级。许可证描述也不代替引入具体版本前的重新核验；收录参考项目不代表批准引入第三方依赖或数据，也不改变现有排期。

## 可直接借鉴代码的参考项目

### `FeOs`

- 项目：[`feos-org/feos`](https://github.com/feos-org/feos)
- 许可证：`Apache-2.0` / `MIT`
- 技术栈：Rust
- 当前最值得借鉴的点：
  - EOS 与状态对象抽象
  - 参数结构、模型组织和 trait 风格接口
  - Rust 原生热力学库的 crate 组织方式
- 对 RadishFlow 的当前价值：
  - 对 `rf-thermo` 后续从 Antoine / ideal K-value 向更强 EOS 扩展最有帮助
  - 适合借鉴长期的热力学内核结构，而不是直接照搬其完整研究型范围

### `thermo`

- 项目：[`CalebBell/thermo`](https://github.com/CalebBell/thermo)
- 许可证：`MIT`
- 技术栈：Python
- 当前最值得借鉴的点：
  - phase equilibrium / flash 接口设计
  - mixture property package 组织方式
  - 广泛的工程测试和边界情况处理
- 对 RadishFlow 的当前价值：
  - 适合作为 `rf-thermo` / `rf-flash` 的实现参考与黄金样例来源
  - 适合提取接口思想和测试样例，不建议把 Python 风格对象模型原样照搬到 Rust

### `chemicals`

- 项目：[`CalebBell/chemicals`](https://github.com/CalebBell/chemicals)
- 许可证：GitHub 仓库标记为 `MIT`
- 技术栈：Python
- 当前最值得借鉴的点：
  - Antoine 等常见关联式
  - 纯组分数据与工程常数组织方式
  - 可用于热力学与 flash 基线验证的公开计算函数
- 对 RadishFlow 的当前价值：
  - 适合补 `rf-thermo` 的关联式、参数样例和黄金测试
  - 适合做公开算法和数据组织的参考来源

### `fluids`

- 项目：[`CalebBell/fluids`](https://github.com/CalebBell/fluids)
- 许可证：`MIT`
- 技术栈：Python
- 当前最值得借鉴的点：
  - 阀门、压降、管道与工程经验公式
  - 较轻量的工程计算函数组织方式
- 对 RadishFlow 的当前价值：
  - 对后续 `Valve`、部分 `Heater/Cooler` 配套工程公式实现很有帮助
  - 当前不是 M3 第一优先级，但应尽早纳入后续单元模块参考池

### `CoolProp`

- 项目：[`CoolProp/CoolProp`](https://github.com/CoolProp/CoolProp)
- 许可证：`MIT`
- 技术栈：C++
- 当前最值得借鉴的点：
  - 热物性核心的库化接口设计
  - 跨语言 wrapper 组织方式
  - 状态查询 API 与多语言绑定经验
- 对 RadishFlow 的当前价值：
  - 对 `rf-thermo` 长期库化、`rf-ffi` 边界与外部调用接口设计有帮助
  - 当前更偏长期参考，不是 `M3` 的最短路径

### `ThermoPack`

- 项目：[`thermotools/thermopack`](https://github.com/thermotools/thermopack)
- 许可证：`Apache-2.0`
- 技术栈：Fortran，带 C/C++ / Python 接口
- 当前最值得借鉴的点：
  - 更成熟的 EOS / phase equilibrium 能力组织
  - 多语言绑定与数值内核分层方式
- 对 RadishFlow 的当前价值：
  - 对未来从 MVP 理想体系扩展到更真实 EOS 很有帮助
  - 但语言栈距离较远，当前以架构和算法组织参考为主，直接借鉴实现成本较高

## 只借鉴架构与行为的参考项目

### `DWSIM`

- 项目：[`DanWBR/dwsim`](https://github.com/DanWBR/dwsim)
- 许可证：`GPL-3.0`
- 当前最值得借鉴的点：
  - `Interfaces / FlowsheetBase / FlowsheetSolver / UnitOperations` 的模块拆分
  - 自动化 API、测试入口和示例流程组织方式
  - 图形对象与求解对象分离、由连接关系驱动求解顺序的整体思路
- 当前边界：
  - 只吸收架构和行为经验
  - 不直接复制实现代码
  - 不把其大而全的范围提前压入当前 MVP

### `ChemE-as-Code`

- 项目：[`ChemE-as-Code/core`](https://github.com/ChemE-as-Code/core)；研究依据：[论文 v1](https://arxiv.org/html/2611.07947v1)。2026-11-14 核对仓库标记为 `AGPL-3.0`，属于本节仅借鉴架构与行为的范围，不复制源码或改写式移植；后续引入代码、数据或资产须另行核对具体许可。
- 参考价值：机器可读工程文件、确定性计算工具、来源 / 输入指纹、跨文件变更影响，以及规则覆盖与独立审阅的区分。论文案例证明工作流可执行，不证明工程设计已获独立验收或已建立生产率收益。
- 本项目落点：[工程变更影响](../topics/engineering-basis-and-cases.md#工程变更影响与证据复核)、[分层检查与工程差异](../topics/results-review-diagnostics.md#工程检查与交付规划尚未实现)。现有 JSON、稳定身份、正式事务与结果资格继续使用，不把 Git、DEXPI、外部 Agent 或多文件容器设为基础功能前置。
- 本次仅登记参考与规划；未安装工具、引入依赖、迁移工程或启动整厂设计能力。文献转述核对与当前实施边界见 [W46](../devlogs/2026-11/2026-W46.md#2026-11-14两篇文章复核与工程变更规划吸收)。

### `Flowstead`

- 项目：[`HazemMancy/Flowstead`](https://github.com/HazemMancy/Flowstead)
- 许可证：`GPL-3.0-or-later`，见上游 [LICENSE](https://github.com/HazemMancy/Flowstead/blob/main/LICENSE) 与 [NOTICE](https://github.com/HazemMancy/Flowstead/blob/main/NOTICE.md)；其 DWSIM 引擎及第三方组件另有相应许可。
- 参考定位：工程工作台、交互、诊断与交付体验的专项参考；不是 RadishFlow 的核心架构模板或已批准依赖。
- 审阅范围：2026-11-01 对上游 `main` 文档和代表源码做静态抽查，未构建或运行 Android 应用。以下为当时观察，不构成其完整能力或工程精度验收。
- 技术路线：Kotlin / Jetpack Compose 前端、`.NET 10` 引擎层与 DWSIM 10.2.12；支持手机内嵌引擎及自托管服务。稳态流程与热物性主要复用 DWSIM，另有自建工程与动态模型，见 [项目说明](https://github.com/HazemMancy/Flowstead/blob/main/README.md)。

值得借鉴的点：

| 方向 | 参考内容与来源 | 对 RadishFlow 的价值 |
| --- | --- | --- |
| 画布可读性 | [端口朝向、正交连线、回流绕行与命中区域](https://github.com/HazemMancy/Flowstead/blob/main/android/app/src/main/kotlin/app/flowstead/ui/flowsheet/CanvasGeometry.kt) | 完善流程图表达与交互；二维符号的立体视觉不代表真实设备几何或三维模拟 |
| 对象状态与结果资格 | [缺项及依赖等待传播](https://github.com/HazemMancy/Flowstead/blob/main/android/core/src/main/kotlin/app/flowstead/core/model/Readiness.kt)、[结果过期与缺值表达](https://github.com/HazemMancy/Flowstead/blob/main/android/core/src/main/kotlin/app/flowstead/core/model/Results.kt) | 让诊断定位到对象；布局变化与求解输入变化分开。部分流程求解需按我们的任务与结果契约另行设计 |
| 设备工程检查 | [求解后设备检查](https://github.com/HazemMancy/Flowstead/blob/main/engine/src/Flowstead.Engine.Worker/UnitChecks.cs)：泵入口汽相、压机入口液相、压力方向、换热温度交叉等 | 分别表达数值收敛、工程适用性与指定校核结果；检查必须依赖已验证的物性及单元模型 |
| 工程交付 | [设备数据表](https://github.com/HazemMancy/Flowstead/blob/main/android/core/src/main/kotlin/app/flowstead/core/export/DataSheets.kt)、[热量与物料衡算表](https://github.com/HazemMancy/Flowstead/blob/main/android/core/src/main/kotlin/app/flowstead/core/export/HeatMaterialBalance.kt)，以及 PFD、PDF / Excel 输出的组织方式 | 作为结果审阅和计算记录的产品样板；沿用我们的单位、来源、结果资格与显式采纳边界 |
| 验证组织 | [基准说明](https://github.com/HazemMancy/Flowstead/blob/main/benchmarks/README.md)区分自生成回归值与独立参考；[重算测试](https://github.com/HazemMancy/Flowstead/blob/main/engine/tests/Flowstead.Engine.Tests/RecalculationTests.cs)比较增量与全量求解 | 学习算例组织和对照方法，不把上游自生成预期值直接当作物理准确性基准 |
| 动态场景 | [自建动态模型](https://github.com/HazemMancy/Flowstead/blob/main/engine/src/Flowstead.Engine.Worker/DynamicsSim.cs)以分离器积存、阀门、PIC / LIC、扰动与趋势组成闭环，调用 DWSIM 闪蒸和物性 | 可供未来首个动态场景选题；其显式积分和有限设备覆盖不替代我们的动态模型与数值方法验收 |

许可与采用边界：

- 按本文 GPL 参考规则，只吸收行为、分层、交互和测试组织思路；不复制源码，不做逐段翻译或改写式移植。图标、素材和数据也不得因源码公开而默认复制，使用前须核对具体来源与许可；确需引入时另行评估兼容性。
- 公式、关联式和工程准则回到有来源的论文、标准或教材后自行实现并验证。[分离器计算](https://github.com/HazemMancy/Flowstead/blob/main/android/core/src/main/kotlin/app/flowstead/core/model/SeparatorSizing.kt)中的 K 值、持液时间等默认值不直接成为我们的工程准则，继续遵守[设备工程](../topics/equipment-design-and-rating.md)的数据来源、适用域与采纳契约。
- 上游 `engine-provisional` 只证明回归可重复；抽查的[两相闪蒸案例](https://github.com/HazemMancy/Flowstead/blob/main/benchmarks/03-two-phase-flash/expected.json)尚无独立 `reference`。算例、截图和功能清单均不能替代独立物理验证。
- 本次仅收录参考，不修改代码、依赖、验证基线或现有排期；不据此启动移动端、DWSIM 引擎接入、完整动态或安全泄放功能。Rust Core 与 `.NET` CAPE-OPEN / COM 的既有边界保持。

### `IDAES`

- 项目：[`IDAES/idaes-pse`](https://github.com/IDAES/idaes-pse)
- 许可证：GitHub 仓库提供 `LICENSE.md`，当前文档仅将其视为框架设计参考
- 技术栈：Python
- 当前最值得借鉴的点：
  - process modeling framework 组织方式
  - unit model / property package / solver 的职责分层
  - 规模化流程建模与求解流程设计
- 当前边界：
  - 主要借鉴建模和框架思想
  - 不作为当前 `M3` 的直接实现来源

### `Cantera`

- 项目：[`Cantera/cantera`](https://github.com/Cantera/cantera)
- 许可证：公开资料显示为 `BSD-3-Clause`
- 技术栈：C++
- 当前最值得借鉴的点：
  - phase object 与 thermo / transport 分层
  - 稳定跨语言 API 的组织方式
- 当前边界：
  - 更适合作为相对象、状态对象和库边界设计参考
  - 其主线偏反应与传输，不是当前流程模拟 MVP 的直接主参考

## 原 MVP 数值参考顺序

本节保留当时对数值内核参考的排序，供对应领域研究使用；不构成当前基础功能主线的排序。

### 第一优先级

- `FeOs`
- `thermo`
- `chemicals`

原因：

- 最直接服务 `rf-thermo`、`rf-flash`
- 其中 `FeOs` 与 Rust 技术栈最接近
- `thermo` / `chemicals` 适合转化为算法来源和黄金样例来源

### 第二优先级

- `fluids`
- `DWSIM`

原因：

- `fluids` 对后续阀门与工程经验公式很实用
- `DWSIM` 对 `rf-solver`、自动化入口和整体模块拆分非常有启发

### 第三优先级

- `CoolProp`
- `ThermoPack`
- `Cantera`
- `IDAES`

原因：

- 这些项目更适合作为中长期能力扩展和接口设计参考
- 对当前 `M3` 最小闭环不是最短路径

## 对当前仓库的具体落地建议

### `rf-thermo`

- 优先参考 `FeOs` 的 Rust 抽象方式
- 优先参考 `chemicals` / `thermo` 的关联式、参数组织和测试样例
- 后续若扩 EOS，再看 `ThermoPack` 和 `CoolProp`

### `rf-flash`

- 优先参考 `thermo` / `chemicals` 的 flash 与相平衡计算组织
- 当前仍应坚持“小范围、强回归”的黄金样例策略

### `rf-unitops`

- 后续 `Valve`、部分工程经验公式优先参考 `fluids`
- 单元接口边界继续保持“标准流股输入输出 + 必要服务注入”

### `rf-flowsheet` / `rf-solver`

- 优先参考 `DWSIM` 的 flowsheet / solver 分层思想
- 借鉴其自动化和测试入口思路，但不引入其 GPL 实现代码

## 参考来源

- [FeOs GitHub 仓库](https://github.com/feos-org/feos)
- [thermo GitHub 仓库](https://github.com/CalebBell/thermo)
- [chemicals GitHub 仓库](https://github.com/CalebBell/chemicals)
- [fluids GitHub 仓库](https://github.com/CalebBell/fluids)
- [CoolProp GitHub 仓库](https://github.com/CoolProp/CoolProp)
- [ThermoPack GitHub 仓库](https://github.com/thermotools/thermopack)
- [DWSIM GitHub 仓库](https://github.com/DanWBR/dwsim)
- [Flowstead GitHub 仓库](https://github.com/HazemMancy/Flowstead)
- [IDAES GitHub 仓库](https://github.com/IDAES/idaes-pse)
- [Cantera GitHub 仓库](https://github.com/Cantera/cantera)

补充说明：

- 上述许可证口径以各项目官方仓库当前公开信息为准
- 若未来确实要直接引入第三方实现代码，落地前仍应再次复核目标仓库的 LICENSE、NOTICE 与归属要求
