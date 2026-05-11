# 当前状态

更新时间：2026-05-11

## 用途

本文档是新会话恢复上下文、判断“今天做什么”的轻量入口，也是当前阶段、当前重点、当前验证基线和暂不推进项的默认真相源。

默认先读本文档。只有当任务需要具体实现细节、历史依据或专题边界时，再读取下方列出的专题文档。`AGENTS.md` / `CLAUDE.md` 只保留长期协作规则，不再重复承载这类易过期内容。

## 当前阶段

- 产品定位：以 Rust Core + Rust UI + `.NET 10` CAPE-OPEN/COM 适配层构建稳态流程模拟软件。
- 当前主线：在已具备最短可运行建模路径和 Stream Inspector 组成编辑安全边界后，转回数值主线和结果审阅收口。
- 当前重点：保持 Canvas 与 Stream Inspector 不继续横向扩张，把精力放到 `SolveSnapshot` 结果消费、热力学 / 闪蒸基础能力和可验证闭环上。
- 当前验证基线：功能改动优先执行相关 focused tests；阶段性收口执行 `pwsh ./scripts/check-repo.ps1`。

## 最近已完成

- Studio Canvas 已支持最短可求解建模路径：`Feed -> Flash Drum`、`Feed -> Heater/Cooler/Valve -> Flash Drum`、`Feed + Feed -> Mixer -> Flash Drum`。
- 空白项目打开时已补 MVP 默认二元组件与本地 `binary-hydrocarbon-lite-v1` 物性包缓存，可保存后重新打开继续运行最短闭环。
- Canvas placement 坐标已保存到 `<project>.rfstudio-layout.json` sidecar；离散 nudge 只更新 sidecar，不改项目文档、不进入 undo/redo。
- Result Inspector 已补 stream-centric 与 unit-centric 两个消费面，可审阅当前快照内输入/输出流股、相结果、关联步骤和关联诊断。
- `rf-thermo` / `rf-flash` 已补 MVP 常热容显热焓值；`Flash Drum` outlet 会传递 liquid / vapor / overall molar enthalpy。
- `rf-thermo` 已补基于当前 Antoine / Raoult MVP 假设的 bubble/dew pressure 边界估算，以及 fixed-pressure bubble/dew temperature 边界估算；`rf-flash` 的 `TP Flash` 结果现在会显式物化 `liquid-only / two-phase / vapor-only` phase region 与对应 bubble/dew pressure / temperature 窗口，并已补齐 golden / focused tests。
- `tests/thermo-golden` 与 `tests/flash-golden` 现在都已覆盖 `liquid-only / two-phase / vapor-only` 三类正式金样；`rf-flash` 与 `rf-types` focused tests 也已锁定 exact bubble/dew boundary 和 tolerance 内外的 phase region 语义，避免边界漂移被集成层才发现。
- `tests/thermo-golden` 与 `tests/flash-golden` 已继续补齐 near-boundary `±ΔP / ±ΔT` 小扰动金样；当前不仅覆盖 `binary-hydrocarbon-lite-v1` 原有 `z=[0.2, 0.8]` two-phase 基线，还补到靠 bubble / dew 两侧的 `z=[0.195, 0.805]` 与 `z=[0.23, 0.77]` 两组 two-phase 组成，并继续覆盖现有 synthetic `liquid-only / vapor-only` 样例；`rf-thermo` 与 `rf-flash` focused tests 也已锁定 bubble/dew 两侧跨 boundary 前后的 phase region 与 `bubble_dew_window` 稳定行为，避免边界附近漂移只在集成层或 UI 审阅时才暴露。
- `rf-model::MaterialStreamState`、`rf-solver` 与 `rf-ui::SolveSnapshot` 现在已为 flash 产物流股正式透传结构化 `bubble_dew_window`；`Flash Drum` liquid / vapor outlet 会按各自 outlet 组成重算并携带这组窗口，而不是复用 overall flash feed 的边界。
- `Mixer`、`Heater/Cooler` 与 `Valve` 的 outlet 结果现在也会在 unit operation 层直接物化同一组结构化 `bubble_dew_window`，并通过 `rf-solver -> rf-ui::SolveSnapshot -> Result Inspector / Active Inspector` 只读透传；这层继续只消费正式 thermo DTO，不在 shell / UI 中重算或分叉第二套相平衡语义。
- `Mixer`、`Heater/Cooler` 与 `Valve` 的 flowing outlet 现在也会在 unit operation 层用同一条 `TP Flash` 参考路径补出 overall molar enthalpy，并继续只把已物化的 `overall` phase `H` 透传到 `SolveSnapshot` / workspace run path / Studio consumer，不在集成层分叉第二套 enthalpy 求值语义。
- `Feed/source stream` 的 solved path 现在也会沿同一条 `TP Flash` 参考路径同时物化 overall molar enthalpy 与结构化 `bubble_dew_window`；`flowsheet_examples`、`studio_solver_bridge`、`studio_workspace_control`、dedicated flash inlet boundary 以及 Studio window-model/runtime focused tests 当前已继续锁定 source stream `T / P / z / overall H / phase_region / bubble_dew_window` 与 first consumer consumed stream 的同一份 DTO 语义，不在 upstream solved path、workspace run path 或结果消费层分叉 / 重算第二套判断。
- `rf-solver::UnitSolveStep` 现在也会直接物化 consumed stream 快照；`rf-ui::SolveSnapshot` 对 solver step 的 consumed stream 不再按 id 二次回填或合成 placeholder DTO，`studio_solver_bridge` / `studio_workspace_control` 与 dedicated flash inlet boundary 回归继续锁定这条 `solver -> SolveSnapshot -> workspace run / consumer` 的同一份 consumed stream 语义。
- `rf-solver::UnitSolveStep` 现在也会直接物化 produced stream 快照；`rf-ui::SolveSnapshot` 对 solver step 的 produced stream 不再按 `produced_stream_ids` 二次回填，solver step 的输入/输出流当前都继续沿 `solver -> SolveSnapshot -> workspace run / consumer` 走同一份 DTO，不在集成层再留 step stream 组装 fallback。
- `rf-solver::UnitSolveStep` 当前已进一步移除内部 `consumed_stream_ids / produced_stream_ids` 并改为从结构化 stream 快照现场派生；`rf-ffi` 导出与 Studio 最新单元结果若仍需 stream id 列表，也只从同一份 consumed / produced stream DTO 派生，不再在 solver / Studio 结果面保留并行 id-only 真相源。
- `apps/radishflow-studio::StudioGuiWindowSolveStepModel` 当前也已进一步移除窗口结果面内部 `consumed_streams / produced_streams` 缓存；相关筛选、摘要和 diagnostic action 现在都直接从 `consumed_stream_results / produced_stream_results` 派生 stream id，不再在 Studio solve step presentation 再保留一份并行 id-only 列表。
- `rf-ffi` 的 demo package 与 runtime package test fixture 当前也已对齐到现行 fixed-pressure bubble/dew temperature 数值基线；先前在 `feed-1` solved path 上触发的 `Antoine correlation produced a non-finite saturation temperature` 已收敛，`cargo test -p rf-ffi` 当前恢复通过。
- `examples/flowsheets/failures` 下仍承担连接 / 恢复 / 诊断语义的历史 failure 样例当前也已统一回到 official `methane / ethane` 组件目录；`flowsheet_examples`、`studio_solver_bridge` 与 `studio_workspace_control` 中直接消费这些 failure fixture 的路径，以及相关 cached-package helper，当前都已显式切到 `binary-hydrocarbon-lite-v1` official hydrocarbon 语义，不再借由旧 `component-a / component-b` synthetic package 混过去。
- `tests/rust-integration` 中旧 `feed-*-flash.rfproj.json` synthetic demo 族当前也已从 official hydrocarbon package id 语义下拆开：相关 helper 已改为显式 synthetic demo 命名，并切到独立 `binary-hydrocarbon-synthetic-demo-v1` package id；`rf-ffi` 内建 demo package 也已同步改回这条 synthetic demo 语义，不再让 `binary-hydrocarbon-lite-v1` 同时指代 official hydrocarbon 与旧 `component-a/component-b` 两套目录。
- `tests/rust-integration` 中 residual generic cached-package helper 当前也已继续收口：shared official methane/ethane writer 现已改名为显式 `write_official_binary_hydrocarbon_cached_package`，旧 synthetic 示例文件也统一改成 `*-synthetic-demo.rfproj.json`，并同步更新 `rf-solver` / `rf-ffi` / Studio / 文档引用，不再让 generic helper 或泛化文件名继续模糊 official 与 synthetic 两套 fixture 语义。
- `apps/radishflow-studio` 本地测试模块里残留的 crate-local `write_cached_package` helper 当前也已继续收口到底：`app_facade`、`run_panel_driver` 与 `workspace_control` 中那层只负责补默认 `timestamp/expires_at` 的 local wrapper 已删除，相关测试现统一直接调用 shared `write_default_official_binary_hydrocarbon_cached_package`，不再在 Studio crate 内保留一层重复 cache fixture scaffolding。
- `apps/radishflow-studio::test_support` 共享 fixture/provider helper 当前也已继续收口：public surface 现只保留显式 official binary hydrocarbon provider / in-memory provider / cached-package writer，并新增 shared `OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID` 常量与默认 official cache fixture helper；底层仍需 remap `package_id` / component ids 的 official sample-derived payload helper 已退回私有实现，`solver_bridge` / `workspace_solve_service` / runtime / window-model / app facade 测试不再直接散落 `binary-hydrocarbon-lite-v1` literal 或重复拼装默认缓存元数据。
- `tests/rust-integration` 现在已把 `binary-hydrocarbon-lite-v1` 三组 two-phase 组成 `z=[0.195, 0.805] / [0.2, 0.8] / [0.23, 0.77]` 的 near-boundary `±ΔP / ±ΔT` case 前推到 `feed-heater-flash-binary-hydrocarbon` / `feed-cooler-flash-binary-hydrocarbon` / `feed-valve-flash-binary-hydrocarbon` / `feed-mixer-flash-binary-hydrocarbon` 正式链路；同一套回归也已补到 synthetic `liquid-only / vapor-only` 单相样例的 `feed-heater-flash-synthetic-demo` / `feed-cooler-flash-synthetic-demo` / `feed-valve-flash-synthetic-demo` / `feed-mixer-flash-synthetic-demo` 链路。`flowsheet_examples`、`studio_solver_bridge` 与 `studio_workspace_control` 现在会继续锁定非 flash 中间流股 `T / P / z / overall H / phase_region / bubble_dew_window` 与后续 flash inlet consumed stream 的同一份 DTO 语义，不在 raw solver path、workspace run path 或结果消费层分叉第二套判断。
- `flowsheet_examples`、`studio_solver_bridge` 与 `studio_workspace_control` 现在也已把 `Flash Drum` liquid / vapor outlet 自身的 `bubble_dew_window` 边界语义前推到仓库级回归：liquid outlet 会锁定 `bubble_pressure/temperature == stream pressure/temperature`，vapor outlet 会锁定 `dew_pressure/temperature == stream pressure/temperature`，避免这层语义只停留在 unit/focused tests。
- `flowsheet_examples`、`studio_solver_bridge_flash_inlet_boundary` 与 `studio_workspace_control_flash_inlet_boundary` 现在也会在同一批 near-boundary dedicated case 上继续锁定 `Flash Drum` outlet 语义：`liquid-only / vapor-only` case 会验证零流量对侧 outlet 的 `bubble_dew_window` 明确缺席，`two-phase` case 会继续锁定 liquid/vapor outlet 的饱和边界窗口，不让这层差异只在 unit test 中成立。
- Result Inspector / Active Inspector 现在会只读消费 `SolveSnapshot` 已物化的 `bubble_dew_window`，显式展示 `phase_region` 与 bubble/dew pressure / temperature；这层继续只消费 DTO，不在 shell 中重算热力学或分叉第二套相平衡语义。
- `apps/radishflow-studio::studio_gui_window_model` 现在也会在真实 solver snapshot 上锁定 `Result Inspector / Active Inspector` 的 flash outlet 展示语义：two-phase outlet 继续显示各自饱和边界窗口，`liquid-only / vapor-only` case 的零流量对侧 outlet 保持 `bubble_dew_window` 缺席，不把三类结果误展示成同一种窗口态。
- `apps/radishflow-studio::studio_gui_shell::tests::runtime` 现在也已把这组语义前推到最终 runtime 渲染面：`Result Inspector`、`Active Inspector` 与整块 runtime 面板都会只在 `bubble_dew_window` 存在时渲染 `Bubble/dew window` 区块，零流量对侧 outlet 不会冒出伪窗口；同一股流股同时出现在两处 inspector 时也已补上独立 widget id scope，避免 `egui` duplicate-id diagnostics 干扰正式展示；同时 official `binary-hydrocarbon-lite-v1` two-phase `Flash Drum` liquid/vapor outlet 的 `H` 摘要与 `overall/liquid/vapor` phase comparison rows 也已锁进 shell 最终渲染回归，不在 runtime 面分叉第二套结果消费语义。
- Studio bootstrap 默认项目、空白项目默认组件与 example catalog 现在也已统一回到 official `binary-hydrocarbon-lite-v1` 语义；`project lifecycle`、canvas presentation / layout / widget 与 window-model 相关回归继续复用同一套 official hydrocarbon fixture / shared helper，不再把 `*-synthetic-demo` 占位样例当作 official hydrocarbon sample 混用。
- Studio bootstrap 内置的 `binary-hydrocarbon-lite-v1` 样例包 Antoine 系数现在也已与当前 bubble/dew temperature 数值基线对齐，空白项目和 shell/solver 回归继续共享同一套相平衡假设。
- Result Inspector / Active Inspector 的流股相结果与相对比现在会显式展示各相摩尔流量，并继续只消费 `SolveSnapshot` 已物化的 phase fraction / molar enthalpy，不在 shell 中重算热力学。
- Solve step / Active Inspector / unit-centric Result Inspector 现在会为输入和输出流股显式展示 `T / P / F / H` 结果摘要，便于直接审阅单元前后变化；这层仍只消费已有 DTO 与既有 `InspectorTarget` command。
- Diagnostics 列表与 failure diagnostic 现在会前推相关流股数值上下文：成功路径直接显示 `SolveSnapshot` 已物化的 `T / P / F / H` 摘要，失败路径在诊断 revision 与当前文档匹配时显示文档态 `T / P / F / z` 与 port 绑定流股上下文；这层仍只消费结构化 snapshot，不在 shell 中反查文档或反解析错误消息。
- `rf-thermo` / `rf-flash` 已收紧直接数值 API 的 mole fraction 输入契约，未归一组成会被拒绝；unit operation 层继续在调用 flash 前归一化文档流股组成。
- Stream Inspector 已补正式 composition normalize / draft discard command surface，并展示当前组成总和、归一化预览与只读草稿提示；写回、归一化或丢弃草稿都必须由用户显式触发。
- Stream Inspector 已补受控组分添加 / 删除入口：添加只能从当前 flowsheet 已定义但流股组成尚未包含的组件中显式选择；删除不能移除最后一个组成条目；新增或删除都不自动补偿其他组分。
- `studio_gui_driver::command_surface_tests`、`studio_local_rules`、`studio_gui_shell::tests::canvas` 与 `workspace_run_command` 现在也已对齐到 official `methane/ethane` 组件目录；当前 `apps/radishflow-studio` 内剩余的 `component-a/component-b` 主要收敛在 intentional synthetic 草稿命令单元测试与 window-model synthetic helper，不再混入默认 bootstrap / local-rules / command-surface 主路径。
- intentional synthetic 草稿命令单元测试、window-model synthetic helper 与额外可添加组件的 command-surface 回归现在也已统一收拢到显式 `synthetic-component-*` 常量；当前 `apps/radishflow-studio` 里剩余的 `component-c` 只保留在 bootstrap official payload 的 extra-catalog 回归语义中。
- Studio 运行入口现在会在存在未提交/无效 Inspector 草稿或文档流股总体组成未归一时通过既有 run panel notice 阻塞运行，不做隐式自动补偿。
- 仓库基础治理已继续收口：根 `README.md` 改回稳定入口，`.gitattributes` / `.gitignore` 补齐基础规则，`*.idl` 已纳入 UTF-8 / LF 文本门禁。
- 协作文档已新增 Docs 简约入口约束与代码规范专题文档：`docs/development/code-style.md`。

