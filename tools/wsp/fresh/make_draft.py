import json, csv, os, sys, collections
sys.path.insert(0, 'tools/wsp/ext')
import flame_design as fd

S = sys.argv[1]
names = fd.listing()
cond = {r['test']: r for r in csv.DictReader(open('runs/dev/data_all/work/transfer_acme/fd_fuel/flame_design/conditions.csv', encoding='utf-8'))}
dom = {d['test']: d for f in ('fd_dom.json', 'fd_dom_inv.json') for d in json.load(open(f'{S}/{f}'))}
part = {}
s = json.load(open('docs/wsp/atlas/splits/PSI-10.json'))
for fam, st in (('acme_normal', 'normal_flames'), ('acme_inverse', 'inverse_flames')):
    for p, g in s['strata'][st]['groups'].items():
        for x in g:
            part[(fam, x.split(':')[-1])] = p
TIERS = {
    'A1': ('scripted short constant stage then ramp to zero; test flows equal to an included constant-flow test (sibling) of the same family',
           '19115B2 19115G1 19123L4 19123L5 19123L6 19123L7 19137G1 19137H1 19137J1 19137K2 19137L2 19137L3 19137L4 19137L5 19137L6 19137L8 19156J5 19189K2 19189K3 19189P1'),
    'A2': ('constant flow, table conditions complete and inside the F2 domain; excluded only because the _TP.txt stage file is missing',
           '19150H1 19206L6 21301C2'),
    'A3': ('inverse, scripted multi-stage flows (v2); stage-1 flows equal to an included sibling',
           '22024B2 22027B4 22027B8'),
    'B1': ('scripted ignition-to-test ramp (flows still changing at the cutoffs), or no sibling: conditions need a GCF conversion, and a ramp or dilution law that F2 does not carry',
           '19115D2 19142G4 19142G5 19156B2 19156J1 19171B1 19171B3 19171J1 19171J3 19171J4 19175A2 19182G1 19182G6 19182G7 19182G8 19182A1 19189A2 19189A3 19189A4 19189D2 19196F1 19156J6 19156J7 19137K1 21349M4 21349M5 21349M6 22024B3 22024M2'),
}
RAW = 'runs/dev/data_all/raw/PSI-10/Raw Data/'
tests = []
for tier, (why, ids) in TIERS.items():
    for t in ids.split():
        r = cond[t]
        fam = r['family']
        fn = fd.file_names(t, names)
        files = {k: dict(name=v, bytes=names[v], on_disk=os.path.exists(RAW + v)) for k, v in fn.items() if v}
        d = dom.get(t, {})
        table_full = bool(r['injected_mass_flow_kg_s'] and r['tad_k_table'] and r['x_o2_role'])
        if table_full:
            src = 'conditions.csv (experiment table)'
        elif d.get('n_sib'):
            src = 'table pressure (and O2 when given) + test flows of sibling(s) ' + ','.join(d.get('siblings', []))
        else:
            src = 'table pressure only; flows from the script, conversion not done'
        tests.append(dict(
            test_id=t, family=fam, group=f"PSI-10:day:{t[:5]}", partition_of_day=part[(fam, t[:5])], tier=tier,
            table_note=r['table_note'] or None,
            conditions=dict(source=src,
                            pressure_pa=float(r['pressure_pa']) if r['pressure_pa'] else None,
                            mass_flow_kg_s=d.get('mass_flow_kg_s'), x_fuel_role=d.get('x_fuel'), x_o2_role=d.get('x_o2'),
                            tad_k=d.get('tad_sibling_k'), outside_f2_domain=d.get('outside')),
            script_planned_first_flow_change_s=d.get('planned_first_flow_change_s'), script_flags=d.get('flags'),
            files=files, bytes=sum(f['bytes'] for f in files.values()), held=True))

