# 结果审阅、诊断与恢复

更新时间：2026-09-15

> 本文定义该专题的能力、开发范围与验收要求；具体迭代切片和优先级以 [当前状态](../status/current.md) 为准。

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
- “工具 → 变量浏览器”可按对象查询输入与结果，旧结果保留快照来源但隐藏数值；应用查询与 [无界面 CLI](../reference/headless-cli.md) 共用变量语义。CLI 运行失败不执行结果读取，返回正式诊断及非零退出码。

已知缺口：

- 完整报表、模板、打印、批量导出和跨快照报表仍未进入范围。
- 结果审阅 smoke 主要覆盖当前轻量路径，没有做完整后处理验收。
- 默认“结果”页与 Runtime 区域共用当前结果复制 / 导出入口；macOS 原生导出已实窗验证，Windows 保留实现、待原生复验；Linux 文件选择器尚未接入，明确提示暂不支持，仍可复制文本。

## 用户路径

1. 完成流程图运行后获得当前 `SolveSnapshot`。
2. 通过右侧模块结果查看当前选中单元的 latest unit result。
3. 通过底部结果表定位 stream 或 unit。
4. 通过状态汇总判断当前结果是否匹配最新文档 revision。
5. 如果运行失败，通过 Run Panel 诊断和 recovery target 跳到具体对象。
6. 编辑文档后旧结果显示 stale，不继续作为当前审阅事实源。

## 范围

本专题纳入：

- 当前快照的轻量文本复制 / 导出，契约见 B2-1。

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

## B2-1：轻量结果输出闭环

2026-09-14 B2 首轮复核：macOS 普通空白来源 Feed–Heater–Flash 保存重开后，结果表显示 Heater Outlet 360 K、1 mol/s、有限焓值，当前快照与文档一致；非法 `-1 K` 草稿 Enter 不提交，不改变当前结果，显式丢弃可恢复。演示物性只证明软件状态与输出一致性，不构成物理准确性验收。

2026-09-14 经确认实施，当前契约：

- 默认结果页显示“复制当前结果 / 导出当前结果”，与 Runtime 区域复用同一 shell 操作；剪贴板和原生文件选择属于 shell，结果仍来自正式 runtime snapshot，不引入第二套命令或结果缓存。
- current 可输出；stale / missing 的两个入口禁用并提示先运行。复制、选择路径和确认覆盖时，从 host 重新取得快照，核对 ID、sequence、document revision 与当前文档 revision，再使用新取得的数据序列化；过时请求拒绝执行。
- 文本包含 snapshot ID、sequence、document revision，以及 `Review / Streams / Units / Steps / Diagnostics` 和 SI 单位。仍是轻量文本，不是稳定公共 API 格式或完整报告。
- macOS / Windows 使用现有 `rfd` 文本保存对话框；Linux 显示平台暂不支持和复制替代入口。用户取消、平台不支持、写入失败和成功分别反馈，不把缺失能力当作取消。
- 路径缺少 `.txt` 时追加，已有扩展名不区分大小写；中文和空格保留。归一化后的目标已存在时，应用再显示具体路径并要求确认覆盖；原生选择器可能先显示系统覆盖提示。取消不写文件，也不改变工程路径、已保存 revision 或 dirty 状态。
- `rf-store::write_text_file` 与 JSON 保存共用 staged write：先向独占创建的同目录临时文件写入并同步，完成后才提交。未授权覆盖使用 hard-link 提交，目标在选择后出现时也拒绝覆盖；文件系统不支持 hard-link 时明确报错，不降级为直接截断写入。已确认覆盖复用现有平台替换与 Windows 备份恢复序列；双重恢复失败仍保留备份路径与错误上下文。不承诺断电或外部进程干预后的恢复。

验收：新增 5 项 shell 与 2 项存储测试，既有 4 项 Windows 替换与故障注入测试继续通过；macOS 全仓 1,154 项测试及严格 clippy 通过。实窗覆盖普通空白来源 Feed–Heater–Flash 重开运行、missing / stale 禁用、复制反馈、原生取消、中文含空格路径导出、覆盖取消及确认、文件读回与工程文件不被导出修改。详细数据见 [W38](../devlogs/2026-09/2026-W38.md#2026-09-14-b2-1-轻量结果输出闭环)。Windows / Linux 原生路径、真实磁盘写满和恢复失败仍未实窗验证。

## B2-2：运行失败诊断与恢复

2026-09-14 完成典型连接 / 参数错误的定位、修复、保存重开和结果恢复复核，并修复默认用户路径缺口：

- 顶部上下文工具栏附近直接显示当前恢复入口，并按正式 recovery action 的 mutation 区分“定位问题（不修改模型）”与“修复模型（可撤销）”。底部失败摘要的恢复按钮提前到详细诊断之前；避免长错误信息和有限面板高度遮住下一步操作。
- 入口继续派发 `run_panel.recover_failure`。只定位不改变 revision、dirty 或 Error 状态，不表示错误已修复；模型修改沿用既有单笔文档事务和 Undo / Redo，修复后仍需运行才能得到当前结果。
- shell 在统一事件出口读取正式 `StudioRuntimeDispatch`：恢复后依据 `applied_target` 请求现有画布定位，检查器与画布选中同一对象。若恢复删除了目标，清理旧导航，不把主动删除误报成锚点失效；禁用 / 未执行的命令不产生恢复反馈。
- 运行和恢复的面板切换以实际执行结果为准，按钮、命令入口、F5 / F8 与直接恢复事件保持一致。运行失败显示检查器 / 消息；成功显示模块结果 / 结果表；文档已修复但未重跑仍无当前结果。
- 不改求解规则、诊断分类、恢复策略、项目格式、依赖或 Core / COM 边界；不由 shell 解析错误字符串来决定修复动作。

验证：5 项新增回归测试覆盖三种恢复入口的定位、只定位不改文档、单次事务与撤销重做、禁用重试、删除目标清理、顶部动作语义与 F5 结果面板；既有失败生命周期测试继续覆盖组分 readiness、参数、缺失引用、重复连接、孤立流股与拓扑失败。macOS 237 项 shell 测试及全仓 1,159 项测试、严格 clippy 通过。

macOS 实窗：普通空白来源 Feed–Heater–Flash，降低 Feed 压力产生 `solver.step.parameter`，恢复定位 Heater 后修正压力并重跑；断开中间流股目标端产生 `unbound_inlet_port`，恢复定位 Flash 后显式重连、保存重开、F5 重跑及当前结果复制通过。具体 revision、数据及环境见 [W38](../devlogs/2026-09/2026-W38.md#2026-09-14-b2-2-运行失败诊断与恢复)。Windows / Linux 原生键盘、极端远距离视口、所有恢复动作逐项实窗仍待验证。

后续进展：B2-3 / B2-4 / B2-5 已分别完成 Valve、Mixer、Cooler 的 macOS 空白建模、参数联动、保存重跑和结果输出，证据见各单元专题与 W38。下一步按当前状态补剩余用户路径；完整报表和公共 API 保持独立规划。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 当前轻量审阅冻结 | Module Results、结果表、状态汇总同源且 focused test 覆盖 |
| M2 | 诊断恢复复核 | 典型失败能定位到 stream / unit / port 或 recovery action |
| M3 | 报表专题评估 | 模板、跨快照和完整报表另开专题；当前快照轻量输出由 B2-1 承接 |

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
- 最近更新：2026-09-14，B2-1 结果输出和 B2-2 失败诊断恢复完成当前 macOS 闭环。
- 下一步：优先补 Valve 普通空白建模与参数联动路径验收；其余单元和跨平台实窗继续按优先级补齐。
