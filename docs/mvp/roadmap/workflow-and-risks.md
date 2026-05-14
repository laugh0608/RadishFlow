# MVP Roadmap Workflow And Risks

更新时间：2026-05-14

## 用途

用途：保留 MVP 推荐工作流、中后期 Studio 交互方向、首批任务拆分、风险和 DoD。
读者：需要查看路线图历史工作流和风险清单的开发者。
不包含：当前执行状态、验收清单和详细开发流水。

## 推荐工作流

### 内核优先

先保证以下三层完全可本地跑通：

- `rf-thermo`
- `rf-flash`
- `rf-solver`

### FFI 第二

等 Rust 内核最小流程闭环稳定后，再导出 FFI。

### CAPE-OPEN 第三

等 FFI 稳定后，再实现 `.NET 10` 的 CAPE-OPEN 外壳。

### Studio 第四

Rust Studio UI 可以在 `M2` 后并行开始，但不要阻塞 `M4/M5` 主线。

## 中后期 Studio 交互演进方向

以下方向纳入正式规划，但不作为当前 `M2/M3` 的阻塞退出条件：

### 1. 画布视图模式

- 流程图画布后续允许在平面视图与立体投影视图之间切换
- 立体模式优先理解为同一份 flowsheet 的增强展示，而不是单独的 3D 建模系统
- 项目文件、端口语义、连接关系、求解输入输出与命令历史继续只有一套真相源

### 2. 流线状态可视化

- 后续把物流线、能量流线、信号流线纳入统一的画布表现体系
- 每类流线都应支持静态/动态两种显示模式，动态模式用于表达方向、活动性与状态变化
- 推荐把“流线类型”和“运行状态”拆成正交编码，而不是只靠单一颜色承担全部语义
- 推荐配色方向可先按主类型区分：物流偏青绿系、能量流偏琥珀/橙系、信号流偏紫/石板系
- 推荐状态表达优先结合低饱和/半透明、虚线、箭头动效、发光强弱或小型状态徽标；例如未求解或待补全可先走灰态/低饱和表达，收敛后恢复对应类型主色并增强可读性
- 具体配色方案留待真实 UI 主题阶段再做可访问性校验，不在当前路线图里提前写死

### 3. 与 RadishMind 的辅助建模联动

- 对标准单元放置后的常见后续动作，后续允许提供“行为预测 / 候选补全”式辅助
- 第一批建议从标准端口拓扑最稳定的单元开始，例如 `Flash Drum` 的 `inlet / vapor / liquid`
- 候选补全应先以灰态 ghost 入口、出口或连线显示，等待用户按 `Tab` 或显式确认后再写入正式文档
- `RadishMind` 的角色优先是建议排序、命名补全和常见建模模式提示，不直接绕过本地连接校验、端口规则和命令系统
- 这部分需要同步到 `RadishMind` 项目，单独补出 suggestion schema、接受/拒绝动作和 prompt 契约

### 4. 建议的前置任务

- 在 `rf-model` / `rf-flowsheet` 中继续冻结标准单元 canonical ports 与后续能量/信号端口扩展策略
- 在 `rf-ui` 中为 ghost suggestion、接受/拒绝动作和状态叠加预留正式 DTO / state 边界
- 在 `rf-canvas` 中为视图模式、流线样式层和状态叠加层预留分层渲染口径
- 在 `RadishMind` 侧补出“放置单元 -> 返回候选补全列表 -> 用户接受某项”的最小提示词与输出结构

## 建议的首批任务拆分

### Sprint A：基础骨架

- 初始化仓库
- 初始化 workspace
- 建立核心 crates
- 建立文档骨架

### Sprint B：热力学与闪蒸

- 二元组分参数结构
- Antoine
- Rachford-Rice
- TP Flash
- 黄金样例测试

### Sprint C：流程求解

- 流股对象
- 单元模块
- 流程连接
- 顺序模块法
- JSON 示例流程
- 最小集成测试与端到端闭环样例

### Sprint D：互操作边界

- Rust FFI
- .NET Adapter
- 基础互调测试

### Sprint E：PMC 暴露

- Unit Operation PMC
- 注册工具
- PME 冒烟验证

## 风险清单

