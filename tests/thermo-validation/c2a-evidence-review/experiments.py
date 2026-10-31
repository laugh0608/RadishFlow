"""Additional ThermoML evidence; frozen baseline criteria, no fitting or relabeling."""
from __future__ import annotations
import argparse
from collections import Counter
from pathlib import Path
import subprocess
from common import ROOT, load_json, write_json, verify_baseline, reference as ref
import thermoml

SPECS = {"je0496701": ("10.1021/je0496701", [1, 2]), "j.jct.2004.07.017": ("10.1016/j.jct.2004.07.017", [11, 13, 14])}


def join_partial(pressures, compositions):
    key = lambda row: (row["temperature_k"], row["x_propane"])
    mapping = {key(row): row for row in compositions}
    if len(mapping) != len(compositions) or len({key(row) for row in pressures}) != len(pressures):
        raise ValueError("Nonunique T/x join")
    paired, unmatched = [], []
    for row in pressures:
        other = mapping.pop(key(row), None)
        if other is None:
            unmatched.append({"id": row["id"], "reason": "missing_reported_vapor_composition"})
        else:
            paired.append({**row, "y_propane": other["y_propane"], "paired_id": other["id"], "y_expanded_uncertainty": other["expanded_uncertainty"]})
    unmatched.extend({"id": row["id"], "reason": "missing_reported_pressure"} for row in mapping.values())
    return paired, unmatched


def build_cases(data):
    datasets = {d["id"]: d for d in data["datasets"]}
    pairs, unpaired = join_partial(datasets["je0496701/d1"]["rows"], datasets["je0496701/d2"]["rows"])
    cases, exclusions = [], []
    for row in pairs:
        z = (row["x_propane"] + row["y_propane"])/2
        reasons = ref.domain_reasons(row["temperature_k"], row["pressure_pa"], z)
        if not 0 < row["x_propane"] < row["y_propane"] < 1:
            reasons.append("unresolved_tie_line")
        case = {"kind": "binary_vle", "observed": row, "constructed_z_propane": z, "constructed_beta": .5}
        if reasons:
            exclusions.append({**case, "reasons": reasons})
        else:
            cases.append(case)
    pressures = {row["temperature_k"]: row for row in datasets["j.jct.2004.07.017/d11"]["rows"]}
    if len(pressures) != len(datasets["j.jct.2004.07.017/d11"]["rows"]):
        raise ValueError("Duplicate saturation pressure temperatures")
    used_pressure_ids = set()
    for row in datasets["j.jct.2004.07.017/d13"]["rows"]:
        pressure = pressures.get(row["temperature_k"])
        if pressure is None:
            raise ValueError("Saturated vapor density has no exact measured pressure")
        used_pressure_ids.add(pressure["id"])
        observed = {**row, "pressure_pa": pressure["pressure_pa"], "paired_id": pressure["id"], "pressure_expanded_uncertainty": pressure["expanded_uncertainty"]}
        case = {"kind": "pure_saturated_vapor_density", "observed": observed}
        reasons = ref.domain_reasons(observed["temperature_k"], observed["pressure_pa"], None)
        (exclusions if reasons else cases).append({**case, **({"reasons": reasons} if reasons else {})})
    for row in datasets["j.jct.2004.07.017/d11"]["rows"]:
        if row["id"] not in used_pressure_ids:
            unpaired.append({"id": row["id"], "reason": "pressure_only_no_saturated_vapor_density"})
    butane = ref.PARAMETERS["components"][1]
    rhoc = butane["critical_molar_density_mol_m3"] * butane["molar_mass_kg_mol"]
    for row in datasets["j.jct.2004.07.017/d14"]["rows"]:
        reasons = ref.domain_reasons(row["temperature_k"], row["pressure_pa"], None)
        if row["density_kg_m3"] >= rhoc:
            reasons.append("observed_density_not_gas_branch")
        case = {"kind": "pure_vapor_TP_density", "observed": row, "observed_phase_basis": "Subcritical measured density below frozen critical density; no model phase label used for selection"}
        (exclusions if reasons else cases).append({**case, **({"reasons": reasons} if reasons else {})})
    return cases, exclusions, unpaired


def queries(case, backend):
    row = case["observed"]
    t, p = row["temperature_k"], row["pressure_pa"]
    if case["kind"] == "binary_vle":
        return {"bubble": ref.attempt(ref.query_saturation, backend, t, 0., row["x_propane"]), "tp": ref.attempt(ref.query_tp, backend, t, p, case["constructed_z_propane"])}
    if case["kind"] == "pure_saturated_vapor_density":
        return {"saturation": ref.attempt(ref.query_saturation, backend, t, 1., fluid="n-Butane"), "density_at_measured_pressure": ref.attempt(ref.query_density, backend, t, p, "vapor", fluid="n-Butane")}
    return {"density_at_measured_pressure": ref.attempt(ref.query_density, backend, t, p, "vapor", fluid="n-Butane"), "raw_tp": ref.attempt(ref.query_tp, backend, t, p, fluid="n-Butane")}


