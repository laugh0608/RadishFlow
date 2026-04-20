namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostActionDefinitionCatalog
{
    public static UnitOperationHostActionDefinition InitializeRequired { get; } = new(
        IssueKind: UnitOperationHostConfigurationIssueKind.InitializeRequired,
        GroupKind: UnitOperationHostActionGroupKind.Lifecycle,
        GroupTitle: "Lifecycle",
        GroupOrder: 0,
        TargetKind: UnitOperationHostActionTargetKind.Unit,
        IsBlocking: true,
        ResolveTargetNames: static (_, issue) => [issue.TargetName]);

    public static UnitOperationHostActionDefinition RequiredParameterMissing { get; } = new(
        IssueKind: UnitOperationHostConfigurationIssueKind.RequiredParameterMissing,
        GroupKind: UnitOperationHostActionGroupKind.Parameters,
        GroupTitle: "Parameters",
        GroupOrder: 1,
        TargetKind: UnitOperationHostActionTargetKind.Parameter,
        IsBlocking: true,
        ResolveTargetNames: static (context, issue) => [ResolveName(context.ParameterNames, issue.TargetName)]);

    public static UnitOperationHostActionDefinition CompanionParameterMismatch { get; } = new(
        IssueKind: UnitOperationHostConfigurationIssueKind.CompanionParameterMismatch,
        GroupKind: UnitOperationHostActionGroupKind.Parameters,
        GroupTitle: "Parameters",
        GroupOrder: 1,
        TargetKind: UnitOperationHostActionTargetKind.Parameter,
        IsBlocking: true,
        ResolveTargetNames: static (context, issue) => ResolveCompanionNames(context.ParameterNames, issue.TargetName));

    public static UnitOperationHostActionDefinition RequiredPortDisconnected { get; } = new(
        IssueKind: UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected,
        GroupKind: UnitOperationHostActionGroupKind.Ports,
        GroupTitle: "Ports",
        GroupOrder: 2,
        TargetKind: UnitOperationHostActionTargetKind.Port,
        IsBlocking: true,
        ResolveTargetNames: static (context, issue) => [ResolveName(context.PortNames, issue.TargetName)]);

    public static UnitOperationHostActionDefinition Terminated { get; } = new(
        IssueKind: UnitOperationHostConfigurationIssueKind.Terminated,
        GroupKind: UnitOperationHostActionGroupKind.Terminal,
        GroupTitle: "Terminal State",
        GroupOrder: 3,
        TargetKind: UnitOperationHostActionTargetKind.Unit,
        IsBlocking: true,
        ResolveTargetNames: static (_, issue) => [issue.TargetName]);

    private static readonly IReadOnlyDictionary<UnitOperationHostConfigurationIssueKind, UnitOperationHostActionDefinition> DefinitionsByIssueKindValue =
        new Dictionary<UnitOperationHostConfigurationIssueKind, UnitOperationHostActionDefinition>
        {
            [UnitOperationHostConfigurationIssueKind.InitializeRequired] = InitializeRequired,
            [UnitOperationHostConfigurationIssueKind.RequiredParameterMissing] = RequiredParameterMissing,
            [UnitOperationHostConfigurationIssueKind.CompanionParameterMismatch] = CompanionParameterMismatch,
            [UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected] = RequiredPortDisconnected,
            [UnitOperationHostConfigurationIssueKind.Terminated] = Terminated,
        };

    public static IReadOnlyList<UnitOperationHostActionDefinition> OrderedDefinitions { get; } =
    [
        InitializeRequired,
        RequiredParameterMissing,
        CompanionParameterMismatch,
        RequiredPortDisconnected,
        Terminated,
    ];

    public static bool TryGetByIssueKind(
        UnitOperationHostConfigurationIssueKind issueKind,
        out UnitOperationHostActionDefinition definition)
    {
        return DefinitionsByIssueKindValue.TryGetValue(issueKind, out definition!);
    }

    public static UnitOperationHostActionDefinition GetByIssueKind(UnitOperationHostConfigurationIssueKind issueKind)
    {
        if (TryGetByIssueKind(issueKind, out var definition))
        {
            return definition;
        }

        throw new ArgumentException($"Unknown unit operation host action definition for issue kind `{issueKind}`.", nameof(issueKind));
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
}

public sealed record UnitOperationHostActionDefinition(
    UnitOperationHostConfigurationIssueKind IssueKind,
    UnitOperationHostActionGroupKind GroupKind,
    string GroupTitle,
    int GroupOrder,
    UnitOperationHostActionTargetKind TargetKind,
    bool IsBlocking,
    Func<UnitOperationHostActionResolutionContext, UnitOperationHostConfigurationIssue, IReadOnlyList<string>> ResolveTargetNames);

public sealed record UnitOperationHostActionResolutionContext(
    IReadOnlyDictionary<string, string> ParameterNames,
    IReadOnlyDictionary<string, string> PortNames);
