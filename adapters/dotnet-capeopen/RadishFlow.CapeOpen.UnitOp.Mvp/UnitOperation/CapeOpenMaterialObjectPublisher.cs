using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Thermo;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using System.Diagnostics;
using System.Globalization;
using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal static class CapeOpenMaterialObjectPublisher
{
    private const string InterfaceName = "ICapeThermoMaterial";
    private const string OperationName = "PublishProductMaterial";
    private const string UnitScope = "RadishFlow.CapeOpen.UnitOp.Mvp";
    private const int CapeUnknownPhaseStatus = (int)CapePhaseStatus.CAPE_UNKNOWNPHASESTATUS;
    private const int CapeAtEquilibriumPhaseStatus = (int)CapePhaseStatus.CAPE_ATEQUILIBRIUM;

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
            if (TryPublishCapeOpen11Material(materialObject, streams) ||
                TryPublishCapeOpen10Material(materialObject, streams))
            {
                return;
            }

            UnitOperationComTrace.Write(
                OperationName,
                "skip",
                $"Connected output object `{materialObject.GetType().FullName}` does not expose {nameof(ICapeThermoMaterial)} or {nameof(ICapeThermoMaterialObject)}.");
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

    private static bool TryPublishCapeOpen11Material(
        object materialObject,
        IReadOnlyList<UnitOperationCalculationStream> streams)
    {
        if (TryCast<ICapeThermoMaterial>(materialObject) is not { } material)
        {
            UnitOperationComTrace.Write(
                OperationName,
                "skip",
                "No CAPE-OPEN 1.1 material interface.");
            return false;
        }

        UnitOperationComTrace.Write(OperationName, "capeopen-11", "Connected output object exposes ICapeThermoMaterial.");

        var aggregate = BoundaryMaterialStreamAggregate.Create(streams);
        var components = GetComponentDescriptors(materialObject, aggregate);
        var componentIds = components.Select(static component => component.Id).ToArray();
        var fractions = CreateCompositionVector(aggregate.OverallMoleFractions, components, "overall-capeopen-11");
        var componentFlows = fractions.Select(value => value * aggregate.TotalMolarFlowMolS).ToArray();

        TryClearAllProps(material);
        if (!TrySetRequiredOverallProp(material, CreateOverallPropAttempts("fraction", "Mole", fractions)) ||
            !TrySetRequiredOverallProp(material, CreateOverallPropAttempts("totalFlow", "Mole", aggregate.TotalMolarFlowMolS)) ||
            !TrySetRequiredOverallProp(material, CreateOverallPropAttempts("temperature", string.Empty, aggregate.TemperatureK)) ||
            !TrySetRequiredOverallProp(material, CreateOverallPropAttempts("pressure", string.Empty, aggregate.PressurePa)))
        {
            UnitOperationComTrace.Write(OperationName, "capeopen-11-fallback", "Required CAPE-OPEN 1.1 material publication failed; trying CAPE-OPEN 1.0 material object interface.");
            return false;
        }

        TrySetOverallProp(material, CreateOverallPropAttempts("flow", "Mole", componentFlows));

        if (IsCofeHostProcess())
        {
            PublishEquilibriumCandidatePhases(material, aggregate);
            if (TryCalcCapeOpen11Equilibrium(materialObject))
            {
                TracePresentPhasesReadback(material);
                UnitOperationComTrace.Write(
                    OperationName,
                    "flash-cofe-capeopen-11",
                    "COFE accepted CAPE-OPEN 1.1 CalcEquilibrium(TP); skipped CAPE-OPEN 1.0 equilibrium fallback.");
                return true;
            }

            PublishPresentPhases(material, aggregate, components);
            TracePresentPhasesReadback(material);
            UnitOperationComTrace.Write(
                OperationName,
                "manual-phases-cofe",
                "COFE accepted overall material properties; CAPE-OPEN 1.1 equilibrium was unavailable or failed, so published present phases manually and skipped the unsafe CAPE-OPEN 1.0 equilibrium fallback.");
            return true;
        }

        if (TryCalcEquilibrium(materialObject))
        {
            UnitOperationComTrace.Write(
                OperationName,
                "flash",
                $"CalcEquilibrium(TP); streams={string.Join(",", streams.Select(static stream => stream.Id))}");
            return true;
        }

        PublishPresentPhases(material, aggregate, components);
        UnitOperationComTrace.Write(
            OperationName,
            "manual-phases",
            $"streams={string.Join(",", streams.Select(static stream => stream.Id))}");
        return true;
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
        var components = GetComponentDescriptors(materialObject, aggregate);
        var componentIds = components.Select(static component => component.Id).ToArray();
        var fractions = CreateCompositionVector(aggregate.OverallMoleFractions, components, "overall-capeopen-10");
        var componentFlows = fractions.Select(value => value * aggregate.TotalMolarFlowMolS).ToArray();

        if (!TrySetRequiredProp(materialObject10, CreateOverallVectorPropertyAttempts("fraction", "Mole", fractions, componentIds)))
        {
            UnitOperationComTrace.Write(OperationName, "capeopen-10-failed", "Required CAPE-OPEN 1.0 overall composition publication failed.");
            throw new CapeUnknownException(
                "Failed to publish RadishFlow product material composition through the connected CAPE-OPEN 1.0 material object.",
                new CapeOpenExceptionContext(
                    InterfaceName,
                    UnitScope,
                    OperationName,
                    MoreInfo: "The output material object exposed ICapeThermoMaterialObject, but rejected all traced SetProp argument shapes for the required overall fraction property."));
        }

        if (!TrySetRequiredProp(materialObject10, CreateOverallScalarPropertyAttempts("totalFlow", "Mole", aggregate.TotalMolarFlowMolS)) ||
            (!TrySetIndependentVariables(materialObject10, aggregate.TemperatureK, aggregate.PressurePa) &&
             (!TrySetRequiredProp(materialObject10, CreateStatePropertyAttempts("temperature", aggregate.TemperatureK)) ||
              !TrySetRequiredProp(materialObject10, CreateStatePropertyAttempts("pressure", aggregate.PressurePa)))))
        {
            UnitOperationComTrace.Write(OperationName, "capeopen-10-failed", "Required CAPE-OPEN 1.0 state publication failed after composition and total flow were accepted.");
            throw new CapeUnknownException(
                "Failed to publish RadishFlow product material state variables through the connected CAPE-OPEN 1.0 material object.",
                new CapeOpenExceptionContext(
                    InterfaceName,
                    UnitScope,
                    OperationName,
                    MoreInfo: "The output material object exposed ICapeThermoMaterialObject, accepted composition publishing, but rejected both SetIndependentVar and SetProp for temperature or pressure."));
        }

        TrySetProp(materialObject10, CreateOverallVectorPropertyAttempts("flow", "Mole", componentFlows, componentIds));

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

    private static void TryClearAllProps(ICapeThermoMaterial material)
    {
        const string memberName = $"{OperationName}.ClearAllProps";
        UnitOperationComTrace.Write(memberName, "attempt", string.Empty);
        try
        {
            material.ClearAllProps();
            UnitOperationComTrace.Write(memberName, "success", string.Empty);
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(memberName, error);
        }
    }

    private static void TrySetOverallProp(
        ICapeThermoMaterial material,
        IReadOnlyList<CapeOpen11OverallPropAttempt> attempts)
    {
        foreach (var attempt in attempts)
        {
            if (TrySetOverallPropAttempt(material, attempt))
            {
                return;
            }
        }
    }

    private static bool TrySetRequiredOverallProp(
        ICapeThermoMaterial material,
        IReadOnlyList<CapeOpen11OverallPropAttempt> attempts)
    {
        foreach (var attempt in attempts)
        {
            if (TrySetOverallPropAttempt(material, attempt))
            {
                return true;
            }
        }

        return false;
    }

    private static bool TrySetOverallPropAttempt(
        ICapeThermoMaterial material,
        CapeOpen11OverallPropAttempt attempt)
    {
        var memberName = $"{OperationName}.SetOverallProp.{attempt.Property}";
        UnitOperationComTrace.Write(memberName, "attempt", attempt.Describe());
        try
        {
            material.SetOverallProp(attempt.Property, attempt.Basis, attempt.Values);
            UnitOperationComTrace.Write(memberName, "success", attempt.Describe());
            return true;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(memberName, error);
            return false;
        }
    }

    private static void TrySetProp(
        ICapeThermoMaterialObject materialObject,
        IReadOnlyList<CapeOpen10SetPropAttempt> attempts)
    {
        foreach (var attempt in attempts)
        {
            if (TrySetPropAttempt(materialObject, attempt))
            {
                return;
            }
        }
    }

    private static bool TrySetRequiredProp(
        ICapeThermoMaterialObject materialObject,
        IReadOnlyList<CapeOpen10SetPropAttempt> attempts)
    {
        foreach (var attempt in attempts)
        {
            if (TrySetPropAttempt(materialObject, attempt))
            {
                return true;
            }
        }

        return false;
    }

    private static bool TrySetIndependentVariables(
        ICapeThermoMaterialObject materialObject,
        double temperatureK,
        double pressurePa)
    {
        var attempts = new[]
        {
            new CapeOpen10IndependentVarAttempt(
                "lowercase-temperature-pressure",
                ["temperature", "pressure"],
                [temperatureK, pressurePa]),
            new CapeOpen10IndependentVarAttempt(
                "titlecase-temperature-pressure",
                ["Temperature", "Pressure"],
                [temperatureK, pressurePa]),
            new CapeOpen10IndependentVarAttempt(
                "short-t-p",
                ["T", "P"],
                [temperatureK, pressurePa]),
        };

        foreach (var attempt in attempts)
        {
            var memberName = $"{OperationName}.SetIndependentVar";
            UnitOperationComTrace.Write(memberName, "attempt", attempt.Describe());
            try
            {
                materialObject.SetIndependentVar(attempt.IndependentVariables, attempt.Values);
                UnitOperationComTrace.Write(memberName, "success", attempt.Describe());
                return true;
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception(memberName, error);
            }
        }

        return false;
    }

    private static bool TrySetPropAttempt(
        ICapeThermoMaterialObject materialObject,
        CapeOpen10SetPropAttempt attempt)
    {
        var memberName = $"{OperationName}.SetProp.{attempt.Property}";
        UnitOperationComTrace.Write(memberName, "attempt", attempt.Describe());
        try
        {
            materialObject.SetProp(
                attempt.Property,
                attempt.Phase,
                attempt.ComponentIds,
                attempt.CalcType,
                attempt.Basis,
                attempt.Values);
            UnitOperationComTrace.Write(memberName, "success", attempt.Describe());
            return true;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(memberName, error);
            return false;
        }
    }

    private static IReadOnlyList<CapeOpen10SetPropAttempt> CreateStatePropertyAttempts(
        string property,
        double value)
    {
        var values = new[] { value };
        return
        [
            new CapeOpen10SetPropAttempt($"{property}-null-qualifiers", property, null, null, null, null, values),
            new CapeOpen10SetPropAttempt($"{property}-overall-null-qualifiers", property, "Overall", null, null, null, values),
            new CapeOpen10SetPropAttempt($"{property}-overall-empty-qualifiers", property, "Overall", null, string.Empty, string.Empty, values),
        ];
    }

    private static IReadOnlyList<CapeOpen11OverallPropAttempt> CreateOverallPropAttempts(
        string property,
        string basis,
        double value)
    {
        return CreateOverallPropAttempts(property, basis, [value]);
    }

    private static IReadOnlyList<CapeOpen11OverallPropAttempt> CreateOverallPropAttempts(
        string property,
        string basis,
        double[] values)
    {
        if (!string.IsNullOrEmpty(basis))
        {
            return [new CapeOpen11OverallPropAttempt($"{property}-basis-{basis}", property, basis, values)];
        }

        return
        [
            new CapeOpen11OverallPropAttempt($"{property}-empty-basis", property, string.Empty, values),
            new CapeOpen11OverallPropAttempt($"{property}-UNDEFINED-basis", property, "UNDEFINED", values),
            new CapeOpen11OverallPropAttempt($"{property}-undefined-basis", property, "undefined", values),
            new CapeOpen11OverallPropAttempt($"{property}-null-basis", property, null, values),
        ];
    }

    private static IReadOnlyList<CapeOpen10SetPropAttempt> CreateOverallVectorPropertyAttempts(
        string property,
        string basis,
        double[] values,
        string[] componentIds)
    {
        return
        [
            new CapeOpen10SetPropAttempt($"{property}-component-ids-null-calc", property, "Overall", componentIds, null, basis, values),
            new CapeOpen10SetPropAttempt($"{property}-all-components-null-calc", property, "Overall", null, null, basis, values),
            new CapeOpen10SetPropAttempt($"{property}-component-ids-mixture", property, "Overall", componentIds, "Mixture", basis, values),
            new CapeOpen10SetPropAttempt($"{property}-all-components-mixture", property, "Overall", null, "Mixture", basis, values),
        ];
    }

    private static IReadOnlyList<CapeOpen10SetPropAttempt> CreateOverallScalarPropertyAttempts(
        string property,
        string basis,
        double value)
    {
        var values = new[] { value };
        return
        [
            new CapeOpen10SetPropAttempt($"{property}-overall-null-calc", property, "Overall", null, null, basis, values),
            new CapeOpen10SetPropAttempt($"{property}-overall-mixture", property, "Overall", null, "Mixture", basis, values),
        ];
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
            string[] temperatureSpec = ["temperature", string.Empty, "Overall"];
            string[] pressureSpec = ["pressure", string.Empty, "Overall"];
            equilibriumRoutine.CalcEquilibrium(
                temperatureSpec,
                pressureSpec,
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
        IReadOnlyList<CapeOpenComponentDescriptor> components)
    {
        var phases = aggregate.Phases
            .Where(static phase => !IsOverallPhase(phase.Label))
            .Where(static phase => IsPublishablePhase(phase))
            .ToArray();
        if (phases.Length == 0)
        {
            phases = [new BoundaryMaterialPhase("Liquid", 1.0d, aggregate.OverallMoleFractions)];
        }
        else
        {
            phases = NormalizePublishedPhaseFractions(phases);
        }

        var labels = phases.Select(static phase => NormalizePhaseLabel(phase.Label)).ToArray();
        var statuses = Enumerable.Repeat(CapeAtEquilibriumPhaseStatus, labels.Length).ToArray();
        SetPresentPhases(material, labels, statuses);

        foreach (var phase in phases)
        {
            var label = NormalizePhaseLabel(phase.Label);
            SetSinglePhaseProp(material, "temperature", label, string.Empty, [aggregate.TemperatureK]);
            SetSinglePhaseProp(material, "pressure", label, string.Empty, [aggregate.PressurePa]);
            SetSinglePhaseProp(material, "phaseFraction", label, "Mole", [phase.PhaseFraction]);
            SetSinglePhaseProp(material, "fraction", label, "Mole", CreateCompositionVector(phase.MoleFractions, components, $"phase-{label}"));
        }
    }

    private static void PublishEquilibriumCandidatePhases(
        ICapeThermoMaterial material,
        BoundaryMaterialStreamAggregate aggregate)
    {
        var labels = aggregate.Phases
            .Where(static phase => !IsOverallPhase(phase.Label))
            .Select(static phase => NormalizePhaseLabel(phase.Label))
            .Where(static label => !string.IsNullOrWhiteSpace(label))
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .ToArray();
        if (labels.Length == 0)
        {
            labels = ["Liquid", "Vapor"];
        }

        var statuses = Enumerable.Repeat(CapeUnknownPhaseStatus, labels.Length).ToArray();
        SetPresentPhases(material, labels, statuses);
    }

    private static void SetPresentPhases(
        ICapeThermoMaterial material,
        string[] labels,
        int[] statuses)
    {
        const string memberName = $"{OperationName}.SetPresentPhases";
        var detail = $"labels={FormatStringList(labels)}; statuses={string.Join(",", statuses)}";
        UnitOperationComTrace.Write(memberName, "attempt", detail);
        material.SetPresentPhases(labels, statuses);
        UnitOperationComTrace.Write(memberName, "success", detail);
    }

    private static void SetSinglePhaseProp(
        ICapeThermoMaterial material,
        string property,
        string phaseLabel,
        string basis,
        double[] values)
    {
        var memberName = $"{OperationName}.SetSinglePhaseProp.{property}";
        var detail = string.Join(
            "; ",
            $"phase={phaseLabel}",
            $"basis=`{basis}`",
            $"values={values.GetType().FullName}[{values.Length}]");
        UnitOperationComTrace.Write(memberName, "attempt", detail);
        material.SetSinglePhaseProp(property, phaseLabel, basis, values);
        UnitOperationComTrace.Write(memberName, "success", detail);
    }

    private static void TracePresentPhasesReadback(ICapeThermoMaterial material)
    {
        const string memberName = $"{OperationName}.GetPresentPhases";
        try
        {
            object? phaseLabels = null;
            object? phaseStatus = null;
            material.GetPresentPhases(ref phaseLabels, ref phaseStatus);
            var labels = ConvertStringEnumerable(phaseLabels) ?? [];
            UnitOperationComTrace.Write(
                memberName,
                "readback",
                $"labels={FormatStringList(labels)}; statuses={FormatVariantValue(phaseStatus)}");

            foreach (var label in labels)
            {
                TraceSinglePhasePropReadback(material, label, "temperature", string.Empty);
                TraceSinglePhasePropReadback(material, label, "pressure", string.Empty);
                TraceSinglePhasePropReadback(material, label, "phaseFraction", "Mole");
                TraceSinglePhasePropReadback(material, label, "fraction", "Mole");
            }
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(memberName, error);
        }
    }

    private static void TraceSinglePhasePropReadback(
        ICapeThermoMaterial material,
        string phaseLabel,
        string property,
        string basis)
    {
        var memberName = $"{OperationName}.GetSinglePhaseProp.{property}";
        try
        {
            object? results = null;
            material.GetSinglePhaseProp(property, phaseLabel, basis, ref results);
            UnitOperationComTrace.Write(
                memberName,
                "readback",
                $"phase={phaseLabel}; basis=`{basis}`; values={FormatVariantValue(results)}");
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(memberName, error);
        }
    }

    private static bool IsPublishablePhase(BoundaryMaterialPhase phase)
    {
        if (phase.PhaseFraction <= 1.0e-12d || !double.IsFinite(phase.PhaseFraction))
        {
            return false;
        }

        var compositionSum = phase.MoleFractions.Values
            .Where(double.IsFinite)
            .Where(static value => value > 0.0d)
            .Sum();
        return compositionSum > 1.0e-12d;
    }

    private static BoundaryMaterialPhase[] NormalizePublishedPhaseFractions(
        IReadOnlyList<BoundaryMaterialPhase> phases)
    {
        var sum = phases.Sum(static phase => Math.Max(phase.PhaseFraction, 0.0d));
        if (sum <= 0.0d || !double.IsFinite(sum))
        {
            return phases.ToArray();
        }

        return phases
            .Select(phase => phase with { PhaseFraction = phase.PhaseFraction / sum })
            .ToArray();
    }

    private static IReadOnlyList<CapeOpenComponentDescriptor> GetComponentDescriptors(
        object materialObject,
        BoundaryMaterialStreamAggregate aggregate)
    {
        if (TryCast<ICapeThermoCompounds>(materialObject) is { } compounds)
        {
            try
            {
                object? compIds = null;
                object? formulae = null;
                object? names = null;
                object? boilTemps = null;
                object? molwts = null;
                object? casnos = null;

                compounds.GetCompoundList(ref compIds, ref formulae, ref names, ref boilTemps, ref molwts, ref casnos);

                if (ConvertStringEnumerable(compIds) is { Count: > 0 } convertedComponentIds)
                {
                    var convertedFormulae = ConvertStringEnumerable(formulae) ?? [];
                    var convertedNames = ConvertStringEnumerable(names) ?? [];
                    var convertedCasNos = ConvertStringEnumerable(casnos) ?? [];
                    var descriptors = convertedComponentIds
                        .Select((componentId, index) => new CapeOpenComponentDescriptor(
                            componentId,
                            GetOptionalString(convertedFormulae, index),
                            GetOptionalString(convertedNames, index),
                            GetOptionalString(convertedCasNos, index)))
                        .ToArray();

                    UnitOperationComTrace.Write(
                        OperationName,
                        "compound-list",
                        string.Join(
                            "; ",
                            "source=capeopen-11",
                            $"compIds={FormatStringList(convertedComponentIds)}",
                            $"names={FormatStringList(convertedNames)}",
                            $"formulae={FormatStringList(convertedFormulae)}",
                            $"casNos={FormatStringList(convertedCasNos)}"));
                    return descriptors;
                }

                UnitOperationComTrace.Write(
                    OperationName,
                    "compound-list-empty",
                    $"source=capeopen-11; compIds={DescribeObject(compIds)}");
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception($"{OperationName}.GetCompoundList", error);
            }
        }

        if (TryCast<ICapeThermoMaterialObject>(materialObject) is { } materialObject10)
        {
            try
            {
                var componentIds = materialObject10.ComponentIds;
                if (ConvertStringEnumerable(componentIds) is { Count: > 0 } convertedComponentIds)
                {
                    UnitOperationComTrace.Write(
                        OperationName,
                        "component-ids",
                        $"source=capeopen-10; compIds={FormatStringList(convertedComponentIds)}");
                    return convertedComponentIds
                        .Select(static componentId => new CapeOpenComponentDescriptor(componentId, null, null, null))
                        .ToArray();
                }
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception($"{OperationName}.ComponentIds", error);
            }
        }

        var fallbackIds = aggregate.OverallMoleFractions.Keys
            .Order(StringComparer.OrdinalIgnoreCase)
            .ToArray();
        UnitOperationComTrace.Write(
            OperationName,
            "component-ids",
            $"source=radishflow-output-fallback; compIds={FormatStringList(fallbackIds)}");
        return fallbackIds
            .Select(static componentId => new CapeOpenComponentDescriptor(componentId, null, null, null))
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
        IReadOnlyList<CapeOpenComponentDescriptor> components,
        string traceContext)
    {
        var mappings = components
            .Select(component => TryGetCompositionValue(moleFractions, component))
            .ToArray();
        var values = mappings.Select(static mapping => mapping.Value).ToArray();
        var sum = values.Sum();
        var normalized = sum <= 0.0d || !double.IsFinite(sum)
            ? values
            : values.Select(value => value / sum).ToArray();
        var nativeComponentIds = moleFractions.Keys.ToArray();
        var targetComponentIds = components.Select(static component => component.Id).ToArray();
        var missingComponentIds = mappings
            .Where(static mapping => mapping.SourceKey is null)
            .Select(static mapping => mapping.ComponentId)
            .ToArray();

        UnitOperationComTrace.Write(
            $"{OperationName}.Composition",
            traceContext,
            string.Join(
                "; ",
                $"radishflowKeys={FormatStringList(nativeComponentIds)}",
                $"targetCompIds={FormatStringList(targetComponentIds)}",
                $"matches={FormatCompositionMatches(mappings)}",
                $"missing={FormatStringList(missingComponentIds)}",
                $"raw={FormatDoubleList(values)}",
                $"normalized={FormatDoubleList(normalized)}",
                $"sum={FormatDouble(sum)}"));

        if (components.Count > 0 && mappings.All(static mapping => mapping.SourceKey is null))
        {
            UnitOperationComTrace.Write(
                $"{OperationName}.Composition",
                "mismatch",
                $"context={traceContext}; radishflowKeys={FormatStringList(nativeComponentIds)}; targetCompIds={FormatStringList(targetComponentIds)}");
            throw CreateCompositionMismatchException(nativeComponentIds, targetComponentIds);
        }

        return normalized;
    }

    private static CapeInvalidArgumentException CreateCompositionMismatchException(
        IReadOnlyList<string> nativeComponentIds,
        IReadOnlyList<string> targetComponentIds)
    {
        return new CapeInvalidArgumentException(
            "Cannot publish RadishFlow product material composition because the connected CAPE-OPEN material object compounds do not overlap with the native output composition.",
            new CapeOpenExceptionContext(
                InterfaceName,
                UnitScope,
                OperationName,
                MoreInfo: string.Join(
                    " ",
                    $"Native output compounds: {FormatStringList(nativeComponentIds)}.",
                    $"Connected product material compounds: {FormatStringList(targetComponentIds)}.",
                    "Configure the PME material stream/property package to use the same compounds as the RadishFlow flowsheet, or configure the RadishFlow flowsheet/property package to produce compounds present in the PME material stream.")));
    }

    private static CompositionMapping TryGetCompositionValue(
        IReadOnlyDictionary<string, double> moleFractions,
        CapeOpenComponentDescriptor component)
    {
        foreach (var alias in component.Aliases())
        {
            if (moleFractions.TryGetValue(alias, out var value))
            {
                return new CompositionMapping(component.Id, alias, value);
            }
        }

        foreach (var alias in component.Aliases())
        {
            foreach (var (key, candidate) in moleFractions)
            {
                if (string.Equals(key, alias, StringComparison.OrdinalIgnoreCase))
                {
                    return new CompositionMapping(component.Id, key, candidate);
                }
            }
        }

        var normalizedAliases = component.Aliases()
            .Select(NormalizeCompositionIdentifier)
            .Where(static alias => alias.Length > 0)
            .ToArray();
        foreach (var (key, candidate) in moleFractions)
        {
            var normalizedKey = NormalizeCompositionIdentifier(key);
            if (normalizedAliases.Contains(normalizedKey, StringComparer.Ordinal))
            {
                return new CompositionMapping(component.Id, key, candidate);
            }
        }

        return new CompositionMapping(component.Id, null, 0.0d);
    }

    private static string NormalizeCompositionIdentifier(string value)
    {
        return new string(value
            .Where(char.IsLetterOrDigit)
            .Select(char.ToLowerInvariant)
            .ToArray());
    }

    private static string? GetOptionalString(IReadOnlyList<string> values, int index)
    {
        if (index < 0 || index >= values.Count)
        {
            return null;
        }

        return string.IsNullOrWhiteSpace(values[index]) ? null : values[index];
    }

    private static string FormatStringList(IEnumerable<string> values)
    {
        var array = values
            .Where(static value => !string.IsNullOrWhiteSpace(value))
            .ToArray();
        return array.Length == 0 ? "<empty>" : string.Join(",", array);
    }

    private static string FormatDoubleList(IEnumerable<double> values)
    {
        var array = values.Select(FormatDouble).ToArray();
        return array.Length == 0 ? "<empty>" : string.Join(",", array);
    }

    private static string FormatDouble(double value)
    {
        return value.ToString("G17", CultureInfo.InvariantCulture);
    }

    private static string FormatCompositionMatches(IEnumerable<CompositionMapping> mappings)
    {
        var array = mappings
            .Select(static mapping => mapping.SourceKey is null
                ? $"{mapping.ComponentId}=<missing>"
                : $"{mapping.ComponentId}<={mapping.SourceKey}")
            .ToArray();
        return array.Length == 0 ? "<empty>" : string.Join(",", array);
    }

    private static string DescribeObject(object? value)
    {
        return value switch
        {
            null => "<null>",
            Array array => $"{value.GetType().FullName}[{array.Length}]",
            _ => value.GetType().FullName ?? value.GetType().Name,
        };
    }

    private static string FormatVariantValue(object? value)
    {
        if (value is null)
        {
            return "<null>";
        }

        if (value is double[] doubleArray)
        {
            return $"{value.GetType().FullName}[{doubleArray.Length}]={FormatDoubleList(doubleArray)}";
        }

        if (value is string[] stringArray)
        {
            return $"{value.GetType().FullName}[{stringArray.Length}]={FormatStringList(stringArray)}";
        }

        if (value is Array array)
        {
            var items = array.Cast<object?>()
                .Select(static item => item?.ToString() ?? "<null>")
                .ToArray();
            return $"{value.GetType().FullName}[{array.Length}]={FormatStringList(items)}";
        }

        return value.ToString() ?? DescribeObject(value);
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

    private static bool IsCofeHostProcess()
    {
        try
        {
            return string.Equals(
                Process.GetCurrentProcess().ProcessName,
                "COFE",
                StringComparison.OrdinalIgnoreCase);
        }
        catch
        {
            return false;
        }
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

    private sealed record CapeOpenComponentDescriptor(
        string Id,
        string? Formula,
        string? Name,
        string? CasNo)
    {
        public IEnumerable<string> Aliases()
        {
            yield return Id;

            if (!string.IsNullOrWhiteSpace(Name))
            {
                yield return Name;
            }

            if (!string.IsNullOrWhiteSpace(CasNo))
            {
                yield return CasNo;
            }

            if (!string.IsNullOrWhiteSpace(Formula))
            {
                yield return Formula;
            }
        }
    }

    private sealed record CompositionMapping(
        string ComponentId,
        string? SourceKey,
        double Value);

    private sealed record CapeOpen10SetPropAttempt(
        string Name,
        string Property,
        string? Phase,
        object? ComponentIds,
        string? CalcType,
        string? Basis,
        double[] Values)
    {
        public string Describe()
        {
            return string.Join(
                "; ",
                $"name={Name}",
                $"phase={DescribeNullableString(Phase)}",
                $"calcType={DescribeNullableString(CalcType)}",
                $"basis={DescribeNullableString(Basis)}",
                $"compIds={DescribeObject(ComponentIds)}",
                $"values={Values.GetType().FullName}[{Values.Length}]");
        }

        private static string DescribeNullableString(string? value)
        {
            return value is null ? "<null>" : $"`{value}`";
        }

        private static string DescribeObject(object? value)
        {
            return value switch
            {
                null => "<null>",
                Array array => $"{value.GetType().FullName}[{array.Length}]",
                _ => value.GetType().FullName ?? value.GetType().Name,
            };
        }
    }

    private sealed record CapeOpen11OverallPropAttempt(
        string Name,
        string Property,
        string? Basis,
        double[] Values)
    {
        public string Describe()
        {
            return string.Join(
                "; ",
                $"name={Name}",
                $"basis={DescribeNullableString(Basis)}",
                $"values={Values.GetType().FullName}[{Values.Length}]");
        }

        private static string DescribeNullableString(string? value)
        {
            return value is null ? "<null>" : $"`{value}`";
        }
    }

    private sealed record CapeOpen10IndependentVarAttempt(
        string Name,
        string[] IndependentVariables,
        double[] Values)
    {
        public string Describe()
        {
            return string.Join(
                "; ",
                $"name={Name}",
                $"indVars={string.Join(",", IndependentVariables)}",
                $"values={Values.GetType().FullName}[{Values.Length}]");
        }
    }
}
