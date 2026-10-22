"""Evidence integrity, exact pairing and independent thermodynamic diagnostics."""
import math
import unittest
from collections import Counter
from common import ROOT, BASELINE, load_json, sha256, verify_baseline
from experiments import join_partial, assess


class EvidenceReviewTests(unittest.TestCase):
    def test_accepted_baseline_is_unchanged(self):
        verify_baseline()

    def test_all_selected_measurements_are_accounted_once(self):
        data = load_json(ROOT / "experimental-data.json")
        expected = [r["id"] for d in data["datasets"] for r in d["rows"]]
        self.assertEqual(len(expected), 131)
        results = load_json(ROOT / "experimental-results.json")
        observed = []
        for case in results["cases"] + results["excluded"]:
            row = case["observed"]
            observed.append(row["id"])
            if "paired_id" in row:
                observed.append(row["paired_id"])
        observed.extend(r["id"] for r in results["unpaired"])
        self.assertEqual(Counter(expected), Counter(observed))
        self.assertEqual(len(set(observed)), len(observed))

    def test_exact_join_keeps_missing_row_and_rejects_duplicate(self):
        pressure = {"id": "P", "temperature_k": 300., "x_propane": .3, "pressure_pa": 400000.}
        vapor = {"id": "Y", "temperature_k": 300., "x_propane": .30000001, "y_propane": .6, "expanded_uncertainty": .001}
        paired, unmatched = join_partial([pressure], [vapor])
        self.assertEqual(paired, [])
        self.assertEqual({r["id"] for r in unmatched}, {"P", "Y"})
        with self.assertRaises(ValueError):
            join_partial([pressure, pressure], [])

    def test_vle_and_gas_density_coverage(self):
        results = load_json(ROOT / "experimental-results.json")
        self.assertEqual(Counter(c["kind"] for c in results["cases"]), {"binary_vle": 10, "pure_vapor_TP_density": 5, "pure_saturated_vapor_density": 3})
        unpaired = [r for r in results["unpaired"] if r["id"].startswith("je0496701")]
        self.assertEqual(unpaired, [{"id": "je0496701/d1/p15", "reason": "missing_reported_vapor_composition"}])
        self.assertTrue(all(r["pressure_pa"] >= 200000 for r in (c["observed"] for c in results["cases"])))

    def test_score_uses_accepted_limits_and_retains_failures(self):
        results = load_json(ROOT / "experimental-results.json")
        self.assertEqual(assess(results["cases"]), results["assessment"])
        thresholds = load_json(BASELINE / "protocol.json")["thresholds"]
        for check in results["assessment"]["checks"]:
            if "limit" in check:
                key = "pressure_relative" if "pressure" in check["quantity"] and "density" not in check["quantity"] else "density_relative" if "density" in check["quantity"] else "beta_absolute" if "beta" in check["quantity"] else "composition_absolute"
                self.assertEqual(check["limit"], thresholds[key])
        self.assertEqual(Counter(c["status"] for c in results["assessment"]["checks"]), {"pass": 103, "fail": 25})

    def test_phase_controls_do_not_change_density(self):
        data = load_json(ROOT / "phase-8.0.0.json")
        self.assertEqual(len(data["cases"]), 102)
        anomalies = [c for c in data["cases"] if c["baseline_audit"]["consistent"] is False]
        self.assertEqual(len(anomalies), 27)
        self.assertEqual(sum(not c.get("excluded_reasons") for c in anomalies), 25)
        for c in data["cases"]:
            self.assertEqual(c["operations"]["fresh"], c["operations"]["repeat"])
            self.assertEqual(c["operations"]["fresh"], c["operations"]["default_parameters"])
        for c in anomalies:
            rho = c["operations"]["fresh"]["density_mol_m3"]
            roots = c["independent_homogeneous_PR"]
            self.assertLess(abs(rho/roots["minimum_homogeneous_gibbs_density_mol_m3"]-1), 1e-6)
            if "fluid" in c:
                roundtrip = c["operations"]["DmolarT_roundtrip"]
                self.assertEqual(roundtrip["density_mol_m3"], rho)
                self.assertEqual(roundtrip["phase"], data["phase_enum"]["iphase_liquid"])
            else:
                self.assertEqual(c["operations"]["fresh"]["reducing_density_mol_m3"], 1.)

    def test_independent_roots_reproduce_pressure(self):
        parameters = load_json(BASELINE / "parameters.json")
        r = parameters["gas_constant_j_mol_k"]
        for version in ("8.0.0", "7.2.0"):
            for c in load_json(ROOT / f"phase-{version}.json")["cases"]:
                t, p = c["temperature_k"], c["pressure_pa"]
                x = [c["z_propane"], 1-c["z_propane"]] if "z_propane" in c else [float(v["coolprop_name"] == c["fluid"]) for v in parameters["components"]]
                algebra = c["independent_homogeneous_PR"]
                ai, bi = [], []
                for comp in parameters["components"]:
                    tc, pc, w = (comp[k] for k in ("critical_temperature_k", "critical_pressure_pa", "acentric_factor"))
                    ai.append(algebra["omega_a"]*(r*tc)**2/pc*(1+(.37464+1.54226*w-.26992*w*w)*(1-math.sqrt(t/tc)))**2)
                    bi.append(algebra["omega_b"]*r*tc/pc)
                a = sum(x[i]*x[j]*math.sqrt(ai[i]*ai[j]) for i in range(2) for j in range(2))
                b = sum(x[i]*bi[i] for i in range(2))
                for root in algebra["roots"]:
                    v = 1/root["density_mol_m3"]
                    reconstructed = r*t/(v-b)-a/(v*v+2*b*v-b*b)
                    self.assertLess(abs(reconstructed/p-1), 1e-9)

    def test_old_version_is_not_promoted_to_oracle(self):
        data = load_json(ROOT / "phase-7.2.0.json")
        self.assertEqual(Counter(c["operations"]["fresh"]["status"] for c in data["cases"]), {"ok": 89, "reference_error": 13})
        self.assertTrue(all(c["operations"]["DmolarT_roundtrip"]["status"] == "reference_error" for c in data["cases"] if "fluid" in c))

    def test_supplement_manifest(self):
        manifest = load_json(ROOT / "manifest.json")
        for name, digest in manifest["files"].items():
            self.assertEqual(sha256(ROOT / name), digest, name)


if __name__ == "__main__":
    unittest.main()
