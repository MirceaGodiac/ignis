//! Identity and strictness of the forward code.
//!
//! * [`forward_hash`] digests the sources that turn a release and a prefix into numbers, with all
//!   whitespace removed: it is stable across identical builds, platforms, line endings and
//!   `cargo fmt`, changes with any other edit of a forward source, and is distinct from the weights,
//!   so an evaluator can tell "same weights, other forward" from "same model".
//! * [`strict_route_check`] is the build check recorded in every manifest. The strict route cannot
//!   reach an optimizer because this crate does not link one: its only workspace dependency is
//!   `ignis-core`. `tests/strict.rs` enforces that on the resolved dependency graph and scans these
//!   sources for calls into fitting code.

use sha2::{Digest as _, Sha256};

use ignis_core::contracts::{BuildCheck, BuildCheckKind, CheckStatus, Digest, Family};

/// Detail of a passed strict-forward audit (one text for every model, so release records do not
/// depend on which function wrote it).
const STRICT_OK: &str = "forward sources name no fitting, sampling or random routine; ignis-wsp links no optimizer crate (tests/strict.rs checks the dependency graph)";

/// Sources shared by every family's forward.
const SHARED: [(&str, &str); 4] = [
    ("nn.rs", include_str!("nn.rs")),
    ("quad.rs", include_str!("quad.rs")),
    ("common.rs", include_str!("common.rs")),
    ("spec.rs", include_str!("spec.rs")),
];

fn family_sources(family: Family) -> &'static [(&'static str, &'static str)] {
    match family {
        Family::Flex => &[("flex.rs", include_str!("flex.rs"))],
        Family::AcmeNormal | Family::AcmeInverse => &[("acme.rs", include_str!("acme.rs"))],
        Family::Bass => &[("bass.rs", include_str!("bass.rs"))],
        _ => &[],
    }
}

/// Every source file a family's forward is made of, by file name.
pub fn forward_sources(family: Family) -> Vec<(&'static str, &'static str)> {
    let mut v: Vec<_> = SHARED.to_vec();
    v.extend_from_slice(family_sources(family));
    v
}

/// Digest of the forward sources of a family: files in a fixed order with their names, every
/// whitespace character dropped, so formatting is not identity. Computed once per process and family.
pub fn forward_hash(family: Family) -> Digest {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<Vec<(Family, Digest)>>> =
        std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((_, d)) = guard.iter().find(|(f, _)| *f == family) {
        return d.clone();
    }
    let d = compute_forward_hash(family);
    guard.push((family, d.clone()));
    d
}

fn compute_forward_hash(family: Family) -> Digest {
    let mut h = Sha256::new();
    for (name, text) in forward_sources(family) {
        h.update(name.as_bytes());
        h.update([0u8]);
        let mut buf = [0u8; 4];
        for c in text.chars().filter(|c| !c.is_whitespace()) {
            h.update(c.encode_utf8(&mut buf).as_bytes());
        }
        h.update([0u8]);
    }
    Digest::parse(&format!("sha256:{}", hex::encode(h.finalize()))).expect("digest")
}

