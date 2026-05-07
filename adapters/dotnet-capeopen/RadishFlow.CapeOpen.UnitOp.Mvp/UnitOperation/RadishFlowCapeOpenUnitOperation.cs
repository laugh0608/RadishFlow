using RadishFlow.CapeOpen.Adapter;
using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Ole;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Persistence;
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
[ComDefaultInterface(typeof(ICapeUtilities))]
public sealed partial class RadishFlowCapeOpenUnitOperation : ICapeIdentification, ICapeUtilities, ICapeUnit, ICapeUnitReport, ECapeRoot, ECapeUser, IPersistStreamInit, IPersistStorage, IOleObject, IDisposable
{
    private const string UtilitiesInterfaceName = nameof(ICapeUtilities);
    private const string UnitInterfaceName = nameof(ICapeUnit);
    private const string UnitReportInterfaceName = nameof(ICapeUnitReport);
    private const string UnitScope = "RadishFlow.CapeOpen.UnitOp.Mvp";
    private const string DefaultReportName = "RadishFlow calculation report";
    private const string SimulationContextMemberName = "SimulationContext";
    private const string NoRecordedCapeOpenError = "No CAPE-OPEN error has been recorded for this unit instance.";
    private readonly UnitOperationSimulationContextPlaceholder _simulationContextFallback = new();
    private bool _simulationContextProvided;
    private UnitOperationCalculationResult? _lastCalculationResult;
    private UnitOperationCalculationFailure? _lastCalculationFailure;
    private CapeOpenException? _lastCapeOpenError;
    private string _componentName;
    private string _componentDescription;
    private string _selectedReportName = DefaultReportName;
    private bool _materialResultsStale;
    private UnitOperationLifecycleState _lifecycleState;
    private IntPtr _oleClientSite;
    private OleSize _oleExtent = new(2540, 2540);

    static RadishFlowCapeOpenUnitOperation()
    {
        UnitOperationComTrace.Write(nameof(RadishFlowCapeOpenUnitOperation), "static-init");
    }

    public RadishFlowCapeOpenUnitOperation()
    {
        UnitOperationComTrace.Write(nameof(RadishFlowCapeOpenUnitOperation), "constructor-enter");

        _componentName = UnitOperationComIdentity.DisplayName;
        _componentDescription = UnitOperationComIdentity.Description;

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
        UnitOperationComTrace.Write(nameof(RadishFlowCapeOpenUnitOperation), "constructor-exit");
    }

    public string Name => nameof(RadishFlowCapeOpenUnitOperation);

    public int Code => _lastCapeOpenError?.Code ?? 0;

    public string Description => _lastCapeOpenError?.Description ?? NoRecordedCapeOpenError;

    public string Scope => _lastCapeOpenError?.Scope ?? UnitScope;

    public string InterfaceName => _lastCapeOpenError?.InterfaceName ?? UtilitiesInterfaceName;

    public string Operation => _lastCapeOpenError?.Operation ?? string.Empty;

    public string? MoreInfo => _lastCapeOpenError?.MoreInfo;

    public string ComponentName
    {
        get
        {
            UnitOperationComTrace.Write(nameof(ComponentName), "get-enter");
            try
            {
                return _componentName;
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception(nameof(ComponentName), error);
                throw;
            }
            finally
            {
                UnitOperationComTrace.Write(nameof(ComponentName), "get-exit");
            }
        }

        set
        {
            UnitOperationComTrace.Write(nameof(ComponentName), "set-enter", value);
            try
            {
                _componentName = value;
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception(nameof(ComponentName), error);
                throw;
            }
            finally
            {
                UnitOperationComTrace.Write(nameof(ComponentName), "set-exit");
            }
        }
    }