## 下一步建议

1. 若继续推进 `rf-thermo` / `rf-flash`，优先把当前 two-phase 与 synthetic 单相样例在 `Feed/Heater/Cooler/Valve/Mixer -> Flash` 已前推到 `tests/rust-integration` / workspace run path / Studio consumer 的 near-boundary `±ΔP / ±ΔT` 基线维持为正式回归，并继续锁定 source stream 与非 flash 中间流股 `T / P / z / overall H / phase_region / bubble_dew_window` 和 downstream consumed stream 的同一份 DTO，不在集成层分叉第二套判断语义。
2. 若继续推进 `rf-thermo` / `rf-flash`，保持现有 golden 目录多样例遍历、focused tests 与 `Feed/Heater/Cooler/Valve/Mixer -> Flash` source/intermediate 到 consumer 的端到端一致性回归为同一套正式基线，不额外分叉第二套窗口估算、焓值求解或判断语义；fixture / provider 语义这条线当前已基本收干净，下一步更值得做的是继续删减 residual test boilerplate，或直接回到 near-boundary / `SolveSnapshot` consumer 主线，而不是继续围绕 helper 身份命名打转。
3. 若继续推进 Stream Inspector，优先收紧 flowsheet component catalog / presentation 边界；不要提前做完整组件库、项目级组件删除迁移或隐式差值补偿。
4. 若推进 Studio，优先继续消费已结构化 DTO 和既有 command surface，不新增第二套 shell 私有状态机。
5. 若发现入口文档继续膨胀，优先更新本文档和对应专题文档，不把长篇历史写回 `overview.md` 或 `scope.md`。

