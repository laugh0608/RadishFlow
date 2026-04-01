# RadishFlow MVP 开发路线图

更新时间：2026-03-31

## 文档目的

本文档用于把 `RadishFlow` 的第一阶段目标拆成可执行的开发路线。

本文档只覆盖 MVP，不覆盖以下内容：

- 第三方 CAPE-OPEN 模型导入
- 完整 Thermo PMC
- recycle 全功能收敛
- 复杂 GUI 风格打磨
- 大规模组件数据库

## MVP 目标声明

MVP 的目标固定为：

构建一个以 Rust 为核心、以 Rust UI 为主界面、以 .NET 10 暴露 CAPE-OPEN Unit Operation PMC 的最小稳态流程模拟闭环，并让至少一个自有单元模型可被外部 PME 识别与调用。

补充系统口径：

- 桌面端主线固定为 Rust 客户端，负责 UI、求解、项目文件读写与本地授权缓存使用
- 认证、授权、离线租约、受控物性包清单与下载票据由外部 `ASP.NET Core / .NET 10` 控制面承担
- 派生资产下载优先走对象存储 / CDN / 下载网关，不把控制面 API 设计成长期大文件出口
- 桌面端最终交付形态默认为“压缩包展开后直接运行”，不以单文件可执行产物为阶段目标

## MVP 功能边界

MVP 建议只包含以下能力：

- 二元体系
- 最小物性参数集
- 简化热力学模型
- `TP Flash`
- 流股对象
- 单元模块：`Feed`、`Mixer`、`Heater/Cooler`、`Valve`、`Flash Drum`
- 无回路流程或极简回路
- JSON 项目格式
- 一个最小 Rust 桌面工作台
- 一个可注册的 CAPE-OPEN Unit Operation PMC

## 不做项

MVP 阶段明确不做：

- 加载外部 CAPE-OPEN 单元
- 加载外部 CAPE-OPEN 物性包
- 完整 Thermodynamics 1.0/1.1 主机兼容
- 完整物性数据库
- 动态模拟
- 复杂报表系统
- 多套热力学包切换

## 总体里程碑

建议把 MVP 分成 5 个里程碑：

1. `M1`：仓库与基础骨架初始化
2. `M2`：二元体系 `TP Flash` 核心跑通
3. `M3`：最小稳态流程闭环跑通
4. `M4`：Rust FFI 与 .NET 10 适配层打通
5. `M5`：外部 PME 识别并调用自有 PMC

## M1：仓库与基础骨架初始化

### 目标

建立 `RadishFlow` 新仓库与最小开发骨架。

### 任务

- 创建 `RadishFlow` 新仓库
- 初始化 Rust workspace
- 创建核心 crates
- 创建 `apps/radishflow-studio`
- 创建 `.NET 10` 适配层解决方案目录
- 冻结外部 `.NET 10` 控制面的职责边界与最小 API 契约
- 初始化根文档与 `docs/README.md`
- 建立基础脚本目录与测试目录

### 建议优先创建的 crate

- `rf-types`
- `rf-model`
- `rf-thermo`
- `rf-flash`
- `rf-unitops`
- `rf-flowsheet`
- `rf-solver`
- `rf-ffi`

### 退出标准

- 新仓库可正常克隆
- Rust workspace 可成功 `cargo check`
- .NET 适配层目录已存在并可打开
- 文档已说明 MVP 边界与目录职责
- 外部 `.NET 10` 控制面边界与桌面客户端职责已经冻结

## M2：二元体系 TP Flash 核心跑通

### 目标

先打通最小的热力学与闪蒸核心。

### 任务

- 定义组分与基础物性结构
- 支持二元体系输入
- 实现 Antoine 饱和蒸气压
- 实现简化 `K` 值逻辑
- 实现 Rachford-Rice
- 实现 `TP Flash`
- 输出相分率、汽液相组成、基础焓值
- 建立黄金样例测试

### 推荐 crate 分工

- `rf-types`：基础枚举、ID、相标签
- `rf-model`：流股状态对象
- `rf-thermo`：组分参数与热力学模型
- `rf-flash`：闪蒸算法

### 退出标准

- 至少一个二元样例可稳定计算
- 数值结果可回归测试
- API 已可供单元模块调用
- `tests/thermo-golden` 与 `tests/flash-golden` 已建立首批黄金样例

## M3：最小稳态流程闭环跑通

### 目标

构建最小 flowsheet 求解能力。

### 任务

- 定义端口与连接关系
- 实现 `Feed`
- 实现 `Mixer`
- 实现 `Heater/Cooler`
- 实现 `Valve`
- 实现 `Flash Drum`
- 实现流程图完整性校验
- 实现顺序模块法求解
- 建立至少一个最小流程示例
- 形成至少一个可从 `*.rfproj.json` 加载到求解输出的端到端闭环样例

### 推荐示例流程

- `Feed -> Heater -> Flash Drum`
- `Feed -> Valve -> Flash Drum`
- `Feed1 + Feed2 -> Mixer -> Flash Drum`

### 推荐 crate 分工

- `rf-unitops`
- `rf-flowsheet`
- `rf-solver`
- `rf-store`

### 当前收口口径

