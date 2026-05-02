using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using System.Globalization;
using System.Text.Json;
using System.Text.Json.Nodes;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal static class FlowsheetBoundaryFeedMaterialOverlay
{
    private const string OperationName = "ApplyFeedMaterialOverlay";

    public static string ApplyOrOriginal(
        string flowsheetJson,
        UnitOperationConfiguredBoundaryMaterialBindings bindings,
        CapeOpenFeedMaterialSnapshot? feedMaterial)
    {
        if (feedMaterial is null)
        {
            return flowsheetJson;
        }

        if (bindings.BoundaryInputStreamIds.Count == 0)
        {
            UnitOperationComTrace.Write(OperationName, "skip", "No configured boundary input streams.");
            return flowsheetJson;
        }

        if (bindings.BoundaryInputStreamIds.Count != 1)
        {
            UnitOperationComTrace.Write(
                OperationName,
                "skip",
                $"Expected exactly one boundary input stream for MVP feed overlay; count={bindings.BoundaryInputStreamIds.Count}.");
            return flowsheetJson;
        }

        try
        {
            var streamId = bindings.BoundaryInputStreamIds[0];
            var root = JsonNode.Parse(flowsheetJson)?.AsObject();
            var stream = root?["document"]?["flowsheet"]?["streams"]?[streamId]?.AsObject();
            if (root is null || stream is null)
            {
                UnitOperationComTrace.Write(OperationName, "skip", $"Boundary input stream `{streamId}` was not found in the configured flowsheet JSON.");
                return flowsheetJson;
            }

            var mappedComposition = MapComposition(stream, feedMaterial);
            if (mappedComposition.Count == 0)
            {
                UnitOperationComTrace.Write(
                    OperationName,
                    "skip",
                    $"Boundary input stream `{streamId}` compounds do not overlap with the connected feed material.");
                return flowsheetJson;
            }

            stream["temperature_k"] = feedMaterial.TemperatureK;
            stream["pressure_pa"] = feedMaterial.PressurePa;
            stream["total_molar_flow_mol_s"] = feedMaterial.TotalMolarFlowMolS;
            stream["overall_mole_fractions"] = CreateCompositionObject(mappedComposition);

            UnitOperationComTrace.Write(
                OperationName,
                "applied",
                string.Join(
                    "; ",
                    $"stream={streamId}",
                    $"temperature={FormatDouble(feedMaterial.TemperatureK)}",
                    $"pressure={FormatDouble(feedMaterial.PressurePa)}",
                    $"totalFlow={FormatDouble(feedMaterial.TotalMolarFlowMolS)}",
                    $"composition={FormatComposition(mappedComposition)}"));

            return root.ToJsonString(new JsonSerializerOptions { WriteIndented = false });
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(OperationName, error);
            return flowsheetJson;
        }
    }

    private static IReadOnlyDictionary<string, double> MapComposition(
        JsonObject stream,
        CapeOpenFeedMaterialSnapshot feedMaterial)
    {
        var targetKeys = ReadExistingCompositionKeys(stream);
        if (targetKeys.Count == 0)
        {
            targetKeys = feedMaterial.ComponentFractions
                .Select(static component => component.Id)
                .Where(static id => !string.IsNullOrWhiteSpace(id))
                .ToArray();
        }

        var mappings = targetKeys
            .Select(key => new
            {
                Key = key,
                Value = TryGetFeedMoleFraction(feedMaterial, key),
            })
            .ToArray();
        var matched = mappings
            .Where(static mapping => mapping.Value is not null)
            .ToArray();
        if (matched.Length == 0)
        {
            return new Dictionary<string, double>(StringComparer.OrdinalIgnoreCase);
        }

        var raw = mappings.ToDictionary(
            static mapping => mapping.Key,
            static mapping => mapping.Value ?? 0.0d,
            StringComparer.OrdinalIgnoreCase);
        var sum = raw.Values.Sum();
        if (sum <= 0.0d || !double.IsFinite(sum))
        {
            return raw;
        }

        return raw.ToDictionary(
            static pair => pair.Key,
            pair => pair.Value / sum,
            StringComparer.OrdinalIgnoreCase);
    }

    private static IReadOnlyList<string> ReadExistingCompositionKeys(JsonObject stream)
    {
        return stream["overall_mole_fractions"] is JsonObject composition
            ? composition.Select(static property => property.Key).ToArray()
            : [];
    }

    private static double? TryGetFeedMoleFraction(
        CapeOpenFeedMaterialSnapshot feedMaterial,
        string targetKey)
    {
        foreach (var component in feedMaterial.ComponentFractions)
        {
            if (component.Aliases().Any(alias => string.Equals(alias, targetKey, StringComparison.OrdinalIgnoreCase)) ||
                component.Aliases().Any(alias => NormalizeIdentifier(alias) == NormalizeIdentifier(targetKey)))
            {
                return component.MoleFraction;
            }
        }

        return null;
    }

    private static JsonObject CreateCompositionObject(
        IReadOnlyDictionary<string, double> composition)
    {
        var json = new JsonObject();
        foreach (var (componentId, fraction) in composition)
        {
            json[componentId] = fraction;
        }

        return json;
    }

    private static string NormalizeIdentifier(string value)
    {
        return new string(value
            .Where(char.IsLetterOrDigit)
            .Select(char.ToLowerInvariant)
            .ToArray());
    }

    private static string FormatComposition(IReadOnlyDictionary<string, double> composition)
    {
        return string.Join(
            ",",
            composition.Select(static pair => $"{pair.Key}={FormatDouble(pair.Value)}"));
    }

    private static string FormatDouble(double value)
    {
        return value.ToString("G17", CultureInfo.InvariantCulture);
    }
}
