using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal static class UnitOperationSmokePortMaterialAssertions
{
    public static void AssertSnapshotState(
        UnitOperationHostPortMaterialSnapshot snapshot,
        UnitOperationHostPortMaterialState expectedState,
        string scenario)
    {
        UnitOperationSmokeReportAssertions.EnsureCondition(
            snapshot.State == expectedState,
            $"{scenario} should expose port/material state `{expectedState}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            !string.IsNullOrWhiteSpace(snapshot.Headline),
            $"{scenario} should expose a non-empty port/material headline.");
    }

    public static void AssertPort(
        UnitOperationHostPortMaterialSnapshot snapshot,
        UnitOperationPortDefinition definition,
        UnitOperationHostPortMaterialState expectedState,
        bool expectedConnected,
        string? expectedConnectedTargetName,
        IReadOnlyList<string> expectedBoundStreamIds,
        IReadOnlyList<string> expectedMaterialStreamIds,
        string scenario)
    {
        var entry = snapshot.GetPort(definition.Name);
        UnitOperationSmokeReportAssertions.EnsureCondition(
            entry.Name == definition.Name &&
            entry.Description == definition.Description &&
            entry.Direction == definition.Direction &&
            entry.PortType == definition.PortType &&
            entry.IsRequired == definition.IsRequired &&
            entry.BoundaryMaterialRole == definition.BoundaryMaterialRole,
            $"{scenario} should preserve frozen port catalog metadata.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            entry.MaterialState == expectedState,
            $"{scenario} should expose the expected port/material state for `{definition.Name}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            entry.IsConnected == expectedConnected,
            $"{scenario} should expose the expected connection flag for `{definition.Name}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            string.Equals(entry.ConnectedTargetName, expectedConnectedTargetName, StringComparison.Ordinal),
            $"{scenario} should expose the expected connected target name for `{definition.Name}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            entry.BoundStreamIds.SequenceEqual(expectedBoundStreamIds),
            $"{scenario} should expose the expected bound stream ids for `{definition.Name}`.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            entry.MaterialEntries.Select(static material => material.StreamId).SequenceEqual(expectedMaterialStreamIds),
            $"{scenario} should expose the expected material stream ids for `{definition.Name}`.");
    }
}
