namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public enum UnitOperationCalculationReportState
{
    None,
    Success,
    Failure,
}

public sealed record UnitOperationCalculationReport(
    UnitOperationCalculationReportState State,
    string Headline,
    IReadOnlyList<string> DetailLines)
{
    internal static UnitOperationCalculationReport Empty()
    {
        return new UnitOperationCalculationReport(
            State: UnitOperationCalculationReportState.None,
            Headline: "No calculation result is available.",
            DetailLines: Array.Empty<string>());
    }

    internal static UnitOperationCalculationReport FromSuccess(UnitOperationCalculationResult result)
    {
        ArgumentNullException.ThrowIfNull(result);

        var details = new List<string>(4 + result.Diagnostics.Count)
        {
            $"status={result.Status}",
            $"highestSeverity={result.Summary.HighestSeverity}",
            $"diagnosticCount={result.Summary.DiagnosticCount}",
        };
        if (result.Summary.RelatedUnitIds.Count > 0)
        {
            details.Add($"relatedUnitIds={string.Join(", ", result.Summary.RelatedUnitIds)}");
        }

        if (result.Summary.RelatedStreamIds.Count > 0)
        {
            details.Add($"relatedStreamIds={string.Join(", ", result.Summary.RelatedStreamIds)}");
        }

        details.AddRange(result.Diagnostics.Select(
            diagnostic => $"[{diagnostic.Severity}] {diagnostic.Code}: {diagnostic.Message}"));

        return new UnitOperationCalculationReport(
            State: UnitOperationCalculationReportState.Success,
            Headline: result.Summary.PrimaryMessage,
            DetailLines: details);
    }

    internal static UnitOperationCalculationReport FromFailure(UnitOperationCalculationFailure failure)
    {
        ArgumentNullException.ThrowIfNull(failure);

        var details = new List<string>
        {
            $"error={failure.ErrorName}",
            $"operation={failure.Operation}",
        };

        if (!string.IsNullOrWhiteSpace(failure.RequestedOperation))
        {
            details.Add($"requestedOperation={failure.RequestedOperation}");
        }

        if (!string.IsNullOrWhiteSpace(failure.NativeStatus))
        {
            details.Add($"nativeStatus={failure.NativeStatus}");
        }

        if (!string.IsNullOrWhiteSpace(failure.Summary.DiagnosticCode))
        {
            details.Add($"diagnosticCode={failure.Summary.DiagnosticCode}");
        }

        if (failure.Summary.RelatedUnitIds.Count > 0)
        {
            details.Add($"relatedUnitIds={string.Join(", ", failure.Summary.RelatedUnitIds)}");
        }

        if (failure.Summary.RelatedStreamIds.Count > 0)
        {
            details.Add($"relatedStreamIds={string.Join(", ", failure.Summary.RelatedStreamIds)}");
        }

        details.AddRange(failure.Summary.RelatedPortTargets.Select(
            target => $"relatedPortTarget={target.UnitId}.{target.PortName}"));

        return new UnitOperationCalculationReport(
            State: UnitOperationCalculationReportState.Failure,
            Headline: failure.Summary.PrimaryMessage,
            DetailLines: details);
    }
}
