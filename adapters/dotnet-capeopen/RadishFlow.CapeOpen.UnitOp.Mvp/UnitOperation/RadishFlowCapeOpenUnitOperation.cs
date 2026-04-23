using RadishFlow.CapeOpen.Adapter;
using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using System.Runtime.InteropServices;
using System.Text.Json;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

[ComVisible(true)]
[Guid(UnitOperationComIdentity.ClassId)]
[ProgId(UnitOperationComIdentity.ProgId)]
[ClassInterface(ClassInterfaceType.None)]
public sealed class RadishFlowCapeOpenUnitOperation : ICapeIdentification, ICapeUtilities, ICapeUnit, IDisposable
{
    private const string UtilitiesInterfaceName = nameof(ICapeUtilities);
    private const string UnitInterfaceName = nameof(ICapeUnit);
    private const string UnitScope = "RadishFlow.CapeOpen.UnitOp.Mvp";
    private object? _simulationContext;
    private UnitOperationCalculationResult? _lastCalculationResult;
    private UnitOperationCalculationFailure? _lastCalculationFailure;
    private bool _materialResultsStale;
    private UnitOperationLifecycleState _lifecycleState;

    public RadishFlowCapeOpenUnitOperation()
    {
        ComponentName = UnitOperationComIdentity.DisplayName;
        ComponentDescription = UnitOperationComIdentity.Description;

        Parameters = new UnitOperationParameterCollection(
            UnitOperationParameterCatalog.CollectionDefinition,
            UnitOperationParameterCatalog.OrderedDefinitions.Select(
                definition => new UnitOperationParameterPlaceholder(
                    definition,
                    ensureOwnerAccess: EnsurePlaceholderAccess,
                    onStateChanged: InvalidateValidation)),
            ensureOwnerAccess: EnsurePlaceholderAccess);
        Ports = new UnitOperationPortCollection(
            UnitOperationPortCatalog.CollectionDefinition,
            UnitOperationPortCatalog.OrderedDefinitions.Select(
                definition => new UnitOperationPortPlaceholder(
                    definition,
                    ensureOwnerAccess: EnsurePlaceholderAccess,
                    onStateChanged: InvalidateValidation)),
            ensureOwnerAccess: EnsurePlaceholderAccess);

        ValStatus = CapeValidationStatus.NotValidated;
        _lifecycleState = UnitOperationLifecycleState.Constructed;
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }

    public UnitOperationParameterCollection Parameters { get; }

    object? ICapeUtilities.Parameters => Parameters;

    public UnitOperationPortCollection Ports { get; }

    object? ICapeUnit.Ports => Ports;

    public object? SimulationContext
    {
        get => _simulationContext;
        set
        {
            ThrowIfDisposed();
            ThrowIfTerminated(nameof(SimulationContext), UtilitiesInterfaceName);
            _simulationContext = value;
            InvalidateValidation();
        }
    }

    public CapeValidationStatus ValStatus { get; private set; }

    public UnitOperationCalculationResult? LastCalculationResult => _lastCalculationResult;

    public UnitOperationCalculationFailure? LastCalculationFailure => _lastCalculationFailure;

    public UnitOperationCalculationReport GetCalculationReport()
    {
        ThrowIfDisposed();

        if (_lastCalculationResult is not null)
        {
            return UnitOperationCalculationReport.FromSuccess(_lastCalculationResult);
        }

        if (_lastCalculationFailure is not null)
        {
            return UnitOperationCalculationReport.FromFailure(_lastCalculationFailure);
        }

        return UnitOperationCalculationReport.Empty();
    }

    public IReadOnlyList<string> GetCalculationReportLines()
    {
        ThrowIfDisposed();
        return GetCalculationReport().GetDisplayLines();
    }

    public UnitOperationCalculationReportState GetCalculationReportState()
    {
        ThrowIfDisposed();
        return GetCalculationReport().GetDisplayState();
    }

    public string GetCalculationReportHeadline()
    {
        ThrowIfDisposed();
        return GetCalculationReport().GetDisplayHeadline();
    }

    public int GetCalculationReportDetailKeyCount()
    {
        ThrowIfDisposed();
        return GetCalculationReport().GetDetailKeyCount();
    }

