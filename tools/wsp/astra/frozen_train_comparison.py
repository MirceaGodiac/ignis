"""Execute only the committed Astra TRAIN protocol; never writes model parameters."""
import argparse
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import random
import statistics
import subprocess

from binary_snapshot import capture_binary

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / 'docs/wsp/feedback/astra-frozen-train-20261114'
PROTOCOL = ROOT / 'docs/wsp/dossier/ASTRA_FROZEN_TRAIN_20261114.md'


def read(p):
    return json.loads(Path(p).read_text(encoding='utf-8'))


def write(p, value):
    with Path(p).open('x', encoding='utf-8', newline='\n') as f:
        json.dump(value, f, indent=2, allow_nan=False)
        f.write('\n')


def verify(frozen):
    for name, expected in frozen['file_sha256'].items():
        assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == expected, name
    for name, expected in frozen['mixed_record_file_stats'].items():
        s = (ROOT / name).stat()
        assert {'size': s.st_size, 'mtime_ns': s.st_mtime_ns} == expected, name


def paired(a, b, metric, scale=1.0):
    keys = sorted(k for k in a.keys() & b.keys()
                  if isinstance(a[k]['metrics'].get(metric), (int, float))
                  and isinstance(b[k]['metrics'].get(metric), (int, float))
                  and math.isfinite(a[k]['metrics'][metric]) and math.isfinite(b[k]['metrics'][metric]))
    groups = defaultdict(list)
    for k in keys:
        assert a[k]['cluster'] == b[k]['cluster'] and a[k]['cluster'], k
        groups[a[k]['cluster']].append(k)
    av = [statistics.mean(a[k]['metrics'][metric] * scale for k in ks) for ks in groups.values()]
    bv = [statistics.mean(b[k]['metrics'][metric] * scale for k in ks) for ks in groups.values()]
    ds = [x-y for x,y in zip(av,bv)]
    ci = None
    if len(ds) > 1:
        rng = random.Random(20261114)
        boot = sorted(statistics.mean(rng.choices(ds, k=len(ds))) for _ in range(10000))
        ci = [boot[249], boot[9749]]
    return {'common_cases': len(keys), 'clusters': len(groups),
            'candidate': statistics.mean(av) if av else None,
            'baseline': statistics.mean(bv) if bv else None,
            'difference': statistics.mean(ds) if ds else None, 'ci95': ci,
            'cluster_differences': dict(zip(groups, ds))}


def generic_rows(r):
    out = {}
    for c in r['cases']:
        if not c['scored']: continue
        key = (c['run_id'], c['cutoff_s'])
        assert key not in out, key
        out[key] = {'metrics': c['metrics'], 'cluster': c['cluster']}
    return out


def front_rows(detail, rid, days):
    out = {}
    for c in detail['rows']:
        if c['release_id'] != rid or 'cutoff_s' not in c or 'refusal' in c: continue
        metrics = {k: c[k] for k in ['crps','ae_median','coverage_50','coverage_90']}
        ev = c.get('events', [])
        if ev: metrics['ended_brier'] = statistics.mean((e['p_ended']-e['observed'])**2 for e in ev)
        key = (c['run_id'], c['cutoff_s'])
        assert key not in out
        out[key] = {'metrics': metrics, 'cluster': days[c['run_id']]}
    return out


