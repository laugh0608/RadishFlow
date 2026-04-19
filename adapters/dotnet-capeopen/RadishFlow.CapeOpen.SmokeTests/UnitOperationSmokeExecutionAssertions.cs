using RadishFlow.CapeOpen.UnitOp.Mvp.Results;

internal static class UnitOperationSmokeExecutionAssertions
{
    public static void AssertState(
        UnitOperationHostExecutionSnapshot snapshot,
        UnitOperationHostExecutionState expectedState,
        bool expectedCurrent,
        string scenario)
    {
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.State == expectedState,
            $"{scenario} should expose execution state `{expectedState}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.IsCurrentConfigurationExecution == expectedCurrent,
            $"{scenario} should expose current-execution flag `{expectedCurrent}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            !string.IsNullOrWhiteSpace(snapshot.Headline),
            $"{scenario} should expose a non-empty execution headline.");
    }

    public static void AssertStepOrder(
        UnitOperationHostExecutionSnapshot snapshot,
        string scenario,
        params string[] expectedUnitIds)
    {
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.StepEntries.Select(static step => step.UnitId).SequenceEqual(expectedUnitIds),
            $"{scenario} should expose the expected execution step order.");
    }
}
