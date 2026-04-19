using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostReportReader
{
    public static UnitOperationHostReportSnapshot Read(
        RadishFlowCapeOpenUnitOperation unitOperation)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);

        var detailKeys = new string[unitOperation.GetCalculationReportDetailKeyCount()];
        for (var index = 0; index < detailKeys.Length; index++)
        {
            detailKeys[index] = unitOperation.GetCalculationReportDetailKey(index);
        }

        var detailEntries = detailKeys
            .Select(key => new UnitOperationHostReportDetailEntry(
                key,
                unitOperation.GetCalculationReportDetailValue(key)))
            .ToArray();

        var scalarLines = new string[unitOperation.GetCalculationReportLineCount()];
        for (var index = 0; index < scalarLines.Length; index++)
        {
            scalarLines[index] = unitOperation.GetCalculationReportLine(index);
        }

        return new UnitOperationHostReportSnapshot(
            State: unitOperation.GetCalculationReportState(),
            Headline: unitOperation.GetCalculationReportHeadline(),
            DetailEntries: detailEntries,
            ScalarLines: scalarLines,
            VectorLines: unitOperation.GetCalculationReportLines().ToArray(),
            Text: unitOperation.GetCalculationReportText());
    }
}

public sealed record UnitOperationHostReportSnapshot(
    UnitOperationCalculationReportState State,
    string Headline,
    IReadOnlyList<UnitOperationHostReportDetailEntry> DetailEntries,
    IReadOnlyList<string> ScalarLines,
    IReadOnlyList<string> VectorLines,
    string Text)
{
    public int DetailKeyCount => DetailEntries.Count;

    public IReadOnlyList<string> DetailKeys => DetailEntries.Select(static entry => entry.Key).ToArray();

    public string? GetDetailValue(string detailKey)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(detailKey);

        foreach (var entry in DetailEntries)
        {
            if (string.Equals(entry.Key, detailKey, StringComparison.Ordinal))
            {
                return entry.Value;
            }
        }

        return null;
    }
}

public sealed record UnitOperationHostReportDetailEntry(
    string Key,
    string? Value);
