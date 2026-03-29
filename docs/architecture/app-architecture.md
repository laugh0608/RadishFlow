# App Architecture

更新时间：2026-03-29

## 当前目标

现阶段 App 方向不以“尽快做出可点击界面”为优先，而以“先把桌面应用架构边界和未来扩展面定清楚”为优先。

这意味着当前阶段关注的是：

- App 壳层职责
- UI 与 Core 的边界
- 文档模型与状态组织方式
- 画布、属性面板、运行控制、结果展示之间的关系

而不是立即深挖视觉打磨或复杂交互实现。

## 当前已冻结决策

截至 2026-03-29，以下 App 架构决策已经冻结：

1. MVP 保持单文档工作区，不做多文档容器优先设计
2. “单文档工作区”不等于“单文件实现”，代码仍按职责拆分，避免单文件持续膨胀
3. 属性编辑采用字段级草稿态，只有在语义提交时才写回文档
4. 求解控制采用 `SimulationMode` 与 `RunStatus` 分离建模
5. 求解结果采用独立 `SolveSnapshot`，不直接污染 `FlowsheetDocument`
6. 结果快照保留按步展开能力，为后续撤回/前进和操作脚本留接口

## 顶层分层

桌面应用建议固定为三层协作：

1. `apps/radishflow-studio`
2. `crates/rf-ui`
3. `crates/rf-canvas`

并通过 `rf-model`、`rf-solver`、`rf-store` 等核心 crate 提供数据与服务。

## 各层职责

### `apps/radishflow-studio`

这是应用组合根，而不是业务逻辑仓库。

职责：

- 应用启动
- 窗口初始化
- 顶层菜单与工具栏装配
- 工作区与文档生命周期管理
- 将 `rf-ui`、`rf-canvas`、`rf-store`、`rf-solver` 能力组装成桌面应用

不应承担：

- 热力学计算
- 单元求解逻辑
- 画布图元绘制细节
- 项目文件读写细节

### `rf-ui`

这是 App 的应用层和交互层。

职责：

- 当前文档状态
- 选择集状态
- 命令分发
- 面板状态
- 属性编辑事务
- 运行请求与结果视图状态

建议未来逐步形成的子模块：

- `app_shell`
- `document_workspace`
- `selection`
- `commands`
- `inspector`
- `run_panel`
- `result_panel`
- `log_panel`

### `rf-canvas`

这是纯画布能力层，不应承载流程求解或业务决策。

职责：

- 节点绘制
- 端口绘制
- 连线绘制
- 视口缩放、平移、框选
- 命中测试和拖拽反馈

不应承担：

- 物性计算
- 流股状态解释
- 单元求解调度
- 项目持久化

## 文档模型

桌面应用当前建议采用“单文档工作区优先”的模型，而不是一开始就做多文档复杂容器。

当前推荐结构：

- `AppState`
- `WorkspaceState`
- `FlowsheetDocument`
- `UiPanelsState`
- `RunSessionState`

这样做的原因：

- MVP 阶段只有一个主流程图文档就足够
- 更容易把编辑态和求解态分开
- 后续要扩展多文档，也可以在外层再包一层文档容器

这里需要特别说明：

- 单文档工作区是产品形态决策
- 单文件不超过 1000 行只是工程实现约束

两者不能混为一谈。  
即使当前保持单文档模式，源码层面也应拆分 `AppState`、文档状态、命令系统、求解状态和面板状态，而不是堆进一个大文件。

## 推荐状态结构

当前建议先冻结以下状态层次：

- `AppState`
- `WorkspaceState`
- `FlowsheetDocument`
- `UiPanelsState`
- `SelectionState`
- `CommandHistory`
- `SolveSessionState`
- `SolveSnapshot`

建议职责如下：

### `AppState`

- 顶层应用状态
- 当前工作区
- 全局偏好设置
- 应用级消息与日志入口

### `WorkspaceState`

- 当前打开的单文档
- 当前选择集
- 当前面板状态
- 当前求解会话

### `FlowsheetDocument`

- 流程图对象模型
- 单元参数
- 连接关系
- 用户显式设定值
- 文档版本号

不应直接承载：

- 求解中间结果
- 运行诊断缓存
- 画布瞬时交互态

### `CommandHistory`

- 有效编辑命令序列
- 撤回/重做游标
- 未来脚本录制的基础事件源

### `SolveSessionState`

- 当前 `SimulationMode`
- 当前 `RunStatus`
- 当前快照引用
- 最近一次诊断摘要
- “是否需要重新检查/重新求解”的脏标记

## 最小状态草案

在真正开始写 `rf-ui` 之前，建议先把最小状态对象控制在以下轮廓内：

