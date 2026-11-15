//! The second committee forward of the ACME burner flames (`committee2.rs`): its spec is ten member
//! specs under ten slot prefixes plus its own ten tensors; with the weights and the offset of a
//! release of the first committee it answers as that release, bit for bit; the weights follow the
//! era of the test, read from the date the prefix gives; the planned stop is read at up to three
//! offsets; one member on one node answers bit for bit; weights that are no weights, days out of
//! order and a member that refuses are refusals; the forward identity covers the three member
//! sources; the committed roots hold their source releases' numbers unchanged.
//!
//! Every number typed here is test data on synthetic prefixes, not a candidate.

mod common;

use ignis_core::contracts::{
    CheckStatus, Family, ModelRelease, ModelSpec, PrefixBundle, RawValue, ReleaseId, SupportGate,
};
use ignis_core::registry::{LoadedRelease, TensorMap};
use ignis_wsp::committee2::{DateSource, Forecast, SLOTS};
use ignis_wsp::{
    acme_tools, assemble, audit, cli, committee, committee2, flame, flame3, flame4, models, text,
};
use serde_json::json;

const FAMILIES: [(Family, &str); 2] = [
    (Family::AcmeNormal, "normal"),
    (Family::AcmeInverse, "inverse"),
];

/// Forecast times on both sides of the stops the tests schedule.
const TIMES: [f64; 10] = [3.0, 6.0, 15.0, 40.0, 47.5, 51.0, 54.5, 90.0, 200.0, 500.0];

