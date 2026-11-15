//! Strictness of the WSP route: the crate that runs literal releases links no fitting, sampling or
//! random code, so no request can reach `newton_mode`, `local_inference`, L-BFGS, NUTS or a
//! least-squares routine, by fallback or otherwise.

use std::process::Command;

use ignis_core::contracts::{CheckStatus, Family};
use ignis_wsp::audit;

/// Crates that hold optimizers, samplers, random number generators or the fitted experts.
const FORBIDDEN: [&str; 15] = [
    "ignis-science",
    "ignis-flex",
    "ignis-acme",
    "ignis-bass",
    "ignis-train",
    "ignis-accel",
    "ignis-lab",
    "nuts-rs",
    "rand",
    "rand_distr",
    "rand_core",
    "faer",
    "argmin",
    "nalgebra",
    "ndarray",
];

#[test]
fn the_dependency_graph_of_the_strict_route_holds_no_optimizer() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = Command::new(cargo)
        .args([
            "tree",
            "-p",
            "ignis-wsp",
            "-e",
            "normal",
            "--prefix",
            "none",
            "--offline",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo tree");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let tree = String::from_utf8_lossy(&out.stdout);
    let names: Vec<&str> = tree
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .collect();
    assert!(
        names.contains(&"ignis-core"),
        "unexpected cargo tree output:\n{tree}"
    );
    for bad in FORBIDDEN {
        assert!(
            !names.contains(&bad),
            "ignis-wsp links {bad}: the strict forward must not be able to reach fitting code"
        );
    }
    let workspace: Vec<&&str> = names.iter().filter(|n| n.starts_with("ignis-")).collect();
    assert!(
        workspace
            .iter()
            .all(|n| ["ignis-wsp", "ignis-core"].contains(n)),
        "workspace crates linked by ignis-wsp: {workspace:?}"
    );
}

#[test]
fn forward_sources_name_no_fitting_routine_and_have_a_stable_identity() {
    for family in ignis_wsp::FAMILIES {
        assert_eq!(audit::banned_hits(family), Vec::<String>::new());
        assert_eq!(
            audit::strict_route_check(family).status,
            CheckStatus::Passed
        );
        assert_eq!(audit::forward_hash(family), audit::forward_hash(family));
    }
    // the family's own source is part of its identity
    assert!(audit::forward_sources(Family::Flex)
        .iter()
        .any(|(n, _)| *n == "flex.rs"));
    // each family's forward has its own identity; the two ACME configurations share one forward
    // (their specs, not their code, keep them apart)
    let h = audit::forward_hash;
    assert_ne!(h(Family::Flex), h(Family::AcmeNormal));
    assert_ne!(h(Family::Flex), h(Family::Bass));
    assert_eq!(h(Family::AcmeNormal), h(Family::AcmeInverse));
}
