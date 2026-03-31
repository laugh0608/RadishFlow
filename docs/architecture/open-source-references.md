# Open-Source References

更新时间：2026-03-31

## 文档目的

本文档用于冻结 `RadishFlow` 当前阶段可参考的开源流程模拟、热力学与工程计算项目，并区分：

- 哪些项目可在许可证允许范围内借鉴实现代码
- 哪些项目只适合作为架构、行为或测试样例参考
- 哪些项目对当前 `M3/M4` 阶段最有价值

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

对 `RadishFlow` 当前阶段，外部参考价值按以下优先级理解：

1. 直接帮助 `rf-thermo`、`rf-flash`、`rf-unitops`、`rf-flowsheet`、`rf-solver`
2. 直接帮助黄金样例、端到端测试和自动化入口设计
3. 再考虑更长远的 EOS、反应、优化和更复杂物性框架

这意味着：

- 当前最值得吸收的是热力学接口、物性关联式、flash 算法、单元模块边界和 flowsheet / solver 组织方式
- 当前不应因为外部参考项目功能很大，就把动态模拟、完整 Thermo PMC、复杂优化建模提前带入 MVP

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

## 当前最推荐的吸收顺序

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
- [IDAES GitHub 仓库](https://github.com/IDAES/idaes-pse)
- [Cantera GitHub 仓库](https://github.com/Cantera/cantera)

补充说明：

- 上述许可证口径以各项目官方仓库当前公开信息为准
- 若未来确实要直接引入第三方实现代码，落地前仍应再次复核目标仓库的 LICENSE、NOTICE 与归属要求
