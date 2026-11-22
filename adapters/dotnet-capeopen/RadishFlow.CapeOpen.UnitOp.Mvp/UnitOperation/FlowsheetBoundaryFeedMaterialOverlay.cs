using System.Text.Json.Nodes;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal static class FlowsheetBoundaryFeedMaterialOverlay
{
    public static string Apply(string flowsheetJson, MaterialBoundaryConfiguration boundary, MaterialState feed)
    {
        var root = JsonNode.Parse(flowsheetJson)!.AsObject();
        var stream = root["document"]!["flowsheet"]!["streams"]![boundary.InputStreamId]!.AsObject();
        var composition = new JsonObject();
        for (var i = 0; i < boundary.Compounds.Length; i++) composition[boundary.Compounds[i].Id] = feed.Fractions[i];
        stream["temperature_k"] = feed.TemperatureK;
        stream["pressure_pa"] = feed.PressurePa;
        stream["total_molar_flow_mol_s"] = feed.FlowMolS;
        stream["overall_mole_fractions"] = composition;
        // Stored phases are not evidence about the newly supplied feed.
        stream["phases"] = new JsonArray();
        return root.ToJsonString();
    }
}
