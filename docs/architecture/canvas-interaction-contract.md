# Canvas Interaction Contract

更新时间：2026-05-16

## 文档目的

本文档用于冻结 `RadishFlow Studio` 画布交互层的长期契约，重点覆盖以下三类能力：

- 画布平面视图与立体投影视图的关系
- 物流线、能量流线、信号流线的视觉表达模型
- 基于本地规则与 `RadishMind` 的 ghost suggestion 交互边界

本文档的目标不是立即推动复杂 GUI 实现，而是先把 `rf-ui`、`rf-canvas`、`apps/radishflow-studio` 与未来 `RadishMind` 对接的正式边界冻结下来，避免后续各层各自发明一套交互语义。

## 当前定位

当前阶段，这份文档冻结交互契约，并记录 Studio 已经落地的 MVP α 画布边界。它不承诺以下内容立即进入实现主线：

- 真正的 3D 渲染器
- 高保真动画系统
- 完整能量流 / 信号流核心语义
- 通用 AI 自动建模

当前阶段仍应坚持：

- `M2/M3` 以内核闭环和求解基线为先
- 画布创新不反向侵入 `rf-model` / `rf-flowsheet` 核心语义
- 模型建议不绕过本地连接校验、命令系统与求解诊断
- 已落地的 placement palette、local suggestions、对象选择、layout sidecar 和 viewport 呈现优化都仍属于 UI / shell 边界，不改变 `FlowsheetDocument` 的求解语义

## 核心原则

### 单一真相源

- `FlowsheetDocument` 继续是流程图语义、端口连接和几何持久化信息的唯一真相源
- 平面视图与立体投影视图只是同一份文档的不同渲染投影，不引入第二套文档模型
- ghost suggestion 在被接受前不是正式文档内容

### 语义与表现分层

- `rf-model` / `rf-flowsheet` 只承载正式流程语义
- `rf-ui` 承载建议状态、选中状态、聚焦状态与派生视觉状态
- `rf-canvas` 承载投影、绘制、命中测试和交互反馈
- `apps/radishflow-studio` 负责把 UI 命令、求解结果、授权状态和建议来源编排起来

### 本地规则优先

- canonical ports、连接合法性和文档命令边界由本地规则定义
- `RadishMind` 只提供候选建议与排序，不直接修改文档
- 即使是高置信建议，接受前后都必须遵守本地校验

### 少而稳的前置接口

- 当前先冻结少量稳定 DTO / state 边界
- 不为了未来的 3D、动画或 AI 能力，提前把底层对象模型做成抽象大泥球

## 分层职责

### `rf-model` / `rf-flowsheet`

职责：

- 保存正式单元、端口、连接和几何信息
- 维护 canonical material ports 与正式连接关系
- 承担正式文档命令写回后的校验

当前补充约束：

- 现阶段材料流仍是核心真相源
- 能量流 / 信号流暂只在交互契约中预留，不要求立刻进入核心求解语义

### `rf-ui`

职责：

- 保存 `CanvasViewMode`
- 保存 stream 的派生视觉状态与 suggestion state
- 保存 ghost element 列表、聚焦 suggestion、接受/拒绝结果
- 生成供 `rf-canvas` 消费的稳定画布 DTO

### `rf-canvas`

职责：

- 根据 `rf-ui` 提供的 DTO 执行平面或立体投影渲染
- 渲染节点、端口、正式流线与 ghost element
- 提供命中测试、拖拽反馈、hover 高亮与 suggestion focus 高亮

不承担：

- 猜测流程语义
- 直接决定 suggestion 是否落盘
- 直接调用 `RadishMind`

### `apps/radishflow-studio`

职责：

- 在“用户动作 -> UI suggestion 请求 -> 本地规则 / `RadishMind` 返回候选 -> 用户接受 -> 本地命令写回”之间做编排
- 在接受 suggestion 后触发本地命令和连接校验
- 在建议失效、文档变化或求解状态变化时使 suggestion 失效
- 将 placement palette、suggestion 命令、对象选择、右侧检查器切换和底部消息摘要组织成可复现的 Studio 工作台流程
- 维护 Canvas layout sidecar 的读写与 fallback pinning，但不得把 shell 私有布局状态混入流程语义或求解输入

### 当前 MVP α 已落地边界

截至 2026-05-16，Studio 画布已经具备以下最小闭环：

- 左侧 `放置` 入口可创建 `Feed / Mixer / Heater / Cooler / Valve / Flash Drum` MVP 单元。
- 当前最短可求解路径覆盖 `Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum`。
- 本地 suggestion 可补齐标准材料端口连接和必要 outlet stream；显示动词为 `连接` / `Connect`，不再使用泛化 `Apply`。
- suggestion 接受仍转换为正式 `DocumentCommand::ConnectPorts` 或等价文档命令后写回；接受 / 拒绝本身不进入 `CommandHistory`。
- Canvas 对象选择会驱动右侧 `检查器` / 结果定位，但不缓存第二份求解结果。
- Canvas placement sidecar 使用 `<project>.rfstudio-layout.json` 保存 shell / layout 状态；项目文件 `*.rfproj.json` 仍是流程语义真相源。

