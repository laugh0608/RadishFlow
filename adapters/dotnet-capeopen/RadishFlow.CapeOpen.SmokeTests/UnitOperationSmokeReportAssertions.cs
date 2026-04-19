using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;

internal static class UnitOperationSmokeReportAssertions
{
    public static void AssertEmpty(
        UnitOperationHostReportBundle bundle,
        string scenario)
    {
        var report = bundle.Snapshot;
        var presentation = bundle.Presentation;
        var document = bundle.Document;

        EnsureCondition(
            report.State == UnitOperationCalculationReportState.None,
            $"{scenario} should expose an empty calculation report.");
        EnsureCondition(
            string.Equals(report.Headline, "No calculation result is available.", StringComparison.Ordinal),
            $"{scenario} should expose the frozen empty headline.");
        EnsureCondition(
            report.DetailKeyCount == 0,
            $"{scenario} should not expose detail keys.");
        EnsureCondition(
            report.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Status) is null,
            $"{scenario} should not expose status detail values.");
        EnsureCondition(
            report.ScalarLines.Count == 1,
            $"{scenario} should expose exactly one display line.");
        EnsureCondition(
            string.Equals(report.ScalarLines[0], report.Headline, StringComparison.Ordinal),
            $"{scenario} scalar line export should collapse to the headline only.");
        AssertLineApisAgree(report, scenario);
        EnsureCondition(
            string.Equals(report.Text, report.Headline, StringComparison.Ordinal),
            $"{scenario} text should match the headline.");
        EnsureCondition(
            string.Equals(presentation.StateLabel, "NoResult", StringComparison.Ordinal) &&
            !presentation.RequiresAttention &&
            !presentation.HasStableDetails &&
            !presentation.HasSupplementalLines,
            $"{scenario} should expose idle presentation without stable details or supplemental lines.");
        AssertSectionTitles(document, scenario, "Overview");
        EnsureCondition(
            document.Sections[0].Lines.Count == 3,
            $"{scenario} overview section should contain exactly three lines.");
    }

    public static void AssertFailure(
        UnitOperationHostReportBundle bundle,
        CapeOpenException error,
        string scenario,
        IReadOnlyList<string> expectedDetailKeys,
        string? expectedRequestedOperation = null,
        string? expectedNativeStatus = null,
        bool expectSupplementalLines = false)
    {
        var report = bundle.Snapshot;
        var presentation = bundle.Presentation;
        var document = bundle.Document;

        EnsureCondition(
            string.Equals(
                report.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Error),
                error.ErrorName,
                StringComparison.Ordinal),
            $"{scenario} should preserve the CAPE-OPEN semantic error name.");
        EnsureCondition(
            string.Equals(
                report.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Operation),
                error.Operation,
                StringComparison.Ordinal),
            $"{scenario} should preserve the failing operation name.");
        EnsureCondition(
            report.State == UnitOperationCalculationReportState.Failure,
            $"{scenario} should switch the host-visible report into failure state.");
        EnsureCondition(
            !string.IsNullOrWhiteSpace(report.Headline),
            $"{scenario} should expose a non-empty headline.");
        EnsureCondition(
            report.DetailKeys.SequenceEqual(expectedDetailKeys, StringComparer.Ordinal),
            $"{scenario} detail key enumeration should follow the frozen failure key order.");

        if (expectedRequestedOperation is null)
        {
            EnsureCondition(
                report.GetDetailValue(UnitOperationCalculationReportDetailCatalog.RequestedOperation) is null,
                $"{scenario} should not invent requested operation details.");
        }
        else
        {
            EnsureCondition(
                string.Equals(
                    report.GetDetailValue(UnitOperationCalculationReportDetailCatalog.RequestedOperation),
                    expectedRequestedOperation,
                    StringComparison.Ordinal),
                $"{scenario} should expose the expected requested operation.");
            EnsureCondition(
                report.VectorLines.Any(line => line.Contains($"requestedOperation={expectedRequestedOperation}", StringComparison.Ordinal)),
                $"{scenario} should expose the requested follow-up operation in host report lines.");
        }

        if (expectedNativeStatus is null)
        {
            EnsureCondition(
                report.GetDetailValue(UnitOperationCalculationReportDetailCatalog.NativeStatus) is null,
                $"{scenario} should not invent native status.");
        }
        else
        {
            EnsureCondition(
                string.Equals(
                    report.GetDetailValue(UnitOperationCalculationReportDetailCatalog.NativeStatus),
                    expectedNativeStatus,
                    StringComparison.Ordinal),
                $"{scenario} should expose the expected native status.");
            EnsureCondition(
                report.VectorLines.Any(line => line.Contains($"nativeStatus={expectedNativeStatus}", StringComparison.Ordinal)),
                $"{scenario} should expose the mapped native status in host report lines.");
        }

        AssertLineApisAgree(report, scenario);
        EnsureCondition(
            string.Equals(presentation.StateLabel, "Failure", StringComparison.Ordinal) &&
            presentation.RequiresAttention &&
            presentation.HasStableDetails &&
            presentation.HasSupplementalLines == expectSupplementalLines,
            $"{scenario} should expose failure label, attention hint and the expected supplemental line state.");
        AssertSectionTitles(document, scenario, "Overview", "Stable Details");
    }

    public static void AssertSuccess(
        UnitOperationHostReportBundle bundle,
        string scenario)
    {
        var report = bundle.Snapshot;
        var presentation = bundle.Presentation;
        var document = bundle.Document;

        EnsureCondition(
            report.State == UnitOperationCalculationReportState.Success,
            $"{scenario} should switch the host-visible report into success state.");
        EnsureCondition(
            !string.IsNullOrWhiteSpace(report.Headline),
            $"{scenario} should expose a non-empty headline.");
        EnsureCondition(
            report.DetailKeys.SequenceEqual(
                UnitOperationCalculationReportDetailCatalog.SuccessStableKeyOrder,
                StringComparer.Ordinal),
            $"{scenario} should expose the frozen success key order.");
        EnsureCondition(
            string.Equals(
                report.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Status),
                "converged",
                StringComparison.Ordinal) &&
            string.Equals(
                report.GetDetailValue(UnitOperationCalculationReportDetailCatalog.HighestSeverity),
                "info",
                StringComparison.Ordinal),
            $"{scenario} should expose stable status and highest severity detail values.");
        EnsureCondition(
            int.TryParse(
                report.GetDetailValue(UnitOperationCalculationReportDetailCatalog.DiagnosticCount),
                out var diagnosticCount) &&
            diagnosticCount > 0,
            $"{scenario} should expose a positive diagnostic count.");
        EnsureCondition(
            report.ScalarLines.Count > report.DetailKeyCount + 1,
            $"{scenario} should expose non-key diagnostic display lines in addition to stable detail entries.");
        EnsureCondition(
            report.Text.Contains(report.Headline, StringComparison.Ordinal) &&
            report.Text.Contains("diagnosticCount=", StringComparison.Ordinal) &&
            report.Text.Contains("[info]", StringComparison.Ordinal),
            $"{scenario} should include headline, stable detail lines and diagnostic display lines in text export.");
        AssertLineApisAgree(report, scenario);
        EnsureCondition(
            string.Equals(presentation.StateLabel, "Success", StringComparison.Ordinal) &&
            !presentation.RequiresAttention &&
            presentation.HasStableDetails &&
            presentation.HasSupplementalLines &&
            presentation.SupplementalLines.All(line => line.StartsWith("[", StringComparison.Ordinal)),
            $"{scenario} should expose success label, stable details and diagnostic supplemental lines.");
        AssertSectionTitles(document, scenario, "Overview", "Stable Details", "Supplemental");
        EnsureCondition(
            document.FormattedText.Contains("[Overview]", StringComparison.Ordinal) &&
            document.FormattedText.Contains("[Stable Details]", StringComparison.Ordinal) &&
            document.FormattedText.Contains("[Supplemental]", StringComparison.Ordinal),
            $"{scenario} formatted text should include all section headers.");
    }

    public static void AssertRepeatedSuccessShape(
        UnitOperationHostReportBundle bundle,
        string scenario)
    {
        EnsureCondition(
            bundle.Snapshot.State == UnitOperationCalculationReportState.Success &&
            bundle.Snapshot.DetailKeys.SequenceEqual(
                UnitOperationCalculationReportDetailCatalog.SuccessStableKeyOrder,
                StringComparer.Ordinal),
            $"{scenario} should preserve the frozen success report shape.");
    }

    public static void AssertLineApisAgree(
        UnitOperationHostReportSnapshot report,
        string scenario)
    {
        EnsureCondition(
            report.ScalarLines.SequenceEqual(report.VectorLines, StringComparer.Ordinal),
            $"{scenario} scalar and vector line exports should match.");
        EnsureCondition(
            string.Equals(report.Text, string.Join(Environment.NewLine, report.ScalarLines), StringComparison.Ordinal),
            $"{scenario} text export should match the scalar line export.");
    }

    public static void AssertSectionTitles(
        UnitOperationHostReportDocument document,
        string scenario,
        params string[] expectedTitles)
    {
        EnsureCondition(document.HasSections, $"{scenario} should expose sectioned host output.");
        EnsureCondition(
            document.Sections.Count == expectedTitles.Length,
            $"{scenario} should expose {expectedTitles.Length} section(s).");

        for (var index = 0; index < expectedTitles.Length; index++)
        {
            EnsureCondition(
                string.Equals(document.Sections[index].Title, expectedTitles[index], StringComparison.Ordinal),
                $"{scenario} section {index} should be `{expectedTitles[index]}`.");
        }
    }

    public static void EnsureCondition(bool condition, string message)
    {
        if (!condition)
        {
            throw new InvalidOperationException(message);
        }
    }
}
