# Studio V2 流股温度图层

2026-11-16，R0／R1 待评审。用途是在画布上追踪同一份稳态结果中，Feed、Heater 和 Flash Drum 各流股的绝对温度。 [所属专题](../../../../topics/results-review-diagnostics.md#可视化接下来事项)

## 待评审决定

- **入口与数据来源。** 沿用画布「图层」入口，在「连接方向」和「摩尔流量」旁增加「流股温度」。打开工程仍默认显示连接方向，切换图层保留流股选择。取 `FlowResultView::Current` 的流股 `temperature_k`，图例显示项目绝对温度单位及运行／修订。
- **范围与显示。** 有效范围取当前快照中实际流动流股的最小与最大绝对温度，在 K 中线性取色。切换到 °C 等项目单位时，标签、图例和流股读数用单位目录换算，线身及箭头保持原色。所有有效流股同温时，按单色和一个温度读数展示。快照范围用于当前单次审阅，跨工况共用标尺另见结果专题。
- **零流量、缺值和诊断。** 建议对零摩尔流量的流股保留中性虚线及「零流量」说明，其记录温度可在结果面查看。温度缺失或无效时同样保留可定位的连接，文案显示缺值或无效。只有有限正流量及有限正绝对温度进入色带范围。过期、缺失或失败时收起色带和数值，保留拓扑及诊断入口。诊断标记沿用既有独立通道。

## 算例依据

用 `8d1689e7` 的锁定离线 Studio 构建求解 [Feed–Heater–Flash 算例](../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json)，从 headless `results` 同时读取四条流股的 `temperature` 和 `molar_flow`。基于 [v1 请求模板](../../../../../examples/automation/heater-330k.request.json)，r0 不写入参数，r1 写入 `heater-1` 的 `outlet_temperature = 300 K`，r2 在同一请求中另写 `flash-1` 的同一字段。写入只作用于运行内存。

| 快照 | Feed | Heated Outlet | Liquid Outlet | Vapor Outlet | 拟显示温度范围 |
| --- | --- | --- | --- | --- | --- |
| #1 · r0，原算例 | 300 K | 345 K | 345 K，0 mol/s | 345 K | 300–345 K |
| #1 · r1，仅调整 Heater | 300 K | 300 K | 345 K，0 mol/s | 345 K | 300–345 K |
| #1 · r2，Heater 与 Flash 同温 | 300 K | 300 K | 300 K，0 mol/s | 300 K | 300 K |

三个快照中 Feed、Heated Outlet、Vapor Outlet 均为 `5 mol/s`。r1 说明 Flash 的结果温度受本单元设定控制，画布应从各流股结果取值。r0 的显示范围换成摄氏度为 `26.85–71.85 °C`，r2 为 `26.85 °C`。缺值场景模拟删去 Vapor Outlet 的温度，过期和无结果场景为画板状态。失败场景沿用 [Heater 入口断口](../2026-11-15-studio-v1-flow-path/README.md#验收结果)。

## 交互画板

打开 [boards.html](boards.html)。底部「画板评审」可切换算例状态和语言。「项目显示单位」模拟 K／°C 的项目偏好。选择 Liquid Outlet 后，可切到摩尔流量查看其 `0 mol/s`。画板沿用 [摩尔流量画板样式](../2026-11-16-studio-v2-molar-flow/styles.css) 与仓库字体。

| 截图 | 场景 |
| --- | --- |
| [当前快照](current-zh-1280.png) | 中文，1280 px，14 px 基准字号，300–345 K |
| [等温快照](isothermal-en-400.png) | 英文，400 px，20 px 基准字号，26.85 °C |
| [过期结果](stale-zh-400.png) | 中文，400 px，20 px 基准字号，旧范围已撤下 |

Chrome 157.0.8103.51／Playwright 的 41 项检查覆盖原始值、等温、单位切换后画布像素一致、零流量、缺值、失效、诊断回焦和图层往返。中英文在 400、560、768、970、1280 px 复核控件与图形标注的布局，400／560 px 使用 20 px 基准字号。

## 评审后的接线验收

同一共享结果查询提供范围与逐股温度，采用单位目录的 `AbsoluteTemperature`。定向测试覆盖多值、同温、缺值、零流量、同修订重新运行以及单位换算前后颜色一致。macOS 实窗沿原算例走图层键盘切换、范围和零流量说明，复用摩尔流量已验的窄区路径验证布局。