## 视图模式契约

### 目标

后续允许画布在平面视图与立体投影视图之间切换，但不得分裂文档语义。

### 建议模型

```rust
pub enum CanvasViewMode {
    Planar,
    Perspective,
}
```

冻结约束：

- `Planar` 是默认编辑视图
- `Perspective` 是增强展示视图，优先承担空间层次感与状态可读性增强
- 视图模式切换不应改变端口连接、文档命令、求解输入或持久化结果
- 视图模式切换默认不进入 `CommandHistory`

### 几何口径

- 文档层继续保存稳定的 2D 几何真相源
- 立体模式所需的深度、抬升、层级偏移和视觉装饰优先由 UI / Canvas 派生
- 如后续确实需要少量 3D 呈现参数，也应先作为视图偏好或样式参数存在，不直接污染流程语义模型

### Viewport 初始呈现

当前 Canvas viewport 的初始呈现属于 UI 状态，不属于流程语义。

冻结约束：

- 打开示例或项目后，shell 可根据当前单元与流股 bounds 做初始 fit-to-content / center，让小流程自然处于可视区域中央。
- 初始 fit-to-content 不写入 `FlowsheetDocument`，也不进入 `CommandHistory`。
- 该行为不等同于自动布线、自由连线、自动整理布局或视口持久化；它只决定打开后的第一帧可视区域。
- 若存在 `<project>.rfstudio-layout.json` sidecar，单元 placement 仍以 sidecar 为准；viewport 只基于这些位置计算初始可见范围。

## 流线视觉模型

### 目标

把“这是什么流线”和“它当前是什么状态”拆成两层表达，避免单一颜色承担全部语义。

### 流线类型

建议先冻结画布层类型枚举：

```rust
pub enum StreamVisualKind {
    Material,
    Energy,
    Signal,
}
```

补充约束：

- `Material` 直接对应当前 MVP 主线
- `Energy` / `Signal` 先作为 UI / Canvas 预留视觉类型，不要求当前立刻落到求解主线
- 若后续核心语义尚未接通，`Energy` / `Signal` 可先只用于编辑占位、辅助连线或未来端口规划

### 流线状态

建议先冻结画布层派生状态：

```rust
pub enum StreamVisualState {
    Suggested,
    Incomplete,
    PendingSolve,
    Converged,
    Unconverged,
    Warning,
    Error,
    Disabled,
}
```

解释：

- `Suggested`：ghost suggestion，尚未写回文档
- `Incomplete`：正式对象已存在，但端口或连接仍不完整
- `PendingSolve`：连接已形成，但当前结果仍待求解或已过期
- `Converged`：当前结果可视为收敛
- `Unconverged`：已有结果但未达到收敛要求
- `Warning`：存在非致命诊断
- `Error`：存在阻断性问题
- `Disabled`：当前被过滤、冻结或非活动显示

补充约束：

- `StreamVisualState` 是 UI 派生状态，不应直接替代求解器内部状态枚举
- 求解器如补出更细诊断，可在 UI 层映射到这些稳定视觉状态

### 动效模式

建议先冻结：

```rust
pub enum StreamAnimationMode {
    Static,
    Directional,
    Pulsing,
}
```

解释：

- `Static`：静态显示
- `Directional`：沿流向显示轻量方向动效
- `Pulsing`：用于强调待关注、待补全或异常

冻结约束：

- 动效是增强信息，不是唯一信息来源
- 禁止只靠动画表达关键状态，静止截图下仍应可读

### 风格编码原则

- 主色优先表达 `StreamVisualKind`
- 饱和度、透明度、线型、箭头动效、发光强弱与徽标优先表达 `StreamVisualState`
- 不把“未收敛一定淡蓝、收敛一定绿色”写死为产品语义
- 具体主题色在后续 UI 主题阶段再做可访问性和对比度校验

当前建议的非冻结方向：

- `Material`: 青绿系
- `Energy`: 琥珀 / 橙系
- `Signal`: 紫灰 / 石板系
- `Suggested` / `Incomplete`: 灰态或低饱和表达优先

## Suggestion 交互契约

### 目标

让标准单元放置后的常见下一步动作，可以以 ghost 形式出现，并在用户确认后快速转成正式文档命令。

### Suggestion 来源

建议先冻结来源枚举：

```rust
pub enum SuggestionSource {
    LocalRules,
    RadishMind,
}
```

原则：

- `LocalRules` 适合标准端口补全、邻近节点连线补全和明显命名补全
- `RadishMind` 适合在局部上下文更复杂时做候选排序和补充推荐
- 两者可并存，但最终都统一进入同一套 UI suggestion 状态

### Suggestion 类型

建议先冻结：

```rust
pub enum GhostElementKind {
    Port,
    Connection,
    StreamName,
}
```

补充约束：

- ghost element 是候选 UI 元素，不是正式 flowsheet 对象
- 只有在用户接受后，才转成正式命令

