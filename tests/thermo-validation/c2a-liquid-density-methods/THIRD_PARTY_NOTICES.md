# C2-A 液体密度方法来源与许可

- `parameters.json` 的两行 `omega_SRK/Vchar/Z_RA` 数值选自 MIT 许可 ChEDL chemicals 1.5.2，固定 commit、原表 URL / SHA-256 见参数文件。完整许可归 [chemicals-LICENSE.txt](chemicals-LICENSE.txt)，保留原版权说明。只保存所选数值，不复制全表或上游实现。
- 参数表声称来源于 Hankinson / Thomson 的 COSTALD 工作；本次只核验官方仓库转录的列、CAS、数值和版本，未取得原论文参数表的可视认证。方法推导以原论文 DOI 为追溯，并以固定软件项目公开公式作为实际实现规范，不声称原作者全表已认证。
- [Hankinson & Thomson 1979](https://doi.org/10.1002/aic.690250412)、[Thomson / Brobst / Hankinson 1982](https://doi.org/10.1002/aic.690280420) 只作方法引用，不随仓库分发论文 PDF。作者公开会议材料 [Petree 等 1983](https://nfogm.no/wp-content/uploads/2019/02/1983-02-Calculation-of-Liquid-Densities-and-Their-Mixtures-Petree-Phillips.pdf) 的公开索引文本可追溯方法；本轮本地下载被拒，未绕过限制，未目视核验其表图。
- 修正 Rackett 的原始方法追溯 [Rackett 1970](https://doi.org/10.1021/je60047a012)；本轮具体混合式使用 [固定 ChEDL 源文档](https://github.com/CalebBell/chemicals/blob/7fca9acf7708e2a646bc04d2b6f6996d0325e77d/chemicals/volume.py) 的 `Rackett_mixture` 规范（其引用 Danner / DIPPR 1982），不分发商业手册。
- 原实验记录、CoolProp 常数与 PR / HEOS 输出只通过已冻结仓库包引用，许可及归属沿用 [首轮说明](../c2a-pr76/THIRD_PARTY_NOTICES.md)、[第二轮说明](../c2a-evidence-review/THIRD_PARTY_NOTICES.md)、[第三轮说明](../c2a-source-audit/THIRD_PARTY_NOTICES.md)。本轮没有取得新的实验数据或更改其不确定度。
- 新脚本没有复制外部项目实现；公开公式、两个组分参数和少量外部文档数值例用于可追溯检验。网络仅用于来源核对，常规复现完全离线。
