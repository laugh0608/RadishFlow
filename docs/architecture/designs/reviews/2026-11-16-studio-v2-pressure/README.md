# Studio V2 流股绝对压力图层

2026-11-16，项目所有者确认绝压标尺及零流量着色规则。Studio 压力图层用于在同一稳态快照上沿 Feed、Valve 和 Flash Drum 核对绝压、选择流股查看读数，并在运行失败时定位 Valve 诊断。 [所属专题](../../../../topics/results-review-diagnostics.md#可视化接下来事项)

## 已确认决定

1. **用当前快照的绝压范围。** 沿用画布「图层」入口，增加「流股绝对压力」。工程打开时默认显示连接方向，切换图层保留流股选择。仅使用 `FlowResultView::Current` 中有限且大于零的 `StreamStateSnapshot.pressure_pa`。在 Pa 中按最小值与最大值线性取色，等压时显示单色和一个读数。图例标明绝压、单位、范围和运行／修订。读数通过单位目录的 `AbsolutePressure` 转换，切换 Pa／kPa／MPa／bar 时保留线色。跨工况固定标尺另行评审。 [结果契约](../../../../reference/solve-snapshot-results.md#streamstatesnapshot-的稳定语义)
2. **零流量但有有效绝压时，用带色虚线并纳入标尺。** Flash 在某相流量为零时仍会写入出口绝压。带色虚线和读数旁的「零流量」共同表达这两个事实，箭头保留连接方向，流动效果沿用既有流量资格。

压力缺失或无效时使用中性虚线，逐股说明原因并保留定位。当前结果不可用或没有任何有效压力时，撤下色带和压力值，保留拓扑与诊断入口。警告和错误沿用独立标记。

## 算例依据

从 `3e118e59` 锁定离线构建 Studio，以 [Feed–Valve–Flash 工程](../../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json) 运行 [headless v1](../../../../reference/headless-cli.md#请求版本-1)，从 `results` 读取 `pressure`、`molar_flow` 和 `temperature`。两组参数调整只作用于各次请求内存，原工程 SHA-256 复验一致。

| 快照 | Feed | Valve Outlet | Liquid Outlet | Vapor Outlet | 绝压标尺 |
| --- | --- | --- | --- | --- | --- |
| #1 · r0，原算例 | 700 kPa | 650 kPa | 650 kPa | 650 kPa | 650–700 kPa |
| #1 · r1，Flash 温度设为 345 K | 700 kPa | 650 kPa | 650 kPa，0 mol/s | 650 kPa | 650–700 kPa |
| #1 · r2，Valve 和 Flash 出口压力均设为 700 kPa | 700 kPa | 700 kPa | 700 kPa | 700 kPa，0 mol/s | 700 kPa |

两处出口压力均通过 `outlet_pressure` 写入 700,000 Pa。过期与尚无结果为画板状态。失败画板依据临时工程副本的真实失败。副本的阀出口设为 710,000 Pa，高于 700,000 Pa 的入口压力，返回 `solver.step.parameter`，定位到 `valve-1`，结果变量为空。

真实零流量分支与同工况的另一 Flash 出口等压，两种方案的范围相同。画板另设模拟对照，仅将零流量的 Liquid Outlet 改为 600 kPa。纳入时显示 600–700 kPa 并重新映射其他流股色值，排除时保持 650–700 kPa。另一个模拟状态移除 Vapor Outlet 的压力，用于检查缺值呈现。

## R1 交互画板

打开 [boards.html](boards.html)。在底部「画板评审」中选择「零流量支路」查看真实快照，选择「模拟数据 · 零流量压力为极值」后切换「零流量压力」以比较两种规则。「项目显示单位」模拟 Studio 的单位偏好。画板沿用 [摩尔流量画板样式](../2026-11-16-studio-v2-molar-flow/styles.css) 及仓库内 Inter 和思源黑体。

| 截图 | 场景 |
| --- | --- |
| [当前快照](current-zh-1280.png) | 中文，1280 px，14 px 基准字号，650–700 kPa |
| [零流量支路](zero-zh-400.png) | 中文，400 px，20 px 基准字号，带色虚线及 650 kPa |
| [等压快照](equal-en-400.png) | 英文，400 px，20 px 基准字号，7 bar 单值 |
| [过期结果](stale-zh-400.png) | 中文，400 px，20 px 基准字号，拓扑和上次运行身份 |
| [运行失败](failed-en-400.png) | 英文，400 px，20 px 基准字号，Valve 诊断和返回入口 |

Chrome 157.0.8103.51／Playwright 的 52 项检查覆盖单位切换前后画布像素一致、标尺极值对照、等压、零流量与缺值、失效撤色、诊断回焦及图层往返。以中英文分别复核 400、560、768、970、1280 px，400／560 px 使用 20 px 基准字号。

## 产品接线

- Studio 复用当前快照查询、色带、单位目录和现有诊断定位。图层入口使用和单位设置共享的下拉选择，避免第四项在英文窄区挤压画布。Tab 聚焦，Enter 打开，方向键选择，Enter 应用，Esc 取消。
- 图例显示所选流股的当前压力及零流量提示，可直接核对短线和裁剪区域内不完整的画布标签。当前结果失效时一并收起该读数。
- 9 项图层定向测试覆盖真实阀门算例、等压、零流量极值、缺失和无效压力、Pa／kPa／MPa／bar 同色、同修订重跑、换工程、状态失效和中英文键盘。锁定离线仓库检查通过，共 2,150 项 Rust 测试。 [W47 验收证据](../../../../devlogs/2026-11/2026-W47.md#2026-11-16-studio-v2-绝压图层产品接线与验收)
- macOS 常规实窗验证 Pa／bar 单位切换、零流量带色虚线及所选流股读数、失效撤色和 Valve 入口诊断。中英文原生窄区为 400 逻辑点／20 px，键盘可操作图层选择，滚动后可查看等压单值及所选支路。Windows／Linux 原生操作与实际读屏待验证。
