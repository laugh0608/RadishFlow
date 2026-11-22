namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal static class CapeOpenFeedMaterialReader
{
    public static MaterialState Read(CapeOpenMaterialAccess material, MaterialBoundaryConfiguration boundary)
    {
        var compounds = material.Compounds();
        var indices = MaterialCompoundMapping.Match(compounds, boundary.Compounds, "Feed.compoundMapping");
        var state = material.ReadState();
        MaterialBoundaryValues.State(state.TemperatureK, state.PressurePa, state.FlowMolS, "Feed.state");
        var fractions = MaterialBoundaryValues.Composition(state.Fractions, compounds.Length, "Feed.fraction");
        return state with { Fractions = indices.Select(index => fractions[index]).ToArray() };
    }
}
