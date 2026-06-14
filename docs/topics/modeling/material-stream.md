# Material Stream 物流股专题

更新时间：2026-06-14

## 用途

用途：定义当前 MVP 中物流股的字段、单位、组成、连接、结果和诊断边界。  
读者：负责 `rf-model`、`rf-flowsheet`、Stream Inspector、Canvas connection、readiness 和结果审阅的开发者、用户、AI / Agent。  
不包含：完整物流数据库、多基准组成切换、能量 / 信号流股、动态状态历史和工业数据采集。

## 专题层级

- 层级：二级功能
- 父专题：`docs/topics/flowsheet-modeling-and-solve.md`
- 关联专题：`docs/topics/results-review-diagnostics.md`、`docs/topics/property-basis-and-components.md`

## 专题目标

- Material Stream 是当前 flowsheet 的物料连接和结果审阅基础对象。
- 用户能显式输入 source stream 的 T/P/F/z，并在运行后审阅 current snapshot 的 stream result。
- 连接、readiness、结果表和 Inspector 使用同一 stream id / command 口径。

## 当前实现快照

已完成：

- SI 字段：temperature K、pressure Pa、total molar flow mol/s。
- composition 使用项目组分下的 mole fraction。
- Stream Inspector 支持字段级 draft、commit、batch commit、受控添加 / 删除组成条目。
- `inspector.focus_stream:*` 是正式 stream 定位入口。
- 结果表和 Result Inspector 可审阅当前 snapshot stream result。

已知缺口：

- 当前不支持质量分率、体积分率、多单位集或动态历史。
- 物流线视觉不代表自由连线编辑器已完成。

## 范围

本专题纳入：

- Material stream identity、T/P/F/z、overall / phase result。
- source-only / sink / terminal stream 语义。
- 一股一源一汇连接校验。
- stream result、phase rows、composition 和 enthalpy 只读审阅。

本专题不纳入：

- 能量流股、信号流股。
- 多单位集和组成基准切换。
- 完整流线编辑器。
- 动态历史、趋势图和数据采集。

## 设计边界

### 数据与状态

- 项目文件保存 stream 规格和连接，不保存当前运行结果作为文档语义。
- 运行结果来自 `SolveSnapshot`。
- 组成以项目组分为边界，不从 stream 自行扩展组分目录。

### 命令与接口

- stream 字段修改走 Stream Inspector command。
- stream 连接走 flowsheet connection command / Canvas suggestion。
- stream focus 走 `inspector.focus_stream:*`。

### UI 与交互

- source stream 输入在 Inspector。
- 对象树和结果表都可以定位 stream，但不维护第二套选择真相源。
- Canvas 显示物流线，但不承担完整自由连线编辑。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 当前 MVP stream 冻结 | 三类小流程中 source/intermediate/terminal stream 都可输入和审阅 |
| M2 | 诊断和结果复核 | 缺输入、组成异常、结果旧化和 focus command 一致 |
| M3 | 新流股类型评估 | 若推进 energy/signal stream，先开新专题 |

## 验收标准

- source stream 缺 T/P/F/z 或 composition 时 readiness 能定位。
- intermediate stream 和 downstream consumed stream 数值一致。
- terminal stream 可作为最终结果审阅。
- 编辑文档后旧 stream result 不再冒充当前结果。

## 验证计划

- focused test：source stream 输入、composition normalize、stream focus command、intermediate / terminal result、stale snapshot。
- 仓库级：涉及 stream 模型或连接语义时执行 `./scripts/check-repo.sh`。

## 状态记录

- 当前状态：Active
- 最近更新：2026-06-14 建立二级功能专题。
- 下一步：只修当前 material stream 输入 / 连接 / 结果真实 blocker。
