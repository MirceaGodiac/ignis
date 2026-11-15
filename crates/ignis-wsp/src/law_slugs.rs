//! The table of the generic law forward's slugs (`law.rs`, models `<slug>.wsp`).
//!
//! Adding a slug is one line here plus its declaration file
//! `configs/models/law/<slug>/decl.json` (names, transforms, targets; no number). This file is not
//! a forward source: a slug's forward identity is `law.rs`, the shared kernels and its own
//! declaration, so adding a slug changes no other slug's identity. See
//! `docs/wsp/research/law_forward.md`.

/// One registered slug: its name, release `model_name`, and declaration (compiled in).
#[derive(Debug, PartialEq, Eq)]
pub struct Slug {
    pub slug: &'static str,
    pub model_name: &'static str,
    /// Repository path of the declaration, as named in the forward sources.
    pub decl_path: &'static str,
    pub decl: &'static str,
}

macro_rules! slug {
    ($s:literal) => {
        Slug {
            slug: $s,
            model_name: concat!($s, ".wsp"),
            decl_path: concat!("configs/models/law/", $s, "/decl.json"),
            decl: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../configs/models/law/",
                $s,
                "/decl.json"
            )),
        }
    };
}

pub const SLUGS: &[Slug] = &[
    // synthetic: the tests and the example of docs/wsp/research/law_forward.md (never NASA data)
    slug!("synthetic_demo"),
    slug!("sflame_event"),
    slug!("bre"),
    slug!("cld"),
    slug!("efield"),
    slug!("slice"),
    slug!("spice"),
    slug!("samer"),
    slug!("same"),
    slug!("daft"),
    slug!("bass1"),
];

/// Index of a slug by its model key (the slug itself).
pub fn by_key(key: &str) -> Option<usize> {
    SLUGS.iter().position(|s| s.slug == key)
}

/// Index of a slug by its release `model_name` (`<slug>.wsp`).
pub fn by_model_name(name: &str) -> Option<usize> {
    SLUGS.iter().position(|s| s.model_name == name)
}
