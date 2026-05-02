using System.Text.Json;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public sealed record UnitOperationCalculationResult(
    string Status,
    UnitOperationCalculationSummary Summary,
    IReadOnlyList<UnitOperationCalculationDiagnostic> Diagnostics,
    IReadOnlyList<UnitOperationCalculationStream> Streams,
    IReadOnlyList<UnitOperationCalculationStep> Steps)
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
                "$.diagnostics"),
            Streams: ParseStreams(
                GetRequiredProperty(root, "streams", "$"),
                "$.streams"),
            Steps: ParseSteps(
                GetRequiredProperty(root, "steps", "$"),
                "$.steps"));
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

    private static IReadOnlyList<UnitOperationCalculationStream> ParseStreams(
        JsonElement element,
        string path)
    {
        if (element.ValueKind != JsonValueKind.Array)
        {
            throw new InvalidDataException(
                $"Expected `{path}` to be a JSON array but found `{element.ValueKind}`.");
        }

        var streams = new List<UnitOperationCalculationStream>(element.GetArrayLength());
        var index = 0;
        foreach (var item in element.EnumerateArray())
        {
            var streamPath = $"{path}[{index}]";
            var stream = RequireObject(item, streamPath);
            streams.Add(new UnitOperationCalculationStream(
                Id: GetRequiredString(stream, "id", streamPath),
                Name: GetRequiredString(stream, "name", streamPath),
                TemperatureK: GetRequiredDouble(stream, "temperature_k", streamPath),
                PressurePa: GetRequiredDouble(stream, "pressure_pa", streamPath),
                TotalMolarFlowMolS: GetRequiredDouble(stream, "total_molar_flow_mol_s", streamPath),
                OverallMoleFractions: ReadRequiredDoubleMap(stream, "overall_mole_fractions", streamPath),
                Phases: ParsePhases(
                    GetRequiredProperty(stream, "phases", streamPath),
                    $"{streamPath}.phases")));
            index++;
        }

        return streams;
    }

    private static IReadOnlyList<UnitOperationCalculationPhase> ParsePhases(
        JsonElement element,
        string path)
    {
        if (element.ValueKind != JsonValueKind.Array)
        {
            throw new InvalidDataException(
                $"Expected `{path}` to be a JSON array but found `{element.ValueKind}`.");
        }

        var phases = new List<UnitOperationCalculationPhase>(element.GetArrayLength());
        var index = 0;
        foreach (var item in element.EnumerateArray())
        {
            var phasePath = $"{path}[{index}]";
            var phase = RequireObject(item, phasePath);
            phases.Add(new UnitOperationCalculationPhase(
                Label: GetRequiredString(phase, "label", phasePath),
                MoleFractions: ReadRequiredDoubleMap(phase, "mole_fractions", phasePath),
                PhaseFraction: GetRequiredDouble(phase, "phase_fraction", phasePath)));
            index++;
        }

        return phases;
    }

    private static IReadOnlyList<UnitOperationCalculationStep> ParseSteps(
        JsonElement element,
        string path)
    {
        if (element.ValueKind != JsonValueKind.Array)
        {
            throw new InvalidDataException(
                $"Expected `{path}` to be a JSON array but found `{element.ValueKind}`.");
        }

        var steps = new List<UnitOperationCalculationStep>(element.GetArrayLength());
        var index = 0;
        foreach (var item in element.EnumerateArray())
        {
            var stepPath = $"{path}[{index}]";
            var step = RequireObject(item, stepPath);
            steps.Add(new UnitOperationCalculationStep(
                Index: GetRequiredInt32(step, "index", stepPath),
                UnitId: GetRequiredString(step, "unitId", stepPath),
                UnitName: GetRequiredString(step, "unitName", stepPath),
                UnitKind: GetRequiredString(step, "unitKind", stepPath),
                ConsumedStreamIds: ReadRequiredStringArray(step, "consumedStreamIds", stepPath),
                ProducedStreamIds: ReadRequiredStringArray(step, "producedStreamIds", stepPath),
                Summary: GetRequiredString(step, "summary", stepPath)));
            index++;
        }

        return steps;
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

    private static double GetRequiredDouble(JsonElement element, string propertyName, string path)
    {
        var property = GetRequiredProperty(element, propertyName, path);
        if (!property.TryGetDouble(out var value))
        {
            throw new InvalidDataException(
                $"Expected `{path}.{propertyName}` to be a floating-point number but found `{property.ValueKind}`.");
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

    private static IReadOnlyDictionary<string, double> ReadRequiredDoubleMap(
        JsonElement element,
        string propertyName,
        string path)
    {
        var property = GetRequiredProperty(element, propertyName, path);
        if (property.ValueKind != JsonValueKind.Object)
        {
            throw new InvalidDataException(
                $"Expected `{path}.{propertyName}` to be a JSON object but found `{property.ValueKind}`.");
        }

        var values = new Dictionary<string, double>(StringComparer.Ordinal);
        foreach (var item in property.EnumerateObject())
        {
            if (!item.Value.TryGetDouble(out var value))
            {
                throw new InvalidDataException(
                    $"Expected `{path}.{propertyName}.{item.Name}` to be a floating-point number but found `{item.Value.ValueKind}`.");
            }

            values[item.Name] = value;
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

public sealed record UnitOperationCalculationStream(
    string Id,
    string Name,
    double TemperatureK,
    double PressurePa,
    double TotalMolarFlowMolS,
    IReadOnlyDictionary<string, double> OverallMoleFractions,
    IReadOnlyList<UnitOperationCalculationPhase> Phases);

public sealed record UnitOperationCalculationPhase(
    string Label,
    IReadOnlyDictionary<string, double> MoleFractions,
    double PhaseFraction);

public sealed record UnitOperationCalculationStep(
    int Index,
    string UnitId,
    string UnitName,
    string UnitKind,
    IReadOnlyList<string> ConsumedStreamIds,
    IReadOnlyList<string> ProducedStreamIds,
    string Summary);
