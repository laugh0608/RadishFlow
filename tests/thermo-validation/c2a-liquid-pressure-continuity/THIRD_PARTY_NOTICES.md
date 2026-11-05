# 压力连续性候选的来源与许可

- 方法规范沿用 ChEDL chemicals 1.5.2 固定 commit `7fca9acf7708e2a646bc04d2b6f6996d0325e77d` 的 [COSTALD_mixture_compressed](https://github.com/CalebBell/chemicals/blob/7fca9acf7708e2a646bc04d2b6f6996d0325e77d/chemicals/volume.py)。其公开公式在整个组成区间使用广义 Pcm，本包独立展开公式；不移植上游的域外兜底行为。公式文件 SHA-256 与前一包记录一致，见 environment.json。
- 专用 `omega_SRK/V*/Z_RA` 两行转录及全部参数原样引用上一包，参数文件逐字节保持，MIT 完整许可保存在 [chemicals-LICENSE.txt](chemicals-LICENSE.txt)。不复制全表或论文 PDF；原论文参数表尚无目视认证，不能把转录软件的例值或精度说明当成本包已测精度。
- 原论文 DOI、修正 Rackett 混合式、实验数据与 PR / HEOS 输出的来源和使用限制，沿用[前一包许可说明](../c2a-liquid-density-methods/THIRD_PARTY_NOTICES.md)及其引用的前三轮材料。本片没有新增外部数据、私有资料摘录或新分发权限。
- 物理 Pc 保留，统一广义 Pcm 仅为独立研究候选的压力关联规则；不能宣称新值是真实临界压力、真实泡点压力或整域稳定液相判据。
