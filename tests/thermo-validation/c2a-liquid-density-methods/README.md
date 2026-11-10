# C2-A：液体密度离线方法对比

2026-11-04 按已认可的后续方案实施；结果待所有者审阅。范围为丙烷 / 正丁烷的 COSTALD、修正 Rackett 与冻结 PR76 对照，随后保留 PR 相平衡和气相性质，计算实际液体流量与既有 S1/S2 敏感性。没有产品依赖、运行时、接口或项目格式变更，C2-B / C3 未开工。

## 参数、公式和来源资格

[protocol.json](protocol.json) 冻结方法、混合规则、压力修正、域检查、原判据和基线摘要；[parameters.json](parameters.json) 给出所有输入。未拟合参数、估算缺参、调整 PR alpha / kij 或引入体积平移。

- `M/Tc/Pc/R` 原样使用首轮 CoolProp 8.0.0 参数。COSTALD 的 `omega_SRK/V*` 和 Rackett 的 `Z_RA` 是专用关联参数，来自 ChEDL chemicals **1.5.2** 的固定 commit `7fca9acf7708e2a646bc04d2b6f6996d0325e77d`，不是用 PR 的 Pitzer omega 或 `1/rhoc` 替代。两个来源各自承载不同量；这是一组明确冻结的候选参数组合，不声称重建了 1979 年原作者的全部常数组合。
- [官方参数转录表](https://github.com/CalebBell/chemicals/blob/7fca9acf7708e2a646bc04d2b6f6996d0325e77d/chemicals/Density/COSTALD%20Parameters.tsv) 的原列为 `CAS/Chemical/omega_SRK/Vchar/Z_RA`：丙烷 `0.1532/0.0002001/0.2766`，正丁烷 `0.2008/0.0002544/0.273`；体积单位 m³/mol。原 TSV SHA-256 为 `32a43781673544c0e0d09d2f1dd58c4632b32f7f9128cd0335af4b0832f2168b`，全表不入库；保留两行数值和完整 MIT 许可。
- COSTALD 饱和体积使用 [Hankinson–Thomson 1979](https://doi.org/10.1002/aic.690250412) 方程；`V*m=¼[ΣxV*+3Σx(V*)^(⅔)Σx(V*)^(⅓)]`、`Tcm=[Σx√(TcV*)]²/V*m`、omega 摩尔平均，交互项明确为零。
- 修正 Rackett 使用拟合 `Z_RA`，采用 [ChEDL 1.5.2 的 Rackett_mixture](https://github.com/CalebBell/chemicals/blob/7fca9acf7708e2a646bc04d2b6f6996d0325e77d/chemicals/volume.py) 所记质量加权混合式，Tc 和 ZRA 摩尔平均；没有用临界压缩因子填补 ZRA。
- 两者均使用 [Thomson–Brobst–Hankinson 1982](https://doi.org/10.1002/aic.690280420) 的 Tait 体积因子。修正 Rackett＋Tait 是本次明确组合的筛查候选，不能称为 Rackett 自带或已验证的压力模型。压力伪临界参数复用 COSTALD 混合；纯物质用原 Pc，二元用 `(0.291−0.080 omega_SRK) R Tcm/V*m`。Ps 采用上述固定软件版本记载的广义 Riedel 式，系数固定为 `4.86601`，不是外部 PR / HEOS 饱和压力。
- 原论文参数表的合法全文 PDF 尚未取得可视核验；公开作者材料 / 出版社摘要及官方软件转录提供本轮方法依据。**参数转录追溯不等于原表全表认证**，不得把本轮结果标为原作者方法的完整独立认证。许可和核对层次见 [第三方说明](THIRD_PARTY_NOTICES.md)。

脚本 [methods.py](methods.py) 仅用 Python 标准库独立展开公开公式，没有调用 RadishFlow 物性实现或安装 chemicals。公式例值、纯物质极限、置换、单调性和拒绝行为单独验证；例值一致不代替物理准确性验证。

## 输入、域和不可用状态

原 [C2-A 数据包](../c2a-pr76/README.md)、[第二轮](../c2a-evidence-review/README.md)及[第三轮](../c2a-source-audit/README.md)的文件与 manifest 未改写。重放检查全部原清单；唯一已知仓库漂移是后续 AF2b 提交 `b09873b2` 改动的 Cargo.lock，明确冻结旧 / 当前摘要并说明原因。原 harness 与 S1/S2 源模块摘要仍吻合；这不是首轮构建环境逐字节重建。

- 在首轮已选实验中保留全部 **47 个液体密度工况**，包括纯液体、二元压缩液体和明确饱和 / 泡点液体；给定实验 T/P/x，不先用 EOS 重算液相组成。来源、不确定度、配对记录和原排除点通过原清单追溯；未取得新的实验数据，也没有重选首轮失败点。
- 实验域保持 280–320 K、压缩输入 0.2–1.0 MPa、二元实验液相 x=0.2–0.8；明确饱和观测沿用只按 T 选择的原约定。饱和观测直接比较饱和体积，测量压力留在结果中，不用其反向拟合压力修正。
- 两候选要求 `0.25 <= T/Tcm <= 0.95`；压缩输入要求 `P >= Ps_correlation`，不夹紧 P、不回退饱和密度。工程部分使用 PR 的实际相组成（可超出实验选点 x 范围），不据此宣称连续域已验证。
- 唯一不可用实验点 `j.jct.2006.07.012/d1/p1`：丙烷 280 K / 581400 Pa，关联 Ps=584624.915 Pa。两候选都记不可用；不将压力差自动归因于实验错误，也不删除该点或放宽压力门槛。原 PR / HEOS 的该点输出保持。

## 实测结果

[results.json](results.json) 保留逐点观测、原 PR / HEOS 密度、候选值 / 不可用原因、相流量和所有工程比较。原密度 5%、相流量 10%、直径 5%、负荷 10% 判据不变。

| 比较 | COSTALD＋Tait | 修正 Rackett＋Tait |
| --- | --- | --- |
| 47 个实验液体工况 | 46 pass、1 unavailable、0 fail | 46 pass、1 unavailable、0 fail |
| 可评分实验最大密度偏差 | 0.3072% | 0.7236% |
| 同一 23 个两相点，实际 PR 液相组成，液体密度相对 HEOS 最大差 | 0.0933% | 0.6097% |
| HEOS beta 在 0.1–0.9 的 11 点实际液体 Q | 全部 ≤10%，最大 3.5272% | 全部 ≤10%，最大 4.1090% |
| 另外 12 个近消失相点 | 保留绝对 Q 差，Q 相对判据 unscored | 同左 |
| S1 最小直径相对 HEOS，138 对 | 全部 ≤5%，最大 1.2991% | 全部 ≤5%，最大 1.2813% |
| S2 气负荷相对 HEOS | 24/138 超过 10% | 24/138 超过 10% |
| S2 液负荷相对 HEOS | 42/138 超过 10% | 42/138 超过 10% |

最大实验偏差均在 `j.jct.2007.06.014/d2/p3`。同组 47 点的原 PR 最大偏差仍为 **5.7049%**；新方法没有改写原失败。HEOS 是模型对照，且可能使用过相关实验作开发资料，不能写成新盲验实验或独立物理真值。

### 工程敏感性

沿用首轮 23 个双方自洽且两相的工况、0.1 / 1 / 10 倍总流量和两种几何；原 PR / HEOS 276 行原样重放，新增两候选各 138 行，合计 **552 次**真实 S1/S2 公共 API 计算。PR beta、液相 x、气相密度和气相 Q 保持；候选仅按 `Q_liq=n_liq V_liq` 更新实际液体流量 / 密度。`rho_liq × Q_liq` 逐行保持相质量流量。

[run_engineering.py](run_engineering.py) 将数值传入未修改的旧 [harness](../../../crates/rf-unitops/examples/c2a_separator.rs)：旧 `backend` 只允许 PR/HEOS，是输出标签，不选择计算公式。适配器仅在传输时使用临时 ID / PR 标签，在严格核对行数、顺序和身份后恢复真实候选名；永久输入 / 输出保留候选名。没有用这个标签声称候选属于 PR。原 276 行工程输出与首轮逐字段完全相同。

近相界的负荷相对误差仍可非常大：COSTALD 相对 HEOS 气负荷最大约 1315%、液负荷约 3249%，不能用液体密度改善核销相分率分歧。非相界液负荷最大差分别为 3.5272% / 4.1090%。

`interior-T280-z0.5-scale10-R01` 原 PR 液负荷 0.976421 / WithinCriteria，HEOS 1.024947 / Exceeded；COSTALD 1.029530、Rackett 1.033898，均为 Exceeded。两候选相对原 PR 都发生液体准则 / 综合结论翻转（一个工况、两项状态），需审阅。虽在这 138 对中均与 HEOS 的分类一致，也不能据此放行工程应用。

## 结论与剩余门槛

支持继续优先研究 COSTALD 液体密度，修正 Rackett 保留对照；本轮可交付为**带来源限制和不可用点的离线比较报告**。尚无获准的连续适用域，第三轮来源审计和本轮结果均待审阅。

VLE 汽相组成冲突、原 PR TP 标签资格、二元实验气相密度、原文参数表认证、不可用压力点和工程结论翻转分别保持。PR 体积平移未进入数值比较；焓熵、PH/PS 和动态没有资格推论。C2-B / C3 不实施；如需进入产品，仍须按正式物性专题审定类型、能力、兼容与适用域。

## 离线复现

已有 Python 标准库即可计算；不下载数据、不安装工具，不改 Cargo.toml / Cargo.lock。实际 Python / 平台、上游公式文件摘要归 [environment.json](environment.json)，其他平台不承诺数学库逐字节一致。先生成输入，再使用当前离线锁定的旧 harness，最后形成完整报告；首个命令只生成暂态报告，成功退出不表示物理门槛通过。

```bash
python3 tests/thermo-validation/c2a-liquid-density-methods/compare.py --output-dir target/c2a-liquid-replay
CARGO_NET_OFFLINE=true cargo build --locked -p rf-unitops --example c2a_separator
python3 tests/thermo-validation/c2a-liquid-density-methods/run_engineering.py --inputs target/c2a-liquid-replay/engineering-inputs.tsv --harness target/debug/examples/c2a_separator --output target/c2a-liquid-replay/engineering-results.tsv
python3 tests/thermo-validation/c2a-liquid-density-methods/compare.py --output-dir target/c2a-liquid-replay --engineering-results target/c2a-liquid-replay/engineering-results.tsv
cmp tests/thermo-validation/c2a-liquid-density-methods/results.json target/c2a-liquid-replay/results.json
cmp tests/thermo-validation/c2a-liquid-density-methods/engineering-inputs.tsv target/c2a-liquid-replay/engineering-inputs.tsv
cmp tests/thermo-validation/c2a-liquid-density-methods/engineering-results.tsv target/c2a-liquid-replay/engineering-results.tsv
python3 -m unittest discover -s tests/thermo-validation/c2a-liquid-density-methods -p 'test_*.py'
```

[manifest.json](manifest.json) 固定本包代码、参数、协议、许可与产物摘要；常规复现不重冻结清单。八项证据测试覆盖公式外部例值、专用参数、置换 / 压力响应、非法 / 域外拒绝、来源 / 点数、完整重放与缺行拒绝、相质量流量和原 276 行保持，以及摘要。两次独立完整重放逐字节一致，详见 W45。