fn config(path: &str) -> String {
    let path = format!(
        "{}/../../configs/models/acme/{path}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn key(family: Family, generation: u8) -> String {
    let name = if family == Family::AcmeInverse {
        "acme_inverse"
    } else {
        "acme_normal"
    };
    format!("{name}{generation}")
}

/// An in-memory release of `key` from an edits text: a root, or a revision of `parent`.
fn try_release(
    key: &str,
    src: &str,
    parent: Option<&LoadedRelease>,
) -> Result<LoadedRelease, String> {
    let model = models::by_key(key).expect("model");
    let spec = model.spec();
    let ops = text::compile(src, &spec).unwrap_or_else(|e| panic!("{key}: {e}"));
    let mut meta = common::meta("test-committee2");
    meta.parent_release_id = parent.map(|p| ReleaseId::new(p.release.id.clone()).unwrap());
    let patch = assemble::make_patch(&spec, meta, ops);
    let tensors = assemble::materialize(&spec, &patch, parent).expect("materialize");
    let mut release = model
        .build_release(&patch, &tensors, parent, &common::binding())
        .map_err(|e| format!("{e:#}"))?;
    // the content id the registry would give it (a revision names it)
    release.id = ignis_core::hashing::content_hash(&release, &["id", "created_at"]);
    Ok(LoadedRelease {
        release,
        params: tensors,
        posterior: TensorMap::new(),
    })
}

fn release(key: &str, src: &str) -> LoadedRelease {
    try_release(key, src, None).unwrap_or_else(|e| panic!("{key}: {e}"))
}

fn copy(r: &LoadedRelease) -> LoadedRelease {
    LoadedRelease {
        release: r.release.clone(),
        params: r.params.clone(),
        posterior: TensorMap::new(),
    }
}

fn set(r: &mut LoadedRelease, tensor: &str, data: &[f64]) {
    let t = r.params.get_mut(tensor).expect("tensor");
    assert_eq!(t.data.len(), data.len());
    t.data = data.to_vec();
}

fn list(v: &[f64]) -> String {
    v.iter()
        .map(|x| text::literal(*x))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The committee's own tensors.
#[derive(Clone)]
struct Own {
    w_early: [f64; SLOTS],
    w_late: [f64; SLOTS],
    ws_early: [f64; SLOTS],
    ws_late: [f64; SLOTS],
    last_early: f64,
    first_late: f64,
    p_between: f64,
    p_unknown: f64,
    offset_s: [f64; 3],
    offset_w: [f64; 3],
}

/// `v` in the first slots, zero in the rest.
fn pad(v: &[f64]) -> [f64; SLOTS] {
    let mut out = [0.0; SLOTS];
    out[..v.len()].copy_from_slice(v);
    out
}

impl Own {
    /// The values of the committed roots.
    fn root() -> Own {
        Own {
            w_early: pad(&[1.0, 0.0, 1.0]),
            w_late: pad(&[1.0, 0.0, 1.0]),
            ws_early: pad(&[]),
            ws_late: pad(&[]),
            last_early: 1_900_001.0,
            first_late: 2_100_366.0,
            p_between: 0.5,
            p_unknown: 0.5,
            offset_s: [0.0; 3],
            offset_w: [1.0, 0.0, 0.0],
        }
    }

    fn text(&self) -> String {
        format!(
            "mix.w_early[0:10] = [{}]\nmix.w_late[0:10] = [{}]\nmix.ws_early[0:10] = [{}]\n\
             mix.ws_late[0:10] = [{}]\nera.last_early_day = {}\nera.first_late_day = {}\n\
             era.p_late_between = {}\nera.p_late_unknown = {}\nscript.offset_s[0:3] = [{}]\n\
             script.offset_w[0:3] = [{}]\n",
            list(&self.w_early),
            list(&self.w_late),
            list(&self.ws_early),
            list(&self.ws_late),
            text::literal(self.last_early),
            text::literal(self.first_late),
            text::literal(self.p_between),
            text::literal(self.p_unknown),
            list(&self.offset_s),
            list(&self.offset_w),
        )
    }
}

/// The reference baselines of the three member specs and the root of the first committee spec.
struct Sources {
    family: Family,
    a: LoadedRelease,
    b: LoadedRelease,
    c: LoadedRelease,
    w5: LoadedRelease,
}

fn sources(family: Family, dir: &str) -> Sources {
    Sources {
        family,
        a: release(
            &key(family, 2),
            &config(&format!("wsp2/{dir}/baseline-clock/edits.wsp")),
        ),
        b: release(
            &key(family, 3),
            &config(&format!("wsp3/{dir}/baseline-theory/edits.wsp")),
        ),
        c: release(
            &key(family, 8),
            &config(&format!("wsp8/{dir}/baseline-clock-theory/edits.wsp")),
        ),
        w5: release(
            &key(family, 5),
            &config(&format!("wsp5/{dir}/baseline-set/edits.wsp")),
        ),
    }
}

/// The edits of a release of the second committee: `own`, the six slots of a release of the first
/// committee statement by statement, and a `flame4.rs` release in each of the four `c` slots.
fn edits7(family: Family, w5: &LoadedRelease, c: &LoadedRelease, own: &Own) -> String {
    let mut src = own.text();
    for line in cli::edits_of(&committee::spec(family), w5).lines() {
        if committee::SLOT_PREFIX.iter().any(|p| line.starts_with(p)) {
            src.push_str(line);
            src.push('\n');
        }
    }
    let member = cli::edits_of(&flame4::spec(family), c);
    for slot in &committee2::SLOT_PREFIX[6..] {
        for line in member.lines() {
            src.push_str(slot);
            src.push_str(line);
            src.push('\n');
        }
    }
    src
}

fn wsp7(family: Family, w5: &LoadedRelease, c: &LoadedRelease, own: &Own) -> LoadedRelease {
    release(&key(family, 7), &edits7(family, w5, c, own))
}

/// The committed root of the first committee spec with other weights and another offset.
fn wsp5(family: Family, dir: &str, w: [f64; 6], w_script: [f64; 6], onset: f64) -> LoadedRelease {
    let mut src = config(&format!("wsp5/{dir}/baseline-set/edits.wsp"));
    for (typed, new) in [
        (
            "mix.w[0:6] = [1.0, 0.0, 1.0, 0.0, 0.0, 0.0]",
            format!("mix.w[0:6] = [{}]", list(&w)),
        ),
        (
            "mix.w_script[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]",
            format!("mix.w_script[0:6] = [{}]", list(&w_script)),
        ),
        (
            "script.onset_s = 0.0",
            format!("script.onset_s = {}", text::literal(onset)),
        ),
    ] {
        assert!(src.contains(typed), "the root types {typed}");
        src = src.replace(typed, &new);
    }
    release(&key(family, 5), &src)
}

fn rehash(p: &mut PrefixBundle) {
    p.input_hash = ignis_core::hashing::content_hash(&*p, &["input_hash"]);
}

/// The prefix with the script's planned stop as a condition.
fn with_stop(p: &PrefixBundle, t: f64) -> PrefixBundle {
    let mut p = p.clone();
    let mut stop = p.known_conditions["pressure"].clone();
    stop.raw_value = Some(RawValue::Number(t));
    stop.value_si = Some(t);
    stop.raw_unit = Some("s".into());
    stop.unit_si = Some("s".into());
    p.known_conditions
        .insert(committee2::PROGRAMMED_STOP.into(), stop);
    rehash(&mut p);
    p
}

/// The prefix with a session id and, when given, another run id.
fn dated(p: &PrefixBundle, session: Option<&str>, run: Option<&str>) -> PrefixBundle {
    let mut p = p.clone();
    p.session_id = session.map(str::to_string);
    if let Some(run) = run {
        p.run_id = run.to_string();
    }
    rehash(&mut p);
    p
}

fn settings() -> serde_json::Value {
    json!({ "times_s": TIMES })
}

fn run7(family: Family, r: &LoadedRelease, p: &PrefixBundle) -> Forecast {
    committee2::forecast(&committee2::spec(family), r, p, &settings())
        .expect("runnable")
        .unwrap_or_else(|s| panic!("refused: {:?}", s.reasons))
}

fn slots_of(f: &Forecast) -> Vec<&'static str> {
    f.members.iter().map(|m| m.slot).collect()
}

/// `sum_i w[i] t[i]` row by row.
fn mean(parts: &[(f64, &[Vec<f64>; 4])]) -> [Vec<f64>; 4] {
    std::array::from_fn(|k| {
        (0..parts[0].1[k].len())
            .map(|j| parts.iter().map(|(w, t)| w * t[k][j]).sum())
            .collect()
    })
}

fn assert_close(have: &[Vec<f64>; 4], want: &[Vec<f64>; 4], what: &str) {
    for k in 0..4 {
        assert_eq!(have[k].len(), want[k].len());
        for j in 0..have[k].len() {
            assert!(
                (have[k][j] - want[k][j]).abs() < 1e-12,
                "{what}: row {k} at {} s: {} against {}",
                TIMES[j],
                have[k][j],
                want[k][j]
            );
        }
    }
}

/// The three member baselines run alone on a prefix.
fn alone(s: &Sources, p: &PrefixBundle) -> [[Vec<f64>; 4]; 3] {
    let f = s.family;
    [
        flame::forecast(&flame::spec(f), &s.a, p, &settings())
            .unwrap()
            .expect("member a admitted")
            .table,
        flame3::forecast(&flame3::spec(f), &s.b, p, &settings())
            .unwrap()
            .expect("member b admitted")
            .table,
        flame4::forecast(&flame4::spec(f), &s.c, p, &settings())
            .unwrap()
            .expect("member c admitted")
            .table,
    ]
}

#[test]
fn the_committed_specs_are_the_member_specs_under_ten_prefixes() {
    assert_eq!(committee2::ALLOWED, committee::ALLOWED);
    assert_eq!(committee2::SLOT_PREFIX[..6], committee::SLOT_PREFIX);
    for (family, dir) in FAMILIES {
        let spec = committee2::spec(family);
        spec.validate().expect("spec valid");
        assert_eq!(
            models::by_key(&key(family, 7)).unwrap().spec().spec_hash(),
            spec.spec_hash()
        );
        let on_disk: ModelSpec =
            serde_json::from_str(&config(&format!("wsp7/{dir}/spec.json"))).unwrap();
        assert_eq!(
            on_disk.spec_hash(),
            spec.spec_hash(),
            "configs/models/acme/wsp7/{dir}/spec.json drifted from committee2::spec(); a changed \
             spec is a new spec id (re-export with `ignis wsp spec --family {} --out …`), never an \
             in-place edit",
            key(family, 7)
        );
        let members = [
            flame::spec(family),
            flame3::spec(family),
            flame4::spec(family),
        ];
        let kind = |i: usize| match i {
            0..=1 => 0,
            2..=5 => 1,
            _ => 2,
        };
        assert_eq!(
            spec.tensors.len(),
            committee2::OWN_TENSORS.len()
                + (0..SLOTS)
                    .map(|i| members[kind(i)].tensors.len())
                    .sum::<usize>()
        );
        for (i, name) in committee2::OWN_TENSORS.iter().enumerate() {
            assert_eq!(&spec.tensors[i].name, name);
        }
        for t in &spec.tensors[..4] {
            assert_eq!(t.shape, vec![SLOTS as u32]);
        }
        assert_eq!(spec.tensors[8].shape, vec![3]);
        assert_eq!(spec.tensors[9].shape, vec![3]);
        let mut at = committee2::OWN_TENSORS.len();
        for (i, prefix) in committee2::SLOT_PREFIX.iter().enumerate() {
            for want in &members[kind(i)].tensors {
                let mut want = want.clone();
                want.name = format!("{prefix}{}", want.name);
                assert_eq!(&spec.tensors[at], &want);
                at += 1;
            }
        }
        // the six slots it shares with the first committee spec are that spec's, name for name
        let first = committee::spec(family);
        for t in first
            .tensors
            .iter()
            .filter(|t| committee::SLOT_PREFIX.iter().any(|p| t.name.starts_with(p)))
        {
            assert_eq!(spec.tensor(&t.name), Some(t), "{}", t.name);
        }
    }
}

#[test]
fn with_the_weights_of_a_release_of_the_first_committee_it_answers_as_that_release() {
    let neutral = |w5: &LoadedRelease, own: Own| {
        let onset = w5.params["script.onset_s"].data[0];
        Own {
            w_early: pad(&w5.params["mix.w"].data),
            w_late: pad(&w5.params["mix.w"].data),
            ws_early: pad(&w5.params["mix.w_script"].data),
            ws_late: pad(&w5.params["mix.w_script"].data),
            offset_s: [0.0 - onset, 0.0, 0.0],
            offset_w: [1.0, 0.0, 0.0],
            ..own
        }
    };
    for (family, dir) in FAMILIES {
        let s = sources(family, dir);
        let mut firsts = vec![
            ("the root", copy(&s.w5)),
            (
                "the root with other weights and the planned stop read",
                wsp5(
                    family,
                    dir,
                    [1.0, 2.0, 0.5, 0.0, 0.0, 0.5],
                    [0.0, 1.0, 0.0, 3.0, 0.0, 0.0],
                    0.4,
                ),
            ),
        ];
        if family == Family::AcmeNormal {
            // the committed candidate of the first committee spec: a revision of its root
            firsts.push((
                "p1-fable",
                try_release(
                    &key(family, 5),
                    &config("wsp5/normal/p1-fable/edits.wsp"),
                    Some(&s.w5),
                )
                .expect("p1-fable"),
            ));
        }
        let fixture = acme_tools::default_fixture(family);
        let prefixes = [
            ("no planned stop", fixture.clone()),
            ("planned stop", with_stop(&fixture, 51.0)),
            (
                "planned stop, dated",
                dated(&with_stop(&fixture, 51.0), Some("2021-03-29"), None),
            ),
        ];
        for (what, w5) in &firsts {
            for era in [
                Own::root(),
                // any probability of the late era: both eras hold the same weights
                Own {
                    p_between: 0.3,
                    p_unknown: 0.7,
                    ..Own::root()
                },
                Own {
                    last_early: 2_021_200.0,
                    first_late: 2_021_201.0,
                    ..Own::root()
                },
            ] {
                let w7 = wsp7(family, w5, &s.c, &neutral(w5, era));
                for (which, p) in &prefixes {
                    let f5 = committee::forecast(&committee::spec(family), w5, p, &settings())
                        .unwrap()
                        .unwrap_or_else(|s| panic!("{what}, {which}: {:?}", s.reasons));
                    let f7 = run7(family, &w7, p);
                    assert_eq!(f7.times_s, f5.times_s, "{what}, {which}");
                    assert_eq!(f7.table, f5.table, "{what}, {which}");
                    assert_eq!(
                        slots_of(&f7),
                        f5.members.iter().map(|m| m.slot).collect::<Vec<_>>()
                    );
                    for (m7, m5) in f7.members.iter().zip(&f5.members) {
                        assert_eq!(m7.weight, m5.weight);
                        assert_eq!(m7.table, m5.table);
                        assert_eq!(m7.scheduled_stop_s, m5.scheduled_stop_s);
                    }
                    let stop7 = match f7.offsets.as_slice() {
                        [] => None,
                        [one] => Some(one.scheduled_stop_s),
                        more => panic!("{} nodes", more.len()),
                    };
                    assert_eq!(stop7, f5.scheduled_stop_s, "{what}, {which}");
                }
            }
            // the release under test reads the planned stop exactly when the first one does
            let reads = w5.params["mix.w_script"].data.iter().any(|x| *x > 0.0);
            let w7 = wsp7(family, w5, &s.c, &neutral(w5, Own::root()));
            assert_eq!(
                !run7(family, &w7, &prefixes[1].1).offsets.is_empty(),
                reads,
                "{what}"
            );
        }
    }
}

#[test]
fn the_weights_follow_the_era_of_the_test() {
    for (family, dir) in FAMILIES {
        let s = sources(family, dir);
        let own = Own {
            w_early: [2.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            w_late: [0.0, 1.0, 0.0, 0.0, 0.0, 3.0, 1.0, 0.0, 0.0, 0.0],
            last_early: 2_021_100.0,
            first_late: 2_021_200.0,
            p_between: 0.3,
            p_unknown: 0.6,
            ..Own::root()
        };
        let r = wsp7(family, &s.w5, &s.c, &own);
        let fixture = acme_tools::default_fixture(family);
        let on = |session: &str| run7(family, &r, &dated(&fixture, Some(session), None));
        let [a, b, c] = alone(&s, &fixture);

        // at or before the last early day: the early weights alone
        let early = on("2021-03-29");
        assert_eq!(early.p_late, 0.0);
        assert_eq!(slots_of(&early), ["a0.", "b0."]);
        assert_close(
            &early.table,
            &mean(&[(2.0 / 3.0, &a), (1.0 / 3.0, &b)]),
            "early",
        );
        let last_early = on("2021-04-10");
        assert_eq!(last_early.date.unwrap().ordinal(), 2_021_100);
        assert_eq!(last_early.p_late, 0.0);
        assert_eq!(last_early.table, early.table);

        // at or after the first late day: the late weights alone
        let late = on("2021-07-19");
        assert_eq!(late.date.unwrap().ordinal(), 2_021_200);
        assert_eq!(late.p_late, 1.0);
        assert_eq!(slots_of(&late), ["a1.", "b3.", "c0."]);
        assert_close(
            &late.table,
            &mean(&[(0.2, &a), (0.6, &b), (0.2, &c)]),
            "late",
        );
        assert_eq!(on("2022-01-05").table, late.table);

        // strictly between, and without a date: the two forecasts with the typed probability
        for (session, p_late) in [
            (Some("2021-06-01"), 0.3),
            (Some("2021-04-11"), 0.3),
            (Some("2021-07-18"), 0.3),
            (None, 0.6),
        ] {
            let f = run7(family, &r, &dated(&fixture, session, None));
            assert_eq!(f.p_late, p_late, "{session:?}");
            assert_eq!(slots_of(&f), ["a0.", "a1.", "b0.", "b3.", "c0."]);
            assert_close(
                &f.table,
                &mean(&[(1.0 - p_late, &early.table), (p_late, &late.table)]),
                "between",
            );
            for j in 0..TIMES.len() {
                let sum: f64 = (0..4).map(|k| f.table[k][j]).sum();
                assert!((sum - 1.0).abs() < 1e-12, "rows sum to {sum}");
            }
        }

        // the diagnostics say what was read and which slots were run
        let model = models::by_key(&key(family, 7)).unwrap();
        for (session, source, date, day, p_late, slots) in [
            (
                Some("2021-03-29"),
                "session_id",
                json!("2021-03-29"),
                json!(2_021_088),
                0.0,
                vec!["a0", "b0"],
            ),
            (
                Some("2021-07-19"),
                "session_id",
                json!("2021-07-19"),
                json!(2_021_200),
                1.0,
                vec!["a1", "b3", "c0"],
            ),
            (
                None,
                "none",
                json!(null),
                json!(null),
                0.6,
                vec!["a0", "a1", "b0", "b3", "c0"],
            ),
        ] {
            let artifact = model
                .artifact(&r, &dated(&fixture, session, None), &settings())
                .expect("artifact");
            artifact.validate().expect("valid artifact");
            assert!(artifact.support.admitted);
            let d = &artifact.diagnostics;
            assert_eq!(d["test_date_source"], json!(source));
            assert_eq!(d["test_date"], date);
            assert_eq!(d["test_date_ordinal"], day);
            assert_eq!(d["p_late"], json!(p_late));
            assert_eq!(d["weight_tensors"], json!(["mix.w_early", "mix.w_late"]));
            assert_eq!(d["script_offsets"], json!([]));
            let members = d["members"].as_array().expect("members");
            let run: Vec<&str> = members
                .iter()
                .map(|m| m["slot"].as_str().unwrap())
                .collect();
            assert_eq!(run, slots);
            assert!(members.iter().all(|m| m["node"].is_null()));
            assert_eq!(
                members.last().unwrap()["forward"],
                json!(if slots.last() == Some(&"c0") {
                    "ignis_wsp::flame4::forecast"
                } else {
                    "ignis_wsp::flame3::forecast"
                })
            );
        }

        // a vector whose era has probability zero is neither checked nor used
        let mut no_late = copy(&r);
        set(&mut no_late, "mix.w_late", &[0.0; SLOTS]);
        let p = dated(&fixture, Some("2021-03-29"), None);
        assert_eq!(run7(family, &no_late, &p).table, early.table);
        let mut bad_late = copy(&r);
        set(&mut bad_late, "mix.w_late", &pad(&[-1.0, 2.0]));
        assert_eq!(run7(family, &bad_late, &p).table, early.table);
        let mut bad_early = copy(&r);
        set(&mut bad_early, "mix.w_early", &pad(&[-1.0, 2.0]));
        let p = dated(&fixture, Some("2021-07-19"), None);
        assert_eq!(run7(family, &bad_early, &p).table, late.table);
    }
}

#[test]
fn the_date_of_a_test_is_read_from_the_session_then_from_the_run() {
    let fixture = acme_tools::default_fixture(Family::AcmeNormal);
    let read = |session: Option<&str>, run: Option<&str>| {
        committee2::test_date(&dated(&fixture, session, run))
            .map(|d| (d.ordinal(), d.source, d.iso()))
    };
    let session = |day: u32, iso: &str| Some((day, DateSource::SessionId, iso.to_string()));
    let run = |day: u32, iso: &str| Some((day, DateSource::RunId, iso.to_string()));
    // a session id that contains a date
    assert_eq!(
        read(Some("2021-03-29"), None),
        session(2_021_088, "2021-03-29")
    );
    assert_eq!(
        read(Some("acme_normal:2021-03-29:GMT088"), None),
        session(2_021_088, "2021-03-29")
    );
    // otherwise the run id: five digits YYDDD after its last colon
    assert_eq!(
        read(None, Some("PSI-159:21270H3")),
        run(2_021_270, "2021-09-27")
    );
    assert_eq!(read(None, Some("21270H3")), run(2_021_270, "2021-09-27"));
    assert_eq!(
        read(None, Some("a:22001:PSI-159:20366B1")),
        run(2_020_366, "2020-12-31")
    );
    // the session id wins when both give a date
    assert_eq!(
        read(Some("2021-03-29"), Some("PSI-159:21270H3")),
        session(2_021_088, "2021-03-29")
    );
    // a session id without a date leaves the run id
    assert_eq!(
        read(Some("session 7"), Some("PSI-159:21270H3")),
        run(2_021_270, "2021-09-27")
    );
    // the first substring that is a calendar date: 30 February and month 13 are not
    assert_eq!(
        read(Some("2021-02-30 2021-13-01 2020-02-29 2021-03-29"), None),
        session(2_020_060, "2020-02-29")
    );
    assert_eq!(read(Some("2021-02-29"), None), None);
    assert_eq!(read(Some("2021-3-29"), None), None);
    // a run id that does not start with five digits gives no date
    for id in [
        "PSI-159:H21270",
        "PSI-159:2127",
        "PSI-159:2127xH3",
        "PSI-159:",
        "wsp-fixture-acme_normal",
    ] {
        assert_eq!(read(None, Some(id)), None, "{id}");
    }
    // day 366 of a common year, day 0 and day 367 are no days
    assert_eq!(read(None, Some("PSI-159:21366H3")), None);
    assert_eq!(read(None, Some("PSI-159:21000H3")), None);
    assert_eq!(read(None, Some("PSI-159:20367H3")), None);
    assert_eq!(
        read(None, Some("PSI-159:21365H3")),
        run(2_021_365, "2021-12-31")
    );

    // and the forward uses what was read
    let s = sources(Family::AcmeNormal, "normal");
    let own = Own {
        w_early: pad(&[1.0]),
        w_late: pad(&[0.0, 0.0, 1.0]),
        last_early: 2_021_100.0,
        first_late: 2_021_200.0,
        p_between: 0.25,
        p_unknown: 0.75,
        ..Own::root()
    };
    let r = wsp7(Family::AcmeNormal, &s.w5, &s.c, &own);
    for (session, run, p_late) in [
        (None, Some("PSI-159:21270H3"), 1.0),
        (None, Some("PSI-159:21088H3"), 0.0),
        (None, Some("PSI-159:21150H3"), 0.25),
        (Some("2021-03-29"), Some("PSI-159:21270H3"), 0.0),
        (None, Some("PSI-159:H21270"), 0.75),
        (None, Some("PSI-159:21366H3"), 0.75),
    ] {
        let f = run7(Family::AcmeNormal, &r, &dated(&fixture, session, run));
        assert_eq!(f.p_late, p_late, "{session:?} {run:?}");
    }
}

#[test]
fn the_planned_stop_is_read_at_each_weighted_offset() {
    for (family, dir) in FAMILIES {
        let s = sources(family, dir);
        let ws = [1.0, 0.0, 2.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
        let own = Own {
            ws_early: ws,
            ws_late: ws,
            offset_s: [-3.0, 0.5, 4.0],
            offset_w: [1.0, 2.0, 0.5],
            ..Own::root()
        };
        let r = wsp7(family, &s.w5, &s.c, &own);
        let fixture = acme_tools::default_fixture(family);
        let p = with_stop(&fixture, 50.0);
        let f = run7(family, &r, &p);
        // three nodes, three slots each: nodes in the outer order, slots in the inner
        assert_eq!(f.weight_tensors, ["mix.ws_early", "mix.ws_late"]);
        assert_eq!(f.offsets.len(), 3);
        for (k, (node, (offset, weight))) in f
            .offsets
            .iter()
            .zip([(-3.0, 1.0 / 3.5), (0.5, 2.0 / 3.5), (4.0, 0.5 / 3.5)])
            .enumerate()
        {
            assert_eq!(node.node, k);
            assert_eq!(node.offset_s, offset);
            assert_eq!(node.scheduled_stop_s, 50.0 + offset);
            assert!((node.weight - weight).abs() < 1e-15);
        }
        assert_eq!(
            slots_of(&f),
            ["a0.", "b0.", "c0.", "a0.", "b0.", "c0.", "a0.", "b0.", "c0."]
        );
        for (i, m) in f.members.iter().enumerate() {
            assert_eq!(m.node, Some(i / 3));
            assert_eq!(m.scheduled_stop_s, Some(f.offsets[i / 3].scheduled_stop_s));
            assert_eq!(m.node_weight, f.offsets[i / 3].weight);
        }
        // the weighted sum of the three single-node runs
        let mut singles = Vec::new();
        for k in 0..3 {
            let mut one = copy(&r);
            let mut w = [0.0; 3];
            w[k] = own.offset_w[k];
            set(&mut one, "script.offset_w", &w);
            let g = run7(family, &one, &p);
            assert_eq!(g.offsets.len(), 1);
            assert_eq!(g.offsets[0].node, k);
            assert_eq!(g.offsets[0].weight, 1.0);
            // a single node is the members run on the prefix with that stop as a known control
            let mut stopped = fixture.clone();
            stopped
                .known_controls
                .push(ignis_core::contracts::ControlStage {
                    t_start_s: 50.0 + own.offset_s[k],
                    t_end_s: None,
                    control: "scheduled_stop".into(),
                    value: None,
                    operator_action: false,
                });
            let [a, b, c] = alone(&s, &stopped);
            assert_close(
                &g.table,
                &mean(&[(0.25, &a), (0.5, &b), (0.25, &c)]),
                "one node",
            );
            singles.push(g.table);
        }
        assert_close(
            &f.table,
            &mean(&[
                (1.0 / 3.5, &singles[0]),
                (2.0 / 3.5, &singles[1]),
                (0.5 / 3.5, &singles[2]),
            ]),
            "three nodes",
        );
        // between the first stop and the last the forecast holds flames of both kinds
        let j = TIMES.iter().position(|t| *t == 51.0).unwrap();
        assert!(f.table[0][j] > 0.0 && f.table[0][j] < singles[2][0][j]);
        assert_eq!(f.table[0][j + 1], 0.0, "every node has stopped by 54.5 s");

        // the artifact reports the offsets and, per member, its node
        let artifact = models::by_key(&key(family, 7))
            .unwrap()
            .artifact(&r, &p, &settings())
            .expect("artifact");
        artifact.validate().expect("valid artifact");
        let d = &artifact.diagnostics;
        assert_eq!(d["weight_tensors"], json!(["mix.ws_early", "mix.ws_late"]));
        assert_eq!(d["script_offsets"].as_array().unwrap().len(), 3);
        assert_eq!(d["script_offsets"][1]["offset_s"], json!(0.5));
        assert_eq!(d["script_offsets"][1]["scheduled_stop_s"], json!(50.5));
        assert_eq!(d["members"][4]["node"], json!(1));
        assert_eq!(d["members"][4]["slot"], json!("b0"));
        assert!(d["scheduled_stop_from_script_s"].is_null());

        // without the condition, or without a weight for it, the planned stop is not read
        let plain = run7(family, &r, &fixture);
        assert!(plain.offsets.is_empty());
        assert_eq!(plain.weight_tensors, ["mix.w_early", "mix.w_late"]);
        let mut deaf = copy(&r);
        set(&mut deaf, "script.offset_w", &[0.0; 3]);
        let unread = run7(family, &deaf, &p);
        assert!(unread.offsets.is_empty());
        assert_eq!(unread.table, plain.table);
        let mut silent = copy(&r);
        set(&mut silent, "mix.ws_early", &[0.0; SLOTS]);
        set(&mut silent, "mix.ws_late", &[0.0; SLOTS]);
        assert_eq!(run7(family, &silent, &p).table, plain.table);
        let mut unresolved = p.clone();
        unresolved
            .known_conditions
            .get_mut(committee2::PROGRAMMED_STOP)
            .unwrap()
            .value_si = Some(0.0);
        assert_eq!(run7(family, &r, &unresolved).table, plain.table);
    }
}

#[test]
fn one_member_on_one_node_answers_bit_for_bit() {
    for (family, dir) in FAMILIES {
        let s = sources(family, dir);
        // a `flame4.rs` release that is not the root: one burner limit that is reached (test data)
        let heated = config(&format!("wsp8/{dir}/baseline-clock-theory/edits.wsp"))
            .replace("bt.limit_c[0] = 0.0", "bt.limit_c[0] = 100.0")
            .replace("bt.limit_w[0] = 0.0", "bt.limit_w[0] = 1.0");
        let c = release(&key(family, 8), &heated);
        let fixture = acme_tools::default_fixture(family);
        let own_table = flame4::forecast(&flame4::spec(family), &c, &fixture, &settings())
            .unwrap()
            .expect("admitted");
        assert!(own_table.clocks.burner_limits[0].2.is_some());
        assert_ne!(own_table.table, alone(&s, &fixture)[2], "the limit is felt");

        // all weight on c0
        let only = |slot: usize| Own {
            w_early: std::array::from_fn(|i| if i == slot { 2.5 } else { 0.0 }),
            w_late: std::array::from_fn(|i| if i == slot { 0.5 } else { 0.0 }),
            p_unknown: 0.3,
            ..Own::root()
        };
        let r = wsp7(family, &s.w5, &c, &only(6));
        let f = run7(family, &r, &fixture);
        assert_eq!(slots_of(&f), ["c0."]);
        assert_eq!(f.table, own_table.table);
        assert_eq!(f.members[0].table, own_table.table);
        // and the other kinds of member, alone
        let [a, b, _] = alone(&s, &fixture);
        for (slot, name, want) in [(1, "a1.", &a), (5, "b3.", &b)] {
            let r = wsp7(family, &s.w5, &c, &only(slot));
            let f = run7(family, &r, &fixture);
            assert_eq!(slots_of(&f), [name]);
            assert_eq!(&f.table, want);
        }
        // one slot on one node of the planned stop: the member under that stop
        let scripted = Own {
            ws_early: only(9).w_early,
            ws_late: only(9).w_late,
            offset_s: [0.0, 2.0, 0.0],
            offset_w: [0.0, 4.0, 0.0],
            ..only(6)
        };
        let r = wsp7(family, &s.w5, &c, &scripted);
        let f = run7(family, &r, &with_stop(&fixture, 45.0));
        assert_eq!(slots_of(&f), ["c3."]);
        assert_eq!(f.members[0].node, Some(1));
        let mut stopped = fixture.clone();
        stopped
            .known_controls
            .push(ignis_core::contracts::ControlStage {
                t_start_s: 47.0,
                t_end_s: None,
                control: "scheduled_stop".into(),
                value: None,
                operator_action: false,
            });
        let want = flame4::forecast(&flame4::spec(family), &c, &stopped, &settings())
            .unwrap()
            .expect("admitted");
        assert_eq!(want.scheduled_stop_s, Some(47.0));
        assert_eq!(f.table, want.table);
        // a release of the second committee is not a member release
        assert!(flame4::forecast(&flame4::spec(family), &r, &fixture, &settings()).is_err());
    }
}

#[test]
fn days_out_of_order_weights_that_are_none_and_a_member_refusal_are_refused() {
    let family = Family::AcmeInverse;
    let spec = committee2::spec(family);
    let s = sources(family, "inverse");
    let fixture = acme_tools::default_fixture(family);
    let good = wsp7(family, &s.w5, &s.c, &Own::root());

    // the last early day at or after the first late day: the release cannot be run, so it cannot
    // be assembled either (its finite-forward build check fails)
    for (last, first) in [(2_021_200.0, 2_021_200.0), (2_021_201.0, 2_021_200.0)] {
        let own = Own {
            last_early: last,
            first_late: first,
            ..Own::root()
        };
        let err = try_release(&key(family, 7), &edits7(family, &s.w5, &s.c, &own), None)
            .err()
            .expect("days out of order must not assemble");
        assert!(err.contains("failed build check"), "{err}");
        let mut r = copy(&good);
        set(&mut r, "era.last_early_day", &[last]);
        set(&mut r, "era.first_late_day", &[first]);
        let refusal = committee2::forecast(&spec, &r, &fixture, &settings())
            .err()
            .expect("days out of order");
        assert_eq!(refusal.code, "era_days");
    }

    // a negative weight, and a vector with no weight, in a vector that is used
    let stop = with_stop(&fixture, 50.0);
    let scripted = wsp7(
        family,
        &s.w5,
        &s.c,
        &Own {
            ws_early: pad(&[1.0]),
            ws_late: pad(&[1.0]),
            ..Own::root()
        },
    );
    for (base, p, tensor, data) in [
        (&good, &fixture, "mix.w_early", pad(&[1.0, -0.5])),
        (&good, &fixture, "mix.w_late", pad(&[1.0, -0.5])),
        (&good, &fixture, "mix.w_early", pad(&[])),
        (&good, &fixture, "mix.w_late", pad(&[])),
        (&scripted, &stop, "mix.ws_early", pad(&[1.0, -0.5])),
        (&scripted, &stop, "mix.ws_late", pad(&[])),
    ] {
        let mut r = copy(base);
        set(&mut r, tensor, &data);
        let refused = committee2::forecast(&spec, &r, p, &settings())
            .unwrap()
            .expect_err(tensor);
        assert_eq!(refused.gate, SupportGate::Semantic);
        assert!(
            refused.reasons[0].starts_with(tensor),
            "{:?}",
            refused.reasons
        );
    }
    // a negative weight of an offset
    let mut r = copy(&scripted);
    set(&mut r, "script.offset_w", &[1.0, -0.5, 0.0]);
    let refused = committee2::forecast(&spec, &r, &stop, &settings())
        .unwrap()
        .expect_err("offset weights");
    assert_eq!(refused.gate, SupportGate::Semantic);
    assert!(refused.reasons[0].starts_with("script.offset_w"));
    // weights that sum to nothing cannot become a release
    let own = Own {
        w_early: pad(&[]),
        ..Own::root()
    };
    assert!(try_release(&key(family, 7), &edits7(family, &s.w5, &s.c, &own), None).is_err());

    // a member that refuses names its slot, and on the script path its node
    let mut not_established = fixture.clone();
    not_established.phase_established_at_s = None;
    let r = wsp7(
        family,
        &s.w5,
        &s.c,
        &Own {
            w_early: pad(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]),
            w_late: pad(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]),
            ws_early: pad(&[0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
            ws_late: pad(&[0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
            offset_s: [0.0, 0.0, -18.0],
            offset_w: [0.0, 1.0, 1.0],
            ..Own::root()
        },
    );
    let refused = committee2::forecast(&spec, &r, &not_established, &settings())
        .unwrap()
        .expect_err("phase not established");
    assert!(
        refused.reasons[0].starts_with("member b1: "),
        "{:?}",
        refused.reasons
    );
    // node 1 stops at 20 s; node 2 puts the stop at 2 s, before the cutoff of 3 s: the members
    // refuse it there
    let stop = with_stop(&fixture, 20.0);
    let refused = committee2::forecast(&spec, &r, &stop, &settings())
        .unwrap()
        .expect_err("a stop before the cutoff");
    assert_eq!(refused.gate, SupportGate::Semantic);
    assert!(
        refused.reasons[0].starts_with("member a1, node 2: "),
        "{:?}",
        refused.reasons
    );
    assert!(refused.reasons[0].contains("at or before the cutoff"));
    // and the artifact of a refusal carries no numbers
    let artifact = models::by_key(&key(family, 7))
        .unwrap()
        .artifact(&r, &stop, &settings())
        .expect("artifact");
    artifact.validate().expect("valid artifact");
    assert!(!artifact.support.admitted && artifact.event_cdf.is_empty());
}

#[test]
fn the_forward_identity_covers_the_three_member_sources() {
    for name in ["acme_normal.wsp7", "acme_inverse.wsp7"] {
        let files: Vec<&str> = audit::forward_sources_of(name)
            .expect("a literal model")
            .iter()
            .map(|(n, _)| *n)
            .collect();
        for f in ["flame.rs", "flame3.rs", "flame4.rs", "committee2.rs"] {
            assert!(files.contains(&f), "{name} lacks {f}: {files:?}");
        }
        assert!(!files.contains(&"committee.rs"), "{files:?}");
        assert_eq!(audit::banned_hits_of(name), Vec::<String>::new());
    }
    let c = audit::forward_hash_of("acme_inverse.wsp7");
    assert_eq!(c, audit::forward_hash_of("acme_normal.wsp7"));
    for other in [
        "acme_inverse.wsp8",
        "acme_inverse.wsp6",
        "acme_inverse.wsp5",
        "acme_inverse.wsp3",
    ] {
        assert_ne!(c, audit::forward_hash_of(other), "{other}");
    }
}

#[test]
fn the_committed_roots_hold_their_source_releases_numbers_unchanged() {
    let read = |path: &str| -> ModelRelease {
        serde_json::from_str(&config(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
    };
    for (family, dir) in FAMILIES {
        let root = read(&format!("wsp7/{dir}/baseline-set/release.json"));
        let first = read(&format!("wsp5/{dir}/baseline-set/release.json"));
        let hybrid = read(&format!("wsp8/{dir}/baseline-clock-theory/release.json"));
        let entry = |r: &ModelRelease, name: &str| {
            r.tensors
                .iter()
                .find(|e| e.name == name)
                .map(|e| (e.shape.clone(), e.sha256.clone()))
        };
        let mut expected = committee2::OWN_TENSORS.len();
        // slots a0 to b3: the same slots of the root of the first committee spec
        for t in first
            .tensors
            .iter()
            .filter(|t| committee::SLOT_PREFIX.iter().any(|p| t.name.starts_with(p)))
        {
            expected += 1;
            assert_eq!(
                entry(&root, &t.name),
                Some((t.shape.clone(), t.sha256.clone())),
                "{dir}: {} differs from {}",
                t.name,
                first.id
            );
        }
        // slots c0 to c3: the root of the hinge-law spec with the burner-heating law
        for prefix in &committee2::SLOT_PREFIX[6..] {
            for t in &hybrid.tensors {
                expected += 1;
                assert_eq!(
                    entry(&root, &format!("{prefix}{}", t.name)),
                    Some((t.shape.clone(), t.sha256.clone())),
                    "{dir}: {prefix}{} differs from {}",
                    t.name,
                    hybrid.id
                );
            }
        }
        assert_eq!(root.tensors.len(), expected);
        let c = root.construction.as_ref().expect("a literal release");
        assert_eq!(
            c.forward_hash,
            audit::forward_hash_of(&root.model_name),
            "{dir}: re-assemble (tools/wsp/reassemble.py)"
        );
        assert!(c.checks.iter().all(|k| k.status == CheckStatus::Passed));

        // its own tensors: the weights of that root in both eras, no era told apart, the planned
        // stop not read; and the committed edits are the committed release
        let s = sources(family, dir);
        let from_edits = release(
            &key(family, 7),
            &config(&format!("wsp7/{dir}/baseline-set/edits.wsp")),
        );
        let weights = &from_edits
            .release
            .construction
            .as_ref()
            .unwrap()
            .weights_hash;
        assert_eq!(weights, &c.weights_hash);
        let rebuilt = wsp7(family, &s.w5, &s.c, &Own::root());
        assert!(from_edits.params == rebuilt.params);
        let p = &from_edits.params;
        for era in ["mix.w_early", "mix.w_late"] {
            assert_eq!(p[era].data[..6], s.w5.params["mix.w"].data[..]);
            assert_eq!(p[era].data[6..], [0.0; 4]);
        }
        for era in ["mix.ws_early", "mix.ws_late"] {
            assert_eq!(p[era].data[..6], s.w5.params["mix.w_script"].data[..]);
            assert_eq!(p[era].data[6..], [0.0; 4]);
        }
        // so the root answers as the root of the first committee spec
        let fixture = acme_tools::default_fixture(family);
        for prefix in [fixture.clone(), with_stop(&fixture, 51.0)] {
            let f5 = committee::forecast(&committee::spec(family), &s.w5, &prefix, &settings())
                .unwrap()
                .expect("admitted");
            assert_eq!(run7(family, &from_edits, &prefix).table, f5.table);
        }
    }
}