```rust
pub struct AppState {
    pub workspace: WorkspaceState,
    pub preferences: UserPreferences,
    pub log_feed: AppLogFeed,
}

pub struct WorkspaceState {
    pub document: FlowsheetDocument,
    pub selection: SelectionState,
    pub panels: UiPanelsState,
    pub command_history: CommandHistory,
    pub solve_session: SolveSessionState,
}

pub struct FlowsheetDocument {
    pub revision: u64,
    pub flowsheet: Flowsheet,
    pub metadata: DocumentMetadata,
}

pub struct SolveSessionState {
    pub mode: SimulationMode,
    pub status: RunStatus,
    pub pending_reason: Option<SolvePendingReason>,
    pub latest_snapshot: Option<SolveSnapshotId>,
    pub latest_diagnostic: Option<DiagnosticSummary>,
}
```

这里有几个刻意保守的约束：

- `WorkspaceState` 当前只持有一个 `FlowsheetDocument`
- `FlowsheetDocument` 只表示用户编辑态，不持有求解器内部态
- `SolveSessionState` 只引用快照，不直接内嵌完整结果对象
- `CommandHistory` 与 `SolveSessionState` 并列，而不是互相吞并

## 命令对象草案

命令系统建议从一开始就区分“文档命令”和“UI 临时行为”。

当前建议进入历史栈的命令最小集合：

- `CreateUnit`
- `DeleteUnit`
- `MoveUnit`
- `ConnectPorts`
- `DisconnectPorts`
- `RenameUnit`
- `SetUnitParameter`
- `SetStreamSpecification`
- `SetSimulationMode`

当前不建议进入历史栈的行为：

- 框选
- 缩放
- 画布平移
- 面板展开/收起
- 临时输入中的草稿字符变化

这条边界如果现在不冻结，后面 undo/redo 会很快被无价值噪声淹没。

## 草稿态结构建议

字段级草稿态建议不要散落在控件内部，而是集中表达成可检查对象。

当前建议最小结构：

```rust
pub struct FieldDraft<T> {
    pub original: T,
    pub current: T,
    pub is_dirty: bool,
    pub validation: DraftValidationState,
}
```

这样做的好处：

- 输入校验可以发生在提交前
- 不同控件类型可以共享一套“草稿 -> 提交”语义
- 后续如果要做“批量应用本面板修改”，也还有扩展空间

## 求解快照草案

独立结果快照建议至少分为三层：

```rust
pub struct SolveSnapshot {
    pub id: SolveSnapshotId,
    pub document_revision: u64,
    pub status: RunStatus,
    pub diagnostics: Vec<DiagnosticSnapshot>,
    pub steps: Vec<StepSnapshot>,
}

pub struct StepSnapshot {
    pub index: usize,
    pub unit_id: UnitId,
    pub summary: String,
    pub streams: Vec<StreamStateSnapshot>,
}
```

这里最重要的不是字段名字，而是结构关系：

- 快照关联文档修订号
- 步骤序列保持稳定顺序
- 步骤内部记录单元执行结果和流股状态
- 诊断信息与数值结果并列保存

## 关键事件流

### 参数提交流

推荐事件流如下：

1. 用户编辑字段，形成草稿态
2. 用户触发语义提交
3. UI 生成命令并写回 `FlowsheetDocument`
4. 文档修订号递增
5. 命令写入 `CommandHistory`
6. `SolveSessionState` 标记为 `Dirty`
7. 若 `SimulationMode = Active`，立即进入检查与求解流程

### 自动求解流

当模式为 `Active` 时：

1. 结构校验
2. 自由度检查
3. 若可解，进入求解
4. 生成新的 `SolveSnapshot`
5. 更新 `latest_snapshot`
6. 更新 `RunStatus`

### 失败转 Hold 流

当求解报错或不收敛时：

1. 记录诊断
2. 生成失败快照或失败结果摘要
3. `RunStatus` 置为 `Error` 或 `Unconverged`
4. `SimulationMode` 自动切换到 `Hold`

### Hold 恢复流

当用户修正参数后：

1. 文档继续递增修订号
2. `RunStatus` 进入 `Dirty`
3. 系统不自动继续求解
4. 用户手动切换 `SimulationMode = Active`
5. 再次进入检查与求解

## 属性编辑模型

属性编辑当前正式采用“字段级草稿态 + 语义提交”的模式。

具体规则：

- 用户输入过程中，UI 内部持有草稿值
- 草稿值不立即写回 `FlowsheetDocument`
- 当发生 `Enter`、失焦、点击应用等语义提交时，才生成命令并写回文档
- 写回文档后再决定是否触发结构检查与自动求解

采用这个方案的原因：

- 避免用户输入 `100` 时触发 `1 -> 10 -> 100` 三次无意义求解
- 比整页“大草稿后统一应用”更接近工程软件的即时反馈体验
- 能天然接入命令历史，而不是让半成品输入污染撤回栈

当前约定：

- 草稿态不进入命令历史
- 只有成功提交到文档的变更才形成命令
- 只有影响方程系统的提交才触发求解相关检查

## 求解模式与运行状态

求解控制当前明确采用“模式”和“状态”分离建模。

建议的最小模型：

