# C2-A 参考标签核验与实验证据补强

日期：2026-10-22。用途：供项目所有者审定第二轮 C2-A 的证据、失败和缺口。读者：维护者、物性验证与工程审查人员。不包含：C2-B / C3 产品实现、模型调参、缩域批准或原生保存修复。

## 审定结论

前轮结果已按授权提交为 `196c5d54`（`test(thermo): 完成 C2-A 独立验证与失败报告`）。[原包](../c2a-pr76/README.md)及其全部失败、阈值和 SHA-256 保持原样；本目录是追加证据，不生成覆盖原结论的新“通过率”。本轮结果仍为 **C2-A 门禁未通过，不能批准 B / C3 或声明连续适用域**。

- **相态标签核验完成到固定版本、固定状态集的诊断层级。** CoolProp 8.0.0 的 25 个入域异常均复现；密度与独立 PR 方程的稳定均相根相符，标记与该版本自身相包络冲突。源码提供明确的赋值 / 判定路径。没有修改 CoolProp，也未把诊断性强制相结果替换进原验证包。
- **独立 VLE 已补入。** Kayukawa 等 2005 的 ThermoML 数值来源独立于原 Seong 2008 来源；10 个入域配对点仍有超差，不能把原有冲突归结为单一来源问题。
- **正丁烷气相密度缺口部分补齐。** Glos 等 2004 的 5 个气相 PVT 点和 3 个饱和气相密度点均通过原 5% 判据；覆盖仅为 300–320 K、约 0.201–0.456 MPa 的离散点，不能外推整个候选域。
- **丙烷 / 正丁烷混合物的独立气相密度仍缺可评分的逐点数据。** 已核验公开文献线索与学位论文，记录为何未采用；模型之间的一致性不填补这一缺口。

新实验共 128 个比较项：103 pass、25 fail；比较项会复用测量点，不能当成 128 个独立实验。原 2,088 项的 247 fail、25 reference_inconsistent、2 review_required 等记录全部保留，原液体密度超差与 S2 单个工况的结论翻转均未消除。

## 1. 参考相态标签

输入是原包的全部 84 个二元 TP 状态及 18 个纯物质 TP 状态，共 102 个；包含原先排除的 8 个二元域外状态。诊断操作、版本和不改判据原则见 [protocol.json](protocol.json)。输出见 [8.0.0](phase-8.0.0.json)、[7.2.0](phase-7.2.0.json)及 [汇总](summary.json)。

### CoolProp 8.0.0

| 对照 | 结果 | 可得结论 |
| --- | --- | --- |
| fresh / 同一对象重复 TP / 不调用显式零参数 setter | 102 状态的三个原始快照完全一致 | 本批异常不是重复调用残留或零参数 setter 特有现象 |
| 原包自身包络审计 | 18 个二元、9 个纯物质标签冲突；其中 2 个二元在域外 | 对应原包 25 个入域异常，另完整记录 2 个域外异常 |
| 异常点与指定正确相的 TP 根比较 | 27 个密度完全一致 | 原返回密度本身没有因诊断改相而变化 |
| 独立 PR 三次方程 + 机械稳定性 + 均相 Gibbs 排序 | 27 个异常的密度相对差最大 `6.44e-14` | 在这些已知单相点，与相同参数的最小均相 Gibbs 根一致 |
| 9 个纯液体 TP → 同密度 DmolarT | 密度不变，标记从 gas 变为 liquid | 同一物理状态的标记依赖更新路径 |

独立代数只验证 PR 均相方程和根，不是二元全局相稳定性证明，也不是新 Flash 实现。不能据此认定所有 CoolProp 输出正确。强制 gas / liquid 查询仅是对照，包含亚稳根的可能性；不会作为正式相判据或回退。

**二元路径：** 固定 commit 的 `CubicBackend.cpp:606` 在 `solver_rho_Tp_global` 中按 `rho < rhomolar_reducing()` 标记气 / 液。本批 PR 二元对象的 reducing density 为 `1 mol/m³`，18 个实际气相根密度均大于它，因此被标为 liquid。例：300 K、200000 Pa、z=0.5，返回 `83.83519163708297 mol/m³`、`4.284725264212426 kg/m³`，原始标签 liquid；自身露点约 414606 Pa，状态在气区。

**纯物质路径：** `CubicBackend.cpp:347` 将 `solver_rho_Tp` 返回值赋给 `_rhomolar`，但求根函数在返回前（约 675 行）已调用 `recalculate_singlephase_phase()`。此前 `pre_update` / `clear()` 将 `_rhomolar` 清为 `-_HUGE`；`HelmholtzEOSMixtureBackend.cpp:233` 又读取对象中的 `rhomolar()` 判气 / 液。源码执行顺序结合同密度 DmolarT 对照，支持“赋值前读取旧密度造成纯液体标签错误”的诊断。未打补丁或使用调试器插桩，不将此扩展为全版本结论。

