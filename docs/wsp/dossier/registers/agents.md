# Register: delegated agents of the workstream

Made by `tools/wsp/dossier_registers.py --transcripts …` from the transcripts of the author's sessions
(local files, not in git). "Asked" is what the launch requested; "answered" is the model id recorded on the
agent's own messages. A report is matched to a saved file by its text. An agent with no saved report either
did not finish, returned a short answer that was used in the session only, or was a measurement or
review helper whose result went into another file; the dossier names the ones that matter.

| Started (UTC) | Launched by | Task | Asked: model / effort | Answered by | Finished | Product |
|---|---|---|---|---|---|---|
| 2026-11-13 18:07 | author | MIT WSP artifact study | opus / medium | claude-opus-5-5 | yes | docs/wsp/research/mit_wsp.md (reformatted) |
| 2026-11-13 18:07 | author | WSP replications and related literature | opus / medium | claude-opus-5-5 | yes | docs/wsp/research/related_work.md (reformatted) |
| 2026-11-13 18:07 | author | Legora citation-net experiment research | opus / medium | claude-opus-5-5 | yes | docs/wsp/research/legora.md (reformatted) |
| 2026-11-13 18:09 | author | Map ACME expert and data | opus / medium | claude-opus-5-5 | yes | code map used in the session only |
| 2026-11-13 18:09 | author | Map evaluation and registry machinery | opus / medium | claude-opus-5-5 | yes | code map used in the session only |
| 2026-11-13 18:09 | author | Map BASS expert and observables | opus / medium | claude-opus-5-5 | yes | code map used in the session only |
| 2026-11-13 20:24 | author | FLEX experiment science dossier | opus / medium | claude-opus-5-5 | yes | docs/wsp/research/exp_flex.md (reformatted) |
| 2026-11-13 20:25 | author | BASS experiment science dossier | opus / medium | claude-opus-5-5 | yes | docs/wsp/research/exp_bass.md (reformatted) |
| 2026-11-13 20:25 | author | ACME CFI-G experiment dossier | opus / medium | claude-opus-5-5 | yes | docs/wsp/research/exp_acme.md (reformatted) |
| 2026-11-13 22:25 | author | Methanol train residual audit | opus / medium | claude-opus-5-5 | yes | docs/wsp/feedback/flex_methanol_d2_train_audit.md (reformatted) |
| 2026-11-13 22:25 | author | Heptane train residual audit | opus / medium | claude-opus-5-5 | yes | docs/wsp/feedback/flex_heptane_d1_train_audit.md |
| 2026-11-13 22:26 | author | BASS train video measurement | opus / medium | claude-opus-5-5 | yes | docs/wsp/feedback/bass_train_measurement.md (reformatted) |
| 2026-11-13 22:26 | author | ACME train session-transfer audit | opus / medium | claude-opus-5-5 | yes | docs/wsp/feedback/acme_d1_train_audit.md |
| 2026-11-13 22:53 | author | Wire third-generation droplet models | opus / medium | claude-opus-5-5 | yes | code (crates/ignis-wsp), reviewed by the author |
| 2026-11-13 23:14 | author | Tests for droplet3 forward | opus / medium | claude-opus-5-5 | yes | code (crates/ignis-wsp/tests), reviewed by the author |
| 2026-11-14 05:49 | author | BASS train tracks packaging | opus / medium | claude-opus-5-5 | yes | docs/wsp/feedback/bass_train_tracks.md |
| 2026-11-14 05:50 | author | BASS validation measurement B | opus / medium | claude-opus-5-5 | yes | sealed track files (runs/dev/bass_front/val_sealed, not in git); no report |
| 2026-11-14 05:50 | author | BASS validation measurement A | opus / medium | claude-opus-5-5 | yes | sealed track files (runs/dev/bass_front/val_sealed, not in git); no report |
| 2026-11-14 05:51 | author | Heptane D2 train error anatomy | opus / medium | claude-opus-5-5 | yes | docs/wsp/feedback/flex_heptane_d2_train_anatomy.md |
| 2026-11-14 05:51 | author | ACME clocks transfer audit | opus / medium | claude-opus-5-5 | yes | docs/wsp/feedback/acme_transfer_audit.md |
| 2026-11-14 05:51 | author | Methanol D3 train error anatomy | opus / medium | claude-opus-5-5 | yes | docs/wsp/feedback/flex_methanol_d3_train_anatomy.md |
| 2026-11-14 05:53 | author | Literature: microgravity droplet combustion | opus / medium | claude-opus-5-5 | yes | docs/wsp/research/lit_droplets.md |
| 2026-11-14 05:54 | author | Literature: spherical burner diffusion flames | opus / medium | claude-opus-5-5 | yes | docs/wsp/research/lit_spherical_flames.md |
| 2026-11-14 05:54 | author | Literature: concurrent flame spread theory | opus / medium | claude-opus-5-5 | yes | docs/wsp/research/lit_flame_spread.md |
| 2026-11-14 06:20 | author | Droplet fourth-generation forward | opus / medium | claude-opus-5-5 | no | code (crates/ignis-wsp), finished by the author |
| 2026-11-14 06:21 | author | Growth-clock forward tests | opus / medium | claude-opus-5-5 | yes | code (crates/ignis-wsp/tests), reviewed by the author |
| 2026-11-14 06:32 | author | BASS front scoring tool and tests | opus / medium | claude-opus-5-5 | yes | code (crates/ignis-wsp-dev), reviewed by the author |
| 2026-11-14 08:04 | author | Implement the WSP ModelService | opus / medium | claude-opus-5-5 | yes | code (crates/ignis-wsp/src/service.rs), reviewed by the author |
| 2026-11-14 09:15 | author | Covariate shift map, inputs only | opus / medium | claude-opus-5-5 | yes | docs/wsp/feedback/shift_map_validation_inputs.md |
| 2026-11-14 09:15 | author | ACME raw signals, train only | opus / medium | claude-opus-5-5 | yes | docs/wsp/feedback/acme_raw_signals_train.md |
| 2026-11-14 09:16 | author | BASS-I fabric burns as extra data | opus / medium | claude-opus-5-5 | no | — |
| 2026-11-14 09:16 | author | FLEX flame traces, train only | opus / medium | claude-opus-5-5 | yes | docs/wsp/feedback/flex_flame_traces_train.md |
| 2026-11-14 10:26 | author | Normal burner flame physics dossier | fable / high | claude-fable-5-2 | no | — |
| 2026-11-14 10:26 | author | Methanol droplet physics dossier | fable / high | claude-fable-5-2 | no | — |
| 2026-11-14 10:26 | author | Heptane droplet physics dossier | fable / high | claude-fable-5-2 | no | — |
| 2026-11-14 10:32 | author | Heptane droplet physics dossier | fable / high | claude-fable-5-2 | yes | docs/wsp/research/phys_flex_heptane.md |
| 2026-11-14 10:32 | author | Methanol droplet physics dossier | fable / high | claude-fable-5-2 | yes | docs/wsp/research/phys_flex_methanol.md |
| 2026-11-14 10:33 | author | Finish BASS-I measurement | fable / high | claude-fable-5-2 | yes | docs/wsp/feedback/bass1_psi26_tracks.md |
| 2026-11-14 10:33 | author | Inverse burner flame physics dossier | fable / high | claude-fable-5-2 | yes | docs/wsp/research/phys_acme_inverse.md |
| 2026-11-14 10:33 | author | Normal burner flame physics dossier | fable / high | claude-fable-5-2 | yes | docs/wsp/research/phys_acme_normal.md |
| 2026-11-14 10:34 | a delegate | Lit: radiative extinction and properties | opus / medium | claude-opus-5-5 | yes | not saved in the repository |
| 2026-11-14 10:34 | a delegate | Lit: methanol water uptake theory | opus / medium | claude-opus-5-5 | yes | not saved in the repository |
| 2026-11-14 10:34 | author | BASS flame spread physics dossier | fable / high | claude-fable-5-2 | yes | docs/wsp/research/phys_bass.md |
| 2026-11-14 10:38 | a delegate | Spherical flame theory and soot | inherit / (default) | claude-opus-5-5 | yes | not saved in the repository |
| 2026-11-14 10:38 | a delegate | Planck-mean fits and property data | inherit / (default) | claude-opus-5-5 | yes | not saved in the repository |
| 2026-11-14 10:38 | a delegate | Literature: spherical flame physics | opus / medium | claude-opus-5-5 | yes | not saved in the repository |
| 2026-11-14 10:38 | a delegate | Cool flames, pressure, radiometers | inherit / (default) | claude-opus-5-5 | yes | not saved in the repository |
| 2026-11-14 10:45 | a delegate | Literature: inverse flame oscillation physics | fable / high | claude-fable-5-2 | yes | not saved in the repository |
| 2026-11-14 11:39 | author | Verify property values from primary sources | fable / high | claude-fable-5-2 | yes | docs/wsp/research/properties_verified.md |
| 2026-11-14 11:39 | author | Independent check: heptane physics claims | fable / high | claude-fable-5-2 | yes | docs/wsp/feedback/verify_phys_flex_heptane.md |
| 2026-11-14 11:39 | author | Independent check: inverse flame claims | fable / high | claude-fable-5-2 | yes | docs/wsp/feedback/verify_phys_acme_inverse.md |
| 2026-11-14 11:40 | author | Independent check: normal flame claims | fable / high | claude-fable-5-2 | yes | docs/wsp/feedback/verify_phys_acme_normal.md |
| 2026-11-14 11:40 | author | Independent check: fabric flame claims | fable / high | claude-fable-5-2 | yes | docs/wsp/feedback/verify_phys_bass.md |
| 2026-11-14 11:41 | author | Implement droplet6 forward | fable / high | claude-fable-5-2 | yes | not saved in the repository |
| 2026-11-14 11:42 | author | Implement front2 forward | fable / high | claude-fable-5-2 | yes | not saved in the repository |

55 agents.
