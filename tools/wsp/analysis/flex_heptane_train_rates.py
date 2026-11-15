"""Sheet facts and windowed finite-difference rates of the n-heptane TRAIN runs.

usage: flex_heptane_train_rates.py main
       flex_heptane_train_rates.py series <t0> <t1> <step> <run-suffix> [...]

`main`: per train run the atmosphere, D0, the sheet's burn time tb and extinction diameter, the
clock at the end of the hot flame s_ext = tb / D0^2, and d(D^2)/dt (mm2/s) early in the burn, in the
two seconds before tb and in four windows after it (the cool flame, where there is one).
`series`: window means of D and the rate between consecutive windows.
Train partition only (lib.partition_ids); it prints tables and fits nothing. Used for
docs/wsp/RESULTS_HEPTANE_D3.md.
"""
import json
import math
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import lib  # noqa: E402

ids = lib.partition_ids('flex_heptane','train')
recs = lib.experiments('flex_heptane', ids)
post = lib.jsonl_for('flex_heptane','post.jsonl', ids)
RAW = lib.ROOT / 'data/raw/samples/PSI-69'
names = sorted(os.listdir(RAW))
def load(rid):
    keys = {a['name'] for a in recs[rid]['aliases']}
    f = [n for n in names if n.endswith('_DropletData.txt') and any(('_'+k+'_') in n for k in keys)]
    if not f: return []
    rows=[]
    for line in (RAW/f[0]).read_text(encoding='utf-8',errors='replace').splitlines()[1:]:
        p=line.split('\t')
        try: rows.append((float(p[0]), float(p[1])))
        except ValueError: pass
    return rows
def wmean(rows, a, b):
    v=[(t,d*d) for t,d in rows if a<=t<=b and d>0]
    if len(v)<3: return None
    return (sum(t for t,_ in v)/len(v), sum(q for _,q in v)/len(v))
def rate(rows, a, b, h=0.4):
    A=wmean(rows,a-h,a+h); B=wmean(rows,b-h,b+h)
    if not A or not B or B[0]<=A[0]: return None
    return (A[1]-B[1])/(B[0]-A[0])
def fmt(x, n=2):
    return '   ?' if x is None else f'{x:5.{n}f}'
out=[]
for rid in ids:
    r=recs[rid]; c=r['conditions']
    g=lambda k:(c.get(k) or {}).get('value_si')
    facts=(post.get(rid) or [{}])[0].get('facts',{})
    f=lambda k:(facts.get(k) or {}).get('raw_value')
    rows=load(rid)
    out.append(dict(rid=rid.split(':')[1], o2=g('x_o2'), co2=g('x_co2'), he=g('x_he'), p=(g('pressure') or 0)/101325, d0=(g('initial_diameter') or 0)*1e3, bt=f('burn_time'), dext=f('dext'), k=f('burning_rate'), rows=rows, sess=r['session_id']))
out.sort(key=lambda o:(round(o['he'] or 0,2), round(o['co2'] or 0,2), -(o['o2'] or 0), o['d0']))
mode=sys.argv[1] if len(sys.argv)>1 else 'main'
if mode=='main':
    print('run       O2   CO2  He   P    D0   | tb    Dext  s_ext | tlast | K: 0.5-1.5 1.5-3  3-5   | pre(-3..-1) post(+1..+3) (+3..+6) (+6..+10) (+10..+15)')
    for o in out:
        rows=o['rows']; bt=o['bt']; tl=rows[-1][0] if rows else None
        try: bt=float(bt)
        except (TypeError, ValueError): bt=None
        s=(bt/(o['d0']**2)) if bt and o['d0'] else None
        ks=[rate(rows,0.5,1.5,0.25), rate(rows,1.5,3.0), rate(rows,3.0,5.0)]
        if bt:
            ps=[rate(rows,bt-3,bt-1), rate(rows,bt+1,bt+3), rate(rows,bt+3,bt+6), rate(rows,bt+6,bt+10), rate(rows,bt+10,bt+15)]
        else: ps=[None]*5
        print(f"{o['rid']:9s} {o['o2']:.2f} {o['co2']:.2f} {o['he']:.2f} {o['p']:.2f} {o['d0']:.2f} | {fmt(bt,1)} {str(o['dext']):>5s} {fmt(s,3)} | {fmt(tl,1)} | {fmt(ks[0])} {fmt(ks[1])} {fmt(ks[2])} | {fmt(ps[0])} {fmt(ps[1])} {fmt(ps[2])} {fmt(ps[3])} {fmt(ps[4])}  {o['sess'][-10:]}")
elif mode=='series':
    t0,t1,step=map(float,sys.argv[2:5])
    for suf in sys.argv[5:]:
        o=[x for x in out if suf in x['rid']][0]
        print('==',o['rid'],'D0',round(o['d0'],2),'tb',o['bt'],'Dext',o['dext'])
        t=t0; line=[]
        while t<t1:
            m=wmean(o['rows'],t,t+step)
            m2=wmean(o['rows'],t+step,t+2*step)
            if m:
                k=(m[1]-m2[1])/(m2[0]-m[0]) if m2 else None
                line.append(f"{m[0]:5.1f} D={math.sqrt(m[1]):.3f} K={fmt(k)}")
            t+=step
        for i in range(0,len(line),4): print('   '+' | '.join(line[i:i+4]))
