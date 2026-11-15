//! Rights of a web snapshot. A domain rule decides first; otherwise the page's own declaration is
//! recorded as declared (not verified); otherwise the status is `unverified` with no licence: the
//! explicit "unknown", never assumed free.

use ignis_core::contracts::{Rights, RightsStatus};

/// How a domain's documents are licensed.
#[derive(Debug, Clone, PartialEq)]
pub enum DomainRule {
    /// Every document of the domain carries this licence (e.g. U.S. Government works).
    Blanket {
        license: &'static str,
        status: RightsStatus,
    },
    /// Rights differ per document (a repository of mixed works): no domain default is applied,
    /// only what the document itself declares.
    PerDocument,
}

/// Domain rules, most specific first. A rule for `nasa.gov` covers its subdomains.
pub fn default_rules() -> Vec<(&'static str, DomainRule)> {
    vec![
        // NTRS holds NASA works next to copyrighted journal and conference papers.
        ("ntrs.nasa.gov", DomainRule::PerDocument),
        (
            "nasa.gov",
            DomainRule::Blanket {
                license:
                    "public domain: U.S. Government work (domain policy nasa.gov; third-party \
                          material on a page keeps its own rights)",
                status: RightsStatus::Verified,
            },
        ),
    ]
}

fn host_matches(host: &str, domain: &str) -> bool {
    host == domain || host.ends_with(&format!(".{domain}"))
}

/// The host of an http(s) URL, lowercased, without port.
pub fn host(url: &str) -> Option<String> {
    let rest = url.split_once("://")?.1;
    let auth = rest.split(['/', '?', '#']).next()?;
    let h = auth.rsplit_once('@').map_or(auth, |(_, h)| h);
    let h = if h.starts_with('[') {
        h.split(']').next().map(|x| x.trim_start_matches('['))?
    } else {
        h.split(':').next()?
    };
    Some(h.trim_end_matches('.').to_ascii_lowercase())
}

#[derive(Debug, Clone)]
pub struct RightsPolicy {
    rules: Vec<(&'static str, DomainRule)>,
}

impl Default for RightsPolicy {
    fn default() -> Self {
        RightsPolicy {
            rules: default_rules(),
        }
    }
}

impl RightsPolicy {
    /// Rights of the document at `final_url`, given what the page declared (`<link rel=license>`,
    /// licence/rights `<meta>`), if anything.
    pub fn rights(&self, final_url: &str, declared: Option<&str>) -> Rights {
        let host = host(final_url).unwrap_or_default();
        let rule = self
            .rules
            .iter()
            .find(|(d, _)| host_matches(&host, d))
            .map(|(_, r)| r);
        if let Some(DomainRule::Blanket { license, status }) = rule {
            return Rights {
                license: Some((*license).to_string()),
                status: *status,
                statement_locator: None,
            };
        }
        match declared.map(str::trim).filter(|d| !d.is_empty()) {
            Some(d) => {
                let lower = d.to_ascii_lowercase();
                // A declared non-commercial licence restricts use whatever else is true; anything
                // else declared by the page is recorded but stays unverified.
                let nc = lower.contains("by-nc") || lower.contains("noncommercial");
                Rights {
                    license: Some(format!("declared by the page: {d}")),
                    status: if nc {
                        RightsStatus::Noncommercial
                    } else {
                        RightsStatus::Unverified
                    },
                    statement_locator: None,
                }
            }
            None => Rights {
                license: None,
                status: RightsStatus::Unverified,
                statement_locator: None,
            },
        }
    }
}
