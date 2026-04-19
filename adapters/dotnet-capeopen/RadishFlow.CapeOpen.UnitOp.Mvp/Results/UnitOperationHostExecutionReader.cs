using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostExecutionReader
{
    public static UnitOperationHostExecutionSnapshot Read(
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
            return new UnitOperationHostExecutionSnapshot(
                State: UnitOperationHostExecutionState.Terminated,
                Headline: "Unit operation has been terminated.",
                CalculationStatus: null,
                IsCurrentConfigurationExecution: false,
                Summary: null,
                DiagnosticEntries: [],
                StepEntries: []);
        }

        if (unitOperation.LastCalculationResult is not null)
        {
            return CreateAvailableSnapshot(unitOperation.LastCalculationResult);
        }

        if (unitOperation.HostExecutionResultsStale)
        {
            return new UnitOperationHostExecutionSnapshot(
                State: UnitOperationHostExecutionState.Stale,
                Headline: "Execution result is stale and requires Calculate() to refresh.",
                CalculationStatus: null,
                IsCurrentConfigurationExecution: false,
                Summary: null,
                DiagnosticEntries: [],
                StepEntries: []);
        }

        if (unitOperation.LastCalculationFailure is not null)
        {
            return new UnitOperationHostExecutionSnapshot(
                State: UnitOperationHostExecutionState.None,
                Headline: "No current execution result is available because the last calculation failed.",
                CalculationStatus: null,
                IsCurrentConfigurationExecution: false,
                Summary: null,
                DiagnosticEntries: [],
                StepEntries: []);
        }

        return new UnitOperationHostExecutionSnapshot(
            State: UnitOperationHostExecutionState.None,
            Headline: "No current execution result is available.",
            CalculationStatus: null,
            IsCurrentConfigurationExecution: false,
            Summary: null,
            DiagnosticEntries: [],
            StepEntries: []);
    }

    private static UnitOperationHostExecutionSnapshot CreateAvailableSnapshot(
        UnitOperationCalculationResult result)
    {
        ArgumentNullException.ThrowIfNull(result);

        return new UnitOperationHostExecutionSnapshot(
            State: UnitOperationHostExecutionState.Available,
            Headline: result.Summary.PrimaryMessage,
            CalculationStatus: result.Status,
            IsCurrentConfigurationExecution: true,
            Summary: new UnitOperationHostExecutionSummary(
                HighestSeverity: result.Summary.HighestSeverity,
                PrimaryMessage: result.Summary.PrimaryMessage,
                DiagnosticCount: result.Summary.DiagnosticCount,
                RelatedUnitIds: result.Summary.RelatedUnitIds,
                RelatedStreamIds: result.Summary.RelatedStreamIds),
            DiagnosticEntries: result.Diagnostics
                .Select(static diagnostic => new UnitOperationHostExecutionDiagnosticEntry(
                    Severity: diagnostic.Severity,
                    Code: diagnostic.Code,
                    Message: diagnostic.Message,
                    RelatedUnitIds: diagnostic.RelatedUnitIds,
                    RelatedStreamIds: diagnostic.RelatedStreamIds))
                .ToArray(),
            StepEntries: result.Steps
                .Select(static step => new UnitOperationHostExecutionStepEntry(
                    Index: step.Index,
                    UnitId: step.UnitId,
                    UnitName: step.UnitName,
                    UnitKind: step.UnitKind,
                    ConsumedStreamIds: step.ConsumedStreamIds,
                    ProducedStreamIds: step.ProducedStreamIds,
                    Summary: step.Summary))
                .ToArray());
    }
}

public sealed record UnitOperationHostExecutionSnapshot(
    UnitOperationHostExecutionState State,
    string Headline,
    string? CalculationStatus,
    bool IsCurrentConfigurationExecution,
    UnitOperationHostExecutionSummary? Summary,
    IReadOnlyList<UnitOperationHostExecutionDiagnosticEntry> DiagnosticEntries,
    IReadOnlyList<UnitOperationHostExecutionStepEntry> StepEntries)
{
    public int DiagnosticCount => DiagnosticEntries.Count;

    public int StepCount => StepEntries.Count;

    public UnitOperationHostExecutionStepEntry GetStep(int zeroBasedIndex)
    {
        ArgumentOutOfRangeException.ThrowIfNegative(zeroBasedIndex);
        if (zeroBasedIndex >= StepEntries.Count)
        {
            throw new ArgumentOutOfRangeException(nameof(zeroBasedIndex));
        }

        return StepEntries[zeroBasedIndex];
    }
}

public sealed record UnitOperationHostExecutionSummary(
    string HighestSeverity,
    string PrimaryMessage,
    int DiagnosticCount,
    IReadOnlyList<string> RelatedUnitIds,
    IReadOnlyList<string> RelatedStreamIds);

public sealed record UnitOperationHostExecutionDiagnosticEntry(
    string Severity,
    string Code,
    string Message,
    IReadOnlyList<string> RelatedUnitIds,
    IReadOnlyList<string> RelatedStreamIds);

public sealed record UnitOperationHostExecutionStepEntry(
    int Index,
    string UnitId,
    string UnitName,
    string UnitKind,
    IReadOnlyList<string> ConsumedStreamIds,
    IReadOnlyList<string> ProducedStreamIds,
    string Summary);

public enum UnitOperationHostExecutionState
{
    None,
    Stale,
    Available,
    Terminated,
}
