"""Distil frozen TRAIN losses without changing any model or selecting a coefficient."""
import collections
import json
from pathlib import Path
import statistics

ROOT = Path(__file__).resolve().parents[3]
SRC = ROOT / 'docs/wsp/feedback/astra-frozen-train-20261114/results'
OUT = ROOT / 'docs/wsp/feedback/astra-r2-20261115'


def write_same(path, value):
    text = json.dumps(value, indent=2) + '\n'
    if path.exists():
        assert path.read_text(encoding='utf-8') == text, path
    else:
        path.write_text(text, encoding='utf-8')


def main():
    OUT.mkdir(exist_ok=True)
    for family in ['flex_methanol', 'flex_heptane', 'acme_normal', 'acme_inverse']:
        d = json.loads((SRC / (family + '.json')).read_text())
        def index(r):
            return {(c['run_id'], c['cutoff_s']): c for c in r['cases'] if c['scored']}
        b, a, j = map(index, d['releases'])
        metrics = ['crps', 'mae_median'] if family.startswith('flex') else ['ibs_contrib[extinction]', 'ibs_contrib[operator_termination]']
        rows = []
        for k in sorted(b.keys() & a.keys()):
            c = a[k]
            row = {'run': k[0], 'cutoff_s': k[1], 'cluster': c['cluster'], 'conditions': c['conditions'], 'endpoint': c['endpoint'], 'encoder': c['encoder'], 'bias_baseline': b[k]['bias'], 'bias_astra': c['bias'], 'metrics': {}}
            for m in metrics:
                factor = 1e6 if family.startswith('flex') else 1
                row['metrics'][m] = {'baseline': b[k]['metrics'][m]*factor, 'r1': a[k]['metrics'][m]*factor, 'joint': j[k]['metrics'][m]*factor, 'delta': (a[k]['metrics'][m]-b[k]['metrics'][m])*factor}
            rows.append(row)
        metric = metrics[0]
        changed = [r for r in rows if abs(r['metrics'][metric]['delta']) > 1e-10]
        groups = collections.defaultdict(list)
        for r in rows:
            groups[r['cluster']].append(r)
        bycluster = [{'cluster': g, 'cases': len(rr), 'delta': statistics.mean(r['metrics'][metric]['delta'] for r in rr), 'baseline': statistics.mean(r['metrics'][metric]['baseline'] for r in rr), 'r1': statistics.mean(r['metrics'][metric]['r1'] for r in rr)} for g, rr in groups.items()]
        bycut = []
        for cut in [1, 3, 5]:
            rr = [r for r in rows if r['cutoff_s'] == cut]
            bycut.append({'cutoff': cut, 'cases': len(rr), 'delta': statistics.mean(r['metrics'][metric]['delta'] for r in rr)})
        value = {'metric': metric, 'scored': len(rows), 'changed': len(changed), 'by_cutoff_case_mean': bycut, 'by_cluster': sorted(bycluster, key=lambda v: v['delta'], reverse=True), 'largest_losses': sorted(changed, key=lambda r: r['metrics'][metric]['delta'], reverse=True)[:12], 'largest_gains': sorted(changed, key=lambda r: r['metrics'][metric]['delta'])[:6]}
        write_same(OUT / (family + '-diagnostic.json'), value)
    d = json.loads((SRC / 'bass_front-cases.json').read_text())
    ids = d['release_ids']
    runs = collections.defaultdict(dict)
    for c in d['rows']:
        if 'cutoff_s' not in c or 'refusal' in c:
            continue
        runs[(c['run_id'], c['cutoff_s'])][ids.index(c['release_id'])] = c
    rows = [{'run': run, 'cutoff': cut, 'baseline': cs[0], 'r1': cs[1], 'joint': cs[2], 'delta_crps': cs[1]['crps']-cs[0]['crps'], 'joint_delta_crps': cs[2]['crps']-cs[0]['crps']} for (run, cut), cs in runs.items()]
    write_same(OUT / 'bass_front-diagnostic.json', {'rows': rows, 'scored_sessions': 1})
    print('Verified five reproducible TRAIN diagnostic files')


if __name__ == '__main__':
    main()
