//! Sources: the NASA PSI connector (read-only), Source manifests, bounded sample selection and the
//! international adapters (kept as separate investigations).

pub mod adopt;
pub mod http;
pub mod intl;
pub mod manifest;
pub mod ntrs;
pub mod psi;
pub mod samples;
pub mod storage;

pub const INVESTIGATIONS: [&str; 4] = ["PSI-69", "PSI-159", "PSI-25", "PSI-26"];
