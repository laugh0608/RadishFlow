using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Guids;
using RadishFlow.CapeOpen.Interop.Ole;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Persistence;
using RadishFlow.CapeOpen.Interop.Thermo;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text.Json;

internal sealed class ContractConnectedObject : ICapeIdentification
{
    public ContractConnectedObject(string componentName)
    {
        ComponentName = componentName;
        ComponentDescription = "UnitOp.Mvp contract test connected object.";
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }
}

internal sealed class ContractThermoMaterial : ICapeIdentification, ICapeThermoMaterial, ICapeThermoCompounds, ICapeThermoEquilibriumRoutine
{
    private readonly Dictionary<string, double[]> _overallProps = new(StringComparer.OrdinalIgnoreCase);
    private readonly string[] _componentIds;
    private readonly string[] _componentNames;
    private double _temperatureK;
    private double _pressurePa;

    private ContractThermoMaterial(
        string componentName,
        string[] componentIds,
        string[] componentNames,
        double temperatureK,
        double pressurePa)
    {
        ComponentName = componentName;
        ComponentDescription = "UnitOp.Mvp contract test CAPE-OPEN material object.";
        _componentIds = componentIds;
        _componentNames = componentNames;
        _temperatureK = temperatureK;
        _pressurePa = pressurePa;
    }

    public static ContractThermoMaterial CreateFeed(
        string componentName,
        string[] componentIds,
        string[] componentNames,
        double temperatureK,
        double pressurePa,
        double totalMolarFlowMolS,
        double[] overallMoleFractions)
    {
        var material = new ContractThermoMaterial(componentName, componentIds, componentNames, temperatureK, pressurePa);
        material._overallProps["fraction"] = overallMoleFractions;
        material._overallProps["totalFlow"] = [totalMolarFlowMolS];
        material._overallProps["flow"] = overallMoleFractions.Select(fraction => fraction * totalMolarFlowMolS).ToArray();
        return material;
    }

    public static ContractThermoMaterial CreateEmptyProduct(
        string componentName,
        string[] componentIds,
        string[] componentNames)
    {
        return new ContractThermoMaterial(componentName, componentIds, componentNames, 298.15d, 101325.0d);
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }

    public double GetStoredOverallScalar(string property)
    {
        return GetStoredOverallVector(property)[0];
    }

    public double[] GetStoredOverallVector(string property)
    {
        return _overallProps.TryGetValue(property, out var values)
            ? values
            : throw new InvalidOperationException($"Expected stored overall property `{property}`.");
    }

    public void ClearAllProps()
    {
        _overallProps.Clear();
    }

    public void CopyFromMaterial(ref object source)
    {
    }

    public object CreateMaterial()
    {
        return CreateEmptyProduct(ComponentName + " Copy", _componentIds, _componentNames);
    }

    public void GetOverallProp(string property, string? basis, ref object? results)
    {
        results = _overallProps.TryGetValue(property, out var values) ? values : null;
    }

    public void GetOverallTPFraction(ref double temperature, ref double pressure, ref object? composition)
    {
        temperature = _temperatureK;
        pressure = _pressurePa;
        composition = _overallProps.TryGetValue("fraction", out var values) ? values : null;
    }

    public void GetPresentPhases(ref object? phaseLabels, ref object? phaseStatus)
    {
        phaseLabels = new[] { "Liquid" };
        phaseStatus = new[] { (int)CapePhaseStatus.CAPE_ATEQUILIBRIUM };
    }

    public void GetSinglePhaseProp(string property, string phaseLabel, string? basis, ref object? results)
    {
        results = property switch
        {
            "temperature" => new[] { _temperatureK },
            "pressure" => new[] { _pressurePa },
            "fraction" => _overallProps.TryGetValue("fraction", out var fractions) ? fractions : null,
            "phaseFraction" => new[] { 1.0d },
            _ => null,
        };
    }

    public void GetTPFraction(string phaseLabel, ref double temperature, ref double pressure, ref object? composition)
    {
        GetOverallTPFraction(ref temperature, ref pressure, ref composition);
    }

    public void GetTwoPhaseProp(string property, object? phaseLabels, string? basis, ref object? results)
    {
        results = null;
    }

    public void SetOverallProp(string property, string? basis, object? values)
    {
        var converted = ConvertDoubleArray(values);
        _overallProps[property] = converted;
        if (string.Equals(property, "temperature", StringComparison.OrdinalIgnoreCase) && converted.Length > 0)
        {
            _temperatureK = converted[0];
        }
        else if (string.Equals(property, "pressure", StringComparison.OrdinalIgnoreCase) && converted.Length > 0)
        {
            _pressurePa = converted[0];
        }
    }

    public void SetPresentPhases(object? phaseLabels, object? phaseStatus)
    {
    }

    public void SetSinglePhaseProp(string property, string phaseLabel, string? basis, object? values)
    {
    }

    public void SetTwoPhaseProp(string property, object? phaseLabels, string? basis, object? values)
    {
    }

    public object? GetCompoundConstant(object? props, object? compIds)
    {
        return null;
    }

    public void GetCompoundList(
        ref object? compIds,
        ref object? formulae,
        ref object? names,
        ref object? boilTemps,
        ref object? molwts,
        ref object? casnos)
    {
        compIds = _componentIds;
        formulae = Enumerable.Repeat(string.Empty, _componentIds.Length).ToArray();
        names = _componentNames;
        boilTemps = Enumerable.Repeat(0.0d, _componentIds.Length).ToArray();
        molwts = Enumerable.Repeat(0.0d, _componentIds.Length).ToArray();
        casnos = Enumerable.Repeat(string.Empty, _componentIds.Length).ToArray();
    }

    public void CalcEquilibrium(object? specification1, object? specification2, string solutionType)
    {
    }

    public bool CheckEquilibriumSpec(object? specification1, object? specification2, string solutionType)
    {
        return true;
    }

    private static double[] ConvertDoubleArray(object? value)
    {
        return value switch
        {
            null => [],
            double scalar => [scalar],
            double[] values => values,
            object[] values => values.Select(Convert.ToDouble).ToArray(),
            Array values => values.Cast<object?>().Select(Convert.ToDouble).ToArray(),
            _ => throw new InvalidOperationException($"Unsupported test material value type `{value.GetType().FullName}`."),
        };
    }
}
