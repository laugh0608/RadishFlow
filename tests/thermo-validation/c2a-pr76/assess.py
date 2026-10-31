"""Assess frozen references against the approved limits, preserving every failure."""

from __future__ import annotations

import argparse
import csv
import math
from collections import Counter, defaultdict
from pathlib import Path

from thermoml import ROOT, load_json, sha256, write_json

PROTOCOL = load_json(ROOT / "protocol.json")
LIMITS = PROTOCOL["thresholds"]
ENGINEERING_HEADER = ["id", "backend", "temperature_k", "pressure_pa", "gas_flow_m3_s", "liquid_flow_m3_s",
                      "gas_density_kg_m3", "liquid_density_kg_m3", "effective_k_m_s", "velocity_utilization",
                      "working_time_s", "maximum_working_height_m", "rating_diameter_m", "rating_height_m"]


def numeric_check(case_id: str, category: str, backend: str, metric: str, actual: float, reference: float, limit: float | None, relative: bool) -> dict:
    if not math.isfinite(actual) or not math.isfinite(reference):
        raise ValueError("Nonfinite numbers cannot enter a scored comparison")
    if relative and reference == 0.0:
        raise ValueError("Relative error is undefined at zero; use an explicitly unscored absolute error")
    error = (actual - reference) / reference if relative else actual - reference
    return {"case_id": case_id, "category": category, "backend": backend, "metric": metric,
            "actual": actual, "reference": reference, "signed_error": error, "absolute_error": abs(error),
            "error_basis": "relative" if relative else "absolute", "limit": limit,
            "status": "unscored" if limit is None else "pass" if abs(error) <= limit else "fail"}


def categorical_check(case_id: str, category: str, backend: str, metric: str, actual, reference) -> dict:
    return {"case_id": case_id, "category": category, "backend": backend, "metric": metric,
            "actual": actual, "reference": reference, "status": "pass" if actual == reference else "fail"}


def reference_error(case_id: str, category: str, backend: str, result: dict) -> dict:
    return {"case_id": case_id, "category": category, "backend": backend, "metric": "reference_query",
            "status": "reference_error", "detail": result}


def assess_states(data: dict) -> tuple[list[dict], list[dict]]:
    checks, diagnostics = [], []
    for case in data["states"]:
        if case["excluded_reasons"]:
            continue
        identifier, group = case["id"], case["group"]
        results = case["results"]
        for backend in ("PR", "HEOS"):
            audit = case["phase_audits"][backend]
            if not audit["consistent"]:
                diagnostics.append({"case_id": identifier, "backend": backend, "kind": "tp_label_vs_own_envelope",
                                    "raw_tp_region": results[backend].get("region"), "audit": audit})
        if any(r["status"] != "ok" for r in results.values()):
            for backend, result in results.items():
                if result["status"] != "ok":
                    checks.append(reference_error(identifier, group, backend, result))
            continue
        if any(case["phase_audits"][b]["consistent"] is not True for b in ("PR", "HEOS")):
            checks.append({"case_id": identifier, "category": group, "backend": "PR-vs-HEOS",
                           "metric": "phase", "status": "reference_inconsistent",
                           "detail": "Raw TP label conflicts with the same backend's saturation bracket; no relabeling or forced-root substitution"})
            continue
        pr, heos = results["PR"], results["HEOS"]
        checks.append(categorical_check(identifier, group, "PR-vs-HEOS", "phase", pr["region"], heos["region"]))
        if pr["region"] != heos["region"]:
            continue
        checks.append(numeric_check(identifier, group, "PR-vs-HEOS", "beta_molar", pr["beta_molar"], heos["beta_molar"], LIMITS["beta_absolute"], False))
        for phase in heos["phases"]:
            a, b = pr["phases"][phase], heos["phases"][phase]
            checks.append(numeric_check(identifier, group, "PR-vs-HEOS", f"{phase}_density", a["mass_density_kg_m3"], b["mass_density_kg_m3"], LIMITS["density_relative"], True))
            if heos["region"] == "two_phase":
                checks.append(numeric_check(identifier, group, "PR-vs-HEOS", f"{phase}_propane_fraction", a["mole_fractions"][0], b["mole_fractions"][0], LIMITS["composition_absolute"], False))
                relative = LIMITS["phase_flow_beta_window"][0] <= heos["beta_molar"] <= LIMITS["phase_flow_beta_window"][1]
                checks.append(numeric_check(identifier, group, "PR-vs-HEOS", f"{phase}_actual_flow", a["actual_volume_flow_m3_s"], b["actual_volume_flow_m3_s"], LIMITS["phase_flow_relative"] if relative else None, relative))
        for backend, result in results.items():
            fractions = result["phases"]
            material = [sum(p["molar_fraction"] * p["mole_fractions"][i] for p in fractions.values()) for i in (0, 1)]
            errors = [abs(material[0] - case["z_propane"]), abs(material[1] - (1 - case["z_propane"]))]
            checks.append(numeric_check(identifier, "reference_self_consistency", backend, "material_fraction_residual", max(errors), 0.0, LIMITS["material_balance_relative"], False))
            for name, phase in fractions.items():
                checks.append(numeric_check(identifier, "reference_self_consistency", backend, f"{name}_normalization_residual", sum(phase["mole_fractions"]) - 1, 0.0, LIMITS["normalization_absolute"], False))
    return checks, diagnostics