def summarize(frozen, output):
    comparisons=[]
    for plan in frozen['comparisons']:
        key=plan['family']; report=read(output/(key+'.json'))
        if key=='bass_front':
            assert set(report['counts']['partitions'])=={'train'}
            days={}
            for p in (ROOT/'tools/wsp/bass_optical/tracks/train').glob('T*.json'):
                t=read(p); assert t['partition']=='train'
                days[t['run_id']]=t['conditions']['gmt_day']
            detail=read(output/(key+'-cases.json'))
            rows=[front_rows(detail,rid,days) for rid in plan['release_ids']]
            refusals=[sum(c['release_id']==rid and 'refusal' in c for c in detail['rows']) for rid in plan['release_ids']]
            metrics=['crps','ae_median','coverage_50','coverage_90','ended_brier']; primary=['crps']; guards=['ended_brier']
        else:
            assert report['partition']=='train'
            assert [r['release_id'] for r in report['releases']]==plan['release_ids']
            rows=[generic_rows(r) for r in report['releases']]
            refusals=[len(r['refused']) for r in report['releases']]
            if key.startswith('acme'):
                primary=['ibs_contrib[extinction]','ibs_contrib[operator_termination]']; metrics=primary; guards=[]
            else:
                primary=['crps'];metrics=['crps','mae_median','coverage@0.5','coverage@0.9'];guards=[]
        for i,label in [(1,'r1-astra'),(2,'joint-astra')]:
            result={m:paired(rows[i],rows[0],m,1e6 if key.startswith('flex') and m in ['crps','mae_median'] else 1.0) for m in metrics}
            lost=len(rows[0].keys()-rows[i].keys()); gained=len(rows[i].keys()-rows[0].keys())
            means=[result[m]['difference'] for m in primary]
            if not all(x is not None for x in means): verdict='unscoreable'
            elif lost or refusals[i]>refusals[0]: verdict='coverage loss; no overall win'
            elif any(x>1e-12 for x in means): verdict='worse primary mean' if all(x>=-1e-12 for x in means) else 'primary trade-off'
            elif all(abs(x)<=1e-12 for x in means): verdict='level primary means'
            elif any(result[m]['difference'] is not None and result[m]['difference']>1e-12 for m in guards): verdict='lower primary mean; secondary trade-off'
            elif all(result[m]['ci95'] is not None and result[m]['ci95'][1]<=1e-12 for m in primary): verdict='lower TRAIN error; paired interval supports improvement'
            else: verdict='lower TRAIN mean; improvement uncertain'
            comparisons.append({'family':key,'candidate':label,'release_id':plan['release_ids'][i],
                                'baseline_release_id':plan['release_ids'][0], 'verdict':verdict,
                                'baseline_scored_cases':len(rows[0]),'candidate_scored_cases':len(rows[i]),
                                'lost_baseline_cases':lost,'gained_cases':gained,
                                'baseline_refusals':refusals[0],'candidate_refusals':refusals[i],
                                'metrics':result})
    summary={'candidate_status':'data-informed; not zero-shot','partition':'train','comparison_unit':'equal-weight independent session/split clusters',
             'limits':['Descriptive 95% intervals; no multiplicity correction or generalisation claim.',
                       'IPCW contributions use frozen observed censoring weights, not re-estimated inside the diagnostic bootstrap.',
                       'BASS track-ended target includes leaving view; it is not pure physical extinction.'], 'comparisons':comparisons}
    write(output/'summary.json',summary)
    return summary


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--execute',action='store_true');args=p.parse_args()
    assert args.execute, 'requires --execute'
    frozen=read(BASE/'freeze.json')
    rels=[str(p.relative_to(ROOT)).replace('\\','/') for p in [BASE/'freeze.json',PROTOCOL,Path(__file__).resolve()]]
    for path in rels:
        subprocess.run(['git','ls-files','--error-unmatch',path],cwd=ROOT,check=True,capture_output=True)
    assert not subprocess.check_output(['git','status','--porcelain','--',*rels],cwd=ROOT,text=True).strip(), 'commit protocol before scoring'
    commit=subprocess.check_output(['git','log','-1','--format=%H','--',*rels],cwd=ROOT,text=True).strip()
    verify(frozen)
    output=ROOT/frozen['output_dir']; output.mkdir(exist_ok=False)
    snap=capture_binary(args.binary,build_identity=frozen['source_commit'])
    env=os.environ.copy();env['IGNIS_DATA']=str(ROOT/'data');env['IGNIS_ARTIFACTS']=str(ROOT/'artifacts')
    def run(plan):
        key=plan['family']; command=['front-feedback' if key=='bass_front' else 'feedback']
        for rid in plan['release_ids']: command+=['--release',rid]
        if key=='bass_front':
            command+=['--tracks',str(ROOT/'tools/wsp/bass_optical/tracks/train'),'--label','frozen-astra-train-20261114','--detail',str(output/(key+'-cases.json'))]
        else:
            command+=['--partition','train','--cutoffs','1,3,5','--horizon-s','10','--n-boot','400']
            if key.startswith('acme'): command+=['--times','6,8,10,12,15,20,25,30,40,50,60,80,100,120']
        command+=['--out',str(output/(key+'.json'))]
        start=datetime.now(timezone.utc).isoformat()
        r=snap.run(command,cwd=ROOT,env=env,capture_output=True,text=True)
        (output/(key+'.log')).write_text(r.stdout+r.stderr,encoding='utf-8')
        write(output/(key+'-execution.json'),{'protocol_commit':commit,'command':command,'binary':snap.receipt(),'started_at':start,'finished_at':datetime.now(timezone.utc).isoformat(),'exit_code':r.returncode})
        assert r.returncode==0,(key,r.stderr)
        print(key+' scored',flush=True)
    with ThreadPoolExecutor(max_workers=3) as pool: list(pool.map(run,frozen['comparisons']))
    verify(frozen); snap.verify()
    summary=summarize(frozen,output)
    write(output/'receipt.json',{'protocol_commit':commit,'finished_at':datetime.now(timezone.utc).isoformat(),'binary':snap.receipt(),'files_sha256':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(output.iterdir()) if p.is_file()}})
    for c in summary['comparisons']: print(c['family'],c['candidate'],c['verdict'])

if __name__=='__main__': main()