/// Forward sources of a literal model by its `model_name`. The first model of each family keeps the
/// family's source list (and so its identity); later models name their own files.
pub fn forward_sources_of(model_name: &str) -> Option<Vec<(&'static str, &'static str)>> {
    let family = match model_name {
        "flex.wsp" => Family::Flex,
        "acme_normal.wsp" => Family::AcmeNormal,
        "acme_inverse.wsp" => Family::AcmeInverse,
        "bass.wsp" => Family::Bass,
        "flex_methanol.wsp" | "flex_heptane.wsp" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("droplet.rs", include_str!("droplet.rs")));
            return Some(v);
        }
        "acme_normal.wsp2" | "acme_inverse.wsp2" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("flame.rs", include_str!("flame.rs")));
            return Some(v);
        }
        "flex_methanol.wsp2" | "flex_heptane.wsp2" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("droplet3.rs", include_str!("droplet3.rs")));
            return Some(v);
        }
        "bass_front.wsp" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("front.rs", include_str!("front.rs")));
            return Some(v);
        }
        "bass_front.wsp2" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("front2.rs", include_str!("front2.rs")));
            return Some(v);
        }
        "acme_normal.wsp3" | "acme_inverse.wsp3" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("flame3.rs", include_str!("flame3.rs")));
            return Some(v);
        }
        "flex_methanol.wsp3" | "flex_heptane.wsp3" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("droplet4.rs", include_str!("droplet4.rs")));
            return Some(v);
        }
        // a blend release is run by its own file and by both members' forwards
        "acme_normal.wsp4" | "acme_inverse.wsp4" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("flame.rs", include_str!("flame.rs")));
            v.push(("flame3.rs", include_str!("flame3.rs")));
            v.push(("blend.rs", include_str!("blend.rs")));
            return Some(v);
        }
        "flex_methanol.wsp4" | "flex_heptane.wsp4" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("droplet5.rs", include_str!("droplet5.rs")));
            return Some(v);
        }
        "flex_methanol.wsp5" | "flex_heptane.wsp5" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("droplet6.rs", include_str!("droplet6.rs")));
            return Some(v);
        }
        // a committee release is run by its own file and by both member forwards
        "acme_normal.wsp5" | "acme_inverse.wsp5" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("flame.rs", include_str!("flame.rs")));
            v.push(("flame3.rs", include_str!("flame3.rs")));
            v.push(("committee.rs", include_str!("committee.rs")));
            return Some(v);
        }
        "flex_methanol.wsp6" | "flex_heptane.wsp6" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("droplet7.rs", include_str!("droplet7.rs")));
            v.push(("thermal.rs", include_str!("thermal.rs")));
            return Some(v);
        }
        "bass_front.wsp3" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("front2.rs", include_str!("front2.rs")));
            v.push(("front3.rs", include_str!("front3.rs")));
            return Some(v);
        }
        "bass_front.wsp4" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("front2.rs", include_str!("front2.rs")));
            v.push(("front4.rs", include_str!("front4.rs")));
            return Some(v);
        }
        "acme_normal.wsp6" | "acme_inverse.wsp6" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("flame.rs", include_str!("flame.rs")));
            v.push(("flame3.rs", include_str!("flame3.rs")));
            v.push(("committee.rs", include_str!("committee.rs")));
            v.push((
                "condition_committee.rs",
                include_str!("condition_committee.rs"),
            ));
            return Some(v);
        }
        "acme_normal.wsp8" | "acme_inverse.wsp8" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("flame4.rs", include_str!("flame4.rs")));
            return Some(v);
        }
        // a release of the second committee is run by its own file and by the three member forwards
        "acme_normal.wsp7" | "acme_inverse.wsp7" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("flame.rs", include_str!("flame.rs")));
            v.push(("flame3.rs", include_str!("flame3.rs")));
            v.push(("flame4.rs", include_str!("flame4.rs")));
            v.push(("committee2.rs", include_str!("committee2.rs")));
            return Some(v);
        }
        // the three forwards of the fuel table: each its own file and the table's
        "acme_normal.wsp9" | "acme_inverse.wsp9" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("fuel.rs", include_str!("fuel.rs")));
            v.push(("flame5.rs", include_str!("flame5.rs")));
            return Some(v);
        }
        "acme_normal.wsp10" | "acme_inverse.wsp10" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("fuel.rs", include_str!("fuel.rs")));
            v.push(("flame6.rs", include_str!("flame6.rs")));
            return Some(v);
        }
        "acme_normal.wsp11" | "acme_inverse.wsp11" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("fuel.rs", include_str!("fuel.rs")));
            v.push(("flame7.rs", include_str!("flame7.rs")));
            return Some(v);
        }
        // the forwards of the second fuel table (phys_sflame_f3.md): each its own file and both tables'
        "acme_normal.wsp14" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("fuel.rs", include_str!("fuel.rs")));
            v.push(("fuel2.rs", include_str!("fuel2.rs")));
            v.push(("flame8.rs", include_str!("flame8.rs")));
            return Some(v);
        }
        "acme_normal.wsp15" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("fuel.rs", include_str!("fuel.rs")));
            v.push(("fuel2.rs", include_str!("fuel2.rs")));
            v.push(("flame9.rs", include_str!("flame9.rs")));
            return Some(v);
        }
        "acme_normal.wsp16" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("fuel.rs", include_str!("fuel.rs")));
            v.push(("fuel2.rs", include_str!("fuel2.rs")));
            v.push(("flame10.rs", include_str!("flame10.rs")));
            return Some(v);
        }
        // a release of the fourth committee is run by its own file and by the three member forwards
        "acme_normal.wsp13" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("fuel.rs", include_str!("fuel.rs")));
            v.push(("fuel2.rs", include_str!("fuel2.rs")));
            v.push(("flame8.rs", include_str!("flame8.rs")));
            v.push(("flame9.rs", include_str!("flame9.rs")));
            v.push(("flame10.rs", include_str!("flame10.rs")));
            v.push(("committee4.rs", include_str!("committee4.rs")));
            return Some(v);
        }
        // a release of the third committee is run by its own file and by the three member forwards
        "acme_normal.wsp12" | "acme_inverse.wsp12" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("fuel.rs", include_str!("fuel.rs")));
            v.push(("flame5.rs", include_str!("flame5.rs")));
            v.push(("flame6.rs", include_str!("flame6.rs")));
            v.push(("flame7.rs", include_str!("flame7.rs")));
            v.push(("committee3.rs", include_str!("committee3.rs")));
            return Some(v);
        }
        // bass_front.wsp5: the fifth front forward alone
        "bass_front.wsp5" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("front5.rs", include_str!("front5.rs")));
            return Some(v);
        }
        "flex_methanol.wsp7"
        | "flex_heptane.wsp7"
        | "flex_octane.wsp7"
        | "flex_isooctane.wsp7"
        | "flex_decane.wsp7"
        | "flex_dodecane.wsp7"
        | "flex_farnesane.wsp7"
        | "flex_ethanol.wsp7" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("droplet8.rs", include_str!("droplet8.rs")));
            return Some(v);
        }
        "flex_methanol.wsp8"
        | "flex_heptane.wsp8"
        | "flex_octane.wsp8"
        | "flex_isooctane.wsp8"
        | "flex_decane.wsp8"
        | "flex_dodecane.wsp8"
        | "flex_farnesane.wsp8"
        | "flex_ethanol.wsp8"
        | "flex_heptane_isooctane.wsp8"
        | "flex_dodecane_isododecane.wsp8"
        | "flex_propanol_glycerol.wsp8" => {
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("droplet9.rs", include_str!("droplet9.rs")));
            return Some(v);
        }
        // <slug>.wsp: the generic law forward and the slug's own declaration
        name if crate::law_slugs::by_model_name(name).is_some() => {
            let slug = &crate::law_slugs::SLUGS[crate::law_slugs::by_model_name(name)?];
            let mut v: Vec<_> = SHARED.to_vec();
            v.push(("law.rs", include_str!("law.rs")));
            v.push((slug.decl_path, slug.decl));
            return Some(v);
        }
        _ => return None,
    };
    Some(forward_sources(family))
}