    public string ComponentDescription
    {
        get
        {
            UnitOperationComTrace.Write(nameof(ComponentDescription), "get-enter");
            try
            {
                return _componentDescription;
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception(nameof(ComponentDescription), error);
                throw;
            }
            finally
            {
                UnitOperationComTrace.Write(nameof(ComponentDescription), "get-exit");
            }
        }

        set
        {
            UnitOperationComTrace.Write(nameof(ComponentDescription), "set-enter", value);
            try
            {
                _componentDescription = value;
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception(nameof(ComponentDescription), error);
                throw;
            }
            finally
            {
                UnitOperationComTrace.Write(nameof(ComponentDescription), "set-exit");
            }
        }
    }

    public UnitOperationParameterCollection Parameters { get; }

    object? ICapeUtilities.Parameters
    {
        get
        {
            UnitOperationComTrace.Write(nameof(ICapeUtilities.Parameters), "get-enter");
            try
            {
                return Parameters;
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception(nameof(ICapeUtilities.Parameters), error);
                throw;
            }
            finally
            {
                UnitOperationComTrace.Write(nameof(ICapeUtilities.Parameters), "get-exit");
            }
        }
    }

    public UnitOperationPortCollection Ports { get; }

    object? ICapeUnit.Ports
    {
        get
        {
            UnitOperationComTrace.Write(nameof(ICapeUnit.Ports), "get-enter");
            try
            {
                return Ports;
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception(nameof(ICapeUnit.Ports), error);
                throw;
            }
            finally
            {
                UnitOperationComTrace.Write(nameof(ICapeUnit.Ports), "get-exit");
            }
        }
    }

    IntPtr ICapeUtilities.get_SimulationContext()
    {
        UnitOperationComTrace.Write(SimulationContextMemberName, "get-enter");
        try
        {
            ThrowIfDisposed();
#pragma warning disable CA1416 // UnitOp.Mvp COM activation is Windows-only.
            var context = Marshal.GetIDispatchForObject(_simulationContextFallback);
#pragma warning restore CA1416
            UnitOperationComTrace.Write(
                SimulationContextMemberName,
                "get-result",
                _simulationContextProvided ? "fallback=provided; hostContext=provided" : "fallback=provided; hostContext=missing");
            return context;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(SimulationContextMemberName, error);
            throw;
        }
        finally
        {
            UnitOperationComTrace.Write(SimulationContextMemberName, "get-exit");
        }
    }

    void ICapeUtilities.set_SimulationContext(IntPtr value)
    {
        UnitOperationComTrace.Write(
            SimulationContextMemberName,
            "set-enter",
            value == IntPtr.Zero ? "context=null" : "context=provided");
        try
        {
            ThrowIfDisposed();
            ThrowIfTerminated(SimulationContextMemberName, UtilitiesInterfaceName);
            _simulationContextProvided = value != IntPtr.Zero;
            InvalidateValidation();
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(SimulationContextMemberName, error);
            throw;
        }
        finally
        {
            UnitOperationComTrace.Write(SimulationContextMemberName, "set-exit");
        }
    }

    public CapeValidationStatus ValStatus { get; private set; }

    public UnitOperationCalculationResult? LastCalculationResult => _lastCalculationResult;

    public UnitOperationCalculationFailure? LastCalculationFailure => _lastCalculationFailure;

    public object reports
    {
        get
        {
            UnitOperationComTrace.Write(nameof(reports), "get-enter");
            try
            {
                ThrowIfDisposed();
                return new[] { DefaultReportName };
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception(nameof(reports), error);
                throw;
            }
            finally
            {
                UnitOperationComTrace.Write(nameof(reports), "get-exit");
            }
        }
    }

