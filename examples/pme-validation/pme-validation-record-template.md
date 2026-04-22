# PME Validation Record Template

建议文件名：`YYYY-MM-DD-<pme>-<scope>.md`

```text
Date:
Validator:
RadishFlow commit:
OS:
PME:
PME version:
PME bitness:
Registry scope:
Comhost path:
Dry-run command:
Registration command:
Unregistration command:
Preflight result:
Warnings accepted:
Register post-check:
Discovery:
Activation:
Identity:
Parameters:
Ports:
Connection:
Validate:
Calculate:
Report:
Unregister:
Unregister post-check:
Logs:
Decision:
Follow-up:
```

补充建议：

- `Register post-check` 记录三棵 registry tree 是否按顺序确认存在
- `Unregister post-check` 记录三棵 registry tree 是否按顺序确认删除
- `Logs` 至少写明本地 `artifacts/registration-validation/...` 目录、execution log 文件和任何 PME 截图/诊断输出的存放位置
- `Decision` 建议直接写 `Pass`、`Blocked` 或 `Fail`
