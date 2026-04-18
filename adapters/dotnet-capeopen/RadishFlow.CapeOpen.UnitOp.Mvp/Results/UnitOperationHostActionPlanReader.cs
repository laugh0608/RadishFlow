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

        var actions = configurationSnapshot.BlockingIssues
            .Select(CreateFactory(configurationSnapshot))
            .ToArray();
        var groups = actions
            .GroupBy(static action => action.GroupKind)
            .OrderBy(static group => GetGroupOrder(group.Key))
            .Select(static group => new UnitOperationHostActionGroup(
                Kind: group.Key,
                Title: GetGroupTitle(group.Key),
                Actions: group.OrderBy(static action => action.RecommendedOrder).ToArray()))
            .ToArray();

        return new UnitOperationHostActionPlan(
            State: configurationSnapshot.State,
            Headline: configurationSnapshot.Headline,
            Groups: groups,
            Actions: actions);
    }

    private static Func<UnitOperationHostConfigurationIssue, int, UnitOperationHostActionItem> CreateFactory(
        UnitOperationHostConfigurationSnapshot configurationSnapshot)
    {
        var parameterNames = configurationSnapshot.ParameterEntries
            .Select(static entry => entry.Name)
            .ToDictionary(static name => name, static name => name, StringComparer.OrdinalIgnoreCase);
        var portNames = configurationSnapshot.PortEntries
            .Select(static entry => entry.Name)
            .ToDictionary(static name => name, static name => name, StringComparer.OrdinalIgnoreCase);

        return (issue, index) => CreateAction(parameterNames, portNames, issue, index + 1);
    }

    private static UnitOperationHostActionItem CreateAction(
        IReadOnlyDictionary<string, string> parameterNames,
        IReadOnlyDictionary<string, string> portNames,
        UnitOperationHostConfigurationIssue issue,
        int recommendedOrder)
    {
        var target = issue.Kind switch
        {
            UnitOperationHostConfigurationIssueKind.InitializeRequired => new UnitOperationHostActionTarget(
                Kind: UnitOperationHostActionTargetKind.Unit,
                Names: [issue.TargetName]),
            UnitOperationHostConfigurationIssueKind.RequiredParameterMissing => new UnitOperationHostActionTarget(
                Kind: UnitOperationHostActionTargetKind.Parameter,
                Names: [ResolveName(parameterNames, issue.TargetName)]),
            UnitOperationHostConfigurationIssueKind.CompanionParameterMismatch => new UnitOperationHostActionTarget(
                Kind: UnitOperationHostActionTargetKind.Parameter,
                Names: ResolveCompanionNames(parameterNames, issue.TargetName)),
            UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected => new UnitOperationHostActionTarget(
                Kind: UnitOperationHostActionTargetKind.Port,
                Names: [ResolveName(portNames, issue.TargetName)]),
            UnitOperationHostConfigurationIssueKind.Terminated => new UnitOperationHostActionTarget(
                Kind: UnitOperationHostActionTargetKind.Unit,
                Names: [issue.TargetName]),
            _ => throw new ArgumentOutOfRangeException(nameof(issue), issue.Kind, "Unknown host configuration issue kind."),
        };

        var groupKind = issue.Kind switch
        {
            UnitOperationHostConfigurationIssueKind.InitializeRequired => UnitOperationHostActionGroupKind.Lifecycle,
            UnitOperationHostConfigurationIssueKind.RequiredParameterMissing => UnitOperationHostActionGroupKind.Parameters,
            UnitOperationHostConfigurationIssueKind.CompanionParameterMismatch => UnitOperationHostActionGroupKind.Parameters,
            UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected => UnitOperationHostActionGroupKind.Ports,
            UnitOperationHostConfigurationIssueKind.Terminated => UnitOperationHostActionGroupKind.Terminal,
            _ => throw new ArgumentOutOfRangeException(nameof(issue), issue.Kind, "Unknown host configuration issue kind."),
        };

        return new UnitOperationHostActionItem(
            RecommendedOrder: recommendedOrder,
            GroupKind: groupKind,
            Target: target,
            Reason: issue.Message,
            IsBlocking: true,
            CanonicalOperationName: issue.OperationName,
            IssueKind: issue.Kind);
    }

    private static string ResolveName(
        IReadOnlyDictionary<string, string> names,
        string name)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(name);
        return names.TryGetValue(name, out var resolvedName)
            ? resolvedName
            : name;
    }

    private static IReadOnlyList<string> ResolveCompanionNames(
        IReadOnlyDictionary<string, string> parameterNames,
        string pairKey)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(pairKey);
        return pairKey
            .Split('|', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
            .Select(name => ResolveName(parameterNames, name))
            .ToArray();
    }

    private static int GetGroupOrder(UnitOperationHostActionGroupKind groupKind)
    {
        return groupKind switch
        {
            UnitOperationHostActionGroupKind.Lifecycle => 0,
            UnitOperationHostActionGroupKind.Parameters => 1,
            UnitOperationHostActionGroupKind.Ports => 2,
            UnitOperationHostActionGroupKind.Terminal => 3,
            _ => throw new ArgumentOutOfRangeException(nameof(groupKind), groupKind, "Unknown action group kind."),
        };
    }

    private static string GetGroupTitle(UnitOperationHostActionGroupKind groupKind)
    {
        return groupKind switch
        {
            UnitOperationHostActionGroupKind.Lifecycle => "Lifecycle",
            UnitOperationHostActionGroupKind.Parameters => "Parameters",
            UnitOperationHostActionGroupKind.Ports => "Ports",
            UnitOperationHostActionGroupKind.Terminal => "Terminal State",
            _ => throw new ArgumentOutOfRangeException(nameof(groupKind), groupKind, "Unknown action group kind."),
        };
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
