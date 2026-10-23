"""Protect source identity, uncertainty provenance and accepted evidence."""
import copy
import unittest
from audit import ROOT, read, digest, verify_accepted, match_point, interval_gap


class SourceAuditTests(unittest.TestCase):
    def test_both_accepted_packs_and_new_manifest(self):
        verify_accepted()
        for name, expected in read(ROOT / 'manifest.json')['files'].items():
            self.assertEqual(digest(ROOT / name), expected, name)

    def test_original_value_or_composition_change_is_rejected(self):
        original = read(ROOT / 'source-observations.json')['points'][0]
        match_point(original)
        for key in ('temperature_k', 'pressure_pa', 'x_propane', 'y_propane'):
            changed = copy.deepcopy(original)
            changed[key] += .00001
            with self.assertRaises(ValueError):
                match_point(changed)

    def test_interval_overlap_and_separation(self):
        self.assertEqual(interval_gap(3., 5., 6., 2.), 0.)
        self.assertEqual(interval_gap(3., 5., 7., 1.), 1.)
        self.assertEqual(interval_gap(3., 5., 1., .5), 1.5)

    def test_author_and_compiler_uncertainty_remain_distinct(self):
        results = read(ROOT / 'results.json')['results']
        self.assertEqual(results[0]['compiler_uncertainty']['y_expanded'], .007)
        self.assertEqual(results[0]['author_pressure_uncertainty_pa'], 1000)
        self.assertEqual(results[1]['compiler_uncertainty']['y_expanded'], .0264)
        self.assertAlmostEqual(results[1]['author_pressure_uncertainty_pa'], 1167.197)
        sources = read(ROOT / 'source-observations.json')['sources']
        self.assertIsNone(sources[0]['author_uncertainty']['coverage_factor'])
        self.assertEqual(sources[1]['author_uncertainty']['coverage_factor'], 2)

    def test_complete_samples_and_no_fabricated_density(self):
        data = read(ROOT / 'results.json')
        self.assertEqual(data['new_mixture_gas_density_measurements'], 0)
        self.assertEqual(len(data['results']), 3)
        for row in data['results']:
            for backend in row['backends'].values():
                self.assertEqual(len(backend['samples']), 9)
                self.assertTrue(all(s['result']['status']=='ok' for s in backend['samples']))
                self.assertEqual(backend['summary']['original_y_status'], 'fail')


if __name__ == '__main__':
    unittest.main()
