#!/usr/bin/env python3
"""Re-assemble every literal (WSP) release from its committed edits, in lineage order.

A release is its weights and its forward, so a release id changes whenever the forward identity
changes. This script rebuilds the whole lineage under the current forward:

  W0:  edits.wsp + meta.json -> patch.json -> release (registry)        (edits and meta untouched)
  W1+: the parent id in meta.json (and in the edits header comment) is pointed at the freshly built
       parent, then compiled and assembled the same way.

It never touches a number: only the parent ids of revisions (provenance). Weight hashes are printed
so that "same weights as before" can be checked. Run from the repository root:

    python tools/wsp/reassemble.py [--ignis target/release/ignis] [--only acme_inverse3,acme_inverse5]
"""
import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

CHAINS = [
    ("flex", "configs/models/flex/wsp", "binding.json", ["w0-fable", "w1-fable", "w2-fable"]),
    ("acme_normal", "configs/models/acme/wsp", "normal-binding.json",
     ["normal-w0-fable", "normal-w1-fable", "normal-w2-fable"]),
    ("acme_inverse", "configs/models/acme/wsp", "inverse-binding.json",
     ["inverse-w0-fable", "inverse-w1-fable"]),
    ("bass", "configs/models/bass/wsp", "binding.json", ["w0-fable"]),
    # multistate droplet models (phase 2). A data-informed first candidate of a new spec is a
    # literal revision of that spec's D^2-law baseline; the heptane lineage starts at its W0.
    ("flex_methanol", "configs/models/flex/wsp/methanol", "binding.json", ["baseline-d2", "d1-fable", "d2-fable"]),
    ("flex_heptane", "configs/models/flex/wsp/heptane", "binding.json", ["w0-fable", "d1-fable"]),
    ("flex_heptane", "configs/models/flex/wsp/heptane", "binding.json", ["baseline-d2"]),
    # clock models of the burner flames (phase 2)
    ("acme_normal2", "configs/models/acme/wsp2/normal", "binding.json", ["baseline-clock", "d1-fable", "d2-fable", "d3-fable"]),
    ("acme_inverse2", "configs/models/acme/wsp2/inverse", "binding.json", ["baseline-clock", "d1-fable", "d2-fable"]),
    # the inverse D3 revises D1 (the carried candidate), not D2
    ("acme_inverse2", "configs/models/acme/wsp2/inverse", "binding.json", ["baseline-clock", "d1-fable", "d3-fable"]),
    # third-generation droplet forward (droplet3.rs); skipped with a note while absent
    ("flex_methanol2", "configs/models/flex/wsp2/methanol", "binding.json", ["baseline-d2", "d3-fable"]),
    ("flex_heptane2", "configs/models/flex/wsp2/heptane", "binding.json", ["baseline-d2", "d2-fable"]),
    # fourth generation (14 November). Droplets on droplet4.rs: the candidate, and a neutral carry of
    # the third-generation numbers that stands in for them in paired comparisons (the fourth-generation
    # prefix also carries the sheet's initial diameter, so its cases do not pair with older releases).
    ("flex_methanol3", "configs/models/flex/wsp3/methanol", "binding.json", ["baseline-d2", "d4-fable"]),
    ("flex_methanol3", "configs/models/flex/wsp3/methanol", "binding.json", ["baseline-d2", "carry-d3"]),
    ("flex_heptane3", "configs/models/flex/wsp3/heptane", "binding.json", ["baseline-d2", "d3-fable"]),
    ("flex_heptane3", "configs/models/flex/wsp3/heptane", "binding.json", ["baseline-d2", "carry-d2"]),
    # the broad component alone on the third-generation numbers (written after the first candidates failed on validation)
    ("flex_methanol3", "configs/models/flex/wsp3/methanol", "binding.json", ["baseline-d2", "d5-flags"]),
    ("flex_heptane3", "configs/models/flex/wsp3/heptane", "binding.json", ["baseline-d2", "d4-flags"]),
    # burner flames on flame3.rs: published theory alone, then the data-informed candidate
    ("acme_normal3", "configs/models/acme/wsp3/normal", "binding.json", ["baseline-theory", "d1-fable"]),
    ("acme_inverse3", "configs/models/acme/wsp3/inverse", "binding.json", ["baseline-theory", "d1-fable", "d2-fable"]),
    # the two hand-written burner-flame laws answering together (blend.rs). Member values are the
    # stored values of two existing releases, copied by tools/wsp/blend_edits.py; only mix.w is typed.
    ("acme_normal4", "configs/models/acme/wsp4/normal", "binding.json", ["baseline-pair", "d1-fable"]),
    ("acme_inverse4", "configs/models/acme/wsp4/inverse", "binding.json", ["baseline-pair", "d1-fable"]),
    # droplets on droplet5.rs, which also reads the flame cameras: the third-generation numbers with the
    # new entries switched off (carry), with a clock that can be wrong (R1), and with the cameras on as well (F1)
    ("flex_methanol4", "configs/models/flex/wsp4/methanol", "binding.json", ["baseline-d2", "carry-d3"]),
    ("flex_methanol4", "configs/models/flex/wsp4/methanol", "binding.json", ["baseline-d2", "r1-fable"]),
    ("flex_methanol4", "configs/models/flex/wsp4/methanol", "binding.json", ["baseline-d2", "f1-fable"]),
    ("flex_heptane4", "configs/models/flex/wsp4/heptane", "binding.json", ["baseline-d2", "carry-d2"]),
    ("flex_heptane4", "configs/models/flex/wsp4/heptane", "binding.json", ["baseline-d2", "r1-fable"]),
    ("flex_heptane4", "configs/models/flex/wsp4/heptane", "binding.json", ["baseline-d2", "f1-fable"]),
    # committees of scenarios (committee.rs): up to six member releases with typed weights. Slot values
    # are copied from existing releases by tools/wsp/committee_edits.py; only mix.w is typed.
    ("acme_normal5", "configs/models/acme/wsp5/normal", "binding.json", ["baseline-set"]),
    ("acme_inverse5", "configs/models/acme/wsp5/inverse", "binding.json", ["baseline-set", "d1-fable"]),
    # flame front of the BASS fabric burns on front.rs
    ("bass_front", "configs/models/bass/front", "binding.json", ["baseline-persistence", "d1-fable", "d2-fable"]),
    # fifth generation, physics-first (14 November). Burner flames: revisions of the member laws in
    # physical variables, and copies of members with their guessed stop atoms switched off (for
    # prefixes that carry the script's planned stop); committees of them.
    ("acme_normal2", "configs/models/acme/wsp2/normal", "binding.json",
     ["baseline-clock", "d1-fable", "d2-fable", "d3-fable", "d3-nostops"]),
    ("acme_normal3", "configs/models/acme/wsp3/normal", "binding.json", ["baseline-theory", "d1-fable", "d2-fable", "d2-nostops"]),
    ("acme_inverse3", "configs/models/acme/wsp3/inverse", "binding.json", ["baseline-theory", "d1-fable", "d3-fable"]),
    ("acme_inverse3", "configs/models/acme/wsp3/inverse", "binding.json", ["baseline-theory", "d1-fable", "d1-nostops"]),
    ("acme_inverse3", "configs/models/acme/wsp3/inverse", "binding.json", ["baseline-theory", "d1-fable", "d5-fable", "d5-nostops"]),
    ("acme_normal5", "configs/models/acme/wsp5/normal", "binding.json", ["baseline-set", "p1-fable"]),
    ("acme_normal5", "configs/models/acme/wsp5/normal", "binding.json", ["baseline-set", "carry-d1"]),
    ("acme_inverse5", "configs/models/acme/wsp5/inverse", "binding.json", ["baseline-set", "p1-fable"]),
    ("acme_inverse5", "configs/models/acme/wsp5/inverse", "binding.json", ["baseline-set", "carry-d1"]),
    # droplets on droplet6.rs (self-cooling evaporation, cool-flame end in oxygen partial pressure, a
    # dying flame): the carried numbers with the new entries switched off, then the candidate
    ("flex_methanol5", "configs/models/flex/wsp5/methanol", "binding.json", ["baseline-d2", "carry-d3"]),
    ("flex_methanol5", "configs/models/flex/wsp5/methanol", "binding.json", ["baseline-d2", "carry-d3", "p0-fable", "p1-fable"]),
    ("flex_heptane5", "configs/models/flex/wsp5/heptane", "binding.json", ["baseline-d2", "carry-d2"]),
    ("flex_heptane5", "configs/models/flex/wsp5/heptane", "binding.json", ["baseline-d2", "carry-d2", "p0-fable", "p1-fable"]),
    # flame front on front2.rs (two oxygen powers, slow growth and fast shrinking of the flame length,
    # flow paths): the carried numbers with the new entries switched off, then the candidate
    ("bass_front2", "configs/models/bass/front2", "binding.json", ["baseline-persistence", "carry-d1"]),
    ("bass_front2", "configs/models/bass/front2", "binding.json", ["baseline-persistence", "carry-d1", "p1-fable"]),
    # the hinge-law clocks with the burner-heating law as their burner-limit clock (flame4.rs). The root
    # holds the hinge-law tensors of the wsp2 root and the burner tensors of the wsp3 root, copied by
    # tools/wsp/flame4_edits.py.
    # sixth generation of the normal burner flames (15 November): the member laws with three statements
    # corrected and one growth member per era of the burner limit, each with a copy without guessed stops
    ("acme_normal2", "configs/models/acme/wsp2/normal", "binding.json",
     ["baseline-clock", "d1-fable", "d2-fable", "d3-fable", "d4-fable", "d4-nostops"]),
    ("acme_normal3", "configs/models/acme/wsp3/normal", "binding.json",
     ["baseline-theory", "d1-fable", "d2-fable", "d3-early", "d3-early-nostops"]),
    ("acme_normal3", "configs/models/acme/wsp3/normal", "binding.json",
     ["baseline-theory", "d1-fable", "d2-fable", "d3-early", "d3-late", "d3-late-nostops"]),
    ("acme_normal8", "configs/models/acme/wsp8/normal", "binding.json", ["baseline-clock-theory"]),
    ("acme_inverse8", "configs/models/acme/wsp8/inverse", "binding.json", ["baseline-clock-theory"]),
    ("acme_normal8", "configs/models/acme/wsp8/normal", "binding.json", ["baseline-clock-theory", "d1-early"]),
    ("acme_normal8", "configs/models/acme/wsp8/normal", "binding.json", ["baseline-clock-theory", "d1-late"]),
    ("acme_normal8", "configs/models/acme/wsp8/normal", "binding.json", ["baseline-clock-theory", "d1-early-nostops"]),
    ("acme_normal8", "configs/models/acme/wsp8/normal", "binding.json", ["baseline-clock-theory", "d1-late-nostops"]),
    # the second committee (committee2.rs): ten member slots of the three burner-flame forwards, weights
    # by the era of the test, three readings of the planned stop. Slot values are copied from stored
    # releases by tools/wsp/committee2_edits.py; the roots hold the slots of the wsp5 roots and, in the
    # four flame4.rs slots, the wsp8 roots.
    ("acme_normal7", "configs/models/acme/wsp7/normal", "binding.json", ["baseline-set"]),
    ("acme_normal7", "configs/models/acme/wsp7/normal", "binding.json", ["baseline-set", "p2-fable"]),
    ("acme_normal7", "configs/models/acme/wsp7/normal", "binding.json", ["baseline-set", "p3-fable"]),
    ("acme_normal7", "configs/models/acme/wsp7/normal", "binding.json", ["baseline-set", "carry-p1"]),
    ("acme_inverse7", "configs/models/acme/wsp7/inverse", "binding.json", ["baseline-set"]),
    # the burner-flame laws with a typed fuel table in place of the two fuel indicators (flame5.rs,
    # flame6.rs, flame7.rs; design docs/wsp/research/phys_spherical_fuels.md, section 3). The roots hold
    # the law tensors of the wsp2, wsp3 and wsp8 roots, copied by tools/wsp/fuel_member_edits.py, and the
    # neutral fuel table; the committee root (committee3.rs) holds them in its slots, copied by
    # tools/wsp/committee3_edits.py, with the committee lines of the wsp7 root.
    ("acme_normal9", "configs/models/acme/wsp9/normal", "binding.json", ["baseline-clock"]),
    ("acme_inverse9", "configs/models/acme/wsp9/inverse", "binding.json", ["baseline-clock"]),
    ("acme_normal10", "configs/models/acme/wsp10/normal", "binding.json", ["baseline-theory"]),
    ("acme_inverse10", "configs/models/acme/wsp10/inverse", "binding.json", ["baseline-theory"]),
    ("acme_normal11", "configs/models/acme/wsp11/normal", "binding.json", ["baseline-clock-theory"]),
    ("acme_inverse11", "configs/models/acme/wsp11/inverse", "binding.json", ["baseline-clock-theory"]),
    ("acme_normal12", "configs/models/acme/wsp12/normal", "binding.json", ["baseline-set"]),
    ("acme_inverse12", "configs/models/acme/wsp12/inverse", "binding.json", ["baseline-set"]),
    # fuel-table candidates F1 (15 November): the members of acme_normal.wsp7 P3 and of acme_inverse.wsp5 P1
    # with the typed fuel table of configs/models/acme/fuel_tables/, and the committees of them
    ("acme_normal9", "configs/models/acme/wsp9/normal", "binding.json", ["baseline-clock", "f1-d4"]),
    ("acme_normal9", "configs/models/acme/wsp9/normal", "binding.json", ["baseline-clock", "f1-d4-nostops"]),
    ("acme_normal10", "configs/models/acme/wsp10/normal", "binding.json", ["baseline-theory", "f1-d3-early"]),
    ("acme_normal10", "configs/models/acme/wsp10/normal", "binding.json", ["baseline-theory", "f1-d3-late"]),
    ("acme_normal10", "configs/models/acme/wsp10/normal", "binding.json", ["baseline-theory", "f1-d3-early-nostops"]),
    ("acme_normal10", "configs/models/acme/wsp10/normal", "binding.json", ["baseline-theory", "f1-d3-late-nostops"]),
    ("acme_normal11", "configs/models/acme/wsp11/normal", "binding.json", ["baseline-clock-theory", "f1-d1-early"]),
    ("acme_normal11", "configs/models/acme/wsp11/normal", "binding.json", ["baseline-clock-theory", "f1-d1-late"]),
    ("acme_normal11", "configs/models/acme/wsp11/normal", "binding.json", ["baseline-clock-theory", "f1-d1-early-nostops"]),
    ("acme_normal11", "configs/models/acme/wsp11/normal", "binding.json", ["baseline-clock-theory", "f1-d1-late-nostops"]),
    ("acme_normal12", "configs/models/acme/wsp12/normal", "binding.json", ["baseline-set", "f1-fable"]),
    ("acme_inverse10", "configs/models/acme/wsp10/inverse", "binding.json", ["baseline-theory", "f1-d1-fable"]),
    ("acme_inverse10", "configs/models/acme/wsp10/inverse", "binding.json", ["baseline-theory", "f1-d5-fable"]),
    ("acme_inverse10", "configs/models/acme/wsp10/inverse", "binding.json", ["baseline-theory", "f1-d1-nostops"]),
    ("acme_inverse10", "configs/models/acme/wsp10/inverse", "binding.json", ["baseline-theory", "f1-d5-nostops"]),
    ("acme_inverse12", "configs/models/acme/wsp12/inverse", "binding.json", ["baseline-set", "f1-fable"]),
    # F2 (15 November): F1 revised once from the train days of Flame Design; the ethylene row of the fuel
    # table in the members, and the committee lines for the planned stop computed from the script file
    ("acme_normal9", "configs/models/acme/wsp9/normal", "binding.json", ["baseline-clock", "f2-d4"]),
    ("acme_normal9", "configs/models/acme/wsp9/normal", "binding.json", ["baseline-clock", "f2-d4-nostops"]),
    ("acme_normal10", "configs/models/acme/wsp10/normal", "binding.json", ["baseline-theory", "f2-d3-early"]),
    ("acme_normal10", "configs/models/acme/wsp10/normal", "binding.json", ["baseline-theory", "f2-d3-late"]),
    ("acme_normal10", "configs/models/acme/wsp10/normal", "binding.json", ["baseline-theory", "f2-d3-early-nostops"]),
    ("acme_normal10", "configs/models/acme/wsp10/normal", "binding.json", ["baseline-theory", "f2-d3-late-nostops"]),
    ("acme_normal11", "configs/models/acme/wsp11/normal", "binding.json", ["baseline-clock-theory", "f2-d1-early"]),
    ("acme_normal11", "configs/models/acme/wsp11/normal", "binding.json", ["baseline-clock-theory", "f2-d1-late"]),
    ("acme_normal11", "configs/models/acme/wsp11/normal", "binding.json", ["baseline-clock-theory", "f2-d1-early-nostops"]),
    ("acme_normal11", "configs/models/acme/wsp11/normal", "binding.json", ["baseline-clock-theory", "f2-d1-late-nostops"]),
    ("acme_normal12", "configs/models/acme/wsp12/normal", "binding.json", ["baseline-set", "f2-fable"]),
    ("acme_inverse12", "configs/models/acme/wsp12/inverse", "binding.json", ["baseline-set", "f2-fable"]),
    # bass_front.wsp5 (front5.rs): the tip law of front.rs with the boundary law of ignition and quench.
    # Root: persistence with the boundary law neutral; then the carry of D1. Their tip statements are
    # copied from stored front.rs releases by tools/wsp/front5_edits.py.
    ("bass_front5", "configs/models/bass/front5", "binding.json", ["baseline-persistence", "carry-d1"]),
    # Q0 (15 November): the tip law of D1 with the zero-shot boundary law of docs/wsp/research/phys_quench_w0.md
    ("bass_front5", "configs/models/bass/front5", "binding.json", ["baseline-persistence", "q0-fable"]),
    # the sixth droplet law for eight fuels (droplet8.rs, flex_<fuel>.wsp7). Methanol and n-heptane:
    # the fifth-generation chains copied without change (carries). The other six: the anchor's
    # baseline-d2 copied without change as the lineage root, then the one-shot candidate of the fuel.
    ("flex_methanol7", "configs/models/flex/wsp7/methanol", "binding.json", ["baseline-d2", "carry-d3"]),
    ("flex_methanol7", "configs/models/flex/wsp7/methanol", "binding.json", ["baseline-d2", "carry-d3", "carry-p0", "carry-p1"]),
    ("flex_heptane7", "configs/models/flex/wsp7/heptane", "binding.json", ["baseline-d2", "carry-d2"]),
    ("flex_heptane7", "configs/models/flex/wsp7/heptane", "binding.json", ["baseline-d2", "carry-d2", "carry-p0", "carry-p1"]),
    ("flex_octane7", "configs/models/flex/wsp7/octane", "binding.json", ["baseline-d2", "w0-fable"]),
    ("flex_isooctane7", "configs/models/flex/wsp7/isooctane", "binding.json", ["baseline-d2", "w0-fable"]),
    ("flex_decane7", "configs/models/flex/wsp7/decane", "binding.json", ["baseline-d2", "w0-fable"]),
    ("flex_dodecane7", "configs/models/flex/wsp7/dodecane", "binding.json", ["baseline-d2", "w0-fable"]),
    ("flex_farnesane7", "configs/models/flex/wsp7/farnesane", "binding.json", ["baseline-d2", "w0-fable"]),
    ("flex_ethanol7", "configs/models/flex/wsp7/ethanol", "binding.json", ["baseline-d2", "w0-fable"]),
    # droplet8 with the mechanisms of sections 3.2 and 3.3 (droplet9.rs, flex_<fuel>.wsp8): the lineage
    # roots only, the droplet8 roots with every new entry switched off
    ("flex_methanol8", "configs/models/flex/wsp8/methanol", "binding.json", ["baseline-d2"]),
    ("flex_heptane8", "configs/models/flex/wsp8/heptane", "binding.json", ["baseline-d2"]),
    ("flex_octane8", "configs/models/flex/wsp8/octane", "binding.json", ["baseline-d2"]),
    ("flex_isooctane8", "configs/models/flex/wsp8/isooctane", "binding.json", ["baseline-d2"]),
    ("flex_decane8", "configs/models/flex/wsp8/decane", "binding.json", ["baseline-d2"]),
    ("flex_dodecane8", "configs/models/flex/wsp8/dodecane", "binding.json", ["baseline-d2"]),
    ("flex_farnesane8", "configs/models/flex/wsp8/farnesane", "binding.json", ["baseline-d2"]),
    ("flex_ethanol8", "configs/models/flex/wsp8/ethanol", "binding.json", ["baseline-d2"]),
    ("flex_heptane_isooctane8", "configs/models/flex/wsp8/heptane_isooctane", "binding.json", ["baseline-d2"]),
    ("flex_dodecane_isododecane8", "configs/models/flex/wsp8/dodecane_isododecane", "binding.json", ["baseline-d2"]),
    ("flex_propanol_glycerol8", "configs/models/flex/wsp8/propanol_glycerol", "binding.json", ["baseline-d2"]),
    # the generic law forward (law.rs, <slug>.wsp): the synthetic demo of docs/wsp/research/law_forward.md,
    # its neutral root generated by `ignis wsp law-root` and the invented example (never NASA data)
    ("synthetic_demo", "configs/models/law/synthetic_demo", "binding.json", ["root", "example"]),
    # seventh generation (15 November): the zero-shot W0 laws of docs/wsp/research/phys_<slug>_w0.md, each a revision
    # of its slug's neutral root; spice has two candidates (variant B registered, the main law beside)
    ("bre", "configs/models/law/bre", "binding.json", ["root", "w0-fable"]),
    ("spice", "configs/models/law/spice", "binding.json", ["root", "w0-fable"]),
    ("spice", "configs/models/law/spice", "binding.json", ["root", "lit-fable"]),
    ("efield", "configs/models/law/efield", "binding.json", ["root", "w0-fable"]),
    ("samer", "configs/models/law/samer", "binding.json", ["root", "w0-fable"]),
    ("same", "configs/models/law/same", "binding.json", ["root", "w0-fable"]),
    ("daft", "configs/models/law/daft", "binding.json", ["root", "w0-fable"]),
    ("bass1", "configs/models/law/bass1", "binding.json", ["root", "w0-fable"]),
    ("slice", "configs/models/law/slice", "binding.json", ["root", "w0-fable"]),
    ("cld", "configs/models/law/cld", "binding.json", ["root", "w0-fable"]),
    # D1 revisions of PROTOCOL 13.3 (data-informed, train days): revisions of the W0 releases
    ("bre", "configs/models/law/bre", "binding.json", ["root", "w0-fable", "d1-fable"]),
    ("cld", "configs/models/law/cld", "binding.json", ["root", "w0-fable", "d1-fable"]),
    ("slice", "configs/models/law/slice", "binding.json", ["root", "w0-fable", "d1-fable"]),
    ("samer", "configs/models/law/samer", "binding.json", ["root", "w0-fable", "d1-fable"]),
    ("bass1", "configs/models/law/bass1", "binding.json", ["root", "w0-fable", "d1-fable"]),
    # second laws of PROTOCOL 13.4 (new slugs, revisions of their own neutral roots)
    ("cld2", "configs/models/law/cld2", "binding.json", ["root", "s1-fable"]),
    ("bre", "configs/models/law/bre", "binding.json", ["root", "w0-fable", "d2-fable"]),
    ("bre", "configs/models/law/bre", "binding.json", ["root", "w0-fable", "d3-fable"]),
    ("bre", "configs/models/law/bre", "binding.json", ["root", "w0-fable", "d3m-fable"]),
    ("efield2", "configs/models/law/efield2", "binding.json", ["root", "s1-fable"]),
    ("slice2", "configs/models/law/slice2", "binding.json", ["root", "s1-fable"]),
    ("bass2", "configs/models/law/bass2", "binding.json", ["root", "s1-fable"]),
    ("bass2", "configs/models/law/bass2", "binding.json", ["root", "s1-fable", "k1-fable"]),
    ("bass3", "configs/models/law/bass3", "binding.json", ["root", "s1-fable"]),
    # third laws of PROTOCOL 13.6 (new slugs): efield3 (phys_efield_study.md section 4), cld3 (phys_cld_study.md section 5)
    ("efield3", "configs/models/law/efield3", "binding.json", ["root", "s1-fable"]),
    ("cld3", "configs/models/law/cld3", "binding.json", ["root", "s1-fable"]),
    ("bass4", "configs/models/law/bass4", "binding.json", ["root", "s1-fable"]),
    # PROTOCOL 13.8: the post-final plastics revision (typed after the final run with every partition seen)
    ("bass5", "configs/models/law/bass5", "binding.json", ["root", "s1-fable"]),
    # candidates of the jet-flame studies (PROTOCOL 13.6): revisions of the current releases, numbers only
    ("cld", "configs/models/law/cld", "binding.json", ["root", "w0-fable", "d1-fable", "c1-fable"]),
    ("efield", "configs/models/law/efield", "binding.json", ["root", "w0-fable", "e1-fable"]),
    ("spice", "configs/models/law/spice", "binding.json", ["root", "w0-fable", "p2-fable"]),
    ("slice2", "configs/models/law/slice2", "binding.json", ["root", "s1-fable", "s2-fable"]),
    ("spice", "configs/models/law/spice", "binding.json", ["root", "w0-fable", "p2-fable", "p3-fable"]),
    # plastics comparators of docs/wsp/research/phys_bass_study2.md section E (S1 in os1-fable: s1-fable is bass3 itself)
    ("bass3", "configs/models/law/bass3", "binding.json", ["root", "s1-fable", "os1-fable"]),
    ("bass3", "configs/models/law/bass3", "binding.json", ["root", "s1-fable", "q1-fable"]),
    # SAME-R S1 (docs/wsp/research/phys_samer_study.md): a revision of samer W0, not of D1
    ("samer", "configs/models/law/samer", "binding.json", ["root", "w0-fable", "s1-fable"]),
    # the fourth committee (committee4.rs; F3 of docs/wsp/research/phys_sflame_f3.md): its root, the neutral carry
    # of F2 and F3 stage 1, slot values copied by tools/wsp/committee4_edits.py
    ("acme_normal13", "configs/models/acme/wsp13/normal", "binding.json", ["baseline-set", "carry-f2"]),
    ("acme_normal13", "configs/models/acme/wsp13/normal", "binding.json", ["baseline-set", "f3-s1"]),
    ("acme_normal13", "configs/models/acme/wsp13/normal", "binding.json", ["baseline-set", "f3-s1", "f3-s2"]),
    # F4 (docs/wsp/research/phys_sflame_f4.md): three comparators, each a revision of the one before
    ("acme_normal13", "configs/models/acme/wsp13/normal", "binding.json",
     ["baseline-set", "f3-s1", "f4-methane", "f4-helium", "f4-hydrogen"]),
]


