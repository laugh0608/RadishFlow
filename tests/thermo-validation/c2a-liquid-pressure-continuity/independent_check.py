"""Independent Decimal density and SI engineering arithmetic; no candidate imports."""

from decimal import Decimal, localcontext
import csv
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parent
D = Decimal


def total(values):
    return sum(values, D(0))


def independent_density(method, temperature, pressure, fractions, saturated, parameters):
    components = parameters["components"]
    gas_constant = parameters["gas_constant_j_mol_k"]
    temperature = D(temperature)
    fractions = [D(str(value)) for value in fractions]
    volumes = [component["costald_characteristic_volume_m3_mol"] for component in components]
    one = D(1)
    vstar = (total(x * v for x, v in zip(fractions, volumes))
             + 3 * total(x * v ** (one / 3) for x, v in zip(fractions, volumes))
             * total(x * v ** (D(2) / 3) for x, v in zip(fractions, volumes))) / 4
    # Full double sum independently checks the candidate's factorized expression.
    tc = total(fractions[i] * fractions[j]
               * (components[i]["critical_temperature_k"] * volumes[i]
                  * components[j]["critical_temperature_k"] * volumes[j]).sqrt()
               for i in range(len(components)) for j in range(len(components))) / vstar
    omega = total(x * component["omega_srk"] for x, component in zip(fractions, components))
    pc = (D(".291") - D(".080") * omega) * gas_constant * tc / vstar
    tr = temperature / tc
    tau = 1 - tr
    if method == "COSTALD-Tait-uniform-Pcm-v1":
        reduced = (1 - D("1.52816") * tau ** (one / 3) + D("1.43907") * tau ** (D(2) / 3)
                   - D(".81446") * tau + D(".190454") * tau ** (D(4) / 3))
        delta = (-D(".296123") + D(".386914") * tr - D(".0427258") * tr**2
                 - D(".0480645") * tr**3) / (tr - D("1.00001"))
        volume = vstar * reduced * (1 - omega * delta)
    elif method == "modified-Rackett-Tait-uniform-Pcm-v1":
        rackett_tc = total(x * component["critical_temperature_k"] for x, component in zip(fractions, components))
        zr = total(x * component["rackett_z_ra"] for x, component in zip(fractions, components))
        mw = total(x * component["molar_mass_kg_mol"] for x, component in zip(fractions, components))
        weighted = total(x * component["critical_temperature_k"]
                         / (component["molar_mass_kg_mol"] * component["critical_pressure_pa"])
                         for x, component in zip(fractions, components))
        volume = gas_constant * mw * weighted * zr ** (1 + (1 - temperature / rackett_tc) ** (D(2) / 7))
    else:
        raise ValueError(f"Unknown candidate {method}")
    log_tr = tr.log10()
    alpha = 35 - 36 / tr - D("96.736") * log_tr + tr**6
    ps = pc * D(10) ** (D("5.8031817") * log_tr + D(".07608141") * alpha
                       + omega * D("4.86601") * (log_tr + D(".03721754") * alpha))
    if not saturated:
        pressure = D(pressure)
        if pressure < ps:
            return None
        b = pc * (-1 - D("9.070217") * tau ** (one / 3) + D("62.45326") * tau ** (D(2) / 3)
                  - D("135.1102") * tau
                  + (D("4.79594") + D(".250047") * omega + D("1.14188") * omega**2).exp()
                  * tau ** (D(4) / 3))
        # Direct logarithmic ratio, independent of the candidate's log1p evaluation.
        volume *= 1 - (D(".0861488") + D(".0344483") * omega) * ((b + pressure) / (b + ps)).ln()
    return total(x * component["molar_mass_kg_mol"] for x, component in zip(fractions, components)) / volume


