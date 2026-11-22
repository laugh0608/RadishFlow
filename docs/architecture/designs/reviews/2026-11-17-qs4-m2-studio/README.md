# QS4 M2 Studio 局部评审

2026-11-17，项目所有者已确认 [Studio 接入方向](../../../../topics/modeling/mixed-partition-execution.md#m2-studio-界面接入评审)，并认可本版局部画板。场景沿用 CLI v9 的 methane／ethane 验收工况，配置使用唯一撕裂 `s-rba`。

打开 [交互画板](boards.html)，用顶部控件切换场景、语言和字号。可编辑源为 [HTML](boards.html)、[CSS](styles.css) 和 [交互脚本](boards.js)，沿用仓库 Inter 与思源黑体。画板用代表数值模拟已接收结果，文件选择、保存和 Run 命令的产品验收走 Studio 实窗。

| 渲染证据 | 场景 |
| --- | --- |
| [计划草稿](m2-draft-zh-1140.png) | 中文 1140 px，单撕裂绑定、组分初值、预算及应用 |
| [首次保存](m2-upgrade-en-400.png) | 英文 400 px／200%，取消默认获焦，影响区可滚动，保存动作可见 |
| [当前结果](m2-current-zh-400.png) | 中文 400 px／200%，复核、守恒、物性来源，以及逐组分最后一轮的实际值 |
| [运行失败](m2-failed-en-400.png) | 英文 400 px／200%，预算耗尽、诊断和流股定位 |
| [等待取消回收](m2-cancelling-en-1140.png) | 英文 1140 px，先前接收的结果保持可审阅 |
| [重开后无结果](m2-missing-zh-400.png) | 中文 400 px／200%，已保存配置等待 Run／F5 |
| [未知能力只读](m2-readonly-en-400.png) | 英文 400 px，未知能力明确显示，提供返回入口 |

Chrome `157.0.8103.51`／Playwright 复核了 48 个布局组合，覆盖 1140 px／100%、400 px／100% 与 400 px／200% 下的两种语言和八个场景。验证取消默认焦点、Tab／Shift+Tab、Esc、Enter、F5，以及「应用 → 显式保存 → 运行 → 重开 → F5」顺序。400 px 下的逐组分证据纵向排布，数值完整显示。截图保存的是这版可编辑源的渲染。

按 [R1 局部评审规则](../../../studio-ui-topic-plan.md#设计工具与文档规则)获认可后，Studio 用同一份解码后的工程启动宿主，以 `PreparedMixedRecycleSave` 提交已确认保存。Run／F5 使用正式手动命令，结果来自 `FlowResultView`。原生侧栏在窄区逐组分纵向呈现撕裂明细。 [实施与实窗证据](../../../../devlogs/2026-11/2026-W47.md#2026-11-17-qs4-m2-studio-接入与验收)
