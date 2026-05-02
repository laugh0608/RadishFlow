namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostActionExecutionRequestPlanner
{
    public static UnitOperationHostActionExecutionRequestPlan Plan(
        UnitOperationHostActionPlan actionPlan)
    {
        return Plan(actionPlan, UnitOperationHostActionExecutionInputSet.Empty);
    }

    public static UnitOperationHostActionExecutionRequestPlan Plan(
        UnitOperationHostActionPlan actionPlan,
        UnitOperationHostActionExecutionInputSet inputSet)
    {
        ArgumentNullException.ThrowIfNull(actionPlan);
        ArgumentNullException.ThrowIfNull(inputSet);

        return new UnitOperationHostActionExecutionRequestPlan(
            actionPlan.Actions
                .Select(action => PlanAction(action, inputSet))
                .ToArray());
    }

    private static UnitOperationHostActionExecutionRequestPlanEntry PlanAction(
        UnitOperationHostActionItem action,
        UnitOperationHostActionExecutionInputSet inputSet)
    {
        var binding = UnitOperationHostActionMutationBridge.Describe(action);
        return binding.Kind switch
        {
            UnitOperationHostActionMutationBindingKind.LifecycleOperation => CreateLifecycleEntry(action, binding),
            UnitOperationHostActionMutationBindingKind.ParameterValues => CreateParameterEntry(action, binding, inputSet),
            UnitOperationHostActionMutationBindingKind.PortConnection => CreatePortEntry(action, binding, inputSet),
            UnitOperationHostActionMutationBindingKind.Unsupported => CreateUnsupportedEntry(action, binding),
            _ => throw new ArgumentOutOfRangeException(nameof(binding), binding.Kind, "Unknown host action mutation binding kind."),
        };
    }

    private static UnitOperationHostActionExecutionRequestPlanEntry CreateLifecycleEntry(
        UnitOperationHostActionItem action,
        UnitOperationHostActionMutationBinding binding)
    {
        return new UnitOperationHostActionExecutionRequestPlanEntry(
            Action: action,
            BindingKind: binding.Kind,
            Disposition: UnitOperationHostActionExecutionRequestPlanningDisposition.LifecycleOperationRequired,
            RequiredInputNames: [],
            MissingInputNames: [],
            Request: UnitOperationHostActionExecutionRequest.ForAction(action));
    }

    private static UnitOperationHostActionExecutionRequestPlanEntry CreateParameterEntry(
        UnitOperationHostActionItem action,
        UnitOperationHostActionMutationBinding binding,
        UnitOperationHostActionExecutionInputSet inputSet)
    {
        var parameterValues = new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase);
        var missingInputNames = new List<string>();

        foreach (var targetName in action.Target.Names)
        {
            if (inputSet.TryGetParameterValue(targetName, out var value))
            {
                parameterValues[targetName] = value;
                continue;
            }

            missingInputNames.Add(targetName);
        }

        if (missingInputNames.Count > 0)
        {
            return CreateMissingInputsEntry(action, binding, action.Target.Names, missingInputNames);
        }

        return new UnitOperationHostActionExecutionRequestPlanEntry(
            Action: action,
            BindingKind: binding.Kind,
            Disposition: UnitOperationHostActionExecutionRequestPlanningDisposition.RequestReady,
            RequiredInputNames: action.Target.Names,
            MissingInputNames: [],
            Request: UnitOperationHostActionExecutionRequest.ForParameterValues(action, parameterValues));
    }

    private static UnitOperationHostActionExecutionRequestPlanEntry CreatePortEntry(
        UnitOperationHostActionItem action,
        UnitOperationHostActionMutationBinding binding,
        UnitOperationHostActionExecutionInputSet inputSet)
    {
        var portName = action.Target.PrimaryName;
        if (!inputSet.TryGetPortObject(portName, out var portObject))
        {
            return CreateMissingInputsEntry(action, binding, [portName], [portName]);
        }

        return new UnitOperationHostActionExecutionRequestPlanEntry(
            Action: action,
            BindingKind: binding.Kind,
            Disposition: UnitOperationHostActionExecutionRequestPlanningDisposition.RequestReady,
            RequiredInputNames: [portName],
            MissingInputNames: [],
            Request: UnitOperationHostActionExecutionRequest.ForPortConnection(action, portObject));
    }

    private static UnitOperationHostActionExecutionRequestPlanEntry CreateMissingInputsEntry(
        UnitOperationHostActionItem action,
        UnitOperationHostActionMutationBinding binding,
        IReadOnlyList<string> requiredInputNames,
        IReadOnlyList<string> missingInputNames)
    {
        return new UnitOperationHostActionExecutionRequestPlanEntry(
            Action: action,
            BindingKind: binding.Kind,
            Disposition: UnitOperationHostActionExecutionRequestPlanningDisposition.MissingInputs,
            RequiredInputNames: requiredInputNames,
            MissingInputNames: missingInputNames,
            Request: null);
    }

    private static UnitOperationHostActionExecutionRequestPlanEntry CreateUnsupportedEntry(
        UnitOperationHostActionItem action,
        UnitOperationHostActionMutationBinding binding)
    {
        return new UnitOperationHostActionExecutionRequestPlanEntry(
            Action: action,
            BindingKind: binding.Kind,
            Disposition: UnitOperationHostActionExecutionRequestPlanningDisposition.Unsupported,
            RequiredInputNames: [],
            MissingInputNames: [],
            Request: null);
    }
}

