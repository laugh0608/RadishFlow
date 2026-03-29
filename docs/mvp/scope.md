# MVP Scope

更新时间：2026-03-29

## MVP 目标

第一阶段 MVP 目标保持不变：

构建一个以 Rust 为核心、以 Rust UI 为主界面、以 .NET 10 暴露 CAPE-OPEN Unit Operation PMC 的最小稳态流程模拟闭环，并让至少一个自有单元模型可被外部 PME 识别与调用。

## 当前冻结范围

第一阶段当前冻结为以下内容：

- 二元体系
- 最小物性参数集
- 简化热力学模型
- `TP Flash`
- 流股对象
- 单元模块：`Feed`、`Mixer`、`Heater/Cooler`、`Valve`、`Flash Drum`
- 无回路或极简回路的顺序模块法
- JSON 项目格式
- 一个可注册的自有 CAPE-OPEN Unit Operation PMC

## 明确不做

以下内容当前阶段明确不做：

- 加载第三方 CAPE-OPEN 单元
- 加载第三方 CAPE-OPEN Thermo/Property Package
- 完整 Thermodynamics PMC
- recycle 全功能收敛
- 动态模拟
- 大规模组分数据库
- UI 视觉精修优先级高于内核闭环

## 当前阶段细化决策

为避免范围漂移，当前阶段补充冻结以下实现细节：

- 统一使用 SI 基本单位，温度用 K，压力用 Pa，摩尔流量用 mol/s
- 流股组成先统一为摩尔分率，不在第一轮引入质量基和体积分率切换
- 相标签当前只保留 `overall`、`liquid`、`vapor`
- `rf-model` 只负责对象模型，不先塞进求解策略和 COM 语义
- `rf-thermo` 与 `rf-flash` 先定接口，再补 Antoine、Raoult 和 Rachford-Rice
- `.NET 10` 适配层在 `M4` 前只允许文档和最小占位，不提前展开复杂运行时实现

App 与交互层当前进一步冻结以下口径：

- MVP 保持单文档工作区，不急于做多文档容器
- 单文档工作区不等于单文件实现，源码仍按职责拆分
- 属性编辑采用字段级草稿态，语义提交后才写回文档
- 求解控制采用 `SimulationMode(Active/Hold)` 与 `RunStatus` 分离模型
- 求解结果采用独立 `SolveSnapshot`，不直接覆盖文档对象
- 结果快照应保留按步展开能力，为撤回/前进和脚本化扩展留接口

## 当前阶段优先目标

在真正恢复主线功能推进前，当前阶段优先目标先调整为仓库地基建设：

- 完善仓库规范
- 完善代码与文档格式规范
- 建立分支、PR 和 CI 基线
- 完善 App 架构规划
- 完善设计文档与进度文档

当前判断逻辑是：

- 这些工作不直接产出功能，但会决定后续功能开发是否可持续
- 在仓库还很新时完成这些约束，成本远低于中后期补治理
- 当前主线还没有复杂历史包袱，适合现在就冻结工程基础口径

## 近期开发节奏

当前建议以周为单位推进，先把主线拆细：

### 2026-W13

- 完成仓库骨架初始化提交
- 建立第一批 Rust 基础类型和领域模型骨架
- 完善初始化文档、协作约定与周志体系

### 2026-W14

- 完善分支与 PR 治理规则
- 建立 GitHub Actions PR 检查
- 建立文本编码、文件格式与 Rust 基础验证脚本
- 完善 App 架构与当前阶段开发规划文档

### 2026-W15

- 冻结 `AppState`、`WorkspaceState`、`FlowsheetDocument` 的基础结构
- 冻结字段级草稿提交流程和命令历史边界
- 冻结 `SimulationMode` / `RunStatus` / `SolveSnapshot` 关系
- 明确 UI 交互层与求解层的数据边界

### 2026-W16

- 在 `rf-thermo` 中实现 Antoine 饱和蒸气压
- 在 `rf-thermo` 中实现理想体系 `K` 值估算
- 在 `rf-flash` 中实现 Rachford-Rice 和真正的二元 `TP Flash`
- 在 `tests/thermo-golden` 与 `tests/flash-golden` 建立首批黄金样例

### 2026-W17

- 在 `rf-unitops` 中建立 `Feed`、`Mixer`、`Flash Drum` 的最小统一接口
- 在 `rf-flowsheet` 中建立端口连接与基本校验
- 明确单元输入输出的标准流股接口

### 2026-W18

- 在 `rf-solver` 中实现无回路顺序模块法
- 在 `rf-store` 中建立最小 JSON 格式
- 在 `examples/flowsheets` 中增加第一个可求解示例

### 2026-W19 以后

- 设计 `rf-ffi` 的句柄式 C ABI
- 衔接 `.NET 10` 适配层
- 再开始 PME 侧人工验证

## 当前阶段的判断标准

当前不是“做得多”就对，而是满足以下判断标准才算推进正确：

- 边界清晰
- 工作区始终可 `cargo check`
- 文档、代码和阶段目标互相一致
- 不把 `M4/M5` 的复杂度提前压进 `M2/M3`
