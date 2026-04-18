using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal sealed class UnitOperationSmokeHostDriver : IDisposable
{
    private readonly RadishFlowCapeOpenUnitOperation _unitOperation;
    private readonly SmokeOptions _options;
    private readonly string _projectJson;
    private bool _disposed;

    public UnitOperationSmokeHostDriver(SmokeOptions options, string projectJson)
    {
        ArgumentNullException.ThrowIfNull(options);
        ArgumentException.ThrowIfNullOrWhiteSpace(projectJson);

        _options = options;
        _projectJson = projectJson;
        _unitOperation = new RadishFlowCapeOpenUnitOperation();

        if (!string.IsNullOrWhiteSpace(options.NativeLibraryDirectory))
        {
            _unitOperation.ConfigureNativeLibraryDirectory(options.NativeLibraryDirectory);
        }

        Parameters = _unitOperation.Parameters;
        Ports = _unitOperation.Ports;
        ParameterCollection = (ICapeCollection)Parameters;
        PortCollection = (ICapeCollection)Ports;

        FlowsheetParameter = Parameters.GetByName(UnitOperationParameterCatalog.FlowsheetJson.Name);
        PackageIdParameter = Parameters.GetByName(UnitOperationParameterCatalog.PropertyPackageId.Name);
        ManifestPathParameter = Parameters.GetByName(UnitOperationParameterCatalog.PropertyPackageManifestPath.Name);
        PayloadPathParameter = Parameters.GetByOneBasedIndex(4);
        FeedPort = Ports.GetByName(UnitOperationPortCatalog.Feed.Name);
        ProductPort = Ports.GetByOneBasedIndex(2);
    }

    public RadishFlowCapeOpenUnitOperation UnitOperation => _unitOperation;

    public UnitOperationPlaceholderCollection<UnitOperationParameterPlaceholder> Parameters { get; }

    public UnitOperationPlaceholderCollection<UnitOperationPortPlaceholder> Ports { get; }

    public ICapeCollection ParameterCollection { get; }

    public ICapeCollection PortCollection { get; }

    public UnitOperationParameterPlaceholder FlowsheetParameter { get; }

    public UnitOperationParameterPlaceholder PackageIdParameter { get; }

    public UnitOperationParameterPlaceholder ManifestPathParameter { get; }

    public UnitOperationParameterPlaceholder PayloadPathParameter { get; }

    public UnitOperationPortPlaceholder FeedPort { get; }

    public UnitOperationPortPlaceholder ProductPort { get; }

    public void Initialize()
    {
        ThrowIfDisposed();
        _unitOperation.Initialize();
    }

    public void ConfigureMinimumInputs(bool includePackageId)
    {
        ThrowIfDisposed();

        FlowsheetParameter.value = _projectJson;
        if (_options.LoadPackageFiles)
        {
            ManifestPathParameter.value = _options.ManifestPath!;
            PayloadPathParameter.value = _options.PayloadPath!;
        }

        if (includePackageId)
        {
            PackageIdParameter.value = _options.PackageId;
        }
    }

    public void ConnectRequiredPorts()
    {
        ThrowIfDisposed();
        FeedPort.Connect(new SmokeConnectedObject("Smoke Feed"));
        ProductPort.Connect(new SmokeConnectedObject("Smoke Product"));
    }

    public UnitOperationSmokeValidationResult Validate()
    {
        ThrowIfDisposed();

        var message = string.Empty;
        var isValid = _unitOperation.Validate(ref message);
        return new UnitOperationSmokeValidationResult(isValid, message);
    }

    public UnitOperationSmokeCalculationAttempt Calculate()
    {
        ThrowIfDisposed();

        try
        {
            _unitOperation.Calculate();
            return UnitOperationSmokeCalculationAttempt.FromSuccess(ReadReport());
        }
        catch (CapeOpenException error)
        {
            return UnitOperationSmokeCalculationAttempt.FromFailure(
                ReadReport(),
                error,
                ClassifyFailure(error));
        }
    }

    public UnitOperationHostConfigurationSnapshot ReadConfiguration()
    {
        ThrowIfDisposed();
        return UnitOperationHostConfigurationReader.Read(_unitOperation);
    }

    public UnitOperationHostActionPlan ReadActionPlan()
    {
        ThrowIfDisposed();
        return UnitOperationHostActionPlanReader.Read(_unitOperation);
    }

    public UnitOperationHostPortMaterialSnapshot ReadPortMaterial()
    {
        ThrowIfDisposed();
        return UnitOperationHostPortMaterialReader.Read(_unitOperation);
    }

    public UnitOperationHostReportBundle ReadReport()
    {
        ThrowIfDisposed();

        var snapshot = UnitOperationHostReportReader.Read(_unitOperation);
        var presentation = UnitOperationHostReportPresenter.Present(snapshot);
        var document = UnitOperationHostReportFormatter.Format(presentation);
        return new UnitOperationHostReportBundle(snapshot, presentation, document);
    }

    public void Terminate()
    {
        if (_disposed)
        {
            return;
        }

        _unitOperation.Terminate();
    }

    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }

        _unitOperation.Dispose();
        _disposed = true;
    }

    private static UnitOperationHostDriverFailureKind ClassifyFailure(CapeOpenException error)
    {
        if (!string.IsNullOrWhiteSpace(error.NativeStatus))
        {
            return UnitOperationHostDriverFailureKind.Native;
        }

        if (error is CapeBadInvocationOrderException or CapeFailedInitialisationException)
        {
            if (string.Equals(
                error.RequestedOperation,
                nameof(RadishFlowCapeOpenUnitOperation.Initialize),
                StringComparison.Ordinal))
            {
                return UnitOperationHostDriverFailureKind.InvocationOrder;
            }

            return UnitOperationHostDriverFailureKind.Validation;
        }

        return UnitOperationHostDriverFailureKind.Unknown;
    }

    private void ThrowIfDisposed()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
    }
}

