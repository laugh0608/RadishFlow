"""Frozen CoolProp PR/HEOS queries for C2-A; no RadishFlow thermo implementation."""

from __future__ import annotations

import argparse
import math
from pathlib import Path

import CoolProp
import CoolProp.CoolProp as CP

from thermoml import ROOT, load_json, sha256, write_json

PARAMETERS = load_json(ROOT / "parameters.json")
PROTOCOL = load_json(ROOT / "protocol.json")
MOLAR_MASSES = [c["molar_mass_kg_mol"] for c in PARAMETERS["components"]]


def validate_environment() -> None:
    if CoolProp.__version__ != PARAMETERS["coolprop_version"]:
        raise ValueError("CoolProp version differs from the frozen reference")
    if CP.get_global_param_string("gitrevision") != PARAMETERS["coolprop_git_revision"]:
        raise ValueError("CoolProp git revision differs from the frozen reference")
    for component in PARAMETERS["components"]:
        state = CP.AbstractState("PR", component["coolprop_name"])
        observed = (state.T_critical(), state.p_critical(), state.acentric_factor(), state.molar_mass())
        expected = tuple(component[k] for k in ("critical_temperature_k", "critical_pressure_pa", "acentric_factor", "molar_mass_kg_mol"))
        if observed != expected or state.gas_constant() != PARAMETERS["gas_constant_j_mol_k"]:
            raise ValueError(f"PR component parameters changed: {component['coolprop_name']}")
    pair = PARAMETERS["heos_pair"]
    for key, value in pair["parameters"].items():
        if CP.get_mixture_binary_pair_data(*pair["cas_order"], key) != value:
            raise ValueError(f"HEOS binary parameter changed: {key}")


def new_state(backend: str, z_propane: float | None = None, fluid: str | None = None):
    if fluid is not None:
        state = CP.AbstractState(backend, fluid)
    else:
        if z_propane is None or not 0.0 < z_propane < 1.0:
            raise ValueError("Binary query requires a strictly interior composition")
        state = CP.AbstractState(backend, "Propane&n-Butane")
        state.set_mole_fractions([z_propane, 1.0 - z_propane])
        if backend == "PR":
            state.set_binary_interaction_double(0, 1, "kij", 0.0)
            state.set_binary_interaction_double(1, 0, "kij", 0.0)
    if backend == "PR":
        state.set_fluid_parameter_double(0, "c", 0.0)
    return state


def finite_positive(value: float) -> float:
    if not math.isfinite(value) or value <= 0.0:
        raise ValueError(f"Reference returned a nonpositive/nonfinite property: {value}")
    return value


def phase_properties(state, fractions: list[float], density_molar: float, fraction: float, flow: float, fluid: str | None = None) -> dict:
    density_molar = finite_positive(density_molar)
    if fluid is None:
        molar_mass = sum(x * m for x, m in zip(fractions, MOLAR_MASSES, strict=True))
    else:
        molar_mass = next(c["molar_mass_kg_mol"] for c in PARAMETERS["components"] if c["coolprop_name"] == fluid)
    return {
        "mole_fractions": fractions,
        "molar_fraction": fraction,
        "molar_density_mol_m3": density_molar,
        "molar_volume_m3_mol": 1.0 / density_molar,
        "molar_mass_kg_mol": molar_mass,
        "mass_density_kg_m3": molar_mass * density_molar,
        "compressibility_factor": state.p() / (density_molar * state.gas_constant() * state.T()),
        "molar_flow_mol_s": flow * fraction,
        "actual_volume_flow_m3_s": flow * fraction / density_molar,
    }