| 风险 | 说明 | 应对策略 |
| --- | --- | --- |
| 范围膨胀 | 同时想做 PME、Thermo PMC、外部模型加载 | 严格锁死 MVP 不做项 |
| 热力学模型漂移 | 计算结果无法稳定复现 | 建立黄金样例测试 |
| FFI 设计过早复杂化 | 边界接口难维护 | 第一版只做句柄 + JSON |
| UI 干扰主线 | 过早投入画布与视觉细节 | UI 不阻塞内核与 CAPE-OPEN 主线 |
| 交互创新反向侵入内核 | 为 3D 画布、动态流线或 AI 建议过早扩张核心语义 | 先冻结共享 flowsheet 语义与 suggestion 契约，再逐层推进视图与模型实验 |
| PME 兼容不确定 | 外部软件实际行为偏标准之外 | 先选一个目标 PME 做主验证 |

截至 2026-04-27，`DWSIM / COFE` 已完成 water/ethanol 人工验证闭环，CAPE-OPEN / PME 兼容主线应阶段性冻结为回归基线。除非出现明确回归，后续不再主动扩张新的 PME 兼容接口、PME 自动化互调、完整 OLE 持久化或完整 Thermo PMC；主线应回到 Rust Studio 的最小可操作工作台闭环。

同日 Studio 主线已补出第一版可见闭环：内置示例切换、运行求解、结果/诊断查看、中文 shell 选项与系统 CJK 字体 fallback 已进入真实 `egui` 壳层；2026-04-28 又补出路径输入式项目打开入口、Windows 原生打开选择器、打开成功/失败反馈、未保存改动打开前确认、shell 级最近项目列表及其独立 preferences 持久化、结构化流股结果 presentation 与结果区基础本地化；2026-04-29 又继续补出 Result Inspector、失败结果、诊断目标命令、活动 Inspector 详情、通用 action DTO、Stream Inspector 字段级草稿更新/提交和基础文档历史 undo/redo；2026-04-30 又补出 `Save / Save As` 文档生命周期边界、Stream Inspector 多字段批量提交、字段编辑快捷键焦点策略、项目文件 staged write 与 `Save As` 覆盖确认，保存会写回 `*.rfproj.json` 并刷新 `last_saved_revision`，另存为会更新当前项目路径与最近项目列表，多个 valid dirty 字段会合并为一次文档提交；2026-05-01 又补出保存 / 另存失败恢复、Result Inspector 摘要可读性、产出单元诊断关联、当前快照内两股流股对比、Stream Inspector 总体组成字段编辑边界，以及 Canvas pending edit、单类型放置、单元块、物流线、选择反馈、对象列表、焦点气泡、端口 marker、端口 hover、运行/诊断 badge 和对象列表临时筛选；2026-05-02 又补出 Canvas legend、viewport/focus anchor、对象定位结果、pending edit 创建/失败统一 command result 反馈、最近一次 command result 的 command-surface 只读摘要，以及 Feed/Mixer/Heater/Cooler/Valve/Flash Drum 多单元 placement palette，并通过 `pwsh ./scripts/check-repo.ps1` 完成仓库级收口验证；2026-05-03 又补齐多单元 placement 提交端回归矩阵，逐类锁定 `CreateUnit kind`、canonical ports、Inspector 焦点、Canvas focus anchor 和 command result 反馈。2026-05-04 已继续补出本地 Canvas suggestions，让 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum` 与 `Feed + Feed -> Mixer -> Flash Drum` 三类最短建模路径可通过 placement、建议接受、`ConnectPorts` / outlet stream 创建和手动运行走到求解收敛；`Mixer` 入口建议只在 source-only stream 数量与未绑定 inlet 数量匹配时生成，避免多来源场景静默猜测。同日又补齐真正空白项目的固定 MVP 二元热力学基线：打开无组件项目会初始化 `component-a / component-b` 和本地 `binary-hydrocarbon-lite-v1` 缓存，Feed source stream 默认 `T / P / F / z` 会保存落盘，保存后重开仍可手动运行 `Feed -> Flash Drum`。随后又补出逐条 suggestion Apply，允许用户显式接受指定本地连接或 outlet stream 创建建议，并已验证非 focused suggestion 也能先被接受后继续补齐运行闭环。同日还补出 Canvas placement 坐标最小持久化，落点保存到 `<project>.rfstudio-layout.json` sidecar，保存并重开后单元位置可恢复，且仍可继续 Apply suggestions 与运行；同日进一步补出 Active Inspector 单元最新执行结果审阅，选中已运行单元时可看到执行状态、step 序号、summary 和产出流股跳转。随后又把求解步骤、失败摘要、失败详情、Canvas attention 与 Active Inspector port attention 收口到结构化 diagnostic target presentation：port target 只作为只读摘要展示，定位继续复用现有 `InspectorTarget`，不新增端口点击编辑、端口级命令或第二套错误处理状态机。日终又补出 Result Inspector unit-centric 视图：当前快照下结果区可在流股视图之外按单元维度选择、查看最新 unit execution result、关联求解步骤、关联诊断和诊断目标动作；选择仍是 shell 临时态，命令仍走既有 `inspector.focus_unit:*` / `inspector.focus_stream:*`，与 stream selection 互不联动。2026-05-05 起 solver step 的结构化 consumed / produced stream 快照已继续前推到 `rf-ui::StepSnapshot` 与 Studio DTO，让单元最新结果和求解步骤可按“输入流股 -> 单元 -> 产出流股”审阅当前快照；同日还把 MVP 常热容相焓接到 `rf-thermo / rf-flash / Flash Drum outlet / Studio Result Inspector`，stream comparison 继续补齐相结果比较、selector 焓值摘要、unit selector 输入/产出摘要和 base / compared stream Inspector focus action。2026-05-06 又完成 Docs 入口和代码规范收口，并把 `rf-thermo` / `rf-flash` 直接数值 API 的 mole fraction 归一契约前移到接口边界；Studio 同日补齐 Stream Inspector composition normalize、draft discard、run-blocking、受控添加缺失组分和受控删除非最后组分的正式 command surface，新增或删除都不做隐式差值补偿，shell 不保存私有组成状态。当前 Canvas 与 Stream Inspector 都不继续扩周边 presentation 细节；下一步应继续收口结果审阅导航/错误定位、热力学 / 闪蒸基础能力或更明确的布局编辑边界，而不是回到 CAPE-OPEN 兼容扩张，也不直接跳到完整组件库、完整物性包选择器、完整报表导出、结果报表系统、完整画布编辑器、自由连线编辑器、拖拽布局编辑器、视口持久化或视觉精修。

## 2026-05-12 阶段复盘

截至 2026-05-12，M1-M5 都已越过 MVP 的最小完成线：Rust 内核和 Studio 可跑通最小稳态流程，`TP Flash` / `SolveSnapshot` / `rf-ffi` 已有 focused 与集成回归，CAPE-OPEN / PME 侧已有 DWSIM / COFE 人工验证基线。近期围绕 near-boundary、Stream Inspector presentation、`SolveSnapshot` command surface 和 runtime action 的收口仍符合路线图中的风险控制目标，但继续沿同一类细节主动扩测试会进入收益递减。

下一阶段不应直接扩完整组件库、结果报表、完整画布编辑器或新 PME 兼容接口，而应切到 MVP α 验收与发布硬化：

- 定义 MVP α acceptance checklist，覆盖 Rust Studio 用户路径、数值与结果审阅、`rf-ffi` JSON/error、CAPE-OPEN / PME 回归基线和首批交付文档
- 运行仓库级验证与 2-3 条用户视角 smoke，优先修真实验收 blocker
- 保持既有 golden / focused tests 为回归基线，但不主动把 near-boundary 或 command surface 扩成无限矩阵

## 建议的首批交付物

MVP 第一轮完成时，建议至少交付：

- 一个可运行的 Rust workspace
- 一个可运行的最小桌面程序壳
- 一个可求解的二元流程示例
- 一个 Rust FFI 动态库
- 一个 .NET 10 CAPE-OPEN Unit Operation PMC
- 一份目标 PME 验证说明

## Definition of Done

以下条件同时满足时，可认为 MVP 完成：

- Rust 内核可完成最小稳态流程求解
- Rust UI 可打开、编辑并运行至少一个示例流程
- .NET 10 适配层可成功调用 Rust 内核
- 自有 Unit Operation PMC 可被目标 PME 识别并调用
- 有最小自动化测试与人工验证记录