def short(rid: str) -> str:
    return f"sha256:{rid[7:15]}…{rid[-4:]}"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ignis", default="target/release/ignis")
    ap.add_argument("--only", default="", help="comma-separated model-key prefixes: rebuild only these chains")
    args = ap.parse_args()
    only = [x for x in args.only.split(",") if x]
    exe = Path(args.ignis)
    if not exe.exists() and exe.with_suffix(".exe").exists():
        exe = exe.with_suffix(".exe")
    exe = str(exe.resolve())

    def ignis(*a: str) -> dict:
        out = subprocess.run([exe, "wsp", *a], capture_output=True, text=True)
        if out.returncode != 0:
            sys.exit(f"ignis wsp {' '.join(a)} failed:\n{out.stderr}{out.stdout}")
        return json.loads(out.stdout)

    mapping = {}
    for family, root, binding, chain in CHAINS:
        if only and not any(family.startswith(x) for x in only):
            continue
        if not Path(root).is_dir():
            print(f"{family:<13} skipped: {root} does not exist yet")
            continue
        parent_old, parent_new = None, None
        for step in chain:
            d = Path(root) / step
            if not d.is_dir():
                # a later step of a chain may not be written yet; nothing after it can be rebuilt
                print(f"{family:<13} {step:<17} skipped: {d} does not exist yet")
                break
            meta_path, edits_path, release_path = d / "meta.json", d / "edits.wsp", d / "release.json"
            old = json.loads(release_path.read_text(encoding="utf-8")) if release_path.exists() else None
            meta = json.loads(meta_path.read_text(encoding="utf-8"))
            if "parent_release_id" in meta:
                if parent_new is None:
                    sys.exit(f"{d}: a revision without a rebuilt parent")
                text = meta_path.read_text(encoding="utf-8")
                for o, n in mapping.items():
                    text = text.replace(o, n)
                meta = json.loads(text)
                meta["parent_release_id"] = parent_new
                meta_path.write_text(json.dumps(meta, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
                # the header comment of the edits names the parent; keep it truthful (no number changes)
                edits = edits_path.read_text(encoding="utf-8")
                new_edits = re.sub(r"sha256:[0-9a-f]{8}…[0-9a-f]{4}", short(parent_new), edits, count=1)
                if new_edits != edits:
                    edits_path.write_text(new_edits, encoding="utf-8", newline="\n")
            ignis("compile", "--family", family, "--edits", str(edits_path), "--meta", str(meta_path),
                  "--out", str(d / "patch.json"))
            built = ignis("assemble", "--family", family, "--patch", str(d / "patch.json"),
                          "--binding", str(Path(root) / binding), "--out", str(release_path))
            new_id = built["release_id"]
            same = "" if old is None else (
                "  weights unchanged" if old["construction"]["weights_hash"] == built["weights_hash"]
                else "  WEIGHTS DIFFER")
            print(f"{family:<13} {step:<17} {new_id}  nonzero {built['counts']['authored_nonzero']:>3}"
                  f"/{built['counts']['authored_total']}{same}")
            if old is not None and old["id"] != new_id:
                mapping[old["id"]] = new_id
            parent_old, parent_new = (old["id"] if old else None), new_id
    if mapping:
        print("\nformer id -> current id")
        for o, n in mapping.items():
            print(f"  {o} -> {n}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
