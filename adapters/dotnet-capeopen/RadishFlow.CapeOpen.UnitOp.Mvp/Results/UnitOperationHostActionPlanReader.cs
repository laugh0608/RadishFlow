using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostActionPlanReader
{
    public static UnitOperationHostActionPlan Read(
        RadishFlowCapeOpenUnitOperation unitOperation)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);
        return Read(UnitOperationHostConfigurationReader.Read(unitOperation));
    }

    public static UnitOperationHostActionPlan Read(
        UnitOperationHostConfigurationSnapshot configurationSnapshot)
    {
        ArgumentNullException.ThrowIfNull(configurationSnapshot);

        var resolutionContext = CreateResolutionContext(configurationSnapshot);
        var actions = configurationSnapshot.BlockingIssues
            .Select((issue, index) => CreateAction(resolutionContext, issue, index + 1))
            .ToArray();
        var groups = actions
            .GroupBy(static action => action.GroupKind)
            .OrderBy(static group => UnitOperationHostActionDefinitionCatalog.GetByIssueKind(group.First().IssueKind).GroupOrder)
            .Select(static group => new UnitOperationHostActionGroup(
                Kind: group.Key,
                Title: UnitOperationHostActionDefinitionCatalog.GetByIssueKind(group.First().IssueKind).GroupTitle,
                Actions: group.OrderBy(static action => action.RecommendedOrder).ToArray()))
            .ToArray();

        return new UnitOperationHostActionPlan(
            State: configurationSnapshot.State,
            Headline: configurationSnapshot.Headline,
            Groups: groups,
            Actions: actions);
    }

    private static UnitOperationHostActionResolutionContext CreateResolutionContext(
        UnitOperationHostConfigurationSnapshot configurationSnapshot)
    {
        var parameterNames = configurationSnapshot.ParameterEntries
            .Select(static entry => entry.Name)
            .ToDictionary(static name => name, static name => name, StringComparer.OrdinalIgnoreCase);
        var portNames = configurationSnapshot.PortEntries
            .Select(static entry => entry.Name)
            .ToDictionary(static name => name, static name => name, StringComparer.OrdinalIgnoreCase);

        return new UnitOperationHostActionResolutionContext(parameterNames, portNames);
    }

    private static UnitOperationHostActionItem CreateAction(
        UnitOperationHostActionResolutionContext resolutionContext,
        UnitOperationHostConfigurationIssue issue,
        int recommendedOrder)
    {
        var definition = UnitOperationHostActionDefinitionCatalog.GetByIssueKind(issue.Kind);
        var target = new UnitOperationHostActionTarget(
            Kind: definition.TargetKind,
            Names: definition.ResolveTargetNames(resolutionContext, issue));

        return new UnitOperationHostActionItem(
            RecommendedOrder: recommendedOrder,
            GroupKind: definition.GroupKind,
            Target: target,
            Reason: issue.Message,
            IsBlocking: definition.IsBlocking,
            CanonicalOperationName: issue.OperationName,
            IssueKind: issue.Kind);
    }
}

public sealed record UnitOperationHostActionPlan(
    UnitOperationHostConfigurationState State,
    string Headline,
    IReadOnlyList<UnitOperationHostActionGroup> Groups,
    IReadOnlyList<UnitOperationHostActionItem> Actions)
{
    public int ActionCount => Actions.Count;

    public bool HasBlockingActions => Actions.Any(static action => action.IsBlocking);

    public bool ContainsCanonicalOperation(string operationName)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(operationName);
        return Actions.Any(action =>
            string.Equals(action.CanonicalOperationName, operationName, StringComparison.Ordinal));
    }
}

public sealed record UnitOperationHostActionGroup(
    UnitOperationHostActionGroupKind Kind,
    string Title,
    IReadOnlyList<UnitOperationHostActionItem> Actions);

public sealed record UnitOperationHostActionItem(
    int RecommendedOrder,
    UnitOperationHostActionGroupKind GroupKind,
    UnitOperationHostActionTarget Target,
    string Reason,
    bool IsBlocking,
    string? CanonicalOperationName,
    UnitOperationHostConfigurationIssueKind IssueKind);

public sealed record UnitOperationHostActionTarget(
    UnitOperationHostActionTargetKind Kind,
    IReadOnlyList<string> Names)
{
    public string PrimaryName => Names.Count == 0 ? string.Empty : Names[0];
}

public enum UnitOperationHostActionGroupKind
{
    Lifecycle,
    Parameters,
    Ports,
    Terminal,
}

public enum UnitOperationHostActionTargetKind
{
    Unit,
    Parameter,
    Port,
}
