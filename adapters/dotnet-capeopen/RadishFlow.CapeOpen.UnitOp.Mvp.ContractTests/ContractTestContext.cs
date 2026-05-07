using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Guids;
using RadishFlow.CapeOpen.Interop.Ole;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Persistence;
using RadishFlow.CapeOpen.Interop.Thermo;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text.Json;

internal sealed class ContractTestContext : IDisposable
{
    private readonly ContractTestOptions _options;

    public ContractTestContext(ContractTestOptions options)
    {
        _options = options;
        UnitOperation = new RadishFlowCapeOpenUnitOperation();
        UnitOperation.ConfigureNativeLibraryDirectory(options.NativeLibraryDirectory);
        ParameterCollection = (ICapeCollection)UnitOperation.Parameters;
        PortCollection = (ICapeCollection)UnitOperation.Ports;
        FlowsheetParameter = UnitOperation.Parameters.GetByName(UnitOperationParameterCatalog.FlowsheetJson.Name);
        PackageIdParameter = UnitOperation.Parameters.GetByName(UnitOperationParameterCatalog.PropertyPackageId.Name);
        ManifestPathParameter = UnitOperation.Parameters.GetByName(UnitOperationParameterCatalog.PropertyPackageManifestPath.Name);
        PayloadPathParameter = UnitOperation.Parameters.GetByOneBasedIndex(4);
        FeedPort = UnitOperation.Ports.GetByName(UnitOperationPortCatalog.Feed.Name);
        ProductPort = UnitOperation.Ports.GetByName(UnitOperationPortCatalog.Product.Name);
    }

    public RadishFlowCapeOpenUnitOperation UnitOperation { get; }

    public string ManifestPath => _options.ManifestPath;

    public string PayloadPath => _options.PayloadPath;

    public string PackageId => _options.PackageId;

