using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Thermo;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal sealed record MaterialState(double TemperatureK, double PressurePa, double FlowMolS, double[] Fractions);
internal sealed record MaterialPhase(string Label, double Fraction, double[] Composition);

// Select a protocol by supported interfaces, never by a failed property/equilibrium call.
internal abstract class CapeOpenMaterialAccess
{
    protected CapeOpenMaterialAccess(string port) => Port = port;
    public string Port { get; }
    public abstract string Version { get; }
    public abstract MaterialCompound[] Compounds();
    public abstract MaterialState ReadState();
    public abstract IReadOnlyList<MaterialPhase> ReadPhases();
    public abstract void WriteState(MaterialState state);
    public abstract void Equilibrate();
    public abstract void CheckPublicationCapabilities();

    public static CapeOpenMaterialAccess Select(object? material, string port, bool output)
    {
        if (material is ICapeThermoMaterial v11)
        {
            if (material is not ICapeThermoCompounds compounds)
                throw Unsupported(port, "Thermo 1.1 material does not provide ICapeThermoCompounds.");
            var equilibrium = material as ICapeThermoEquilibriumRoutine;
            if (output && equilibrium is null)
                throw Unsupported(port, "Thermo 1.1 Product does not provide ICapeThermoEquilibriumRoutine.");
            var phases = material as ICapeThermoPhases;
            if (output && phases is null)
                throw Unsupported(port, "Thermo 1.1 Product does not provide ICapeThermoPhases.");
            return new Material11(port, v11, compounds, equilibrium, phases);
        }
        if (material is ICapeThermoMaterialObject v10) return new Material10(port, v10);
        throw Unsupported(port, "Connected object does not provide a supported CAPE-OPEN material interface.");
    }

    private static CapeNoImplementationException Unsupported(string port, string message) => new(
        $"{port}: {message}", MaterialBoundaryValues.Context($"{port}.SelectMaterialInterface"));

    protected T Call<T>(string member, Func<T> call, bool equilibrium = false)
    {
        var stage = $"{Port}.{Version}.{member}";
        try { return call(); }
        catch (CapeOpenException) { throw; }
        catch (Exception error)
        {
            throw MaterialBoundaryValues.Propagate(error, stage, $"{stage}: material call failed. {error.Message}", equilibrium);
        }
    }

    protected void Call(string member, Action call, bool equilibrium = false) =>
        Call(member, () => { call(); return true; }, equilibrium);

