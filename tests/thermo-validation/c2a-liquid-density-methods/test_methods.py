"""Equation examples, rejected states, provenance and independent flow accounting."""

import csv
import math
import tempfile
import unittest
from pathlib import Path

from compare import (BASE, ROOT, digest, engineering_checks, engineering_inputs,
                     experiments, load, phase_comparisons, summarize, verify_baselines)
from methods import (Unavailable, costald_volume, evaluate, mixture_parameters,
                     rackett_volume, tait_factor)


class LiquidDensityEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.parameters = load(ROOT / "parameters.json")
        cls.protocol = load(ROOT / "protocol.json")
        cls.baseline = load(BASE / "reference-results.json")
        cls.results = load(ROOT / "results.json")

    def test_published_equation_examples(self):
        # ChEDL 1.5.2 documented examples, separate from candidate fluid inputs.
        volume = costald_volume(272.03889, 369.83333, 0.20008161e-3, 0.1532)
        self.assertAlmostEqual(0.044097 / volume, 530.3009967969844, places=8)
        components = [{"critical_temperature_k": t, "costald_characteristic_volume_m3_mol": v,
                       "omega_srk": omega}
                      for t, v, omega in [(512.58, .000117, .559), (647.29, 5.6e-5, .344)]]
        tc, vstar, omega, _ = mixture_parameters([.4576, .5424], components, self.parameters["gas_constant_j_mol_k"])
        self.assertLess(abs(costald_volume(298, tc, vstar, omega) - 2.706589e-5), 5e-12)
        self.assertAlmostEqual(0.000105047 * tait_factor(303, 9.8e7, 85857.9, 466.7, 3640000, .281),
                               9.287482879788505e-5, places=16)
        components = [{"critical_temperature_k": t, "critical_pressure_pa": p,
                       "molar_mass_kg_mol": m, "rackett_z_ra": z}
                      for t, p, m, z in [(512.58, 8.096e6, .03204, .2332), (647.29, 2.209e7, .01801, .2374)]]
        self.assertAlmostEqual(rackett_volume(298, [.4576, .5424], components, self.parameters["gas_constant_j_mol_k"]),
                               2.6252894930056885e-5, places=16)

    def test_no_silent_parameter_substitution(self):
        propane = self.parameters["components"][0]
        self.assertEqual(propane["omega_srk"], .1532)
        self.assertNotEqual(propane["omega_srk"], load(BASE / "parameters.json")["components"][0]["acentric_factor"])
        self.assertEqual(propane["costald_characteristic_volume_m3_mol"], .0002001)
        tc, vstar, omega, pc = mixture_parameters([1, 0], self.parameters["components"], self.parameters["gas_constant_j_mol_k"])
        self.assertAlmostEqual(tc, propane["critical_temperature_k"])
        self.assertAlmostEqual(vstar, propane["costald_characteristic_volume_m3_mol"])
        self.assertEqual(omega, propane["omega_srk"])
        self.assertEqual(pc, propane["critical_pressure_pa"])

    def test_permutation_and_pressure_monotonicity(self):
        reversed_parameters = dict(self.parameters, components=list(reversed(self.parameters["components"])))
        for method in self.protocol["methods"]:
            a = evaluate(method, 300, 8e5, [.25, .75], self.parameters)
            b = evaluate(method, 300, 8e5, [.75, .25], reversed_parameters)
            self.assertAlmostEqual(a["molar_volume_m3_mol"], b["molar_volume_m3_mol"], places=16)
            higher = evaluate(method, 300, 1e6, [.25, .75], self.parameters)
            self.assertGreater(higher["density_kg_m3"], a["density_kg_m3"])
        self.assertEqual(tait_factor(300, 1e6, 1e6, 400, 4e6, .2), 1)

    def test_invalid_and_outside_states_rejected(self):
        for xs in ([.5, .6], [-.1, 1.1], [math.nan, .5], [.5]):
            with self.assertRaises(ValueError):
                evaluate(self.protocol["methods"][0], 300, 1e6, xs, self.parameters)
        with self.assertRaises(ValueError):
            evaluate("unknown", 300, 1e6, [.5, .5], self.parameters)
        for t, p in [(279, 1e6), (321, 1e6), (300, 1e5), (300, None)]:
            with self.assertRaises(Unavailable):
                evaluate(self.protocol["methods"][0], t, p, [.5, .5], self.parameters)
        with self.assertRaises(ValueError):
            evaluate(self.protocol["methods"][0], math.nan, 1e6, [.5, .5], self.parameters)
        with self.assertRaises(Unavailable):
            tait_factor(300, 999999, 1e6, 400, 4e6, .2)

    def test_frozen_sources_and_no_lost_experimental_cases(self):
        verify_baselines()
        expected = {c["id"] for c in self.baseline["experiments"]
                    if "density_kg_m3" in c["observed"] and c.get("observed_phase") != "vapor"}
        self.assertEqual(len(expected), 47)
        self.assertEqual({c["id"] for c in self.results["experimental"]}, expected)
        self.assertEqual(experiments(self.baseline, self.parameters, self.protocol), self.results["experimental"])
        for method in self.protocol["methods"]:
            unavailable = [c["id"] for c in self.results["experimental"] if c["candidates"][method]["status"] != "ok"]
            self.assertEqual(unavailable, ["j.jct.2006.07.012/d1/p1"])

    def test_engineering_replay_and_missing_result_rejection(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "inputs.tsv"
            attempts = engineering_inputs(self.baseline, self.parameters, self.protocol, path)
            self.assertEqual(path.read_bytes(), (ROOT / "engineering-inputs.tsv").read_bytes())
            self.assertEqual(attempts, self.results["engineering_attempts"])
            self.assertEqual(phase_comparisons(self.baseline, attempts, self.protocol), self.results["phase_comparisons"])
            checks = engineering_checks(path, ROOT / "engineering-results.tsv", self.protocol)
            self.assertEqual(checks, self.results["engineering_checks"])
            self.assertEqual(summarize(self.results["experimental"], checks, self.protocol["methods"]), self.results["summary"])
            missing = Path(directory) / "missing.tsv"
            missing.write_text((ROOT / "engineering-results.tsv").read_text().rsplit("\n", 2)[0] + "\n")
            with self.assertRaises(ValueError):
                engineering_checks(path, missing, self.protocol)

    def test_phase_amounts_and_baseline_engineering_unchanged(self):
        def rows(path):
            with path.open(newline="", encoding="utf-8") as file:
                return list(csv.DictReader(file, delimiter="\t"))
        inputs, outputs = rows(ROOT / "engineering-inputs.tsv"), rows(ROOT / "engineering-results.tsv")
        self.assertEqual(len(inputs), 552)
        self.assertEqual(inputs[:276], rows(BASE / "engineering-inputs.tsv"))
        self.assertEqual(outputs[:276], rows(BASE / "engineering-results.tsv"))
        indexed = {(r["id"], r["backend"]): r for r in inputs}
        for row in self.results["engineering_attempts"]:
            candidate = indexed[row["id"], row["method"]]
            pr = indexed[row["id"], "PR"]
            for field in ("gas_flow_m3_s", "gas_density_kg_m3", "temperature_k", "pressure_pa"):
                self.assertEqual(candidate[field], pr[field])
            # rho*Q is the phase mass flow; its preservation is independent of the volume formula.
            self.assertAlmostEqual(float(candidate["liquid_density_kg_m3"]) * float(candidate["liquid_flow_m3_s"]),
                                   float(pr["liquid_density_kg_m3"]) * float(pr["liquid_flow_m3_s"]), places=12)

    def test_package_manifest(self):
        for file, sha in load(ROOT / "manifest.json")["files"].items():
            self.assertEqual(digest(ROOT / file), sha, file)


if __name__ == "__main__":
    unittest.main()
