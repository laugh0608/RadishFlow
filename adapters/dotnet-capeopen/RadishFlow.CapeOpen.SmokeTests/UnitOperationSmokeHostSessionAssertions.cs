using RadishFlow.CapeOpen.UnitOp.Mvp.Results;

internal static class UnitOperationSmokeHostSessionAssertions
{
    public static void AssertSummary(
        UnitOperationHostSessionSnapshot snapshot,
        UnitOperationHostSessionState expectedState,
        bool expectedReady,
        bool expectedBlockingActions,
        bool expectedCurrentMaterialResults,
        bool expectedCurrentExecution,
        bool expectedCurrentResults,
        bool expectedRefresh,
        bool expectedFailureReport,
        string scenario,
        params string[] expectedRecommendedOperations)
    {
        UnitOperationSmokeReportAssertions.EnsureCondition(
            !string.IsNullOrWhiteSpace(snapshot.Headline),
            $"{scenario} should expose a non-empty host session headline.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.State == expectedState,
            $"{scenario} should expose host session state `{expectedState}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.Summary.IsReadyForCalculate == expectedReady,
            $"{scenario} should expose ready-for-calculate `{expectedReady}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.Summary.HasBlockingActions == expectedBlockingActions,
            $"{scenario} should expose blocking-actions `{expectedBlockingActions}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.Summary.HasCurrentMaterialResults == expectedCurrentMaterialResults,
            $"{scenario} should expose current-material-results `{expectedCurrentMaterialResults}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.Summary.HasCurrentExecution == expectedCurrentExecution,
            $"{scenario} should expose current-execution `{expectedCurrentExecution}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.Summary.HasCurrentResults == expectedCurrentResults,
            $"{scenario} should expose current-results `{expectedCurrentResults}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.Summary.RequiresCalculateRefresh == expectedRefresh,
            $"{scenario} should expose requires-refresh `{expectedRefresh}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.Summary.HasFailureReport == expectedFailureReport,
            $"{scenario} should expose failure-report `{expectedFailureReport}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.Summary.RecommendedOperations.SequenceEqual(expectedRecommendedOperations),
            $"{scenario} should expose the expected recommended operations.");
    }
}
