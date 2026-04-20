using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostActionExecutionOrchestrator
{
    public static UnitOperationHostActionExecutionOrchestrationResult ExecutePlannedActions(
        RadishFlowCapeOpenUnitOperation unitOperation)
    {
        return ExecutePlannedActions(unitOperation, UnitOperationHostActionExecutionInputSet.Empty);
    }

    public static UnitOperationHostActionExecutionOrchestrationResult ExecutePlannedActions(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostActionExecutionInputSet inputSet)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);
        ArgumentNullException.ThrowIfNull(inputSet);

        var actionPlan = UnitOperationHostActionPlanReader.Read(unitOperation);
        return ExecutePlannedActions(unitOperation, actionPlan, inputSet);
    }

    public static UnitOperationHostActionExecutionOrchestrationResult ExecutePlannedActions(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostActionPlan actionPlan,
        UnitOperationHostActionExecutionInputSet inputSet)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);
        ArgumentNullException.ThrowIfNull(actionPlan);
        ArgumentNullException.ThrowIfNull(inputSet);

        var requestPlan = UnitOperationHostActionExecutionRequestPlanner.Plan(actionPlan, inputSet);
        var execution = UnitOperationHostActionExecutionDispatcher.ApplyActionBatch(unitOperation, requestPlan.Requests);
        return CreateResult(unitOperation, actionPlan, requestPlan, execution);
    }

    private static UnitOperationHostActionExecutionOrchestrationResult CreateResult(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostActionPlan initialActionPlan,
        UnitOperationHostActionExecutionRequestPlan requestPlan,
        UnitOperationHostActionExecutionBatchResult execution)
    {
        var configuration = UnitOperationHostConfigurationReader.Read(unitOperation);
        var refreshedActionPlan = UnitOperationHostActionPlanReader.Read(configuration);
        var session = UnitOperationHostSessionReader.Read(unitOperation);

        return new UnitOperationHostActionExecutionOrchestrationResult(
            InitialActionPlan: initialActionPlan,
            RequestPlan: requestPlan,
            Execution: execution,
            Configuration: configuration,
            ActionPlan: refreshedActionPlan,
            Session: session);
    }
}

