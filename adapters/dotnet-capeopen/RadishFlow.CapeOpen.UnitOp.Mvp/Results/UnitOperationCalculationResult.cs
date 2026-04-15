using System.Text.Json;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public sealed record UnitOperationCalculationResult(
    string Status,
    UnitOperationCalculationSummary Summary,
    IReadOnlyList<UnitOperationCalculationDiagnostic> Diagnostics)
{
    internal static UnitOperationCalculationResult Parse(string snapshotJson)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(snapshotJson);

        using var document = JsonDocument.Parse(snapshotJson);
        var root = RequireObject(document.RootElement, "$");

        return new UnitOperationCalculationResult(
            Status: GetRequiredString(root, "status", "$"),
            Summary: ParseSummary(GetRequiredProperty(root, "summary", "$"), "$.summary"),
            Diagnostics: ParseDiagnostics(
                GetRequiredProperty(root, "diagnostics", "$"),
                "$.diagnostics"));
    }

    private static UnitOperationCalculationSummary ParseSummary(JsonElement element, string path)
    {
        var summary = RequireObject(element, path);
        return new UnitOperationCalculationSummary(
            HighestSeverity: GetRequiredString(summary, "highestSeverity", path),
            PrimaryMessage: GetRequiredString(summary, "primaryMessage", path),
            DiagnosticCount: GetRequiredInt32(summary, "diagnosticCount", path),
            RelatedUnitIds: ReadRequiredStringArray(summary, "relatedUnitIds", path),
            RelatedStreamIds: ReadRequiredStringArray(summary, "relatedStreamIds", path));
    }

    private static IReadOnlyList<UnitOperationCalculationDiagnostic> ParseDiagnostics(
        JsonElement element,
        string path)
    {
        if (element.ValueKind != JsonValueKind.Array)
        {
            throw new InvalidDataException(
                $"Expected `{path}` to be a JSON array but found `{element.ValueKind}`.");
        }

        var diagnostics = new List<UnitOperationCalculationDiagnostic>(element.GetArrayLength());
        var index = 0;
        foreach (var item in element.EnumerateArray())
        {
            var diagnosticPath = $"{path}[{index}]";
            var diagnostic = RequireObject(item, diagnosticPath);
            diagnostics.Add(new UnitOperationCalculationDiagnostic(
                Severity: GetRequiredString(diagnostic, "severity", diagnosticPath),
                Code: GetRequiredString(diagnostic, "code", diagnosticPath),
                Message: GetRequiredString(diagnostic, "message", diagnosticPath),
                RelatedUnitIds: ReadRequiredStringArray(diagnostic, "relatedUnitIds", diagnosticPath),
                RelatedStreamIds: ReadRequiredStringArray(diagnostic, "relatedStreamIds", diagnosticPath)));
            index++;
        }

        return diagnostics;
    }

    private static JsonElement RequireObject(JsonElement element, string path)
    {
        if (element.ValueKind != JsonValueKind.Object)
        {
            throw new InvalidDataException(
                $"Expected `{path}` to be a JSON object but found `{element.ValueKind}`.");
        }

        return element;
    }

    private static JsonElement GetRequiredProperty(JsonElement element, string propertyName, string path)
    {
        if (!element.TryGetProperty(propertyName, out var property))
        {
            throw new InvalidDataException(
                $"Required property `{path}.{propertyName}` is missing from the native solve snapshot.");
        }

        return property;
    }

    private static string GetRequiredString(JsonElement element, string propertyName, string path)
    {
        var property = GetRequiredProperty(element, propertyName, path);
        if (property.ValueKind != JsonValueKind.String)
        {
            throw new InvalidDataException(
                $"Expected `{path}.{propertyName}` to be a JSON string but found `{property.ValueKind}`.");
        }

        var value = property.GetString();
        if (string.IsNullOrWhiteSpace(value))
        {
            throw new InvalidDataException(
                $"Required property `{path}.{propertyName}` must not be empty.");
        }

        return value;
    }

    private static int GetRequiredInt32(JsonElement element, string propertyName, string path)
    {
        var property = GetRequiredProperty(element, propertyName, path);
        if (!property.TryGetInt32(out var value))
        {
            throw new InvalidDataException(
                $"Expected `{path}.{propertyName}` to be a 32-bit integer but found `{property.ValueKind}`.");
        }

        return value;
    }

    private static IReadOnlyList<string> ReadRequiredStringArray(
        JsonElement element,
        string propertyName,
        string path)
    {
        var property = GetRequiredProperty(element, propertyName, path);
        if (property.ValueKind != JsonValueKind.Array)
        {
            throw new InvalidDataException(
                $"Expected `{path}.{propertyName}` to be a JSON array but found `{property.ValueKind}`.");
        }

        var values = new List<string>(property.GetArrayLength());
        var index = 0;
        foreach (var item in property.EnumerateArray())
        {
            if (item.ValueKind != JsonValueKind.String)
            {
                throw new InvalidDataException(
                    $"Expected `{path}.{propertyName}[{index}]` to be a JSON string but found `{item.ValueKind}`.");
            }

            var value = item.GetString();
            if (string.IsNullOrWhiteSpace(value))
            {
                throw new InvalidDataException(
                    $"Required property `{path}.{propertyName}[{index}]` must not be empty.");
            }

            values.Add(value);
            index++;
        }

        return values;
    }
}

public sealed record UnitOperationCalculationSummary(
    string HighestSeverity,
    string PrimaryMessage,
    int DiagnosticCount,
    IReadOnlyList<string> RelatedUnitIds,
    IReadOnlyList<string> RelatedStreamIds);

public sealed record UnitOperationCalculationDiagnostic(
    string Severity,
    string Code,
    string Message,
    IReadOnlyList<string> RelatedUnitIds,
    IReadOnlyList<string> RelatedStreamIds);
