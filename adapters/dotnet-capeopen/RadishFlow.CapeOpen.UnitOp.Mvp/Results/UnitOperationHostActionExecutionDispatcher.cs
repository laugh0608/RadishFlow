using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostActionExecutionDispatcher
{
    public static UnitOperationHostActionExecutionOutcome ApplyAction(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostActionExecutionRequest request)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);
        ArgumentNullException.ThrowIfNull(request);

        var binding = UnitOperationHostActionMutationBridge.Describe(request.Action);

        return binding.Kind switch
        {
            UnitOperationHostActionMutationBindingKind.LifecycleOperation => CreateLifecycleOutcome(request.Action, binding),
            UnitOperationHostActionMutationBindingKind.ParameterValues => ApplyMutationCommandBatch(
                unitOperation,
                request.Action,
                binding,
                UnitOperationHostActionMutationBridge.CreateParameterCommandBatch(
                    request.Action,
                    request.RequireParameterValues())),
            UnitOperationHostActionMutationBindingKind.PortConnection => ApplyMutationCommandBatch(
                unitOperation,
                request.Action,
                binding,
                UnitOperationHostActionMutationBridge.CreatePortConnectionCommandBatch(
                    request.Action,
                    request.RequirePortObject())),
            UnitOperationHostActionMutationBindingKind.Unsupported => CreateUnsupportedOutcome(request.Action, binding),
            _ => throw new ArgumentOutOfRangeException(nameof(binding), binding.Kind, "Unknown host action mutation binding kind."),
        };
    }

    public static UnitOperationHostActionExecutionBatchResult ApplyActionBatch(
        RadishFlowCapeOpenUnitOperation unitOperation,
        IReadOnlyList<UnitOperationHostActionExecutionRequest> requests)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);
        ArgumentNullException.ThrowIfNull(requests);

        var outcomes = new List<UnitOperationHostActionExecutionOutcome>(requests.Count);
        var appliedMutationCount = 0;
        var hasLifecycleOperations = false;
        var hasUnsupportedActions = false;
        var invalidatedValidation = false;
        var invalidatedCalculationReport = false;

        foreach (var request in requests)
        {
            var outcome = ApplyAction(unitOperation, request);
            outcomes.Add(outcome);
            appliedMutationCount += outcome.AppliedMutationCount;
            hasLifecycleOperations |= outcome.Disposition == UnitOperationHostActionExecutionDisposition.LifecycleOperationRequired;
            hasUnsupportedActions |= outcome.Disposition == UnitOperationHostActionExecutionDisposition.Unsupported;
            invalidatedValidation |= outcome.InvalidatedValidation;
            invalidatedCalculationReport |= outcome.InvalidatedCalculationReport;
        }

        return new UnitOperationHostActionExecutionBatchResult(
            AppliedActionCount: outcomes.Count,
            AppliedMutationCount: appliedMutationCount,
            Outcomes: outcomes,
            HasLifecycleOperations: hasLifecycleOperations,
            HasUnsupportedActions: hasUnsupportedActions,
            InvalidatedValidation: invalidatedValidation,
            InvalidatedCalculationReport: invalidatedCalculationReport);
    }

    private static UnitOperationHostActionExecutionOutcome ApplyMutationCommandBatch(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostActionItem action,
        UnitOperationHostActionMutationBinding binding,
        UnitOperationHostActionMutationCommandBatch commandBatch)
    {
        var mutationBatch = UnitOperationHostObjectMutationDispatcher.DispatchBatch(unitOperation, commandBatch.Commands);

        return new UnitOperationHostActionExecutionOutcome(
            Action: action,
            BindingKind: binding.Kind,
            Disposition: UnitOperationHostActionExecutionDisposition.MutationApplied,
            ExecutedCommands: commandBatch.Commands,
            MutationOutcomes: mutationBatch.Outcomes,
            AppliedMutationCount: mutationBatch.AppliedCount,
            LifecycleOperationName: null,
            InvalidatedValidation: mutationBatch.InvalidatedValidation,
            InvalidatedCalculationReport: mutationBatch.InvalidatedCalculationReport,
            Summary: $"Applied {mutationBatch.AppliedCount} mutation command(s) for host action `{action.IssueKind}`.");
    }

    private static UnitOperationHostActionExecutionOutcome CreateLifecycleOutcome(
        UnitOperationHostActionItem action,
        UnitOperationHostActionMutationBinding binding)
    {
        return new UnitOperationHostActionExecutionOutcome(
            Action: action,
            BindingKind: binding.Kind,
            Disposition: UnitOperationHostActionExecutionDisposition.LifecycleOperationRequired,
            ExecutedCommands: [],
            MutationOutcomes: [],
            AppliedMutationCount: 0,
            LifecycleOperationName: action.CanonicalOperationName,
            InvalidatedValidation: false,
            InvalidatedCalculationReport: false,
            Summary: $"Host action `{action.IssueKind}` requires lifecycle operation `{action.CanonicalOperationName}`.");
    }

    private static UnitOperationHostActionExecutionOutcome CreateUnsupportedOutcome(
        UnitOperationHostActionItem action,
        UnitOperationHostActionMutationBinding binding)
    {
        return new UnitOperationHostActionExecutionOutcome(
            Action: action,
            BindingKind: binding.Kind,
            Disposition: UnitOperationHostActionExecutionDisposition.Unsupported,
            ExecutedCommands: [],
            MutationOutcomes: [],
            AppliedMutationCount: 0,
            LifecycleOperationName: null,
            InvalidatedValidation: false,
            InvalidatedCalculationReport: false,
            Summary: $"Host action `{action.IssueKind}` is not translatable into object mutations.");
    }
}