internal enum UnitOperationHostDriverFailureKind
{
    InvocationOrder,
    Validation,
    Native,
    Unknown,
}

internal sealed record UnitOperationSmokeValidationResult(
    bool IsValid,
    string Message);

internal sealed record UnitOperationHostReportBundle(
    UnitOperationHostReportSnapshot Snapshot,
    UnitOperationHostReportPresentation Presentation,
    UnitOperationHostReportDocument Document);

internal sealed record UnitOperationSmokeCalculationAttempt(
    bool Succeeded,
    UnitOperationHostReportBundle Report,
    CapeOpenException? Failure,
    UnitOperationHostDriverFailureKind? FailureKind)
{
    public static UnitOperationSmokeCalculationAttempt FromSuccess(UnitOperationHostReportBundle report)
    {
        return new UnitOperationSmokeCalculationAttempt(
            Succeeded: true,
            Report: report,
            Failure: null,
            FailureKind: null);
    }

    public static UnitOperationSmokeCalculationAttempt FromFailure(
        UnitOperationHostReportBundle report,
        CapeOpenException failure,
        UnitOperationHostDriverFailureKind failureKind)
    {
        return new UnitOperationSmokeCalculationAttempt(
            Succeeded: false,
            Report: report,
            Failure: failure,
            FailureKind: failureKind);
    }

    public TFailure ExpectFailure<TFailure>(
        UnitOperationHostDriverFailureKind expectedFailureKind,
        string scenario)
        where TFailure : CapeOpenException
    {
        if (Succeeded)
        {
            throw new InvalidOperationException($"Expected {typeof(TFailure).Name} failure for {scenario}, but calculation succeeded.");
        }

        if (Failure is not TFailure typedFailure)
        {
            var actualType = Failure?.GetType().Name ?? "<null>";
            throw new InvalidOperationException(
                $"Expected {typeof(TFailure).Name} for {scenario}, but received {actualType}.");
        }

        if (FailureKind != expectedFailureKind)
        {
            throw new InvalidOperationException(
                $"Expected {expectedFailureKind} failure classification for {scenario}, but received {FailureKind?.ToString() ?? "<null>"}.");
        }

        return typedFailure;
    }
}
