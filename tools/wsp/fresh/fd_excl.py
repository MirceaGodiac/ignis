import csv,json,sys,collections
sys.path.insert(0,'tools/wsp/ext')
import acme_script2 as a2
R='runs/dev/data_all/raw/PSI-10/Raw Data/'
r=list(csv.DictReader(open('runs/dev/data_all/work/transfer_acme/fd_fuel/flame_design/conditions.csv',encoding='utf-8')))
s=json.load(open('docs/wsp/atlas/splits/PSI-10.json'))
part={}
for fam,st in (('acme_normal','normal_flames'),('acme_inverse','inverse_flames')):
  for p,g in s['strata'][st]['groups'].items():
    for x in g: part[(fam,x.split(':')[-1])]=p
KEYS=['fuelBottleIgnitionFlowRateN2Slpm','nitrogenIgnitionFlowRateN2Slpm','fuelBottleTestFlowRateN2Slpm','nitrogenTestFlowRateN2Slpm','ignitionToTestRampTimeSec','constantFlowTimeSec','constantFlowTime1Sec','testFlowRamp1To2TimeSec','constantFlowTime2Sec','constantFlowTime3Sec','useRampFlowToZero','rampFlowToZeroSec',
 'oxygenTestFlowRateN2Slpm','oxygenIgnitionFlowRateN2Slpm','fuelBottleTestFlowRate1N2Slpm','fuelBottleTestFlowRate2N2Slpm','oxidizerBottleTestFlowRate1N2Slpm']
out=[]
for x in r:
    if x['include_transfer']=='1': continue
    t=x['test']; p=part.get((x['family'],x['session']),'held(no group)')
    try:
        sp=a2.script_path('PSI-10',t)
        kv=a2.parse(sp) if sp and sp.exists() else None
    except Exception as e:
        kv=None
    row=dict(test=t,family=x['family'],part=p,note=x['table_note'],fuel=x['fuel_species'],p=x['pressure_pa'],xo2=x['x_o2_role'],files=''.join(k[4] for k in ('has_data','has_setpoints','has_tp','has_script') if x[k]=='1'),data_bytes=x['data_bytes'])
    if kv:
        row['params']={k:kv[k] for k in kv if any(s in k for s in ('FlowRate','constantFlowTime','RampTime','RampFlowToZero','rampFlowToZero','testFlowRamp'))}
    out.append(row)
json.dump(out,open(sys.argv[1],'w'),indent=0)
c=collections.Counter((o['family'],o['part'],o['note'] or ('missing files' if o['files']!='dstc' else 'outside:'+next(x['outside'] for x in r if x['test']==o['test'])),o['files'],'script' if 'params' in o else 'noscript') for o in out)
for k,v in sorted(c.items()): print(v,k)
