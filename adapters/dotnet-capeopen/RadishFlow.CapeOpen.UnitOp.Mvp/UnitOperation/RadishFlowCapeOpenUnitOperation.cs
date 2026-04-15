using RadishFlow.CapeOpen.Adapter;
using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

public sealed class RadishFlowCapeOpenUnitOperation : ICapeIdentification, ICapeUtilities, ICapeUnit, IDisposable
{
    private const string UtilitiesInterfaceName = nameof(ICapeUtilities);
    private const string UnitInterfaceName = nameof(ICapeUnit);
    private const string UnitScope = "RadishFlow.CapeOpen.UnitOp.Mvp";
    private const string ConnectPortOperation = nameof(SetPortConnected);

    private readonly UnitOperationParameterPlaceholder _flowsheetParameter;
    private readonly UnitOperationParameterPlaceholder _packageIdParameter;
    private readonly UnitOperationParameterPlaceholder _manifestPathParameter;
    private readonly UnitOperationParameterPlaceholder _payloadPathParameter;
    private readonly UnitOperationPortPlaceholder _feedPort;
    private readonly UnitOperationPortPlaceholder _productPort;

    private object? _simulationContext;
    private string? _lastFlowsheetSnapshotJson;
    private bool _initialized;
    private bool _terminated;
    private bool _disposed;

    public RadishFlowCapeOpenUnitOperation()
    {
        ComponentName = "RadishFlow Unit Operation";
        ComponentDescription = "Minimal CAPE-OPEN unit operation skeleton.";

        _flowsheetParameter = new UnitOperationParameterPlaceholder(
            "Flowsheet Json",
            "StoredProjectFile JSON used by the MVP unit operation skeleton.",
            isRequired: true,
            onStateChanged: InvalidateValidation);
        _packageIdParameter = new UnitOperationParameterPlaceholder(
            "Property Package Id",
            "Identifier of the property package selected for the MVP unit operation skeleton.",
            isRequired: true,
            onStateChanged: InvalidateValidation);
        _manifestPathParameter = new UnitOperationParameterPlaceholder(
            "Property Package Manifest Path",
            "Optional manifest path for a local property package payload.",
            isRequired: false,
            onStateChanged: InvalidateValidation);
        _payloadPathParameter = new UnitOperationParameterPlaceholder(
            "Property Package Payload Path",
            "Optional payload path for a local property package payload.",
            isRequired: false,
            onStateChanged: InvalidateValidation);

        _feedPort = new UnitOperationPortPlaceholder(
            "Feed",
            "Required inlet material placeholder port.",
            direction: CapePortDirection.CAPE_INLET,
            portType: CapePortType.CAPE_MATERIAL,
            isRequired: true,
            onStateChanged: InvalidateValidation);
        _productPort = new UnitOperationPortPlaceholder(
            "Product",
            "Required outlet material placeholder port.",
            direction: CapePortDirection.CAPE_OUTLET,
            portType: CapePortType.CAPE_MATERIAL,
            isRequired: true,
            onStateChanged: InvalidateValidation);

        Parameters = new UnitOperationPlaceholderCollection<UnitOperationParameterPlaceholder>(
            "Parameters",
            "Public CAPE-OPEN parameter collection for the MVP unit operation.",
        [
            _flowsheetParameter,
            _packageIdParameter,
            _manifestPathParameter,
            _payloadPathParameter,
        ]);
        Ports = new UnitOperationPlaceholderCollection<UnitOperationPortPlaceholder>(
            "Ports",
            "Public CAPE-OPEN port collection for the MVP unit operation.",
        [
            _feedPort,
            _productPort,
        ]);

        ValStatus = CapeValidationStatus.NotValidated;
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }

    public UnitOperationPlaceholderCollection<UnitOperationParameterPlaceholder> Parameters { get; }

    object? ICapeUtilities.Parameters => Parameters;

    public UnitOperationPlaceholderCollection<UnitOperationPortPlaceholder> Ports { get; }

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

    public string? LastFlowsheetSnapshotJson => _lastFlowsheetSnapshotJson;

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

