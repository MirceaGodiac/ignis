//! `skills_read` (skills on demand): the tool serves a skill's full text from the harness bundle
//! the episode is pinned to, the same bytes the pinned digest covers; an unknown name is refused
//! with the names that bundle has, and a bundle this host does not hold is never substituted.

mod common;

use common::*;
use ignis_core::contracts::*;
use ignis_research::runtime::{HarnessBundle, HarnessSource};
use serde_json::json;

fn episode(h: &Host, sc: &ExecutionScope, crid: &str) -> EpisodeId {
    h.store
        .create_episode(
            sc,
            "Synthetic fixture goal",
            &ClientRequestId::new(crid).unwrap(),
            EpisodePins::default(),
            Origin::SyntheticFixture,
        )
        .unwrap()
        .0
        .episode_id
}

fn pin(h: &Host, sc: &ExecutionScope, ep: &EpisodeId, b: &HarnessBundle) {
    let build = RuntimeBuildId::new("fixture-runtime-build").unwrap();
    h.store
        .pin_harness(sc, ep, b.pin(&build, Some("fixture-model"), None))
        .unwrap();
}

fn tools() -> Vec<String> {
    vec![
        "sources_search".into(),
        "answer_submit".into(),
        "skills_read".into(),
    ]
}

#[test]
fn skills_read_serves_the_pinned_bundle_and_refuses_unknown_names() {
    let h = host();
    assert!(h.host.registered().iter().any(|d| d.name == "skills_read"));
    let sc = scope("ws_skills");

    // Two bundles on the host; the episode is pinned to the older one.
    let h1 = HarnessBundle::embedded(&tools());
    let mut src = HarnessSource::embedded();
    let i = src
        .skills
        .iter()
        .position(|(n, _)| n == "source-grounding")
        .unwrap();
    src.skills[i].1.push_str("\nA step only variant h2 has.\n");
    let h2 = HarnessBundle::render("h2", &src, &tools()).unwrap();
    assert_eq!(h1.text(), h2.text(), "a skill edit leaves the prefix alone");
    assert_ne!(h1.digest(), h2.digest());
    h.host.harness().add(h1.clone());
    h.host.harness().add(h2.clone());
    let ep = episode(&h, &sc, "crid-skills-1");
    pin(&h, &sc, &ep, &h1);

    let r = h.host.call_in(
        &sc,
        Some(&ep),
        "skills_read",
        json!({"name": "source-grounding"}),
    );
    r.validate().unwrap();
    assert_eq!(r.execution_status, ExecutionStatus::Completed, "{r:#?}");
    let text: SkillText = serde_json::from_value(r.result.unwrap().value).unwrap();
    text.validate().unwrap();
    let want = h1.skill_text("source-grounding").unwrap();
    assert_eq!(text, want, "the pinned bundle's bytes");
    assert_eq!(&text.harness_digest, h1.digest());
    assert!(!text.body.contains("only variant h2 has"));

    // Unknown name: refused, listing the names of the pinned bundle.
    let r = h.host.call_in(
        &sc,
        Some(&ep),
        "skills_read",
        json!({"name": "no-such-skill"}),
    );
    r.validate().unwrap();
    assert_eq!(r.execution_status, ExecutionStatus::Refused, "{r:#?}");
    let e = r.error.unwrap();
    assert_eq!(e.code, ErrorCode::NotFound);
    for n in [
        "model-qualification",
        "quantitative-analysis",
        "source-grounding",
    ] {
        assert!(e.message.contains(n), "{}", e.message);
    }

    // Arguments are the contract: unknown fields are refused.
    let r = h.host.call_in(
        &sc,
        Some(&ep),
        "skills_read",
        json!({"name": "source-grounding", "workspace_id": "x"}),
    );
    assert_eq!(r.execution_status, ExecutionStatus::Refused, "{r:#?}");

    // An episode pinned to a bundle this host does not hold is refused, never served another.
    let mut other = HarnessSource::embedded();
    other
        .investigator
        .push_str("\nA rule of a bundle this host never rendered.\n");
    let gone = HarnessBundle::render("h0", &other, &tools()).unwrap();
    let ep2 = episode(&h, &sc, "crid-skills-2");
    pin(&h, &sc, &ep2, &gone);
    let r = h.host.call_in(
        &sc,
        Some(&ep2),
        "skills_read",
        json!({"name": "source-grounding"}),
    );
    r.validate().unwrap();
    assert_eq!(r.execution_status, ExecutionStatus::Refused, "{r:#?}");
    let e = r.error.unwrap();
    assert_eq!(e.code, ErrorCode::CapabilityUnavailable);
    assert!(e.message.contains(&gone.digest().hex()), "{}", e.message);
}
