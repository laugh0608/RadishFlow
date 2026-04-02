# MVP Scope

更新时间：2026-04-02

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
- `rf-unitops` 第一轮统一围绕标准 `MaterialStreamState` 输入输出，不提前把 flowsheet 调度或 FFI 细节塞进单元接口
- `Feed`、`Mixer`、`Flash Drum` 当前先冻结为 canonical material ports：`Feed(outlet)`、`Mixer(inlet_a/inlet_b/outlet)`、`Flash Drum(inlet/liquid/vapor)`
- `rf-flowsheet` 第一轮连接校验只覆盖 canonical material ports、流股存在性与“一股一源一汇”；终端产品流允许只有 source、没有 sink
- `.NET 10` 适配层在 `M4` 前只允许文档和最小占位，不提前展开复杂运行时实现

App 与交互层当前进一步冻结以下口径：

- MVP 保持单文档工作区，不急于做多文档容器
- 单文档工作区不等于单文件实现，源码仍按职责拆分
- 属性编辑采用字段级草稿态，语义提交后才写回文档
- 求解控制采用 `SimulationMode(Active/Hold)` 与 `RunStatus` 分离模型
- 求解结果采用独立 `SolveSnapshot`，不直接覆盖文档对象
- 结果快照应保留按步展开能力，为撤回/前进和脚本化扩展留接口
- `DocumentMetadata` 只保存文档身份与序列化元信息，不保存文件路径、求解态和用户偏好
- `UserPreferences` 只保存应用级偏好与快照窗口策略，不污染文档语义
- `CommandHistory` 只记录语义化文档命令，运行控制和文档生命周期动作不进入撤回栈
- `SolveSessionState` 必须绑定当前观察的文档修订号，`SolveSnapshot` 由工作区持有有界历史窗口
- Studio 当前应用层运行入口先冻结为 `StudioAppFacade + WorkspaceRunCommand + WorkspaceSolveService + solver_bridge` 四层，不让 UI 直接拼接底层 provider/solver 细节
- `rf-ui` 当前运行栏状态先冻结为 `RunPanelState + RunPanelIntent + RunPanelCommandModel + RunPanelWidgetModel`，把按钮意图、主动作、按钮槽位、文本布局和最小渲染/触发所需状态都留在 UI 层，不让视图层或 Studio 侧重复发明一套按钮语义
- Studio 当前对运行栏的最小消费也已前推到 `RunPanelWidgetEvent`，不再只接受裸 `RunPanelIntent`
- 当前最小桌面入口 `run_studio_bootstrap` 也已补出 `StudioBootstrapTrigger`，允许样例入口显式选择“走 intent 触发”或“走 widget action 触发”
- Studio 当前运行触发先明确区分 `Manual` / `Automatic`，并把 `SimulationMode` / `pending_reason` 的运行门控收口在应用层
- Studio 当前默认包选择采取保守策略：只有唯一候选包明确时才自动选中；多包场景必须显式指定 package，不在当前阶段隐式猜包
- Studio 当前 Automatic 触发在命中 `HoldMode` / `NoPendingRequest` 时应先返回 skip，再决定是否需要 package 解析，避免多包缓存场景下的无意义失败
- 当前最小桌面入口 `run_studio_bootstrap` 也已改为直接消费 `RunPanelIntent`，并向入口层输出 `RunPanelWidgetModel`，确保“桌面触发点 -> UI 意图 -> Studio 控制动作 -> UI 组件 DTO”边界在样例入口里就成立

认证、授权与受控物性资产当前进一步冻结以下口径：

- 桌面端统一走 `OIDC Authorization Code + PKCE`
- RadishFlow 桌面端是 `public client`，不内置长期 `client_secret`
- 登录默认采用系统浏览器 + loopback redirect，不照搬 Web 客户端 `localStorage` token 方案
- Access Token / Refresh Token 只允许落在操作系统安全存储
- 外部控制面默认采用 `ASP.NET Core / .NET 10`，不额外引入 Go 服务主线
- 高价值原始物性资产不默认完整下发到客户端
- 本地求解热路径继续本地执行，远端服务只承担身份、授权、租约、清单和派生包分发
- 派生物性包分发优先采用对象存储 / CDN / 下载网关 + 短时票据，不把控制面 API 设计成长时大文件出口
- 允许引入离线租约与本地派生物性包缓存，但不承诺客户端绝对防提取
- 项目文件继续固定为单文件 `*.rfproj.json` 真相源，授权缓存索引与派生包缓存继续留在应用私有缓存根目录
- MVP 默认不把 `snapshot_history`、token 明文或授权缓存索引混进项目文件
- 桌面交付默认采用“压缩包 + 主入口 + 附带资源目录”的原生客户端形态，不以单文件可执行为当前阶段目标

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

