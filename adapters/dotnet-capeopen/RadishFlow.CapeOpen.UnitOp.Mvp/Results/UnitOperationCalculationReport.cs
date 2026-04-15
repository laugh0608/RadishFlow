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
    public int GetDetailKeyCount()
    {
        var count = 0;
        foreach (var detailLine in DetailLines)
        {
            if (TrySplitDetailLine(detailLine, out _, out _))
            {
                count++;
            }
        }

        return count;
    }

    public string GetDetailKey(int detailKeyIndex)
    {
        ArgumentOutOfRangeException.ThrowIfNegative(detailKeyIndex);

        var currentIndex = 0;
        foreach (var detailLine in DetailLines)
        {
            if (!TrySplitDetailLine(detailLine, out var key, out _))
            {
                continue;
            }

            if (currentIndex == detailKeyIndex)
            {
                return key;
            }

            currentIndex++;
        }

        throw new ArgumentOutOfRangeException(nameof(detailKeyIndex));
    }

    public string? GetDetailValue(string detailKey)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(detailKey);

        foreach (var detailLine in DetailLines)
        {
            if (!TrySplitDetailLine(detailLine, out var key, out var value))
            {
                continue;
            }

            if (string.Equals(key, detailKey, StringComparison.OrdinalIgnoreCase))
            {
                return value;
            }
        }

        return null;
    }

    public UnitOperationCalculationReportState GetDisplayState()
    {
        return State;
    }

    public string GetDisplayHeadline()
    {
        return Headline;
    }

    public int GetDisplayLineCount()
    {
        return DetailLines.Count + 1;
    }

    public string GetDisplayLine(int lineIndex)
    {
        ArgumentOutOfRangeException.ThrowIfNegative(lineIndex);

        if (lineIndex == 0)
        {
            return Headline;
        }

        if (lineIndex > DetailLines.Count)
        {
            throw new ArgumentOutOfRangeException(nameof(lineIndex));
        }

        return DetailLines[lineIndex - 1];
    }

    public IReadOnlyList<string> GetDisplayLines()
    {
        if (DetailLines.Count == 0)
        {
            return [Headline];
        }

        var lines = new string[GetDisplayLineCount()];
        for (var index = 0; index < lines.Length; index++)
        {
            lines[index] = GetDisplayLine(index);
        }

        return lines;
    }

    public string GetDisplayText()
    {
        return string.Join(Environment.NewLine, GetDisplayLines());
    }

    private static bool TrySplitDetailLine(
        string detailLine,
        out string key,
        out string value)
    {
        var separatorIndex = detailLine.IndexOf('=');
        if (separatorIndex <= 0 || separatorIndex >= detailLine.Length - 1)
        {
            key = string.Empty;
            value = string.Empty;
            return false;
        }

        key = detailLine[..separatorIndex];
        value = detailLine[(separatorIndex + 1)..];
        return true;
    }

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
