# C2-A 来源与分发说明

用途：记录本验证包的数据来源、许可依据与转换。读者：维护者和复核者。不包含：论文全文或产品运行时授权。

## NIST ThermoML 数值数据

`experimental-data.json` 是 NIST/TRC **ThermoML/Data Archive**（DOI [10.18434/mds2-2422](https://doi.org/10.18434/mds2-2422)）的数值摘录与单位转换。感谢原测量作者与 NIST/TRC 编目人员。逐项作者、题名、DOI、数据集编号、原 JSON URL 和 SHA-256 见 [sources.json](sources.json)。档案提取日期为 2020-09-29；本包取得及转换日期为 2026-10-22。

[官方目录元数据](https://catalog.data.gov/dataset/thermoml-data-archive)的 `license` 字段指向 [NIST 数据许可](https://www.nist.gov/open/license)。本包按该档案公开许可中的归属、修改标识及免责声明要求保留数值派生数据；不把它改标为仓库代码许可或 CC0，也不把论文出版许可延伸成全文再分发许可。关联档案说明论文为 [10.1002/jcc.26842](https://doi.org/10.1002/jcc.26842)。

修改说明：选择 `sources.json` 列出的 14 个数据集，保留全部 2,882 个数值记录及编目不确定度；压力由 kPa 乘 1000 转为 Pa；重命名字段并分配可回溯的点 ID。双性质按原报告 T/x 精确连接；其他转换和筛选见脚本与 `protocol.json`。没有复制论文摘要、正文、图表图像或原作者拟合代码。原始 JSON 由复现者下载到忽略的缓存。

以下保留 NIST 对非 SRD 数据的分发声明。该声明描述 NIST 的编目作品，不将原始实验作者误归为 NIST 员工。

> This data/work was created by employees of the National Institute of Standards and Technology (NIST), an agency of the Federal Government. Pursuant to title 17 United States Code Section 105, works of NIST employees are not subject to copyright protection in the United States. This data/work may be subject to foreign copyright.
>
> The data/work is provided by NIST as a public service and is expressly provided “AS IS.” NIST MAKES NO WARRANTY OF ANY KIND, EXPRESS, IMPLIED OR STATUTORY, INCLUDING, WITHOUT LIMITATION, THE IMPLIED WARRANTY OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT AND DATA ACCURACY. NIST does not warrant or make any representations regarding the use of the data or the results thereof, including but not limited to the correctness, accuracy, reliability or usefulness of the data.
>
> NIST SHALL NOT BE LIABLE AND YOU HEREBY RELEASE NIST FROM LIABILITY FOR ANY INDIRECT, CONSEQUENTIAL, SPECIAL, OR INCIDENTAL DAMAGES (INCLUDING DAMAGES FOR LOSS OF BUSINESS PROFITS, BUSINESS INTERRUPTION, LOSS OF BUSINESS INFORMATION, AND THE LIKE), WHETHER ARISING IN TORT, CONTRACT, OR OTHERWISE, ARISING FROM OR RELATING TO THE DATA (OR THE USE OR INABILITY TO USE THIS DATA), EVEN IF NIST HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH DAMAGES.

## CoolProp

参数和离线输出来自 CoolProp 8.0.0，commit `ae81610e7d23efc57f9d051c8e70a4d66e87537f`。PR / HEOS 结果是计算值，不是实验值；参考 EOS 作者键、混合物参数和 PR 常数出处见 [parameters.json](parameters.json)。原始 cubic JSON 摘要用于标识所选数据版本；本包仅保留所需参数，没有引入 CoolProp 实现源码或二进制。CoolProp 代码与所选参数随该版本按 MIT 许可提供，保留许可原文如下。

MIT License

Copyright (c) 2012-2018 Ian H. Bell and other CoolProp developers

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## 隔离环境

NumPy 2.5.4 是该 CoolProp wheel 的安装依赖（BSD-3-Clause；其 wheel 另附捆绑组件许可），不在本包分发。复现时安装器取得的 wheel 及其完整许可保留于本地虚拟环境；准确下载 URL / SHA-256 见 [environment.json](environment.json)。本次未更改 Cargo 依赖或产品部署内容。