def query_tp(backend: str, temperature: float, pressure: float, z: float | None = None, fluid: str | None = None) -> dict:
    state = new_state(backend, z, fluid)
    state.update(CP.PT_INPUTS, pressure, temperature)
    phase = state.phase()
    phases = {}
    flow = PROTOCOL["baseline_molar_flow_mol_s"]
    if phase == CP.iphase_twophase:
        beta = state.Q()
        if not 0.0 <= beta <= 1.0:
            raise ValueError(f"Invalid two-phase molar fraction: {beta}")
        region = "two_phase"
        if beta < 1.0:
            phases["liquid"] = phase_properties(state, state.mole_fractions_liquid(), state.saturated_liquid_keyed_output(CP.iDmolar), 1.0 - beta, flow, fluid)
        if beta > 0.0:
            phases["vapor"] = phase_properties(state, state.mole_fractions_vapor(), state.saturated_vapor_keyed_output(CP.iDmolar), beta, flow, fluid)
    elif phase in (CP.iphase_liquid, CP.iphase_gas):
        region = "liquid" if phase == CP.iphase_liquid else "vapor"
        beta = 0.0 if region == "liquid" else 1.0
        fractions = [1.0] if fluid else [z, 1.0 - z]
        phases[region] = phase_properties(state, fractions, state.rhomolar(), 1.0, flow, fluid)
    else:
        raise ValueError(f"Reference phase is outside this subcritical validation scope: {phase}")
    return {"status": "ok", "region": region, "beta_molar": beta, "raw_quality": state.Q(),
            "raw_phase": str(phase), "gas_constant_j_mol_k": state.gas_constant(), "phases": phases}


def query_saturation(backend: str, temperature: float, quality: float, z: float | None = None, fluid: str | None = None) -> dict:
    state = new_state(backend, z, fluid)
    state.update(CP.QT_INPUTS, quality, temperature)
    return {
        "status": "ok", "pressure_pa": finite_positive(state.p()),
        "x": state.mole_fractions_liquid(), "y": state.mole_fractions_vapor(),
        "liquid_density_kg_m3": phase_properties(state, state.mole_fractions_liquid(), state.saturated_liquid_keyed_output(CP.iDmolar), 0.0, 0.0, fluid)["mass_density_kg_m3"],
        "vapor_density_kg_m3": phase_properties(state, state.mole_fractions_vapor(), state.saturated_vapor_keyed_output(CP.iDmolar), 0.0, 0.0, fluid)["mass_density_kg_m3"],
    }


def query_density(backend: str, temperature: float, pressure: float, phase: str, z: float | None = None, fluid: str | None = None) -> dict:
    state = new_state(backend, z, fluid)
    state.specify_phase(CP.iphase_liquid if phase == "liquid" else CP.iphase_gas)
    state.update(CP.PT_INPUTS, pressure, temperature)
    return {"status": "ok", "density_kg_m3": finite_positive(state.rhomass()), "imposed_phase": phase}


def attempt(operation, *args, **kwargs) -> dict:
    try:
        return operation(*args, **kwargs)
    except ValueError as error:
        # Preserve reference failures. Never retry with another model or root.
        return {"status": "reference_error", "operation": operation.__name__, "message": str(error)}


def domain_reasons(t: float, p: float | None, z: float | None) -> list[str]:
    d = PROTOCOL["domain"]
    reasons = []
    for value, bounds, name in ((t, d["temperature_k"], "temperature"), (p, d["pressure_pa"], "pressure"), (z, d["propane_overall_mole_fraction"], "composition")):
        if value is not None and not bounds[0] <= value <= bounds[1]:
            reasons.append(f"outside_{name}_domain")
    return reasons


