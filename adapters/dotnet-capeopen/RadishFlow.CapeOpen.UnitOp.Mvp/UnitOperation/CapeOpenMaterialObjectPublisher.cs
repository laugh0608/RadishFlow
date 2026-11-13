using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal static class CapeOpenMaterialObjectPublisher
{
    public static UnitOperationCalculationStream Publish(CapeOpenMaterialAccess material,
        MaterialBoundaryConfiguration boundary, UnitOperationCalculationResult result)
    {
        const string stage = "Product.PreparePublication";
        if (result.Status != "converged") throw MaterialBoundaryValues.Invalid(stage, "Native calculation did not converge.");
        var outputs = result.Streams.Where(stream => stream.Id == boundary.OutputStreamId).ToArray();
        if (outputs.Length != 1) throw MaterialBoundaryValues.Invalid(stage, $"Expected exactly one result for `{boundary.OutputStreamId}`.");
        var output = outputs[0];
        var source = output.OverallMoleFractions.Keys.Select(id => new MaterialCompound(id)).ToArray();
        MaterialCompoundMapping.Match(source, boundary.Compounds, stage);
        var compounds = material.Compounds();
        var mapping = MaterialCompoundMapping.Match(source, compounds, stage);
        MaterialBoundaryValues.State(output.TemperatureK, output.PressurePa, output.TotalMolarFlowMolS, stage);
        var fractions = MaterialBoundaryValues.Composition(output.OverallMoleFractions.Values.ToArray(), source.Length, stage);
        var requested = new MaterialState(output.TemperatureK, output.PressurePa, output.TotalMolarFlowMolS,
            mapping.Select(index => fractions[index]).ToArray());
        material.CheckPublicationCapabilities();

        // All candidate validation precedes the first mutation. COM writes are not a transaction.
        try
        {
            material.WriteState(requested);
            material.Equilibrate();
            var actual = material.ReadState();
            MaterialBoundaryValues.State(actual.TemperatureK, actual.PressurePa, actual.FlowMolS, "Product.Readback");
            MaterialBoundaryValues.Equal(actual.TemperatureK, requested.TemperatureK, "Product.temperature");
            MaterialBoundaryValues.Equal(actual.PressurePa, requested.PressurePa, "Product.pressure");
            MaterialBoundaryValues.Equal(actual.FlowMolS, requested.FlowMolS, "Product.totalFlow");
            var actualFractions = MaterialBoundaryValues.Composition(actual.Fractions, compounds.Length, "Product.fraction");
            for (var i = 0; i < compounds.Length; i++)
                if (Math.Abs(actualFractions[i] - requested.Fractions[i]) > MaterialBoundaryValues.CompositionTolerance)
                    throw MaterialBoundaryValues.Invalid("Product.fraction", $"Readback composition differs for `{compounds[i].Id}`.");
            var phases = material.ReadPhases();
            if (phases.Count == 0 || phases.Select(phase => phase.Label).Distinct(StringComparer.Ordinal).Count() != phases.Count)
                throw MaterialBoundaryValues.Invalid("Product.phases", "Equilibrium must produce distinct present phases.");
            var phaseFractions = MaterialBoundaryValues.Composition(phases.Select(phase => phase.Fraction).ToArray(), phases.Count, "Product.phaseFractions");
            var phaseCompositions = phases.Select(phase => MaterialBoundaryValues.Composition(phase.Composition,
                compounds.Length, $"Product.{phase.Label}.fraction")).ToArray();
            for (var i = 0; i < compounds.Length; i++)
            {
                var reconstructed = Enumerable.Range(0, phases.Count).Sum(p => phaseFractions[p] * phaseCompositions[p][i]);
                if (Math.Abs(reconstructed - actualFractions[i]) > MaterialBoundaryValues.CompositionTolerance)
                    throw MaterialBoundaryValues.Invalid("Product.phaseBalance", $"Phase compositions do not close for `{compounds[i].Id}`.");
            }
            var toProject = MaterialCompoundMapping.Match(compounds, boundary.Compounds, "Product.ReadbackMapping");
            IReadOnlyDictionary<string, double> ProjectComposition(double[] values) => boundary.Compounds
                .Select((compound, i) => (compound.Id, Value: values[toProject[i]]))
                .ToDictionary(pair => pair.Id, pair => pair.Value, StringComparer.Ordinal);
            return output with
            {
                TemperatureK = actual.TemperatureK, PressurePa = actual.PressurePa, TotalMolarFlowMolS = actual.FlowMolS,
                OverallMoleFractions = ProjectComposition(actualFractions),
                Phases = phases.Select((phase, i) => new UnitOperationCalculationPhase(phase.Label,
                    ProjectComposition(phaseCompositions[i]), phaseFractions[i])).ToArray(),
            };
        }
        catch (CapeOpenException error)
        {
            // Preserve semantic category and original call details while making partial publication explicit.
            var description = $"Product `{boundary.OutputStreamId}` publication failed; the host material may be partially updated and has not been rolled back. {error.Description}";
            throw MaterialBoundaryValues.Propagate(error, error.Operation, description);
        }
    }
}
