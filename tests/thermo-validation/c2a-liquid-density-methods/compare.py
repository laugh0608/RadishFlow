"""Reproduce C2-A liquid-density screening without changing accepted evidence."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
from collections import Counter, defaultdict
from pathlib import Path

from methods import Unavailable, evaluate

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
BASE = ROOT.parent / "c2a-pr76"


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(path: Path, value) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def verify_baselines() -> None:
    for name, expected in load(ROOT / "protocol.json")["baseline_manifests"].items():
        directory = ROOT.parent / name
        if digest(directory / "manifest.json") != expected:
            raise ValueError(f"Accepted manifest changed: {name}")
        manifest = load(directory / "manifest.json")
        file_root = REPO if "repository_relative_sha256" in manifest else directory
        for file, sha in manifest.get("repository_relative_sha256", manifest.get("files", {})).items():
            current = digest(file_root / file)
            drift = load(ROOT / "protocol.json").get("historical_repository_drift", {}).get(file)
            if drift and sha == drift["historical_sha256"] and current == drift["current_sha256"]:
                continue
            if current != sha:
                raise ValueError(f"Accepted evidence changed: {file}")


def attempt(method: str, t: float, p: float | None, xs: list[float], parameters: dict, saturated=False) -> dict:
    try:
        return {"status": "ok", **evaluate(method, t, p, xs, parameters, saturated)}
    except Unavailable as error:
        return {"status": "unavailable", "reason": str(error)}


def experiments(data: dict, parameters: dict, protocol: dict) -> list[dict]:
    output = []
    for case in data["experiments"]:
        row = case["observed"]
        if "density_kg_m3" not in row or case.get("observed_phase") == "vapor":
            continue
        saturated = case["kind"] == "bubble_density" or "pressure_pa" not in row
        if case["fluid"] is None:
            x = row["x_propane"]
        elif case["fluid"] == "Propane":
            x = 1.0
        elif case["fluid"] == "n-Butane":
            x = 0.0
        else:
            raise ValueError(f"Unknown experimental fluid: {case['fluid']}")
        operation = "saturation" if saturated else "density_at_measured_pressure"
        baselines = {backend: result[operation]["liquid_density_kg_m3" if saturated else "density_kg_m3"]
                     for backend, result in case["results"].items()}
        candidates = {}
        for method in protocol["methods"]:
            result = attempt(method, row["temperature_k"], row.get("pressure_pa"), [x, 1 - x], parameters, saturated)
            if result["status"] == "ok":
                error = result["density_kg_m3"] / row["density_kg_m3"] - 1
                result.update(signed_relative_error=error, absolute_relative_error=abs(error),
                              verdict="pass" if abs(error) <= protocol["thresholds"]["density_relative"] else "fail")
            candidates[method] = result
        output.append({"id": case["id"], "dataset_id": case["dataset_id"], "observed": row,
                       "x_propane": x, "saturation_observation": saturated,
                       "uncertainty_source": "Unchanged baseline source/dataset metadata; expanded_uncertainty may be pressure, paired density uncertainty is separately retained",
                       "baseline_operation": operation, "baseline_densities_kg_m3": baselines,
                       "candidates": candidates})
    return output


def engineering_inputs(data: dict, parameters: dict, protocol: dict, output: Path) -> list[dict]:
    states = {c["id"]: c for c in data["states"]}
    with (BASE / "engineering-inputs.tsv").open(encoding="utf-8", newline="") as file:
        reader = csv.DictReader(file, delimiter="\t")
        fields, rows = reader.fieldnames, list(reader)
    extra, attempts = [], []
    for row in rows:
        if row["backend"] != "PR":
            continue
        case = states[row["id"].rsplit("-scale", 1)[0]]
        liquid = case["results"]["PR"]["phases"]["liquid"]
        for method in protocol["methods"]:
            result = attempt(method, case["temperature_k"], case["pressure_pa"], liquid["mole_fractions"], parameters)
            attempts.append({"id": row["id"], "case_id": case["id"], "method": method,
                             "temperature_k": case["temperature_k"], "pressure_pa": case["pressure_pa"],
                             "pr_liquid_mole_fractions": liquid["mole_fractions"], "result": result})
            if result["status"] != "ok":
                continue
            candidate = dict(row, backend=method)
            candidate["liquid_density_kg_m3"] = str(result["density_kg_m3"])
            candidate["liquid_flow_m3_s"] = str(float(row["liquid_flow_m3_s"])
                                               * result["molar_volume_m3_mol"] / liquid["molar_volume_m3_mol"])
            extra.append(candidate)
    with output.open("w", encoding="utf-8", newline="") as file:
        writer = csv.DictWriter(file, fields, delimiter="\t", lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows + extra)
    return attempts


def engineering_checks(inputs: Path, results: Path, protocol: dict) -> list[dict]:
    with inputs.open(encoding="utf-8", newline="") as file:
        expected = {(r["id"], r["backend"]) for r in csv.DictReader(file, delimiter="\t")}
    groups = defaultdict(dict)
    with results.open(encoding="utf-8", newline="") as file:
        for row in csv.DictReader(file, delimiter="\t"):
            if row["backend"] in groups[row["id"]]:
                raise ValueError("Duplicate engineering result")
            groups[row["id"]][row["backend"]] = row
    if {(k, b) for k, v in groups.items() for b in v} != expected:
        raise ValueError("Engineering result coverage differs from generated inputs")
    checks = []
    for identifier, group in groups.items():
        for method in protocol["methods"]:
            if method not in group:
                continue
            for reference in ("PR", "HEOS"):
                candidate, baseline = group[method], group[reference]
                for field, limit in (("minimum_diameter_m", "diameter_relative"), ("gas_load", "load_relative"), ("liquid_load", "load_relative")):
                    value, target = float(candidate[field]), float(baseline[field])
                    if target == 0:
                        raise ValueError("Undefined relative engineering comparison")
                    error = value / target - 1
                    checks.append({"id": identifier, "method": method, "reference": reference,
                                   "metric": field, "signed_relative_error": error, "absolute_relative_error": abs(error),
                                   "verdict": "pass" if abs(error) <= protocol["thresholds"][limit] else "fail"})
                for field in ("sizing_control", "gas_status", "liquid_status", "conclusion"):
                    checks.append({"id": identifier, "method": method, "reference": reference, "metric": field,
                                   "candidate": candidate[field], "reference_value": baseline[field],
                                   "verdict": "pass" if candidate[field] == baseline[field] else "review_required"})
    return checks


def phase_comparisons(data: dict, attempts: list[dict], protocol: dict) -> list[dict]:
    states = {c["id"]: c for c in data["states"]}
    seen, output = set(), []
    for attempt_row in attempts:
        key = attempt_row["case_id"], attempt_row["method"]
        if key in seen:
            continue
        seen.add(key)
        case, result = states[key[0]], attempt_row["result"]
        row = {"case_id": key[0], "method": key[1], "result": result}
        if result["status"] == "ok":
            pr, heos = case["results"]["PR"], case["results"]["HEOS"]
            liquid = pr["phases"]["liquid"]
            q = liquid["molar_flow_mol_s"] * result["molar_volume_m3_mol"]
            target = heos["phases"]["liquid"]
            low, high = protocol["thresholds"]["phase_flow_beta_window"]
            scored = low <= heos["beta_molar"] <= high
            error = q / target["actual_volume_flow_m3_s"] - 1
            row.update(pr_beta_molar=pr["beta_molar"], heos_beta_molar=heos["beta_molar"],
                       liquid_flow_m3_s=q, heos_liquid_flow_m3_s=target["actual_volume_flow_m3_s"],
                       liquid_flow_absolute_error_m3_s=q - target["actual_volume_flow_m3_s"],
                       liquid_flow_signed_relative_error=error,
                       liquid_flow_verdict=("pass" if abs(error) <= protocol["thresholds"]["phase_flow_relative"] else "fail") if scored else "unscored_near_vanishing_phase",
                       liquid_density_signed_relative_error=result["density_kg_m3"] / target["mass_density_kg_m3"] - 1,
                       vapor_flow_m3_s=pr["phases"]["vapor"]["actual_volume_flow_m3_s"],
                       vapor_density_kg_m3=pr["phases"]["vapor"]["mass_density_kg_m3"])
        output.append(row)
    return output


def summarize(experimental: list[dict], engineering: list[dict], methods: list[str]) -> dict:
    output = {"experimental": {}, "engineering": []}
    for method in methods:
        values = [(r["id"], r["candidates"][method]) for r in experimental]
        scored = [(i, v) for i, v in values if v["status"] == "ok"]
        worst = max(scored, key=lambda pair: pair[1]["absolute_relative_error"]) if scored else None
        output["experimental"][method] = {"counts": dict(Counter(v.get("verdict", "unavailable") for _, v in values)),
                                           "worst_case_id": None if worst is None else worst[0],
                                           "maximum_absolute_relative_error": None if worst is None else worst[1]["absolute_relative_error"],
                                           "unavailable_reasons": dict(Counter(v["reason"] for _, v in values if v["status"] != "ok"))}
    groups = defaultdict(list)
    for row in engineering:
        groups[(row["method"], row["reference"], row["metric"])].append(row)
    for (method, reference, metric), rows in sorted(groups.items()):
        numeric = [r for r in rows if "absolute_relative_error" in r]
        worst = max(numeric, key=lambda r: r["absolute_relative_error"]) if numeric else None
        output["engineering"].append({"method": method, "reference": reference, "metric": metric,
                                     "counts": dict(Counter(r["verdict"] for r in rows)),
                                     "worst_case_id": None if worst is None else worst["id"],
                                     "maximum_absolute_relative_error": None if worst is None else worst["absolute_relative_error"]})
    return output


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--engineering-results", type=Path)
    args = parser.parse_args()
    if args.output_dir.resolve() in [(ROOT.parent / n).resolve() for n in load(ROOT / "protocol.json")["baseline_manifests"]]:
        raise ValueError("Cannot overwrite accepted baseline")
    verify_baselines()
    parameters, protocol = load(ROOT / "parameters.json"), load(ROOT / "protocol.json")
    data = load(BASE / "reference-results.json")
    args.output_dir.mkdir(parents=True, exist_ok=True)
    experimental = experiments(data, parameters, protocol)
    inputs = args.output_dir / "engineering-inputs.tsv"
    attempts = engineering_inputs(data, parameters, protocol, inputs)
    checks = engineering_checks(inputs, args.engineering_results, protocol) if args.engineering_results else []
    report = {"schema": "radishflow.c2a-liquid-density-methods", "version": 1,
              "protocol_sha256": digest(ROOT / "protocol.json"), "parameters_sha256": digest(ROOT / "parameters.json"),
              "baseline_reference_sha256": digest(BASE / "reference-results.json"),
              "engineering_results_sha256": digest(args.engineering_results) if args.engineering_results else None,
              "experimental": experimental, "phase_comparisons": phase_comparisons(data, attempts, protocol),
              "engineering_attempts": attempts, "engineering_checks": checks,
              "summary": summarize(experimental, checks, protocol["methods"]),
              "unchanged_gates": ["No continuous qualified domain", "VLE conflicts remain", "Binary gas-density evidence missing", "PR phase-label qualification incomplete", "Original failures retained; C2-B/C3 not started"]}
    write(args.output_dir / "results.json", report)
    print(json.dumps({"experimental_cases": len(experimental), "engineering_candidate_attempts": len(attempts),
                      "available_attempts": sum(a["result"]["status"] == "ok" for a in attempts),
                      "summary": report["summary"]["experimental"]}, ensure_ascii=False))


if __name__ == "__main__":
    main()
