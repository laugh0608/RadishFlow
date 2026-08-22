# 结果审阅、诊断与恢复

更新时间：2026-06-14

## 用途

用途：定义当前求解结果、结果表、模块结果、状态汇总、诊断和 recovery action 的同源审阅边界。
读者：负责 `SolveSnapshot`、Result Inspector、Module Results、Run Panel 诊断和结果相关 UI 的开发者、用户、AI / Agent。
不包含：完整报表系统、跨快照历史报表、模板导出、打印、批量导出和完整后处理平台。

## 专题目标

- 结果审阅只消费当前 revision 的 latest `SolveSnapshot`，旧结果只作为 stale notice。
- stream / unit 定位、模块结果、底部结果表和状态汇总使用同一套 snapshot / review summary / command 口径。
- 诊断和 recovery action 指向可修复对象，不用 UI 私有状态掩盖真实失败。

## 子专题关系

- Material Stream 结果、phase rows、composition 和 stream focus 细节见 `modeling/material-stream.md`。
- Feed、Heater / Cooler、Flash Drum、Mixer、Valve 的单元结果语义见 `unitops/` 下对应专题。
- 本专题只保留跨对象的 `SolveSnapshot`、结果表、模块结果、状态汇总、诊断和 recovery 统一口径。

## 当前实现快照

已完成：

- Result Inspector 可审阅 stream summary、composition、phase rows、unit latest step。
- 底部结果表 stream / unit 行点击复用 `inspector.focus_stream:*` / `inspector.focus_unit:*`。
- 右侧 `模块结果` current 态使用紧凑单元、状态、step、执行摘要和消费 / 产出流股 chip。
- `window.status_summary` 可扫读 run、convergence、steps、diagnostics、snapshot 一致性和 unit result count。
- 顶部 `结果工具栏` 不再展开所有 result focus command。

已知缺口：

- 完整报表、模板、打印、批量导出和跨快照报表仍未进入范围。
- 结果审阅 smoke 主要覆盖当前轻量路径，没有做完整后处理验收。

## 用户路径

1. 完成流程图运行后获得当前 `SolveSnapshot`。
2. 通过右侧模块结果查看当前选中单元的 latest unit result。
3. 通过底部结果表定位 stream 或 unit。
4. 通过状态汇总判断当前结果是否匹配最新文档 revision。
5. 如果运行失败，通过 Run Panel 诊断和 recovery target 跳到具体对象。
6. 编辑文档后旧结果显示 stale，不继续作为当前审阅事实源。

## 范围

本专题纳入：

- 当前 revision latest `SolveSnapshot` 的轻量审阅。
- stream / unit focus command。
- Module Results、底部结果表、状态汇总之间的同源摘要。
- 诊断目标、recovery action 和 Inspector focus。
- 结果旧化提示。

本专题不纳入：

- 完整报表系统。
- 跨快照历史和多版本结果对比。
- 模板、打印、批量导出。
- 完整收敛曲线和高级后处理。
- 第二套 shell 私有结果状态。

## 设计边界

### 数据与状态

- 当前结果事实源：latest current-revision `SolveSnapshot`。
- `review_summary` 是当前 case-level 摘要，不是第二套结果计算。
- 旧快照只表达 stale notice，不驱动当前结果审阅入口。
- 状态汇总只扫读，不承担完整结果表或报表职责。

### 命令与接口

- stream 定位走 `inspector.focus_stream:*`。
- unit 定位走 `inspector.focus_unit:*` 并进入匹配右侧模块结果。
- recovery action 使用正式 Run Panel / Inspector target command。
- 新结果入口必须能回到正式 command surface。

### UI 与交互

- 右侧模块结果只审阅当前选中单元。
- 底部结果表承接当前 snapshot 的 stream / unit 列表。
- 顶部结果工具栏承接入口和状态扫读，不渲染长对象清单。
- 结果表不维护第二套选择状态。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 当前轻量审阅冻结 | Module Results、结果表、状态汇总同源且 focused test 覆盖 |
| M2 | 诊断恢复复核 | 典型失败能定位到 stream / unit / port 或 recovery action |
| M3 | 报表专题评估 | 若确需导出或报表，先另开完整专题 |

## 验收标准

- 运行成功后 stream / unit 结果可定位。
- 右侧模块结果和底部结果表不展示互相冲突的状态。
- 编辑文档后旧结果不会冒充当前结果。
- 失败诊断能定位到可修复对象。
- 顶部结果工具栏不会退回长列表导航。

## 验证计划

- focused test：result table focus、module results current state、status summary snapshot consistency、stale result、diagnostic recovery target。
- 仓库级：阶段收口执行 `./scripts/check-repo.sh`。
- 人工 smoke：新增结果展示区域或 recovery 入口时复核。

## 状态记录

- 当前状态：Active
- 最近更新：2026-06-14 从结果区 UI 收束和通用小流程结果审阅中拆出独立专题。
- 下一步：只处理当前轻量审阅真实 blocker；完整报表另开专题。
