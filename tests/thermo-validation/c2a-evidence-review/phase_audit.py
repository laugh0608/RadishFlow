"""Diagnose frozen reference phase labels; never rewrite accepted TP results."""
from __future__ import annotations
import argparse
import math
import platform
import sys
from pathlib import Path
import numpy as np
import CoolProp
import CoolProp.CoolProp as CP
from common import ROOT, BASELINE, load_json, write_json, verify_baseline, reference as ref

REVISIONS = {"8.0.0": "ae81610e7d23efc57f9d051c8e70a4d66e87537f", "7.2.0": "98b3523d5daa98454618d381d2ae53f7471d216b"}


def snapshot(state):
    return {"status": "ok", "phase": str(state.phase()), "quality": state.Q(),
            "density_mol_m3": state.rhomolar(), "density_kg_m3": state.rhomass(),
            "reducing_density_mol_m3": state.rhomolar_reducing(),
            "temperature_k": state.T(), "pressure_pa": state.p()}


def operation(case, phase=None, repeat=False, roundtrip=False, default=False):
    z, fluid = case.get("z_propane"), case.get("fluid")
    state = CP.AbstractState("PR", fluid or "Propane&n-Butane") if default else ref.new_state("PR", z, fluid)
    if default and not fluid:
        state.set_mole_fractions([z, 1-z])
    if phase is not None:
        state.specify_phase(phase)
    state.update(CP.PT_INPUTS, case["pressure_pa"], case["temperature_k"])
    if repeat:
        state.update(CP.PT_INPUTS, case["pressure_pa"], case["temperature_k"])
    if roundtrip:
        rho = state.rhomolar()
        state.update(CP.DmolarT_INPUTS, rho, case["temperature_k"])
    return snapshot(state)


def pr_roots(case):
    """Independent homogeneous PR cubic; not a mixture flash/stability test."""
    t, p = case["temperature_k"], case["pressure_pa"]
    params = ref.PARAMETERS
    omega_a, omega_b = (.45724, .07780) if CoolProp.__version__ == "7.2.0" else (params["pr_constants"]["omega_a"], params["pr_constants"]["omega_b"])
    components = params["components"]
    fractions = [case["z_propane"], 1-case["z_propane"]] if "z_propane" in case else [float(c["coolprop_name"] == case["fluid"]) for c in components]
    r = params["gas_constant_j_mol_k"]
    aa, bb = [], []
    for c in components:
        tc, pc, w = c["critical_temperature_k"], c["critical_pressure_pa"], c["acentric_factor"]
        m = .37464 + 1.54226*w - .26992*w*w
        aa.append(omega_a*r*r*tc*tc/pc*(1+m*(1-math.sqrt(t/tc)))**2)
        bb.append(omega_b*r*tc/pc)
    a = sum(x*math.sqrt(ai) for x, ai in zip(fractions, aa, strict=True))**2
    b = sum(x*bi for x, bi in zip(fractions, bb, strict=True))
    A, B = a*p/(r*r*t*t), b*p/(r*t)
    coeffs = [1, B-1, A-3*B*B-2*B, -A*B+B*B+B**3]
    roots = []
    for value in np.roots(coeffs):
        if abs(value.imag) > 1e-9 or value.real <= B:
            continue
        z = float(value.real)
        v = z*r*t/p
        denom = v*v+2*b*v-b*b
        dpdv = -r*t/(v-b)**2 + a*(2*v+2*b)/(denom*denom)
        gibbs = z-1-math.log(z-B)-A/(2*math.sqrt(2)*B)*math.log((z+(1+math.sqrt(2))*B)/(z+(1-math.sqrt(2))*B))
        roots.append({"Z": z, "density_mol_m3": 1/v, "mechanically_stable": dpdv < 0,
                      "homogeneous_gibbs_residual_over_rt": gibbs, "polynomial_residual": float(np.polyval(coeffs, z))})
    roots.sort(key=lambda root: root["density_mol_m3"])
    stable = [root for root in roots if root["mechanically_stable"]]
    return {"omega_a": omega_a, "omega_b": omega_b, "roots": roots, "minimum_homogeneous_gibbs_density_mol_m3": min(stable, key=lambda root: root["homogeneous_gibbs_residual_over_rt"])["density_mol_m3"]}


def generate():
    verify_baseline()
    version, revision = CoolProp.__version__, CP.get_global_param_string("gitrevision")
    if REVISIONS.get(version) != revision:
        raise ValueError("Unfrozen CoolProp build")
    # Verify identical PR component constants in both diagnostic versions.
    for component in ref.PARAMETERS["components"]:
        state = ref.new_state("PR", fluid=component["coolprop_name"])
        assert [state.T_critical(), state.p_critical(), state.acentric_factor(), state.molar_mass(), state.gas_constant()] == [component[k] for k in ("critical_temperature_k", "critical_pressure_pa", "acentric_factor", "molar_mass_kg_mol")] + [ref.PARAMETERS["gas_constant_j_mol_k"]]
    baseline = load_json(BASELINE / "reference-results.json")
    output = []
    for original in baseline["states"] + [c for c in baseline["pure"] if c["kind"] == "TP"]:
        case = {k: original[k] for k in ("id", "temperature_k", "pressure_pa", "z_propane", "fluid", "excluded_reasons") if k in original}
        case["baseline_audit"] = original["phase_audits"]["PR"]
        case["baseline_raw_tp"] = original["results"]["PR"]
        operations = {"fresh": {}, "repeat": {"repeat": True}, "default_parameters": {"default": True}, "imposed_gas": {"phase": CP.iphase_gas}, "imposed_liquid": {"phase": CP.iphase_liquid}}
        if "fluid" in case:
            operations["DmolarT_roundtrip"] = {"roundtrip": True}
        case["operations"] = {name: ref.attempt(operation, case, **kwargs) for name, kwargs in operations.items()}
        case["independent_homogeneous_PR"] = pr_roots(case)
        output.append(case)
    return {"schema": "radishflow.c2a-phase-review", "coolprop_version": version, "coolprop_git_revision": revision,
            "python": sys.version, "numpy": np.__version__, "platform": platform.platform(), "phase_enum": {name: str(getattr(CP, name)) for name in ("iphase_liquid", "iphase_gas", "iphase_twophase")}, "cases": output}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    write_json(args.output, generate())

if __name__ == "__main__":
    main()
