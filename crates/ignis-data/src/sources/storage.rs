//! What the server stores from NASA's PSI archive and how the UI reaches the rest (R6).
//!
//! The rule is Iancu's atlas (`docs/wsp/atlas/SERVER_STORAGE.md`, from NASA's own listings of the 19
//! flight combustion investigations): documents and tables of the document categories, and the
//! sensor/settings text of Raw Data, are stored with their hash; images, video and archives stay at
//! NASA as manifest entries and are opened there (`AccessMode::Direct` through a signed URL resolved
//! by the backend, LANES_V2 §10). The split is by PSI category and file type, never by content, so
//! it is decided from a listing alone, before any byte is fetched.

use ignis_core::contracts::AccessMode;

/// One PSI flight combustion investigation (atlas table "By investigation").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flight {
    pub accession: &'static str,
    pub acronym: &'static str,
    /// Licence as stated in NASA's metadata on the atlas date; `None` = the field is blank and the
    /// licence must be verified before the investigation's documents are used (PSI-10).
    pub licence: Option<&'static str>,
    /// Documents hold outcomes of tests held out for model evaluation: every passage added from them
    /// goes through the lockbox (`strip_for_sealed`) for authoring scopes.
    pub lockbox: bool,
}

const fn f(accession: &'static str, acronym: &'static str, lockbox: bool) -> Flight {
    Flight {
        accession,
        acronym,
        licence: Some("CC0-1.0"),
        lockbox,
    }
}

/// The 19 flight investigations, in the atlas order.
pub const FLIGHTS: [Flight; 19] = [
    f("PSI-69", "FLEX", true),
    f("PSI-68", "FLEX-2", false),
    f("PSI-39", "CFI", false),
    f("PSI-159", "ACME CFI-G", true),
    Flight {
        accession: "PSI-10",
        acronym: "ACME/Flame Design",
        licence: None,
        lockbox: false,
    },
    f("PSI-23", "ACME s-Flame", false),
    f("PSI-20", "ACME BRE", false),
    f("PSI-21", "ACME CLD Flame", false),
    f("PSI-22", "ACME E-FIELD Flames", false),
    f("PSI-107", "SPICE", false),
    f("PSI-106", "SLICE", false),
    f("PSI-25", "BASS-II", true),
    f("PSI-26", "BASS", true),
    f("PSI-98", "SAFFIRE-I", false),
    f("PSI-99", "SAFFIRE-II", false),
    f("PSI-100", "SAFFIRE-III", false),
    f("PSI-102", "SAME", false),
    f("PSI-101", "SAME-R", false),
    f("PSI-47", "DAFT / DAFT-2", false),
];

pub fn flight(accession: &str) -> Option<&'static Flight> {
    FLIGHTS.iter().find(|f| f.accession == accession)
}

/// PSI categories whose files of a stored type are documents.
pub const DOCUMENT_CATEGORIES: [&str; 9] = [
    "Experimental table",
    "Science Documents",
    "Reports",
    "Presentations",
    "Engineering Documents",
    "Analyzed Data",
    "Reference Data",
    "Calibration",
    "Investigation Metadata Files",
];

/// File types the server stores (lower case, without the dot).
pub const STORED_TYPES: [&str; 15] = [
    "pdf", "doc", "docx", "ppt", "pptx", "xls", "xlsx", "csv", "txt", "rtf", "pxp", "log", "json",
    "xml", "md",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StorageClass {
    /// Stored and hashed: documents, tables, analysed data.
    Document,
    /// Stored and hashed: sensor and settings text of Raw Data (some investigations file documents
    /// here too; both groups are stored, so nothing is lost).
    SensorText,
    /// Not stored: images, video, archives. Name, size, version and address only; opened at NASA.
    ManifestOnly,
}

fn stored_type(file_name: &str) -> bool {
    file_name
        .rsplit_once('.')
        .is_some_and(|(_, ext)| STORED_TYPES.contains(&ext.to_ascii_lowercase().as_str()))
}

/// The atlas rule, from the PSI category and the file name alone.
pub fn storage_class(category: &str, file_name: &str) -> StorageClass {
    if !stored_type(file_name) {
        StorageClass::ManifestOnly
    } else if category == "Raw Data" {
        StorageClass::SensorText
    } else if DOCUMENT_CATEGORIES.contains(&category) {
        StorageClass::Document
    } else {
        StorageClass::ManifestOnly
    }
}

/// How a viewer opens a PSI file (LANES_V2 §10, measured 2026-11-14): PSI bytes are on S3 with
/// Range and CORS, reached through a signed URL the backend resolves at view time, so every PSI
/// file is `direct`, stored or not; the stored copy serves indexing, not delivery.
pub fn psi_access_mode() -> AccessMode {
    AccessMode::Direct
}

/// NTRS answers Range but sends no CORS headers and refuses framing: a pass-through proxy.
pub fn ntrs_access_mode() -> AccessMode {
    AccessMode::Proxy
}
