# PME Validation

该目录存放外部 PME 人工验证记录、验证模板与后续补充说明。

当前约定：

- 目标 PME 的每次人工验证都应在此目录留档
- 建议文件名使用 `YYYY-MM-DD-<pme>-<scope>.md`
- 可直接复制 `pme-validation-record-template.md` 作为首版记录
- 记录内容应与 `docs/capeopen/pme-validation.md` 中的通过标准、失败分类和运行手册保持一致

当前目录不存放：

- 自动化 PME 驱动脚本
- 第三方 PME 安装包
- Windows Registry 导出文件或本地执行日志

本机 register / unregister 备份与 execution log 继续放在本地 `artifacts/registration-validation/`，不提交到 Git。
