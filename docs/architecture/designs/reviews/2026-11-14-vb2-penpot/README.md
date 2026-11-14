# Studio VB2 跨对象批量草稿核对

2026-11-14 · 局部设计 v1 · 待所有者评审

从变量浏览器关注清单选取两个流股的摩尔流量，在 Studio 400 px 侧区核对草稿。两个流股及其值是评审样例。正式约束见 [变量与对象浏览器专题](../../../../topics/variable-and-object-browser.md#接下来事项)。

1. 草稿中分别列出每项的工程当前值与草稿值。示例当前值为 `5 mol/s` 和 `3 mol/s`，草稿为 `7.25 mol/s` 和 `1e- mol/s`。
2. 点击「检查 2 项草稿」后，第二项的指数不完整，整批阻断。状态提示「已应用 0 项」，保留两项原文，第一项标明已通过检查，第二项说明修正方式。
3. 点击「返回编辑第 2 项」可查看目标字段的高亮，第一项草稿继续可见。原型里的字段和工程值为静态样例；点击「返回草稿核对」可重复评审此路径。

## 可编辑源与预览

[下载 Penpot 源稿](./studio-vb2-batch-review.penpot) · [本机原稿](http://localhost:9001/?screen=workspace&team-id=bdbf295a-e866-8018-8008-f1646a78f9db&file-id=bdbf295a-e866-8018-8008-f172fb7a8cfe&page-id=bdbf295a-e866-8018-8008-f172fb7a8cff) · [本机恢复副本](http://localhost:9001/?screen=workspace&team-id=bdbf295a-e866-8018-8008-f1646a78f9db&file-id=bdbf295a-e866-8018-8008-f1748c14a915&page-id=bdbf295a-e866-8018-8008-f172fb7a8cff)

| 语言与字号 | 草稿 | 整批阻断 | 返回编辑 |
| --- | --- | --- | --- |
| 中文 100% | [草稿](./zh-draft-100.png) | [阻断](./zh-blocked-100.png) | [返回编辑](./zh-edit-100.png) |
| 中文 200% | [草稿](./zh-draft-200.png) | [阻断](./zh-blocked-200.png) | [返回编辑](./zh-edit-200.png) |
| English 100% | [Draft](./en-draft-100.png) | [Blocked](./en-blocked-100.png) | [Return to edit](./en-edit-100.png) |
| English 200% | [Draft](./en-draft-200.png) | [Blocked](./en-blocked-200.png) | [Return to edit](./en-edit-200.png) |

画板统一宽 400 px，内容单列向下排布。长对象名和提示按样例换行。中文用 Noto Sans SC，英文用 Inter Tight。正文及控件字号由 12–20 px 提升到 24–40 px，这里的 200% 指逐项将文字字号翻倍。画板高度随内容增长，用于审阅可滚动的完整内容。实际产品的系统字体缩放、动态排版、键盘焦点与读屏、输入及工程事务需要等 Studio 实现后验收。

## 导出恢复验证

在本机 Penpot 2.19.1 通过 MCP 创建、读回和渲染，导出文件通过 ZIP 完整性检查。重新导入到同一项目下的独立文件后，MCP 读回的 12 张画板、180 个文字图层及其文案、字号、布局、4 个原型流程及 12 条跳转与原稿一致。恢复副本中，中英文各一张 200% 阻断画板重新导出后的 PNG 与原稿逐字节一致。12 张 PNG 均为原尺寸导出，文本量测均落在所属面板内。分别在原稿和恢复副本里点通中英文各两档的「草稿 → 阻断 → 返回编辑 → 草稿」。

源文件 SHA-256 为 `f06c6206d61363e7795aff88157d65ad5002c5ada40ace94f9c44858a7e0e891`。
