using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.SmokeTests;

internal sealed class UnitOperationHostReportConsumer
{
    private readonly RadishFlowCapeOpenUnitOperation _unitOperation;

    public UnitOperationHostReportConsumer(RadishFlowCapeOpenUnitOperation unitOperation)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);
        _unitOperation = unitOperation;
    }

    public UnitOperationHostReportSnapshot ReadCurrentReport()
    {
        var detailKeys = new string[_unitOperation.GetCalculationReportDetailKeyCount()];
        for (var index = 0; index < detailKeys.Length; index++)
        {
            detailKeys[index] = _unitOperation.GetCalculationReportDetailKey(index);
        }

        var detailEntries = detailKeys
            .Select(key => new UnitOperationHostReportDetailEntry(
                key,
                _unitOperation.GetCalculationReportDetailValue(key)))
            .ToArray();

        var scalarLines = new string[_unitOperation.GetCalculationReportLineCount()];
        for (var index = 0; index < scalarLines.Length; index++)
        {
            scalarLines[index] = _unitOperation.GetCalculationReportLine(index);
        }

        return new UnitOperationHostReportSnapshot(
            State: _unitOperation.GetCalculationReportState(),
            Headline: _unitOperation.GetCalculationReportHeadline(),
            DetailEntries: detailEntries,
            ScalarLines: scalarLines,
            VectorLines: _unitOperation.GetCalculationReportLines().ToArray(),
            Text: _unitOperation.GetCalculationReportText());
    }
}

internal sealed record UnitOperationHostReportSnapshot(
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

internal sealed record UnitOperationHostReportDetailEntry(
    string Key,
    string? Value);
