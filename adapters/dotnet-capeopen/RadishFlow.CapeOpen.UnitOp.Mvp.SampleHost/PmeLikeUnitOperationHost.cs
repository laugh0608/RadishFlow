using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal sealed class PmeLikeUnitOperationHost
{
    private readonly string? _nativeLibraryDirectory;

    public PmeLikeUnitOperationHost(string? nativeLibraryDirectory)
    {
        _nativeLibraryDirectory = nativeLibraryDirectory;
    }

    public PmeLikeUnitOperationSession CreateSession()
    {
        var unitOperation = new RadishFlowCapeOpenUnitOperation();
        try
        {
            if (!string.IsNullOrWhiteSpace(_nativeLibraryDirectory))
            {
                unitOperation.ConfigureNativeLibraryDirectory(_nativeLibraryDirectory);
            }

            var constructedViews = UnitOperationHostViewReader.Read(unitOperation);
            unitOperation.Initialize();
            var initializedViews = UnitOperationHostViewReader.Read(unitOperation);

            return new PmeLikeUnitOperationSession(
                unitOperation,
                constructedViews,
                initializedViews);
        }
        catch
        {
            unitOperation.Dispose();
            throw;
        }
    }
}

internal sealed class PmeLikeUnitOperationSession : IDisposable
{
    private readonly RadishFlowCapeOpenUnitOperation _unitOperation;
    private bool _terminated;
    private bool _disposed;

    public PmeLikeUnitOperationSession(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostViewSnapshot constructedViews,
        UnitOperationHostViewSnapshot initializedViews)
    {
        _unitOperation = unitOperation;
        ConstructedViews = constructedViews;
        InitializedViews = initializedViews;
    }

    public UnitOperationHostViewSnapshot ConstructedViews { get; }

    public UnitOperationHostViewSnapshot InitializedViews { get; }

    public UnitOperationHostViewSnapshot ReadViews()
    {
        ThrowIfDisposed();
        return UnitOperationHostViewReader.Read(_unitOperation);
    }

    public UnitOperationHostActionExecutionRequestPlan PlanInputApplication(
        PmeLikeUnitOperationInput input)
    {
        ArgumentNullException.ThrowIfNull(input);
        ThrowIfClosed();

        return UnitOperationHostActionExecutionRequestPlanner.Plan(
            ReadViews().ActionPlan,
            input.ToActionInputSet());
    }

    public PmeLikeUnitOperationRoundResult ExecuteRound(
        PmeLikeUnitOperationInput input)
    {
        ArgumentNullException.ThrowIfNull(input);
        ThrowIfClosed();

        var views = ReadViews();
        var actionInputSet = input.ToActionInputSet();
        var requestPlan = UnitOperationHostActionExecutionRequestPlanner.Plan(
            views.ActionPlan,
            actionInputSet);
        var supplementalCommands = input.CreateSupplementalMutationCommands(views.ActionPlan);

        var roundOutcome = UnitOperationHostRoundOrchestrator.Execute(
            _unitOperation,
            new UnitOperationHostRoundRequest(
                actionInputSet: actionInputSet,
                executeReadyActions: true,
                runValidation: true,
                runCalculation: true,
                supplementalMutationCommands: supplementalCommands,
                actionPlan: views.ActionPlan));

        return new PmeLikeUnitOperationRoundResult(
            RequestPlan: requestPlan,
            SupplementalMutationCommands: supplementalCommands,
            Outcome: roundOutcome);
    }

    public UnitOperationHostSessionSnapshot Terminate()
    {
        if (_disposed)
        {
            throw new ObjectDisposedException(nameof(PmeLikeUnitOperationSession));
        }

        if (!_terminated)
        {
            _unitOperation.Terminate();
            _terminated = true;
        }

        return UnitOperationHostSessionReader.Read(_unitOperation);
    }

    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }

        _unitOperation.Dispose();
        _terminated = true;
        _disposed = true;
    }

    private void ThrowIfDisposed()
    {
        if (_disposed)
        {
            throw new ObjectDisposedException(nameof(PmeLikeUnitOperationSession));
        }
    }

    private void ThrowIfClosed()
    {
        ThrowIfDisposed();
        if (_terminated)
        {
            throw new InvalidOperationException("The PME-like unit operation session has already been terminated.");
        }
    }
}

internal sealed class PmeLikeUnitOperationInput
{
    public PmeLikeUnitOperationInput(
        string flowsheetJson,
        string packageId,
        string? manifestPath,
        string? payloadPath,
        object feedMaterialObject,
        object productMaterialObject)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(flowsheetJson);
        ArgumentException.ThrowIfNullOrWhiteSpace(packageId);
        ArgumentNullException.ThrowIfNull(feedMaterialObject);
        ArgumentNullException.ThrowIfNull(productMaterialObject);

        if (string.IsNullOrWhiteSpace(manifestPath) != string.IsNullOrWhiteSpace(payloadPath))
        {
            throw new ArgumentException("Manifest and payload paths must be provided together.");
        }

        FlowsheetJson = flowsheetJson;
        PackageId = packageId;
        ManifestPath = manifestPath;
        PayloadPath = payloadPath;
        FeedMaterialObject = feedMaterialObject;
        ProductMaterialObject = productMaterialObject;
    }

    public string FlowsheetJson { get; }

    public string PackageId { get; }

    public string? ManifestPath { get; }

    public string? PayloadPath { get; }

    public object FeedMaterialObject { get; }

    public object ProductMaterialObject { get; }

    public bool HasPackageFiles =>
        !string.IsNullOrWhiteSpace(ManifestPath) &&
        !string.IsNullOrWhiteSpace(PayloadPath);

    public UnitOperationHostActionExecutionInputSet ToActionInputSet()
    {
        return new UnitOperationHostActionExecutionInputSet(
            parameterValues: new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
            {
                [UnitOperationParameterCatalog.FlowsheetJson.Name] = FlowsheetJson,
                [UnitOperationParameterCatalog.PropertyPackageId.Name] = PackageId,
            },
            portObjects: new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase)
            {
                [UnitOperationPortCatalog.Feed.Name] = FeedMaterialObject,
                [UnitOperationPortCatalog.Product.Name] = ProductMaterialObject,
            });
    }

    public IReadOnlyList<UnitOperationHostObjectMutationCommand> CreateSupplementalMutationCommands(
        UnitOperationHostActionPlan actionPlan)
    {
        ArgumentNullException.ThrowIfNull(actionPlan);

        if (!HasPackageFiles ||
            actionPlan.ContainsCanonicalOperation(nameof(RadishFlowCapeOpenUnitOperation.LoadPropertyPackageFiles)))
        {
            return [];
        }

        return
        [
            UnitOperationHostObjectMutationCommand.SetParameterValue(
                UnitOperationParameterCatalog.PropertyPackageManifestPath.Name,
                ManifestPath),
            UnitOperationHostObjectMutationCommand.SetParameterValue(
                UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name,
                PayloadPath),
        ];
    }
}

internal sealed record PmeLikeUnitOperationRoundResult(
    UnitOperationHostActionExecutionRequestPlan RequestPlan,
    IReadOnlyList<UnitOperationHostObjectMutationCommand> SupplementalMutationCommands,
    UnitOperationHostRoundOutcome Outcome);

internal sealed class PmeLikeMaterialObject : ICapeIdentification
{
    public PmeLikeMaterialObject(string componentName)
    {
        ComponentName = componentName;
        ComponentDescription = "PME-like host placeholder material object.";
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }
}