def score(identifier, backend, quantity, actual, expected, limit, relative=False):
    error = abs(actual-expected) / abs(expected) if relative else abs(actual-expected)
    return {"id": identifier, "backend": backend, "quantity": quantity, "actual": actual, "observed": expected, "absolute_error": abs(actual-expected), "error": error, "error_type": "relative" if relative else "absolute", "limit": limit, "status": "pass" if error <= limit else "fail"}


def assess(cases):
    checks = []
    limits = ref.PROTOCOL["thresholds"]
    for case in cases:
        row = case["observed"]
        for backend, results in case["results"].items():
            def add(name, actual, observed, limit, relative=False):
                checks.append(score(row["id"], backend, name, actual, observed, limit, relative))
            if case["kind"] == "binary_vle":
                bubble, tp = results["bubble"], results["tp"]
                if bubble["status"] == "ok":
                    add("bubble_pressure", bubble["pressure_pa"], row["pressure_pa"], limits["pressure_relative"], True)
                    add("bubble_y", bubble["y"][0], row["y_propane"], limits["composition_absolute"])
                if tp["status"] == "ok" and tp["region"] == "two_phase":
                    add("TP_x", tp["phases"]["liquid"]["mole_fractions"][0], row["x_propane"], limits["composition_absolute"])
                    add("TP_y", tp["phases"]["vapor"]["mole_fractions"][0], row["y_propane"], limits["composition_absolute"])
                    add("TP_constructed_beta", tp["beta_molar"], .5, limits["beta_absolute"])
                else:
                    checks.append({"id": row["id"], "backend": backend, "quantity": "TP_phase", "status": "reference_error" if tp["status"] != "ok" else "fail", "raw": tp})
            elif case["kind"] == "pure_saturated_vapor_density":
                sat = results["saturation"]
                if sat["status"] == "ok":
                    add("saturated_vapor_density", sat["vapor_density_kg_m3"], row["density_kg_m3"], limits["density_relative"], True)
                    add("saturation_pressure", sat["pressure_pa"], row["pressure_pa"], limits["pressure_relative"], True)
            density = results.get("density_at_measured_pressure")
            if density and density["status"] == "ok":
                add("vapor_density_at_measured_pressure", density["density_kg_m3"], row["density_kg_m3"], limits["density_relative"], True)
            for name, result in results.items():
                if result["status"] != "ok":
                    checks.append({"id": row["id"], "backend": backend, "quantity": name, "status": "reference_error", "raw": result})
    summary = []
    for backend, quantity in sorted({(c["backend"], c["quantity"]) for c in checks}):
        selected = [c for c in checks if (c["backend"], c["quantity"]) == (backend, quantity)]
        summary.append({"backend": backend, "quantity": quantity, "counts": dict(Counter(c["status"] for c in selected)), "maximum_error": max((c["error"] for c in selected if "error" in c), default=None)})
    return {"summary": summary, "checks": checks}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-dir", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--freeze", action="store_true")
    parser.add_argument("--fetch", action="store_true")
    args = parser.parse_args()
    verify_baseline()
    ref.validate_environment()
    if args.fetch:
        args.source_dir.mkdir(parents=True, exist_ok=True)
        for name, (doi, _) in SPECS.items():
            subprocess.run(["curl", "--fail", "--location", "--silent", "--show-error", "--max-time", "60", f"https://trc.nist.gov/ThermoML/{doi}.json", "--output", str(args.source_dir / f"{name}.json")], check=True)
    source_path = ROOT / "sources.json"
    if args.freeze and source_path.exists():
        parser.error("Sources already frozen")
    thermoml.SPECS = SPECS
    sources, data = thermoml.extract(args.source_dir, None if args.freeze else load_json(source_path))
    if args.freeze:
        write_json(source_path, sources)
    cases, exclusions, unpaired = build_cases(data)
    for case in cases:
        case["results"] = {backend: queries(case, backend) for backend in ("PR", "HEOS")}
    args.output_dir.mkdir(parents=True, exist_ok=True)
    write_json(args.output_dir / "experimental-data.json", data)
    write_json(args.output_dir / "experimental-results.json", {"cases": cases, "excluded": exclusions, "unpaired": unpaired, "assessment": assess(cases)})
    print(f"{len(cases)} cases, {len(exclusions)} exclusions, {len(unpaired)} unpaired measurements")

if __name__ == "__main__":
    main()