    private sealed class Material11(string port, ICapeThermoMaterial material,
        ICapeThermoCompounds compounds, ICapeThermoEquilibriumRoutine? equilibrium, ICapeThermoPhases? phases) : CapeOpenMaterialAccess(port)
    {
        private string[]? _candidatePhases;
        public override string Version => "Thermo1.1";
        public override MaterialCompound[] Compounds() => Call("GetCompoundList", () =>
        {
            object? ids = null, formulae = null, names = null, boiling = null, weights = null, cas = null;
            compounds.GetCompoundList(ref ids, ref formulae, ref names, ref boiling, ref weights, ref cas);
            var identifiers = MaterialBoundaryValues.Strings(ids, $"{Port}.compounds");
            return identifiers.Select((id, index) => new MaterialCompound(id,
                OptionalText(names, index, identifiers.Length), OptionalText(cas, index, identifiers.Length))).ToArray();
        });

        private string? OptionalText(object? value, int index, int count)
        {
            if (value is null) return null;
            if (value is not Array array || array.Length != count)
                throw MaterialBoundaryValues.Invalid($"{Port}.compounds", "Compound metadata length does not match the compound list.");
            var text = array.GetValue(index + array.GetLowerBound(0));
            if (text is not null && text is not string)
                throw MaterialBoundaryValues.Invalid($"{Port}.compounds", "Compound metadata must contain strings.");
            return text as string;
        }

        private object? Overall(string property, string? basis) => Call($"GetOverallProp.{property}", () =>
        {
            object? values = null;
            material.GetOverallProp(property, basis, ref values);
            return values;
        });

        public override MaterialState ReadState() => Call("ReadState", () =>
        {
            double temperature = 0, pressure = 0;
            object? fractions = null;
            material.GetOverallTPFraction(ref temperature, ref pressure, ref fractions);
            return new MaterialState(temperature, pressure,
                MaterialBoundaryValues.Scalar(Overall("totalFlow", "Mole"), $"{Port}.totalFlow"),
                MaterialBoundaryValues.Numbers(fractions, $"{Port}.fraction"));
        });

        public override void WriteState(MaterialState state)
        {
            if (_candidatePhases is null) throw Unsupported(Port, "Publication capabilities have not been checked.");
            Call("ClearAllProps", material.ClearAllProps);
            // Candidate phases are host-defined; only the equilibrium routine may mark them equilibrated.
            Call("SetPresentPhases", () => material.SetPresentPhases(_candidatePhases,
                _candidatePhases.Select(_ => (int)CapePhaseStatus.CAPE_UNKNOWNPHASESTATUS).ToArray()));
            Set("fraction", "Mole", state.Fractions);
            Set("totalFlow", "Mole", [state.FlowMolS]);
            Set("temperature", null, [state.TemperatureK]);
            Set("pressure", null, [state.PressurePa]);
        }

        private void Set(string property, string? basis, double[] values) =>
            Call($"SetOverallProp.{property}", () => material.SetOverallProp(property, basis, values));

        private static string[] TemperatureSpec => ["temperature", string.Empty, "Overall"];
        private static string[] PressureSpec => ["pressure", string.Empty, "Overall"];

        public override void CheckPublicationCapabilities()
        {
            if (equilibrium is null || phases is null) throw Unsupported(Port, "Missing Thermo 1.1 publication interfaces.");
            _candidatePhases = Call("GetPhaseList", () =>
            {
                object? labels = null, aggregation = null, keyCompounds = null;
                phases.GetPhaseList(ref labels, ref aggregation, ref keyCompounds);
                return MaterialBoundaryValues.Strings(labels, $"{Port}.candidatePhases");
            });
        }

        public override void Equilibrate()
        {
            // The standard requires SetPresentPhases before specification checks too.
            if (!Call("CheckEquilibriumSpec", () => equilibrium!.CheckEquilibriumSpec(TemperatureSpec, PressureSpec, "unspecified")))
                throw new CapeNoImplementationException("TP equilibrium specification is not supported.",
                    MaterialBoundaryValues.Context($"{Port}.Thermo1.1.CheckEquilibriumSpec"));
            Call("CalcEquilibrium", () => equilibrium!.CalcEquilibrium(TemperatureSpec, PressureSpec, "unspecified"), equilibrium: true);
        }

        public override IReadOnlyList<MaterialPhase> ReadPhases() => Call("ReadPresentPhases", () =>
        {
            object? labelsValue = null, statusValue = null;
            material.GetPresentPhases(ref labelsValue, ref statusValue);
            var labels = MaterialBoundaryValues.Strings(labelsValue, $"{Port}.phases");
            if (_candidatePhases is null || labels.Any(label => !_candidatePhases.Contains(label, StringComparer.Ordinal)))
                throw MaterialBoundaryValues.Invalid($"{Port}.phases", "Equilibrium returned an unrequested phase.");
            if (statusValue is not Array statuses || statuses.Length != labels.Length ||
                statuses.Cast<object?>().Any(value => value is not int status || status != (int)CapePhaseStatus.CAPE_ATEQUILIBRIUM))
                throw MaterialBoundaryValues.Invalid($"{Port}.phases", "Present phases must report equilibrium status.");
            return labels.Select(label =>
            {
                double temperature = 0, pressure = 0;
                object? composition = null, fraction = null;
                Call($"GetTPFraction.{label}", () => material.GetTPFraction(label, ref temperature, ref pressure, ref composition));
                var overall = ReadState();
                MaterialBoundaryValues.Equal(temperature, overall.TemperatureK, $"{Port}.{label}.temperature");
                MaterialBoundaryValues.Equal(pressure, overall.PressurePa, $"{Port}.{label}.pressure");
                Call($"GetSinglePhaseProp.{label}.phaseFraction", () => material.GetSinglePhaseProp("phaseFraction", label, "Mole", ref fraction));
                return new MaterialPhase(label, MaterialBoundaryValues.Scalar(fraction, $"{Port}.{label}.phaseFraction"),
                    MaterialBoundaryValues.Numbers(composition, $"{Port}.{label}.fraction"));
            }).ToArray();
        });
    }