def verify_densities(parameters, results):
    differences = []
    unavailable = 0
    for row in results["experimental"]:
        observation = row["observed"]
        fractions = [row["x_propane"], 1 - row["x_propane"]]
        for method, result in row["candidates"].items():
            expected = independent_density(method, observation["temperature_k"], observation.get("pressure_pa"),
                                           fractions, row["saturation_observation"], parameters)
            if expected is None:
                if result["status"] != "unavailable":
                    raise ValueError("Independent availability disagrees")
                unavailable += 1
            else:
                if result["status"] != "ok":
                    raise ValueError("Independent density disagrees with unavailable result")
                differences.append(abs(result["density_kg_m3"] / expected - 1))
    seen = set()
    for row in results["engineering_attempts"]:
        key = row["case_id"], row["method"]
        if key in seen:
            continue
        seen.add(key)
        expected = independent_density(row["method"], row["temperature_k"], row["pressure_pa"],
                                       row["pr_liquid_mole_fractions"], False, parameters)
        if expected is None or row["result"]["status"] != "ok":
            raise ValueError("Independent engineering-phase availability disagrees")
        differences.append(abs(row["result"]["density_kg_m3"] / expected - 1))
    if len(differences) != 138 or unavailable != 2 or max(differences) >= D("1e-12"):
        raise ValueError("Independent density coverage or arithmetic differs")
    return differences, unavailable


def read_rows(path):
    with path.open(encoding="utf-8", newline="") as file:
        return list(csv.DictReader(file, delimiter="\t"))


def verify_engineering():
    inputs = read_rows(ROOT / "engineering-inputs.tsv")
    outputs = read_rows(ROOT / "engineering-results.tsv")
    if len(inputs) != 552 or len(outputs) != 552:
        raise ValueError("Engineering row coverage differs")
    differences = []
    for inputs_row, result in zip(inputs, outputs):
        if (inputs_row["id"], inputs_row["backend"]) != (result["id"], result["backend"]):
            raise ValueError("Engineering identity differs")
        values = {key: float(value) for key, value in inputs_row.items() if key not in ("id", "backend")}
        allowable_velocity = (values["effective_k_m_s"] * values["velocity_utilization"]
                              * math.sqrt((values["liquid_density_kg_m3"] - values["gas_density_kg_m3"])
                                          / values["gas_density_kg_m3"]))
        gas_area = values["gas_flow_m3_s"] / allowable_velocity
        liquid_area = values["liquid_flow_m3_s"] * values["working_time_s"] / values["maximum_working_height_m"]
        rating_area = math.pi * values["rating_diameter_m"]**2 / 4
        expected = {"minimum_diameter_m": math.sqrt(4 * max(gas_area, liquid_area) / math.pi),
                    "gas_load": values["gas_flow_m3_s"] / (rating_area * allowable_velocity),
                    "liquid_load": values["liquid_flow_m3_s"] * values["working_time_s"]
                                   / (rating_area * values["rating_height_m"])}
        for key, value in expected.items():
            differences.append(abs(float(result[key]) / value - 1))
        statuses = {"sizing_control": "GasArea" if gas_area > liquid_area else "LiquidWorkingVolume",
                    "gas_status": "Exceeded" if expected["gas_load"] > 1 else "WithinCriteria",
                    "liquid_status": "Exceeded" if expected["liquid_load"] > 1 else "WithinCriteria"}
        statuses["conclusion"] = "Exceeded" if "Exceeded" in (statuses["gas_status"], statuses["liquid_status"]) else "WithinCriteria"
        if any(result[key] != value for key, value in statuses.items()):
            raise ValueError("Independent engineering verdict differs")
    if len(differences) != 1656 or max(differences) >= 1e-12:
        raise ValueError("Independent engineering arithmetic differs")
    return differences, len(inputs)


def main():
    with localcontext() as context:
        context.prec = 60
        parameters = json.loads((ROOT / "parameters.json").read_text(encoding="utf-8"), parse_float=D)
        results = json.loads((ROOT / "results.json").read_text(encoding="utf-8"), parse_float=D)
        density, unavailable = verify_densities(parameters, results)
    engineering, engineering_rows = verify_engineering()
    print(json.dumps({"density_values_checked": len(density), "unavailable_retained": unavailable,
                      "maximum_density_relative_difference_60_digit_decimal": float(max(density)),
                      "engineering_rows_checked": engineering_rows, "independent_numeric_checks": len(engineering),
                      "maximum_engineering_relative_difference": max(engineering),
                      "engineering_control_and_verdict_rows_checked": engineering_rows,
                      "policy": "Independent arithmetic of uniform-Pcm-v1; no physical qualification or C2-B/C3 approval."}, indent=2))


if __name__ == "__main__":
    main()
