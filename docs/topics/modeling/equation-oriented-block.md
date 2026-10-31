# QS3：多单元联立方程、规格反算与正式接入

更新时间：2026-10-31

用途：定义 QS3 首批案例、模型方程能力、结构分析、数值方法、工程与消费者契约，以及具体批准项和阶段验收。
读者：维护 unitops、solver、store、运行与结果、Studio 和自动化入口的协作者。
不包含：热 / 相平衡 EO、通用非线性求解器、优化、多变量设计规格、动态 / 瞬态、混合分区、插件 ABI 或 COM 自动化。

**状态：QS3-D01—D08 已批准并完成本机实现与阶段验证。** 模型 / 结构、正向 / 单比例反算、工程 v4、后台运行、显式采纳及 GUI / CLI v6 已接通；GUI 六类七张画板先获认可再接线。正式门禁 2,031 项通过，macOS 代表实窗和历史 reader / writer 验证通过。Windows / Linux、I5、原生 OS 缩放与 PME 缺口保持；不表示通用非线性或热 / 相平衡 EO 已实现。

父专题：[多策略求解](solver-strategies-and-assembly.md)。既有基础：[QS0 有限物料](solver-strategies-and-assembly.md#qs0-首片实施2026-10-31)、[QS2 核心](simultaneous-modular-block.md)、[QS2-C 正式消费者](simultaneous-project-and-consumers.md)。本批反算也是 [DS0 / DS1](../flowsheet-modeling-and-solve.md#设计规格求解规划尚未实现) 的限定场景实现，不代表全部 Design Specification 已实现。

## 实施进展（2026-10-31）

- `rf-unitops::component_equations` 声明 Feed / Mixer / Splitter 的类型化物料行、端口依赖、残差和解析偏导；Feed 有效边界复用原局部工厂解析，目录不对其他模型宣称方程能力。
- `rf-solver::equation_oriented` 已实现全端口装配、稳定身份、双尺度、确定性最大匹配与交替区域；结构查询不调用物性。规格未配对能形成欠 / 过规格报告，结构通过不代替数值检查。
- 正向直接 LU 与反算条件 LU / 二分已落地；共享内部 `dense_linear` 保持 QS2 算法、门槛、外部错误及计数。每次条件评价和最终完整 Jacobian 分别留存证据。
- 最终用真实目录工厂逐单元复核，保留实际消费 / 生产者、候选与最终残差、零流组成锚点、目标及总体衡算。候选只在隔离单元副本使用，不修改源工程。
- 核心定向测试覆盖 E1 / E2 独立算术、导数扰动、QS2 对比、端点 / 未括区 / 预算、结构与数值相关、奇异区间、尺度 / 容量 / 绑定拒绝、零流串联、局部故障及各阶段取消。`rf-solver` 141 项、`rf-unitops` 83 项通过；跨层正式基线为 2,031 项。
- 工程 v4 新方法 / capability、严格规格与完整尺度映射、ID 保留、同版本保存升级与只读保护已实现；历史 `f2ba51c1` 独立构建实测读取新工程为只读、拒绝序列化及覆盖，原字节保持。
- 复用单 worker、Ready 双资格、取消 / join；结构未就绪在预检 Blocked，worker 独立复验。成功 / 失败 / 取消证据分开，实际包与核心来源不伪造。
- 正式采纳从 Current 结果取得候选，预览 / 提交复验输入、计划、执行及访问资格；单次输入事务、计划保留、Stale、不自动运行、no-op 和 Undo 后旧资格不恢复均有自动验证。
- GUI 已接通规格、结构、尺度、诊断、真实结果及显式采纳；自动帧覆盖中英 400 px / 20 px 字体、未完成草稿、默认取消焦点、失败无 Current 数值、保存升级与重开。macOS 实窗覆盖反算 Run、采纳、保存、重开 Missing 及显式 rerun。
- CLI v6 公开查询 / 运行 / 建图 / 采纳后切正向重算模板已加入；真实进程验证整个临时目录、资源和工程字节不变，旧协议及外部执行引用拒绝。接口和运行方式见 [CLI 参考](../../reference/headless-cli.md#qs3-cli-v62026-10-31已实现)。
- 阶段验证与未覆盖原生平台记录见 [W44](../../devlogs/2026-10/2026-W44.md)。

## 实施起点与职责分配

| 已核对入口 | 实施前事实 | QS3 增量与复用边界 |
| --- | --- | --- |
| [QS0 问题](../../../crates/rf-solver/src/component_balance/mod.rs) | `MixerComponentBalanceProblem` 是单 Mixer 专用装配，直接前代求解；有领域变量 / 方程与 Jacobian | 保留原入口 / 结果语义；新建多单元方程装配，不能只给现有模板改名 |
| [物料关系](../../../crates/rf-unitops/src/component_balance.rs)、[模块响应](../../../crates/rf-unitops/src/component_response.rs) | 共享混合 / 非选择性分流关系；QS2 导数针对固定参数下的输入输出响应 | 新增端口变量、单元残差及解析偏导能力；分流比例释放后增加参数导数，不谎称已有 QS2 响应包含它 |
| [QS2 装配](../../../crates/rf-solver/src/simultaneous/assembly.rs)、[稠密后端](../../../crates/rf-solver/src/simultaneous/dense.rs) | 八个内部连接组分未知量，LU 与错误 / 成本 / 进度直接耦合 Modular 类型 | QS3 展开全部单元端口与连接方程；需要时按实际两位消费者提取内部稠密数值操作，保持 QS2 门槛、顺序和证据 |
| [正式计划](../../../crates/rf-ui/src/flow_plan.rs)、[存储 DTO](../../../crates/rf-store/src/project_v4/plan.rs) | 三种方法，shape 与图就绪分层、独立计划历史、ID 保留 | 新增独立 EO 配置；规格与数值配置属于计划，Jacobian / 匹配 / 执行索引不进工程对象 |
| [运行资格](../../../crates/rf-ui/src/flow_run.rs)、[运行 bridge](../../../apps/radishflow-studio/src/solver_bridge/execution.rs) | 已有冻结输入＋计划、Ready 复验、单 worker、取消 / join | 增加 EO 载荷与检查点，复用所有权和双资格，不另起执行服务 |
| [结果](../../../crates/rf-ui/src/flow_results.rs)、[计划动作](../../../apps/radishflow-studio/src/flow_plan_actions.rs) | 当前结果没有 EO / 释放参数含义，也没有反算候选采纳事务 | 新增 EO 结果、规格证据与正式采纳动作；不能把候选直接写回参数或伪装旧 steps |
| [v4 reader](../../../crates/rf-store/src/project_v4/mod.rs)、[CLI v5](../../../apps/radishflow-studio/src/headless/v5/mod.rs) | parser 最新支持 Simultaneous；v5 严格词汇已包含三种方法 | 固定旧消费者能力范围；提案为工程 v4 新能力＋独立 CLI v6，旧协议不随共享 parser 扩展 |

## D01：物理范围、正向与反算案例

### 能力范围

首批支持固定同一 T/P、无反应的 Feed、双入口 Mixer 和非选择性 Splitter 的组分物料方程。正向配置中全部分流比例固定；反算配置最多释放一个 Splitter 的 `outlet_a_split_fraction`，增加一个终端产品总摩尔流量规格。其余模型 / 参数 / 热力学规格明确不支持，不自动换策略。

正向装配针对当前连接图中的上述单元，支持无环、单回流和多回流；全图形成一个有界 EO 问题，不按物流 SCC 冒充方程分块。每条流股有唯一生产者、至多一个消费者；每个规范入口必须连接，产品出口允许没有消费者；保留现有端口 / 连接合法性。至少有一个正总量 Feed；无法从 Feed 到达的单元、无锚定的循环及未绑定出口分别诊断。最多 128 个未知量、128 条活动方程，计入所有端口和释放参数；容量不足不截断求解。

T/P、物性包身份和组分仍从冻结工程 / 计划取得。数学残差不需要热物性或相平衡求值；成功发布前仍用真实单元与选定物性服务物化和独立复核。零流量不构成组成方程；其来源规则见 D04。不得把本批结果解释为能量、焓或相平衡方程已经闭合。

计划固定 T/P 是本任务的能力边界，不覆盖工程 Feed / 单元的物理设置；有效边界或实际局部输出与任务 T/P 不一致时拒绝。比例释放只作用于明确选中的字段，其余输入不因装配方便而静默更改。

### 正向主例 E1

沿用[公开交叉回流工程](../../../examples/flowsheets/fixed-tp-cross-recycle-binary-hydrocarbon.rfproj.json)：两股新鲜进料 FA、FB 分别为 2、3 mol/s；按 `(methane, ethane)` 顺序，组分流量为 `(8/5,2/5)`、`(3/5,12/5)`。固定 300 K、650000 Pa，两个分流比例 `alpha=3/5`、`beta=1/4`。

连接为 FA→MA.a、FB→MB.a、MA.out→SA.in、MB.out→SB.in、SA.a→MB.b、SB.a→MA.b；SA.b、SB.b 为产品。公式中的 MA / MB / SA / SB 对应工程 `mixer-1` / `mixer-2` / `splitter-1` / `splitter-2`，不能将别名作为持久身份。

采用“端口变量＋显式连接方程”：2 个 Feed 出口＋2×3 个 Mixer 端口＋2×3 个 Splitter 端口，共 14 个端口、28 个组分流量未知量。方程为 Feed 边界 4 条、Mixer 衡算 4 条、Splitter 分流 8 条、六条连接的组分等式 12 条，共 28 条；不再添加重复的总流量或组成归一约束。

### 规格反算主例 E2

保持 E1 的进料、T/P 和 `beta=1/4`，释放 SA 的分流比例 alpha，指定 SA.b 产品总量 `q=3/2 mol/s`，搜索区间 `[0,19/20]`。工程原 alpha=3/5 作为输入基线保留，但不再进入本次方程的固定参数集合。增加一个无量纲未知量 alpha 和一条产品总量方程，成为 29 个未知量／29 条方程。

对每个组分，独立消元有：

```text
MA = (FA + beta*FB) / (1-alpha*beta)
MB = (FB + alpha*FA) / (1-alpha*beta)
SA.a = alpha*MA       SA.b = (1-alpha)*MA
SB.a = beta*MB        SB.b = (1-beta)*MB
C = FA_total + beta*FB_total = 11/4 mol/s
q(alpha) = C*(1-alpha)/(1-alpha*beta)
alpha = (C-q)/(C-beta*q) = 10/19
```

| 流股 | E1 methane / ethane，mol/s | E2 methane / ethane，mol/s |
| --- | --- | --- |
| MA.out | 35/17；20/17 | 133/66；38/33 |
| MB.out | 156/85；264/85 | 274/165；496/165 |
| SA.a | 21/17；12/17 | 35/33；20/33 |
| SB.a | 39/85；66/85 | 137/330；124/165 |
| SA.b 产品 | 14/17；8/17，总量 22/17 | 21/22；6/11，总量 3/2 |
| SB.b 产品 | 117/85；198/85，总量 63/17 | 137/110；124/55，总量 7/2 |

E2 的 `dq/dalpha=-C*(1-beta)/(1-alpha*beta)^2`；解处为 `-361/132 mol/s`，在指定区间严格为负，且分母非零。因此该**案例**具有连续、唯一的可行根。区间两端的 q 分别为 `11/4`、`11/61 mol/s`；q=3 mol/s 在本案例区间不可达。这个解析证明用于独立验收，不把公式硬编码为通用产品求解路径，也不把对本例的唯一性推广到所有网络。

## D02：模型方程、身份与结构诊断

### 方程能力

新增 `rf-unitops` 固定温压组分方程能力，与局部 run / 模块响应分别声明；用具体类型及类型化行角色承载，不引入字符串表达式语言、全模式大 trait 或新 crate。目录只暴露真实支持；模型拥有下列物理关系、局部偏导和定义域，solver 拥有全局索引、连接、规格与矩阵。

| 方程角色 | 残差 | 非零偏导 |
| --- | --- | --- |
| FeedBoundary(c) | `n_out,c - n_feed,c` | outlet 为 1；Feed 的有效源量沿现有覆盖 / 组成规则解析 |
| MixerBalance(c) | `n_out,c - n_a,c - n_b,c` | outlet 为 1，两入口为 -1 |
| SplitA(c) | `n_a,c - alpha*n_in,c` | outlet 为 1，inlet 为 -alpha；释放时对 alpha 为 -n_in,c |
| SplitB(c) | `n_b,c - (1-alpha)*n_in,c` | outlet 为 1，inlet 为 alpha-1；释放时对 alpha 为 n_in,c |
| Connection(c) | `n_sink,c - n_source,c` | sink 为 1，source 为 -1 |
| ProductTotalTarget | `sum_c(n_product,c) - q_target` | 所有产品组分变量为 1 |

port 组分变量使用 `(UnitId, canonical port, StreamId, ComponentId)`；参数变量使用 `(UnitId, typed field)`。单元方程以 unit＋行角色＋component 标识，连接方程含 stream＋两端 unit / port＋component，规格方程含目标类型＋产品 stream / source。索引只属于本次装配。固定比例 0 / 1 的恒零项可在结构中消去；释放比例的结构使用允许区间内可能非零的依赖，不能因某个试点的数值零删掉关联。

变量界限、残差量纲、固定量与释放量在模型 / 计划中可查询。只有所选比例被释放；其原字段可以未填写，检查器按本次任务标为“由规格求解”，不填入默认固定值。其他比例仍要求合法固定输入。未使用的热规格、同一量既固定又释放、重复目标 / 身份明确拒绝。

### 结构与数值分层

1. 先做连接 / 版本 / 能力与规格检查，列出精确分析范围。一个释放量必须对应一个产品目标；只有一边时可保存合法配置，但 readiness 报欠 / 过规格，不运行。
2. 从真实方程关联建立二分图，进行确定性最大匹配；报告变量数、方程数、匹配数、未匹配变量 / 方程及交替路径关联范围。欠定与过定部分可以共存，不能仅用全局相减数给出“可解”。
3. 完全匹配只证明结构层面的候选可解性；系数相关、数值奇异、尺度和条件问题在数值阶段判断。稠密 LU 的阈值拒绝属于数值证据，不称作精确符号秩证明。过规格报告也不自动等于矛盾，冗余与不相容分别需要进一步证据。
4. 数值错误映射回相关变量 / 方程与对象；匹配的某一个未匹配节点不是唯一应删除的规格，不自动给出破坏性“修复”。未覆盖模型仍是 Unsupported / NotAnalyzed。

本批覆盖 [A4](../modeling-assistance-and-specifications.md) 的有限方程范围；不把旧 readiness 改成全模型自由度分析。结构报告只读、不加载物性、不改变文档；按输入＋计划资格缓存 / 失效。

## D03：数值方法、尺度与验收

### 按方程结构选择已声明方法

**正向：** 固定全部比例后，完整端口方程是仿射系统，组装 SI 的 `A*n=b`，在双尺度系统中直接 LU 求解，不制造非线性迭代记录。

**单比例反算：** 完整 EO 问题含双线性项。给定 alpha 后，物料子系统对所有端口流量仍是线性的，求 `A(alpha)*n=b`，再求标量规格残差 `g(alpha)=sum(product)-q_target`。采用明确的“条件线性消元＋二分区间求根”，每次评价从原方程重建所需系数，不调用 QS1 回流或 QS2 模块求解作为隐藏内层。数值消元不改变完整 29 变量 / 方程报告，也不将该算法描述为通用 Newton。

区间必须满足 `0 <= lower < upper <= 1`。先依次评价两端；端点达到残差门槛时可作为候选，仍须完整 Jacobian 与最终独立复核。否则要求端点异号，再逐次二分；不扩区间、不夹紧候选、不把失败样本替换为旧值。同号返回 `TargetNotBracketed`，仅表示当前方法未建立括区，不能普遍声称无解。端点或中点的线性系统奇异、病态或非有限立即返回该阶段失败及已有证据；不跨过未定义区继续假设连续。最终必须同时满足区间宽度 `<=1e-12` 和目标残差门槛；浮点中点不再缩小时为 Stagnated。二分预算默认 64，允许 1—128，端点评价、最终复核与失败尝试另外计数。

初始化只冻结 Feed 边界、固定参数、尺度及反算区间；本方法没有自由的流量初值或 Newton 初值选项，不保存无效旋钮。前述案例解析连续性独立证明；一般网络的端点异号只是方法进入证据，最终残差 / 物理复核仍不可省略，方法不承诺全局找根或任意初值能力。

### 尺度、导数与线性边界

默认流量变量 / 物料残差 / 目标残差尺度为本次所有正 Feed 的总量 `F_ref`（mol/s），比例变量尺度为 1；F_ref 非正、溢出或非有限明确拒绝。也允许完整显式变量 / 残差尺度映射，按稳定领域身份保存，量纲与绑定一致且正有限；缺项 / 多项 / 身份失效不自动补齐。形状合法的失效配置可保存并继续修复。

Feed 的负流量先按输入非法拒绝，不通过从总量求和中排除它来继续执行；零流 Feed 可以保留，但不能单独提供正 F_ref。

缩放为 `z_j=x_j/s_u,j`、`r_hat_i=r_i/s_r,i`、`J_hat_ij=J_ij*s_u,j/s_r,i`。释放参数列的 SI 导数有 mol/s 量纲，缩放后无量纲，不能套用 QS2“所有 SI 系数都无量纲”的假设。解析 Jacobian 同时用于结构边界、扰动验证与最终完整系统条件检查；求根过程中另记录条件线性子系统，二者维数 / 含义不同。

建议共用现有算法的内部 dense factor / solve / condition 操作，领域层分别映射错误、取消阶段和计数。保持 QS2 `64*n*EPSILON*||A||inf` 主元门槛、`rcond_indicator>1e-12`、原矩阵后向误差 `<=64*n*EPSILON`；不得改变 QS2 外部证据来适配 QS3。QS3 对每个条件线性子问题及最终完整 Jacobian 分别检查，后者可识别目标对释放量无响应等非孤立解。新模块不导出 Modular 类型给 EO，现有 QS0 后端不随之改写。

正式候选要求全部方程的 SI 残差绝对值 `<=1e-10 mol/s` 且缩放最大范数 `<=1e-10`；变量均有限、组分流量非负、比例在声明闭区间。所有变量为物料流量或比例，所有残差为物料流量；未来新增量纲必须另定契约。负值不以绝对值或夹零修补；规范 -0 可按 SI 零处理。固定温度差 `<=1e-9 K`、压力差 `<=1e-6 Pa`，总体组分 / 总量按现有固定温压物料门槛再复核。容差 version 1 固定，不开放一键放宽。

### 方案独立核对

临时 Python `Fraction` 驱动独立按上述端口关系构造：E1 精确秩 28，E2 解处完整 Jacobian 秩 29；释放 alpha 不加目标时为 28 行 / 29 列、秩 28；保持 alpha=3/5 又加 q=3/2 时系数秩 28、增广秩 29。双比例 1 的正进料正向系统系数秩 26、增广秩 27。它们是解析 / 精确算术基准，不是产品结构分析已实现的证据。

独立中心扰动对反算解析 Jacobian 最大差约 `1.62e-11`；临时二分试算 q=3/2 共 43 次条件线性评价（含两端），alpha 误差小于 `1e-12`、目标残差约 `-2.59e-13 mol/s`；两端目标可接受，q=3 返回未括区。先前的单纯边界截步 Newton 在端点出现停滞，故本批不采用该方案。临时驱动未调用产品 solver、未做正式条件估计 / 物性 / worker 验证；实现时须把独立期望与故障案例归入正式测试，不能把试验次数冻结成产品性能承诺。路径和命令证据归 W44。

方法参考仅用于核对语义：[IDAES 2.13 结构与数值诊断分层](https://idaes-pse.readthedocs.io/en/2.13.0/explanations/model_diagnostics/index.html)、[SciPy 二分求根的连续性及端点条件](https://docs.scipy.org/doc/scipy/reference/generated/scipy.optimize.bisect.html)。[SUNDIALS 7.7 KINSOL](https://sundials.readthedocs.io/en/v7.7.0/kinsol/Mathematics_link.html)用于审查 Newton、缩放和线搜索边界；本批不引入这些库、不迁移其实现代码。

## D04：真实结果、独立复核与候选采纳

### 独立最终复核与发布

数学解通过后，在一次执行的隔离副本中将求得的比例提供给该单元；不改变活动工程。用已求端口量构建固定 T/P 输入假设，经现有目录 / 工厂实际运行全部单元，记录真正 consumed / produced 流股、物性 / flash 调用及各阶段成本。Feed、Mixer、Splitter 的局部能力与输入校验仍生效；不能以计算方程残差代替该次局部复核。

正流量组成由求得组分流量归一。零流 Feed 使用冻结的规范组成；零流 Splitter 出口沿其真实入口继承组成，入口若也为零则沿上游非选择性链追溯到有依据的组成；正流输入、Feed 边界或已经物化的生产者可提供依据。零流 Mixer 仍按现有正总流量要求拒绝。无组成锚点的零流循环明确失败，不取任意第一股流或旧模板组成兜底；来源映射进入证据。局部计算没有产出的相 / 焓保持缺失，不填零或复制无来源旧结果。

复核全部生产者输出与 EO 出口变量、实际消费者与 EO 入口变量、连接两端、目标残差，以及各单元 / 全局组分与总量。消费者假设和生产者真实值分别保留，允许容差内差异但不覆写成相等。反算目标用最终生产者实际总量再验。最终完整 Jacobian 与条件数证据使用该次候选点，不能借前次子系统通过代替。

独立 `EquationOriented` 结果变体：包含完整变量 / 方程、结构报告、固定 / 释放量、尺度、矩阵维数 / 条件指标、目标括区与评价轨迹、原始 / 缩放残差、最终局部 I/O、总体衡算、实际方法版本及包来源。公共流股和单元审阅取真实生产者结果；没有虚构的序贯 steps 或 QS2 响应次数。展示明确为“当前 EO 工况”，释放字段的求得值与工程存储值并列，不暗示输入已更新。

计数至少分开装配、匹配、条件线性评价、完整 Jacobian 评价 / 检查、LU 分解、三角求解、目标评价、区间更新、最终局部调用及 thermo / flash；耗时按实际阶段记录。不因 EO 未调用 QS2 而填充伪响应，也不以迭代数较少推导更快。

### 采纳事务

新增限定的“采纳本次分流比例”正式动作，输入是 Current EO 执行 / 候选引用、期望输入＋计划上下文，不能由调用者替换候选数值。预览明确单元 / 字段、原存储值（含未指定）、求得值、来源与影响；提交时再检查只读、草稿、运行状态、两份资格和字段能力。候选超界、错方法、过期、重复或来自另一会话均拒绝。

成功仅通过既有正式物理输入事务更新该比例，一次 revision / 一条输入历史；沿原规则失效流程和依赖该输入的任务。**不同时改计划、不自动切为正向、不自动重跑**；结果成为 Stale，撤销再改回也不恢复旧 Current。当前反算计划仍释放该参数，界面说明此事实；用户另行切换正向计划后才能按固定新比例运行。相同值为 Unchanged，不伪造输入修订；采纳的 source / before / after 在动作回执中记录，不建立结果持久化历史库。

## D05：工程格式、方法与旧读者

| 项目 | 正式值 / 范围 |
| --- | --- |
| 外层工程 | 保持 v4；新增必需 capability，不把未知 EO 内容交给旧 reader 可编辑 |
| 方法 | `radishflow.fixed-tp-material-eo` / version 1 |
| 能力 | `radishflow.solver.fixed-tp-material-eo` / version 1 |
| 模型 / 验收契约 | `equationContractVersion=1`、`acceptanceContractVersion=1` |
| 核心执行身份 | 正向 dense LU 与反算 conditional-LU / bisection 分列实际 backend version；不能只记录 UI 方法名 |
| 正式配置 | 固定 T/P；零或一个 released split binding；零或一个 product-total target；反算区间 / 二分预算；默认或全量显式变量 / 残差尺度 |
| 不保存 | 求得比例候选、结果、匹配 / Jacobian、数值索引、括区轨迹、运行 token、临时草稿 |

`simulation.steadyStatePlan` 的 E2 配置如下；字段只为新方法定义，旧方法不接受这些增量：

```json
{
  "method": { "id": "radishflow.fixed-tp-material-eo", "version": 1 },
  "equationContractVersion": 1,
  "acceptanceContractVersion": 1,
  "fixedBoundary": { "temperatureK": 300.0, "pressurePa": 650000.0 },
  "specifications": [
    {
      "kind": "release_split_fraction",
      "unitId": "splitter-1",
      "field": "outlet_a_split_fraction",
      "interval": { "lower": 0.0, "upper": 0.95, "maxIterations": 64 }
    },
    {
      "kind": "product_total_molar_flow",
      "streamId": "s-pa",
      "source": { "unitId": "splitter-1", "port": "outlet_b" },
      "valueMolS": 1.5
    }
  ],
  "scaling": { "kind": "evaluated_fresh_total" }
}
```

正向 `specifications=[]`；两种记录各最多一个，`field` 只接受该 typed 字段，不是任意可写路径。目标需要完整生产端绑定，不能按重名流股匹配。显式尺度为 `kind=explicit_equations`，分别包含 `variables` 与 `equations` 的 `{binding, scaleSi}` 数组；binding 使用 D02 的 tagged 身份，尺度的量纲由变量 / 行角色决定，不接受自由单位字符串。方法字段严格拒绝未知项、显式 null、重复键、重复身份与非有限数值；规范排序不制造计划修订。

规格独立字段允许暂缺其中一侧的可编辑工程，shape 只检查类型、重复、版本、有限正目标和区间；运行就绪再检查完整配对、对象、目标是否终端、依赖及结构。零目标首批不支持；端点 alpha=1 的零产品正向情况另按现有零流规则检查。释放参数不使用旧输入值作为数值初值或固定条件。所有身份和尺度规范排序；配置摘要包括 SI、规格、区间、预算及方法 / 契约版本，缺省尺度只保存规则，实际 F_ref 在运行证据中冻结。

扩展 `FlowPlanConfiguration` / `StoredFlowPlan` 的 EO 变体与被引用 ID 保留；旧物理 `rf-model` 不加入求解状态。删除 / 重建同名对象不重绑，保存重开仍可诊断失效身份。输入 revision 与 plan revision 仍分别拥有历史和 saved baseline；采纳属于输入历史。

读者能力新增 EO 层级，新程序的旧 CLI v4 固定 Sequential、v5 固定 Simultaneous；默认新 GUI / CLI v6 才支持 EO。未知能力或合法未知方法 / 契约整份只读，已知损坏内容报错，未知外层拒绝；能力缺失 / 伪造不能绕过方法识别。保存复用来源和目标的能力增量检查，同 v4 Save As 新路径也提示；取消 / 失败保留原文件、路径、草稿及基线，确认期间内容或目标变化重新确认。

历史验证用 `f2ba51c1` 独立 reader / writer 实际读取新工程并拒绝覆盖；与新程序的旧协议测试分别记录。重开正式配置后结果 Missing，不自动运行。

## D06：后台运行、资格与失败

继续使用已有单 worker，冻结图、EO 配置与选包请求，经资源装载、Ready 双资格与授权复验后执行。解析规格、匹配或纯结构查询不求物性；数值执行和局部物化的包事实分开记录。计划改动只按 Flow 计划语义处理，不失效无关 QS0 / 设备；采纳改物理输入则沿既有依赖规则处理。

当前工作区的静态未就绪在正式预检返回 Blocked，不伪造一次核心 Failed 或耗时；worker 仍独立核对冻结问题。工作区报告、worker 问题及完成证据共享同一领域校验，身份不同则重建，不使用旧结构报告替本次执行背书。

新增 EO 进度阶段至少为 Assemble、StructuralAnalysis、LinearEvaluation、BracketEndpoint / IntervalEvaluation、FullJacobianCheck、FinalUnitVerification、GlobalVerification、Completing，带领域对象、线性行 / 列、区间序号等实际上下文。匹配遍历、LU 列、条件求解 RHS、每次目标评价、局部单元前后和接收前设协作取消点；单次不可中断物性调用仍等待返回，不宣称已暂停。

取消请求立即撤销本次发布资格，但名额仅在 worker 退出 / join 后释放。取消保留既有 Current 的原资格，已有 EO 诊断跟随本次执行终态；不新建 Failed。失败、取消、丢弃、线程 panic / 断连与包准备失败各有分类；后两者不能伪造方程 / 数值证据。

正式失败种类包括 UnsupportedModel、InvalidSpecification / Binding、StructuralUnder / Over / Singular、LinearSingular / IllConditioned、TargetNotBracketed、BudgetExceeded、Stagnated、NonFinite、UndefinedComposition、LocalVerification、GlobalVerification。只有实际证明才能报告规格矛盾；数值失败不充当无解证明。局部已完成单元、候选与残差只属诊断，失败无正式 fields / streams。None、有限值、NaN / ±Infinity 沿 QS2 的显式三态投影；未取得诊断不填零。核心已调用时真实阶段计数可以为 0，核心未调用时不创建伪核心成本或执行来源。

输入＋计划 generation、session、run acceptance 与载荷类型共同检查 Ready、完成、查询、输出和采纳；旧完成或旧捕获在改计划、改回、重开、重算、覆盖等待后都不能恢复资格。新增 EO 失败证据与其他 Flow 失败载荷互斥，不按提示字符串判断种类。

## D07：GUI 与局部画板

沿现有 Run / F5 增加“固定温压物料联立方程”，同页包含固定边界、正向 / 单比例目标规格、完整变量 / 方程只读预览、双尺度及反算区间 / 预算。比例的原输入仍在单元设置；计划只声明释放与目标，不复制一份物理比例。Apply、恢复已保存、计划 Undo / Redo 与输入 Undo 区分；未完成草稿保留文本和单位，并阻挡 Run / 自动运行 / 当前输出 / 候选采纳。

结构检查显示分析范围、方程 / 变量 / 匹配数、未匹配区域和相关对象，可定位具体端口 / 字段；“结构通过”不写成“必定收敛”。Current 展示原输入与求得参数、目标 / 实际值、残差、括区、完整与条件系统维数 / 条件指标、真实 I/O 与来源；Failed 的部分证据不混入正式结果。Stale / Missing 继续隐藏正式数值，运行取消明确等待退出。

方向获批后在既有 `.pen` 新增局部画板，至少覆盖：①正向配置与方程浏览；②释放比例 / 产品目标及欠 / 过规格；③Current 和采纳预览 / 采纳后 Stale；④病态 / 未括区 / 取消；⑤中英 400 px 草稿、长身份、键盘焦点及放大字体；⑥v4 能力升级 / 未知只读。**本篇批准不替代画板认可；认可对应局部稿后才接 GUI。** 不展开全局 R0 重设计。

### 局部画板评审（2026-10-31，已认可）

主稿 `docs/architecture/designs/studio-client-main.pen` 已保存，保留 QS0 / QS1 / QS2 旧稿。所有者已认可下列六类七张画板并要求继续 GUI 接线；163 个节点的裁切、零尺寸和占位检查通过，逐图视觉复核完成。数值为独立案例的舍入排版示意，失败指标为故障示意；静态检查不代替真实 GUI 验收。

| 场景 | 画板 / PNG |
| --- | --- |
| 正向配置、完整变量 / 方程、尺度与结构限制 | [QS3-01 / KUmK6](../../architecture/designs/reviews/2026-10-31-qs3/KUmK6.png) |
| 单比例 / 产品目标、29／29 与欠 / 过规格 | [QS3-02 / dmqNy](../../architecture/designs/reviews/2026-10-31-qs3/dmqNy.png) |
| Current 原输入 / 求得值、采纳预览及 Stale | [QS3-03 / NDla8](../../architecture/designs/reviews/2026-10-31-qs3/NDla8.png) |
| 未括区、病态、局部失败、取消 / join | [QS3-04 / WYYBO](../../architecture/designs/reviews/2026-10-31-qs3/WYYBO.png) |
| 400 px、20 px 字体、草稿与长绑定 | [QS3-05 中文](../../architecture/designs/reviews/2026-10-31-qs3/c7zijj.png) / [English](../../architecture/designs/reviews/2026-10-31-qs3/FNQ8R.png) |
| v4 同版本能力升级、旧读者 / 未知只读与重开 | [QS3-06 / p923O](../../architecture/designs/reviews/2026-10-31-qs3/p923O.png) |

## D08：CLI v6、验证矩阵与连续开发

独立 `schema_version=6` 的 query / execute，复用现有 planned 执行框架与引用解析。旧 v1—v5 词汇 / 错误 / 磁盘边界保持；v5 在共享 parser 创建可编辑工作区前固定 Simultaneous 能力。EO 方法 wire 为 `radishflow.fixed-tp-material-eo/1`，配置字段与 D05 一一映射，严查未知 / 重复字段、null、单位、版本与引用。工程仍是 v4，不因 CLI 版本变化升级外层。

新增查询为 EO 规格 / 分析范围 / 结构预览和实际结果；`execution_kind=equation_oriented` 独立投影变量 / 方程、目标、求得参数、最终 I/O 和执行证据。能力发现准确列明正向 / 单比例反算限制，不允许任意表达式、多个自由参数或热规格。配置和 Run 继续要求期望输入 / 计划身份；目标和释放绑定可用前序创建的类型化引用。

沿已有 `configure_flow_plan` / `run_flow` 与 `results` 的 `flow_plan` / `flow` scope 查询扩展 v6 投影。EO plan 必填 `equation_contract_version`、`acceptance_contract_version`、`temperature_k`、`pressure_pa`、`specifications`、`scaling`；使用 snake_case，对象位置沿既有 `IdentityRef` 而非照搬存储 ID 字符串。采纳动作命名 `adopt_eo_split_fraction`，执行引用只指向已接收的本请求结果；安全元数据查询保留原文件版本 / SHA-256，内存能力另列，不能把配置或采纳回执称作已保存。

新增采纳动作只引用本请求内已成功并被接收的 EO 执行，以及确切求得字段，不能提交调用者自选数值。返回输入动作回执、Current→Stale 和保留的反算计划；后续步骤可以显式改正向计划再 Run。跨进程执行引用不持久化、不接受伪造历史 token。沿既有受控影响确认规则，v6 使用独立 `headless-v6-confirm-v1:`；普通配置 / Run 不加泛化审批。所有操作仅在请求内存副本发生，无 Save、偏好、缓存写入、sidecar 或结果文件写入。

### 验证矩阵

| 层 | 必须获得的证据 |
| --- | --- |
| 领域 / 导数 | Feed / Mixer / Splitter 同源关系；固定与释放比例导数；多个合法点 / 步长的独立扰动；不同组分数量、量纲、非有限 / 非正尺度及错误绑定 |
| 结构 | E1 28／28、E2 29／29；欠 / 过规格和共存区域、重复方程、完全匹配但数值相关；未支持模型不报零自由度；报告身份和失效正确 |
| 正向与反算 | 分数表逐股比较，正向与同条件 QS2 比较；两个反算端点、内部根、未括区、比例无影响、奇异区间 / 中点、预算与不可缩区间；多回流 / 无环与容量拒绝 |
| 独立复核 | 端口假设与真实生产者差异、微小非零残差、零流组成来源；注入局部偏差、遗漏出口、身份错配、最终目标 / 全局物料失败；不发布数学通过但物化失败的结果 |
| 资格 / worker | 真实 Ready / start / cancel / join；结构、LU、端点、中点、局部及接收前取消；输入 / 计划 A→B→A、迟到成功 / 失败、错载荷 / 重复、重开 / 跨会话、QS0 / 设备互斥及范围 |
| 保存 / 兼容 | shape 与 readiness 分离，默认 / 显式完整映射；未知 / 缺失能力、旧协议拒绝、同 v4 来源 / 目标升级、Save As 取消 / 失败 / 目标变化；历史独立 reader / writer；重开 Missing |
| 结果 / 采纳 | 原参数与解分列，输出转换与量纲 / scope；预览后变更拒绝、错执行 / 自选数值拒绝、只读 / 草稿 / Busy、重复与 no-op；单输入事务 / Undo / Redo，采纳后 Stale、计划不变，切正向 rerun 一致 |
| GUI / CLI | 批准画板后真实配置 / Run / 错误定位 / 采纳 / 保存重开；中英窄区和键盘，真实 CLI 建图 / 查询 / 反算 / 采纳 / 切正向 / 重跑；前后比较整个临时目录、资源与工程字节不变 |
| 基线 / 平台 | 各阶段定向验证后，阶段完成执行正式 check-repo；macOS 代表实窗先告知；Windows / Linux、I5、原生 OS 缩放与 PME 缺口分别登记，不因本机成功消失 |

### 实施节奏

1. **QS3-A 模型与结构：** 落实 D01—02、独立案例 / 导数 / 拒绝基准，形成可查询的多单元问题与结构报告；不交付成功占位算法。
2. **QS3-B 求解与真实复核：** 落实 D03—04 的数值和结果部分，验证正向、单比例反算、故障和取消；共享 dense 提取随真实调用落地，QS2 门槛与行为完整回归。
3. **QS3-C 工程与运行：** 落实 D05—06 及采纳正式动作，贯通保存、历史、ID、冻结 / 接收、结果和旧读者保护；独立字段变化与计划变化分别验收。
4. **QS3-D 正式消费者：** 完成 CLI v6 与公开示例；对应局部画板认可后接 GUI，完成配置、诊断、求解、采纳、保存重开与复现。
5. **QS3-E 阶段验收：** 归集各层证据，一次正式仓库门禁与必要代表实窗，更新专题 / 当前状态 / 周志。后续仅文档修改补文档检查；新失败或代码变化再做相应复验。

按职责分批提交；已批准范围内连续开发，内部小步骤不反复请求批准。接口 / 物理范围、算法类别、依赖或持久化发生实质变化时重新确认。本批不引入外部数值库、不改 lockfile、不创建远端任务服务、不推送 / PR / 发布；AF1 独立故障跟踪、C2-A 专项、C2-B / C3 和平台门禁保持。

## 已批准决策与影响

| 决策 | 具体批准范围 | 主要影响 |
| --- | --- | --- |
| QS3-D01 | 三类固定温压物料模型、多单元正向＋一个分流比例 / 一个产品总流量反算；E1 / E2 与 128 维上限 | 扩展 EO 与 DS 的受支持任务范围，热 / 相平衡不进入 |
| QS3-D02 | 端口 / 参数未知量、模型残差 / 解析偏导、领域身份、最大匹配及结构 / 数值分层 | unitops 新能力、solver 新装配与有限 A4 查询；旧 QS0 不改语义 |
| QS3-D03 | 正向 LU、反算条件线性消元＋二分；双尺度、门槛、预算、最终完整 Jacobian 检查 | 新算法与内部 dense 复用，无依赖升级；不宣称通用非线性 |
| QS3-D04 | 独立 EO 结果、真实局部复核、零流来源和仅显式采纳比例 | 结果接口与新的输入事务；候选不自动写回，不持久化结果 |
| QS3-D05 | 独立 method / capability / 契约 v1，工程外层 v4、规格配置与旧读者保护 | store / UI 类型与格式能力增量、同版本保存升级提示 |
| QS3-D06 | 既有 worker 中增加 EO 阶段 / 载荷，复用双资格 / 取消 / join | 运行和诊断接口增量；单名额、QS0 / 设备资格边界保持 |
| QS3-D07 | 原 Run / F5 下的规格、结构、结果与采纳交互方向 | 局部画板已获认可并完成接线，方向批准不替代局部评审 |
| QS3-D08 | 独立 CLI v6、旧 v5 固定能力、执行引用采纳及完整分阶段验证 | 新 JSON 词汇 / 投影 / 确认命名空间，继续请求内存与磁盘不变 |

所有者已批准本表并要求按计划连续实施、验证和同步文档，不推送或创建 PR。GUI 方向批准不替代局部画板认可；阶段完成以验证证据为准，不以文档或接口命名代替实现。
