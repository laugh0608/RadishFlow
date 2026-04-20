namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostActionMutationBridge
{
    public static IReadOnlyList<UnitOperationHostActionMutationBinding> Describe(
        UnitOperationHostActionPlan actionPlan)
    {
        ArgumentNullException.ThrowIfNull(actionPlan);
        return actionPlan.Actions
            .Select(Describe)
            .ToArray();
    }

    public static UnitOperationHostActionMutationBinding Describe(
        UnitOperationHostActionItem action)
    {
        ArgumentNullException.ThrowIfNull(action);

        return action.IssueKind switch
        {
            UnitOperationHostConfigurationIssueKind.InitializeRequired => CreateBinding(
                action,
                UnitOperationHostActionMutationBindingKind.LifecycleOperation,
                []),
            UnitOperationHostConfigurationIssueKind.RequiredParameterMissing => CreateBinding(
                action,
                UnitOperationHostActionMutationBindingKind.ParameterValues,
                [UnitOperationHostObjectMutationKind.SetParameterValue]),
            UnitOperationHostConfigurationIssueKind.CompanionParameterMismatch => CreateBinding(
                action,
                UnitOperationHostActionMutationBindingKind.ParameterValues,
                Enumerable.Repeat(UnitOperationHostObjectMutationKind.SetParameterValue, action.Target.Names.Count).ToArray()),
            UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected => CreateBinding(
                action,
                UnitOperationHostActionMutationBindingKind.PortConnection,
                [UnitOperationHostObjectMutationKind.ConnectPort]),
            UnitOperationHostConfigurationIssueKind.Terminated => CreateBinding(
                action,
                UnitOperationHostActionMutationBindingKind.Unsupported,
                []),
            _ => throw new ArgumentOutOfRangeException(nameof(action), action.IssueKind, "Unknown host action issue kind."),
        };
    }

    public static UnitOperationHostActionMutationCommandBatch CreateParameterCommandBatch(
        UnitOperationHostActionItem action,
        IReadOnlyDictionary<string, string?> parameterValues)
    {
        ArgumentNullException.ThrowIfNull(action);
        ArgumentNullException.ThrowIfNull(parameterValues);

        var binding = Describe(action);
        if (binding.Kind != UnitOperationHostActionMutationBindingKind.ParameterValues)
        {
            throw new InvalidOperationException(
                $"Host action `{action.IssueKind}` does not accept parameter-value mutation translation.");
        }

        var commands = action.Target.Names
            .Select(targetName =>
            {
                if (!parameterValues.TryGetValue(targetName, out var value))
                {
                    throw new ArgumentException(
                        $"Missing parameter value for host action target `{targetName}`.",
                        nameof(parameterValues));
                }

                return UnitOperationHostObjectMutationCommand.SetParameterValue(targetName, value);
            })
            .ToArray();

        return new UnitOperationHostActionMutationCommandBatch(action, commands);
    }

    public static UnitOperationHostActionMutationCommandBatch CreatePortConnectionCommandBatch(
        UnitOperationHostActionItem action,
        object objectToConnect)
    {
        ArgumentNullException.ThrowIfNull(action);
        ArgumentNullException.ThrowIfNull(objectToConnect);

        var binding = Describe(action);
        if (binding.Kind != UnitOperationHostActionMutationBindingKind.PortConnection)
        {
            throw new InvalidOperationException(
                $"Host action `{action.IssueKind}` does not accept port-connection mutation translation.");
        }

        return new UnitOperationHostActionMutationCommandBatch(
            action,
            [UnitOperationHostObjectMutationCommand.ConnectPort(action.Target.PrimaryName, objectToConnect)]);
    }

    private static UnitOperationHostActionMutationBinding CreateBinding(
        UnitOperationHostActionItem action,
        UnitOperationHostActionMutationBindingKind kind,
        IReadOnlyList<UnitOperationHostObjectMutationKind> mutationKinds)
    {
        return new UnitOperationHostActionMutationBinding(
            Action: action,
            Kind: kind,
            MutationKinds: mutationKinds,
            CommandCount: mutationKinds.Count);
    }
}

public sealed record UnitOperationHostActionMutationBinding(
    UnitOperationHostActionItem Action,
    UnitOperationHostActionMutationBindingKind Kind,
    IReadOnlyList<UnitOperationHostObjectMutationKind> MutationKinds,
    int CommandCount)
{
    public bool CanCreateMutationCommands =>
        Kind is UnitOperationHostActionMutationBindingKind.ParameterValues or
            UnitOperationHostActionMutationBindingKind.PortConnection;
}

public sealed record UnitOperationHostActionMutationCommandBatch(
    UnitOperationHostActionItem Action,
    IReadOnlyList<UnitOperationHostObjectMutationCommand> Commands)
{
    public int CommandCount => Commands.Count;
}

public enum UnitOperationHostActionMutationBindingKind
{
    LifecycleOperation,
    ParameterValues,
    PortConnection,
    Unsupported,
}
