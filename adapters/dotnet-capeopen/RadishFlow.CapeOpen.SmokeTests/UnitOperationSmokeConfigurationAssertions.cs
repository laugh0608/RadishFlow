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

    public static void AssertActionPlan(
        UnitOperationHostActionPlan actionPlan,
        string scenario,
        params UnitOperationSmokeExpectedAction[] expectedActions)
    {
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actionPlan.ActionCount == expectedActions.Length,
            $"{scenario} should expose {expectedActions.Length} action-plan item(s).");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actionPlan.Groups.All(static group => !string.IsNullOrWhiteSpace(group.Title) && group.Actions.Count > 0),
            $"{scenario} should expose non-empty action-plan groups.");

        var expectedGroupKinds = expectedActions
            .Select(static action => action.GroupKind)
            .Distinct()
            .ToArray();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actionPlan.Groups.Select(static group => group.Kind).SequenceEqual(expectedGroupKinds),
            $"{scenario} should expose the expected action-plan group order.");

        for (var index = 0; index < expectedActions.Length; index++)
        {
            expectedActions[index].AssertMatches(actionPlan.Actions[index], scenario, index + 1);
        }
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

    public static UnitOperationSmokeExpectedAction Action(
        UnitOperationHostActionGroupKind groupKind,
        UnitOperationHostActionTargetKind targetKind,
        string? canonicalOperationName,
        UnitOperationHostConfigurationIssueKind issueKind,
        string reasonFragment,
        params string[] targetNames)
    {
        return new UnitOperationSmokeExpectedAction(
            GroupKind: groupKind,
            TargetKind: targetKind,
            TargetNames: targetNames,
            CanonicalOperationName: canonicalOperationName,
            IssueKind: issueKind,
            ReasonFragment: reasonFragment,
            IsBlocking: true);
    }
}

internal sealed record UnitOperationSmokeExpectedAction(
    UnitOperationHostActionGroupKind GroupKind,
    UnitOperationHostActionTargetKind TargetKind,
    IReadOnlyList<string> TargetNames,
    string? CanonicalOperationName,
    UnitOperationHostConfigurationIssueKind IssueKind,
    string ReasonFragment,
    bool IsBlocking)
{
    public void AssertMatches(
        UnitOperationHostActionItem actual,
        string scenario,
        int expectedOrder)
    {
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actual.RecommendedOrder == expectedOrder,
            $"{scenario} action #{expectedOrder} should preserve recommended order.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actual.GroupKind == GroupKind,
            $"{scenario} action #{expectedOrder} should stay in group `{GroupKind}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actual.Target.Kind == TargetKind,
            $"{scenario} action #{expectedOrder} should target `{TargetKind}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actual.Target.Names.SequenceEqual(TargetNames, StringComparer.Ordinal),
            $"{scenario} action #{expectedOrder} should expose the expected target names.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actual.IsBlocking == IsBlocking,
            $"{scenario} action #{expectedOrder} should preserve blocking classification.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actual.IssueKind == IssueKind,
            $"{scenario} action #{expectedOrder} should preserve issue kind `{IssueKind}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            string.Equals(actual.CanonicalOperationName, CanonicalOperationName, StringComparison.Ordinal),
            $"{scenario} action #{expectedOrder} should expose the expected canonical operation.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actual.Reason.Contains(ReasonFragment, StringComparison.Ordinal),
            $"{scenario} action #{expectedOrder} should expose a reason containing `{ReasonFragment}`.");
    }
}