## 暂不推进

- 不扩自由连线编辑器、拖拽布局编辑器、自动布线、视口持久化或完整结果报表。
- 不把 CAPE-OPEN / COM 语义倒灌到 Rust Core。
- 不引入第三方 CAPE-OPEN 模型加载。
- 不把 smoke test driver、PME 调试路径或单个宿主兼容逻辑提升为通用库 API。
- 不为未来可能需求预先堆叠不明意义的 helper / manager / orchestrator / context / adapter。

## 按需阅读

- 需要仓库全局模块边界：`docs/architecture/overview.md`
- 需要 MVP 范围和非目标：`docs/mvp/scope.md`
- 需要最新流水和决策依据：`docs/devlogs/2026-W19.md`
- 需要热力学 / 闪蒸细节：`docs/thermo/mvp-model.md`
- 需要 CAPE-OPEN / COM 边界：`docs/capeopen/boundary.md`
- 需要桌面 App / Canvas 交互契约：`docs/architecture/app-architecture.md`、`docs/architecture/canvas-interaction-contract.md`
- 需要代码风格、命名或抽象判断：`docs/development/code-style.md`

## 更新规则

- 本文档只保留当前阶段、最近状态、下一步建议和按需阅读入口。
- 历史流水写入周志；长期边界写入专题文档；不要把本文档写成长篇进度报告。
- 协作入口文件只保留长期稳定规则；阶段性变化优先更新本文档和对应专题文档，再按需同步入口文件中的引用关系。
- 每次完成重要阶段收口后，优先更新本文档顶部状态和“下一步建议”。