def generate_states() -> tuple[list[dict], list[dict]]:
    cases = [dict(c, group=group) for group in ("grid", "holdout") for c in PROTOCOL[group]]
    boundaries = []
    spec = PROTOCOL["boundary_generation"]
    for t in spec["temperatures_k"]:
        for z in spec["z_propane"]:
            identifier = f"T{t}-z{z}"
            pressures = []
            for name, quality in (("bubble", 0.0), ("dew", 1.0)):
                result = attempt(query_saturation, "HEOS", t, quality, z)
                boundaries.append({"id": f"{identifier}-{name}", "temperature_k": t, "z_propane": z, "reference": result})
                if result["status"] != "ok":
                    continue
                pressures.append(result["pressure_pa"])
                for multiplier in spec["pressure_multipliers"]:
                    cases.append({"id": f"boundary-{identifier}-{name}-{multiplier}", "group": "boundary", "temperature_k": t, "pressure_pa": result["pressure_pa"] * multiplier, "z_propane": z})
            if len(pressures) == 2:
                cases.append({"id": f"interior-{identifier}", "group": "interior", "temperature_k": t, "pressure_pa": sum(pressures) / 2, "z_propane": z})
    for case in cases:
        t, p, z = (case[k] for k in ("temperature_k", "pressure_pa", "z_propane"))
        case["excluded_reasons"] = domain_reasons(t, p, z)
        case["results"] = {backend: attempt(query_tp, backend, t, p, z) for backend in ("PR", "HEOS")}
        # Audit labels against each backend's own envelope. Preserve the raw TP
        # answer; do not substitute a forced root or repair the output in place.
        case["phase_audits"] = {}
        for backend in ("PR", "HEOS"):
            bubble = attempt(query_saturation, backend, t, 0.0, z)
            dew = attempt(query_saturation, backend, t, 1.0, z)
            audit = {"bubble": bubble, "dew": dew, "consistent": None}
            result = case["results"][backend]
            if bubble["status"] == dew["status"] == result["status"] == "ok":
                pb, pd = bubble["pressure_pa"], dew["pressure_pa"]
                if pb <= pd:
                    raise ValueError("Unexpected envelope order in the selected nonazeotropic binary")
                expected = "liquid" if p > pb else "vapor" if p < pd else "two_phase"
                audit.update(envelope_region=expected, consistent=result["region"] == expected)
            case["phase_audits"][backend] = audit
    return cases, boundaries


def generate_pure() -> list[dict]:
    cases = []
    for fluid in ("Propane", "n-Butane"):
        for t in PROTOCOL["pure_temperatures_k"]:
            cases.append({"id": f"pure-saturation-{fluid}-{t}", "kind": "saturation", "fluid": fluid, "temperature_k": t,
                          "results": {b: attempt(query_saturation, b, t, 0.0, fluid=fluid) for b in ("PR", "HEOS")}})
            for p in (200000, 500000, 1000000):
                cases.append({"id": f"pure-TP-{fluid}-{t}-{p}", "kind": "TP", "fluid": fluid, "temperature_k": t, "pressure_pa": p,
                              "results": {b: attempt(query_tp, b, t, p, fluid=fluid) for b in ("PR", "HEOS")}})
    for case in cases:
        if case["kind"] != "TP":
            continue
        case["phase_audits"] = {}
        for backend in ("PR", "HEOS"):
            saturation = attempt(query_saturation, backend, case["temperature_k"], 0.0, fluid=case["fluid"])
            result = case["results"][backend]
            audit = {"saturation": saturation, "consistent": None}
            if saturation["status"] == result["status"] == "ok":
                expected = "liquid" if case["pressure_pa"] > saturation["pressure_pa"] else "vapor"
                audit.update(envelope_region=expected, consistent=result["region"] == expected)
            case["phase_audits"][backend] = audit
    return cases


def joined_rows(left: dict, right: dict) -> list[dict]:
    """Join independent reported properties on exact reported T and liquid x."""
    def key(row):
        return row["temperature_k"], row["x_propane"]
    mapping = {key(r): r for r in right["rows"]}
    if len(mapping) != len(right["rows"]) or len(left["rows"]) != len(right["rows"]):
        raise ValueError("Nonunique or incomplete experimental join")
    output = []
    for row in left["rows"]:
        other = mapping.pop(key(row))
        output.append({**row, right["property"]: other[right["property"]],
                       "paired_id": other["id"], "paired_expanded_uncertainty": other["expanded_uncertainty"]})
    if mapping:
        raise ValueError("Unmatched experimental measurements")
    return output


