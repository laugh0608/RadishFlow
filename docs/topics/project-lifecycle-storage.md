# 项目生命周期与存储

更新时间：2026-10-15

> 本文定义该专题的能力、开发范围与验收要求；具体迭代切片和优先级以 [当前状态](../status/current.md) 为准。

## 用途

用途：定义项目打开、新建、保存、另存为、最近项目、脏改确认、项目文件和 sidecar 的开发边界，以及未来装置工程容器的持久化要求。
读者：负责 Studio 文档生命周期、`rf-store`、GUI shell、项目格式和保存 / 重开验证的开发者、用户、AI / Agent。
不包含：完整多文档容器、云同步、多人协作、版本控制系统、安装包发布和跨设备项目库。

## 专题目标

- 当前项目语义继续以 `*.rfproj.json` 为真相源；未来工程容器按下文规划单独设计迁移。
- GUI 布局、窗口偏好和 sidecar 与项目语义分离。
- 打开 / 保存 / 另存为 / 关闭确认不静默丢失用户工作。

## 子专题关系

- 本地项目文件、sidecar、最近项目和脏改确认留在本专题。
- 控制面授权、派生资产缓存、离线租约和服务端审计属于 `platform/control-plane-service.md`。
- 发布包、安装器和自动更新若进入范围，应新增独立专题。

## 单位保存与后续规划衔接

2026-09-16 规划新增 [单位偏好](units/unit-sets-and-persistence.md) 与 [设备任务](equipment-design-and-rating.md) 的保存恢复。单位配置所有权、独立保存脏标记与兼容规则只在单位子专题维护，本专题提供安全保存共用流程；U2-I1 / I2 已接通外层 v2 / metadata v1、旧 v1 显式升级、独立呈现脏标记与个人默认；普通保存只写已提交 SI 与项目单位选择，保留并提示未提交草稿。I4 临时视图覆盖、输入单位及编辑历史不进文件，统一离开确认已接通，I5 按已确认条件补验。设备几何 / 工况来源的持久化按各切片确定；候选与已采纳输入分开，恢复时重验模型 / 方法与源工况有效性，缺失能力不静默丢弃未知数据。

[工程基础与工况](engineering-basis-and-cases.md) 后续需要持久化共同条件、显式覆盖、来源、工程位号及案例身份；位号更名不改变内部引用。有效输入 / 方法变化与纯显示或说明变更分别管理，保存修订不自动等同求解输入修订。冻结运行依据与当前文档分离，来源更新不覆盖历史记录；具体 schema / 迁移及资产保存边界在 G1 / G3 和 Q2 / Q3 确定，旧工程来源缺失标为未记录，不伪造数据。

## 装置工程容器规划

