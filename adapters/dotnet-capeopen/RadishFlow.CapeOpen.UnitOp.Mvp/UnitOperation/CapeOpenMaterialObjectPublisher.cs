using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Thermo;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal static class CapeOpenMaterialObjectPublisher
{
    private const string InterfaceName = "ICapeThermoMaterial";
    private const string OperationName = "PublishProductMaterial";
    private const string UnitScope = "RadishFlow.CapeOpen.UnitOp.Mvp";

    public static void PublishProductMaterial(
        object? materialObject,
        IReadOnlyList<UnitOperationCalculationStream> streams)
    {
        if (materialObject is null || streams.Count == 0)
        {
            UnitOperationComTrace.Write(OperationName, "skip", "No connected output material or output stream is available.");
            return;
        }

        try
        {
            if (TryPublishCapeOpen10Material(materialObject, streams))
            {
                return;
            }

            if (TryCast<ICapeThermoMaterial>(materialObject) is not { } material)
            {
                UnitOperationComTrace.Write(
                    OperationName,
                    "skip",
                    $"Connected output object `{materialObject.GetType().FullName}` does not expose {nameof(ICapeThermoMaterial)} or {nameof(ICapeThermoMaterialObject)}.");
                return;
            }

            var aggregate = BoundaryMaterialStreamAggregate.Create(streams);
            var componentIds = GetComponentIds(materialObject, aggregate);
            var fractions = CreateCompositionVector(aggregate.OverallMoleFractions, componentIds);
            var componentFlows = fractions.Select(value => value * aggregate.TotalMolarFlowMolS).ToArray();

            material.ClearAllProps();
            material.SetOverallProp("temperature", string.Empty, new[] { aggregate.TemperatureK });
            material.SetOverallProp("pressure", string.Empty, new[] { aggregate.PressurePa });
            material.SetOverallProp("fraction", "mole", fractions);
            TrySetOverallProp(material, "flow", "mole", componentFlows);
            TrySetOverallProp(material, "totalFlow", "mole", new[] { aggregate.TotalMolarFlowMolS });

            if (TryCalcEquilibrium(materialObject))
            {
                UnitOperationComTrace.Write(
                    OperationName,
                    "flash",
                    $"CalcEquilibrium(TP); streams={string.Join(",", streams.Select(static stream => stream.Id))}");
                return;
            }

            PublishPresentPhases(material, aggregate, componentIds);
            UnitOperationComTrace.Write(
                OperationName,
                "manual-phases",
                $"streams={string.Join(",", streams.Select(static stream => stream.Id))}");
        }
        catch (CapeOpenException)
        {
            throw;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(OperationName, error);
            throw new CapeUnknownException(
                $"Failed to publish RadishFlow product material to the connected CAPE-OPEN material object: {error.Message}",
                new CapeOpenExceptionContext(
                    InterfaceName,
                    UnitScope,
                    OperationName,
                    MoreInfo: "The native solve succeeded, but the output material object could not be updated for the PME."));
        }
    }

    private static bool TryPublishCapeOpen10Material(
        object materialObject,
        IReadOnlyList<UnitOperationCalculationStream> streams)
    {
        if (TryCast<ICapeThermoMaterialObject>(materialObject) is not { } materialObject10)
        {
            UnitOperationComTrace.Write(OperationName, "skip", "No CAPE-OPEN 1.0 material object interface.");
            return false;
        }

        var aggregate = BoundaryMaterialStreamAggregate.Create(streams);
        var componentIds = GetComponentIds(materialObject, aggregate);
        var fractions = CreateCompositionVector(aggregate.OverallMoleFractions, componentIds);
        var componentFlows = fractions.Select(value => value * aggregate.TotalMolarFlowMolS).ToArray();

        if (!TrySetRequiredProp(materialObject10, "temperature", "Overall", null, "mixture", string.Empty, new[] { aggregate.TemperatureK }) ||
            !TrySetRequiredProp(materialObject10, "pressure", "Overall", null, "mixture", string.Empty, new[] { aggregate.PressurePa }) ||
            !TrySetRequiredProp(materialObject10, "fraction", "Overall", null, "mixture", "mole", fractions))
        {
            UnitOperationComTrace.Write(OperationName, "capeopen-10-fallback", "Required CAPE-OPEN 1.0 SetProp call failed; trying CAPE-OPEN 1.1 material interface.");
            return false;
        }

        TrySetProp(materialObject10, "flow", "Overall", null, "mixture", "mole", componentFlows);
        TrySetProp(materialObject10, "totalFlow", "Overall", null, "mixture", "mole", new[] { aggregate.TotalMolarFlowMolS });

        if (TryCalcCapeOpen10Equilibrium(materialObject10))
        {
            UnitOperationComTrace.Write(
                OperationName,
                "flash",
                $"CapeOpen10 CalcEquilibrium(TP); streams={string.Join(",", streams.Select(static stream => stream.Id))}");
            return true;
        }

        UnitOperationComTrace.Write(
            OperationName,
            "no-flash",
            $"CAPE-OPEN 1.0 material object was updated, but CalcEquilibrium(TP) failed; streams={string.Join(",", streams.Select(static stream => stream.Id))}");
        return false;
    }

    private static void TrySetOverallProp(ICapeThermoMaterial material, string property, string basis, double[] values)
    {
        try
        {
            material.SetOverallProp(property, basis, values);
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception($"{OperationName}.SetOverallProp.{property}", error);
        }
    }

    private static void TrySetProp(
        ICapeThermoMaterialObject materialObject,
        string property,
        string phase,
        object? compIds,
        string calcType,
        string basis,
        double[] values)
    {
        try
        {
            materialObject.SetProp(property, phase, compIds, calcType, basis, values);
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception($"{OperationName}.SetProp.{property}", error);
        }
    }

    private static bool TrySetRequiredProp(
        ICapeThermoMaterialObject materialObject,
        string property,
        string phase,
        object? compIds,
        string calcType,
        string basis,
        double[] values)
    {
        try
        {
            materialObject.SetProp(property, phase, compIds, calcType, basis, values);
            UnitOperationComTrace.Write($"{OperationName}.SetProp.{property}", "success", $"phase={phase}; basis={basis}; length={values.Length}");
            return true;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception($"{OperationName}.SetProp.{property}", error);
            return false;
        }
    }

    private static bool TryCalcEquilibrium(object materialObject)
    {
        if (TryCalcCapeOpen11Equilibrium(materialObject))
        {
            return true;
        }

        if (TryCast<ICapeThermoMaterialObject>(materialObject) is not { } materialObject10)
        {
            UnitOperationComTrace.Write($"{OperationName}.CalcEquilibrium", "skip", "No CAPE-OPEN 1.0 material object equilibrium interface.");
            return false;
        }

        return TryCalcCapeOpen10Equilibrium(materialObject10);
    }

    private static bool TryCalcCapeOpen10Equilibrium(ICapeThermoMaterialObject materialObject)
    {
        try
        {
            materialObject.CalcEquilibrium("TP", null);
            UnitOperationComTrace.Write($"{OperationName}.CalcEquilibrium", "capeopen-10", "flashType=TP");
            return true;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception($"{OperationName}.CalcEquilibrium.CapeOpen10", error);
            return false;
        }
    }

    private static bool TryCalcCapeOpen11Equilibrium(object materialObject)
    {
        if (TryCast<ICapeThermoEquilibriumRoutine>(materialObject) is not { } equilibriumRoutine)
        {
            UnitOperationComTrace.Write($"{OperationName}.CalcEquilibrium", "skip", "No CAPE-OPEN 1.1 equilibrium routine interface.");
            return false;
        }

        try
        {
            equilibriumRoutine.CalcEquilibrium(
                ["temperature", string.Empty, "Overall"],
                ["pressure", string.Empty, "Overall"],
                "unspecified");
            UnitOperationComTrace.Write($"{OperationName}.CalcEquilibrium", "capeopen-11", "specification=TP; solutionType=unspecified");
            return true;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception($"{OperationName}.CalcEquilibrium.CapeOpen11", error);
            return false;
        }
    }

    private static void PublishPresentPhases(
        ICapeThermoMaterial material,
        BoundaryMaterialStreamAggregate aggregate,
        IReadOnlyList<string> componentIds)
    {
        var phases = aggregate.Phases
            .Where(static phase => !IsOverallPhase(phase.Label))
            .ToArray();
        if (phases.Length == 0)
        {
            phases = [new BoundaryMaterialPhase("Vapor", 1.0d, aggregate.OverallMoleFractions)];
        }

        var labels = phases.Select(static phase => NormalizePhaseLabel(phase.Label)).ToArray();
        var statuses = Enumerable.Repeat(CapePhaseStatus.CAPE_ATEQUILIBRIUM, labels.Length).ToArray();
        material.SetPresentPhases(labels, statuses);

        foreach (var phase in phases)
        {
            var label = NormalizePhaseLabel(phase.Label);
            material.SetSinglePhaseProp("temperature", label, string.Empty, new[] { aggregate.TemperatureK });
            material.SetSinglePhaseProp("pressure", label, string.Empty, new[] { aggregate.PressurePa });
            material.SetSinglePhaseProp("phasefraction", label, "mole", new[] { phase.PhaseFraction });
            material.SetSinglePhaseProp("fraction", label, "mole", CreateCompositionVector(phase.MoleFractions, componentIds));
        }
    }

    private static IReadOnlyList<string> GetComponentIds(
        object materialObject,
        BoundaryMaterialStreamAggregate aggregate)
    {
        if (TryCast<ICapeThermoMaterialObject>(materialObject) is { } materialObject10)
        {
            try
            {
                var componentIds = materialObject10.ComponentIds;
                if (ConvertStringEnumerable(componentIds) is { Count: > 0 } convertedComponentIds)
                {
                    UnitOperationComTrace.Write(OperationName, "component-ids", string.Join(",", convertedComponentIds));
                    return convertedComponentIds;
                }
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception($"{OperationName}.ComponentIds", error);
            }
        }

        return aggregate.OverallMoleFractions.Keys
            .Order(StringComparer.OrdinalIgnoreCase)
            .ToArray();
    }

    private static IReadOnlyList<string>? ConvertStringEnumerable(object? value)
    {
        return value switch
        {
            null => null,
            string[] values => values,
            object[] values => values.OfType<string>().ToArray(),
            IEnumerable<string> values => values.ToArray(),
            Array values => values.Cast<object?>()
                .Select(static item => item?.ToString())
                .Where(static item => !string.IsNullOrWhiteSpace(item))
                .Select(static item => item!)
                .ToArray(),
            _ => null,
        };
    }

    private static double[] CreateCompositionVector(
        IReadOnlyDictionary<string, double> moleFractions,
        IReadOnlyList<string> componentIds)
    {
        var values = componentIds
            .Select(componentId => TryGetCompositionValue(moleFractions, componentId))
            .ToArray();
        var sum = values.Sum();
        if (sum <= 0.0d || !double.IsFinite(sum))
        {
            return values;
        }

        return values.Select(value => value / sum).ToArray();
    }

    private static double TryGetCompositionValue(
        IReadOnlyDictionary<string, double> moleFractions,
        string componentId)
    {
        if (moleFractions.TryGetValue(componentId, out var value))
        {
            return value;
        }

        foreach (var (key, candidate) in moleFractions)
        {
            if (string.Equals(key, componentId, StringComparison.OrdinalIgnoreCase))
            {
                return candidate;
            }
        }

        return 0.0d;
    }

    private static string NormalizePhaseLabel(string label)
    {
        return label.Trim().ToLowerInvariant() switch
        {
            "vapor" or "vapour" => "Vapor",
            "liquid" => "Liquid",
            "liquid2" => "Liquid2",
            "solid" => "Solid",
            _ => label,
        };
    }

    private static bool IsOverallPhase(string label)
    {
        return string.Equals(label, "overall", StringComparison.OrdinalIgnoreCase);
    }

    private static TInterface? TryCast<TInterface>(object value)
        where TInterface : class
    {
        try
        {
            return value as TInterface;
        }
        catch (COMException error)
        {
            UnitOperationComTrace.Exception($"{OperationName}.QueryInterface.{typeof(TInterface).Name}", error);
            return null;
        }
    }

    private sealed record BoundaryMaterialStreamAggregate(
        double TemperatureK,
        double PressurePa,
        double TotalMolarFlowMolS,
        IReadOnlyDictionary<string, double> OverallMoleFractions,
        IReadOnlyList<BoundaryMaterialPhase> Phases)
    {
        public static BoundaryMaterialStreamAggregate Create(
            IReadOnlyList<UnitOperationCalculationStream> streams)
        {
            var totalFlow = streams.Sum(static stream => Math.Max(stream.TotalMolarFlowMolS, 0.0d));
            if (totalFlow <= 0.0d)
            {
                totalFlow = 1.0d;
            }

            var temperature = streams.Sum(stream => stream.TemperatureK * Math.Max(stream.TotalMolarFlowMolS, 0.0d)) / totalFlow;
            var pressure = streams.Sum(stream => stream.PressurePa * Math.Max(stream.TotalMolarFlowMolS, 0.0d)) / totalFlow;
            var composition = CombineCompositions(
                streams.Select(stream => (Weight: Math.Max(stream.TotalMolarFlowMolS, 0.0d), Composition: stream.OverallMoleFractions)),
                totalFlow);
            var phases = streams
                .SelectMany(stream => CreatePhases(stream, totalFlow))
                .GroupBy(static phase => NormalizePhaseLabel(phase.Label), StringComparer.OrdinalIgnoreCase)
                .Select(group => CombinePhaseGroup(group, totalFlow))
                .ToArray();

            return new BoundaryMaterialStreamAggregate(
                temperature,
                pressure,
                streams.Sum(static stream => stream.TotalMolarFlowMolS),
                composition,
                phases);
        }

        private static IEnumerable<BoundaryMaterialPhase> CreatePhases(
            UnitOperationCalculationStream stream,
            double aggregateTotalFlow)
        {
            var nonOverallPhases = stream.Phases
                .Where(static phase => !IsOverallPhase(phase.Label))
                .ToArray();
            if (nonOverallPhases.Length == 0)
            {
                yield return new BoundaryMaterialPhase(
                    "Vapor",
                    Math.Max(stream.TotalMolarFlowMolS, 0.0d) / aggregateTotalFlow,
                    stream.OverallMoleFractions);
                yield break;
            }

            foreach (var phase in nonOverallPhases)
            {
                yield return new BoundaryMaterialPhase(
                    phase.Label,
                    Math.Max(stream.TotalMolarFlowMolS, 0.0d) * phase.PhaseFraction / aggregateTotalFlow,
                    phase.MoleFractions.Count > 0 ? phase.MoleFractions : stream.OverallMoleFractions);
            }
        }

        private static BoundaryMaterialPhase CombinePhaseGroup(
            IEnumerable<BoundaryMaterialPhase> phases,
            double aggregateTotalFlow)
        {
            var phaseArray = phases.ToArray();
            var totalPhaseFraction = phaseArray.Sum(static phase => Math.Max(phase.PhaseFraction, 0.0d));
            var composition = CombineCompositions(
                phaseArray.Select(static phase => (Weight: Math.Max(phase.PhaseFraction, 0.0d), Composition: phase.MoleFractions)),
                totalPhaseFraction <= 0.0d ? 1.0d : totalPhaseFraction);

            return new BoundaryMaterialPhase(
                NormalizePhaseLabel(phaseArray[0].Label),
                totalPhaseFraction,
                composition);
        }

        private static IReadOnlyDictionary<string, double> CombineCompositions(
            IEnumerable<(double Weight, IReadOnlyDictionary<string, double> Composition)> weightedCompositions,
            double totalWeight)
        {
            var combined = new Dictionary<string, double>(StringComparer.OrdinalIgnoreCase);
            foreach (var (weight, composition) in weightedCompositions)
            {
                foreach (var (componentId, fraction) in composition)
                {
                    combined[componentId] = combined.GetValueOrDefault(componentId) + (weight * fraction / totalWeight);
                }
            }

            return combined;
        }
    }

    private sealed record BoundaryMaterialPhase(
        string Label,
        double PhaseFraction,
        IReadOnlyDictionary<string, double> MoleFractions);
}