public sealed record UnitOperationHostActionExecutionRequest(
    UnitOperationHostActionItem Action,
    IReadOnlyDictionary<string, string?>? ParameterValues,
    object? PortObject)
{
    public static UnitOperationHostActionExecutionRequest ForAction(UnitOperationHostActionItem action)
    {
        ArgumentNullException.ThrowIfNull(action);
        return new UnitOperationHostActionExecutionRequest(action, null, null);
    }

    public static UnitOperationHostActionExecutionRequest ForParameterValues(
        UnitOperationHostActionItem action,
        IReadOnlyDictionary<string, string?> parameterValues)
    {
        ArgumentNullException.ThrowIfNull(action);
        ArgumentNullException.ThrowIfNull(parameterValues);
        return new UnitOperationHostActionExecutionRequest(action, parameterValues, null);
    }

    public static UnitOperationHostActionExecutionRequest ForPortConnection(
        UnitOperationHostActionItem action,
        object objectToConnect)
    {
        ArgumentNullException.ThrowIfNull(action);
        ArgumentNullException.ThrowIfNull(objectToConnect);
        return new UnitOperationHostActionExecutionRequest(action, null, objectToConnect);
    }

    public IReadOnlyDictionary<string, string?> RequireParameterValues()
    {
        if (ParameterValues is not null)
        {
            return ParameterValues;
        }

        throw new InvalidOperationException(
            $"Host action `{Action.IssueKind}` requires parameter values before it can be applied.");
    }

    public object RequirePortObject()
    {
        if (PortObject is not null)
        {
            return PortObject;
        }

        throw new InvalidOperationException(
            $"Host action `{Action.IssueKind}` requires a port connection object before it can be applied.");
    }
}

public sealed record UnitOperationHostActionExecutionOutcome(
    UnitOperationHostActionItem Action,
    UnitOperationHostActionMutationBindingKind BindingKind,
    UnitOperationHostActionExecutionDisposition Disposition,
    IReadOnlyList<UnitOperationHostObjectMutationCommand> ExecutedCommands,
    IReadOnlyList<UnitOperationHostObjectMutationOutcome> MutationOutcomes,
    int AppliedMutationCount,
    string? LifecycleOperationName,
    bool InvalidatedValidation,
    bool InvalidatedCalculationReport,
    string Summary)
{
    public bool AppliedMutations => AppliedMutationCount > 0;
}

public sealed record UnitOperationHostActionExecutionBatchResult(
    int AppliedActionCount,
    int AppliedMutationCount,
    IReadOnlyList<UnitOperationHostActionExecutionOutcome> Outcomes,
    bool HasLifecycleOperations,
    bool HasUnsupportedActions,
    bool InvalidatedValidation,
    bool InvalidatedCalculationReport);

public enum UnitOperationHostActionExecutionDisposition
{
    MutationApplied,
    LifecycleOperationRequired,
    Unsupported,
}