def assess_pure(data: dict) -> list[dict]:
    checks = []
    for case in data["pure"]:
        pr, heos = case["results"]["PR"], case["results"]["HEOS"]
        if pr["status"] != "ok" or heos["status"] != "ok":
            for backend, result in case["results"].items():
                if result["status"] != "ok":
                    checks.append(reference_error(case["id"], "pure_model_comparison", backend, result))
            continue
        if case["kind"] == "saturation":
            for key, limit in (("pressure_pa", LIMITS["pressure_relative"]), ("liquid_density_kg_m3", LIMITS["density_relative"]), ("vapor_density_kg_m3", LIMITS["density_relative"])):
                checks.append(numeric_check(case["id"], "pure_model_comparison", "PR-vs-HEOS", key, pr[key], heos[key], limit, True))
        else:
            if any(case["phase_audits"][b]["consistent"] is not True for b in ("PR", "HEOS")):
                checks.append({"case_id": case["id"], "category": "pure_model_comparison", "backend": "PR-vs-HEOS",
                               "metric": "phase", "status": "reference_inconsistent",
                               "detail": "Raw pure-fluid TP label conflicts with the same backend saturation pressure; no relabeling"})
                continue
            checks.append(categorical_check(case["id"], "pure_model_comparison", "PR-vs-HEOS", "phase", pr["region"], heos["region"]))
            if pr["region"] == heos["region"]:
                for phase in heos["phases"]:
                    checks.append(numeric_check(case["id"], "pure_model_comparison", "PR-vs-HEOS", f"{phase}_density", pr["phases"][phase]["mass_density_kg_m3"], heos["phases"][phase]["mass_density_kg_m3"], LIMITS["density_relative"], True))
    return checks


def assess_experiments(data: dict) -> tuple[list[dict], list[dict]]:
    checks, diagnostics = [], []
    for case in data["experiments"]:
        row = case["observed"]
        for backend, results in case["results"].items():
            category = f"experiment:{case['dataset_id']}"
            def score(metric, actual, expected, limit, relative=True):
                checks.append(numeric_check(case["id"], category, backend, metric, actual, expected, limit, relative))
            for operation, result in results.items():
                if result["status"] != "ok":
                    checks.append(reference_error(case["id"], category, backend, {"operation": operation, **result}))
                    continue
                if operation in ("bubble", "saturation"):
                    if "pressure_pa" in row:
                        score("saturation_pressure", result["pressure_pa"], row["pressure_pa"], LIMITS["pressure_relative"])
                    if "y_propane" in row:
                        score("bubble_y_propane", result["y"][0], row["y_propane"], LIMITS["composition_absolute"], False)
                    if "density_kg_m3" in row:
                        score("saturated_liquid_density", result["liquid_density_kg_m3"], row["density_kg_m3"], LIMITS["density_relative"])
                elif operation == "density_at_measured_pressure":
                    score("observed_branch_density", result["density_kg_m3"], row["density_kg_m3"], LIMITS["density_relative"])
                elif operation == "tp":
                    checks.append(categorical_check(case["id"], category, backend, "tie_line_phase", result["region"], "two_phase"))
                    if result["region"] == "two_phase":
                        score("tp_x_propane", result["phases"]["liquid"]["mole_fractions"][0], row["x_propane"], LIMITS["composition_absolute"], False)
                        score("tp_y_propane", result["phases"]["vapor"]["mole_fractions"][0], row["y_propane"], LIMITS["composition_absolute"], False)
                        score("constructed_lever_rule_beta", result["beta_molar"], 0.5, LIMITS["beta_absolute"], False)
                elif operation == "unconstrained_tp" and result["region"] != case["observed_phase"]:
                    diagnostics.append({"case_id": case["id"], "backend": backend, "kind": "measured_branch_vs_unconstrained_tp",
                                        "measured_branch": case["observed_phase"], "raw_tp_region": result["region"],
                                        "note": "Branch-density comparison is retained separately; this is not proof of stable-phase agreement"})
    return checks, diagnostics