2026-09-26 确认 [装置工程愿景](../architecture/simulation-platform.md#装置工程与数字孪生愿景)。目标是让一个工程承载工厂或装置的共同基础、设备 / 流程、多个模型与工况、运行方案、结果及证据。工程目录可作为工作载体，冻结交付包用于复制、归档和交付；具体目录、清单格式、文件后缀及事务机制尚未选定，现有单文件格式与 sidecar 行为保持。

| 持久化关注点 | 后续设计与验收要求 |
| --- | --- |
| 工程清单与引用 | 显式管理工程 / 资产身份、角色、版本及引用；同一设备的模型通过稳定身份关联，不按文件名推定所有权 |
| 配置与数据分类 | 工程输入、呈现偏好、模型参数、观测数据、结果、运行证据和重启检查点分开；缓存或 UI 布局不成为第二工程真相源 |
| 外部大数据 | 可引用历史数据库或外部时序资产，记录数据集版本 / 时间窗和复现所需信息；活动查询不冒充冻结数据，凭据不随工程复制 |
| 冻结与交付 | 按许可和权限收集所需资产或列出缺失项；工程移动后引用可解析，重开校验版本 / 完整性；复制、另存为与克隆的身份规则另行定义 |
| 多文件保存与恢复 | 明确一致性边界、保存失败和中断恢复，不把现有单文件 staged write 当成多文件原子提交；失败后保留可恢复状态与诊断 |
| 旧工程迁移 | 保留现有工程读取与验证依据，显式升级并检查未知能力 / 缺失资产；不静默丢弃数据或自动恢复现场连接、控制输出及运行任务 |

目录工程不要求同时打开多个原生窗口，也不强制将所有在线历史数据打包到本地。未来配置 / 运行记录与当前输入分开保存，记录时间窗和模型版本；完整性校验不等于数据一定可获得或模型仍可执行。

进入实现前，先用一个真实多资产工程确定清单、引用、迁移和保存契约，再验证移动 / 重开、冻结 / 导出、断开外部数据、缺失模型、旧格式导入、部分写入失败与恢复。切片依赖见 [路线图](../radishflow-mvp-roadmap.md#装置工程与数字孪生的演进切片)；本轮仅更新规划，不新增 schema、空目录或数据库依赖。

## 当前实现快照

2026-09-23 [运行与诊断规划](runtime-messages/records-storage-and-consumers.md#存储消费者与架构边界) 将可复用监视规则拟归项目运行配置，运行捕获规则版本；筛选 / 展开 / 跟随归呈现偏好，事件与调试数据归运行证据。N2 / N3 再确定 schema、保存脏标记、容量与迁移，不能每条消息都推进工程修订或 Undo。旧工程无规则保持行为，未知规则和失效目标不静默忽略；重开不自动恢复暂停运行，旧问题须重新检查。结果观察记录不等于求解器重启检查点，本轮格式不变。

已完成：

- 单文档工作区；Studio 按外层版本分派，支持已知 v3 的正式输入恢复及未知方法的整份只读检查；旧无界面协议保持独立 v1/v2 入口，运行态由各调用者建立。
- 项目文件 staged write。
- `Save` / `Save As` / 覆盖确认；macOS 原生项目打开、首次保存与另存为主路径已实窗验证，Windows / Linux 新增路径证据见下文。
- 最近项目列表和当前工作区返回入口。
- 脏项目窗口关闭与 `Cmd+Q` 统一确认。
- Canvas layout sidecar 使用 `<project>.rfstudio-layout.json`。

已知缺口：

- 多文档、多窗口原生宿主、云同步和完整偏好系统未进入范围。
- 跨平台原生文件选择器仍不是完整工作流承诺。

## B0 保存与重开复核（2026-09-13）

基线 `15ac7b97`，macOS ARM64。本轮从普通空白项目构建 Feed–Flash 并求解，分层结论如下；详细操作与测试命令见 [W37](../devlogs/2026-09/2026-W37.md#2026-09-13-b0-基础能力复核)。

| 能力 | 状态 | 证据与边界 |
| --- | --- | --- |
| 底层保存 / 重开 / rerun | 已验证可用 | [空白项目 shell 测试](../../apps/radishflow-studio/src/studio_gui_shell/tests/canvas.rs) 24 项与 [生命周期测试](../../apps/radishflow-studio/src/studio_gui_shell/tests/project_lifecycle.rs) 31 项通过，含参数、重连、物性与组分、sidecar、覆盖取消、失败不推进保存状态；使用真实临时项目文件，部分流程注入 picker，不等同原生选择器实测 |
| macOS 空白项目 GUI 首次保存 / 另存为 | 缺失 | [project_picker.rs](../../apps/radishflow-studio/src/studio_gui_shell/project_picker.rs) 的非 Windows 分支直接返回 `None`。实窗点击“文件 → 保存”不显示对话框，消息为 `Save As canceled`，项目仍未保存；这是未实现平台能力被展示为取消，不是用户主动取消 |
| macOS GUI 任意文件打开 | 缺失 | 同一非 Windows picker 分支返回 `None`；本轮未将示例入口或底层 `open_project_from_input` 当作任意路径 GUI 打开已完成的证据 |
| 本轮空白案例 GUI 保存后重开 | 待验证 | 首次保存受上述缺口阻塞，没有生成本轮 GUI 案例文件；不能把 shell 测试的保存重开声明成这份实窗案例已保存 |
| 脏项目窗口关闭 / 取消 | 已验证可用 | 实窗关闭出现保存 / 舍弃 / 取消；取消保留案例，随后舍弃一次性验证项目并关闭。`Cmd+Q` 有现有测试，本轮未重新实测该快捷键 |
| Windows / Linux 原生生命周期 | 待验证 | Windows 有 rfd 实现，本轮未启动 UTM 或复验其选择器；Linux 与 macOS 共享未实现 picker 分支，不能从 macOS 结果推导 Linux 运行态全部表现 |

应单独确认的修复范围：补 macOS 原生打开 / 首次保存 / 另存为到现有 `ProjectFilePicker` 与 lifecycle 路径，区分平台不支持、取消和失败；验证扩展名、含中文 / 空格路径、覆盖确认、取消保留、IO 失败可重试、保存后重开 / rerun 和 sidecar。保持项目格式、staged write 与 Windows 恢复契约，不把本项扩大成文件管理器或发布打包。B0 当时仅登记方案，后续实现见下节。

这是 [B1-1 单元删除](flowsheet-modeling-and-solve.md#b1-1-单元删除与关联清理2026-09-13) 在 macOS 完成完整 GUI 持久化验收的独立前置缺口；两项已分别明确范围并获项目所有者确认。

## B1 macOS 原生项目文件选择器（2026-09-13）

已将现有 `rfd 0.15` 项目打开 / 保存选择器扩展到 macOS，复用 `ProjectFilePicker`、覆盖确认及 staged write；`Cargo.lock` 不变。不支持项目选择器的平台先显示能力未实现提示，不再伪装成用户取消。当时结果导出选择器保持原范围，后续实现见 [B2-1](results-review-diagnostics.md#b2-1轻量结果输出闭环)。

- 已验证可用：macOS 实窗首次保存、另存为、原生打开与取消打开；中文含空格路径、无扩展名自动补 `.rfproj.json`、显式扩展名不重复。普通空白 Feed–Heater–Flash 的删除版 / Undo 恢复版分别保存重开，恢复版 rerun 收敛。
- 已验证可用：保存同步当前单元位置，首次保存和另存为均写入目标 sidecar；不保存已删除单元位置，会话中仍可 Undo 恢复。统一 shell 保存命令与文件菜单路径，避免命令面板保存漏写布局。
- 自动化已验证：真实临时文件上的删除 / Undo 保存重开、参数与外侧绑定、布局失败警告及重试；布局失败明确“项目已保存，布局保存失败”，不声称工程数据写入失败。既有生命周期测试继续覆盖覆盖取消、输入写入失败与保存状态。
- 待验证：Windows / Linux 新增路径、macOS 原生覆盖对话框的逐按钮组合和真实权限故障；本轮不把未测项登记为缺陷。Linux 项目选择器仍未实现；macOS 结果导出原生选择器不在此修复范围。

B0 表中的 macOS 项目 picker 缺失已由本节覆盖，B0 当时观察保留为历史证据。操作流水见 [W37 B1](../devlogs/2026-09/2026-W37.md#2026-09-13-b1-1-单元删除与-macos-保存闭环)。

## Windows staged write 失败恢复

2026-09-14 B2-1 将 staged write 从 JSON 模块提取到同域存储模块，JSON 与文本导出共用安全写入和恢复序列；工程格式及保存调用语义不变。

Windows 保存已有文件继续先移出备份，再提交 staged 文件。2026-09-12 将替换与恢复序列收口到 [staged_replace](../../crates/rf-store/src/staged_file/staged_replace.rs)，补齐双重失败的信息：

- 备份移动失败：保留原文件，返回原始 IO 错误。
- 替换失败、恢复成功：原文件可重开，保存仍返回失败，不推进 Studio 的已保存 revision。
- 替换和恢复同时失败：返回 `store.file_replace.recovery_failed`；`RfErrorContext::file_recovery()` 提供目标路径、备份路径和两次错误文本，显示消息也包含这些信息。
- 替换成功：沿用既有备份清理行为；清理失败不推翻已完成的保存。

故障注入测试在真实临时文件上执行相同替换序列，覆盖成功、备份失败、恢复成功、双重失败及从备份恢复后重开。该测试不模拟断电，也不保证外部进程删除备份后仍可恢复；不存在依据错误消息自动覆盖用户文件的新增行为。项目格式和现有 `invalid_input` 错误码保持兼容，新的恢复上下文供 Rust 调用方使用，不新增 FFI JSON 字段。

## U2-I1 项目版本与呈现装载（2026-09-23）

已接通 v1 / v2 明确读取、完整显示单位集与来源版本保留；实际格式、严格校验及兼容证据唯一维护在 [单位存储契约](units/unit-sets-and-persistence.md#u2-存储接口)。I2 已接通独立呈现保存与历史、统一保存摘要和个人默认。新建保存 v2；来源 v1 首次保存须明确同意升级，取消不改盘；保存失败不更新基线。I3 / I4 数值会话、首批非 SI 输入和离开保护已接通，I5 完整验收待续。

### U2 离开保护与保存状态

2026-10-02 新建 / 打开 / 关闭统一列出受影响的未提交输入，保存摘要分别表达工程内容、显示设置和未提交输入。确认绑定工程身份 / revision、输入代次及其他草稿、呈现状态、保存基线与路径；确认后现场变化须重验，取消或打开失败保留当前工程。设置候选仍打开时须先应用或取消。

保存仅写入已提交内容和已应用项目配置，不应用或放弃字段草稿；有草稿时“保存并关闭”仍保持窗口，用户可返回处理或明确舍弃。关闭弹窗初始焦点为取消，Escape 取消关闭。保存快捷键复用菜单 / 命令面板的壳层流程，v1 升级确认不会被绕过；确认弹窗期间命令面板与全局快捷键保持隔离。代码 / 事务边界见 [统一离开确认](units/input-drafts-and-interactions.md#i4-统一离开确认与失败保护)，macOS 多对象、保存 / 打开 / 关闭增量证据见 [I5 原生组合](units/input-drafts-and-interactions.md#i5-多对象草稿与离开保护原生验证)；迟到确认与失败事务已有自动回归及部分原生证据，剩余原生组合按 [I5 延期节点](units/requirements-traceability.md#i5-受控延期与-u3-开工安排) 补验。

## 用户路径

1. 新建或打开项目。
2. 编辑物性、流程图、参数或布局。
3. 保存或另存为。
4. 关闭或退出时如有脏改，必须明确保存 / 放弃 / 取消。
5. 重开项目后项目语义、最近项目和可恢复布局符合预期。

## 范围

本专题纳入：

- 单文档工作区生命周期。
- 项目文件读写和 staged write。
- 最近项目和当前工作区返回。
- 脏改确认。
- sidecar 布局文件边界。

当前实现批次不纳入（长期工程容器见上文）：

- 多文档容器。
- 云同步和团队协作。
- 完整项目版本历史。
- 发布包、安装器和自动更新。
- 用户账号、授权缓存和物性资产缓存的完整治理。

## 设计边界

### 数据与状态

- 项目语义：`*.rfproj.json`。
- 布局 sidecar：`<project>.rfstudio-layout.json`。
- 最近项目：Studio shell preferences。
- 授权缓存、派生包缓存和 token 不进入项目文件。

### 命令与接口

- 保存、另存为、关闭确认走 document lifecycle trigger。
- [无界面 CLI](../reference/headless-cli.md) 只读取已有项目并修改本次内存副本，不派发保存、不写 sidecar / 偏好或认证缓存；响应中的修订与写入 receipt 不表示工程已经落盘。
- 项目修改通过语义化 document command 更新 revision。
- 运行控制和文档生命周期不进入 undo history；文档历史只在当前会话保留，重开不恢复 Undo / Redo 栈。
- 保存写入已提交项目输入及已应用呈现配置，不自动应用 Inspector 草稿；单元显示名称随项目保存，单元位置按稳定 ID 写入 sidecar。
- 项目保存成功后再同步当前单元位置；布局写入失败单独警告并允许重试，不回退已经保存的工程文档。

### UI 与交互

- Home 可以显示当前工作区和最近项目，但不把未保存项目伪装成已持久化路径。
- 关闭确认必须覆盖窗口关闭和 `Cmd+Q`。
- 另存为覆盖已有文件必须显式确认。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 当前生命周期冻结 | 已有保存 / 重开 / 关闭确认路径稳定 |
| M2 | 主路径 smoke | 和 Studio 主路径专题一起记录保存 / 重开 / rerun 验收 |
| M3 | 多文档评估 | 若需要多文档，另开架构专题 |

## 验收标准

- 保存不会半写入污染项目文件。
- 另存为目标存在时必须确认。
- 脏改关闭和 `Cmd+Q` 走同一确认路径。
- 重开后项目语义不依赖 UI 私有缓存。
- sidecar 不污染项目语义。

## 验证计划

- focused test：保存 / 另存为 / 脏改确认 / 最近项目 / sidecar 恢复。
- 仓库级：阶段收口执行 `./scripts/check-repo.sh`。
- 人工 smoke：涉及原生窗口关闭、文件选择器或跨平台路径时执行。

## 状态记录

- 当前状态：Active
- 最近更新：2026-10-07，保存快捷键与模态隔离修复、失败重试及迟到关闭等 macOS 增量证据见 [W41](../devlogs/2026-10/2026-W41.md)；I5 未完整收口。历史证据见 [W40](../devlogs/2026-09/2026-W40.md) / [W37](../devlogs/2026-09/2026-W37.md)。
- 下一步：按 I5 已确认触发条件补齐剩余迟到事件、失败事务与原生组合；真实权限故障、Windows / Linux 新增路径保持待验证，Linux 项目选择器实现另行定范围。

## G0/S1 工程内容保存方案（2026-10-11，已认可）

按 [G0 首批逻辑契约](engineering-basis-and-cases.md#g0-首批数据契约2026-10-11已认可)与 [E0/S0 范围](equipment/separator-sizing-and-rating.md#e0s0-首批方法方案2026-10-11已认可)定义完整持久化方向。2026-10-11 已按获准三批方案完成独立基础子集；2026-10-15 已依次完成设备数据 / 纯计算、正式事务及结果资格，并获准实现完整设备保存与最小 Studio v3 生命周期；设备编辑页、全局设备 Undo、计算表导出和多工况留后续。

- 新增工程基础、来源与单工况设备任务属于正式项目数据，不落布局 sidecar。现有 `ProjectFileWire` 使用严格字段检查，已认可新增外层 v3，保留已有 document / presentation 语义，并增加明确的工程区；metadata 的版本含义不借机更改。
- v1/v2 项目无工程内容时保持原打开 / 保存路径；首次保存工程内容须显式升级或另存为，提示旧程序不能读取。取消 / IO 失败保留正式输入、草稿及保存脏状态；不能静默丢工程区回存 v2。
- 工程区只保存稳定 ID、基础 / 来源、任务 / 工况 / 字段绑定、方法版本与准则，不保存活动候选、问题缓存、执行令牌或未提交草稿。重开校验引用与方法能力，结果 missing；未知方法保留数据、只读说明并禁止计算，未知外层版本明确拒绝，不做容错改写。
- 设备正式输入与来源变更加入项目保存脏状态及既有离开保护；设备计算输入代次独立于 `FlowsheetDocument.revision`，无关设备编辑不旧化流程结果。Undo / Redo、会话重建及 late result 的行为由 G0 拥有。
- S1 的 CSV / JSON 计算表是显式导出的冻结记录，捕获同一有效输入、来源、方法、单位、结果与未完成项；不是项目恢复缓存，也不承诺导入回放。首批不增加多文件工程容器、二进制资产、云上传或永久计算历史。

v3 字段 / 缺省 / 必填 / 未知字段规则和迁移样本已冻结，覆盖 v1/v2 无损读取、升级取消、失败原文件恢复、缺引用 / 重复 ID、未知方法、保存重开及结果 missing；复用现有 staged write，不另造保存路径。新 CLI 推荐或设备协议不属于本批；旧 CLI v1/v2 遇所有 v3 文件，在任何操作前明确返回不支持的项目能力，且不得写文件；旧 v1/v2 输入、SI 与退出行为保持。随后有真实消费者时再审定新设备协议，不能吞掉工程区后另存。GE-D04 已认可，下节细化版本契约；已接通完整设备存储及最小 Studio 生命周期，验证范围见下节；I5 和对外候选前限制保持。

### v3 字段与兼容契约（2026-10-11）

以下为已认可 GE-D04 内的存储契约。旧 `StoredProjectFile` 常量仍为 2；Studio 通过独立 v3 DTO 分派，不扩大旧 CLI 能力。采用 UTF-8 JSON、camelCase，保持扩展名及 `kind=radishflow.project-file`；外层 `schemaVersion=3`，`document.metadata.schemaVersion=1`。先按外层版本分派，再交相应 DTO 验证，不能先把 v3 强行读进 v2 严格结构。读取始终不写盘。

| 位置 | 必需字段 / 规则 |
| --- | --- |
| 外层 | `kind`、`schemaVersion`、`document`、`presentation` 必需；`engineering` 可缺省，缺省表示空工程区；显式 null 拒绝。所有外层未知 / 重复键拒绝 |
| `document` | 原 v2 语义和字段保持；无设备专用虚构 unit / stream，不改变流程 revision 或 metadata 版本 |
| `presentation` | 只含 `displayUnits`，每项 `quantityId/unitId`；完整 15 类，各恰一次，写出按固定目录顺序。v2 始终只有原 7 类，详见 [单位契约](units/catalog-and-conversion.md#g0s1-量与单位契约2026-10-11) |
| `engineering` | 出现时必须包含 `basisEntries`、`sources`、`tasks` 三个数组，可为空；已知结构严格拒绝未知字段，不再增加同义版本号 |
| `basisEntries[]` | `id`、`revision`、`name`、`quantityId`、`valueSi`、`sourceId` 必需；量种来自统一目录，valueSi 有限；来源与适用条件从 source 引用 |
| `sources[]` | `id`、`revision`、`kind`、`title`、`reference`、`locator`、`publicationVersion`、`accessedOn`、`adoptionNote`、`review`、`applicability`、`observations` 必需；未取得的文本可为空，不能当作依据充分 |
| `tasks[]` | `id`、`revision`、`tag`、`name`、`kind`、`method`、`case`、`inputs`、可选 `axialSections`；S1 的 kind 为 `verticalGasLiquidSeparatorSizing`，不含流程 unitId |
| `method` | `id=radishflow.separator.vertical-sb-working-volume`、`version=1`；ID 和版本共同标识全部输入语义、公式及适用域，不按名称猜版本 |
| `case` | `id`、`name`、`purpose`、`fluidSystem`、`applicabilitySourceIds`、`applicability` 必需；文本可空，适用性声明按下表逐项保存 |
| `inputs` | 仅接受本节列明的十个字段键；字段缺省表示未填写，可保存但计算阻挡；存在时必须是完整绑定，不接受 null 或数字字符串 |
| `axialSections` | 可缺省；出现时是下列区段键到 length 绑定的映射，允许不完整且总高度未确定；不保存派生 h_work 或 H_TT |

全部实体 ID 非空、大小写敏感、项目内按记录种类唯一；case ID 在项目任务间唯一，数组顺序不构成身份。revision 为非负整数，保存记录版本而非执行凭据，变更单调推进。task/tag 唯一性按 G0；重复键、重复 ID、悬空引用、绑定量种不兼容或引用环拒绝装载，不丢掉坏项后打开。错误提供 JSON 路径及原因。有限但不符合设备方法范围的值可作为有问题的正式输入保留并定位，禁止产生候选；NaN/Inf、非法 JSON 和不完整绑定是格式错误。

十个输入键固定为 `operatingTemperature`、`operatingPressure`、`gasActualVolumeFlow`、`liquidActualVolumeFlow`、`gasMassDensity`、`liquidMassDensity`、`effectiveK`、`gasUtilization`、`workingDuration`、`maximumWorkingLiquidHeight`，映射见单位契约。六个可选轴向区段为 `bottomTangentToLLL`、`nllToHLL`、`hllToInletCenter`、`inletCenterToInternalsBottom`、`internalsThickness`、`internalsTopToTopTangent`，按 S0 定义解释，不按字段顺序重复加工作液层。具体内件和有效面积适用性说明必须由 case 的依据及 K 来源承载，不能只凭一个总括标签放行。

`case.applicability` 的六项枚举字段均必需；显式 `null` 表示未确认，缺字段是格式错误。已声明但不支持的值必须原样保存，不退化为“已确认 / 未确认”：

| 字段 | 允许的非 null 值 |
| --- | --- |
| `orientation` | `vertical`、`horizontal` |
| `gasDirection` | `upward`、`other` |
| `service` | `cleanNonFoamingTwoPhase`、`singlePhase`、`foaming`、`solids`、`slugging`、`threePhase` |
| `geometry` | `fullEqualDiameterCylinder`、`reducedEffectiveArea`、`nonCylindrical` |
| `kBasis` | `effectiveWholeVesselForDeclaredInternals`、`mistEliminatorOnly`、`unmatchedInternals` |
| `flowBasis` | `actualAtOperatingConditions`、`standardConditions` |

另有必需字符串 `internalsDescription`，可空。10 月 11 日设计草案中的 `twoPhasesDeclared` / `configuration` 不能表达完整领域声明，未作为设备产品写出；检测到旧 `case.configuration` 结构时整份只读，不推断或自动迁移。方法 ID / 版本和数值算法保持不变。

字段绑定采用带标签的联合，字段集合严格如下；quantityId 在 Direct/Override 值处必需，不从单位猜量种：

```json
[
  {"kind":"direct","quantityId":"duration","valueSi":120.0,"sourceId":"src-assumption"},
  {"kind":"basisRef","basisEntryId":"basis-working-time"},
  {"kind":"override","basisEntryId":"basis-working-time","quantityId":"duration","valueSi":180.0,"sourceId":"src-override","reason":"独立工况明确覆盖"}
]
```

上述仅演示三种编码，不是可计算工程样本。BasisRef 只指基础条目，不能指其他绑定；Override 必须保留同量种的原基础引用、本地来源和非空理由，不改原基础值。来源日期 accessedOn 为空或 ISO 8601 日期（YYYY-MM-DD），publicationVersion 保留来源发布日期或版本原文；review 变更须推进源 revision。来源 kind 枚举 `measured/vendor/literature/estimated/assumed/unknown`；review 枚举 `unreviewed/adopted/reviewed`。分类与 review 不自动证明可信度；unknown 可保存但不允许计算。

`applicability` 包含必需 `fluidSystem`、`phase`、`operatingConditions`、`scope`，phase 为 `gas/liquid/both/notApplicable`；operatingConditions 为空对象表示未记录，或者含 `temperatureK` 与 `pressurePa` 两个有限正数，必须成对出现，不从 task 自动复制证明匹配。流量 / 密度要求同一物系与工况；准则用 scope 记录适用范围、内件与折减，无法机器确认的条件必须保留审阅说明。

`observations` 是原始量记录数组，每项含 `id`、`quantityId`、`originalValue`（有限数）、`originalUnit`（来源原文）、`canonicalValue`（有限 SI）、`conversionNote` 及 `dependsOn`。dependsOn 为 `{sourceId, observationId}` 引用数组，支持有限的可追溯派生链，禁止环；不是可执行公式或第二套自动换算引擎。originalUnit 可保留不支持的外部单位，但只有统一目录可换算的值或注明外部方法并显式采用的 canonicalValue 才能使用，不从文本执行表达式。缺 observation 不等于缺 source：假设可仅以 adoptionNote 说明，分类始终为 assumed。引用绑定以 sourceId 关联当前来源，计算时捕获具体源 revision 与观察记录，不把捕获写回成输入。

不保存计算 / 检查结果、候选、输入草稿、session ID、计算代次或执行 token。任务 / 来源持久 revision 不充当跨会话执行资格；重开建立新会话和运行时代次，设备候选 missing，显式重算。保存脏标记比较正式 document、presentation、engineering 各自已保存基线；仅设备变更不增加流程 revision。

### v3 基础存储实施范围（2026-10-11）

10 月 11 日先独立完成来源 / 基础、15 类显示单位及空任务往返，非空任务整份只读，产品入口继续拒绝 v3。这是基础阶段的历史范围；10 月 15 日扩展为下节完整设备存储，不建立第二种简化 v3。

### 设备事务阶段的保存保护（2026-10-15）

正式事务阶段先拒绝非空工程区进入旧格式保存，防止尚未持久化的输入被丢弃；该临时阻挡已由下节真实 v3 写入及保存记账替代。流程、呈现、设备分别管理保存基线，项目离开保护继续核验设备编辑 checkpoint。

### 完整设备保存与最小 Studio 生命周期（2026-10-15）

- `rf-store::project_v3` 的 `StoredProjectV3` / `StoredEngineeringInputs` 保存完整基础、来源、任务、单工况、适用性、十项输入及可选轴向区段。缺项、未确认及有限但超适用域的输入可保存，计算门禁仍由设备方法负责；不伪造 K、停留时间或适用声明。
- 已知 kind / method / version 才解码完整任务；严格结构与重复键检查从原始文本执行，随后校验身份、位号、绑定量种、引用及来源链。任何未知方法或旧简化 case 结构均保留整份原始 UTF-8 JSON，禁止序列化和覆写；未知 payload 不经过数值转换，保留大整数、长小数及指数文本。
- Studio 以 `read_studio_project` 分派：已知 v3 恢复正式输入、单位和保存基线，创建新会话，候选 missing、历史为空；未知能力显示独立整份只读检查，隐藏编辑 / 计算 / 保存入口，拦截快捷键。检查期间不将文件部分装入可编辑 host，关闭检查返回原工作区；未知外层版本或格式错误保留原工作区并报错。
- 现有 v3 始终保存 v3。旧项目有非空工程区或新增 8 类量的非 SI 显示选择时，提示升级 / 另存为 / 取消；无新增内容保持既有 v1/v2 规则。确认捕获项目离开 checkpoint 与目标版本，迟到确认失效。`write_upgraded` 仅允许显式升级合法旧文件，普通 v3 写入拒绝隐式覆盖旧文件；v3 写入均拒绝未知能力目标或损坏文件。旧保存路径也不得覆盖 v3。
- 仍使用现有 staged write。写入成功后才将捕获的正式设备输入记为已保存基线，不清除历史、候选或草稿；迟到保存记账不能清除后续修改。取消及 IO 失败保持路径、输入、历史与脏状态。保存后 Undo / Redo 按内容重新比较基线，执行代次不回退；重开不恢复旧 token 或结果。
- 旧 CLI 的 inspect / run / workflow 继续在建模或求解前拒绝所有实际 v3 文件，包括省略 / 空工程区；旧 SI 协议和 v1/v2 行为不变。设备编辑页、全局设备 Undo 路由、计算表导出及多工况未接入本批。

### v3 结构样本与版本行为

以下是可直接放入 v3 外层的 **engineering 子树**，示范可保存的不完整独立任务；没有 T/P、流量、密度或 K，必须返回缺项而不能计算。它不属于 S0-B04，也不替代有来源的工程方法验证。构造完整文件时复制任一合法旧样本的 document，外层版本改为 3，presentation 保留原 7 类再添加单位契约中 8 类的 SI 项；这是测试构造说明，不授权改写现有用户工程。

```json
{
  "basisEntries": [],
  "sources": [],
  "tasks": [{
    "id": "equipment-1",
    "revision": 0,
    "tag": "V-101",
    "name": "待补依据的独立任务",
    "kind": "verticalGasLiquidSeparatorSizing",
    "method": {"id":"radishflow.separator.vertical-sb-working-volume","version":1},
    "case": {
      "id":"case-1", "name":"工况一", "purpose":"初步尺寸",
      "fluidSystem":"", "applicabilitySourceIds":[],
      "applicability": {
        "orientation":null, "gasDirection":null, "service":null,
        "geometry":null, "kBasis":null, "flowBasis":null,
        "internalsDescription":""
      }
    },
    "inputs": {}
  }]
}
```

未知外层版本拒绝打开且不写盘。已知 v3 内未知 task kind / method ID / method version 不按版本 1 解码其 payload：保留原始任务数据，提供只读检查并禁止计算。首批对此文件采用整个工程只读模式，避免不知道依赖关系却更改其基础 / 来源；原文件始终不覆写，不提供剥离未知任务的“兼容保存”。这属于未知能力保护，不能显示已恢复可编辑任务。已知方法的未知字段则为格式错误，防止拼写错误被当作扩展吞掉。

旧 CLI v1/v2 对所有 v3 文件在建模、求解或写入前返回不支持项目版本 / 能力，沿用其结构化错误通道；即使工程区缺省或为空也不降格消费。新设备协议另行审定。无工程区且新增 8 类均为 SI 的旧文件保持原 v1/v2 保存资格；首次有工程内容或新增非 SI 呈现时，显式确认 v3 升级 / 另存为 / 取消。已有 v3 即使清空任务也不自动降级。

| 样本编号 / 构造 | 契约预期（实现覆盖见上节） |
| --- | --- |
| GS-01 原 v1、原 v2 非 SI | 读取保持 document 身份 / 数值；v1 无 presentation，v2 恰 7 项；不因新目录失效或升级；原 CLI 结果一致 |
| GS-02 上述 v3 不完整任务 | 可保存 / 重开，十项缺项明确，候选 missing，绝不补 K / 时间；document/flowsheet 无虚构对象 |
| GS-03 增加来源、基础和三种绑定 | 来源链与一层覆盖无损往返；修改共享基础影响引用任务，Override 不反写；重开不恢复候选 |
| GS-04 删除一项显示单位、重复 ID / 键、null、悬空来源、循环观察引用 | 各自有具体字段路径错误，原文件不变；不按合法部分装载 |
| GS-05 method.version 改为未知值；增加未来 payload 字段 / 大整数 | 原始任务不丢字段或数字精度，工程只读、不可计算 / 覆写；不作为 v1 执行 |
| GS-06 已知 method 添加拼错字段；未知外层 4 | 已知字段严格拒绝；外层不支持明确拒绝；均不写盘、不迁移 |
| GS-07 v2 新建设备后选择取消、升级、另存为 | 取消保留输入 / 草稿 / 脏状态；升级成功才推进保存基线；另存为失败不切当前路径 |
| GS-08 staged write / flush / replace 故障 | 按既有故障注入验证原文件可恢复且不被半写覆盖；失败不清脏，不丢工程数据 / 草稿 |
| GS-09 工程输入、来源、呈现分别变更并 Undo / Redo | 保存基线与数值 / 证据身份正确；不旧化无关流程结果；设备旧 token 不复活 |
| GS-10 CLI 收到有 / 无 engineering 的 v3 | 在任何求解 / 建模 / 文件修改前失败，错误明确；旧 SI DTO 与 v1/v2 样本行为不变 |

GS-01—10 的存储 / 生命周期自动回归已落实：完整字段与三种绑定、不完整 / 超适用域输入、严格结构与引用、未知方法原文、显式升级取消 / 迟到确认 / 成功、IO 失败不清脏、保存后 Undo / Redo、重开 missing 与旧 CLI 预操作拒绝。GS-08 继续复用 staged-write 故障注入，新增生命周期真实 IO 失败验证；设备草稿控件及全局 Undo 随后续 UI 验收。macOS 已实测已知 v3 原生打开、中文路径另存为 / 重开、未知方法只读及快捷键保护；升级对话框通过 shell 自动回归，未宣称设备编辑路径实窗完成。证据见 [W42](../devlogs/2026-10/2026-W42.md)，I5、Windows / Linux 未验路径及发布限制保持。
