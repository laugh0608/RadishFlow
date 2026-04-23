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

    public UnitOperationParameterCollection Parameters { get; }

    public UnitOperationPortCollection Ports { get; }

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

    public UnitOperationHostActionExecutionInputSet CreateMinimumConfigurationInputSet(bool includePackageId)
    {
        ThrowIfDisposed();
        return CreateMinimumConfigurationInputSetCore(includePackageId);
    }

    public IReadOnlyList<UnitOperationHostObjectMutationCommand> CreateOptionalPackageFileMutationCommands()
    {
        ThrowIfDisposed();
        if (!_options.LoadPackageFiles)
        {
            return [];
        }

        var actionPlan = ReadActionPlan();
        if (actionPlan.ContainsCanonicalOperation(nameof(RadishFlowCapeOpenUnitOperation.LoadPropertyPackageFiles)))
        {
            return [];
        }

        return CreateOptionalPackageFileMutationCommandsCore();
    }

    public UnitOperationHostActionExecutionOrchestrationResult ApplyRequiredPortAction(string portName, string componentName)
    {
        ThrowIfDisposed();
        ArgumentException.ThrowIfNullOrWhiteSpace(portName);
        ArgumentException.ThrowIfNullOrWhiteSpace(componentName);

        var actionPlan = ReadActionPlan();
        var action = actionPlan.Actions.Single(action =>
            action.IssueKind == UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected &&
            action.Target.Names.Any(targetName => string.Equals(targetName, portName, StringComparison.OrdinalIgnoreCase)));
        var group = actionPlan.Groups.Single(group => group.Kind == action.GroupKind);
        return UnitOperationHostActionExecutionOrchestrator.ExecutePlannedActions(
            _unitOperation,
            new UnitOperationHostActionPlan(
                State: actionPlan.State,
                Headline: actionPlan.Headline,
                Groups: [new UnitOperationHostActionGroup(group.Kind, group.Title, [action])],
                Actions: [action]),
            new UnitOperationHostActionExecutionInputSet(
                portObjects: new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase)
                {
                    [portName] = new SmokeConnectedObject(componentName),
                }));
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
        return new UnitOperationSmokeValidationResult(
            UnitOperationHostValidationRunner.Validate(_unitOperation));
    }

    public UnitOperationSmokeRoundResult ExecuteRound(UnitOperationHostRoundRequest request)
    {
        ThrowIfDisposed();
        ArgumentNullException.ThrowIfNull(request);

        var outcome = UnitOperationHostRoundOrchestrator.Execute(_unitOperation, request);
        return new UnitOperationSmokeRoundResult(
            outcome,
            CreateReportBundle(outcome.Report));
    }

    public UnitOperationSmokeCalculationAttempt Calculate()
    {
        ThrowIfDisposed();

        var outcome = UnitOperationHostCalculationRunner.Calculate(_unitOperation);
        return UnitOperationSmokeCalculationAttempt.FromOutcome(
            outcome,
            CreateReportBundle(outcome.Report),
            outcome.Failure is null ? null : ClassifyFailure(outcome.Failure));
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

    public UnitOperationHostExecutionSnapshot ReadExecution()
    {
        ThrowIfDisposed();
        return UnitOperationHostExecutionReader.Read(_unitOperation);
    }

    public UnitOperationHostSessionSnapshot ReadSession()
    {
        ThrowIfDisposed();
        return UnitOperationHostSessionReader.Read(_unitOperation);
    }

    public UnitOperationHostReportBundle ReadReport()
    {
        ThrowIfDisposed();

        var snapshot = UnitOperationHostReportReader.Read(_unitOperation);
        return CreateReportBundle(snapshot);
    }

    private static UnitOperationHostReportBundle CreateReportBundle(
        UnitOperationHostReportSnapshot snapshot)
    {
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

    private UnitOperationHostActionExecutionInputSet CreateMinimumConfigurationInputSetCore(bool includePackageId)
    {
        var values = new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase);
        values[UnitOperationParameterCatalog.FlowsheetJson.Name] = _projectJson;

        if (includePackageId)
        {
            values[UnitOperationParameterCatalog.PropertyPackageId.Name] = _options.PackageId;
        }

        if (_options.LoadPackageFiles)
        {
            values[UnitOperationParameterCatalog.PropertyPackageManifestPath.Name] = _options.ManifestPath!;
            values[UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name] = _options.PayloadPath!;
        }

        return new UnitOperationHostActionExecutionInputSet(
            parameterValues: values,
            portObjects: new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase)
            {
                [UnitOperationPortCatalog.Feed.Name] = new SmokeConnectedObject("Feed Smoke"),
                [UnitOperationPortCatalog.Product.Name] = new SmokeConnectedObject("Product Smoke"),
            });
    }

    private IReadOnlyList<UnitOperationHostObjectMutationCommand> CreateOptionalPackageFileMutationCommandsCore()
    {
        if (!_options.LoadPackageFiles)
        {
            return [];
        }

        return
        [
            UnitOperationHostObjectMutationCommand.SetParameterValue(
                UnitOperationParameterCatalog.PropertyPackageManifestPath.Name,
                _options.ManifestPath!),
            UnitOperationHostObjectMutationCommand.SetParameterValue(
                UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name,
                _options.PayloadPath!),
        ];
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
    UnitOperationHostValidationOutcome Outcome)
{
    public bool IsValid => Outcome.IsValid;

    public string Message => Outcome.Message;

    public UnitOperationHostViewSnapshot Views => Outcome.Views;

    public UnitOperationHostFollowUp FollowUp => Outcome.FollowUp;

    public UnitOperationHostSessionSnapshot Session => Outcome.Session;

    public UnitOperationHostReportSnapshot Report => Outcome.Report;
}

