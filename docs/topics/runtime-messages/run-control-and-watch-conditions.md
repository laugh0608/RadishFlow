# 运行控制与条件监视

更新时间：2026-10-24

## 用途与状态

用途：定义暂停、继续、单步、取消和结构化监视规则。
读者：运行任务、求解执行与单位接口维护者。
不包含：通用脚本、现场联锁、热修改及任意节点重启。

- 层级：二级功能；父专题为 [运行、消息与诊断](../runtime-messages-and-diagnostics.md)。
- 状态：Draft / 规划方向已确认，细化决策与实现尚待所属切片；现有能力不能直接算作新增要求已验收。
- 范围、依赖与 N0—N5 顺序以父专题为准；[需求追踪表](requirements-traceability.md) 记录要求、阶段、场景及证据。

## 当前基础与用户路径

当前 GUI 已接 AF3c 后台流程 / 设备与生命周期，旧兼容入口保持同步执行；`Active / Hold` 保留现有触发语义。未来用户从对象 / 步骤 / 变量创建条件，启动运行，等待安全边界停下，检查现场后继续或取消；任何按钮都需要执行能力支持。

2026-10-24 [AF3c 局部交互六图与提案](../../architecture/designs/studio-background-run-brief.md)完成静态复核，所有者认可 D01—04 并授权实施：同源运行摘要、取消 / 自动触发、设备占用与输出、切换 / 退出等待。本文仍为任务与接收契约真相源；局部设计已认可，产品接线已完成，验证范围见验收专题。

## 监视条件与运行控制

监视规则观察数值 / 事件，产生暂停或停止请求，不修改模型值。阈值截断、重新指定参数等属于模型逻辑或正式编辑，不通过消息层偷偷执行。首批使用结构化条件编辑器，不引入任意脚本或通用 DSL。

示例：`E-101 出口温度 / 模块完成后 / > 180 °C / 首次满足 / 请求暂停并保留现场`。条件记录稳定对象 / 变量身份、求值时机、比较量及单位、触发方式与动作；内部比较经规范 SI，保留原始单位解释，显示单位变化不改条件。缺变量、非有限值、上下文不全或目标失效须显式报错 / 禁用，不当成 false；是否继续运行按规则有效性策略明确决定，不能静默无监视运行。

按能力依次支持模块前后、指定对象诊断、变量范围 / 变化量、循环次数，随后扩展有定义的残差停滞和动态时刻 / 事件。循环规则必须说明观察的是试算值还是接受迭代值、每次还是首次触发；抖动抑制和重新武装策略显式定义。实时监视独立于日志过滤 / UI 采样，不能靠读屏刷新执行判断。

| 控制 | 正式含义 | 边界 |
| --- | --- | --- |
| 暂停请求 | 在下一个声明的安全边界停下并保留可继续状态 | “正在暂停”与“已暂停”分开，不保证立刻中断任意内部运算 |
| 单步 | 执行到所选且支持的下一边界 | 模块步、循环步、动态时间步分别命名；无能力不展示可用按钮 |
| 继续 | 恢复同一有效暂停运行 | 致命失败不能保证继续；观察快照不自动等于重启检查点 |
| 停止 / 取消 | 结束本次任务，保留已收集证据 | 与数值失败、暂停分别呈现，不产生当前完整结果的虚假承诺 |

后台任务、取消协议和安全边界是暂停 / 单步的前置条件。黑箱或第三方调用只能在其承诺边界响应；未来并行任务须说明停止范围和仍在结束的工作，不能标“全局已暂停”而仍有任务修改状态。当前 `Active / Hold` 的运行触发语义不改写为上述暂停状态。

首版暂停允许只读检查；修改物理输入后使原运行不能直接继续，按新修订重新运行，旧记录保留。后台执行使用明确输入身份，迟到结果不得覆盖新工程 / 新修订；运行中的编辑策略在 N3 明确，热修改、回退时间和任意节点重启另评估。条件停止是模拟调试功能，不作为现场联锁或安全控制能力。

## 请求与运行竞争的处理

| 场景 | 必须保持的语义 | 实施前仍需决定 |
| --- | --- | --- |
| 暂停请求发出后任务先完成 | 显示实际完成，不能再倒退成暂停 | 请求回执与终态竞争策略 |
| 连续点击暂停 / 取消 | 不创建互相矛盾的运行状态 | 去重身份、幂等与控制优先级 |
| 第三方调用不能中断 | 显示等待支持的边界 | 能力声明、超时提示、退出行为 |
| 暂停后改变物理输入 | 原运行不可直接继续 | 编辑入口与重新运行事务 |
| 一个采集周期跨过多个条件 | 不依赖显示采样判断命中 | 求值边界、多个命中动作的次序 |
| 无效规则、观察值缺失 | 明确错误及是否允许运行 | 启动前拒绝与运行中停止策略 |

