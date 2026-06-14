# 项目物性基础与组分选择

更新时间：2026-06-14

## 用途

用途：定义项目级物性 package、项目组分选择、保存 / 重开和运行请求之间的事实源与开发边界。  
读者：负责物性页、项目文件、运行 package resolution、Home / 项目输入展示和相关测试的开发者、用户、AI / Agent。  
不包含：完整组分数据库、第三方 Property Package 加载、完整 Thermodynamics PMC、物性参数拟合和高级物性分析实现。

## 专题目标

- 项目物性状态只有一个事实源：项目文档中的稳定 package id、项目组分和当前可选内置 package / component choices。
- UI 展示可读 label，项目文件、command id 和运行请求保留稳定 id。
- 空白项目必须显式选择物性 package 和项目组分后再进入建模主路径；official 示例必须持久化自己的 package 选择。

## 子专题关系

- 当前内置 package 与项目组分选择留在本专题。
- 远端授权、派生资产清单、下载票据和离线租约属于 `platform/control-plane-service.md`。
- 后端管理台页面属于 `platform/control-plane-web-ui.md`。
- 若未来推进完整组分数据库、物性分析或第三方 package 加载，应新增独立二级专题，不继续扩写本文。

## 当前实现快照

已完成：

- 普通空白项目初始不预选 package 或项目组分。
- 独立 `物性` 页消费 `StudioGuiWindowPropertyPageModel`。
- `binary-hydrocarbon-lite-v1` 和 methane / ethane 已作为当前受控内置路径。
- Home recent / current / example tile、左侧 `项目输入` 和独立 `物性` 页展示可读 package label。
- official hydrocarbon 示例项目文件已持久化 `binary-hydrocarbon-lite-v1`。

已知缺口：

- 完整物性页长期空间尚未拆成可实现专题。
- 物性分析、参数来源、完整组分查询仍只是未来空间。
- 后续若引入多个内置 package，默认选择策略和冲突提示需要单独补设计。

## 用户路径

1. 新建空白项目后进入 `物性`。
2. 选择内置 package。
3. 选择项目组分。
4. readiness 变为可进入流程图建模。
5. 保存并重开项目后仍显示同一 package label 和项目组分。
6. 运行时使用项目文件中的稳定 package id，而不是从 UI 文案或结果反推。

## 范围

本专题纳入：

- 内置 package / component 选择。
- 物性页 readiness。
- Home、左侧项目输入、Property toolbar 和 package 选择卡的同源展示。
- 保存 / 重开后的 package id 与 label 映射。
- official 示例项目的物性状态一致性。

本专题不纳入：

- 第三方 Property Package。
- 完整 Thermodynamics PMC。
- 完整组分数据库。
- 物性参数拟合、相图、Txy / Pxy 曲线和来源管理实现。
- 远端高价值物性资产分发，除非另开授权 / 资产专题。

## 设计边界

### 数据与状态

- 稳定事实源：`flowsheet.thermo.property_package_id` 与项目组分。
- 展示事实源：当前 document / builtin choices 映射出的 localized label。
- 不从运行结果、Home cache 或 UI 文案反推 package 状态。
- 空白项目显示“未选择”是合法状态，不自动伪造默认 package。

### 命令与接口

- package / component 修改必须走正式 command 或 GUI host dispatch。
- 进入流程图建模必须服从 `StudioGuiWindowPropertyPageModel::flowsheet_modeling_enabled`。
- 运行 package resolution 继续作为求解前正式校验，不被 UI readiness 替代。

### UI 与交互

- `物性` 是独立 screen。
- 当前 MVP 不渲染无操作价值的物性二级导航。
- UI 主文案显示可读 label；稳定 id 只用于项目文件、command id、运行请求和内部状态。

## 分阶段切片

| 阶段 | 目标 | 退出标准 |
| --- | --- | --- |
| M1 | 当前口径冻结 | Home、Property、项目输入和 official 示例同源展示 package label |
| M2 | 多内置 package 准备 | 明确多个内置 package 时的显式选择、冲突提示和保存语义 |
| M3 | 物性页长期空间拆分 | 若推进组分查询、参数或分析，先新增独立专题 |

## 验收标准

- 空白项目新建后 package 和项目组分为未选择状态。
- 选择 package / component 后才能进入流程图建模。
- 保存 / 重开后 package id 与 UI label 一致。
- official 示例不会出现“运行已收敛但物性包未选择”的冲突。
- raw package id 不作为主展示文本外露。

## 验证计划

- focused test：Property readiness、package command dispatch、保存 / 重开、Home / Property / 项目输入展示映射。
- 仓库级：阶段收口执行 `./scripts/check-repo.sh`。
- 人工 smoke：当新增物性页 UI 或多个 package 选择行为时，复核 Home、Property、Workbench 三处展示。

## 状态记录

- 当前状态：Active
- 最近更新：2026-06-14 从 Studio UI 流水中拆出物性基础专题。
- 下一步：只在新增 package / component 行为或展示冲突时推进；完整物性数据库另开专题。