完整源文件 URL、固定 commit 与摘要见 [diagnostic-sources.json](diagnostic-sources.json)，[CoolProp 8.0.0 CubicBackend 官方源码](https://github.com/CoolProp/CoolProp/blob/ae81610e7d23efc57f9d051c8e70a4d66e87537f/src/Backends/Cubics/CubicBackend.cpp)。

### 7.2.0 不是自动替代参考

隔离版本 `7.2.0 / 98b3523d5daa98454618d381d2ae53f7471d216b` 对 102 个无约束 TP 输入只有 89 个返回结果、13 个报错；89 个均标 gas，包括原包的纯液体状态，且未返回 two_phase。18 个纯物质 DmolarT 对照均报 `type not set`。这些错误原样保存。

7.2.0 使用 `a=0.45724`、`b=0.07780` 的舍入常数，8.0.0 使用原包冻结的精确临界约束常数。独立代数按各自常数计算并在输出中显式标识；不能把跨版本密度差当成同模型实现误差。**不降级主参考、不删除 8.0.0 失败、不自动修补标签。**

## 2. 新增实验与逐点结果

[sources.json](sources.json)冻结来源 URL / DOI、作者、归档日期、文件哈希、数据集和单位转换；[experimental-data.json](experimental-data.json)保留选中 5 个数据集的全部 131 个原数值记录。[experimental-results.json](experimental-results.json)记录 18 个入域案例、73 个排除案例和 15 个未配对记录；配对案例可能占两条测量记录，测试保证 131 条逐条守恒且不重复。

### 独立 VLE：Kayukawa、Fujii、Higashi（2005）

[NIST ThermoML / DOI 10.1021/je0496701](https://trc.nist.gov/ThermoML/10.1021/je0496701.html)，只选 propane / n-butane 的 d1 与 d2。17 条压力、16 条气相组成按原 T 和液相丙烷摩尔分数精确连接；`d1/p15` 无对应 y，保留为未配对，不插值。16 对中 10 对满足原 T/P/构造 z 候选域，6 对显式排除。

构造 `z=(x+y)/2`、`β=0.5` 仍只用于检验同一 tie-line 的 TP 一致性，**不是测得的流量或气化率**。直接泡点 P/y 与 TP x/y/β 分列，判据直接读取原包 `protocol.json`，不因实验不确定度修改门槛。

| 比较项 | PR 通过/总数；最大误差 | HEOS 通过/总数；最大误差 | 原判据 |
| --- | --- | --- | --- |
| 泡点 P | 8/10；4.1714% | 8/10；4.1804% | 相对 2% |
| 泡点 y | 8/10；0.06055 | 8/10；0.05870 | 绝对 0.02 |
| TP x | 9/10；0.02668 | 9/10；0.02664 | 绝对 0.02 |
| TP y | 9/10；0.03002 | 9/10；0.02800 | 绝对 0.02 |
| TP 构造 β | 3/10；0.06426 | 4/10；0.05458 | 绝对 0.02 |

两组明显冲突是 `d1/p1`（310 K、471100 Pa、x=0.1692、y=0.355）与 `d1/p4`（300 K、449500 Pa、x=0.309、y=0.5339）。源 JSON 的组分、相、单位、变量键和原始数值均核验；原始期刊表格尚未取得，不能断言是模型错误或编目错误，也不排除这两点以抬高成绩。其余点虽然 P/y 大多较好，TP 构造 β 仍有失败。

“独立”指与首轮 Seong 2008 是不同测量来源，**不等同于对 HEOS 的盲验集**。[GERG 专著](https://www.gerg.eu/wp-content/uploads/2019/10/TM15.pdf)附录 A2.1（PDF 412 页 / 印刷 392 页）将 Kayukawa 2005b 列为对照数据，拟合使用栏为空；这只能说明该专著所述模型的组织，不能替代对当前 HEOS 全部数据沿革的确认。原 Seong 失败仍保留。

### 正丁烷气相密度：Glos、Kleinrahm、Wagner（2004）

[NIST ThermoML / DOI 10.1016/j.jct.2004.07.017](https://trc.nist.gov/ThermoML/10.1016/j.jct.2004.07.017.html)，选择正丁烷 d11（饱和 P）、d13（饱和气体密度）、d14（PVT）。按原温压筛选；泛称 Fluid 的 PVT 用测量密度相对冻结临界密度区分观测气支，不依赖待验证 EOS 的标签。

| 证据 | 入域数 | PR 最大相对误差 | HEOS 最大相对误差 | 结果 |
| --- | --- | --- | --- | --- |
| 气相 PVT 密度 | 5 | 0.4168% | 0.0172% | 全部通过 5% |
| 饱和气相密度（各模型自身饱和 P） | 3 | 1.0013% | 0.0274% | 全部通过 5% |
| 饱和点在实测 P 的指定气支密度 | 3 | 0.5961% | 0.0338% | 全部通过 5% |
| 配对饱和 P | 3 | 0.3762% | 0.0101% | 全部通过 2% |

后两项复用前三个饱和测量，不增加独立样本数。3 个饱和气相点为 300/310/320 K；5 个 PVT 点为 300/310/320 K、200575–399703 Pa。保留原编目扩展不确定度；按原 5% 精度判据通过并不意味着误差落在实验不确定度内。

该实验早于 HEOS 的 Buecker–Wagner 2006 正丁烷 EOS，未证明未参与其建立或评估，故 HEOS 对照不宣称盲验。对 PR 的实验对照也仅证明这些离散点，不验证混合物气相、全域或液体密度。

## 3. 混合物气相密度的剩余缺口

已检查 ThermoML 候选来源、GERG 附录和 Nysewander 原始研究线索。已有 Kayukawa / Miyamoto / Parrish 数据主要覆盖液相，不能改标为气相证据。

GERG A2.1 列出 Nysewander、Sage、Lacey（1940）的二元 pρT 数据，部分温压范围与候选域重叠；期刊 DOI [10.1021/ie50361a026](https://doi.org/10.1021/ie50361a026)在本次环境中未取得全文（403）。公开的 [Nysewander 1938 学位论文](https://thesis.caltech.edu/16876/1/Nysewander_CW_1938.pdf)已完整取得并检查 39 页，重点渲染核验 PDF 21–22 页：正文说明体积测量主要用于确定相界，只给出一组 P-v 曲线，未提供足以直接纳入本包的逐点气相密度表。没有把图中两相区体积、文献统计误差或模型输出转写为实测气相密度，也未把 1938 论文当成已核验的 1940 期刊表。

因此本轮**不能声称混合物气相密度缺口已补齐**。下一步仍应取得可追溯的原始 PρTx 气相数据及不确定度，或审定另选具有公开数据的验证体系；在此之前不能以纯气体和模型间对照替代混合物物理证据。

## 4. 复现与验证

全部依赖为离线验证用途，产品 Cargo 依赖、运行时、数据模型、接口及 S1/S2 实现均未修改。[environment.json](environment.json)含 Python、平台、wheel URL 与 SHA-256。沿用前轮 `CoolProp==8.0.0`、`numpy==2.5.4`；7.2.0 通过独立路径加载。

在仓库根执行（首次安装 / 下载需网络；若已有原环境则跳过安装）：

```bash
python3 -m venv target/c2a-venv
target/c2a-venv/bin/python -m pip install -r tests/thermo-validation/c2a-pr76/requirements.txt
target/c2a-venv/bin/python -m pip install --no-deps --target target/c2a-coolprop72 CoolProp==7.2.0
mkdir -p target/c2a-review-repeat
target/c2a-venv/bin/python tests/thermo-validation/c2a-evidence-review/experiments.py --source-dir target/c2a-review-sources --fetch --output-dir target/c2a-review-repeat
target/c2a-venv/bin/python tests/thermo-validation/c2a-evidence-review/phase_audit.py --output target/c2a-review-repeat/phase-8.0.0.json
PYTHONPATH=target/c2a-coolprop72 target/c2a-venv/bin/python tests/thermo-validation/c2a-evidence-review/phase_audit.py --output target/c2a-review-repeat/phase-7.2.0.json
target/c2a-venv/bin/python -m unittest discover -s tests/thermo-validation/c2a-evidence-review -p 'test_*.py'
target/c2a-venv/bin/python -m unittest discover -s tests/thermo-validation/c2a-pr76 -p 'test_*.py'
```

`--freeze` 仅用于首次来源冻结且拒绝覆盖；复现使用冻结 SHA。相态文件含 Python / 平台信息，跨环境复现应分别检查元数据和数值，不承诺跨平台逐字节一致。源码与 PDF 只读核验按 `diagnostic-sources.json` 的 URL / hash 复查，不要求将全文收入仓库。许可和转换说明见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。

本机已完成两次独立进程生成并比较全部 4 份数据 / 参考输出，逐字节一致；原包 8 项、新包 9 项 Python 测试、仓库治理、文本、文档篇幅与 `git diff --check` 均通过。新包测试覆盖原包不变、131 条测量守恒、缺失配对 / 重复键拒绝、判据与失败保留、相态诊断、独立根代回原 EOS 和 7.2.0 失败保留。此轮仅 Python 验证与文档，未重复前轮已通过的 1,625 项 Rust 全量检查，未做 UI / COM smoke。

## 5. 待审定的后续范围

建议继续限定 C2-A：取得两份 VLE 原始表格核对冲突点，并补齐可评分的混合物气相 PρTx 数据；随后再审定参考版本 / 相态资格方案。当前已定位的标签问题可用于准备上游最小复现，但本轮未向上游发送消息，也未采用本地补丁。

模型参数拟合、体积平移、缩域、模型改选及任何“合格参考”替换均需另行审定；本轮没有实施。原生保存异常仍未修复，B / C3 继续不实施。
