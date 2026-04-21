using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostFollowUpPlanner
{
    public static UnitOperationHostFollowUp CreateFromActionExecution(
        UnitOperationHostActionExecutionRequestPlan requestPlan,
        UnitOperationHostActionExecutionBatchResult execution,
        UnitOperationHostViewSnapshot views)
    {
        ArgumentNullException.ThrowIfNull(requestPlan);
        ArgumentNullException.ThrowIfNull(execution);
        ArgumentNullException.ThrowIfNull(views);

        if (views.Configuration.State == UnitOperationHostConfigurationState.Terminated ||
            views.Session.State == UnitOperationHostSessionState.Terminated)
        {
            return CreateTerminatedFollowUp();
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

            return new UnitOperationHostFollowUp(
                Kind: UnitOperationHostFollowUpKind.LifecycleOperation,
                Summary: "Lifecycle operation is required before host actions can continue.",
                MissingInputNames: [],
                RecommendedOperations: lifecycleOperations,
                CanValidate: false,
                CanCalculate: false);
        }

        if (requestPlan.HasMissingInputs || views.ActionPlan.HasBlockingActions)
        {
            var missingInputNames = requestPlan.HasMissingInputs
                ? requestPlan.Entries
                    .SelectMany(static entry => entry.MissingInputNames)
                    .Distinct(StringComparer.OrdinalIgnoreCase)
                    .ToArray()
                : CollectBlockingInputNames(views.ActionPlan);

            return CreateProvideInputsFollowUp(
                missingInputNames,
                views.Session.Summary.RecommendedOperations);
        }

        if (execution.InvalidatedValidation)
        {
            return new UnitOperationHostFollowUp(
                Kind: UnitOperationHostFollowUpKind.Validate,
                Summary: "Configuration changed; validate before calculate.",
                MissingInputNames: [],
                RecommendedOperations: [],
                CanValidate: true,
                CanCalculate: false);
        }

        return CreateFromCurrentState(views.Session);
    }

    public static UnitOperationHostFollowUp CreateFromCurrentState(
        UnitOperationHostSessionSnapshot session)
    {
        ArgumentNullException.ThrowIfNull(session);

        if (session.State == UnitOperationHostSessionState.Terminated)
        {
            return CreateTerminatedFollowUp();
        }

        if (ContainsLifecycleOperation(session))
        {
            return new UnitOperationHostFollowUp(
                Kind: UnitOperationHostFollowUpKind.LifecycleOperation,
                Summary: "Lifecycle operation is required before host actions can continue.",
                MissingInputNames: [],
                RecommendedOperations: CollectLifecycleOperations(session.ActionPlan),
                CanValidate: false,
                CanCalculate: false);
        }

        if (session.Summary.HasBlockingActions)
        {
            var missingInputNames = CollectBlockingInputNames(session.ActionPlan);
            return CreateProvideInputsFollowUp(
                missingInputNames,
                session.Summary.RecommendedOperations);
        }

        if (session.State == UnitOperationHostSessionState.Available &&
            session.Summary.HasCurrentResults)
        {
            return new UnitOperationHostFollowUp(
                Kind: UnitOperationHostFollowUpKind.CurrentResults,
                Summary: "Current calculation results are available.",
                MissingInputNames: [],
                RecommendedOperations: [],
                CanValidate: true,
                CanCalculate: true);
        }

        if (session.Summary.IsReadyForCalculate)
        {
            return new UnitOperationHostFollowUp(
                Kind: UnitOperationHostFollowUpKind.Calculate,
                Summary: session.Summary.HasFailureReport
                    ? "Configuration is ready; calculate can be retried."
                    : session.Summary.RequiresCalculateRefresh
                        ? "Configuration is ready; calculate should refresh stale results."
                        : "Configuration is ready; calculate can run.",
                MissingInputNames: [],
                RecommendedOperations: [],
                CanValidate: true,
                CanCalculate: true);
        }

        return CreateProvideInputsFollowUp(
            CollectBlockingInputNames(session.ActionPlan),
            session.Summary.RecommendedOperations);
    }

    public static UnitOperationHostFollowUp CreateFromMutationBatch(
        UnitOperationHostObjectMutationBatchResult mutationBatch,
        UnitOperationHostViewSnapshot views)
    {
        ArgumentNullException.ThrowIfNull(mutationBatch);
        ArgumentNullException.ThrowIfNull(views);

        if (views.Configuration.State == UnitOperationHostConfigurationState.Terminated ||
            views.Session.State == UnitOperationHostSessionState.Terminated)
        {
            return CreateTerminatedFollowUp();
        }

        if (mutationBatch.InvalidatedValidation)
        {
            return new UnitOperationHostFollowUp(
                Kind: UnitOperationHostFollowUpKind.Validate,
                Summary: "Configuration changed; validate before calculate.",
                MissingInputNames: [],
                RecommendedOperations: [],
                CanValidate: true,
                CanCalculate: false);
        }

        return CreateFromCurrentState(views.Session);
    }

    private static bool ContainsLifecycleOperation(UnitOperationHostSessionSnapshot session)
    {
        return CollectLifecycleOperations(session.ActionPlan).Length > 0;
    }

    private static UnitOperationHostFollowUp CreateProvideInputsFollowUp(
        IReadOnlyList<string> missingInputNames,
        IReadOnlyList<string> recommendedOperations)
    {
        var distinctMissingNames = missingInputNames
            .Where(static name => !string.IsNullOrWhiteSpace(name))
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .ToArray();

        return new UnitOperationHostFollowUp(
            Kind: UnitOperationHostFollowUpKind.ProvideInputs,
            Summary: distinctMissingNames.Length == 0
                ? "Additional host inputs are required before validation can continue."
                : $"Additional host inputs are required: {string.Join(", ", distinctMissingNames)}.",
            MissingInputNames: distinctMissingNames,
            RecommendedOperations: recommendedOperations,
            CanValidate: false,
            CanCalculate: false);
    }

    private static UnitOperationHostFollowUp CreateTerminatedFollowUp()
    {
        return new UnitOperationHostFollowUp(
            Kind: UnitOperationHostFollowUpKind.Terminated,
            Summary: "Unit operation has been terminated.",
            MissingInputNames: [],
            RecommendedOperations: [],
            CanValidate: false,
            CanCalculate: false);
    }

    private static string[] CollectBlockingInputNames(
        UnitOperationHostActionPlan actionPlan)
    {
        ArgumentNullException.ThrowIfNull(actionPlan);

        return actionPlan.Actions
            .Where(static action => action.IsBlocking)
            .SelectMany(static action => action.Target.Names)
            .Where(static name => !string.IsNullOrWhiteSpace(name))
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .ToArray();
    }

    private static string[] CollectLifecycleOperations(
        UnitOperationHostActionPlan actionPlan)
    {
        ArgumentNullException.ThrowIfNull(actionPlan);

        return actionPlan.Actions
            .Select(static action => action.CanonicalOperationName)
            .Where(static operationName => string.Equals(
                operationName,
                nameof(RadishFlowCapeOpenUnitOperation.Initialize),
                StringComparison.Ordinal))
            .Distinct(StringComparer.Ordinal)
            .Cast<string>()
            .ToArray();
    }
}

public sealed record UnitOperationHostFollowUp(
    UnitOperationHostFollowUpKind Kind,
    string Summary,
    IReadOnlyList<string> MissingInputNames,
    IReadOnlyList<string> RecommendedOperations,
    bool CanValidate,
    bool CanCalculate);

public enum UnitOperationHostFollowUpKind
{
    LifecycleOperation,
    ProvideInputs,
    Validate,
    Calculate,
    CurrentResults,
    Terminated,
}