public sealed class UnitOperationHostActionExecutionInputSet
{
    public static UnitOperationHostActionExecutionInputSet Empty { get; } = new();

    public UnitOperationHostActionExecutionInputSet(
        IReadOnlyDictionary<string, string?>? parameterValues = null,
        IReadOnlyDictionary<string, object>? portObjects = null)
    {
        ParameterValues = CopyParameterValues(parameterValues);
        PortObjects = CopyPortObjects(portObjects);
    }

    public IReadOnlyDictionary<string, string?> ParameterValues { get; }

    public IReadOnlyDictionary<string, object> PortObjects { get; }

    public bool TryGetParameterValue(
        string parameterName,
        out string? value)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(parameterName);
        return ParameterValues.TryGetValue(parameterName, out value);
    }

    public bool TryGetPortObject(
        string portName,
        out object portObject)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(portName);
        return PortObjects.TryGetValue(portName, out portObject!);
    }

    private static IReadOnlyDictionary<string, string?> CopyParameterValues(
        IReadOnlyDictionary<string, string?>? parameterValues)
    {
        var copiedValues = new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase);
        if (parameterValues is null)
        {
            return copiedValues;
        }

        foreach (var (name, value) in parameterValues)
        {
            ArgumentException.ThrowIfNullOrWhiteSpace(name);
            copiedValues[name] = value;
        }

        return copiedValues;
    }

    private static IReadOnlyDictionary<string, object> CopyPortObjects(
        IReadOnlyDictionary<string, object>? portObjects)
    {
        var copiedObjects = new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase);
        if (portObjects is null)
        {
            return copiedObjects;
        }

        foreach (var (name, portObject) in portObjects)
        {
            ArgumentException.ThrowIfNullOrWhiteSpace(name);
            ArgumentNullException.ThrowIfNull(portObject);
            copiedObjects[name] = portObject;
        }

        return copiedObjects;
    }
}

public sealed record UnitOperationHostActionExecutionRequestPlan(
    IReadOnlyList<UnitOperationHostActionExecutionRequestPlanEntry> Entries)
{
    public IReadOnlyList<UnitOperationHostActionExecutionRequest> Requests { get; } = Entries
        .Where(static entry => entry.Request is not null)
        .Select(static entry => entry.Request!)
        .ToArray();

    public int EntryCount => Entries.Count;

    public int RequestCount => Requests.Count;

    public int MissingInputCount => Entries.Sum(static entry => entry.MissingInputNames.Count);

    public bool HasMissingInputs => Entries.Any(static entry =>
        entry.Disposition == UnitOperationHostActionExecutionRequestPlanningDisposition.MissingInputs);

    public bool HasLifecycleOperations => Entries.Any(static entry =>
        entry.Disposition == UnitOperationHostActionExecutionRequestPlanningDisposition.LifecycleOperationRequired);

    public bool HasUnsupportedActions => Entries.Any(static entry =>
        entry.Disposition == UnitOperationHostActionExecutionRequestPlanningDisposition.Unsupported);
}

public sealed record UnitOperationHostActionExecutionRequestPlanEntry(
    UnitOperationHostActionItem Action,
    UnitOperationHostActionMutationBindingKind BindingKind,
    UnitOperationHostActionExecutionRequestPlanningDisposition Disposition,
    IReadOnlyList<string> RequiredInputNames,
    IReadOnlyList<string> MissingInputNames,
    UnitOperationHostActionExecutionRequest? Request)
{
    public bool HasRequest => Request is not null;
}

public enum UnitOperationHostActionExecutionRequestPlanningDisposition
{
    RequestReady,
    MissingInputs,
    LifecycleOperationRequired,
    Unsupported,
}
