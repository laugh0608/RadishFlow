"""Aggregate phase controls and new measurement checks without merging baseline scores."""
from collections import Counter
from common import ROOT, load_json, write_json, verify_baseline


def summarize():
    verify_baseline()
    current = load_json(ROOT / "phase-8.0.0.json")
    previous = load_json(ROOT / "phase-7.2.0.json")
    cases = current["cases"]
    anomalous = [c for c in cases if c["baseline_audit"]["consistent"] is False]
    matched = []
    for case in anomalous:
        actual = case["operations"]["fresh"]["density_mol_m3"]
        root = case["independent_homogeneous_PR"]["minimum_homogeneous_gibbs_density_mol_m3"]
        imposed = case["operations"]["imposed_gas" if case["baseline_audit"]["envelope_region"] == "vapor" else "imposed_liquid"]
        matched.append({"id": case["id"], "in_domain": not case.get("excluded_reasons"),
                        "independent_root_relative_error": abs(actual/root-1),
                        "imposed_root_relative_error": abs(actual/imposed["density_mol_m3"]-1),
                        "root_comparison_limit": 1e-6})
    return {"phase": {"total_states": len(cases), "raw_label_anomalies": len(anomalous), "in_domain_anomalies": sum(not c.get("excluded_reasons") for c in anomalous),
                      "repeat_and_default_parameters_exact": all(c["operations"]["fresh"] == c["operations"]["repeat"] == c["operations"]["default_parameters"] for c in cases),
                      "diagnostic_root_checks": matched,
                      "v7_fresh_statuses": dict(Counter(c["operations"]["fresh"]["status"] for c in previous["cases"])),
                      "v7_fresh_phases": dict(Counter(c["operations"]["fresh"].get("phase", "error") for c in previous["cases"]))},
            "new_experiments": load_json(ROOT / "experimental-results.json")["assessment"]["summary"],
            "baseline_scores": "Unchanged; never aggregate this supplement into a replacement acceptance score"}

if __name__ == "__main__":
    write_json(ROOT / "summary.json", summarize())