    public string selectedReport
    {
        get
        {
            UnitOperationComTrace.Write(nameof(selectedReport), "get-enter");
            try
            {
                ThrowIfDisposed();
                return _selectedReportName;
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception(nameof(selectedReport), error);
                throw;
            }
            finally
            {
                UnitOperationComTrace.Write(nameof(selectedReport), "get-exit");
            }
        }

        set
        {
            UnitOperationComTrace.Write(nameof(selectedReport), "set-enter", value);
            try
            {
                ThrowIfDisposed();
                ThrowIfTerminated(nameof(selectedReport), UnitReportInterfaceName);

                if (!string.Equals(value, DefaultReportName, StringComparison.Ordinal))
                {
                    throw new CapeInvalidArgumentException(
                        $"Unsupported unit report `{value}`.",
                        CreateContext(
                            UnitReportInterfaceName,
                            nameof(selectedReport),
                            moreInfo: $"Supported report: {DefaultReportName}",
                            parameterName: nameof(selectedReport),
                            parameter: value));
                }

                _selectedReportName = value;
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception(nameof(selectedReport), error);
                throw;
            }
            finally
            {
                UnitOperationComTrace.Write(nameof(selectedReport), "set-exit");
            }
        }
    }

    public void ProduceReport(ref string reportContent)
    {
        UnitOperationComTrace.Write(nameof(ProduceReport), "enter");
        try
        {
            ThrowIfDisposed();
            reportContent = GetCalculationReportText();
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(nameof(ProduceReport), error);
            throw;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(ProduceReport), "exit");
        }
    }

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
        UnitOperationComTrace.Write(nameof(Initialize), "enter");
        try
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
                UnitOperationComTrace.Write(nameof(Initialize), "already-initialized");
                return;
            }

            _lifecycleState = UnitOperationLifecycleState.Initialized;
            InvalidateValidation();
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(nameof(Initialize), error);
            throw;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(Initialize), "exit");
        }
    }

    public void Terminate()
    {
        UnitOperationComTrace.Write(nameof(Terminate), "enter");
        try
        {
            if (IsDisposed || IsTerminated)
            {
                UnitOperationComTrace.Write(nameof(Terminate), "already-terminal");
                return;
            }

            _simulationContextProvided = false;
            foreach (var port in Ports)
            {
                port.ReleaseConnectedObject();
            }

            ResetCalculationState(CapeValidationStatus.NotValidated);
            _materialResultsStale = false;
            _lifecycleState = UnitOperationLifecycleState.Terminated;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(nameof(Terminate), error);
            throw;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(Terminate), "exit");
        }
    }

    public int Edit()
    {
        UnitOperationComTrace.Write(nameof(Edit), "enter");
        ThrowIfDisposed();
        ThrowIfTerminated(nameof(Edit), UtilitiesInterfaceName);
        UnitOperationComTrace.Write(nameof(Edit), "no-op", "MVP unit operation has no custom editor.");
        UnitOperationComTrace.Write(nameof(Edit), "exit");
        return 0;
    }

    public bool Validate(ref string message)
    {
        UnitOperationComTrace.Write(nameof(Validate), "enter");
        try
        {
            ThrowIfDisposed();

            var result = EvaluateValidation();
            var isValid = ApplyValidationOutcome(result, ref message);
            UnitOperationComTrace.Write(nameof(Validate), "result", $"isValid={isValid}; message={message}");
            return isValid;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(nameof(Validate), error);
            throw;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(Validate), "exit");
        }
    }

    public void Calculate()
    {
        UnitOperationComTrace.Write(nameof(Calculate), "enter");
        try
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

            PrepareForCalculation();
            var inputs = BuildCalculationInputs();
            var effectiveInputs = ApplyConnectedFeedMaterial(inputs);
            var snapshotJson = ExecuteNativeSolve(effectiveInputs);
            var result = MaterializeCalculationResult(snapshotJson);
            PublishProductMaterial(result);
            RecordCalculationSuccess(result);
            UnitOperationComTrace.Write(nameof(Calculate), "success");
        }
        catch (CapeOpenException error)
        {
            UnitOperationComTrace.Exception(nameof(Calculate), error);
            RememberCapeOpenError(error);
            if (!IsLifecycleCalculationPreconditionFailure(error))
            {
                RecordCalculationFailure(error);
            }

            throw;
        }
        catch (Exception error) when (IsNativeLibraryLoadException(error))
        {
            UnitOperationComTrace.Exception(nameof(Calculate), error);
            var capeOpenError = CreateNativeLibraryLoadException(error);
            RememberCapeOpenError(capeOpenError);
            RecordCalculationFailure(capeOpenError);
            throw capeOpenError;
        }
        finally
        {
            UnitOperationComTrace.Write(nameof(Calculate), "exit");
        }
    }

    public void Dispose()
    {
        if (IsDisposed)
        {
            return;
        }

        Terminate();
        ReleaseOleClientSite();
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

    private CalculationInputs ApplyConnectedFeedMaterial(CalculationInputs inputs)
    {
        var bindings = UnitOperationConfiguredBoundaryMaterialBindings.TryParse(FlowsheetParameter);
        var feedMaterial = CapeOpenFeedMaterialReader.TryRead(
            GetPortPlaceholder(UnitOperationPortCatalog.Feed).ConnectedObjectReference);
        if (feedMaterial is null)
        {
            return inputs;
        }

        var effectiveFlowsheetJson = FlowsheetBoundaryFeedMaterialOverlay.ApplyOrOriginal(
            inputs.FlowsheetJson,
            bindings,
            feedMaterial);
        return string.Equals(effectiveFlowsheetJson, inputs.FlowsheetJson, StringComparison.Ordinal)
            ? inputs
            : inputs with { FlowsheetJson = effectiveFlowsheetJson };
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

    private void PublishProductMaterial(UnitOperationCalculationResult result)
    {
        var bindings = UnitOperationConfiguredBoundaryMaterialBindings.TryParse(FlowsheetParameter);
        var outputStreamIds = bindings.GetBoundStreamIds(UnitOperationPortBoundaryMaterialRole.BoundaryOutputs);
        if (outputStreamIds.Count == 0)
        {
            UnitOperationComTrace.Write(nameof(PublishProductMaterial), "skip", "No configured boundary output streams.");
            return;
        }

        var streamsById = result.Streams.ToDictionary(static stream => stream.Id, StringComparer.Ordinal);
        var outputStreams = outputStreamIds
            .Where(streamsById.ContainsKey)
            .Select(streamId => streamsById[streamId])
            .ToArray();
        CapeOpenMaterialObjectPublisher.PublishProductMaterial(
            GetPortPlaceholder(UnitOperationPortCatalog.Product).ConnectedObjectReference,
            outputStreams);
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
        if (validationStatus == CapeValidationStatus.NotValidated)
        {
            _lastCapeOpenError = null;
        }

        ValStatus = validationStatus;
    }

    private void RememberCapeOpenError(CapeOpenException error)
    {
        _lastCapeOpenError = error;
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

    private static bool IsNativeLibraryLoadException(Exception error)
    {
        return error is DllNotFoundException or BadImageFormatException or EntryPointNotFoundException;
    }

    private bool IsLifecycleCalculationPreconditionFailure(CapeOpenException error)
    {
        return error is CapeBadInvocationOrderException &&
               string.Equals(error.InterfaceName, UnitInterfaceName, StringComparison.Ordinal) &&
               string.Equals(error.Operation, nameof(Calculate), StringComparison.Ordinal) &&
               (IsTerminated || !IsInitialized);
    }

    private static CapeFailedInitialisationException CreateNativeLibraryLoadException(Exception error)
    {
        return new CapeFailedInitialisationException(
            $"Native rf-ffi runtime could not be loaded: {error.Message}",
            CreateContext(
                "rf-ffi",
                "load_native_library",
                moreInfo: "Make sure rf_ffi.dll is available in RADISHFLOW_NATIVE_LIB_DIR, the PME process directory, or the repository target/debug directory."));
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

    private void ReplaceOleClientSite(IntPtr clientSite)
    {
        if (clientSite != IntPtr.Zero)
        {
            Marshal.AddRef(clientSite);
        }

        ReleaseOleClientSite();
        _oleClientSite = clientSite;
    }

    private void ReleaseOleClientSite()
    {
        if (_oleClientSite == IntPtr.Zero)
        {
            return;
        }

        Marshal.Release(_oleClientSite);
        _oleClientSite = IntPtr.Zero;
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
