using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostSessionReader
{
    public static UnitOperationHostSessionSnapshot Read(
        RadishFlowCapeOpenUnitOperation unitOperation)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);

        var configuration = UnitOperationHostConfigurationReader.Read(unitOperation);
        var actionPlan = UnitOperationHostActionPlanReader.Read(configuration);
        var portMaterial = UnitOperationHostPortMaterialReader.Read(unitOperation);
        var execution = UnitOperationHostExecutionReader.Read(unitOperation);
        var report = UnitOperationHostReportReader.Read(unitOperation);
        var summary = CreateSummary(configuration, actionPlan, portMaterial, execution, report);
        var state = DetermineState(configuration, portMaterial, execution, report, summary);

        return new UnitOperationHostSessionSnapshot(
            State: state,
            Headline: CreateHeadline(configuration, portMaterial, execution, report, summary),
            Summary: summary,
            Configuration: configuration,
            ActionPlan: actionPlan,
            PortMaterial: portMaterial,
            Execution: execution,
            Report: report);
    }

    private static UnitOperationHostSessionSummary CreateSummary(
        UnitOperationHostConfigurationSnapshot configuration,
        UnitOperationHostActionPlan actionPlan,
        UnitOperationHostPortMaterialSnapshot portMaterial,
        UnitOperationHostExecutionSnapshot execution,
        UnitOperationHostReportSnapshot report)
    {
        var hasCurrentMaterialResults = portMaterial.State == UnitOperationHostPortMaterialState.Available;
        var hasCurrentExecution = execution.State == UnitOperationHostExecutionState.Available;
        var recommendedOperations = actionPlan.Actions
            .Select(static action => action.CanonicalOperationName)
            .Where(static operationName => !string.IsNullOrWhiteSpace(operationName))
            .Distinct(StringComparer.Ordinal)
            .Cast<string>()
            .ToArray();

        return new UnitOperationHostSessionSummary(
            IsReadyForCalculate: configuration.IsReadyForCalculate,
            HasBlockingActions: actionPlan.HasBlockingActions,
            HasCurrentMaterialResults: hasCurrentMaterialResults,
            HasCurrentExecution: hasCurrentExecution,
            HasCurrentResults: report.State == UnitOperationCalculationReportState.Success &&
                               hasCurrentMaterialResults &&
                               hasCurrentExecution,
            RequiresCalculateRefresh: portMaterial.State == UnitOperationHostPortMaterialState.Stale ||
                                      execution.State == UnitOperationHostExecutionState.Stale,
            HasFailureReport: report.State == UnitOperationCalculationReportState.Failure,
            RecommendedOperations: recommendedOperations);
    }

    private static UnitOperationHostSessionState DetermineState(
        UnitOperationHostConfigurationSnapshot configuration,
        UnitOperationHostPortMaterialSnapshot portMaterial,
        UnitOperationHostExecutionSnapshot execution,
        UnitOperationHostReportSnapshot report,
        UnitOperationHostSessionSummary summary)
    {
        if (configuration.State == UnitOperationHostConfigurationState.Terminated)
        {
            return UnitOperationHostSessionState.Terminated;
        }

        if (summary.HasFailureReport)
        {
            return UnitOperationHostSessionState.Failure;
        }

        if (summary.HasCurrentResults)
        {
            return UnitOperationHostSessionState.Available;
        }

        if (portMaterial.State == UnitOperationHostPortMaterialState.Stale ||
            execution.State == UnitOperationHostExecutionState.Stale)
        {
            return UnitOperationHostSessionState.Stale;
        }

        return configuration.State switch
        {
            UnitOperationHostConfigurationState.Constructed => UnitOperationHostSessionState.Constructed,
            UnitOperationHostConfigurationState.Incomplete => UnitOperationHostSessionState.Incomplete,
            UnitOperationHostConfigurationState.Ready => UnitOperationHostSessionState.Ready,
            UnitOperationHostConfigurationState.Terminated => UnitOperationHostSessionState.Terminated,
            _ => throw new ArgumentOutOfRangeException(nameof(configuration), configuration.State, "Unknown host configuration state."),
        };
    }

    private static string CreateHeadline(
        UnitOperationHostConfigurationSnapshot configuration,
        UnitOperationHostPortMaterialSnapshot portMaterial,
        UnitOperationHostExecutionSnapshot execution,
        UnitOperationHostReportSnapshot report,
        UnitOperationHostSessionSummary summary)
    {
        if (configuration.State == UnitOperationHostConfigurationState.Terminated)
        {
            return configuration.Headline;
        }

        if (summary.HasFailureReport)
        {
            return report.Headline;
        }

        if (summary.HasCurrentExecution)
        {
            return execution.Headline;
        }

        if (summary.RequiresCalculateRefresh)
        {
            return execution.State == UnitOperationHostExecutionState.Stale
                ? execution.Headline
                : portMaterial.Headline;
        }

        return configuration.Headline;
    }
}

public sealed record UnitOperationHostSessionSnapshot(
    UnitOperationHostSessionState State,
    string Headline,
    UnitOperationHostSessionSummary Summary,
    UnitOperationHostConfigurationSnapshot Configuration,
    UnitOperationHostActionPlan ActionPlan,
    UnitOperationHostPortMaterialSnapshot PortMaterial,
    UnitOperationHostExecutionSnapshot Execution,
    UnitOperationHostReportSnapshot Report)
{
    public bool ContainsRecommendedOperation(string operationName)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(operationName);
        return Summary.RecommendedOperations.Any(nextOperation =>
            string.Equals(nextOperation, operationName, StringComparison.Ordinal));
    }
}

public sealed record UnitOperationHostSessionSummary(
    bool IsReadyForCalculate,
    bool HasBlockingActions,
    bool HasCurrentMaterialResults,
    bool HasCurrentExecution,
    bool HasCurrentResults,
    bool RequiresCalculateRefresh,
    bool HasFailureReport,
    IReadOnlyList<string> RecommendedOperations);

public enum UnitOperationHostSessionState
{
    Constructed,
    Incomplete,
    Ready,
    Failure,
    Available,
    Stale,
    Terminated,
}