- `SimulationMode`
  - `Active`
  - `Hold`
- `RunStatus`
  - `Idle`
  - `Dirty`
  - `Checking`
  - `Runnable`
  - `Solving`
  - `Converged`
  - `UnderSpecified`
  - `OverSpecified`
  - `Unconverged`
  - `Error`

这两个对象的职责必须分开：

- `SimulationMode` 表示系统当前采用何种运行策略
- `RunStatus` 表示最近一次检查/求解后的真实状态

当前行为约定：

1. 当系统处于 `Active` 时，提交影响模型的参数后先做结构检查和自由度检查
2. 若检查通过，则自动进入求解
3. 若求解报错或不收敛，则 `RunStatus` 更新为 `Error` 或 `Unconverged`
4. 同时系统自动切换到 `Hold`
5. 用户修改参数后，文档进入 `Dirty`
6. 用户手动切回 `Active` 后，再继续检查和求解

这样做的目的，是保留 HYSYS 式交互体验，同时避免把失败状态和运行模式混在一起。

## 结果快照模型

求解结果当前正式规定为独立快照，而不是直接写回文档对象。

建议最小结构：

- `SolveSnapshot`
- `StepSnapshot`
- `UnitExecutionSnapshot`
- `StreamStateSnapshot`
- `DiagnosticSnapshot`

其中：

- `FlowsheetDocument` 表示“用户当前编辑的真相源”
- `SolveSnapshot` 表示“某个文档版本的一次求解结果”

这种分离有几个直接好处：

- 支持撤回/重做时保留结果快照边界
- 支持未来比较两次求解差异
- 支持按步回放求解过程
- 支持未来的脚本录制与自动化回放
- 避免求解结果污染用户尚在编辑的文档状态

## 按步操作与后续扩展

既然目标是吸收 Aspen、HYSYS、PRO/II 的长处，就不应只停在“得到最终结果”。

当前建议从一开始就保留以下扩展点：

- 每次提交形成显式命令
- 每次求解形成独立快照
- 快照内部保留步骤序列
- 步骤内保留单元执行和流股状态结果

这样后续可以自然扩展：

- undo / redo
- 单步回放
- 操作脚本
- 结果差异比较
- 自动化验证脚本

## 命令与状态变更

建议 UI 侧逐步建立显式命令模型，而不是任由控件直接修改底层对象。

当前建议的命令类别：

- 文档命令：新建、打开、保存、另存为
- 画布命令：新增单元、删除节点、连接端口、移动节点
- 属性命令：编辑参数、重命名单元、修改流股设定
- 运行命令：校验、运行、停止、清空结果

这样做的好处：

- 便于后续加入 undo/redo
- 便于把 UI 操作映射为可测试事务
- 便于将来接入自动化或脚本入口

补充约定：

- 只有语义提交后的有效变更才进入命令历史
- 纯画布浏览行为不进入命令历史
- 纯 UI 布局变化默认不触发求解
- 纯几何移动是否进入命令历史，可进入 `M3` 前再细化

## Core 与 UI 的数据边界

App 不应直接操作底层求解细节，而应通过稳定的数据结构与服务入口交互。

当前建议边界：

- `rf-model` 提供文档级对象模型
- `rf-store` 提供保存与加载
- `rf-solver` 提供运行入口
- `rf-ui` 只持有对这些能力的调用结果和展示态

求解结果边界进一步补充为：

- `rf-solver` 输出结果快照
- `rf-ui` 决定如何展示快照
- `rf-store` 后续决定是否持久化快照或只持久化文档
- `rf-model` 不直接吞入求解器内部态

## 当前阶段不急着做的 UI 能力

- 复杂 Dock 系统
- 多文档标签页
- 可停靠工具窗口
- 复杂主题系统
- 高级快捷键系统
- 运行时插件化 UI

这些内容未来可能需要，但当前阶段会分散地基建设注意力。

## 近期建议

在真正写 `rf-ui` 和 `rf-canvas` 代码之前，建议先完成以下基础设计项：

1. 冻结 `AppState` 和 `FlowsheetDocument` 的基础结构
2. 冻结“命令对象”与“面板状态”的划分
3. 冻结 `SimulationMode` / `RunStatus` / `SolveSnapshot` 三者的关系
4. 冻结画布层只处理几何与交互，不处理求解
5. 冻结求解入口只由应用层触发

## 当前仍待细化的问题

以下问题仍值得在正式进入 App 主线前继续细化，但不再属于“方向未定”：

1. `FlowsheetDocument` 的版本号策略采用递增整数还是更显式的修订号对象
2. 草稿态是按字段统一抽象，还是按控件类型区分文本/数字/枚举草稿
3. `SolveSnapshot` 是否默认只保留最新一次，还是保留有限历史窗口
4. `Hold -> Active` 的恢复入口放在全局运行栏、文档状态栏还是两者同时提供
5. `MoveUnit` 是否在 MVP 第一版进入撤回历史，还是先视为轻量画布操作
