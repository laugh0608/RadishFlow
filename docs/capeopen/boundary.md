# CAPE-OPEN Boundary

更新时间：2026-03-31

## 边界目标

该文档用于冻结 Rust Core 与 `.NET 10` CAPE-OPEN 适配层之间的边界，避免 COM 语义反向污染 Rust 核心。

## 第一阶段原则

第一阶段 CAPE-OPEN 边界必须遵守以下原则：

- Rust 不直接处理 COM
- COM 和 CAPE-OPEN 适配全部放在 `.NET 10`
- 第一阶段只导出自有 Unit Operation PMC
- 第一阶段不支持加载第三方 CAPE-OPEN 模型
- `.NET 10` 负责把 Rust 错误映射为 CAPE-OPEN/ECape 语义

## Rust 与 .NET 的运行时边界

Rust 与 `.NET 10` 之间的正式边界应保持简单稳定：

- 句柄
- 基础数值
- UTF-8 字符串
- JSON 快照
- 明确的错误码

当前第一版 `rf-ffi` 应进一步冻结为以下约束：

- 对象跨边界一律优先使用句柄式生命周期，不直接传递 Rust 结构体
- 字符串跨边界一律使用 UTF-8 编码，并明确由哪一侧负责分配与释放
- 数组跨边界只允许使用“指针 + 长度”形式，并明确只读/可写与所有权规则
- 复杂配置、求解输入输出快照和可扩展元数据优先通过 JSON 传递
- 错误先在 Rust 内部表达为稳定错误类型，再映射为错误码与可选诊断文本

截至 2026-04-14，`rf-ffi` 已开始落地第一版最小运行时边界，当前已实现并冻结以下最小 ABI：

- `engine_create`
- `engine_destroy`
- `engine_last_error_message`
- `engine_last_error_json`
- `rf_string_free`
- `flowsheet_load_json`
- `property_package_load_from_files`
- `flowsheet_solve`
- `flowsheet_get_snapshot_json`
- `stream_get_snapshot_json`

当前这版 ABI 的额外口径为：

- 输入字符串使用 `pointer + length` 传入，解释为 UTF-8
- 输出字符串由 Rust 侧分配为 UTF-8 C string，并统一通过 `rf_string_free` 释放
- 最近一次错误当前同时支持文本导出和结构化 JSON 导出，后续 `.NET` 适配层不必只靠错误文本分支
- `flowsheet_load_json` 当前加载 `StoredProjectFile` JSON
- `property_package_load_from_files` 当前允许把本地 `manifest.json + payload.rfpkg` 注册到 engine 内的 package registry
- `flowsheet_solve` 当前按 `package_id` 选择物性包，并把最新 `SolveSnapshot` 留在 engine 内
- `flowsheet_get_snapshot_json` 当前导出最近一次成功求解的整份 `SolveSnapshot` JSON
- `stream_get_snapshot_json` 当前从最近一次成功求解的 `SolveSnapshot` 导出单股流体 JSON
- 返回状态码当前分为两层：FFI 前置错误（如空指针、非法 UTF-8、未加载 flowsheet / 未生成 snapshot）与 `rf_types::ErrorCode` 映射的内核错误；结构化错误 JSON 当前会额外带出 `ffiStatus`、`code`、`diagnosticCode`、`relatedUnitIds`、`relatedStreamIds` 与 `relatedPortTargets`

当前这版运行时仍是最小实现，额外明确以下暂时约束：

- engine 当前内置一份与仓库示例 flowsheet 对齐的 demo property package，用于打通 Rust Core 调用链
- engine 当前同时允许从本地 `manifest/payload` 文件注册真实 package；相同 `package_id` 会覆盖当前 registry 中已有条目
- 当前还未引入 auth cache、本地缓存目录或真实 `.NET` PInvoke 编排
- 当前先不导出完整 `SolveSnapshot` JSON，也不提前暴露 COM / CAPE-OPEN 语义

当前不允许在边界上直接传递以下内容：

- COM 接口对象
- `IDispatch`
- `VARIANT`
- `SAFEARRAY`
- 复杂对象图

## 当前仓库阶段约束

截至 2026-03-29，`.NET 10` 适配层在仓库中的职责仍然是“冻结边界与目录结构”，而不是提前完成复杂实现。

当前允许推进的内容：

- 文档
- 目录结构
- README 占位说明
- 未来 `Interop`/`Adapter` 的接口落点规划

当前暂不推进的内容：

- 真实 `.NET 10` PInvoke 封装
- COM host 注册细节
- 完整 ECape 异常实现
- PME 互调测试代码

## 对 Rust Core 的约束

为了给后续 `rf-ffi` 留出干净边界，Rust Core 当前应坚持以下约束：

- 领域模型不带 COM 类型
- 错误先在 Rust 内部表达为统一错误类型
- 输出结果优先落在普通 Rust 数据结构与 JSON 友好结构上
- 单元与流股对象先面向内核求解，不直接面向 CAPE-OPEN 接口建模
- 第一版 FFI 接口优先围绕 `engine` / `flowsheet` / `stream snapshot` 等稳定能力展开，不提前暴露过细的内部实现细节

## 结论

第一阶段真正要做的是“让 `.NET 10` 适配层能调用 Rust Core”，不是“让 Rust 看起来像 COM 组件”。  
这条边界如果现在守不住，后面 FFI、PMC 和 UI 都会被一起拖复杂。