    private sealed class Material10(string port, ICapeThermoMaterialObject material) : CapeOpenMaterialAccess(port)
    {
        public override string Version => "Thermo1.0";
        public override MaterialCompound[] Compounds() => Call("ComponentIds", () =>
            MaterialBoundaryValues.Strings(material.ComponentIds, $"{Port}.compounds").Select(id => new MaterialCompound(id)).ToArray());

        private object? Get(string property, string phase, string? basis) =>
            Call($"GetProp.{phase}.{property}", () => material.GetProp(property, phase, null, null, basis));

        public override MaterialState ReadState() => new(
            MaterialBoundaryValues.Scalar(Get("temperature", "Overall", null), $"{Port}.temperature"),
            MaterialBoundaryValues.Scalar(Get("pressure", "Overall", null), $"{Port}.pressure"),
            MaterialBoundaryValues.Scalar(Get("totalFlow", "Overall", "Mole"), $"{Port}.totalFlow"),
            MaterialBoundaryValues.Numbers(Get("fraction", "Overall", "Mole"), $"{Port}.fraction"));

        public override void WriteState(MaterialState state)
        {
            // Thermo 1.0 exposes no ClearAllProps here. Set the full input specification,
            // require CalcEquilibrium, and verify all delivered state before success.
            Set("fraction", "Mole", state.Fractions);
            Set("totalFlow", "Mole", [state.FlowMolS]);
            Set("temperature", null, [state.TemperatureK]);
            Set("pressure", null, [state.PressurePa]);
        }

        private void Set(string property, string? basis, double[] values) =>
            Call($"SetProp.{property}", () => material.SetProp(property, "Overall", null, null, basis, values));

        public override void CheckPublicationCapabilities() { /* 1.0 has no CheckEquilibriumSpec. */ }
        public override void Equilibrate() => Call("CalcEquilibrium", () => material.CalcEquilibrium("TP", null), equilibrium: true);

        public override IReadOnlyList<MaterialPhase> ReadPhases()
        {
            var labels = Call("PhaseIds", () => MaterialBoundaryValues.Strings(material.PhaseIds, $"{Port}.phases"));
            var phases = new List<MaterialPhase>();
            var overall = ReadState();
            foreach (var label in labels.Where(label => !string.Equals(label, "Overall", StringComparison.OrdinalIgnoreCase)))
            {
                var fraction = MaterialBoundaryValues.Scalar(Get("phaseFraction", label, "Mole"), $"{Port}.{label}.phaseFraction");
                if (fraction == 0) continue;
                MaterialBoundaryValues.Equal(MaterialBoundaryValues.Scalar(Get("temperature", label, null), $"{Port}.{label}.temperature"),
                    overall.TemperatureK, $"{Port}.{label}.temperature");
                MaterialBoundaryValues.Equal(MaterialBoundaryValues.Scalar(Get("pressure", label, null), $"{Port}.{label}.pressure"),
                    overall.PressurePa, $"{Port}.{label}.pressure");
                phases.Add(new MaterialPhase(label, fraction,
                    MaterialBoundaryValues.Numbers(Get("fraction", label, "Mole"), $"{Port}.{label}.fraction")));
            }
            return phases;
        }
    }
}
