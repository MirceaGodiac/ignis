"""Assemble GPT-6 Astra's already handwritten R2 deltas; no generated scientific values."""
import argparse
import copy
import json
from pathlib import Path
import subprocess
from binary_snapshot import capture_binary

ROOT = Path(__file__).resolve().parents[3]
CASES = [
    ('flex_methanol6','flex/wsp6/methanol','droplet'),
    ('flex_heptane6','flex/wsp6/heptane','droplet'),
    ('acme_normal6','acme/wsp6/normal','acme'),
    ('acme_inverse6','acme/wsp6/inverse','acme'),
]


def read(p): return json.loads(p.read_text(encoding='utf-8'))


def write(p,v):
    with p.open('x',encoding='utf-8',newline='\n') as f:
        f.write(json.dumps(v,indent=2)+'\n')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--execute',action='store_true'); args=parser.parse_args()
    assert args.execute,'requires --execute'
    source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    snap=capture_binary(ROOT/'runs/tooling/astra-train-comparison-target/debug/ignis.exe',build_identity=source)
    for key,relative,family in CASES:
        base=ROOT/'configs/models'/relative; parent=read(base/'r1-astra/release.json');dest=base/'r2-astra'
        assert (dest/'changes.wsp').exists()
        assert not any((dest/n).exists() for n in ['meta.json','patch.json','release.json','assembly.json']),dest
        meta=copy.deepcopy(read(base/'r1-astra/meta.json'))
        meta['patch_id']=key+'-r2-astra';meta['parent_release_id']=parent['id']
        meta['author']={'provider':'openai','model_id':'gpt-6-astra','effort':'high'}
        meta['exposure']['prior_candidates'].append(parent['id'])
        meta['exposure']['development_feedback'].append('repo:docs/wsp/feedback/astra-frozen-train-20261114/results/summary.json')
        meta['exposure']['sources'].extend([
            'repo:docs/wsp/research/astra_r2_'+family+'_loss_analysis.md',
            'repo:docs/wsp/feedback/astra-r2-20261115/'+key.removesuffix('6')+'-diagnostic.json',
            'exposure:historical-aggregate-validation-prose-incidentally-seen-in-mandatory-instructions-not-used-for-R2;no-per-run-validation-or-test-outcomes',
            'author:gpt-6-astra-high-explicit-physical-author-subagent;literal-delta-written-once-before-R2-scoring',
        ])
        meta['rationale']='DATA-INFORMED R2, not zero-shot. Exact handwritten delta to R1; no fitting or search. '+('Correct vapor heat-capacity species in the Stefan enthalpy term using 300 K NIST values; no empirical gain promised.' if family=='droplet' else 'Withdraw unsupported scenario probability mixture; nominal-condition rollback expected to reproduce adopted Fable, not a novel mechanism.')+' See changes.wsp and the R2 family report for each literal and exposure.'
        write(dest/'meta.json',meta)
        calls=[]
        for args in [
            ['wsp','compile','--family',key,'--edits',str(dest/'changes.wsp'),'--meta',str(dest/'meta.json'),'--out',str(dest/'patch.json')],
            ['wsp','check','--family',key,'--patch',str(dest/'patch.json'),'--root',str(ROOT/'artifacts')],
            ['wsp','assemble','--family',key,'--patch',str(dest/'patch.json'),'--binding',str(base/'binding.json'),'--root',str(ROOT/'artifacts'),'--out',str(dest/'release.json')],
        ]:
            r=snap.run(args,cwd=ROOT,capture_output=True,text=True)
            assert r.returncode==0,(key,args[1],r.stderr)
            calls.append({'arguments':args,'exit_code':r.returncode,'stdout':r.stdout,'stderr':r.stderr})
        release=read(dest/'release.json')
        assert release['trained_on']=='not_trained' and release['state']=='experimental'
        assert release['construction']['forward_hash']==parent['construction']['forward_hash']
        write(dest/'assembly.json',{'release_id':release['id'],'parent_release_id':parent['id'],'candidate_status':'data-informed','binary':snap.receipt(),'calls':calls})
        print(key,release['id'],flush=True)
    snap.verify()

if __name__=='__main__': main()
