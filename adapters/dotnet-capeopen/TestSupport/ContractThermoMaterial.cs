using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Thermo;
using System.Text.Json;

internal class ContractThermoMaterial : ICapeIdentification, ICapeThermoMaterial, ICapeThermoCompounds, ICapeThermoEquilibriumRoutine, ICapeThermoPhases
{
    public static ContractThermoMaterial FromSampleProject(string name, string json)
    {
        using var document = JsonDocument.Parse(json);
        var flowsheet = document.RootElement.GetProperty("document").GetProperty("flowsheet");
        var feed = flowsheet.GetProperty("units").EnumerateObject()
            .Single(unit => unit.Value.GetProperty("kind").GetString() == "feed").Value;
        var id = feed.GetProperty("ports").EnumerateArray().Single().GetProperty("stream_id").GetString()!;
        var stream = flowsheet.GetProperty("streams").GetProperty(id);
        var compounds = flowsheet.GetProperty("components").EnumerateObject().Select(item => item.Name).ToArray();
        return CreateFeed(name, compounds, compounds,
            stream.GetProperty("temperature_k").GetDouble(), stream.GetProperty("pressure_pa").GetDouble(),
            stream.GetProperty("total_molar_flow_mol_s").GetDouble(),
            compounds.Select(compound => stream.GetProperty("overall_mole_fractions").GetProperty(compound).GetDouble()).ToArray());
    }

    private readonly Dictionary<string, double[]> _overallProps = new(StringComparer.OrdinalIgnoreCase);
    private readonly string[] _componentIds;
    private readonly string[] _componentNames;
    private double _temperatureK;
    private double _pressurePa;

    public ContractThermoMaterial(string name) : this(name, ["methane", "ethane"], ["methane", "ethane"], 300, 120000)
    {
        _overallProps["fraction"] = [0.35, 0.65];
        _overallProps["totalFlow"] = [5.0];
    }

    public string? FailAt { get; set; }
    public bool InvalidPhases { get; set; }
    public bool SupportsEquilibrium { get; set; } = true;
    public double ReadbackFlowFactor { get; set; } = 1;
    public double[]? PhaseComposition { get; set; }
    public double PhaseFraction { get; set; } = 1;
    public int FailureHResult { get; set; } = unchecked((int)0x80004005);
    public bool Equilibrated { get; private set; }
    private bool _candidatesSet;
    public List<string> Calls { get; } = [];
    private void Record(string operation)
    {
        Calls.Add(operation);
        if (FailAt == operation) throw new System.Runtime.InteropServices.COMException("Injected " + operation, FailureHResult);
    }

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
        Record("ClearAllProps");
        Equilibrated = false;
        _candidatesSet = false;
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
        Record("GetOverallProp." + property);
        results = _overallProps.TryGetValue(property, out var values) ? values : null;
        if (Equilibrated && property == "totalFlow" && results is double[] flow)
            results = new[] { flow[0] * ReadbackFlowFactor };
    }

    public void GetOverallTPFraction(ref double temperature, ref double pressure, ref object? composition)
    {
        Record("GetOverallTPFraction");
        temperature = _temperatureK;
        pressure = _pressurePa;
        composition = _overallProps.TryGetValue("fraction", out var values) ? values : null;
    }

    public void GetPresentPhases(ref object? phaseLabels, ref object? phaseStatus)
    {
        Record("GetPresentPhases");
        if (!Equilibrated) throw new InvalidOperationException("Equilibrium has not completed.");
        phaseLabels = new[] { "Liquid" };
        phaseStatus = new[] { InvalidPhases ? 0 : (int)CapePhaseStatus.CAPE_ATEQUILIBRIUM };
    }

    public void GetSinglePhaseProp(string property, string phaseLabel, string? basis, ref object? results)
    {
        results = property switch
        {
            "temperature" => new[] { _temperatureK },
            "pressure" => new[] { _pressurePa },
            "fraction" => PhaseComposition ?? (_overallProps.TryGetValue("fraction", out var fractions) ? fractions : null),
            "phaseFraction" => new[] { PhaseFraction },
            _ => null,
        };
    }

    public void GetTPFraction(string phaseLabel, ref double temperature, ref double pressure, ref object? composition)
    {
        GetOverallTPFraction(ref temperature, ref pressure, ref composition);
        composition = PhaseComposition ?? composition;
    }

    public void GetTwoPhaseProp(string property, object? phaseLabels, string? basis, ref object? results)
    {
        results = null;
    }

    public void SetOverallProp(string property, string? basis, object? values)
    {
        Record("SetOverallProp." + property);
        Equilibrated = false;
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
        Record("SetPresentPhases");
        if (phaseLabels is not string[] labels || !labels.SequenceEqual(new[] { "Liquid" }) ||
            phaseStatus is not int[] statuses || !statuses.SequenceEqual(new[] { (int)CapePhaseStatus.CAPE_UNKNOWNPHASESTATUS }))
            throw new InvalidOperationException("Only host-defined candidates with unknown status may be set.");
        _candidatesSet = true;
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
        Record("CalcEquilibrium");
        // Controlled one-phase response for tests; not a thermodynamic model.
        Equilibrated = true;
    }

    public bool CheckEquilibriumSpec(object? specification1, object? specification2, string solutionType)
    {
        Record("CheckEquilibriumSpec");
        if (!_candidatesSet) throw new InvalidOperationException("Candidate phases must precede specification checks.");
        return SupportsEquilibrium;
    }

    public int GetNumPhases() => 1;
    public object? GetPhaseInfo(string phaseLabel, string phaseAttribute) => "Liquid";
    public void GetPhaseList(ref object? labels, ref object? aggregation, ref object? keyCompounds)
    {
        Record("GetPhaseList");
        labels = new[] { "Liquid" };
        aggregation = new[] { "Liquid" };
        keyCompounds = new[] { string.Empty };
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