    public string GetCalculationReportDetailKey(int detailKeyIndex)
    {
        ThrowIfDisposed();
        return GetCalculationReport().GetDetailKey(detailKeyIndex);
    }

    public string? GetCalculationReportDetailValue(string detailKey)
    {
        ThrowIfDisposed();
        return GetCalculationReport().GetDetailValue(detailKey);
    }

    public int GetCalculationReportLineCount()
    {
        ThrowIfDisposed();
        return GetCalculationReport().GetDisplayLineCount();
    }

    public string GetCalculationReportLine(int lineIndex)
    {
        ThrowIfDisposed();
        return GetCalculationReport().GetDisplayLine(lineIndex);
    }

    public string GetCalculationReportText()
    {
        ThrowIfDisposed();
        return GetCalculationReport().GetDisplayText();
    }

    public void ConfigureNativeLibraryDirectory(string directoryPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(directoryPath);
        ThrowIfDisposed();

        RfNativeLibraryLoader.ConfigureSearchDirectory(directoryPath);
    }

    public void LoadFlowsheetJson(string flowsheetJson)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(flowsheetJson);
        ThrowIfDisposed();
        ThrowIfTerminated(nameof(LoadFlowsheetJson), UtilitiesInterfaceName);

        FlowsheetParameter.SetValue(flowsheetJson);
    }

    public void LoadPropertyPackageFiles(string manifestPath, string payloadPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(manifestPath);
        ArgumentException.ThrowIfNullOrWhiteSpace(payloadPath);
        ThrowIfDisposed();
        ThrowIfTerminated(nameof(LoadPropertyPackageFiles), UtilitiesInterfaceName);

        ManifestPathParameter.SetValue(manifestPath);
        PayloadPathParameter.SetValue(payloadPath);
    }

    public void SelectPropertyPackage(string packageId)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(packageId);
        ThrowIfDisposed();
        ThrowIfTerminated(nameof(SelectPropertyPackage), UtilitiesInterfaceName);

        PackageIdParameter.SetValue(packageId);
    }

    public void SetPortConnected(string portName, bool isConnected)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(portName);
        ThrowIfDisposed();
        ThrowIfTerminated(nameof(SetPortConnected), UnitInterfaceName);

        if (!UnitOperationPortCatalog.TryGetByName(portName, out var portDefinition))
        {
            throw new CapeInvalidArgumentException(
            $"Unknown placeholder port `{portName}`.",
            CreateContext(UnitInterfaceName, nameof(SetPortConnected), moreInfo: portName));
        }

        var port = GetPortPlaceholder(portDefinition);
        if (isConnected)
        {
            port.ConnectPlaceholder();
            return;
        }

        port.Disconnect();
    }

    public void Initialize()
    {
        ThrowIfDisposed();
        if (IsTerminated)
        {
            throw CreateBadInvocation(
                UtilitiesInterfaceName,
                nameof(Initialize),
                "This unit instance has already been terminated and cannot be reinitialized.");
        }

        if (IsInitialized)
        {
            return;
        }

        _lifecycleState = UnitOperationLifecycleState.Initialized;
        InvalidateValidation();
    }

    public void Terminate()
    {
        if (IsDisposed || IsTerminated)
        {
            return;
        }

        _simulationContext = null;
        foreach (var port in Ports)
        {
            port.ReleaseConnectedObject();
        }

        ResetCalculationState(CapeValidationStatus.NotValidated);
        _materialResultsStale = false;
        _lifecycleState = UnitOperationLifecycleState.Terminated;
    }

    public int Edit()
    {
        ThrowIfDisposed();
        ThrowIfTerminated(nameof(Edit), UtilitiesInterfaceName);
        throw new CapeNoImplementationException(
            "Edit UI is not implemented for the MVP CAPE-OPEN unit operation skeleton.",
            CreateContext(UtilitiesInterfaceName, nameof(Edit)));
    }

    public bool Validate(ref string message)
    {
        ThrowIfDisposed();

        var result = EvaluateValidation();
        return ApplyValidationOutcome(result, ref message);
    }

    public void Calculate()
    {
        ThrowIfDisposed();
        ThrowIfTerminated(nameof(Calculate), UnitInterfaceName);

        if (!IsInitialized)
        {
            throw CreateBadInvocation(
                UnitInterfaceName,
                nameof(Calculate),
                "Initialize must be called before Calculate.",
                nameof(Initialize));
        }

        try
        {
            PrepareForCalculation();
            var inputs = BuildCalculationInputs();
            var snapshotJson = ExecuteNativeSolve(inputs);
            RecordCalculationSuccess(MaterializeCalculationResult(snapshotJson));
        }
        catch (CapeOpenException error)
        {
            RecordCalculationFailure(error);
            throw;
        }
    }

    public void Dispose()
    {
        if (IsDisposed)
        {
            return;
        }

        Terminate();
        _lifecycleState = UnitOperationLifecycleState.Disposed;
    }

    private ValidationResult EvaluateValidation()
    {
        return
            EvaluateLifecycleValidation() ??
            EvaluateRequiredParameterConfigurationValidation() ??
            EvaluateParameterCompanionValidation() ??
            EvaluateParameterValueValidation() ??
            EvaluateRequiredPortValidation() ??
            ValidationResult.Valid("The MVP CAPE-OPEN unit operation skeleton is configured.");
    }

    private void PrepareForCalculation()
    {
        ResetCalculationState(CapeValidationStatus.NotValidated);

        var validation = EvaluateValidation();
        if (!validation.IsValid)
        {
            throw CreateExceptionForValidationFailure(nameof(Calculate), validation);
        }
    }

    private CapeOpenException CreateExceptionForValidationFailure(string operation, ValidationResult result)
    {
        if (result.RequestedOperation is not null)
        {
            return CreateBadInvocation(
                UnitInterfaceName,
                operation,
                result.Message,
                result.RequestedOperation);
        }

        return new CapeFailedInitialisationException(
            result.Message,
            CreateContext(UnitInterfaceName, operation, moreInfo: result.Message));
    }

    private ValidationResult? EvaluateParameterCompanionValidation()
    {
        var evaluatedPairs = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        foreach (var definition in UnitOperationParameterCatalog.OrderedDefinitions)
        {
            if (definition.RequiredCompanionParameterName is not { Length: > 0 } companionName)
            {
                continue;
            }

            var parameter = GetParameterPlaceholder(definition);
            var companionDefinition = UnitOperationParameterCatalog.GetByName(companionName);
            var companion = GetParameterPlaceholder(companionDefinition);

            var pairKey = string.Compare(
                parameter.ComponentName,
                companion.ComponentName,
                StringComparison.OrdinalIgnoreCase) <= 0
                ? $"{parameter.ComponentName}|{companion.ComponentName}"
                : $"{companion.ComponentName}|{parameter.ComponentName}";
            if (!evaluatedPairs.Add(pairKey))
            {
                continue;
            }

            if (parameter.IsConfigured != companion.IsConfigured)
            {
                return ValidationResult.Invalid(
                    $"Optional parameters `{parameter.ComponentName}` and `{companion.ComponentName}` must be configured together.",
                    definition.ConfigurationOperationName);
            }
        }

        return null;
    }

    private ValidationResult? EvaluateRequiredParameterConfigurationValidation()
    {
        foreach (var definition in UnitOperationParameterCatalog.OrderedDefinitions.Where(static definition => definition.IsRequired))
        {
            var parameter = GetParameterPlaceholder(definition);
            if (!parameter.IsConfigured)
            {
                return ValidationResult.Invalid(
                    $"Required parameter `{parameter.ComponentName}` is not configured.",
                    definition.ConfigurationOperationName);
            }
        }

        return null;
    }

    private ValidationResult? EvaluateParameterValueValidation()
    {
        foreach (var parameter in Parameters)
        {
            var parameterMessage = string.Empty;
            if (!parameter.Validate(ref parameterMessage))
            {
                return ValidationResult.Invalid(parameterMessage);
            }
        }

        return null;
    }

    private ValidationResult? EvaluateRequiredPortValidation()
    {
        foreach (var definition in UnitOperationPortCatalog.OrderedDefinitions.Where(static definition => definition.IsRequired))
        {
            var port = GetPortPlaceholder(definition);
            if (!port.IsConnected)
            {
                return ValidationResult.Invalid(
                    $"Required port `{port.ComponentName}` is not connected.",
                    definition.ConnectionOperationName);
            }
        }

        return null;
    }

    private CalculationInputs BuildCalculationInputs()
    {
        return new CalculationInputs(
            GetRequiredParameterValue(UnitOperationParameterCatalog.FlowsheetJson),
            GetRequiredParameterValue(UnitOperationParameterCatalog.PropertyPackageId),
            GetOptionalParameterValue(UnitOperationParameterCatalog.PropertyPackageManifestPath),
            GetOptionalParameterValue(UnitOperationParameterCatalog.PropertyPackagePayloadPath));
    }

    private static string ExecuteNativeSolve(CalculationInputs inputs)
    {
        using var engine = new RadishFlowNativeEngine();
        engine.LoadFlowsheetJson(inputs.FlowsheetJson);

        if (inputs.ManifestPath is not null && inputs.PayloadPath is not null)
        {
            engine.LoadPropertyPackageFiles(inputs.ManifestPath, inputs.PayloadPath);
        }

        engine.SolveFlowsheet(inputs.PackageId);
        return engine.GetFlowsheetSnapshotJson();
    }

    private UnitOperationCalculationResult ParseCalculationResult(string snapshotJson)
    {
        try
        {
            return UnitOperationCalculationResult.Parse(snapshotJson);
        }
        catch (JsonException error)
        {
            throw CreateCalculationResultContractException(error);
        }
        catch (InvalidDataException error)
        {
            throw CreateCalculationResultContractException(error);
        }
    }

    private UnitOperationCalculationResult MaterializeCalculationResult(string snapshotJson)
    {
        return ParseCalculationResult(snapshotJson);
    }

    private bool ApplyValidationOutcome(ValidationResult result, ref string message)
    {
        message = result.Message;
        ValStatus = result.IsValid ? CapeValidationStatus.Valid : CapeValidationStatus.Invalid;
        return result.IsValid;
    }

    private void InvalidateValidation()
    {
        if (!IsTerminated)
        {
            _materialResultsStale = _materialResultsStale || _lastCalculationResult is not null;
            ResetCalculationState(CapeValidationStatus.NotValidated);
        }
    }

    private void ResetCalculationState(CapeValidationStatus validationStatus)
    {
        _lastCalculationResult = null;
        _lastCalculationFailure = null;
        ValStatus = validationStatus;
    }

    private void ThrowIfDisposed()
    {
        ObjectDisposedException.ThrowIf(IsDisposed, this);
    }

    private void ThrowIfTerminated(string operation, string interfaceName)
    {
        if (IsTerminated)
        {
            throw CreateBadInvocation(
                interfaceName,
                operation,
                "Terminate has already been called for this unit instance.");
        }
    }

    private void EnsurePlaceholderAccess(
        string interfaceName,
        string operation,
        string? parameterName,
        object? parameter)
    {
        if (IsDisposed)
        {
            throw new CapeBadInvocationOrderException(
                "This unit instance has already been disposed.",
                CreateContext(
                    interfaceName,
                    operation,
                    parameterName: parameterName,
                    parameter: parameter));
        }

        if (IsTerminated)
        {
            throw new CapeBadInvocationOrderException(
                "Terminate has already been called for this unit instance.",
                CreateContext(
                    interfaceName,
                    operation,
                    parameterName: parameterName,
                    parameter: parameter));
        }
    }

    private static CapeBadInvocationOrderException CreateBadInvocation(
        string interfaceName,
        string operation,
        string description,
        string? requestedOperation = null)
    {
        return new CapeBadInvocationOrderException(
            description,
            CreateContext(interfaceName, operation, requestedOperation: requestedOperation));
    }

    private static CapeOpenExceptionContext CreateContext(
        string interfaceName,
        string operation,
        string? moreInfo = null,
        string? requestedOperation = null,
        string? parameterName = null,
        object? parameter = null)
    {
        return new CapeOpenExceptionContext(
            InterfaceName: interfaceName,
            Scope: UnitScope,
            Operation: operation,
            MoreInfo: moreInfo,
            RequestedOperation: requestedOperation,
            ParameterName: parameterName,
            Parameter: parameter);
    }

    private static CapeUnknownException CreateCalculationResultContractException(Exception error)
    {
        return new CapeUnknownException(
            $"Native solve snapshot could not be materialized into the MVP unit operation calculation result contract: {error.Message}",
            CreateContext(
                UnitInterfaceName,
                nameof(Calculate),
                moreInfo: "Failed to parse status/summary/diagnostics from native solve snapshot JSON."));
    }

    private void RecordCalculationFailure(CapeOpenException error)
    {
        _lastCalculationResult = null;
        _lastCalculationFailure = UnitOperationCalculationFailure.FromException(error);
        ValStatus = CapeValidationStatus.Invalid;
    }

    private void RecordCalculationSuccess(UnitOperationCalculationResult result)
    {
        _lastCalculationResult = result;
        _lastCalculationFailure = null;
        _materialResultsStale = false;
        ValStatus = CapeValidationStatus.Valid;
    }

    private ValidationResult? EvaluateLifecycleValidation()
    {
        return _lifecycleState switch
        {
            UnitOperationLifecycleState.Terminated => ValidationResult.Invalid(
                "Terminate has already been called for this unit instance."),
            UnitOperationLifecycleState.Constructed => ValidationResult.Invalid(
                "Initialize must be called before Validate.",
                nameof(Initialize)),
            UnitOperationLifecycleState.Initialized => null,
            UnitOperationLifecycleState.Disposed => throw new ObjectDisposedException(GetType().FullName),
            _ => throw new ArgumentOutOfRangeException(nameof(_lifecycleState), _lifecycleState, "Unsupported unit operation lifecycle state."),
        };
    }

    private bool IsInitialized => _lifecycleState == UnitOperationLifecycleState.Initialized;

    private bool IsTerminated => _lifecycleState == UnitOperationLifecycleState.Terminated;

    private bool IsDisposed => _lifecycleState == UnitOperationLifecycleState.Disposed;

    internal UnitOperationLifecycleState HostLifecycleState => _lifecycleState;

    internal bool HostMaterialResultsStale => _materialResultsStale;

    internal bool HostExecutionResultsStale => _materialResultsStale;

    private UnitOperationParameterPlaceholder FlowsheetParameter => GetParameterPlaceholder(UnitOperationParameterCatalog.FlowsheetJson);

    private UnitOperationParameterPlaceholder PackageIdParameter => GetParameterPlaceholder(UnitOperationParameterCatalog.PropertyPackageId);

    private UnitOperationParameterPlaceholder ManifestPathParameter => GetParameterPlaceholder(UnitOperationParameterCatalog.PropertyPackageManifestPath);

    private UnitOperationParameterPlaceholder PayloadPathParameter => GetParameterPlaceholder(UnitOperationParameterCatalog.PropertyPackagePayloadPath);

    private UnitOperationParameterPlaceholder GetParameterPlaceholder(UnitOperationParameterDefinition definition)
    {
        return Parameters.GetByName(definition.Name);
    }

    private UnitOperationPortPlaceholder GetPortPlaceholder(UnitOperationPortDefinition definition)
    {
        return Ports.GetByName(definition.Name);
    }

    private string GetRequiredParameterValue(UnitOperationParameterDefinition definition)
    {
        return GetParameterPlaceholder(definition).Value!;
    }

    private string? GetOptionalParameterValue(UnitOperationParameterDefinition definition)
    {
        var parameter = GetParameterPlaceholder(definition);
        return parameter.IsConfigured ? parameter.Value : null;
    }

    private sealed record CalculationInputs(
        string FlowsheetJson,
        string PackageId,
        string? ManifestPath,
        string? PayloadPath);

    private sealed record ValidationResult(bool IsValid, string Message, string? RequestedOperation)
    {
        public static ValidationResult Valid(string message)
        {
            return new ValidationResult(true, message, null);
        }

        public static ValidationResult Invalid(string message, string? requestedOperation = null)
        {
            return new ValidationResult(false, message, requestedOperation);
        }
    }
}
