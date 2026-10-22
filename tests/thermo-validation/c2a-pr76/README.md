# C2-A：丙烷 / 正丁烷 PR76 独立验证包

用途：提交 2026-10-22 获准的 C2-D01—05 独立验证结果供审定。读者：项目所有者、物性与工程计算维护者。不包含：产品 PR 实现、C2-B / C3 授权、工程设备合格证明。

## 审定结论

**验证包完成；候选模型未通过工程准确性门禁，没有获准的连续适用域。建议保持 C2-B / C3 暂停。** PR76、经典混合规则、显式 `kij=0`、无体积平移在所测液相密度上存在超过 5% 的偏差；参考库相态标记和实验汽相组成亦存在待解决的问题。未改参数、调松容差、替换失败结果或自动缩域。

本轮仅新增离线脚本、公开数值 fixture、调用既有 S1/S2 API 的 Rust example 和文档。正式方案与后续接口影响见 [物性方案](../../../docs/thermo/mvp-model.md#c2-首批范围与验证方案2026-10-22)。

## 冻结条件和证据层次

- 候选域：280–320 K、0.2–1.0 MPa、丙烷总摩尔分率 0.2–0.8；纯组分作为极限检查，纯饱和压力按温度选择。SI 单位与摩尔组成统一。
- [protocol.json](protocol.json) 在首轮批量计算前冻结网格、留出点、相界构造和门槛；后续相态审计只增加诊断，不更改原输出或门槛。
- [parameters.json](parameters.json) 冻结 CAS、摩尔质量、临界常数、偏心因子、R、PR 常数及 HEOS 参数。PR 的 `omega_a=0.4572355289213822`、`omega_b=0.07779607390388456` 与常见四舍五入常数有差别；将来同模型测试必须匹配。HEOS 参数不是 PR 的 `kij`；纯流体 HEOS 的 R 可能沿用原 EOS 常数，各状态记录实际 R。
- CoolProp 8.0.0 / commit `ae81610e7d23efc57f9d051c8e70a4d66e87537f` 仅作离线参考；CPython 3.14.5、NumPy 2.5.4、macOS 27.0.1 ARM64，wheel 摘要见 [environment.json](environment.json)。没有独立 Rust PR 实现，因此“同模型实现一致性”仍是 B 的待执行项。
- 六篇实验来源、14 个数据集的全部 2,882 条数值记录留在 [experimental-data.json](experimental-data.json)，出处 / 哈希见 [sources.json](sources.json)，归属与许可见 [第三方说明](THIRD_PARTY_NOTICES.md)。所报告不确定度保留原编目层级，不把它当完整测量协方差或自动扩大验收容差。
- 参考值不调用 RadishFlow 物性代码生成。相质量密度用同相摩尔密度和分子量，实际相流量用 `F × beta_phase / rho_molar`；没有 STP 换算或独立液体密度修补。

## 覆盖与排除

| 内容 | 数量与口径 |
| --- | --- |
| 二元 TP | 27 网格 + 12 留出 + 36 泡露点两侧 + 9 中部 = 84；76 域内、8 域外保留但不评分 |
| 纯组分 | 6 个饱和状态 + 18 个单相候选状态 = 24 |
| 实验比较 | 154 组；其中二元 VLE 23 组、二元液体 PVT 11 组、二元泡点 / 饱和液密度 8 组，其余纯组分 |
| 原始记录对账 | 154 主记录 + 2,625 排除主记录 + 103 配对记录 = 2,882；所有点恰好对账一次 |
| 工程敏感性 | 两参考均自洽且均两相的 23 点 × 3 流量倍率 × 2 几何 × 2 后端 = 276 次 S1/S2；138 对比较 |

二元 VLE 数据来自 Seong 等（2008）；按相同 T / 液相 x 连接压力和汽相 y，不能按行号配对。构造 `z=(x+y)/2` 后的 `beta=0.5` 是杠杆规则预期，**不是实验流量测量**。原始 x / y 可超出总体 z 域，允许参与对应两相比较。

密度数据包含 Kayukawa 等（2005）、Miyamoto 等（2008）的二元液体记录，McLinden（2009）的丙烷气相 PVT，以及其他纯液体数据。当前域内有 20 个丙烷气相实测点；缺少二元气相及正丁烷气相的独立实测密度覆盖。泛称 `Fluid` 的纯流体数据按实测密度相对冻结临界密度标识观测分支，强制该分支仅用于密度比较；自由 TP 查询另留诊断，不视为稳定相验证。

## 结果

逐点数值、排除原因和原始相态见 [reference-results.json](reference-results.json)；每个判据的误差、最坏点、失败和诊断见 [assessment.json](assessment.json)。最终汇总为 1,790 项 pass、247 项 fail、25 项 reference_inconsistent、24 项 unscored、2 项 review_required。其 `pass` / `fail` 是**比较项**，不是独立实验次数或产品测试通过率；一个工况可产生多项比较。

| 比较 | 结果 |
| --- | --- |
| 实验饱和 / 泡点压力 | 每个后端 95 项，均在 2% 内；PR 最大 1.594%，HEOS 最大 1.375% |
| 实验指定分支密度 | PR 64 项中 22 项超过 5%，最大 5.705%；HEOS 最大 0.428% |
| 实验饱和液密度 | PR 11 项中 5 项超过 5%，最大 5.705%；HEOS 最大 0.156%（与上一行部分共享来源） |
| 二元实验 TP 液相 x | 两后端各 23 项均在绝对 0.02 内 |
| 二元实验 TP 汽相 y | PR 20/23 超限，最大绝对差 0.08076；HEOS 18/23 超限，最大 0.07841 |
| 二元实验构造 beta | 两后端均 23/23 超限；不能用相互接近替代实验一致 |
| 域内二元相态 | 45 点同相且各自相包络自洽；15 点模型相区不同；16 点参考 TP 标签与自身泡露点矛盾，单列参考异常 |
| 纯组分 | PR 在 280 K 丙烷、280 / 300 K 正丁烷的饱和液密度相对 HEOS 超过 5%；另 9 个 TP 相态标签与自身饱和压力矛盾 |
| 参考守恒抽查 | 45 个可比较点的组成归一最大残差 2.23e-16，组分分率重构最大残差 3.42e-14；不等于 B 的完整逸度 / 稳定性验证 |

### 两类待核验问题

1. **参考库标签异常。** [phase-diagnostic.json](phase-diagnostic.json) 的最小查询：300 K、200 kPa、z=0.5，PR 自身露点为 414.606 kPa，却把 TP 标为 liquid；其密度为 4.284725 kg/m³、Z=0.956419。诊断性强制 gas 返回同一密度；HEOS 自由 TP 标为 gas。纯流体另有相反方向的标签异常。保留原始结果，不强制改标签、不拿诊断分支顶替验收答案。需要解决版本 / API 的参考可信性后才能用它核验 B 的相选择；这里不声称已定位 CoolProp 根因。
2. **实验汽相组成不一致。** `je800377b/d3/p17` 在 283.15 K、216 kPa、液相 x=0.146 时，档案 y=0.322、编目扩展不确定度 0.007；PR / HEOS 在该 x 下的泡点 y 约 0.39609 / 0.39308，远大于所报不确定度。已复核组分编号、摩尔分率标签、单位和精确 T/x 连接。两模型共同偏离尚不能判定是模型、原实验还是档案转录问题；需独立第二套 VLE 或原表核验，禁止删除该组失败数据。

### 对既有工程计算的影响

[工程输入](engineering-inputs.tsv)与[输出](engineering-results.tsv)由 [c2a_separator.rs](../../../crates/rf-unitops/examples/c2a_separator.rs) 送入真实 S1/S2 公共 API。其他条件固定为公开算术 fixture 的 `K=0.1 m/s`、利用率 0.8、工作时间 120 s、选型最大工作液高 1 m；校核几何为面积 1 m² / 液高 0.6 m 及面积 2 m² / 液高 1 m。它们是显式假设，不是实际分离效率证据。

- 138 对最小直径全部在 5% 内，最大差 2.693%；选型控制项全部一致。
- 气负荷 24/138、液负荷 42/138 超过 10%，全部来自相界附近。最大相对差分别为 1,286.4% / 3,089.5%，来自微小相流量分母；应结合原始负荷和实际流量绝对差阅读，不能据此声称设备尺寸差同样巨大。
- 非相界点的气负荷最大差 3.890%、液负荷最大差 5.206%，仍有一个中部工况翻转：`interior-T280-z0.5-scale10-R01`（280 K、283238.609 Pa、z=0.5、100 mol/s），PR 液负荷 0.976421，HEOS 1.024947；液体准则及综合结论从 `WithinCriteria` 变为 `Exceeded`。这是**同一工况的两项变化**。
- 近消失相的 Q 单列绝对误差，共 24 项不作相对流量评分；工程负荷仍按原冻结 10% 目标记录失败，没有借近相界重新放宽门槛。

## 后续建议，待审定

保留原域为研究域，**暂不提出可用于产品的缩小有效域**。局部通过不能证明连续域可靠。建议下一轮仍限 A 的证据补强：核验 PR 参考标签、取得独立 VLE 与缺失气相密度，再单独比较有来源的液体密度改进路线。体积平移、改 `kij`、改混合规则、换 EOS 或缩域都会改变原假设，须另行审定，不在本轮实施。

B 的 PR 实现、逸度 / 稳定根、异常输入、组分置换、快照同源性和存储兼容尚未验证。原生保存窗口异常也不属于本验证包，仍未修复。

## 复现

在仓库根目录运行。首次创建隔离环境和下载数据需要网络；不会修改 Cargo 文件。精确 wheel URL / SHA-256 见环境记录；其他平台安装同版本可能采用不同 wheel，不能承诺跨平台逐字节相同。`--freeze` 仅用于首次建包，常规复现禁止重冻结。

```bash
python3 -m venv target/c2a-venv
target/c2a-venv/bin/python -m pip install -r tests/thermo-validation/c2a-pr76/requirements.txt
target/c2a-venv/bin/python tests/thermo-validation/c2a-pr76/thermoml.py --fetch --source-dir target/c2a-sources --output-dir target/c2a-reproduced
cmp tests/thermo-validation/c2a-pr76/experimental-data.json target/c2a-reproduced/experimental-data.json
target/c2a-venv/bin/python tests/thermo-validation/c2a-pr76/reference.py --output-dir target/c2a-reproduced
target/c2a-venv/bin/python tests/thermo-validation/c2a-pr76/assess.py --reference target/c2a-reproduced/reference-results.json --output-dir target/c2a-reproduced
cargo build --locked -p rf-unitops --example c2a_separator
target/debug/examples/c2a_separator < target/c2a-reproduced/engineering-inputs.tsv > target/c2a-reproduced/engineering-results.tsv
target/c2a-venv/bin/python tests/thermo-validation/c2a-pr76/assess.py --reference target/c2a-reproduced/reference-results.json --engineering-results target/c2a-reproduced/engineering-results.tsv --output-dir target/c2a-reproduced
target/c2a-venv/bin/python tests/thermo-validation/c2a-pr76/diagnose_phase.py --output target/c2a-reproduced/phase-diagnostic.json
target/c2a-venv/bin/python -m unittest discover -s tests/thermo-validation/c2a-pr76 -p 'test_*.py'
```

`assess.py` 成功退出表示报告成功生成，**不表示物理判据通过**；必须读取报告状态。原始来源 SHA 变化、参考版本 / 参数不匹配、数值非有限、实验连接不完整或工程结果缺行会失败。首次不带 `--engineering-results` 的调用只生成输入和暂态报告，第二次必须带工程结果，才是完整结果。

## 本次验证记录

2026-10-22 在上述环境完成两次独立生成：实验抽取、参考输出、工程输入 / 输出、评估和定向相态诊断均逐字节一致。8 项 Python 证据完整性测试覆盖单位、连接、域边界、非有限 / 零分母、缺行拒绝、2,882 点对账、276 行独立算术核对和 SHA 清单。正式 `check-repo`（格式、构建、workspace 测试、clippy、治理与文本）通过；既有 `block 0.1.6` future-incompat 提示仍在。未运行 Windows / Linux / COM 或新原生 UI 验收。

[manifest.json](manifest.json) 记录基线提交、脚本 / 参数 / 数值产物 / 报告和 Rust harness 的 SHA-256，另记录实际调用的计算模块摘要；按清单核对，不能以重新生成清单来掩盖漂移。该清单用于完整性复核，不是数字签名或物性准确性证明。
