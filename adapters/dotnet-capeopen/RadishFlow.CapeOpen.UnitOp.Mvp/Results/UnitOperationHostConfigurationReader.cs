using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostConfigurationReader
{
    public static UnitOperationHostConfigurationSnapshot Read(
        RadishFlowCapeOpenUnitOperation unitOperation)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);

        var lifecycleState = unitOperation.HostLifecycleState;
        if (lifecycleState == UnitOperationLifecycleState.Disposed)
        {
            throw new ObjectDisposedException(unitOperation.GetType().FullName);
        }

        if (lifecycleState == UnitOperationLifecycleState.Terminated)
        {
            var terminatedIssue = new UnitOperationHostConfigurationIssue(
                Kind: UnitOperationHostConfigurationIssueKind.Terminated,
                TargetName: unitOperation.ComponentName,
                Message: "Terminate has already been called for this unit instance.",
                OperationName: null);
            return new UnitOperationHostConfigurationSnapshot(
                State: UnitOperationHostConfigurationState.Terminated,
                Headline: "Unit operation has been terminated.",
                ParameterEntries: [],
                PortEntries: [],
                BlockingIssues: [terminatedIssue],
                NextOperations: []);
        }

        var parameterEntries = UnitOperationParameterCatalog.OrderedDefinitions
            .Select(definition => CreateParameterEntry(unitOperation, definition))
            .ToArray();
        var portEntries = UnitOperationPortCatalog.OrderedDefinitions
            .Select(definition => CreatePortEntry(unitOperation, definition))
            .ToArray();

        var issues = new List<UnitOperationHostConfigurationIssue>();
        if (lifecycleState == UnitOperationLifecycleState.Constructed)
        {
            issues.Add(new UnitOperationHostConfigurationIssue(
                Kind: UnitOperationHostConfigurationIssueKind.InitializeRequired,
                TargetName: unitOperation.ComponentName,
                Message: "Initialize must be called before Calculate.",
                OperationName: nameof(RadishFlowCapeOpenUnitOperation.Initialize)));
        }

        AppendRequiredParameterIssues(parameterEntries, issues);
        AppendCompanionIssues(parameterEntries, issues);
        AppendRequiredPortIssues(portEntries, issues);

        var state = issues.Count == 0
            ? UnitOperationHostConfigurationState.Ready
            : lifecycleState == UnitOperationLifecycleState.Constructed
                ? UnitOperationHostConfigurationState.Constructed
                : UnitOperationHostConfigurationState.Incomplete;
        var headline = issues.Count == 0
            ? "Unit operation is ready to calculate."
            : issues[0].Message;
        var nextOperations = issues
            .Select(static issue => issue.OperationName)
            .Where(static operationName => !string.IsNullOrWhiteSpace(operationName))
            .Distinct(StringComparer.Ordinal)
            .Cast<string>()
            .ToArray();

        return new UnitOperationHostConfigurationSnapshot(
            State: state,
            Headline: headline,
            ParameterEntries: parameterEntries,
            PortEntries: portEntries,
            BlockingIssues: issues.ToArray(),
            NextOperations: nextOperations);
    }

    private static UnitOperationHostConfigurationParameterEntry CreateParameterEntry(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationParameterDefinition definition)
    {
        var parameter = unitOperation.Parameters.GetByName(definition.Name);
        return new UnitOperationHostConfigurationParameterEntry(
            Name: definition.Name,
            Description: definition.Description,
            IsRequired: definition.IsRequired,
            IsConfigured: parameter.IsConfigured,
            ValueKind: definition.ValueKind,
            RequiredCompanionParameterName: definition.RequiredCompanionParameterName,
            ConfigurationOperationName: definition.ConfigurationOperationName);
    }

    private static UnitOperationHostConfigurationPortEntry CreatePortEntry(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationPortDefinition definition)
    {
        var port = unitOperation.Ports.GetByName(definition.Name);
        return new UnitOperationHostConfigurationPortEntry(
            Name: definition.Name,
            Description: definition.Description,
            IsRequired: definition.IsRequired,
            IsConnected: port.IsConnected,
            Direction: definition.Direction,
            PortType: definition.PortType,
            ConnectionOperationName: definition.ConnectionOperationName);
    }

    private static void AppendRequiredParameterIssues(
        IReadOnlyList<UnitOperationHostConfigurationParameterEntry> parameterEntries,
        ICollection<UnitOperationHostConfigurationIssue> issues)
    {
        foreach (var parameter in parameterEntries.Where(static entry => entry.IsRequired && !entry.IsConfigured))
        {
            issues.Add(new UnitOperationHostConfigurationIssue(
                Kind: UnitOperationHostConfigurationIssueKind.RequiredParameterMissing,
                TargetName: parameter.Name,
                Message: $"Required parameter `{parameter.Name}` is not configured.",
                OperationName: parameter.ConfigurationOperationName));
        }
    }

    private static void AppendCompanionIssues(
        IReadOnlyList<UnitOperationHostConfigurationParameterEntry> parameterEntries,
        ICollection<UnitOperationHostConfigurationIssue> issues)
    {
        var entriesByName = parameterEntries.ToDictionary(static entry => entry.Name, StringComparer.OrdinalIgnoreCase);
        var evaluatedPairs = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        foreach (var parameter in parameterEntries)
        {
            if (parameter.RequiredCompanionParameterName is not { Length: > 0 } companionName)
            {
                continue;
            }

            var companion = entriesByName[companionName];
            var pairKey = string.Compare(parameter.Name, companion.Name, StringComparison.OrdinalIgnoreCase) <= 0
                ? $"{parameter.Name}|{companion.Name}"
                : $"{companion.Name}|{parameter.Name}";
            if (!evaluatedPairs.Add(pairKey))
            {
                continue;
            }

            if (parameter.IsConfigured == companion.IsConfigured)
            {
                continue;
            }

            issues.Add(new UnitOperationHostConfigurationIssue(
                Kind: UnitOperationHostConfigurationIssueKind.CompanionParameterMismatch,
                TargetName: pairKey,
                Message: $"Optional parameters `{parameter.Name}` and `{companion.Name}` must be configured together.",
                OperationName: parameter.ConfigurationOperationName));
        }
    }

    private static void AppendRequiredPortIssues(
        IReadOnlyList<UnitOperationHostConfigurationPortEntry> portEntries,
        ICollection<UnitOperationHostConfigurationIssue> issues)
    {
        foreach (var port in portEntries.Where(static entry => entry.IsRequired && !entry.IsConnected))
        {
            issues.Add(new UnitOperationHostConfigurationIssue(
                Kind: UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected,
                TargetName: port.Name,
                Message: $"Required port `{port.Name}` is not connected.",
                OperationName: port.ConnectionOperationName));
        }
    }
}

