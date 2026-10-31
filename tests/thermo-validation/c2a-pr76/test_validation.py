"""Checks for evidence selection and assessment integrity, not product physics."""
import copy
import csv
import math
import tempfile
import unittest
from pathlib import Path

from assess import ENGINEERING_HEADER, assess_engineering, numeric_check
from reference import domain_reasons, joined_rows, generate_experiments
from thermoml import ROOT, load_json, sha256, variable_descriptor


class EvidenceTests(unittest.TestCase):
    def test_frozen_manifest(self):
        manifest = load_json(ROOT / "manifest.json")
        repository = ROOT.parents[2]
        for name, digest in manifest["repository_relative_sha256"].items():
            self.assertEqual(sha256(repository / name), digest, name)

    def test_pressure_conversion_and_unknown_units(self):
        self.assertEqual(variable_descriptor({"VariableID": {"VariableType": {"ePressure": "Pressure, kPa"}}}), ("pressure_pa", 1000.0))
        with self.assertRaises(ValueError):
            variable_descriptor({"VariableID": {"VariableType": {"ePressure": "Pressure, bar"}}})

    def test_join_uses_temperature_and_composition_not_order(self):
        left = {"rows": [{"temperature_k": 300, "x_propane": x} for x in (0.2, 0.5)]}
        right = {"property": "y_propane", "rows": [
            {"temperature_k": 300, "x_propane": x, "y_propane": y, "id": str(x), "expanded_uncertainty": 0.01}
            for x, y in ((0.5, 0.8), (0.2, 0.6))]}
        self.assertEqual([r["y_propane"] for r in joined_rows(left, right)], [0.6, 0.8])
        bad = copy.deepcopy(right)
        bad["rows"][1] = bad["rows"][0]
        with self.assertRaises(ValueError):
            joined_rows(left, bad)

    def test_domain_boundaries_and_outside_are_explicit(self):
        self.assertEqual(domain_reasons(280, 200000, 0.2), [])
        self.assertEqual(domain_reasons(320, 1000000, 0.8), [])
        self.assertTrue(domain_reasons(279.999, 200000, 0.2))
        self.assertTrue(domain_reasons(300, 1000001, 0.5))

    def test_no_zero_denominator_or_nonfinite_score(self):
        for actual, expected in ((1, 0), (float("nan"), 1), (1, float("inf"))):
            with self.assertRaises(ValueError):
                numeric_check("case", "category", "PR", "rho", actual, expected, 0.05, True)
        self.assertEqual(numeric_check("c", "g", "PR", "q", 0.01, 0, None, False)["status"], "unscored")

    def test_engineering_missing_pairs_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            inputs, outputs = Path(directory) / "inputs.tsv", Path(directory) / "outputs.tsv"
            with inputs.open("w", newline="") as file:
                writer = csv.writer(file, delimiter="\t")
                writer.writerow(ENGINEERING_HEADER)
                writer.writerow(["case", "PR", *([1] * 12)])
                writer.writerow(["case", "HEOS", *([1] * 12)])
            outputs.write_text("id\tbackend\ncase\tPR\n")
            with self.assertRaises(ValueError):
                assess_engineering(outputs, inputs)

    def test_engineering_harness_matches_independent_arithmetic(self):
        with (ROOT / "engineering-inputs.tsv").open() as file:
            inputs = {(r["id"], r["backend"]): r for r in csv.DictReader(file, delimiter="\t")}
        with (ROOT / "engineering-results.tsv").open() as file:
            outputs = {(r["id"], r["backend"]): r for r in csv.DictReader(file, delimiter="\t")}
        self.assertEqual(set(inputs), set(outputs))
        for key, row in inputs.items():
            v = {k: float(value) for k, value in row.items() if k not in ("id", "backend")}
            velocity = v["effective_k_m_s"] * v["velocity_utilization"] * math.sqrt((v["liquid_density_kg_m3"] - v["gas_density_kg_m3"]) / v["gas_density_kg_m3"])
            gas_area = v["gas_flow_m3_s"] / velocity
            liquid_area = v["liquid_flow_m3_s"] * v["working_time_s"] / v["maximum_working_height_m"]
            area = math.pi * v["rating_diameter_m"] ** 2 / 4
            expected = {"minimum_diameter_m": math.sqrt(4 * max(gas_area, liquid_area) / math.pi),
                        "gas_load": gas_area / area,
                        "liquid_load": v["liquid_flow_m3_s"] * v["working_time_s"] / (area * v["rating_height_m"])}
            for field, value in expected.items():
                self.assertTrue(math.isclose(float(outputs[key][field]), value, rel_tol=1e-12), (key, field))

    def test_all_selected_measurements_accounted_for(self):
        data = load_json(ROOT / "experimental-data.json")
        cases, exclusions = generate_experiments(data)
        used = [row["id"] for row in cases + exclusions]
        used += [row["observed"]["paired_id"] for row in cases if "paired_id" in row["observed"]]
        used += [row["paired_id"] for row in exclusions if row.get("paired_id")]
        expected = [row["id"] for dataset in data["datasets"] for row in dataset["rows"]]
        self.assertEqual(len(used), len(set(used)))
        self.assertEqual(set(used), set(expected))


if __name__ == "__main__":
    unittest.main()