运行中编辑、暂停状态修改、文档关闭和程序退出属于不同操作。应分别规定保留证据、取消等待与保存行为；不能假设关闭窗口等于第三方调用立即结束。用户看到的“已暂停”须来自执行方确认，而非 UI 请求发送成功。

## 切片、待决策与最小验证

- N3 建立运行身份、取消和安全边界，再交付暂停 / 单步；N4 引入结构化条件；N5 的仿真时刻与事件需真实动态能力。退出依据为 MSG-11—MSG-17。
- D-04：N3 实施前定义能力表、请求回执、终态竞争、重复控制、取消优先级、并行停止范围、编辑 / 关闭 / 退出策略和迟到结果处理。
- D-05：N4 实施前定义比较精度、变化量基准、首次 / 重复触发、滞回 / 重新武装、多个条件同时命中，以及非法配置 / 运行中缺值的策略；不得静默失去监视。
- D-06：N5 实施前明确事件时间、回放与实时控制、检查点版本和第三方状态恢复能力；观察记录不作为可恢复承诺。
- 验证见 [S-11—S-17](scenarios-and-acceptance.md#运行控制与条件)：请求竞争、黑箱延迟、单位等价、NaN、过滤不影响触发及修改输入。


## AF3 任务身份、取消与完成接收方案

2026-10-24 基于 `b09873b2` 盘点后，所有者确认方案并授权实施 AF3a。**AF3a—c 已接通身份、后台流程 / 设备、取消与 GUI 产品生命周期**；AF1 保留独立故障跟踪。AF2a / AF2b 已提供内建工厂与冻结物性数据，本轮设计只补现有流程求解和独立设备计算的任务基础，不实现暂停 / 单步、条件监视、算法内强杀、任务持久化、插件、远程计算、新 EOS 或 C2-B / C3。

### AF3 实施前盘点（`b09873b2`）

| 链路 / 代码定位 | 当前可复用事实 | AF3 要解决的缺口 |
| --- | --- | --- |
| GUI → [Facade](../../../apps/radishflow-studio/src/app_facade.rs) → [运行命令](../../../apps/radishflow-studio/src/workspace_run_command.rs) | 草稿、Automatic skip、选包、readiness、entitlement 各有既有入口与拒绝原因；Resume 先检查再激活 | 本地操作链同步执行；新增提交 / 取消不能绕过这些入口，也不能把 Resume 改成恢复暂停的计算栈 |
| [WorkspaceSolveService](../../../apps/radishflow-studio/src/workspace_solve_service.rs) → [solver bridge](../../../apps/radishflow-studio/src/solver_bridge.rs) | 创建 `StudioSolveRequest`，加载包、组装服务、求解、写回 AppState；`Started` 在求解结束后才返回 | 无实际后台提交回执；request 只有 package / snapshot ID / sequence，没有独立运行身份、冻结工程输入或完成接收凭据 |
| [结果与工作区](../../../crates/rf-ui/src/state.rs)、[SolveSessionState](../../../crates/rf-ui/src/run.rs) | revision 变化使结果旧化，历史有限；Active / Hold 管理触发；失败保留既有恢复信息 | `store_solver_snapshot` 使用调用时的当前 revision，`store_snapshot` 直接推进当前状态；只有 revision / snapshot 序号不足以拒绝同 ID 重开、清结果、分叉和迟到完成。序号来自保留历史，不能当唯一 RunId |
| [顺序求解器](../../../crates/rf-solver/src/lib.rs) | 校验图、排执行顺序、逐单元验证 / 构造 / 执行 / 物料化；最终快照有成功步骤与诊断 | `solve` 仅返回最终 `RfResult`，无协作取消或进度入口；失败时内部已完成步骤尚未独立返回，不存在后台 trace 协议 |
| [设备请求与接收](../../../crates/rf-ui/src/engineering/results.rs) | `begin_calculation / begin_rating → execute → complete_calculation`；捕获 document、会话、任务 / case、方法、input / evidence generation、eligibility 和独立 CalculationId | 已能拒绝迟到 / 乱序 / 重开 / 清缓存结果，但 `calculate` 与 [编辑命令](../../../crates/rf-ui/src/engineering/editor/transaction.rs) 仍同步；无取消执行协议、worker 回收或按请求撤销的入口 |
| [设备身份](../../../crates/rf-ui/src/engineering/mod.rs)与[隔离测试](../../../crates/rf-ui/src/engineering/tests.rs) | 不透明 checkpoint 使用实例身份；分叉修改即使 generation 数字相同仍拒绝跨分支完成；Undo 不恢复旧资格 | 可复用其思路，不把设备方法 / 工况资格压缩为流程 revision，也不把 `clear_results` 冒充已停止计算 |
| [Shell 生命周期](../../../apps/radishflow-studio/src/studio_gui_shell/app.rs) | 编辑工程打开成功后整体替换 `platform_host`；打开失败保留原宿主；未知能力工程进入只读覆盖视图，旧宿主仍在 | worker 所有权不能随 project host 被替换而遗失；只读覆盖也要撤销后台结果接收，且不能因渲染提前返回而停止收尾处理 |
| [BootstrapSession](../../../apps/radishflow-studio/src/bootstrap/mod.rs)与[运行宿主](../../../apps/radishflow-studio/src/studio_runtime.rs) | 会话拥有 AppState、临时包缓存根和授权上下文；宿主已有窗口 / timer / effect 身份 | timer / window ID 不等于运行身份；缓存根的生命周期也必须覆盖后台加载，不能只跨线程复制 PathBuf |
| [headless](../../../apps/radishflow-studio/src/headless/mod.rs)与[FFI](../../../crates/rf-ffi/src/engine.rs) | headless 同步消费 Facade 最终结果；FFI 自己持有 Engine 和显式选包，共用 AF2b 物性装配 | GUI 后台化不能让旧一次性响应提前返回“已提交”；C ABI 不新增异步句柄，也不据此宣称同 Engine 可并发调用 |

上述是现有同步设计的异步接入缺口，不表示当前串行求解已发生跨线程数据覆盖。设备尺寸计算在新结果完成时替换旧候选，Rating 重算开始即撤销目标行旧结果，两者的已有资格差异保留。

### 建议职责与输入所有权

**第一批使用仓库内模块和 Rust 标准线程 / 通道，不新增 crate 或外部异步依赖。** `rf-model` 和持久格式不承载任务控制；`rf-thermo / rf-flash / rf-unitops` 不接触 GUI、文档身份或取消按钮。

| 归属 | 建议负责内容 |
| --- | --- |
| `rf-ui` 工作区运行状态 | 流程会话 / 输入 checkpoint、待接收 RunId、结果接收与清结果失效；设备继续用 `EngineeringSession` 的专用 key / generation / eligibility |
| Studio 应用任务模块 | 非 Clone 的运行协调器、提交与取消回执、单 worker 配额、资源租用、进度邮箱、完成接收和回收。宿主适配器只转发正式动作；线程句柄不放进可克隆 AppState、工程、Undo 或 GUI 展示 DTO |
| Shell 所在应用会话 | 持有跨 `platform_host` 替换仍存活的协调器；每帧先处理任务收尾，再处理 Home / 只读视图等提前返回。换工程、最后窗口关闭与退出通过该所有者移交 / 排空任务 |
| `rf-solver` | 提供有类型的可控执行出口；只观察协作取消、报告真实阶段 / 单元边界，保留原校验次序与数值方法。旧 `solve` 委托同一执行循环的无取消模式，保持 `RfResult<SolveSnapshot>` |
| `rf-property-package` | worker 内按明确时间加载已选包并构造本地上下文；资格、来源与数据指纹沿用 AF2b。通用数值 trait 不为调度一律追加 `Send + Sync`；对实际跨线程输入 / 完成类型做编译检查 |

建议流程为：入口草稿 / skip / 选包 / readiness / 授权检查 → 保留执行槽位 → 捕获正式输入和接收凭据 → 返回提交回执 → worker 加载包并计算 → 应用所有者原子判定完成归属 → 发布结果或仅留终止记录 → 回收 worker。入口失败和 Busy 不产生半登记任务；设备 `begin_*` 必须在槽位保留后执行，避免 Busy 先使 Rating 旧结果失效。

流程接收凭据和状态守卫留在 `rf-ui`，含包来源与资源 guard 的实际工作请求留在 Studio，避免新增 `rf-ui → rf-property-package` 反向装配依赖；`rf-solver` 的控制接口不接收文档 / RunId，事件关联由 Studio 包装。

worker 仅拥有 flowsheet 副本或设备 `CalculationRequest`、选包来源快照、加载所需资源及取消句柄；不得借用活动文档、编辑器、UI、Facade 授权客户端或可变注册表，也不把整个 AppState 放入 `Arc<Mutex<_>>`。设备输入继续由既有请求捕获，不扩大为读取活动工程；两个领域只共享运行外壳，不合并结果 DTO。

缓存 provider 的 index / 路径可复制，但临时缓存根必须有覆盖加载阶段的受控租用；旧宿主销毁不能先删除仍在加载的目录。拟由 Studio 缓存所有者提供只延长目录寿命的共享资源 guard，非跨运行可变数值缓存。包读取后冻结实际指纹；外部文件在读取时发生变更按 AF2b 校验明确失败，冻结后不再触盘。

提交时间与开始加载时间分别记录。第一批无任务等待队列，worker 开始加载仍重查资源有效期；应用授权快照记录许可截止和会话代次，在提交及开始握手时确认，过期 / 显式撤销则取消或拒绝开始，不让旧授权无限等待。加载完成需一次应用所有者的 Ready 握手：核对会话 / 输入 / 取消与授权是否仍有效后才能执行数值；等待握手也可取消。开始许可后单纯时间流逝不追溯撤回结果，沿用 AF2b 冻结语义；显式登出 / 撤销则撤销尚未接收任务的发布资格。无网络刷新或远端求值 fallback。

### 身份与接收凭据

下表为拟定内部字段职责，不是公共 wire schema，也不把 snapshot ID 改成 RunId：

| 身份 | 产生 / 变化时机 | 判定用途 |
| --- | --- | --- |
| 文档 DocumentId | 沿用正式文档身份 | 关联工程，但不能单独区分同 ID 重开或副本 |
| WorkspaceSessionId | 每次实际新建、打开 / 重开、独立工作区产生新不透明身份；只读切入撤销旧会话接收 | 隔离同名、同路径、同 DocumentId 和同 revision 的不同打开实例；保存 / 另存为同一内存工作区不换身份 |
| InputCheckpoint + revision | 每次流程正式语义提交、Undo / Redo 产生新 checkpoint；revision 用于展示及旧协议 | 防止不同分叉在相同 revision 下互收结果；回到相同数值也不恢复旧执行凭据。视图、语言、单位显示、选择与未提交草稿不推进此 checkpoint |
| ResultAcceptanceEpoch | 清结果、撤销执行资格、离开当前工作区时更新 | 清历史后不因 revision 未变而接收旧结果；只撤销未接收资格，不单凭 epoch 变化抹除已有有效结果；不写工程、不清脏、不进入 Undo |
| RunId | 每次被接受的运行意图产生不可复用身份，含失败 / 取消；不从历史列表反推 | 绑定提交、控制、进度、完成及待接收槽位；重复点击 / 重发同一控制引用相同 RunId，显式重试必须是新 RunId |
| Package selection / provenance | 捕获入口实际解析的 package ID，加载后关联真实 version / fingerprint / implementation | 显式覆盖与工程缺省保持各入口原义；执行中更换工程选包失效旧任务，缓存被替换不重写已经冻结的来源 |
| 设备领域 key | 沿用 CalculationId、会话、task / case、方法、input / evidence generation 与 eligibility | 通用 RunId 关联控制；真正发布必须再通过 `complete_calculation`，不得只比较工作区 revision |

身份由拥有者签发、字段私有；建议复用设备不透明实例标识的方式，数值序号只用于有界记录和展示，计数耗尽返回结构化失败、不回绕。事务临时 clone 保留同一会话，只有获准提交的状态才能登记任务；独立 fork / 重开要显式新会话，不能将 Clone 当作创建独立运行宿主。测试须覆盖未修改 clone 与分叉修改两种情况。

流程采用 `prepare / execute / complete` 三段能力，已冻结请求不含可变 AppState；完成包携带原凭据和 `Succeeded / Failed / Cancelled` 执行事实。接收时同时检查当前会话、输入 checkpoint、接收 epoch、待接收 RunId 和取消 / 授权撤销位；通过后使用捕获的 revision 构造 UI 快照，再推进当前状态。成功和失败使用同一接收门禁；迟到失败不能把新工程改成 Error / Hold，迟到进度也不能改当前运行栏。

接收须一次性消费凭据：重复完成为 AlreadySettled，无第二次插入、日志计数、快照序号或状态变化；异会话 / 旧输入 / 已清结果分别给结构化拒绝原因，不解析文本推断。旧请求失败也不能清掉较新请求的 pending；当前输入未匹配时不得调用现有无条件 `store_solver_snapshot` 或 `record_failure`。snapshot 原显示命名与序号在同步兼容路径保持，任务身份与之独立。

### AF3a 已实现契约（2026-10-24）

- [rf-ui 流程接收](../../../crates/rf-ui/src/flow_run.rs)持有私有会话、输入 checkpoint、接收 epoch 与 `FlowRunId`，身份采用不可伪造实例；显示快照序号独立，`next_solver_snapshot_sequence` 返回 `RfResult<u64>`，耗尽返回 `workspace.run.snapshot_sequence_exhausted`，不回绕。
- `AppState::prepare_flow_run` 捕获正式 Flowsheet、DocumentId、revision 与凭据；正式提交、Inspector 提交和 Undo / Redo 更新输入 checkpoint，清结果和显式登出撤销接收 epoch。撤销本身不抹掉先前有效结果；保存 / 另存为、Hold、显示设置、未提交草稿、设备独立编辑不使流程凭据失效。
- 新建 / 打开构造新的 `WorkspaceState`；事务 Clone 保留身份。独立流程拥有者使用新 WorkspaceState，或在 clone 上显式 `reset_flow_run_session`；该接口只重置流程接收与结果，不承担设备 fork 或 Shell 生命周期。正式输入必须通过已有事务入口更新；公开模型字段的初始化写入不作为运行中编辑接口。
- [Studio 执行边界](../../../apps/radishflow-studio/src/solver_bridge/execution.rs)提供 `prepare_workspace_solve → PreparedWorkspaceSolve::execute_* → complete_workspace_solve`。准备复制选包 / 快照参数，执行只读取冻结输入、不持有 AppState；加载仍按 AF2b 开始时资格校验，完成保留实际包描述、版本、指纹和执行成功 / 失败事实。输入副本执行后释放，不留完整任务历史。
- 完成接收以一个独占 AppState 借用串行核验并发布；成功、求解错误、包错误和日志均经过同一门禁。通过后用捕获 revision 写快照 / 诊断；拒绝返回 `DifferentSession / InputChanged / AcceptanceRevoked / Superseded / AlreadySettled`，不清较新 pending，不改变当前 Error / Hold、快照或日志。
- prepared / completed / receipt 不实现 Clone；凭据第一次处置后即消费，重复传入同一完成对象返回 `AlreadySettled`，不靠全局可变状态或无限 settled 集合。普通事务 Clone 不共享可变消费标记；只有正式拥有者登记和接收任务，不能把 Clone 当作第二个调度宿主。
- 原 `StudioSolveRequest` 保留同步调用参数含义；旧同步函数内部串行调用同一三段链路，Facade 的草稿 / 选包 / readiness / entitlement 检查和 Started 最终回执保持。缓存准备错误保留原诊断与状态语义，原 headless / FFI JSON 和工程格式未扩展。
- 本层 pending 是结果资格，允许测试顺序提交 A / B 后交错交付；不代表已有队列或并发许可。worker 单槽、Ready 握手、协作取消、取消位、进度、摘要、缓存根租用与退出回收归 AF3b / AF3c，AF3a 未新增线程或 GUI 操作。

### AF3b 已实现契约（2026-10-24）

- [WorkspaceRunOwner](../../../apps/radishflow-studio/src/workspace_run_owner/mod.rs)是非 Clone 的应用拥有者，提供 `submit_cached / cancel / poll / detach / begin_shutdown / try_close`。提交返回 `Accepted / AlreadyRunning / Busy / Blocked / Skipped`，不把接受请求写成已完成；与同步 Facade 共用只读草稿 / skip / 选包 / readiness 检查，槽位检查后才登记 AF3a 凭据。
- 每个拥有者最多一个实际 worker，无等待队列。相同会话 / 输入 / epoch / 选包的未取消运行返回 AlreadyRunning，其余为 Busy，不改领域资格。终态载荷已经到达但线程未退出仍占槽；数值状态保持在 RunStatus，Preparing / Running / CancelRequested 与进度由独立运行投影提供。
- 凭据留在拥有者，worker 只持有不可变 Flowsheet、快照参数、缓存 index 副本与路径租用。[缓存根](../../../apps/radishflow-studio/src/property_package_cache_root.rs)沿用原临时目录创建 / 清理逻辑，增加最后租用释放后删除；加载闭包结束即释放 worker 租用，已装配上下文不再触盘。持久根的外部生命周期仍由其拥有者负责。
- 开始加载前检查取消，加载仍按 AF2b 当时有效期校验；完成加载后经 Ready 通道等拥有者许可。拥有者用提交时身份、当前 pending 和授权事实重验；有效期在许可前检查，许可后仅时间流逝不撤回结果，显式登出、主体 / 授权模式变化或移除允许包会拒收。无授权快照的既有本地运行路径保留，不引入网络刷新或新公共 API 授权政策。
- [受控顺序求解](../../../crates/rf-solver/src/control.rs)使用类型化进度和 `ControlFlow`，在图检查前后、每单元完整执行前后及最终完成前观察取消。旧 `solve` 使用同一循环的无取消适配；单次文件 / 单元 / Flash 调用内部不强杀，返回错误仍保留真实数值事实，取消成功不返回部分快照。
- 用户取消先按精确 RunId 撤销资格，再发信号并将同一工作区置 Hold；回执区分 Requested / AlreadyRequested / AlreadyFinished / UnknownRun。编辑 / 清结果仍立即失效资格；拥有者下一次 poll 发现变更时发内部取消并保留模式。取消先被处理，即使 worker 实际成功也丢弃结果；完成先被接收则后续取消不撤回。迟到错误 / panic 不污染新输入。
- 一个可合并进度槽、一个终态通道和最多 8 条紧凑摘要；进度序号 / RunId 过滤旧、重复、乱序更新，worker 不等逐条进度被消费。摘要保留实际 Succeeded / Failed / Cancelled / InfrastructureFailed、接收处置、取消原因和诊断码，不保留完整输入 / 结果历史。
- 启动失败、可展开 panic、Ready / start 断开和无终态退出均为明确基础设施失败。当前缓存准备失败在后台接收中直接发布 Error / Hold，同步适配仍交给原 Facade；两条路径不重复接收。只有 `JoinHandle::is_finished` 后才 join，取消请求或收到载荷都不等于线程已停止。
- 拥有者须由调用方跨工程宿主替换保留；`begin_shutdown` 禁止新提交并发取消，继续 poll 至 idle，再以 `try_close` 消费。未排空时 `try_close` 返回原拥有者，不阻塞等待或丢弃句柄；取消退出仅恢复接单，不撤销已发送的取消。不得直接 Drop 活动拥有者。永久阻塞调用仍无法保证优雅退出。
- AF3b 阶段交付后台接口与真实 worker 验证，GUI / 设备接线由后续 AF3c 完成。headless / FFI 同步兼容入口保持；AF1 独立跟踪。

### AF3c 已实现契约（2026-10-24）

- [StudioRunOwner](../../../apps/radishflow-studio/src/studio_run_owner/mod.rs) 在 UI 线程拥有流程与设备调度，跨页面、工程宿主替换及只读覆盖保留；共享一个实际 worker 配额和最多 8 条紧凑历史。设备 worker 只接冻结 `CalculationRequest`，开始前及接收时重验专用资格；取消按 `CalculationId` 撤销，不清其他行或较新请求。
- 桌面 Shell 安装后台会话后，Facade 的流程命令和设备 Calculate / CalculateRating 走提交路径；所有既有草稿、输入、选包与授权检查保留。未安装后台会话的 Rust 兼容入口、headless 与 FFI 仍同步，旧最终响应与磁盘只读契约保持。
- 每帧在 Home、只读覆盖及关闭等待之前轮询；仅回收已结束线程。Active 的正式输入变化先使旧任务失效并请求取消，排空后对最新输入最多自动尝试一次，重新检查全部前置条件；清结果、只读返回和取消退出不暗中重启，Busy 无排队。
- 导航下同源摘要显示目标、阶段、已用时与精确取消；“查看运行”展开既有运行日志，不切页、不改设备选择或草稿。日志区分实际执行结果、接收 / 丢弃、取消原因与诊断码。取消后 5 秒只追加等待说明；F5 提交、Shift+F5 Resume 和普通 Escape 原语义保留，命令面板捕获的旧任务取消不能作用于新任务。
- 设备执行状态与领域结果资格分列。Sizing 保留旧候选、Rating 旧结果转 Stale 的差异保持；在途源对象暂禁采纳 / 单行导出，捕获及最终写出再次重验。其他有效行与不完整矩阵输出继续允许。
- 新工程读取成功后才 detach 并替换宿主；失败保留任务。只读覆盖同样 detach，返回不复活旧任务。最后窗口先走脏改 / 草稿 / 保存保护，再禁止新提交、取消并等待回收；只关一个观察窗口不取消。等待时返回编辑保留输入和草稿、撤销放弃意图，已发出的计算取消不可撤回；下次关闭重新确认。

### 执行状态、取消与竞争

任务生命周期与结果资格分开：`Preparing → Running → Finished`，任一未终结阶段可进入 `CancelRequested`，只有执行者返回并收尾后才是 `Cancelled / Failed / Succeeded`。`Preparing` 包括资源加载 / Ready 握手；不增加没有真实队列的 Queued 状态。每条任务另记录结果处置 `Applied / Discarded(reason)`，执行成功但接收失效仍如实记录执行事实，不伪称发生数值失败。

| 场景 | 建议确定的行为 |
| --- | --- |
| 普通 Run / F5 在已有任务运行时重复触发 | 同目标、同正式输入 / 选包且未取消的任务返回 AlreadyRunning，其余返回 Busy；不排队、不隐式取消或替换、不分配新 RunId。自动触发只保留现有最新输入的 pending 意图，不保存一串冻结请求 |
| 用户取消当前 RunId | 原子记录取消请求并撤销其发布资格，然后发取消信号；回执为 Requested / AlreadyRequested / AlreadyFinished / UnknownRun。UI 不因信号写入成功显示“已停止” |
| 取消与完成竞争 | 以应用所有者串行处理顺序为线性化点，不按墙钟猜测。完成已接收后取消返回 AlreadyFinished，不能撤回已发布结果；取消先被接受，则即使 worker 已算完而回包未接收，也丢弃该回包。终态记录保留实际执行 outcome，显示“已取消；计算已返回，结果未接收”等解释 |
| 协作停止点 | 开始加载前后、Ready 等待、图检查前后、每单元开始 / 完整结束和最终发布前检查。单次文件读、`UnitOperation::run` 内部、Flash 内部当前不可中断；最长响应取决于该调用返回，不许强杀线程或伪造秒级停止保证 |
| 数值失败与取消同时发生 | worker 如实返回原错误；若取消已被应用接受，则只作该任务终止证据，不发布为当前工程失败。未取消且资格通过的错误继续使用既有 solver / workspace 代码、定位和恢复动作 |
| 运行中正式编辑、Undo / Redo、改包 / 组分或清结果 | 立即撤销接收资格，向旧 worker 请求协作取消；输入事务可继续，原任务仅待收尾。普通草稿编辑不暗提交、不改变已捕获输入；未提交草稿仍按原规则阻挡新 Run，结果明确对应正式输入 |
| Active / Hold / Resume | Hold 只阻挡后续自动运行，不暂停或自动取消在途任务；Resume 仍是激活并按当前输入请求新运行。用户显式取消流程任务时同时转 Hold，防止自动马上重算；因输入变化产生的内部取消保留模式，旧任务退出后 Active 可对最新输入重新做完整 preflight，至多一次 |
| 旧结果与取消 | 取消不写入部分流股为完整快照；先前已接收结果仅按原输入资格保留为“上次结果”，不能冒充被取消运行的产物。已因修改失效的结果不复活；未取消的当前求解失败仍沿用旧清当前结果 / Hold 行为 |
| 设备控制 | 请求只针对精确 RunId / task / case；不得清整个设备缓存当作取消一行。添加按 CalculationId 撤销待接收项的领域入口，不能删掉后来请求；保留 Sizing 与 Rating 原有旧结果资格差异 |

第一批每个 Studio 应用运行所有者最多一个活动 worker，流程和设备共用此配额；切换工程后旧 worker 未退出仍占槽位。该限制使资源有界，用户能继续编辑 / 切页但需要等收尾后再计算；多任务队列、并行矩阵、任务优先级和自动抢占以后按真实需求设计。headless / FFI 的各独立进程 / Engine 不共用这个 GUI 配额。

### 切换、关闭与故障收尾

1. 新建 / 打开先完成既有离开确认和新工程读取；取消或读取失败保持旧会话与任务，不提前撤销。真正提交切换时先撤销旧接收资格、请求取消、移交 worker 与资源租用，再替换宿主。打开同一工程也产生新会话；只读覆盖虽然保留旧 host，也进入同一 detach 路径，返回原工作区时不得复活旧任务。
2. 普通页面切换 Home / Property / Workbench / Equipment 不等于关闭文档，不撤销资格。设备切行也不重定向任务目标；结果只回原 case。另存为只改变路径 / 保存基线，不取消相同内存输入的运行。
3. 同一文档仍有其他观察窗口时，仅关闭一个逻辑窗口不取消共享任务；最后拥有窗口关闭或整个应用退出才排空所属任务，window / timer ID 不参与结果资格。窗口关闭仍先执行脏改、草稿、格式与保存保护；关闭确认取消不终止任务，保存失败不销毁工作区。确认离开后先禁止新提交、取消剩余任务、在事件循环中等待收尾，句柄确认结束后才 join / 最终退出；不在 UI 回调或 Drop 中阻塞 join，也不直接丢 JoinHandle 让线程失管。
4. 等待取消超过提示阈值只提示“等待当前调用返回”，不自动声称终止或强杀；允许取消退出、返回编辑界面，但已接受的运行取消不可撤回，必须新 Run 才能再算。不可中断永久阻塞的调用无法保证优雅退出，首批不引入外部黑箱；强制进程终止和隔离子进程属于后续范围。
5. worker 启动失败、panic、完成通道断开和 Ready 握手失败均有应用基础设施终态；撤销本次待接收凭据、释放槽位与资源，原输入 / 文件保持。线程顶层只捕获可展开 panic 并标记失败，不吞掉或回退到成功；abort / 进程崩溃不承诺恢复。失效任务的 panic 只归旧任务记录，不污染新工程。

AF1 的原生文件面板仍可能占用 UI 事件循环；后台数值不依赖 UI 轮询才能响应取消位，但 UI 回执 / Ready 握手可能等待面板返回。AF3 不替代 AF1 定因或修复，也不以后台线程宣称原生面板已不阻塞。

### 进度、容量与兼容

- 第一批只报告 Preparing、计划已知后的总单元数、当前单元、已完成数与终止；设备只报告准备 / 计算 / 终止，不制造算法内部百分比或 ETA。先保留阶段与失败位置，不交付完整步骤中间值、循环 trace 或 N4 观察协议。
- 进度带 RunId 和运行内递增序号，陈旧 / 重复更新不得回退当前状态。建议独立“最新进度槽”与一次性完成通道：进度可合并，终态不可因进度拥塞丢弃，worker 不等待 UI 消费每条进度；UI 每次 drain 有上限。完整开始 / 完成步骤审阅仍归 N2，合并后明确不承诺逐事件无损。
- 建议首批容量：一个活动请求 / 一个最新进度 / 一个终态载荷；保留最近 8 次紧凑任务摘要，不保留每次完整输入副本或部分求解快照。现有结果历史仍使用自己的上限。任务摘要驱逐不得影响活动任务资格或复用 RunId；事件 / 文案大小沿既有字段约束，新增大载荷不纳入。
- 新 GUI 提交回执明确 Accepted / AlreadyRunning / Busy / Blocked / Skipped，不能沿用旧 Started 冒充已完成；同步 Rust 兼容入口串行执行同一 prepare / execute / complete，开始校验在本调用内完成，不等待 GUI Ready 握手；旧调用完成后才返回。headless v1 / v2 最终 JSON、SI、错误、磁盘只读和 FFI C ABI / 对象释放保持；本批不发布查询 / 控制远程协议。
- RunStatus 的旧数值状态不用于塞入 CancelRequested 等控制状态；运行栏单独投影任务状态，最终结果格式不加字段。任务记录只在内存，不进项目格式、sidecar、Undo 或保存脏状态；AF5 再决定正式结果来源与导出。
- 新取消入口、Busy、等待收尾与退出提示需最小局部 UI / 双语 / 键盘 / 读屏设计，复用已有运行区域，不重绘完整工作台；按现有 UI 门禁先评审状态组合与操作路径。后台执行测试不替代这项产品验收。

### 已确认实施切片

| 切片 | 可独立评审交付 | 退出条件 |
| --- | --- | --- |
| AF3a 身份与接收 | 流程会话 / 输入 checkpoint / RunId，冻结工程输入，prepare / execute / complete；同步适配继续可用 | 不起线程也能确定性证明重开、分叉、清结果、乱序成功 / 失败和重复接收全部隔离；旧数值 / 诊断 / 协议回归 |
| AF3b 后台流程与协作取消 | 应用所有者 / 资源租用、worker、Ready 握手、单槽策略、solver 安全点、进度 / 完成通道和故障回收 | 可控执行器竞争测试及真实顺序求解通过；后台提交方不执行加载 / 求解，仅回收已退出句柄，无无限 worker / 队列；GUI 接线归 AF3c |
| AF3c 设备接入与产品闭环 | 拆开既有设备同步调用，沿原领域 key 完成接收；连接 GUI 取消、文档切换 / 只读覆盖 / 关闭与自动触发策略 | 保留 Sizing / Rating / 矩阵 / 草稿 / 采纳 / 导出的领域资格；局部 UI 评审、macOS 代表实窗和正式基线通过 |

本方案已确认的后续行为变化为：GUI Run 改为提交回执、单 worker / Busy 策略、运行中修改使旧任务取消且不可发布、显式取消流程时转 Hold、关闭确认后的异步收尾等待，以及上述开始握手 / 授权撤销边界。字段与模块按此职责实施，不将架构方案视为已经通过运行态验收。AF3a—c 已接通，分层验证与保留项见专项验收；纯状态测试不替代原生运行态证据。

验收案例与通过门槛唯一维护在 [AF3 验收边界](scenarios-and-acceptance.md#af3-专项验收边界)，关联 MSG-02 / 06 / 07 / 10 / 12 / 13 / 20 / 21 / 23 / 25 / 26 的必要子集；完整暂停、单步、trace、条件和持久化要求继续待实施。


### 2026-10-31 有限物料任务接入

已确认的[稳态方程任务](../modeling/solver-strategies-and-assembly.md#稳态方程任务实施方案2026-10-31)进入既有 StudioRunOwner 单 worker 配额，与流程 / 设备互斥。配置和输入资格独立冻结；修改配置、工程输入、切换 / 关闭与取消立即撤销发布权，等待线程真实退出 / join 后释放配额。实际完成与是否接收分开记录，旧任务不能撤销新任务。

物料直接求解以完整线性行及 SI 复核为安全点，使用独立 MaterialBalanceProgress，不伪造单元步骤或非线性迭代；旧流程仍按单元边界控制。设备与物料共用数值线程生命周期基础，设备方法的不可中断粒度保持。物料结果与设置只在会话内，未增加暂停、持久化任务或动态时钟；CLI 使用同步正式动作，桌面使用后台正式动作。