### Suggestion 生命周期

建议先冻结：

```rust
pub enum SuggestionStatus {
    Proposed,
    Focused,
    Accepted,
    Rejected,
    Invalidated,
}
```

解释：

- `Proposed`: 已生成，等待用户关注
- `Focused`: 当前默认接受目标或 hover 目标
- `Accepted`: 已触发接受，等待本地命令写回
- `Rejected`: 用户明确拒绝
- `Invalidated`: 因文档变化、校验失败或上下文失效被丢弃

### 接受与拒绝语义

冻结以下交互口径：

- `Tab` 默认接受当前第一条高置信 `Focused` suggestion
- 若没有高置信 suggestion，`Tab` 不应强行落盘
- 鼠标点击或显式命令也可接受指定 suggestion
- `Esc` 或明确 dismiss 动作可拒绝当前 suggestion
- 接受 / 拒绝 suggestion 默认不进入 `CommandHistory`
- suggestion 转成正式文档命令后的实际文档变更，才进入 `CommandHistory`

### 接受后的写回流程

必须遵守以下顺序：

1. 用户接受 ghost suggestion
2. UI 生成显式接受动作
3. Studio 将 suggestion 转换为本地文档命令
4. 本地 canonical port 规则和连接校验执行
5. 校验通过后写回 `FlowsheetDocument`
6. ghost 元素移除，正式对象进入文档

禁止以下路径：

- `RadishMind` 直接改文档
- 画布层直接改文档
- 未校验就把 suggestion 写成正式连接

## 建议的最小 UI DTO

下面这组结构只作为冻结边界的方向，不代表当前马上完整实现：

```rust
pub struct CanvasInteractionState {
    pub view_mode: CanvasViewMode,
    pub suggestions: Vec<CanvasSuggestion>,
    pub focused_suggestion_id: Option<CanvasSuggestionId>,
}

pub struct CanvasSuggestion {
    pub id: CanvasSuggestionId,
    pub source: SuggestionSource,
    pub status: SuggestionStatus,
    pub confidence: f32,
    pub ghost: GhostElement,
    pub acceptance: Option<CanvasSuggestionAcceptance>,
    pub reason: String,
}

pub enum CanvasSuggestionAcceptance {
    MaterialConnection(CanvasSuggestedMaterialConnection),
}

pub struct CanvasSuggestedMaterialConnection {
    pub stream: CanvasSuggestedStreamBinding,
    pub source_unit_id: UnitId,
    pub source_port: String,
    pub sink_unit_id: Option<UnitId>,
    pub sink_port: Option<String>,
}

pub enum CanvasSuggestedStreamBinding {
    Existing { stream_id: StreamId },
    Create { stream: MaterialStreamState },
}

pub struct GhostElement {
    pub kind: GhostElementKind,
    pub target_unit_id: UnitId,
    pub visual_kind: StreamVisualKind,
    pub visual_state: StreamVisualState,
}
```

补充说明：

- `confidence` 只用于 suggestion 排序与默认接受策略，不应直接暴露为业务真相源
- `acceptance` 是 suggestion 对应的正式接受载荷；它必须足够描述本地文档写回所需的 stream / port 语义，不能再只靠 `target_unit_id` 临时猜命令
- `reason` 用于解释为什么推荐，便于 UI 提示和后续调试
- `visual_kind` / `visual_state` 允许 suggestion 与正式流线共用同一套渲染语义

## MVP α 画布闭环口径

当前已不再停留在“下一轮才接本地 LocalRules suggestion”的阶段。MVP α 画布闭环按下面口径维护：

1. `Planar` 继续是默认编辑视图，`Perspective` 仍只是后续增强展示预留。
2. MVP 单元放置、对象选择、suggestion focus / accept / reject、离散 layout nudge 都应通过正式 command surface 或 shell-local UI state 进入，不保留长期并行的 widget 私有状态改写分支。
3. suggestion 转成正式文档命令后的实际文档变更才进入 `CommandHistory`；suggestion focus、reject、viewport、面板切换和 hover 不进入文档历史。
4. Layout sidecar 只保存 shell / layout 相关状态；缺少 sidecar 时可以用 transient grid slot pin 出初始位置，但必须在 presentation 中保持可解释，不反向污染 flowsheet 语义。
5. 下一步只收口 viewport 初始居中 / fit-to-content，不扩自由拉线、自动布线、完整拖拽布局编辑器或复杂视图持久化。

## 当前仍待后续细化的问题

1. `Energy` / `Signal` 在核心语义未接通前，是否允许先以纯 UI 占位对象存在
2. `Perspective` 视图是否需要单独的深度排序策略和遮挡规则
3. suggestion 是否需要批量接受，还是严格先从单条接受开始
4. `Tab` 接受是否需要和属性面板焦点、文本输入焦点做更细的快捷键竞争规则
5. `RadishMind` suggestion schema 是否与本地 `LocalRules` 输出完全同构，还是允许额外解释字段
