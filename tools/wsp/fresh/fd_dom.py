import csv,json,sys,math,collections
sys.path.insert(0,'tools/wsp/ext')
import acme_script2 as a2, flame_design as fd
cond={r['test']:r for r in csv.DictReader(open('runs/dev/data_all/work/transfer_acme/fd_fuel/flame_design/conditions.csv',encoding='utf-8'))}
def sp(t):
    p=a2.script_path('PSI-10',t)
    return a2.parse(p) if p else None
# setpoint -> table values from included constant tests (conditions columns only)
sib=collections.defaultdict(list)
for t,r in cond.items():
    if r['include_transfer']!='1': continue
    s=sp(t)
    if not s: continue
    key=(r['family'],s.get('fuelBottleTestFlowRateN2Slpm') or s.get('oxidizerTestFlow1Slpm'),s.get('nitrogenTestFlowRateN2Slpm') or s.get('nitrogenTestFlow1Slpm'),s.get('constantFlowTime2Sec','') and None)
    sib[key].append(r)
tests=sys.argv[1].split(',')
out=[]
for t in tests:
    r=cond[t]; s=sp(t); fam=r['family']; sc=fd.SCALES[fam]
    key=(fam,s.get('fuelBottleTestFlowRateN2Slpm') or s.get('oxidizerTestFlow1Slpm'),s.get('nitrogenTestFlowRateN2Slpm') or s.get('nitrogenTestFlow1Slpm'),None)
    sibs=sib.get(key,[])
    pl=a2.planned(s)
    row=dict(test=t,family=fam,setpoint=key[1:],siblings=[x['test'] for x in sibs][:4],n_sib=len(sibs),planned_first_flow_change_s=pl.get('planned_first_flow_change_s'),planned_stop_s=pl.get('planned_stop_s'),flags=pl.get('flags'))
    if r['injected_mass_flow_kg_s'] and r['x_fuel_role'] and r['tad_k_table']:
        m=float(r['injected_mass_flow_kg_s']); xf=float(r['x_fuel_role']); tad=float(r['tad_k_table'])
    elif sibs:
        m=sum(float(x['injected_mass_flow_kg_s']) for x in sibs)/len(sibs); xf=sum(float(x['x_fuel_role']) for x in sibs)/len(sibs); tad=sum(float(x['tad_k_table']) for x in sibs)/len(sibs)
    else:
        out.append(row); continue
    p=float(r['pressure_pa']) if r['pressure_pa'] else float('nan'); xo2=float(r['x_o2_role']) if r['x_o2_role'] else (sum(float(x['x_o2_role']) for x in sibs)/len(sibs) if sibs else float('nan'))
    if fam=='acme_normal':
        zst=fd.zst_initial('ethylene',xf,xo2) if xo2==xo2 else float('nan')
    else:
        zst=fd.zst_initial('ethylene',xf,xo2) if xo2==xo2 else float('nan')
    f=dict(c_logp=math.log(p/fd.P_REF_PA) if p==p else float('nan'),c_logm=math.log(m/sc['mdot_ref']),c_o2=(xo2-sc['o2_ref'])/sc['o2_scale'],c_xf=math.log(xf/sc['xf_ref']),c_tad=(tad-fd.TAD_REF_K)/fd.TAD_SCALE_K,c_zst=(zst-sc['zst_ref'])/fd.ZST_SCALE)
    row.update(mass_flow_kg_s=m,x_fuel=xf,x_o2=xo2,pressure_pa=p,tad_sibling_k=tad,outside=[k for k,(lo,hi) in sc['domain'].items() if not (lo<=f[k]<=hi)])
    out.append(row)
json.dump(out,open(sys.argv[2],'w'),indent=1)
for o in out: print(o['test'],o['n_sib'],o['setpoint'],o.get('outside','NO SIBLING'),o.get('planned_first_flow_change_s'),o.get('flags'))
