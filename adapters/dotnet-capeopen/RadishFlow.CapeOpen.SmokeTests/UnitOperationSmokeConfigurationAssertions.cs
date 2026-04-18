using RadishFlow.CapeOpen.UnitOp.Mvp.Results;

internal static class UnitOperationSmokeConfigurationAssertions
{
    public static void AssertState(
        UnitOperationHostConfigurationSnapshot snapshot,
        UnitOperationHostConfigurationState expectedState,
        bool expectedReady,
        string scenario)
    {
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.State == expectedState,
            $"{scenario} should expose configuration state `{expectedState}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.IsReadyForCalculate == expectedReady,
            $"{scenario} should expose readiness `{expectedReady}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            !string.IsNullOrWhiteSpace(snapshot.Headline),
            $"{scenario} should expose a non-empty configuration headline.");
    }

    public static void AssertNextOperations(
        UnitOperationHostConfigurationSnapshot snapshot,
        string scenario,
        params string[] expectedOperations)
    {
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.NextOperations.SequenceEqual(expectedOperations, StringComparer.Ordinal),
            $"{scenario} should expose the expected next-operation order.");
    }

    public static void AssertBlockingIssueKinds(
        UnitOperationHostConfigurationSnapshot snapshot,
        string scenario,
        params UnitOperationHostConfigurationIssueKind[] expectedKinds)
    {
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.BlockingIssues.Select(static issue => issue.Kind).SequenceEqual(expectedKinds),
            $"{scenario} should expose the expected configuration blocking issues.");
    }
}
