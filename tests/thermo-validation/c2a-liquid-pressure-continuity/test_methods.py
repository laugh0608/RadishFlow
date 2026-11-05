"""Equation examples, rejected states, provenance and independent flow accounting."""

import csv
import math
import tempfile
import unittest
from pathlib import Path

from compare import (BASE, ROOT, digest, engineering_checks, engineering_inputs,
                     experiments, load, phase_comparisons, summarize, verify_baselines)
from methods import (Unavailable, costald_volume, evaluate, mixture_parameters,
                     rackett_volume, riedel_pressure, tait_factor)


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
        self.assertAlmostEqual(pc, 4284155.389556637, places=6)
        self.assertNotEqual(pc, propane["critical_pressure_pa"])
        # Physical Pc is retained for Rackett saturated volume; only pressure Pcm changes.
        self.assertEqual(propane["critical_pressure_pa"], 4251200.0)

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

    def test_pressure_parameters_and_density_have_both_pure_limits(self):
        parameters, protocol = self.parameters, self.protocol
        for t in (280, 300, 320):
            for pure in ([1., 0.], [0., 1.]):
                endpoint = mixture_parameters(pure, parameters["components"], parameters["gas_constant_j_mol_k"])
                for epsilon in (1e-6, 1e-9, 1e-12):
                    xs = [1 - epsilon, epsilon] if pure[0] else [epsilon, 1 - epsilon]
                    trace = mixture_parameters(xs, parameters["components"], parameters["gas_constant_j_mol_k"])
                    for actual, target in zip(trace, endpoint):
                        self.assertLess(abs(actual / target - 1), 10 * epsilon)
                xs = [1 - 1e-12, 1e-12] if pure[0] else [1e-12, 1 - 1e-12]
                for method in protocol["methods"]:
                    try:
                        exact = evaluate(method, t, 1e6, pure, parameters)
                    except Unavailable:
                        with self.assertRaises(Unavailable):
                            evaluate(method, t, 1e6, xs, parameters)
                    else:
                        trace = evaluate(method, t, 1e6, xs, parameters)
                        self.assertLess(abs(trace["density_kg_m3"] / exact["density_kg_m3"] - 1), 1e-9)
                        self.assertLess(abs(trace["correlation_psat_pa"] / exact["correlation_psat_pa"] - 1), 1e-9)

    def test_old_endpoint_availability_counterexample_no_longer_switches(self):
        for method in self.protocol["methods"]:
            for xs in ([1, 0], [1 - 1e-12, 1e-12]):
                with self.assertRaisesRegex(Unavailable, "pressure_below_correlation"):
                    evaluate(method, 280, 587000, xs, self.parameters)
            # Above both old and new Ps, the new pure and trace states agree.
            a = evaluate(method, 280, 590000, [1, 0], self.parameters)
            b = evaluate(method, 280, 590000, [1 - 1e-12, 1e-12], self.parameters)
            self.assertLess(abs(a["density_kg_m3"] / b["density_kg_m3"] - 1), 1e-9)

    def test_zero_components_and_permutation_do_not_select_a_different_rule(self):
        for method in self.protocol["methods"]:
            for xs in ([1, 0], [.3, .7], [0, 1]):
                expected = evaluate(method, 280, 1e6, xs, self.parameters)
                padded = dict(self.parameters, components=self.parameters["components"] + [self.parameters["components"][0]])
                actual = evaluate(method, 280, 1e6, list(xs) + [0], padded)
                reversed_parameters = dict(self.parameters, components=list(reversed(padded["components"])))
                reversed_result = evaluate(method, 280, 1e6, list(reversed(list(xs) + [0])), reversed_parameters)
                results = [actual, reversed_result]
                if 1 in xs:
                    component = self.parameters["components"][xs.index(1)]
                    pure_parameters = dict(self.parameters, components=[component])
                    results.append(evaluate(method, 280, 1e6, [1], pure_parameters))
                for result in results:
                    self.assertLess(abs(result["density_kg_m3"] / expected["density_kg_m3"] - 1), 1e-12)
                    self.assertLess(abs(result["correlation_psat_pa"] / expected["correlation_psat_pa"] - 1), 1e-12)

    def test_saturation_and_compression_join_at_ps_without_clamping(self):
        for t, xs in ((280, [1, 0]), (300, [.5, .5]), (320, [0, 1])):
            tc, _, omega, pc = mixture_parameters(xs, self.parameters["components"], self.parameters["gas_constant_j_mol_k"])
            ps = riedel_pressure(t, tc, pc, omega)
            self.assertTrue(2e5 < ps < 1e6)
            for method in self.protocol["methods"]:
                saturated = evaluate(method, t, None, xs, self.parameters, saturated=True)
                at = evaluate(method, t, ps, xs, self.parameters)
                above = evaluate(method, t, ps * (1 + 1e-8), xs, self.parameters)
                self.assertEqual(at["pressure_factor"], 1)
                self.assertEqual(at["molar_volume_m3_mol"], saturated["molar_volume_m3_mol"])
                self.assertGreater(above["density_kg_m3"], at["density_kg_m3"])
                with self.assertRaisesRegex(Unavailable, "pressure_below_correlation"):
                    evaluate(method, t, ps * (1 - 1e-8), xs, self.parameters)

    def test_boundary_matrix_retains_outside_tp_points(self):
        from boundaries import pressure_boundaries
        rows = pressure_boundaries(self.parameters, self.protocol)
        self.assertEqual(rows, self.results["pressure_boundaries"])
        self.assertEqual(len(rows), 3 * 11 * 4 * 2)
        unavailable = [r for r in rows if r["result"]["status"] == "unavailable"]
        self.assertTrue(any(r["result"]["reason"] == "outside_original_temperature_or_pressure_domain" for r in unavailable))
        for row in rows:
            inside = 2e5 <= row["pressure_pa"] <= 1e6
            if inside and row["pressure_probe"] == "below_ps":
                self.assertEqual(row["result"]["status"], "unavailable")
            if inside and row["pressure_probe"] == "at_ps":
                self.assertEqual(row["result"]["pressure_factor"], 1)

    def test_all_previous_interior_and_saturation_outputs_remain_unchanged(self):
        previous = load(ROOT.parent / "c2a-liquid-density-methods" / "results.json")
        old_rows = {r["id"]: r for r in previous["experimental"]}
        for row in self.results["experimental"]:
            for method, old_method in self.protocol["previous_method_names"].items():
                new, old = row["candidates"][method], old_rows[row["id"]]["candidates"][old_method]
                self.assertEqual(new["status"], old["status"])
                if new["status"] == "ok" and (row["saturation_observation"] or 0 < row["x_propane"] < 1):
                    self.assertEqual(new["density_kg_m3"], old["density_kg_m3"])
        def rows(path):
            with path.open(newline="", encoding="utf-8") as file:
                return list(csv.DictReader(file, delimiter="\t"))
        old = rows(ROOT.parent / "c2a-liquid-density-methods" / "engineering-results.tsv")
        current = rows(ROOT / "engineering-results.tsv")
        self.assertEqual(len(current), len(old))
        for new, previous in zip(current, old):
            normalized = dict(new, backend=self.protocol["previous_method_names"].get(new["backend"], new["backend"]))
            self.assertEqual(normalized, previous)

    def test_candidate_identity_and_parameters_are_separate_from_old_evidence(self):
        previous = ROOT.parent / "c2a-liquid-density-methods"
        self.assertEqual((ROOT / "parameters.json").read_bytes(), (previous / "parameters.json").read_bytes())
        self.assertTrue(set(self.protocol["methods"]).isdisjoint(load(previous / "protocol.json")["methods"]))
        self.assertEqual(self.protocol["thresholds"], load(previous / "protocol.json")["thresholds"])
        self.assertEqual(self.protocol["baseline_manifests"]["c2a-liquid-density-methods"], digest(previous / "manifest.json"))


if __name__ == "__main__":
    unittest.main()
