namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostReportFormatter
{
    public static UnitOperationHostReportDocument Format(
        UnitOperationHostReportPresentation presentation)
    {
        ArgumentNullException.ThrowIfNull(presentation);

        var sections = new List<UnitOperationHostReportSection>(3)
        {
            new(
                Title: "Overview",
                Lines:
                [
                    $"state={presentation.StateLabel}",
                    $"headline={presentation.Headline}",
                    $"requiresAttention={presentation.RequiresAttention.ToString().ToLowerInvariant()}",
                ]),
        };

        if (presentation.HasStableDetails)
        {
            sections.Add(new UnitOperationHostReportSection(
                Title: "Stable Details",
                Lines: presentation.StableDetails
                    .Select(static detail => $"{detail.Key}={detail.Value}")
                    .ToArray()));
        }

        if (presentation.HasSupplementalLines)
        {
            sections.Add(new UnitOperationHostReportSection(
                Title: "Supplemental",
                Lines: presentation.SupplementalLines.ToArray()));
        }

        var documentSections = sections.ToArray();
        return new UnitOperationHostReportDocument(
            StateLabel: presentation.StateLabel,
            Headline: presentation.Headline,
            RequiresAttention: presentation.RequiresAttention,
            Sections: documentSections,
            FormattedText: string.Join(
                Environment.NewLine + Environment.NewLine,
                documentSections.Select(static section => section.ToDisplayText())));
    }
}

public sealed record UnitOperationHostReportDocument(
    string StateLabel,
    string Headline,
    bool RequiresAttention,
    IReadOnlyList<UnitOperationHostReportSection> Sections,
    string FormattedText)
{
    public bool HasSections => Sections.Count > 0;
}

public sealed record UnitOperationHostReportSection(
    string Title,
    IReadOnlyList<string> Lines)
{
    public string ToDisplayText()
    {
        if (Lines.Count == 0)
        {
            return $"[{Title}]";
        }

        return $"[{Title}]{Environment.NewLine}{string.Join(Environment.NewLine, Lines)}";
    }
}