not_in_batch = dict(
    printed_by_mistake=dict(tests=['19123L3', '19206N1', '19142G1'],
                            why='the after-test table columns (XO2 final, pfinal) of these rows were printed on 15 Nov while this draft was prepared; dropped'),
    live_ramp=dict(tests='19142G2 19142G3 19142F1 19142J2 19142J3 19171S1'.split(),
                   why='table says RAMP but the pre-ignition script shows a plain long constant stage: the ramp was commanded live and shows only in mfcSetpoints, which also reveal the end'),
    low_o2=dict(tests=['22024K2', '22024K3'], why='table O2 is "LOW O2"; no per-test O2 in a pre-ignition file'),
    outside_domain=dict(tests='21328N1 21328N2 21328N3 21328N4 21328N5 21349A1 21349B1 21349D2 21349D3'.split(),
                        why='constant flow but F2 refuses (c_zst / c_xf); baseline-only'),
    not_published=dict(tests='19137C1 19206A1 19206A5 19206G1 19206G2 19206G3 19206G4 19206G5 19206G7 19206G8 19206G9 19206H1 19206H2 19206H3 19206H4 19206K1 19206K2 21301J1 21301J2 22042M1'.split(),
                       why='no data/setpoint/TP/script text file in the NASA listing retrieved 15 Nov 2026 (22042M1 is also on a day in no partition)'),
    no_table_row={'PSI-10': '19115C1 19115D1 19115H1 19115P1 19123H1 19123L8 19124H1 19142B1 19175B2 19196N2 22024M1'.split(),
                  'PSI-159': '21106A1 21106A2 21106A3 21106G1 21106G2 21106G3 21106G4 21106G5 21106G8 21176M1 21242F1 21337A7'.split(),
                  'why': 'raw files published but no row in the experiment table: no O2/Tad, and whether a hot flame was established is unknown without reading channels'},
    cfig_outside_cohort=dict(tests='73 table rows of PSI-159 (50 normal, 23 inverse): 70 hot_ignition no/flash, 3 hot ignitions with an unresolved endpoint cause (21088C1 train day; two 50 % butane tests on test days)',
                             why='no hot flame to forecast, or the table outcome is already ingested in data/work/records'),
    sflame_test_days=dict(tests='PSI-23 days 20142 (17 test names), 20170 (20), 20176 (24); day 19245 has logs and GC files only',
                          why='already the frozen, unread test partition of docs/wsp/atlas/splits/PSI-23.json; all helium-diluted (F2 has no diluent input), 20142 is a H2/CH4 blend (no blend law), 20170/20176 mix ~21 % and ~30 % O2 with no per-test O2 source'),
)
doc = dict(
    schema='ignis.atlas.batch/1', status='DRAFT, not frozen', investigation_id='PSI-10', drafted_on='2026-11-15', batch='acme-fresh',
    rule=('Flame Design (PSI-10) tests that no release has scored and whose endpoint was never measured. Every day of CFI-G and Flame '
          'Design is already in a spent partition, so these are the tests the transfer rule excluded on those days, chosen from the '
          'table conditions, the script parameter file and the listing only (no channel, setpoint log, TP stamp, comment or after-test '
          'column read). Tier A can be scored by acme_normal/acme_inverse.wsp12 F2 once its conditions are written to a cases file; '
          'tier B needs a conversion or a law that F2 does not carry. Group = test day; partition_of_day is the PSI-10 split of that day, '
          'which this batch does not change. Held: no outcome is read until the batch and its release list are frozen.'),
    split='docs/wsp/atlas/splits/PSI-10.json', endpoint_tool='tools/wsp/ext/flame_design.py (thresholds unchanged)',
    caveats=[
        'On a ramp-to-zero test the flame_design.py label rule calls the end operator_termination whenever the scripted ramp starts before '
        'the flame ends, so tier A1 mostly tests the planned stop (first flow change 5-29 s after onset), not extinction physics.',
        'The day-level PDFs (test sheets, flow traces, daily test point logs) and the _Comments.txt files of these days are published and '
        'may describe outcomes; nothing records whether anyone read them for these tests.',
        'Tier A2 needs flame_design.py to run without _TP.txt (the flow-on fallback used for s-Flame); the transfer rule requires it today.',
        'Sibling conditions copy mass flow, fuel fraction and Tad of an included test with identical script setpoints; Tad is not corrected '
        'for the O2 difference between the two tests.'],
    bytes=sum(t['bytes'] for t in tests),
    bytes_to_download=sum(f['bytes'] for t in tests for f in t['files'].values() if not f['on_disk']),
    counts={k: sum(1 for t in tests if t['tier'] == k) for k in TIERS},
    groups=sorted({t['group'] for t in tests}),
    tiers={k: v[0] for k, v in TIERS.items()}, tests=tests, not_in_batch=not_in_batch)
open('docs/wsp/atlas/splits/ACME-batch-fresh.draft.json', 'w', encoding='utf-8').write(json.dumps(doc, indent=1) + '\n')
print(doc['counts'], doc['bytes'], doc['bytes_to_download'], len(doc['groups']))
print(collections.Counter((t['tier'], t['partition_of_day']) for t in tests))
print(collections.Counter(t['group'] for t in tests if t['tier'].startswith('A')))
print([(t['test_id'], t['conditions']['outside_f2_domain']) for t in tests if t['tier'].startswith('A') and t['conditions']['outside_f2_domain']])