/// Digest of the forward sources of a literal model (same rule as [`forward_hash`]).
///
/// # Panics
/// When `model_name` is not a literal model of this crate.
pub fn forward_hash_of(model_name: &str) -> Digest {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<Vec<(String, Digest)>>> =
        std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((_, d)) = guard.iter().find(|(n, _)| n == model_name) {
        return d.clone();
    }
    let sources = forward_sources_of(model_name)
        .unwrap_or_else(|| panic!("{model_name} is not a literal model of ignis-wsp"));
    let d = digest_sources(&sources);
    guard.push((model_name.to_string(), d.clone()));
    d
}

fn digest_sources(sources: &[(&str, &str)]) -> Digest {
    let mut h = Sha256::new();
    for (name, text) in sources {
        h.update(name.as_bytes());
        h.update([0u8]);
        let mut buf = [0u8; 4];
        for c in text.chars().filter(|c| !c.is_whitespace()) {
            h.update(c.encode_utf8(&mut buf).as_bytes());
        }
        h.update([0u8]);
    }
    Digest::parse(&format!("sha256:{}", hex::encode(h.finalize()))).expect("digest")
}

/// Banned identifiers in the forward sources of a literal model (see [`banned_hits`]).
pub fn banned_hits_of(model_name: &str) -> Vec<String> {
    let mut hits = Vec::new();
    for (name, text) in forward_sources_of(model_name).unwrap_or_default() {
        for (i, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or_default();
            for tok in BANNED_TOKENS {
                if code.contains(tok) {
                    hits.push(format!("{name}:{}: {tok}", i + 1));
                }
            }
        }
    }
    hits
}

/// The `strict_forward_audit` build check of a release of a literal model.
pub fn strict_route_check_of(model_name: &str) -> BuildCheck {
    let hits = banned_hits_of(model_name);
    BuildCheck {
        check: BuildCheckKind::StrictForwardAudit,
        status: if hits.is_empty() {
            CheckStatus::Passed
        } else {
            CheckStatus::Failed
        },
        detail: Some(if hits.is_empty() {
            STRICT_OK.into()
        } else {
            hits.join("; ")
        }),
    }
}

/// Identifiers of fitting, sampling and randomness that must not appear in forward sources.
pub const BANNED_TOKENS: [&str; 16] = [
    "newton_mode",
    "local_inference",
    "importance_locals",
    "prefix_d2_estimate",
    "init_locals",
    "lbfgs",
    "least_squares",
    "lstsq",
    "ignis_science",
    "ignis_flex",
    "ignis_acme",
    "ignis_bass",
    "run_nuts",
    "Adam",
    "rand::",
    "seed_from_u64",
];

/// Scans the forward sources of a family for banned identifiers outside comments. The list of this
/// module itself is not a forward source.
pub fn banned_hits(family: Family) -> Vec<String> {
    let mut hits = Vec::new();
    for (name, text) in forward_sources(family) {
        for (i, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or_default();
            for tok in BANNED_TOKENS {
                if code.contains(tok) {
                    hits.push(format!("{name}:{}: {tok}", i + 1));
                }
            }
        }
    }
    hits
}

/// The `strict_forward_audit` build check of a release of `family`.
pub fn strict_route_check(family: Family) -> BuildCheck {
    let hits = banned_hits(family);
    BuildCheck {
        check: BuildCheckKind::StrictForwardAudit,
        status: if hits.is_empty() {
            CheckStatus::Passed
        } else {
            CheckStatus::Failed
        },
        detail: Some(if hits.is_empty() {
            STRICT_OK.into()
        } else {
            hits.join("; ")
        }),
    }
}