- `rf-unitops` 第一轮统一接口先围绕标准 `MaterialStreamState` 输入输出与必要热力学服务注入
- `Mixer` 当前先固定为两进一出，`Flash Drum` 当前先固定为一进两出，优先建立第一条可求解闭环
- `Heater/Cooler` 与 `Valve` 当前先沿用“一进一出”最小接口，并以 outlet 流股模板承载当前阶段目标状态设定
- `rf-flowsheet` 第一轮只做 canonical material 端口签名、流股存在性与“一股一源一汇”校验，拓扑排序和顺序模块调度继续留给 `rf-solver`
- `rf-solver` 第一轮先只支持无回路、内建单元和标准材料流股执行，当前已覆盖 `Feed1 + Feed2 -> Mixer -> Flash Drum`、`Feed -> Heater -> Flash Drum` 与 `Feed -> Valve -> Flash Drum`
- `rf-solver` 当前已补最小求解诊断层：`SolveSnapshot` 至少包含 summary、仓库级 diagnostics 和逐步执行 step 明细；失败路径至少带 step 序号、unit id / kind 与 inlet stream 上下文
- `examples/flowsheets` 当前应维护至少三条可直接从 `*.rfproj.json` 载入并求解的示例项目，作为内核闭环回归基线
- `tests/rust-integration` 当前应作为仓库级 Rust 集成测试入口，覆盖“加载项目 -> 求解 -> 读取结果”的示例流程回归

### 退出标准

- 能打开一个 JSON 示例流程
- 能完成一次完整求解
- 能输出每股流体的基本状态结果
- 已具备最小集成测试，能覆盖“加载项目 -> 求解 -> 读取结果”的端到端数据流
- 仓库级验证入口 `scripts/check-repo.ps1` / `scripts/check-repo.sh` 已自动覆盖这些集成测试

## M4：Rust FFI 与 .NET 10 适配层打通

### 目标

让 Rust 核心可被 .NET 10 适配层调用。

### 任务

- 定义稳定的 C ABI
- 设计句柄式对象生命周期
- 处理字符串与错误码返回
- 定义最小 JSON 快照接口
- 在 `.NET 10` 中建立 `RadishFlow.CapeOpen.Adapter`
- 完成 PInvoke 封装
- 建立最小互调测试

### 推荐暴露接口

- `engine_create`
- `engine_destroy`
- `flowsheet_load_json`
- `flowsheet_solve`
- `stream_get_snapshot_json`
- `unit_run`

### 退出标准

- .NET 可成功调用 Rust 动态库
- 能从 .NET 获取计算结果
- 有最小自动化测试覆盖对象创建与求解调用

## M5：外部 PME 识别并调用自有 PMC

### 目标

交付第一版可互操作的 CAPE-OPEN Unit Operation PMC。

### 任务

- 建立 `RadishFlow.CapeOpen.Interop`
- 建立 `RadishFlow.CapeOpen.UnitOp.Mvp`
- 设计最小参数与端口映射
- 实现 `ICapeIdentification`、`ICapeUtilities`、`ICapeUnit` 相关能力
- 建立注册工具 `RadishFlow.CapeOpen.Registration`
- 完成 COM host 注册流程
- 在目标 PME 中做人工验证

### 建议验证内容

- PME 能发现组件
- PME 能实例化组件
- PME 能读取参数
- PME 能连接端口
- PME 能触发 `Calculate`
- PME 能得到有效结果或可诊断错误

### 退出标准

- 至少一个目标 PME 成功识别并调用 PMC
- 注册/反注册流程可重复执行
- 文档中已写明验证路径

## 推荐开发顺序

建议严格按以下顺序推进：

1. `M1`
2. `M2`
3. `M3`
4. `M4`
5. `M5`

不要跳过 `M2` 和 `M3` 直接做 CAPE-OPEN 外壳。  
如果内核没有先稳定，后面在 PME 里出现错误时会很难定位到底是内核问题还是 COM 适配问题。

## 当前计划对齐

截至 2026-03-29，M1 阶段中与桌面控制面接线相关的地基项已进一步前推，当前已完成：

- 控制面 `entitlement / manifest / lease / offline refresh` HTTP client
- 控制面 JSON 契约到运行时 DTO 的应用层协议映射
- 按 manifest 申请 lease、下载派生包并回写本地缓存索引的应用层编排

截至 2026-03-30，系统技术口径进一步补充冻结为：

- 远端控制面优先采用 `ASP.NET Core / .NET 10`
- 资产分发优先采用对象存储 / CDN / 下载网关 + 短时票据
- 当前仓库继续只保留客户端侧接线、DTO 与缓存编排，不把服务端代码强行并入 Rust workspace

基于当前状态，下一阶段计划进一步对齐为：

- 今天（2026-03-30）优先细化授权刷新后的 UI 事件流、联网失败提示和离线刷新触发策略
- 随后恢复 `rf-thermo` / `rf-flash` 数值主线，避免地基建设继续挤占核心算法推进
- 在数值主线恢复后，再进入 `rf-solver` 无回路顺序模块法和首个可求解 flowsheet 示例

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
| PME 兼容不确定 | 外部软件实际行为偏标准之外 | 先选一个目标 PME 做主验证 |

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