def generate_experiments(data: dict) -> tuple[list[dict], list[dict]]:
    datasets = {d["id"]: d for d in data["datasets"]}
    cases, exclusions = [], []
    for dataset in datasets.values():
        # These rows are paired by exact T/x with the pressure dataset, not duplicated.
        if dataset["id"] in ("je800377b/d4", "je049671t/d3"):
            continue
        rows = dataset["rows"]
        if dataset["id"] == "je800377b/d3":
            rows = joined_rows(dataset, datasets["je800377b/d4"])
        elif dataset["id"] == "je049671t/d2":
            rows = joined_rows(dataset, datasets["je049671t/d3"])
        for row in rows:
            t, p = row["temperature_k"], row.get("pressure_pa")
            binary = len(dataset["components"]) == 2
            fluid = None if binary else dataset["components"][0]
            x = row.get("x_propane")
            if dataset["id"] == "je800377b/d3":
                z = (x + row["y_propane"]) / 2
                reasons = domain_reasons(t, p, z)
                if not 0.0 < x < 1.0 or not x < row["y_propane"]:
                    reasons.append("pure_endpoint_or_no_resolvable_tie_line")
                kind = "binary_vle"
            else:
                z = x
                # Pure saturation checks cover the approved T interval independently of mixture P bounds.
                pure_saturation = not binary and dataset["property"] == "pressure_pa"
                reasons = domain_reasons(t, None if pure_saturation else p, z)
                kind = "bubble_density" if dataset["id"] == "je049671t/d2" else dataset["property"]
            if reasons:
                exclusions.append({"id": row["id"], "paired_id": row.get("paired_id"), "reasons": reasons})
                continue
            case = {"id": row["id"], "dataset_id": dataset["id"], "kind": kind, "fluid": fluid, "observed": row, "results": {}}
            for backend in ("PR", "HEOS"):
                if kind == "binary_vle":
                    results = {"bubble": attempt(query_saturation, backend, t, 0.0, x),
                               "tp": attempt(query_tp, backend, t, p, z)}
                    case["constructed_overall_z_propane"] = z
                    case["constructed_beta_molar"] = 0.5
                elif kind in ("pressure_pa", "bubble_density"):
                    results = {"saturation": attempt(query_saturation, backend, t, 0.0, x, fluid)}
                    if kind == "bubble_density":
                        results["density_at_measured_pressure"] = attempt(query_density, backend, t, p, "liquid", x)
                elif kind == "density_kg_m3" and p is None:
                    results = {"saturation": attempt(query_saturation, backend, t, 0.0, x, fluid)}
                else:
                    if dataset["property_phase"] == "Liquid":
                        phase, basis = "liquid", "ThermoML declared liquid phase"
                    else:
                        # These selected pure-fluid PVT measurements have a generic 'Fluid' label.
                        # Below Tc, the measured density chooses the observed branch, not the EOS prediction.
                        component = next(c for c in PARAMETERS["components"] if c["coolprop_name"] == fluid)
                        critical_density = component["critical_molar_density_mol_m3"] * component["molar_mass_kg_mol"]
                        phase = "liquid" if row["density_kg_m3"] > critical_density else "vapor"
                        basis = "Subcritical measured density relative to frozen critical density; imposed only for branch-density comparison"
                    case["observed_phase"] = phase
                    case["phase_selection_basis"] = basis
                    results = {"density_at_measured_pressure": attempt(query_density, backend, t, p, phase, x, fluid),
                               "unconstrained_tp": attempt(query_tp, backend, t, p, x, fluid)}
                case["results"][backend] = results
            cases.append(case)
    return cases, exclusions


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args()
    validate_environment()
    cases, boundaries = generate_states()
    experiments, exclusions = generate_experiments(load_json(ROOT / "experimental-data.json"))
    output = {"schema": "radishflow.c2a-reference-results", "version": 1,
              "input_sha256": {name: sha256(ROOT / name) for name in ("protocol.json", "parameters.json", "experimental-data.json", "sources.json")},
              "coolprop_version": CoolProp.__version__, "coolprop_git_revision": CP.get_global_param_string("gitrevision"),
              "states": cases, "boundary_definitions": boundaries, "pure": generate_pure(),
              "experiments": experiments, "experimental_exclusions": exclusions}
    args.output_dir.mkdir(parents=True, exist_ok=True)
    write_json(args.output_dir / "reference-results.json", output)
    print(f"Generated {len(cases)} binary states, {len(output['pure'])} pure cases, {len(experiments)} experimental comparisons, {len(exclusions)} excluded experimental rows")


if __name__ == "__main__":
    main()