public sealed record UnitOperationHostConfigurationSnapshot(
    UnitOperationHostConfigurationState State,
    string Headline,
    IReadOnlyList<UnitOperationHostConfigurationParameterEntry> ParameterEntries,
    IReadOnlyList<UnitOperationHostConfigurationPortEntry> PortEntries,
    IReadOnlyList<UnitOperationHostConfigurationIssue> BlockingIssues,
    IReadOnlyList<string> NextOperations)
{
    public bool IsReadyForCalculate => State == UnitOperationHostConfigurationState.Ready;

    public int BlockingIssueCount => BlockingIssues.Count;

    public UnitOperationHostConfigurationParameterEntry GetParameter(string name)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(name);

        foreach (var entry in ParameterEntries)
        {
            if (string.Equals(entry.Name, name, StringComparison.OrdinalIgnoreCase))
            {
                return entry;
            }
        }

        throw new ArgumentException($"Unknown unit operation host configuration parameter `{name}`.", nameof(name));
    }

    public UnitOperationHostConfigurationPortEntry GetPort(string name)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(name);

        foreach (var entry in PortEntries)
        {
            if (string.Equals(entry.Name, name, StringComparison.OrdinalIgnoreCase))
            {
                return entry;
            }
        }

        throw new ArgumentException($"Unknown unit operation host configuration port `{name}`.", nameof(name));
    }

    public bool ContainsNextOperation(string operationName)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(operationName);
        return NextOperations.Any(nextOperation => string.Equals(nextOperation, operationName, StringComparison.Ordinal));
    }
}

public sealed record UnitOperationHostConfigurationParameterEntry(
    string Name,
    string Description,
    bool IsRequired,
    bool IsConfigured,
    UnitOperationParameterValueKind ValueKind,
    string? RequiredCompanionParameterName,
    string ConfigurationOperationName);

public sealed record UnitOperationHostConfigurationPortEntry(
    string Name,
    string Description,
    bool IsRequired,
    bool IsConnected,
    CapePortDirection Direction,
    CapePortType PortType,
    string ConnectionOperationName);

public sealed record UnitOperationHostConfigurationIssue(
    UnitOperationHostConfigurationIssueKind Kind,
    string TargetName,
    string Message,
    string? OperationName);

public enum UnitOperationHostConfigurationState
{
    Constructed,
    Incomplete,
    Ready,
    Terminated,
}

public enum UnitOperationHostConfigurationIssueKind
{
    InitializeRequired,
    RequiredParameterMissing,
    CompanionParameterMismatch,
    RequiredPortDisconnected,
    Terminated,
}