def write_engineering_inputs(data: dict, path: Path) -> int:
    engineering = PROTOCOL["engineering"]
    rows = []
    for case in data["states"]:
        if case["excluded_reasons"] or not all(case["results"][b].get("region") == "two_phase" and case["phase_audits"][b]["consistent"] for b in ("PR", "HEOS")):
            continue
        for scale in PROTOCOL["flow_scale_factors"]:
            for geometry in engineering["rating_geometries"]:
                for backend, result in case["results"].items():
                    gas, liquid = result["phases"]["vapor"], result["phases"]["liquid"]
                    rows.append([f"{case['id']}-scale{scale}-{geometry['id']}", backend, case["temperature_k"], case["pressure_pa"],
                                 gas["actual_volume_flow_m3_s"] * scale, liquid["actual_volume_flow_m3_s"] * scale,
                                 gas["mass_density_kg_m3"], liquid["mass_density_kg_m3"], engineering["effective_k_m_s"],
                                 engineering["velocity_utilization"], engineering["working_time_s"], engineering["maximum_working_liquid_height_m"],
                                 math.sqrt(4 * geometry["area_m2"] / math.pi), geometry["height_m"]])
    with path.open("w", newline="", encoding="utf-8") as file:
        writer = csv.writer(file, delimiter="\t", lineterminator="\n")
        writer.writerow(ENGINEERING_HEADER)
        writer.writerows(rows)
    return len(rows)


def assess_engineering(path: Path, inputs_path: Path) -> list[dict]:
    pairs = defaultdict(dict)
    with path.open(encoding="utf-8", newline="") as file:
        for row in csv.DictReader(file, delimiter="\t"):
            if row["backend"] in pairs[row["id"]]:
                raise ValueError("Duplicate engineering result")
            pairs[row["id"]][row["backend"]] = row
    with inputs_path.open(encoding="utf-8", newline="") as file:
        expected = {(r["id"], r["backend"]) for r in csv.DictReader(file, delimiter="\t")}
    actual = {(identifier, backend) for identifier, results in pairs.items() for backend in results}
    if actual != expected:
        raise ValueError("Engineering outputs do not cover exactly the generated inputs")
    checks = []
    for identifier, results in pairs.items():
        if set(results) != {"PR", "HEOS"}:
            raise ValueError("Engineering comparison is missing a backend")
        a, b = results["PR"], results["HEOS"]
        for field, limit in (("minimum_diameter_m", LIMITS["diameter_relative"]), ("gas_load", LIMITS["load_relative"]), ("liquid_load", LIMITS["load_relative"])):
            checks.append(numeric_check(identifier, "engineering", "PR-vs-HEOS", field, float(a[field]), float(b[field]), limit, True))
        for field in ("sizing_control", "gas_status", "liquid_status", "conclusion"):
            check = categorical_check(identifier, "engineering", "PR-vs-HEOS", field, a[field], b[field])
            if check["status"] == "fail":
                check["status"] = "review_required"
            checks.append(check)
    return checks


def summarize(checks: list[dict]) -> list[dict]:
    groups = defaultdict(list)
    for check in checks:
        groups[(check["category"], check["backend"], check["metric"])].append(check)
    summary = []
    for key, rows in sorted(groups.items()):
        numbers = [r for r in rows if "absolute_error" in r]
        worst = max(numbers, key=lambda r: r["absolute_error"]) if numbers else None
        summary.append({"category": key[0], "backend": key[1], "metric": key[2], "counts": dict(Counter(r["status"] for r in rows)),
                        "worst_case_id": None if worst is None else worst["case_id"],
                        "maximum_absolute_error": None if worst is None else worst["absolute_error"],
                        "error_basis": None if worst is None else worst["error_basis"]})
    return summary


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", required=True, type=Path)
    parser.add_argument("--engineering-results", type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args()
    data = load_json(args.reference)
    for name, digest in data["input_sha256"].items():
        if sha256(ROOT / name) != digest:
            raise ValueError(f"Frozen reference input changed: {name}")
    checks, diagnostics = assess_states(data)
    checks += assess_pure(data)
    experimental_checks, experimental_diagnostics = assess_experiments(data)
    checks += experimental_checks
    diagnostics += experimental_diagnostics
    args.output_dir.mkdir(parents=True, exist_ok=True)
    engineering_input = args.output_dir / "engineering-inputs.tsv"
    engineering_rows = write_engineering_inputs(data, engineering_input)
    if args.engineering_results:
        checks += assess_engineering(args.engineering_results, engineering_input)
    output = {"schema": "radishflow.c2a-assessment", "version": 1, "reference_sha256": sha256(args.reference),
              "engineering_results_sha256": sha256(args.engineering_results) if args.engineering_results else None,
              "engineering_input_rows": engineering_rows, "checks": checks, "summary": summarize(checks), "diagnostics": diagnostics}
    write_json(args.output_dir / "assessment.json", output)
    print("Checks:", dict(Counter(c["status"] for c in checks)))
    print(f"Reference diagnostics: {len(diagnostics)}; engineering input rows: {engineering_rows}")


if __name__ == "__main__":
    main()
