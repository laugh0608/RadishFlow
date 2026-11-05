"""Explicit composition and pressure probes, separate from experimental scoring."""

from methods import Unavailable, evaluate, mixture_parameters, riedel_pressure


def pressure_boundaries(parameters: dict, protocol: dict) -> list[dict]:
    rows = []
    for t in protocol["boundary_validation"]["temperature_k"]:
        for x in protocol["boundary_validation"]["x_propane"]:
            xs = [x, 1 - x]
            tc, vstar, omega, pc = mixture_parameters(xs, parameters["components"], parameters["gas_constant_j_mol_k"])
            ps = riedel_pressure(t, tc, pc, omega)
            for label, pressure in (("one_mpa", 1e6), ("below_ps", ps * (1 - 1e-8)),
                                    ("at_ps", ps), ("above_ps", ps * (1 + 1e-8))):
                for method in protocol["methods"]:
                    try:
                        result = {"status": "ok", **evaluate(method, t, pressure, xs, parameters)}
                    except Unavailable as error:
                        result = {"status": "unavailable", "reason": str(error)}
                    rows.append({"temperature_k": t, "x_propane": x, "pressure_probe": label,
                                 "pressure_pa": pressure, "correlation_psat_pa": ps,
                                 "correlation_pcm_pa": pc, "method": method, "result": result})
    return rows


def compare_previous(experimental: list[dict], previous: dict, protocol: dict) -> list[dict]:
    old_cases = {r["id"]: r for r in previous["experimental"]}
    if set(old_cases) != {r["id"] for r in experimental}:
        raise ValueError("Experimental identities changed from previous candidate")
    rows = []
    for case in experimental:
        old_case = old_cases[case["id"]]
        if case["observed"] != old_case["observed"] or case["x_propane"] != old_case["x_propane"]:
            raise ValueError("Experimental inputs changed from previous candidate")
        for method, old_method in protocol["previous_method_names"].items():
            new, old = case["candidates"][method], old_case["candidates"][old_method]
            row = {"id": case["id"], "method": method, "previous_method": old_method,
                   "previous_status": old["status"], "status": new["status"],
                   "previous_verdict": old.get("verdict"), "verdict": new.get("verdict")}
            if new["status"] == old["status"] == "ok":
                row["density_signed_relative_change"] = new["density_kg_m3"] / old["density_kg_m3"] - 1
                row["correlation_psat_change_pa"] = new["correlation_psat_pa"] - old["correlation_psat_pa"]
            rows.append(row)
    return rows
