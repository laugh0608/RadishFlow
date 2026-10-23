"""Audit selected original-table values without modifying accepted C2-A evidence."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
sys.path.insert(0, str(ROOT.parent / 'c2a-pr76'))
import reference as ref


def read(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_accepted():
    for pack, expected in read(ROOT / 'protocol.json')['protected_manifests'].items():
        directory = ROOT.parent / pack
        manifest = directory / 'manifest.json'
        if digest(manifest) != expected:
            raise ValueError(f'Accepted manifest changed: {pack}')
        data = read(manifest)
        entries = data.get('repository_relative_sha256', data.get('files'))
        base = REPO if 'repository_relative_sha256' in data else directory
        for name, sha in entries.items():
            if digest(base / name) != sha:
                raise ValueError(f'Accepted evidence changed: {pack}/{name}')


def match_point(point):
    datasets = read(ROOT.parent / point['pack'] / 'experimental-data.json')['datasets']
    rows = {r['id']: (r, d['uncertainty']) for d in datasets for r in d['rows']}
    pressure, pressure_meta = rows[point['pressure_id']]
    vapor, vapor_meta = rows[point['composition_id']]
    for key in ('temperature_k', 'x_propane'):
        if point[key] != pressure[key] or point[key] != vapor[key]:
            raise ValueError(f'Original table join mismatch: {point["pressure_id"]}/{key}')
    for key, row in [('pressure_pa', pressure), ('y_propane', vapor)]:
        if point[key] != row[key]:
            raise ValueError(f'Original table value mismatch: {point["pressure_id"]}/{key}')
    return {'pressure_expanded_pa': pressure['expanded_uncertainty'],
            'y_expanded': vapor['expanded_uncertainty'],
            'pressure_metadata': pressure_meta, 'y_metadata': vapor_meta}


def interval_gap(low, high, observed, uncertainty):
    return max(0., low - (observed + uncertainty), observed - uncertainty - high)


def generate():
    verify_accepted()
    ref.validate_environment()
    observations = read(ROOT / 'source-observations.json')
    sources = {s['id']: s for s in observations['sources']}
    results = []
    for point in observations['points']:
        compiler = match_point(point)
        uncertainty = sources[point['source']]['author_uncertainty']
        dp = uncertainty.get('pressure_pa', uncertainty.get('pressure_additive_pa', 0.)
                             + point['pressure_pa'] * uncertainty.get('pressure_relative', 0.))
        backends = {}
        for backend in ('PR', 'HEOS'):
            samples = []
            for t_sign in (-1, 0, 1):
                for x_sign in (-1, 0, 1):
                    t = point['temperature_k'] + t_sign * uncertainty['temperature_k']
                    x = point['x_propane'] + x_sign * uncertainty['x']
                    samples.append({'temperature_k': t, 'x_propane': x,
                                    'result': ref.attempt(ref.query_saturation, backend, t, 0., x)})
            ok = [s['result'] for s in samples if s['result']['status'] == 'ok']
            nominal = samples[4]['result']
            summary = {'sample_count': len(samples), 'error_count': len(samples)-len(ok)}
            if len(ok) == len(samples):
                p_range = [min(s['pressure_pa'] for s in ok), max(s['pressure_pa'] for s in ok)]
                y_range = [min(s['y'][0] for s in ok), max(s['y'][0] for s in ok)]
                p_error = abs(nominal['pressure_pa'] / point['pressure_pa'] - 1.)
                y_error = abs(nominal['y'][0] - point['y_propane'])
                summary.update({'sampled_pressure_pa': p_range, 'sampled_y': y_range,
                    'pressure_interval_gap_pa': interval_gap(*p_range, point['pressure_pa'], dp),
                    'y_interval_gap': interval_gap(*y_range, point['y_propane'], uncertainty['y']),
                    'nominal_pressure_relative_error': p_error, 'nominal_y_absolute_error': y_error,
                    'original_pressure_status': 'pass' if p_error <= ref.PROTOCOL['thresholds']['pressure_relative'] else 'fail',
                    'original_y_status': 'pass' if y_error <= ref.PROTOCOL['thresholds']['composition_absolute'] else 'fail'})
            backends[backend] = {'summary': summary, 'samples': samples}
        results.append({'point': point, 'table_values_match': True,
                        'compiler_uncertainty': compiler, 'author_pressure_uncertainty_pa': dp,
                        'backends': backends})
    return {'schema': 'radishflow.c2a-source-audit', 'results': results,
            'new_mixture_gas_density_measurements': 0,
            'interpretation': 'Sampled sensitivity only; no change to accepted counts, limits, model parameters or evidence qualification.'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(generate(), ensure_ascii=False, indent=2, allow_nan=False)+'\n')