- 冻结 `AppState`、`WorkspaceState`、`FlowsheetDocument`、`DocumentMetadata`、`UserPreferences` 的字段边界
- 冻结字段级草稿提交流程、`CommandHistory` 边界和工作区保存态口径
- 冻结 `SolveSessionState`、`DiagnosticSummary`、`SolvePendingReason` 与 `SolveSnapshot` 的关系
- 明确 UI 交互层、求解层和快照历史之间的数据所有权
- 冻结桌面登录、授权、离线租约与远端资产控制面的总体边界
- 冻结 `StoredProjectFile` / `StoredAuthCacheIndex` JSON DTO、相对缓存路径布局、客户端注册与 scope 命名

补充说明：

- 截至 2026-03-29，上述大部分基础冻结项已提前完成，不再视为后续待办
- 截至 2026-03-29，本地 `PropertyPackageManifest` / `payload` 实体读写、`PropertyPackageProvider` 的本地缓存接线、下载落盘路径、下载 JSON 到本地 payload 的映射和首个真实样例包也已提前完成
- 截至 2026-03-29，下载获取抽象、基于规范化 payload 的摘要校验、失败回滚和样例摘要也已提前收口完成
- 截至 2026-03-29，下载抓取失败分类与有限次重试策略也已提前收口完成
- 截至 2026-03-29，原始 HTTP 请求/响应适配层与状态码到失败分类的映射也已提前收口完成
- 截至 2026-03-29，基于 `reqwest + rustls` 的真实 HTTP client adapter 也已提前收口完成
- 截至 2026-03-29，控制面 `entitlement / manifest / lease / offline refresh` HTTP client 与应用层编排也已提前收口完成
- 当前剩余重点已经进一步转向联网失败策略细化、数值主线和求解闭环，而不是继续停留在第一轮 DTO 草案

补充对齐：

- 今天（2026-03-30）优先在已接通的控制面 client 与应用层编排之上，细化授权刷新后的 UI 事件流、联网失败提示和离线刷新触发策略
- 之后优先恢复 `rf-thermo` / `rf-flash` 数值主线，再进入 `rf-solver` 的无回路顺序模块法与首个可求解 flowsheet 示例

进一步补充：

- 截至 2026-03-31，`rf-thermo` / `rf-flash` 的最小二元数值主线、黄金样例、`rf-unitops` / `rf-flowsheet` 的第一轮边界，以及 `rf-solver` 的首个无回路闭环都已提前推进
- 当前近期主线已从“补第一条求解闭环”切换为“扩第二个内建单元、增加第二个示例 flowsheet，并细化求解结果与诊断口径”

截至 2026-04-01，再补充对齐：

- `rf-ui` 已能把内核 `SolveSnapshot` 映射并回写到 `AppState`
- `apps/radishflow-studio` 已具备从物性包加载、工作区运行命令、运行门控到结果回写的最小应用层闭环
- 当前近期 Studio 主线已从“只有求解 bridge”切换为“继续把运行命令、结果派发和后续异步执行边界收口”

### 2026-W16

- 已提前完成 `rf-thermo` 中的 Antoine 饱和蒸气压与理想体系 `K` 值估算
- 已提前完成 `rf-flash` 中的 Rachford-Rice 和最小二元 `TP Flash`
- 已提前建立 `tests/thermo-golden` 与 `tests/flash-golden` 的首批黄金样例

### 2026-W17

- 已提前完成 `rf-unitops` 中 `Feed`、`Mixer`、`Flash Drum` 的最小统一接口
- 已提前完成 `rf-flowsheet` 中的端口连接与基本校验
- 已提前明确单元输入输出的标准流股接口与 canonical material ports

### 2026-W18

- 已提前完成 `rf-solver` 中首轮无回路顺序模块法
- 已提前增加第一个可直接从 `*.rfproj.json` 载入并求解的示例 flowsheet
- 在已接通的控制面调用编排之上补授权刷新后的 UI 事件流和更细的联网失败策略
- 扩第二个内建单元闭环示例，并补更完整的端到端回归样例

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
