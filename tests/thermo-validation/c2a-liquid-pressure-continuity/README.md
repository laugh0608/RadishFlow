# C2-A：压力规则连续性独立候选

用途：冻结统一广义 Pcm 的离线候选及端点、实验和工程比较证据。
读者：物性方法与工程验证审阅者。
不包含：产品物性实现、连续域物理资格、C2-B / C3 开工或发布许可。

2026-11-05 所有者批准本片范围。基于 `63e83cff`，旧四轮验证包、参数和失败保持；本包是独立版本，不覆盖旧报告。实现与数值验证已完成，结果采纳和物理资格仍需审阅。

## 唯一规则变化

[protocol.json](protocol.json) 在比较前冻结两种新身份：

- `COSTALD-Tait-uniform-Pcm-v1`：优先研究候选。
- `modified-Rackett-Tait-uniform-Pcm-v1`：修正 Rackett＋Tait 筛查对照，不声称其压力模型已获独立物理验证。

整个组成闭区间，包括纯物质、零量组分和微量组分，统一使用 `Pcm=(0.291−0.080 omega_m) R Tcm/V*m`；不再按正含量组分数切换真实 Pc。Tcm、V*m、omega 的混合规则、Riedel 系数 4.86601、Tait 因子和两种饱和体积公式均沿用固定来源。广义 Pcm 是关联参数，不宣称等于真实临界压力。

[parameters.json](parameters.json) 与上一包逐字节相同。真实 Pc 仍保留，并继续用于修正 Rackett 的饱和体积；未拟合、估参、调整 PR alpha / kij 或加入体积平移。280–320 K、压缩输入 0.2–1.0 MPa、Tr 0.25–0.95、P≥关联 Ps 的域与原误差判据保持；域外明确不可用，不夹紧压力。

本包将上一包的短公式与比较脚本独立版本化，保证旧清单不变；实验输入、PR / HEOS 结果与 S1/S2 运输脚本仍引用原包。复现先校验四轮清单及既有 Cargo.lock 漂移说明，不能将此次重放解释为首轮构建环境逐字节重建。

## 实验与工程结果

| 检查 | COSTALD | Rackett 对照 |
| --- | --- | --- |
| 原 47 个实验液体工况 | 46 pass、1 unavailable | 46 pass、1 unavailable |
| 最大可评分密度偏差 | 0.3072% | 0.7236% |
| 与前一候选相比，最大密度变化 | 约 0.002606% | 约 0.002606% |
| 不可用点 | `j.jct.2006.07.012/d1/p1` | 同左 |

不可用点仍为丙烷 280 K / 581400 Pa。新关联 Ps 约 589156.940 Pa，不能将规则连续性改善误写为解决原实验点。原 PR 最大偏差 5.7049% 与全部原失败不变。47 点中 25 个为纯物质压缩液体工况；二元内部组成和 11 个饱和 / 泡点的密度未变。

23 个原 PR 两相工况、三种流量倍率、两种几何，以及 PR / HEOS 原 276 行加两候选各 138 行，共 **552 次**真实 S1/S2 计算。该组液相组成全部严格位于端点以内；恢复候选名称映射后，新旧 552 行工程输出逐字段相同。相质量流量和气相输入保持。

相对 HEOS 的直径仍全部满足 5%，近相界气 / 液负荷仍有 24/138、42/138 超过 10%；原中部工况相对 PR 的液体准则和综合结论翻转保持。HEOS 仍是模型对照，不是独立实验真值。

## 边界与独立验证

[results.json](results.json) 保存 264 行边界探查：3 个温度、11 个组成、1 MPa 与关联 Ps 下侧 / 等值 / 上侧、两种方法。140 行按既定压力 / 温压域不可用，全部保留。数值边界探查不属于新增实验点，也不构成连续域资格。

- 两端以 1e-6、1e-9、1e-12 微量组成逼近，压力伪参数无分支突变；纯物质、零量组分补位和组分置换保持一致。
- 原反例丙烷 280 K / 587000 Pa 在纯点及微量混合点均不可用；590000 Pa 两者均可计算且连续。这是明确的域边界变化，不是默认成功或压力夹紧。
- 在原 TP 域内，P=Ps 时压缩体积等于饱和体积，稍高压力密度增加，稍低压力拒绝；域外 Ps 探查保留原温压域拒绝，不为边界测试扩域。
- [independent_check.py](independent_check.py) 不导入候选公式，用 60 位 Decimal、完整双重求和和直接 ln 比值重算 138 个可用密度值，最大相对差约 1.41e-15；独立核对 552 行工程的 1,656 项数值及准则，最大相对差约 4.44e-16。结果见 [independent-results.json](independent-results.json)。这是算术核对，不是新增物理证据。

## 离线复现

仅用 Python 标准库和既有锁定 Rust harness；不安装 chemicals / CoolProp，不下载数据，不修改依赖。使用全新输出目录，已冻结目录会被拒绝。

```bash
python3 tests/thermo-validation/c2a-liquid-pressure-continuity/compare.py --output-dir target/c2a-pressure-replay
CARGO_NET_OFFLINE=true cargo build --locked -p rf-unitops --example c2a_separator
python3 tests/thermo-validation/c2a-liquid-density-methods/run_engineering.py --inputs target/c2a-pressure-replay/engineering-inputs.tsv --harness target/debug/examples/c2a_separator --output target/c2a-pressure-replay/engineering-results.tsv
python3 tests/thermo-validation/c2a-liquid-pressure-continuity/compare.py --output-dir target/c2a-pressure-replay --engineering-results target/c2a-pressure-replay/engineering-results.tsv
cmp tests/thermo-validation/c2a-liquid-pressure-continuity/results.json target/c2a-pressure-replay/results.json
cmp tests/thermo-validation/c2a-liquid-pressure-continuity/engineering-inputs.tsv target/c2a-pressure-replay/engineering-inputs.tsv
cmp tests/thermo-validation/c2a-liquid-pressure-continuity/engineering-results.tsv target/c2a-pressure-replay/engineering-results.tsv
python3 -m unittest discover -s tests/thermo-validation/c2a-liquid-pressure-continuity -p 'test_*.py'
python3 tests/thermo-validation/c2a-liquid-pressure-continuity/independent_check.py
```

[manifest.json](manifest.json) 冻结本包，实际执行证据归 [W45](../../../docs/devlogs/2026-11/2026-W45.md)。本包 Python 证据测试不计入 workspace Rust 测试数量。来源与许可见[第三方说明](THIRD_PARTY_NOTICES.md)；原论文参数表认证、VLE 冲突、PR 标签资格、二元气相与新留出密度点仍待补。没有产品代码、接口、工程格式或运行时依赖变更。
