# Run First Flowsheet

更新时间：2026-05-13

## 目的

本文档面向第一次实际运行 `RadishFlow Studio` 的读者，目标是用仓库内置示例走通一次最小求解闭环。

推荐示例：

- `examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json`

这个示例的好处是结构简单，同时已经覆盖：

- 流股输入
- 单元串联
- `Flash Drum` 两相分离
- 结果、步骤和诊断审阅

## 0. 前提

建议当前工作区至少可通过最小构建检查：

```powershell
cargo check --workspace
```

如需完整仓库级验证，执行：

```powershell
pwsh ./scripts/check-repo.ps1
```

然后启动 Studio：

```powershell
cargo run -p radishflow-studio
```

## 1. 打开示例项目

启动后，优先使用顶部快速操作区打开项目：

- `Open Example` 内置示例入口
- `Open Project` Windows 原生文件选择器
- `Command Palette` 中的打开项目相关命令

当前仍保留以下辅助入口：

- 最近项目列表
- 手工输入项目路径

第一次建议直接打开：

```text
examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json
```

如果你更想先看更复杂一点的路径，也可以改用：

```text
examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json
```

## 2. 确认项目内容

成功打开后，当前项目应至少包含：

- `Feed`
- `Heater`
- `Flash Drum`
- `stream-feed`
- `stream-heated`
- `stream-liquid`
- `stream-vapor`

此时你通常可以在 Canvas 和 Inspector 中先确认：

- `Feed.outlet -> Heater.inlet`
- `Heater.outlet -> Flash Drum.inlet`
- `Flash Drum.liquid / vapor` 两个出口流股已存在

## 3. 触发运行

优先点击顶部快速操作区的 `Run`。

`Run` 会派发正式 `run_panel.run_manual` 命令。若按钮不可用，先看 hover 说明、Runtime 摘要和 Diagnostics，而不是反复点击。需要从完整命令面执行时，可打开 `Command Palette` 或显示 `Commands` 面板后查找运行命令。

成功时，当前工作流应从“项目已打开”推进到“已有一份最新 `SolveSnapshot`”。  
失败时，也应至少得到结构化诊断，而不是只有无上下文报错文本。

## 4. 查看结果

第一次运行后，优先看这四个位置：

1. Result Inspector 的 stream-centric 视图
2. Result Inspector 的 unit-centric 视图
3. 求解步骤与诊断
4. Active Inspector

当前你应该特别注意以下结果字段：

- `T / P / F / H`
- `phase_region`
- `bubble_dew_window`
- `liquid / vapor / overall` 相结果

其中：

- `T` = `temperature_k`
- `P` = `pressure_pa`
- `F` = `total_molar_flow_mol_s`
- `H` = `molar_enthalpy_j_per_mol`

按对象读时，建议再多看一层：

- 先看 source stream，例如 `stream-feed`
- 再看 non-flash intermediate，例如 `stream-heated`
- 再看 step 输入/输出是否与全局流股结果保持同一份 DTO 语义
- 最后再看 `stream-liquid / stream-vapor` 这类 flash outlet 的窗口边界和缺席语义

如果你想按这条顺序系统地读一遍结果，继续阅读：

- `docs/guides/review-solve-results.md`

## 5. 保存并重新打开

如果只是验证运行链路，建议再顺手做一次保存与重开：

- 点击顶部快速操作区的 `Save`
- 或通过 `Command Palette` / Commands 面板执行 `file.save`
- 或执行 `Save As` 到新的 `*.rfproj.json` 路径

当前项目保存涉及两类文件：

- 项目真相源：`*.rfproj.json`
- Canvas 布局 sidecar：`*.rfstudio-layout.json`

其中：

- `*.rfproj.json` 保存流程语义、参数、连接和文档元信息
- `*.rfstudio-layout.json` 保存 Canvas placement 等 shell / layout 相关状态

如果当前项目已有 Canvas placement，保存并重开后应能恢复这份 sidecar 状态。

## 6. 常见阻塞点

如果当前示例没有直接跑通，优先检查以下几类问题：

- Stream Inspector 中是否还有未提交或无效草稿
- 文档态流股组成是否未归一到 1
- 当前运行环境下是否出现多包可选且未显式指定 package
- 项目是否被改成了不完整连接或不一致端口绑定
- 顶部 `Run` 是否处于 disabled 状态，以及 hover 文案给出的原因
- 启动 Studio 的终端 stderr 是否有 `[radishflow-studio]` 审计线或 GUI panic 提示

当前系统的默认处理原则是：

- 不做隐式自动补偿
- 不在运行前偷偷改写文档
- 优先通过结构化诊断暴露问题
- 开发态 stderr 只作为 smoke 和排查辅助，不替代 Runtime / Diagnostics 中的用户可见诊断

组成相关提示可按下面理解：

- `Draft`：当前输入值还只是草稿，尚未写入项目文档
- `Unnormalized`：组成已经写入文档，但总和不是 1
- `Normalize composition`：显式把当前组成归一化；它不会代替用户猜测新增或删除组分
- `Remove` / add component：只在当前 flowsheet 已有组件目录内操作，不触发项目级组件迁移

## 7. 下一步建议

如果这次运行已经走通，下一步建议按下面顺序继续：

1. 修改一个流股字段并重新运行
2. 在 Result Inspector 对比不同流股结果
3. 阅读 `docs/guides/review-solve-results.md`
4. 阅读 `docs/reference/units-and-conventions.md`
5. 阅读 `docs/reference/solve-snapshot-results.md`
6. 阅读 `docs/thermo/mvp-model.md`

如果你接下来更关心系统边界，而不是继续点 UI，则直接转到：

- `docs/architecture/overview.md`
