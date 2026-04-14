using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Unit;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

public sealed class RadishFlowCapeOpenUnitOperation : ICapeIdentification, ICapeUtilities, ICapeUnit, IDisposable
{
    private const string UtilitiesInterfaceName = nameof(ICapeUtilities);
    private const string UnitInterfaceName = nameof(ICapeUnit);
    private const string UnitScope = "RadishFlow.CapeOpen.UnitOp.Mvp";

    private object? _simulationContext;
    private string? _flowsheetJson;
    private string? _selectedPackageId;
    private string? _manifestPath;
    private string? _payloadPath;
    private bool _initialized;
    private bool _terminated;
    private bool _disposed;

    public RadishFlowCapeOpenUnitOperation()
    {
        ComponentName = "RadishFlow Unit Operation";
        ComponentDescription = "Minimal CAPE-OPEN unit operation skeleton.";
        ValStatus = CapeValidationStatus.NotValidated;
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }

    public object? Parameters => null;

    public object? Ports => null;

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

    public void LoadFlowsheetJson(string flowsheetJson)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(flowsheetJson);
        ThrowIfDisposed();
        ThrowIfTerminated(nameof(LoadFlowsheetJson), UtilitiesInterfaceName);

        _flowsheetJson = flowsheetJson;
        InvalidateValidation();
    }

    public void LoadPropertyPackageFiles(string manifestPath, string payloadPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(manifestPath);
        ArgumentException.ThrowIfNullOrWhiteSpace(payloadPath);
        ThrowIfDisposed();
        ThrowIfTerminated(nameof(LoadPropertyPackageFiles), UtilitiesInterfaceName);

        _manifestPath = manifestPath;
        _payloadPath = payloadPath;
        InvalidateValidation();
    }

    public void SelectPropertyPackage(string packageId)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(packageId);
        ThrowIfDisposed();
        ThrowIfTerminated(nameof(SelectPropertyPackage), UtilitiesInterfaceName);

        _selectedPackageId = packageId;
        InvalidateValidation();
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

        if (_terminated)
        {
            message = "Terminate has already been called for this unit instance.";
            ValStatus = CapeValidationStatus.Invalid;
            return false;
        }

        if (!_initialized)
        {
            message = "Initialize must be called before Validate.";
            ValStatus = CapeValidationStatus.Invalid;
            return false;
        }

        if (string.IsNullOrWhiteSpace(_flowsheetJson))
        {
            message = "Flowsheet JSON has not been loaded.";
            ValStatus = CapeValidationStatus.Invalid;
            return false;
        }

        if (string.IsNullOrWhiteSpace(_selectedPackageId))
        {
            message = "Property package id has not been selected.";
            ValStatus = CapeValidationStatus.Invalid;
            return false;
        }

        if ((_manifestPath is null) != (_payloadPath is null))
        {
            message = "Property package manifest and payload paths must be provided together.";
            ValStatus = CapeValidationStatus.Invalid;
            return false;
        }

        message = "The MVP CAPE-OPEN unit operation skeleton is configured.";
        ValStatus = CapeValidationStatus.Valid;
        return true;
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

        string validationMessage = string.Empty;
        if (!Validate(ref validationMessage))
        {
            if (string.Equals(validationMessage, "Flowsheet JSON has not been loaded.", StringComparison.Ordinal))
            {
                throw CreateBadInvocation(
                    UnitInterfaceName,
                    nameof(Calculate),
                    validationMessage,
                    nameof(LoadFlowsheetJson));
            }

            if (string.Equals(validationMessage, "Property package id has not been selected.", StringComparison.Ordinal))
            {
                throw CreateBadInvocation(
                    UnitInterfaceName,
                    nameof(Calculate),
                    validationMessage,
                    nameof(SelectPropertyPackage));
            }

            throw new CapeFailedInitialisationException(
                validationMessage,
                CreateContext(UnitInterfaceName, nameof(Calculate), moreInfo: validationMessage));
        }

        throw new CapeNoImplementationException(
            "Native calculate wiring is not implemented in UnitOp.Mvp yet.",
            CreateContext(UnitInterfaceName, nameof(Calculate)));
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

    private void InvalidateValidation()
    {
        if (!_terminated)
        {
            ValStatus = CapeValidationStatus.NotValidated;
        }
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
}