        _flowsheetParameter.SetValue(flowsheetJson);
    }

    public void LoadPropertyPackageFiles(string manifestPath, string payloadPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(manifestPath);
        ArgumentException.ThrowIfNullOrWhiteSpace(payloadPath);
        ThrowIfDisposed();
        ThrowIfTerminated(nameof(LoadPropertyPackageFiles), UtilitiesInterfaceName);

        _manifestPathParameter.SetValue(manifestPath);
        _payloadPathParameter.SetValue(payloadPath);
    }

    public void SelectPropertyPackage(string packageId)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(packageId);
        ThrowIfDisposed();
        ThrowIfTerminated(nameof(SelectPropertyPackage), UtilitiesInterfaceName);

        _packageIdParameter.SetValue(packageId);
    }

    public void SetPortConnected(string portName, bool isConnected)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(portName);
        ThrowIfDisposed();
        ThrowIfTerminated(ConnectPortOperation, UnitInterfaceName);

        var port = FindPort(portName) ?? throw new CapeInvalidArgumentException(
            $"Unknown placeholder port `{portName}`.",
            CreateContext(UnitInterfaceName, ConnectPortOperation, moreInfo: portName));
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
        if (_terminated)
        {
            throw CreateBadInvocation(
                UtilitiesInterfaceName,
                nameof(Initialize),
                "This unit instance has already been terminated and cannot be reinitialized.");
        }

        if (_initialized)
        {
            return;
        }

        _initialized = true;
        InvalidateValidation();
    }

    public void Terminate()
    {
        if (_disposed || _terminated)
        {
            return;
        }

        _initialized = false;
        _terminated = true;
        _simulationContext = null;
        ClearCalculationArtifacts();
        ValStatus = CapeValidationStatus.NotValidated;
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
        message = result.Message;
        ValStatus = result.IsValid ? CapeValidationStatus.Valid : CapeValidationStatus.Invalid;
        return result.IsValid;
    }

    public void Calculate()
    {
        ThrowIfDisposed();
        ThrowIfTerminated(nameof(Calculate), UnitInterfaceName);

        if (!_initialized)
        {
            throw CreateBadInvocation(
                UnitInterfaceName,
                nameof(Calculate),
                "Initialize must be called before Calculate.",
                nameof(Initialize));
        }

        var result = EvaluateValidation();
        if (!result.IsValid)
        {
            throw CreateExceptionForValidationFailure(nameof(Calculate), result);
        }

        ValStatus = CapeValidationStatus.Valid;
        ClearCalculationArtifacts();

        using var engine = new RadishFlowNativeEngine();
        engine.LoadFlowsheetJson(_flowsheetParameter.Value!);

        if (_manifestPathParameter.IsConfigured)
        {
            engine.LoadPropertyPackageFiles(
                _manifestPathParameter.Value!,
                _payloadPathParameter.Value!);
        }

        engine.SolveFlowsheet(_packageIdParameter.Value!);
        _lastFlowsheetSnapshotJson = engine.GetFlowsheetSnapshotJson();
    }

    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }

        Terminate();
        _disposed = true;
    }

    private ValidationResult EvaluateValidation()
    {
        if (_terminated)
        {
            return ValidationResult.Invalid(
                "Terminate has already been called for this unit instance.");
        }

        if (!_initialized)
        {
            return ValidationResult.Invalid(
                "Initialize must be called before Validate.",
                nameof(Initialize));
        }

        if (!_flowsheetParameter.IsConfigured)
        {
            return ValidationResult.Invalid(
                $"Required parameter `{_flowsheetParameter.ComponentName}` is not configured.",
                nameof(LoadFlowsheetJson));
        }

        if (!_packageIdParameter.IsConfigured)
        {
            return ValidationResult.Invalid(
                $"Required parameter `{_packageIdParameter.ComponentName}` is not configured.",
                nameof(SelectPropertyPackage));
        }

        if (_manifestPathParameter.IsConfigured != _payloadPathParameter.IsConfigured)
        {
            return ValidationResult.Invalid(
                $"Optional parameters `{_manifestPathParameter.ComponentName}` and `{_payloadPathParameter.ComponentName}` must be configured together.",
                nameof(LoadPropertyPackageFiles));
        }

        foreach (var parameter in Parameters)
        {
            var parameterMessage = string.Empty;
            if (!parameter.Validate(ref parameterMessage))
            {
                return ValidationResult.Invalid(parameterMessage);
            }
        }

        foreach (var port in Ports.Where(static port => port.IsRequired))
        {
            if (!port.IsConnected)
            {
                return ValidationResult.Invalid(
                    $"Required port `{port.ComponentName}` is not connected.",
                    ConnectPortOperation);
            }
        }

        return ValidationResult.Valid("The MVP CAPE-OPEN unit operation skeleton is configured.");
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

    private UnitOperationPortPlaceholder? FindPort(string portName)
    {
        return Ports.FirstOrDefault(port =>
            string.Equals(port.ComponentName, portName, StringComparison.OrdinalIgnoreCase));
    }

    private void InvalidateValidation()
    {
        ClearCalculationArtifacts();

        if (!_terminated)
        {
            ValStatus = CapeValidationStatus.NotValidated;
        }
    }

    private void ClearCalculationArtifacts()
    {
        _lastFlowsheetSnapshotJson = null;
    }

    private void ThrowIfDisposed()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
    }

    private void ThrowIfTerminated(string operation, string interfaceName)
    {
        if (_terminated)
        {
            throw CreateBadInvocation(
                interfaceName,
                operation,
                "Terminate has already been called for this unit instance.");
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
        string? requestedOperation = null)
    {
        return new CapeOpenExceptionContext(
            InterfaceName: interfaceName,
            Scope: UnitScope,
            Operation: operation,
            MoreInfo: moreInfo,
            RequestedOperation: requestedOperation);
    }

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
