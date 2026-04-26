using RadishFlow.CapeOpen.Interop.Thermo;
using System.Globalization;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal static class CapeOpenFeedMaterialReader
{
    private const string OperationName = "ReadFeedMaterial";

    public static CapeOpenFeedMaterialSnapshot? TryRead(object? materialObject)
    {
        if (materialObject is null)
        {
            UnitOperationComTrace.Write(OperationName, "skip", "No connected feed material object is available.");
            return null;
        }

        if (TryCast<ICapeThermoMaterial>(materialObject) is not { } material)
        {
            UnitOperationComTrace.Write(OperationName, "skip", "Connected feed object does not expose ICapeThermoMaterial.");
            return null;
        }

        var components = TryReadComponents(materialObject);
        if (components.Count == 0)
        {
            UnitOperationComTrace.Write(OperationName, "skip", "Connected feed material did not expose a compound list.");
            return null;
        }

        var (temperatureK, pressurePa, tpFractions) = TryGetOverallTPFraction(material);
        var fractions = tpFractions ?? TryGetOverallVectorProp(material, "fraction", ["Mole", null, string.Empty]);
        var totalFlow = TryGetOverallScalarProp(material, "totalFlow", ["Mole", null, string.Empty]);

        if (temperatureK is null || pressurePa is null || totalFlow is null || fractions is null)
        {
            UnitOperationComTrace.Write(
                OperationName,
                "skip",
                string.Join(
                    "; ",
                    $"temperature={(temperatureK is null ? "<missing>" : FormatDouble(temperatureK.Value))}",
                    $"pressure={(pressurePa is null ? "<missing>" : FormatDouble(pressurePa.Value))}",
                    $"totalFlow={(totalFlow is null ? "<missing>" : FormatDouble(totalFlow.Value))}",
                    $"fractions={(fractions is null ? "<missing>" : FormatDoubleList(fractions))}"));
            return null;
        }

        if (!IsFinitePositive(temperatureK.Value) ||
            !IsFinitePositive(pressurePa.Value) ||
            !double.IsFinite(totalFlow.Value) ||
            totalFlow.Value < 0.0d ||
            fractions.Length != components.Count)
        {
            UnitOperationComTrace.Write(
                OperationName,
                "skip",
                string.Join(
                    "; ",
                    "Connected feed material returned an unusable state.",
                    $"temperature={FormatDouble(temperatureK.Value)}",
                    $"pressure={FormatDouble(pressurePa.Value)}",
                    $"totalFlow={FormatDouble(totalFlow.Value)}",
                    $"componentCount={components.Count}",
                    $"fractionCount={fractions.Length}"));
            return null;
        }

        var normalizedFractions = NormalizeFractions(fractions);
        var componentFractions = components
            .Select((component, index) => new CapeOpenFeedComponentFraction(
                component.Id,
                component.Formula,
                component.Name,
                component.CasNo,
                normalizedFractions[index]))
            .ToArray();

        UnitOperationComTrace.Write(
            OperationName,
            "success",
            string.Join(
                "; ",
                $"temperature={FormatDouble(temperatureK.Value)}",
                $"pressure={FormatDouble(pressurePa.Value)}",
                $"totalFlow={FormatDouble(totalFlow.Value)}",
                $"components={string.Join(",", componentFractions.Select(static component => component.Id))}",
                $"fractions={FormatDoubleList(normalizedFractions)}"));

        return new CapeOpenFeedMaterialSnapshot(
            temperatureK.Value,
            pressurePa.Value,
            totalFlow.Value,
            componentFractions);
    }

    private static (double? TemperatureK, double? PressurePa, double[]? Fractions) TryGetOverallTPFraction(
        ICapeThermoMaterial material)
    {
        const string memberName = $"{OperationName}.GetOverallTPFraction";
        try
        {
            double temperature = 0.0d;
            double pressure = 0.0d;
            object? composition = null;
            material.GetOverallTPFraction(ref temperature, ref pressure, ref composition);
            var fractions = ConvertDoubleArray(composition);
            UnitOperationComTrace.Write(
                memberName,
                "success",
                $"temperature={FormatDouble(temperature)}; pressure={FormatDouble(pressure)}; composition={DescribeDoubleArray(fractions)}");
            return (temperature, pressure, fractions);
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception(memberName, error);
            return (null, null, null);
        }
    }

    private static double? TryGetOverallScalarProp(
        ICapeThermoMaterial material,
        string property,
        IReadOnlyList<string?> basisCandidates)
    {
        var values = TryGetOverallVectorProp(material, property, basisCandidates);
        return values is { Length: > 0 } ? values[0] : null;
    }

    private static double[]? TryGetOverallVectorProp(
        ICapeThermoMaterial material,
        string property,
        IReadOnlyList<string?> basisCandidates)
    {
        foreach (var basis in basisCandidates)
        {
            var memberName = $"{OperationName}.GetOverallProp.{property}";
            UnitOperationComTrace.Write(memberName, "attempt", $"basis={DescribeNullableString(basis)}");
            try
            {
                object? results = null;
                material.GetOverallProp(property, basis, ref results);
                var values = ConvertDoubleArray(results);
                if (values is not null)
                {
                    UnitOperationComTrace.Write(
                        memberName,
                        "success",
                        $"basis={DescribeNullableString(basis)}; values={DescribeDoubleArray(values)}");
                    return values;
                }

                UnitOperationComTrace.Write(
                    memberName,
                    "empty",
                    $"basis={DescribeNullableString(basis)}; values=<unusable>");
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception(memberName, error);
            }
        }

        return null;
    }

    private static IReadOnlyList<CapeOpenFeedComponentDescriptor> TryReadComponents(object materialObject)
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
                    return convertedComponentIds
                        .Select((componentId, index) => new CapeOpenFeedComponentDescriptor(
                            componentId,
                            GetOptionalString(convertedFormulae, index),
                            GetOptionalString(convertedNames, index),
                            GetOptionalString(convertedCasNos, index)))
                        .ToArray();
                }
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
                if (ConvertStringEnumerable(materialObject10.ComponentIds) is { Count: > 0 } componentIds)
                {
                    return componentIds
                        .Select(static componentId => new CapeOpenFeedComponentDescriptor(componentId, null, null, null))
                        .ToArray();
                }
            }
            catch (Exception error)
            {
                UnitOperationComTrace.Exception($"{OperationName}.ComponentIds", error);
            }
        }

        return [];
    }

    private static double[]? ConvertDoubleArray(object? value)
    {
        return value switch
        {
            null => null,
            double scalar => [scalar],
            double[] values => values,
            object[] values => values.Select(Convert.ToDouble).ToArray(),
            Array values => values.Cast<object?>().Select(Convert.ToDouble).ToArray(),
            _ => null,
        };
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

    private static double[] NormalizeFractions(IReadOnlyList<double> fractions)
    {
        var nonNegative = fractions.Select(static fraction => Math.Max(fraction, 0.0d)).ToArray();
        var sum = nonNegative.Sum();
        return sum <= 0.0d || !double.IsFinite(sum)
            ? nonNegative
            : nonNegative.Select(fraction => fraction / sum).ToArray();
    }

    private static bool IsFinitePositive(double value)
    {
        return double.IsFinite(value) && value > 0.0d;
    }

    private static string? GetOptionalString(IReadOnlyList<string> values, int index)
    {
        return index >= 0 && index < values.Count && !string.IsNullOrWhiteSpace(values[index])
            ? values[index]
            : null;
    }

    private static TInterface? TryCast<TInterface>(object value)
        where TInterface : class
    {
        try
        {
            return value as TInterface;
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception($"{OperationName}.QueryInterface.{typeof(TInterface).Name}", error);
            return null;
        }
    }

    private static string DescribeDoubleArray(IReadOnlyList<double>? values)
    {
        return values is null ? "<null>" : $"System.Double[][{values.Count}]={FormatDoubleList(values)}";
    }

    private static string DescribeNullableString(string? value)
    {
        return value is null ? "<null>" : $"`{value}`";
    }

    private static string FormatDoubleList(IEnumerable<double> values)
    {
        return string.Join(",", values.Select(FormatDouble));
    }

    private static string FormatDouble(double value)
    {
        return value.ToString("G17", CultureInfo.InvariantCulture);
    }

    private sealed record CapeOpenFeedComponentDescriptor(
        string Id,
        string? Formula,
        string? Name,
        string? CasNo);
}

internal sealed record CapeOpenFeedMaterialSnapshot(
    double TemperatureK,
    double PressurePa,
    double TotalMolarFlowMolS,
    IReadOnlyList<CapeOpenFeedComponentFraction> ComponentFractions);

internal sealed record CapeOpenFeedComponentFraction(
    string Id,
    string? Formula,
    string? Name,
    string? CasNo,
    double MoleFraction)
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
