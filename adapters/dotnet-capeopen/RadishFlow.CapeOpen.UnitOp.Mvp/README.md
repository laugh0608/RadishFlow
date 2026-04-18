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
- `UnitOperationHostReportReader.Read(...)`、`UnitOperationHostReportSnapshot` 与 `UnitOperationHostReportDetailEntry`，用于让外部最小 host 基于既有公开 report API 一次性材料化状态、stable detail entries、scalar lines、vector lines 与 text，而不必在每个宿主里重复写同样的读取样板
- `UnitOperationHostReportPresenter.Present(...)` 与 `UnitOperationHostReportPresentation`，用于把 host snapshot 继续整理成更接近 UI / 日志组件的展示模型，明确 `StateLabel`、`RequiresAttention`、`StableDetails` 与 `SupplementalLines`
- `UnitOperationHostReportFormatter.Format(...)`、`UnitOperationHostReportDocument` 与 `UnitOperationHostReportSection`，用于把 presentation 收口成固定 section 输出，便于宿主直接渲染 `Overview / Stable Details / Supplemental` 这类展示分区
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
- 当前 `ICapeCollection.Item(object)` 已冻结为“1-based 整数索引或 component name”双入口；除 `int/long` 外，也接受能无损落到整数的 `double/float/decimal` 选择子，以贴近 COM 宿主可能传入的数值 Variant 形状；空白名称、越界索引和非整数数值仍按 `CapeInvalidArgumentException` 拒绝
- 当前 `ICapeCollection`、`ICapeParameter` 与 `ICapeUnitPort` 的 `ComponentName/ComponentDescription` 已冻结为运行时不可变元数据；宿主可以重复读取，但不能在 MVP runtime 中修改这些标识字段，从而保持 collection lookup、required port 规则与 stable detail key 不漂移
- 当前 `UnitOperationParameterCatalog` / `UnitOperationPortCatalog` 已进一步从“名字常量表”推进到“完整定义真相源”，把 canonical name、collection order、description、required/value-kind/mode 与 direction/port-type 一并收口，避免这些宿主契约继续散落在构造函数和测试字面量里
- 当前参数对象内部已补上最小元数据收口：区分 `StructuredJsonText` / `Identifier` / `FilePath` 三类值语义，保留对外 `ICapeParameterSpec.Type = CAPE_OPTION`，并显式记录默认值、是否允许空值与 manifest/payload 这类成对出现约束
- 当前参数对象又已把 `Specification` 从参数实例本身分离为独立只读 spec 对象；宿主重复读取时拿到稳定 spec 引用，而不再让 `ICapeParameter` 与 `ICapeParameterSpec` 混成同一个运行时对象
- 当前参数对象的 `Mode` 也已冻结为运行时不可变元数据；MVP runtime 当前只接受初始化时定义好的 mode，宿主不能在运行过程中把 input 参数改写成 output/input-output
- 当前参数对象的 `Value` / `IsConfigured` 读取也已纳入 owner lifecycle guard；`Terminate()` 后不再允许继续通过这些 helper 旁路访问对象状态
- 当前参数对象的 `Reset()` 语义也已收紧为“无论默认值是否与当前值相同，都把 `ValStatus` 复位到 `NotValidated`”，避免宿主在默认值场景下保留过期 validation 状态
- 当前端口对象已把连接契约收口到最小 `ICapeIdentification` 级别，并要求连接对象提供非空 `ComponentName`；当前仍不提前把运行时扩大到真正 `ICapeThermoMaterialObject`
- 当前端口对象的连接替换语义也已冻结：重复连接同一对象视为 no-op，但若端口已连接到另一对象，则必须先显式 `Disconnect()`，不能静默替换既有连接
- 当前 placeholder 对象已开始收口到宿主生命周期边界：`Terminate()` 后不再允许继续通过集合/参数/端口对象做 CAPE-OPEN 风格访问，并会释放端口上的已连接对象引用
- 当前 `RadishFlowCapeOpenUnitOperation` 内部生命周期又已从分散的 `_initialized / _terminated / _disposed` 布尔位收口为单一 `UnitOperationLifecycleState`，让 `Initialize / Validate / Calculate / Terminate / Dispose`、placeholder access guard 与终止后只读 report 查询共享同一套阶段判断，避免宿主边界继续靠多处布尔分支隐式维持
- 当前 `Calculate()` 内部执行链也已从单方法内联流程收口为显式分段：先做 `PrepareForCalculation()` 前置校验，再构造 `CalculationInputs`，随后执行 native solve、材料化结果并记录失败摘要；这样后续继续扩展宿主/PME 调用路径时，不必再从一段混合逻辑里追踪“校验失败、native 失败、contract parse 失败”分别落在哪个阶段
- 当前 `ValStatus` 与 calculation report 相关内部状态也已继续收口到正式变更入口：`ApplyValidationOutcome()`、`ResetCalculationState()`、`RecordCalculationSuccess()` 与 `RecordCalculationFailure()` 负责统一推进 `ValStatus`、`LastCalculationResult`、`LastCalculationFailure` 与 report 空态，避免 `Initialize / Validate / Calculate / Terminate` 继续各自零散写字段
- `Calculate()` 当前已能在最小前置条件满足后调用 `rf-ffi` 完成求解，并把对外结果面拆成“成功结果”和“失败摘要”两条最小契约：成功时导出 `LastCalculationResult(status / summary / diagnostics)`，失败时导出 `LastCalculationFailure(error / requestedOperation / nativeStatus / summary)`；完整 flowsheet snapshot JSON 与 native error JSON 仍只作为内部桥接输入，不再直接作为 PMC 公开结果面
- 当前又已补出 `GetCalculationReport()` 这一条统一只读查询面，把“尚无结果 / 最近成功 / 最近失败”收口到单一 report DTO；外部宿主若只需要稳定结果状态与结构化 detail，不必自己分支拼装 `LastCalculationResult` 和 `LastCalculationFailure`
- 在这条统一查询面之上，当前又补出 `GetCalculationReportState()` 与 `GetCalculationReportHeadline()` 两条最小标量元数据入口，让最小 host / PME 可以直接读取状态与标题，而不必先取出自定义 report DTO
- 在该元数据面之上，当前又补出 `GetCalculationReportDetailKeyCount()`、`GetCalculationReportDetailKey(int)` 与 `GetCalculationReportDetailValue(string)` 这一组最小 detail 键值读取入口，让宿主既可枚举当前稳定 detail key，又可按 key 直接读取值，而不必硬编码 key 列表或从展示文本里反解析
- 当前又已把 stable detail key 清单正式冻结到公开 catalog `UnitOperationCalculationReportDetailCatalog`：success 路径当前固定按 `status -> highestSeverity -> diagnosticCount -> relatedUnitIds -> relatedStreamIds` 排序；failure 路径当前固定按 `error -> operation -> requestedOperation -> nativeStatus -> diagnosticCode -> relatedUnitIds -> relatedStreamIds -> relatedPortTarget` 排序，其中部分 key 可能按场景缺省
- 在这条 report DTO 之上，当前又继续补出 `GetCalculationReportLines()` 与 `GetCalculationReportText()` 两条最小宿主可显示文本面，统一把 headline/detail lines 收口为可直接展示的行集合或多行文本，避免最小 host / PME 再自己手工拼接显示字符串
- 在上述文本面之上，当前又继续补出 `GetCalculationReportLineCount()` 与 `GetCalculationReportLine(int)` 两条标量读取入口，让最小 host / PME 可以按“行数 + 按索引读取”逐步消费报告文本，而不必依赖自定义 DTO 或一次性整段文本
- 基于上述公开 report API，当前又补出 `UnitOperationHostReportReader.Read(...)` 这一层最小宿主 helper，把 stable detail entries、scalar/vector lines 与 text 的读取样板前推到库内；这样后续外部 host / PME 若只需要消费稳定结果展示面，可以直接复用 helper，而不必在每个入口重复写 detail key 枚举和 line/text 收集逻辑
- 基于 host snapshot，当前又补出 `UnitOperationHostReportPresenter.Present(...)`，把“稳定 detail 行”和“补充展示行”拆开，并显式前推 `StateLabel / RequiresAttention / HasStableDetails / HasSupplementalLines` 这类更接近宿主 UI 和日志组件的展示语义，避免每个宿主再各自推断 failure 高亮、success 附加诊断区或 idle 空态标签
- 基于 presentation，当前又补出 `UnitOperationHostReportFormatter.Format(...)`，把宿主展示继续收口为固定 section 文档；这样最小 host 不只拿到字段化语义，还能直接按 section 渲染 overview、stable details 与 supplemental diagnostics，而不必每个宿主再自己决定分区标题和文本拼接顺序
- 当前“宿主如何驱动 PMC”这条最小 orchestration helper 仍故意留在 `RadishFlow.CapeOpen.SmokeTests`：先用 `UnitOperationSmokeHostDriver` 验证正式调用顺序、最小必需输入和失败分类是否稳定，再决定是否有必要把 driver 上移到库内；`UnitOp.Mvp` 本身当前继续只负责 PMC 对象面与结果读取/展示 helper，不在这一轮提前承诺宿主驱动 convenience API
- 当前又已补出同目录层级的自举 contract test 入口 `RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests`，用于在不依赖外部 NuGet 测试框架的前提下，直接锁定 `Validate/Calculate/Terminate/report transition` 这类库侧行为契约；当前已覆盖 `Validate before Initialize`、validation failure report、native failure report、success report、配置变更 invalidation 与 `Terminate()` 后阻断 6 条核心 case；后续若继续冻结 `UnitOp.Mvp` 对外行为，应优先在这里补细粒度 contract case，而不是只依赖 smoke console 间接覆盖
- 当前 contract tests 又已继续前推到对象面本身，新增 collection selector、parameter reset/lifecycle access 与 port reconnect 约束这三组契约，确保“最小宿主对象运行时”不再只靠 smoke 路径间接覆盖
- 在上述对象面 contract tests 基础上，当前又已补上 spec 对象稳定性、post-terminate spec access guard 与 parameter mode immutability 三条参数语义约束，避免后续再次把参数对象回退成“值对象和 spec 对象合一”的松散实现
- 当前 `SmokeTests` 与 `ContractTests` 也已切到消费同一份 parameter/port catalog；后续若 canonical order 或对象定义继续演进，应优先改 catalog，而不是在多个 host/test 入口各自追字面量
- Rust/.NET 边界仍保持为句柄 + UTF-8 + JSON + 状态码，没有在这里提前引入 COM 注册或更宽的跨边界对象传递

最小 contract tests 运行示例：

```powershell
dotnet build .\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests.csproj -v minimal
.\adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests\bin\Debug\net10.0\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests.exe --native-lib-dir D:\Code\RadishFlow\target\debug
```
