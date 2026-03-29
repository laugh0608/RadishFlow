# RadishFlow 协作约定

## 语言要求

- 所有讨论、实现说明、提交总结、开发日志统一使用中文

## 工作区边界

- 只在 `D:\Code\RadishFlow` 内工作
- 旧仓库 `CapeOpenCore` 只作为外部参考，不在当前协作中跨工作区修改

## 第一阶段产品边界

- Rust 负责模拟核心
- Rust 负责主界面
- `.NET 10` 负责 CAPE-OPEN/COM 适配
- 第一阶段只导出自有 CAPE-OPEN Unit Operation PMC
- 第一阶段不支持加载第三方 CAPE-OPEN 模型
- Rust 不直接处理 COM

## 开工前必读

开始新任务前，应先阅读以下文档：

1. `docs/radishflow-architecture-draft.md`
2. `docs/radishflow-startup-checklist.md`
3. `docs/radishflow-mvp-roadmap.md`
4. `docs/radishflow-capeopen-asset-checklist.md`
5. 与当前任务直接相关的专题文档

## 当前推荐开发顺序

按以下顺序推进，不要跳步扩张范围：

1. `rf-types`
2. `rf-model`
3. `rf-thermo`
4. `rf-flash`
5. `rf-unitops`
6. `rf-flowsheet`
7. `rf-solver`
8. `rf-store`
9. `rf-ffi`
10. `.NET 10` 适配层
11. Rust UI 深化

## 当前阶段工作流

- 先检查仓库状态和相关文档，再动手修改
- 每做完一个可分割子步骤，都要确保 workspace 仍可 `cargo check`
- 优先更新已有文档，不为单次讨论创建大量散文档
- 新的阶段性决策要同步到 `docs/` 和对应周志
- 周志按 `docs/devlogs/YYYY-Www.md` 维护
- 提交信息保持简洁明确，优先使用约定式前缀

## 当前不应提前展开的内容

- 第三方 CAPE-OPEN 模型加载
- 完整 Thermodynamics PMC
- 复杂 recycle 收敛
- 重型 UI 打磨
- 在 `M4` 之前做复杂 `.NET 10` 运行时实现

## 当前阶段的判断标准

如果一个改动同时满足以下条件，则方向通常是对的：

- 边界更清晰
- 工作区仍然稳定可编译
- 文档与代码一致
- 没有把后续阶段复杂度提前压进当前阶段
