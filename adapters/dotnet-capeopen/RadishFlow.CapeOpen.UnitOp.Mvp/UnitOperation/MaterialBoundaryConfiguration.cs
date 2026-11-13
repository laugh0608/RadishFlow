using System.Text.Json;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal sealed record MaterialBoundaryConfiguration(string InputStreamId, string OutputStreamId, MaterialCompound[] Compounds)
{
    public static MaterialBoundaryConfiguration Read(string json)
    {
        const string stage = "ConfigureMaterialBoundary";
        try
        {
            using var document = JsonDocument.Parse(json);
            RejectDuplicateProperties(document.RootElement);
            var root = document.RootElement;
            if (root.GetProperty("kind").GetString() != "radishflow.project-file" ||
                root.GetProperty("schemaVersion").GetInt32() is not (1 or 2))
                throw MaterialBoundaryValues.Invalid(stage, "This PMC accepts project formats 1 and 2 only.");
            var bindings = UnitOperationConfiguredBoundaryMaterialBindings.Parse(json);
            if (bindings.BoundaryInputStreamIds.Count != 1 || bindings.BoundaryOutputStreamIds.Count != 1)
                throw MaterialBoundaryValues.Invalid(stage,
                    $"Exactly one material inlet and one material outlet are required. Inlets=[{string.Join(",", bindings.BoundaryInputStreamIds)}]; outlets=[{string.Join(",", bindings.BoundaryOutputStreamIds)}].");
            var flowsheet = root.GetProperty("document").GetProperty("flowsheet");
            var compounds = flowsheet.GetProperty("components").EnumerateObject().Select(property =>
            {
                if (property.Value.GetProperty("id").GetString() != property.Name)
                    throw MaterialBoundaryValues.Invalid(stage, "Compound key and ID disagree.");
                return new MaterialCompound(property.Name); // Explicit ID aliases only, never formula inference.
            }).ToArray();
            MaterialCompoundMapping.Match(compounds, compounds, stage);
            var streams = flowsheet.GetProperty("streams");
            foreach (var id in new[] { bindings.BoundaryInputStreamIds[0], bindings.BoundaryOutputStreamIds[0] })
            {
                var stream = streams.GetProperty(id);
                if (stream.GetProperty("id").GetString() != id)
                    throw MaterialBoundaryValues.Invalid(stage, $"Stream `{id}` key and ID disagree.");
            }
            return new(bindings.BoundaryInputStreamIds[0], bindings.BoundaryOutputStreamIds[0], compounds);
        }
        catch (CapeOpenException) { throw; }
        catch (Exception error) when (error is JsonException or InvalidDataException or KeyNotFoundException or InvalidOperationException or FormatException)
        {
            throw MaterialBoundaryValues.Invalid(stage, $"Invalid project boundary: {error.Message}");
        }
    }

    public void ValidatePackage(string manifestsJson, string selectedId)
    {
        using var document = JsonDocument.Parse(manifestsJson);
        var packages = document.RootElement.EnumerateArray()
            .Where(package => package.GetProperty("packageId").GetString() == selectedId).ToArray();
        // Let the native resolver report an unknown package with its existing error context.
        if (packages.Length == 0) return;
        if (packages.Length != 1)
            throw MaterialBoundaryValues.Invalid("SelectMaterialPackage", $"Selected package `{selectedId}` is not available uniquely.");
        var ids = packages[0].GetProperty("componentIds").EnumerateArray().Select(item => item.GetString()!).ToArray();
        // Native models index by exact project IDs; aliases apply only across the host boundary.
        if (!ids.Order(StringComparer.Ordinal).SequenceEqual(Compounds.Select(c => c.Id).Order(StringComparer.Ordinal)))
            throw MaterialBoundaryValues.Invalid("SelectMaterialPackage", "Selected package and project must have the same compound IDs.");
    }

    private static void RejectDuplicateProperties(JsonElement element)
    {
        if (element.ValueKind == JsonValueKind.Object)
        {
            var names = new HashSet<string>(StringComparer.Ordinal);
            foreach (var property in element.EnumerateObject())
            {
                if (!names.Add(property.Name)) throw MaterialBoundaryValues.Invalid("ConfigureMaterialBoundary", $"Duplicate JSON key `{property.Name}`.");
                RejectDuplicateProperties(property.Value);
            }
        }
        else if (element.ValueKind == JsonValueKind.Array)
            foreach (var item in element.EnumerateArray()) RejectDuplicateProperties(item);
    }
}
