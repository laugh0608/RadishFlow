# RadishFlow.CapeOpen.UnitOp.Mvp

当前目录已从纯占位推进为第一版最小 `net10.0` PMC 骨架项目，职责只限于：

- 提供一个最小 `ICapeIdentification` + `ICapeUtilities` + `ICapeUnit` 实现类
- 为后续真正的 CAPE-OPEN Unit Operation PMC 留出项目边界和最小状态机
- 提供最小内部 flowsheet/package 配置入口，并通过 `RadishFlow.CapeOpen.Adapter` 接入 `rf-ffi` 求解闭环

当前已包含的最小公共面：

- `RadishFlowCapeOpenUnitOperation`
- `UnitOperationPortPlaceholder` / `UnitOperationParameterPlaceholder`
- `UnitOperationPlaceholderCollection<T>`
- `Initialize / Validate / Calculate / Terminate / Edit` 的第一版状态骨架
- 内部 `LoadFlowsheetJson(...)`、`LoadPropertyPackageFiles(...)`、`SelectPropertyPackage(...)` 配置入口
- `SetPortConnected(...)` 这一类最小端口状态入口
- `ConfigureNativeLibraryDirectory(...)`、`LastCalculationResult`、`LastCalculationFailure`、`GetCalculationReport()`、`GetCalculationReportState()`、`GetCalculationReportHeadline()`、`GetCalculationReportDetailKeyCount()`、`GetCalculationReportDetailKey(int)`、`GetCalculationReportDetailValue(string)`、`GetCalculationReportLineCount()`、`GetCalculationReportLine(int)`、`GetCalculationReportLines()` 与 `GetCalculationReportText()`
- `ConfigureNativeLibraryDirectory(...)`、`LastCalculationResult`、`LastCalculationFailure`、`GetCalculationReport()`、`GetCalculationReportState()`、`GetCalculationReportHeadline()`、`GetCalculationReportDetailKeyCount()`、`GetCalculationReportDetailKey(int)`、`GetCalculationReportDetailValue(string)`、`GetCalculationReportLineCount()`、`GetCalculationReportLine(int)`、`GetCalculationReportLines()`、`GetCalculationReportText()`，以及公开 stable key catalog `UnitOperationCalculationReportDetailCatalog`
- `Calculate()` 对未满足前置条件的最小 ECape 语义抛错，以及经由 `rf-ffi` 的最小真实求解接线
- `ICapeCollection` / `ICapeParameter` / `ICapeUnitPort` 的第一版最小对象运行时
- placeholder 对象对 unit owner 生命周期的最小访问守卫，以及 `Terminate()` 时的端口连接释放

当前明确不包含：

- COM 注册 / 反注册
- 稳定 CLSID / ProgID 策略
- 报告接口的正式实现
- PME 生命周期集成
- 完整 CAPE-OPEN PMC 运行时

说明：

- 当前 `Ports` / `Parameters` 已返回带 `Item(object)` 和 `Count()` 的最小 `ICapeCollection` 风格对象，并支持按 `ComponentName` 或 1-based 索引取项
- 当前参数对象已提供最小 `ICapeParameter` + `ICapeParameterSpec` 语义，端口对象已提供最小 `ICapeUnitPort` 语义，但仍只覆盖 MVP 所需的字符串参数和占位连接对象
- 当前参数对象内部已补上最小元数据收口：区分 `StructuredJsonText` / `Identifier` / `FilePath` 三类值语义，保留对外 `ICapeParameterSpec.Type = CAPE_OPTION`，并显式记录默认值、是否允许空值与 manifest/payload 这类成对出现约束
- 当前端口对象已把连接契约收口到最小 `ICapeIdentification` 级别，并要求连接对象提供非空 `ComponentName`；当前仍不提前把运行时扩大到真正 `ICapeThermoMaterialObject`
- 当前 placeholder 对象已开始收口到宿主生命周期边界：`Terminate()` 后不再允许继续通过集合/参数/端口对象做 CAPE-OPEN 风格访问，并会释放端口上的已连接对象引用
- `Calculate()` 当前已能在最小前置条件满足后调用 `rf-ffi` 完成求解，并把对外结果面拆成“成功结果”和“失败摘要”两条最小契约：成功时导出 `LastCalculationResult(status / summary / diagnostics)`，失败时导出 `LastCalculationFailure(error / requestedOperation / nativeStatus / summary)`；完整 flowsheet snapshot JSON 与 native error JSON 仍只作为内部桥接输入，不再直接作为 PMC 公开结果面
- 当前又已补出 `GetCalculationReport()` 这一条统一只读查询面，把“尚无结果 / 最近成功 / 最近失败”收口到单一 report DTO；外部宿主若只需要稳定结果状态与结构化 detail，不必自己分支拼装 `LastCalculationResult` 和 `LastCalculationFailure`
- 在这条统一查询面之上，当前又补出 `GetCalculationReportState()` 与 `GetCalculationReportHeadline()` 两条最小标量元数据入口，让最小 host / PME 可以直接读取状态与标题，而不必先取出自定义 report DTO
- 在该元数据面之上，当前又补出 `GetCalculationReportDetailKeyCount()`、`GetCalculationReportDetailKey(int)` 与 `GetCalculationReportDetailValue(string)` 这一组最小 detail 键值读取入口，让宿主既可枚举当前稳定 detail key，又可按 key 直接读取值，而不必硬编码 key 列表或从展示文本里反解析
- 当前又已把 stable detail key 清单正式冻结到公开 catalog `UnitOperationCalculationReportDetailCatalog`：success 路径当前固定按 `status -> highestSeverity -> diagnosticCount -> relatedUnitIds -> relatedStreamIds` 排序；failure 路径当前固定按 `error -> operation -> requestedOperation -> nativeStatus -> diagnosticCode -> relatedUnitIds -> relatedStreamIds -> relatedPortTarget` 排序，其中部分 key 可能按场景缺省
- 在这条 report DTO 之上，当前又继续补出 `GetCalculationReportLines()` 与 `GetCalculationReportText()` 两条最小宿主可显示文本面，统一把 headline/detail lines 收口为可直接展示的行集合或多行文本，避免最小 host / PME 再自己手工拼接显示字符串
- 在上述文本面之上，当前又继续补出 `GetCalculationReportLineCount()` 与 `GetCalculationReportLine(int)` 两条标量读取入口，让最小 host / PME 可以按“行数 + 按索引读取”逐步消费报告文本，而不必依赖自定义 DTO 或一次性整段文本
- Rust/.NET 边界仍保持为句柄 + UTF-8 + JSON + 状态码，没有在这里提前引入 COM 注册或更宽的跨边界对象传递