    public string FlowsheetJsonText => File.ReadAllText(_options.ProjectPath);

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
        UnitOperation.Initialize();
    }

    public void LoadFlowsheet()
    {
        UnitOperation.LoadFlowsheetJson(FlowsheetJsonText);
    }

    public void LoadPackageFiles()
    {
        UnitOperation.LoadPropertyPackageFiles(_options.ManifestPath, _options.PayloadPath);
    }

    public void SelectPackage()
    {
        UnitOperation.SelectPropertyPackage(_options.PackageId);
    }

    public IReadOnlyList<string> ReadFeedStreamComponentIds(string streamId)
    {
        using var document = JsonDocument.Parse(FlowsheetJsonText);
        var composition = document.RootElement
            .GetProperty("document")
            .GetProperty("flowsheet")
            .GetProperty("streams")
            .GetProperty(streamId)
            .GetProperty("overall_mole_fractions");
        return composition.EnumerateObject()
            .Select(static property => property.Name)
            .ToArray();
    }

    public void ConnectRequiredPorts()
    {
        UnitOperation.Ports.GetByName(UnitOperationPortCatalog.Feed.Name).Connect(new ContractConnectedObject("Contract Feed"));
        UnitOperation.Ports.GetByName(UnitOperationPortCatalog.Product.Name).Connect(new ContractConnectedObject("Contract Product"));
    }

    public void DisconnectProductPort()
    {
        UnitOperation.Ports.GetByName(UnitOperationPortCatalog.Product.Name).Disconnect();
    }

    public bool IsProductPortConnected()
    {
        return UnitOperation.Ports.GetByName(UnitOperationPortCatalog.Product.Name).connectedObject is not null;
    }

    public UnitOperationHostConfigurationSnapshot ReadConfiguration()
    {
        return UnitOperationHostConfigurationReader.Read(UnitOperation);
    }

    public UnitOperationHostObjectRuntimeSnapshot ReadObjectRuntime()
    {
        return UnitOperationHostObjectRuntimeReader.Read(UnitOperation);
    }

    public UnitOperationHostActionPlan ReadActionPlan()
    {
        return UnitOperationHostActionPlanReader.Read(ReadConfiguration());
    }

    public UnitOperationHostActionExecutionInputSet CreateMinimumConfigurationInputSet(
        bool includePackageId,
        bool includePackageFiles = true)
    {
        var values = new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
        {
            [UnitOperationParameterCatalog.FlowsheetJson.Name] = FlowsheetJsonText,
        };

        if (includePackageId)
        {
            values[UnitOperationParameterCatalog.PropertyPackageId.Name] = PackageId;
        }

        if (includePackageFiles)
        {
            values[UnitOperationParameterCatalog.PropertyPackageManifestPath.Name] = ManifestPath;
            values[UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name] = PayloadPath;
        }

        return new UnitOperationHostActionExecutionInputSet(
            parameterValues: values,
            portObjects: new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase)
            {
                [UnitOperationPortCatalog.Feed.Name] = new ContractConnectedObject("Contract Round Feed"),
                [UnitOperationPortCatalog.Product.Name] = new ContractConnectedObject("Contract Round Product"),
            });
    }

    public IReadOnlyList<UnitOperationHostObjectMutationCommand> CreateOptionalPackageFileMutationCommands()
    {
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

    public UnitOperationHostPortMaterialSnapshot ReadPortMaterial()
    {
        return UnitOperationHostPortMaterialReader.Read(UnitOperation);
    }

    public UnitOperationHostExecutionSnapshot ReadExecution()
    {
        return UnitOperationHostExecutionReader.Read(UnitOperation);
    }

    public UnitOperationHostSessionSnapshot ReadSession()
    {
        return UnitOperationHostSessionReader.Read(UnitOperation);
    }

    public UnitOperationHostValidationOutcome ValidateRound()
    {
        return UnitOperationHostValidationRunner.Validate(UnitOperation);
    }

    public UnitOperationHostCalculationOutcome CalculateRound()
    {
        return UnitOperationHostCalculationRunner.Calculate(UnitOperation);
    }

    public UnitOperationHostRoundOutcome ExecuteRound(UnitOperationHostRoundRequest? request = null)
    {
        return UnitOperationHostRoundOrchestrator.Execute(UnitOperation, request ?? UnitOperationHostRoundRequest.Default);
    }

    public void ConfigureMinimumValidInputs()
    {
        Initialize();
        LoadFlowsheet();
        LoadPackageFiles();
        SelectPackage();
        ConnectRequiredPorts();
    }

    public void Dispose()
    {
        UnitOperation.Dispose();
    }
}

internal sealed record ContractExpectedAction(
    UnitOperationHostActionGroupKind GroupKind,
    UnitOperationHostActionTargetKind TargetKind,
    IReadOnlyList<string> TargetNames,
    string? CanonicalOperationName,
    UnitOperationHostConfigurationIssueKind IssueKind,
    string ReasonFragment,
    bool IsBlocking)
{
    public void AssertMatches(
        UnitOperationHostActionItem actual,
        string scenario,
        int expectedOrder)
    {
        ContractAssert.Equal(expectedOrder, actual.RecommendedOrder, $"{scenario} should preserve recommended order.");
        ContractAssert.Equal(GroupKind, actual.GroupKind, $"{scenario} should preserve action group.");
        ContractAssert.Equal(TargetKind, actual.Target.Kind, $"{scenario} should preserve target kind.");
        ContractAssert.SequenceEqual(TargetNames, actual.Target.Names, $"{scenario} should preserve target names.");
        ContractAssert.Equal(IsBlocking, actual.IsBlocking, $"{scenario} should preserve blocking classification.");
        ContractAssert.Equal(IssueKind, actual.IssueKind, $"{scenario} should preserve issue kind.");
        ContractAssert.Equal(CanonicalOperationName, actual.CanonicalOperationName, $"{scenario} should preserve canonical operation.");
        ContractAssert.Contains(actual.Reason, ReasonFragment, $"{scenario} should preserve action reason.");
    }
}