internal sealed record UnitOperationHostReportBundle(
    UnitOperationHostReportSnapshot Snapshot,
    UnitOperationHostReportPresentation Presentation,
    UnitOperationHostReportDocument Document);

internal sealed record UnitOperationSmokeCalculationAttempt(
    bool Succeeded,
    UnitOperationHostCalculationOutcome Outcome,
    UnitOperationHostReportBundle Report,
    CapeOpenException? Failure,
    UnitOperationHostDriverFailureKind? FailureKind)
{
    public UnitOperationHostViewSnapshot Views => Outcome.Views;

    public UnitOperationHostFollowUp FollowUp => Outcome.FollowUp;

    public UnitOperationHostSessionSnapshot Session => Outcome.Session;

    public UnitOperationHostExecutionSnapshot Execution => Outcome.Execution;

    public static UnitOperationSmokeCalculationAttempt FromOutcome(
        UnitOperationHostCalculationOutcome outcome,
        UnitOperationHostReportBundle report,
        UnitOperationHostDriverFailureKind? failureKind)
    {
        return new UnitOperationSmokeCalculationAttempt(
            Succeeded: outcome.Succeeded,
            Outcome: outcome,
            Report: report,
            Failure: outcome.Failure,
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

internal sealed record UnitOperationSmokeRoundResult(
    UnitOperationHostRoundOutcome Outcome,
    UnitOperationHostReportBundle ReportBundle)
{
    public UnitOperationHostActionExecutionOrchestrationResult? ActionExecution => Outcome.ActionExecution;

    public UnitOperationHostValidationOutcome? Validation => Outcome.Validation;

    public UnitOperationHostCalculationOutcome? Calculation => Outcome.Calculation;

    public UnitOperationHostFollowUp FollowUp => Outcome.FollowUp;

    public UnitOperationHostRoundStopKind StopKind => Outcome.StopKind;

    public UnitOperationHostConfigurationSnapshot Configuration => Outcome.Configuration;

    public UnitOperationHostActionPlan ActionPlan => Outcome.ActionPlan;

    public UnitOperationHostPortMaterialSnapshot PortMaterial => Outcome.PortMaterial;

    public UnitOperationHostExecutionSnapshot Execution => Outcome.Execution;

    public UnitOperationHostSessionSnapshot Session => Outcome.Session;

    public UnitOperationHostReportSnapshot Report => Outcome.Report;
}