public sealed record UnitOperationHostActionExecutionOrchestrationResult(
    UnitOperationHostActionPlan InitialActionPlan,
    UnitOperationHostActionExecutionRequestPlan RequestPlan,
    UnitOperationHostActionExecutionBatchResult Execution,
    UnitOperationHostConfigurationSnapshot Configuration,
    UnitOperationHostActionPlan ActionPlan,
    UnitOperationHostSessionSnapshot Session)
{
    public UnitOperationHostActionExecutionFollowUp FollowUp { get; } =
        CreateFollowUp(RequestPlan, Execution, Configuration, ActionPlan, Session);

    public int PlannedActionCount => RequestPlan.EntryCount;

    public int ReadyRequestCount => RequestPlan.RequestCount;

    public int MissingInputCount => RequestPlan.MissingInputCount;

    public bool HasMissingInputs => RequestPlan.HasMissingInputs;

    public bool HasLifecycleOperations => RequestPlan.HasLifecycleOperations || Execution.HasLifecycleOperations;

    public bool HasUnsupportedActions => RequestPlan.HasUnsupportedActions || Execution.HasUnsupportedActions;

    public bool AppliedMutations => Execution.AppliedMutationCount > 0;

    public bool RequiresValidationRefresh => Execution.InvalidatedValidation;

    public bool RequiresCalculationRefresh =>
        Execution.InvalidatedCalculationReport || Session.Summary.RequiresCalculateRefresh;

    private static UnitOperationHostActionExecutionFollowUp CreateFollowUp(
        UnitOperationHostActionExecutionRequestPlan requestPlan,
        UnitOperationHostActionExecutionBatchResult execution,
        UnitOperationHostConfigurationSnapshot configuration,
        UnitOperationHostActionPlan actionPlan,
        UnitOperationHostSessionSnapshot session)
    {
        if (configuration.State == UnitOperationHostConfigurationState.Terminated ||
            session.State == UnitOperationHostSessionState.Terminated)
        {
            return new UnitOperationHostActionExecutionFollowUp(
                Kind: UnitOperationHostActionExecutionFollowUpKind.Terminated,
                Summary: "Unit operation has been terminated.",
                MissingInputNames: [],
                RecommendedOperations: [],
                CanValidate: false,
                CanCalculate: false);
        }

        if (requestPlan.HasLifecycleOperations)
        {
            var lifecycleOperations = requestPlan.Entries
                .Where(static entry => entry.Disposition == UnitOperationHostActionExecutionRequestPlanningDisposition.LifecycleOperationRequired)
                .Select(static entry => entry.Action.CanonicalOperationName)
                .Where(static operationName => !string.IsNullOrWhiteSpace(operationName))
                .Distinct(StringComparer.Ordinal)
                .Cast<string>()
                .ToArray();

            return new UnitOperationHostActionExecutionFollowUp(
                Kind: UnitOperationHostActionExecutionFollowUpKind.LifecycleOperation,
                Summary: "Lifecycle operation is required before host actions can continue.",
                MissingInputNames: [],
                RecommendedOperations: lifecycleOperations,
                CanValidate: false,
                CanCalculate: false);
        }

        if (requestPlan.HasMissingInputs || actionPlan.HasBlockingActions)
        {
            var missingInputNames = requestPlan.Entries
                .SelectMany(static entry => entry.MissingInputNames)
                .Distinct(StringComparer.OrdinalIgnoreCase)
                .ToArray();

            return new UnitOperationHostActionExecutionFollowUp(
                Kind: UnitOperationHostActionExecutionFollowUpKind.ProvideInputs,
                Summary: missingInputNames.Length == 0
                    ? "Additional host inputs are required before validation can continue."
                    : $"Additional host inputs are required: {string.Join(", ", missingInputNames)}.",
                MissingInputNames: missingInputNames,
                RecommendedOperations: session.Summary.RecommendedOperations,
                CanValidate: false,
                CanCalculate: false);
        }

        if (execution.InvalidatedValidation)
        {
            return new UnitOperationHostActionExecutionFollowUp(
                Kind: UnitOperationHostActionExecutionFollowUpKind.Validate,
                Summary: "Configuration changed; validate before calculate.",
                MissingInputNames: [],
                RecommendedOperations: [],
                CanValidate: true,
                CanCalculate: false);
        }

        if (session.Summary.IsReadyForCalculate)
        {
            return new UnitOperationHostActionExecutionFollowUp(
                Kind: UnitOperationHostActionExecutionFollowUpKind.Calculate,
                Summary: session.Summary.HasFailureReport
                    ? "Configuration is ready; calculate can be retried."
                    : "Configuration is ready; calculate can run.",
                MissingInputNames: [],
                RecommendedOperations: [],
                CanValidate: true,
                CanCalculate: true);
        }

        return new UnitOperationHostActionExecutionFollowUp(
            Kind: UnitOperationHostActionExecutionFollowUpKind.ProvideInputs,
            Summary: "Host state is incomplete and requires additional configuration.",
            MissingInputNames: [],
            RecommendedOperations: session.Summary.RecommendedOperations,
            CanValidate: false,
            CanCalculate: false);
    }
}

public sealed record UnitOperationHostActionExecutionFollowUp(
    UnitOperationHostActionExecutionFollowUpKind Kind,
    string Summary,
    IReadOnlyList<string> MissingInputNames,
    IReadOnlyList<string> RecommendedOperations,
    bool CanValidate,
    bool CanCalculate);

public enum UnitOperationHostActionExecutionFollowUpKind
{
    LifecycleOperation,
    ProvideInputs,
    Validate,
    Calculate,
    Terminated,
}
