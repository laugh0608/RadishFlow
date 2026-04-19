namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostReportPresenter
{
    public static UnitOperationHostReportPresentation Present(
        UnitOperationHostReportSnapshot snapshot)
    {
        ArgumentNullException.ThrowIfNull(snapshot);

        var supplementalLines = snapshot.ScalarLines
            .Skip(1 + snapshot.DetailEntries.Count)
            .ToArray();

        return new UnitOperationHostReportPresentation(
            State: snapshot.State,
            StateLabel: GetStateLabel(snapshot.State),
            Headline: snapshot.Headline,
            RequiresAttention: snapshot.State == UnitOperationCalculationReportState.Failure,
            StableDetails: snapshot.DetailEntries.ToArray(),
            SupplementalLines: supplementalLines,
            DisplayLines: snapshot.ScalarLines.ToArray(),
            DisplayText: snapshot.Text);
    }

    private static string GetStateLabel(UnitOperationCalculationReportState state)
    {
        return state switch
        {
            UnitOperationCalculationReportState.None => "NoResult",
            UnitOperationCalculationReportState.Success => "Success",
            UnitOperationCalculationReportState.Failure => "Failure",
            _ => throw new ArgumentOutOfRangeException(nameof(state)),
        };
    }
}

public sealed record UnitOperationHostReportPresentation(
    UnitOperationCalculationReportState State,
    string StateLabel,
    string Headline,
    bool RequiresAttention,
    IReadOnlyList<UnitOperationHostReportDetailEntry> StableDetails,
    IReadOnlyList<string> SupplementalLines,
    IReadOnlyList<string> DisplayLines,
    string DisplayText)
{
    public bool HasStableDetails => StableDetails.Count > 0;

    public bool HasSupplementalLines => SupplementalLines.Count > 0;
}
