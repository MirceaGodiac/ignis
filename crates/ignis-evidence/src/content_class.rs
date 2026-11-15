//! What a chunk is, structurally: the body of the report, its reference list, its front-matter
//! lists (table of contents, list of figures/tables), an acronym/glossary row, or a fragment too
//! short to carry a finding (a heading, a bullet, a short caption). Deterministic, derived from
//! the chunk's own section heading, kind and text, so it is computed at import and recomputed for
//! stored catalogs by `ignis evidence reclassify` without parsing a PDF again.
//!
//! Why: a page of references is dense with topic words ("oxygen concentration", "flammability
//! limit", "microgravity"), and a row "LFL | Lower Flammability Limit" is nothing but query words;
//! FTS5's BM25 ranks both above the passage that reports the finding. Search ranks by [`tier`]:
//! body first, then glossary rows and fragments, then references and contents pages (all stay
//! reachable on later pages) unless the query asks for that class ([`ranked_with_body`]); reading
//! a page still returns its full text.
//!
//! Signals, in order:
//! 1. the section heading the chunk sits under (Docling section headers; the PDFium heading
//!    detector also emits `REFERENCES`/`BIBLIOGRAPHY`): References, Bibliography, Literature Cited,
//!    Works Cited, Citations, ... -> `references`; Contents, Table of Contents, List of Figures,
//!    List of Tables -> `contents`. A long prose chunk with no citation marks under such a heading
//!    (a missed heading after the list) stays `body`.
//! 2. the text alone (PDFium blocks often carry no heading): the density of citation marks per
//!    word (author initials, years, `et al.`, `pp.`, volume/issue, journal and report
//!    abbreviations, DOIs, numbered entries) over several entry-like lines; dot leaders ending in
//!    page numbers for contents.
//!
//! Normal prose that cites ("as Williams et al. (1985) [12] showed") carries one or two marks in
//! dozens of words and stays `body`.

use std::sync::OnceLock;

use regex::Regex;

pub const BODY: &str = "body";
pub const REFERENCES: &str = "references";
pub const CONTENTS: &str = "contents";
pub const GLOSSARY: &str = "glossary";
pub const FRAGMENT: &str = "fragment";

/// Rank tier of a class (lower first): the body; then glossary rows and fragments; then reference
/// lists and contents pages. A class in `with_body` (the query asks for it) ranks with the body.
pub fn tier(class: &str, with_body: &[String]) -> i64 {
    if class == BODY || with_body.iter().any(|c| c == class) {
        return 0;
    }
    match class {
        GLOSSARY | FRAGMENT => 1,
        REFERENCES | CONTENTS => 2,
        _ => 0,
    }
}

/// [`tier`] as an SQL expression over the class column `col`.
pub fn tier_sql(col: &str, with_body: &[String]) -> String {
    let arms: String = [GLOSSARY, FRAGMENT, REFERENCES, CONTENTS]
        .iter()
        .map(|c| format!(" WHEN '{c}' THEN {}", tier(c, with_body)))
        .collect();
    format!("(CASE {col}{arms} ELSE 0 END)")
}

fn rx(cell: &'static OnceLock<Regex>, pat: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pat).expect("content_class regex"))
}

/// Heading text without numbering ("7.", "VII.", "Appendix B:"), lower-case, single-spaced.
fn heading_key(section: &str) -> String {
    static NUM: OnceLock<Regex> = OnceLock::new();
    let s = section.trim().to_lowercase();
    let s = rx(
        &NUM,
        r"^(?:(?:appendix|annex|section|chapter)\s+[a-z0-9]{1,4}\b[\s.:\-–—]*|(?:\d+(?:\.\d+)*|[ivxlc]+)\s*[.):\-–—]?\s+)",
    )
    .replace(&s, "");
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(|c: char| c == ':' || c == '.' || c == ' ')
        .to_string()
}

fn heading_class(section: &str) -> Option<&'static str> {
    let k = heading_key(section);
    const REF: [&str; 14] = [
        "references",
        "reference",
        "reference list",
        "list of references",
        "references cited",
        "cited references",
        "references and notes",
        "bibliography",
        "selected bibliography",
        "literature cited",
        "literature",
        "works cited",
        "citations",
        "sources cited",
    ];
    const TOC: [&str; 5] = [
        "contents",
        "table of contents",
        "list of figures",
        "list of tables",
        "list of illustrations",
    ];
    const GLOSS: [&str; 14] = [
        "acronyms",
        "abbreviations",
        "acronyms and abbreviations",
        "abbreviations and acronyms",
        "acronyms and definitions",
        "list of acronyms",
        "list of abbreviations",
        "list of acronyms and abbreviations",
        "glossary",
        "glossary of terms",
        "definitions",
        "nomenclature",
        "symbols",
        "list of symbols",
    ];
    if REF.contains(&k.as_str()) {
        Some(REFERENCES)
    } else if TOC.contains(&k.as_str()) {
        Some(CONTENTS)
    } else if GLOSS.contains(&k.as_str()) {
        Some(GLOSSARY)
    } else {
        None
    }
}

/// Citation marks in `text` (each regex counts its matches).
fn citation_marks(text: &str) -> usize {
    static YEAR: OnceLock<Regex> = OnceLock::new();
    static ETAL: OnceLock<Regex> = OnceLock::new();
    static INIT: OnceLock<Regex> = OnceLock::new();
    static PAGES: OnceLock<Regex> = OnceLock::new();
    static VOL: OnceLock<Regex> = OnceLock::new();
    static VENUE: OnceLock<Regex> = OnceLock::new();
    static DOI: OnceLock<Regex> = OnceLock::new();
    let pats: [(&'static OnceLock<Regex>, &str); 7] = [
        (&YEAR, r"\(?\b(?:19|20)\d{2}[a-z]?\b\)?"),
        (&ETAL, r"\bet\s+al\b"),
        // "Williams, F. A.", "F. A. Williams", "Chen, C.-H."
        (
            &INIT,
            r"\b[A-Z][a-z'\-]+,\s+(?:[A-Z]\.[\s\-]*){1,3}|\b(?:[A-Z]\.\s?){1,3}[A-Z][a-z'\-]{2,}",
        ),
        (&PAGES, r"(?i)\bpp?\.\s*\d+|\b\d{1,5}\s*[-–]\s*\d{1,5}\b"),
        (&VOL, r"(?i)\b(?:vol|no)\.\s*\d+|\b\d+\s*\(\d+\)\s*[:,]"),
        (
            &VENUE,
            r"(?i)\b(?:j|proc|symp|conf|trans|combust|sci|technol|univ|int|inst)\.|\b(?:journal|proceedings|symposium|conference|aiaa|asme|nasa\s+(?:tm|cr|tp|technical)|paper|press|university|report)\b",
        ),
        (&DOI, r"(?i)\bdoi\b|\b10\.\d{4,9}/\S+|https?://\S+"),
    ];
    pats.iter()
        .map(|(c, p)| rx(c, p).find_iter(text).count())
        .sum()
}

/// Lines that open like a reference entry: "[12]", "12.", "12)" followed by a capitalised name,
/// or "Surname, I." at the start.
fn entry_lines(text: &str) -> usize {
    static E: OnceLock<Regex> = OnceLock::new();
    let e = rx(
        &E,
        r"^\s*(?:\[\d{1,3}\]|\d{1,3}[.)])\s+[A-Z][A-Za-z'\-]+,?\s+(?:[A-Z]\.|[A-Z][a-z]+,\s+[A-Z]\.)|^\s*[A-Z][A-Za-z'\-]+,\s+(?:[A-Z]\.[\s\-]*){1,3}",
    );
    text.lines().filter(|l| e.is_match(l)).count()
}

fn words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Text that reads as a reference list on its own.
fn looks_like_references(text: &str) -> bool {
    static YEAR: OnceLock<Regex> = OnceLock::new();
    let w = words(text);
    if w < 8 {
        return false;
    }
    // timestamps and dates (instrument logs, data listings) are not publication years
    static STAMP: OnceLock<Regex> = OnceLock::new();
    if rx(
        &STAMP,
        r"\b(?:19|20)\d{2}-\d{2}-\d{2}\b|\b\d{2}:\d{2}:\d{2}",
    )
    .find_iter(text)
    .count()
        >= 2
    {
        return false;
    }
    static AUTHOR: OnceLock<Regex> = OnceLock::new();
    let authors = rx(
        &AUTHOR,
        r"\b[A-Z][a-z'\-]+,\s+(?:[A-Z]\.[\s\-]*){1,3}|\b(?:[A-Z]\.\s?){1,3}[A-Z][a-z'\-]{2,}",
    )
    .find_iter(text)
    .count();
    if authors == 0 {
        return false;
    }
    let years = rx(&YEAR, r"\b(?:19|20)\d{2}[a-z]?\b")
        .find_iter(text)
        .count();
    let marks = citation_marks(text);
    let entries = entry_lines(text);
    let density = marks as f64 / w as f64;
    // one entry (a short block of its own): numbered or "Surname, I." start, a year, and dense marks
    if w <= 60 {
        return entries >= 1 && years >= 1 && marks >= 4 && density >= 0.2;
    }
    // a list: several entries or years, and marks on a large share of the words
    (entries >= 3 || years >= 4) && years >= 2 && density >= 0.15
}

fn looks_like_contents(text: &str) -> bool {
    static LEADER: OnceLock<Regex> = OnceLock::new();
    static TITLE: OnceLock<Regex> = OnceLock::new();
    let lead = rx(
        &LEADER,
        r"(?:\.\s?){4,}\s*[ivxlc\d]{1,4}\s*$|\s{3,}\d{1,4}\s*$",
    );
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let leaders = lines.iter().filter(|l| lead.is_match(l)).count();
    if leaders >= 3 && leaders * 2 >= lines.len() {
        return true;
    }
    // a run-on contents block (one line): many dot leaders
    let dots = rx(&TITLE, r"(?:\.\s?){5,}\s*[ivxlc\d]{1,4}\b")
        .find_iter(text)
        .count();
    dots >= 4 && dots * 12 >= words(text)
}

/// Every line is an acronym row: "LFL | Lower Flammability Limit", "UFL Upper Flammability
/// Limit", "ISS - International Space Station" (an upper-case token, then a short expansion
/// without numbers).
fn looks_like_glossary(text: &str) -> bool {
    static ROW: OnceLock<Regex> = OnceLock::new();
    let row = rx(
        &ROW,
        r"^\s*[A-Z][A-Z0-9/&\-]{1,9}s?\s*(?:\||\t|[-–—=:]|\s)\s*[A-Za-z][A-Za-z\-'/(),.\s]{2,80}$",
    );
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    // the expansion starts with the acronym's first letter and is not itself upper-case
    // ("ID Inner diameter"; not "ZIN Technologies", "SECTION A-A", "POST-TEST CONDITIONS")
    let expands = |l: &str| {
        let l = l.trim();
        let Some((acr, rest)) = l.split_once(|c: char| c == '|' || c == '\t' || c == ' ') else {
            return false;
        };
        let rest =
            rest.trim_start_matches(|c: char| c == '|' || c.is_whitespace() || "-–—=:".contains(c));
        acr.chars().next().map(|c| c.to_ascii_lowercase())
            == rest.chars().next().map(|c| c.to_ascii_lowercase())
            && rest.chars().any(|c| c.is_lowercase())
    };
    !lines.is_empty()
        && lines.iter().all(|l| {
            row.is_match(l)
                && expands(l)
                && !l.chars().any(|c| c.is_ascii_digit())
                && words(l) <= 10
        })
}

/// Too little text to carry a finding on its own: under 120 characters or 15 words. A table row
/// with two or more numeric cells is data, not a fragment.
fn is_fragment(kind: &str, text: &str) -> bool {
    let t = text.trim();
    let short = t.chars().count() < 120 || words(t) < 15;
    if !short {
        return false;
    }
    if kind.starts_with("table") {
        let numeric = t
            .split('|')
            .filter(|c| c.chars().any(|x| x.is_ascii_digit()))
            .count();
        return numeric < 2;
    }
    true
}

/// Class of a chunk from its section heading, kind and text: [`BODY`], [`REFERENCES`],
/// [`CONTENTS`], [`GLOSSARY`] or [`FRAGMENT`].
pub fn classify(section: Option<&str>, kind: &str, text: &str) -> &'static str {
    // a heading chunk names its own class ("References" opens the list); other headings are
    // fragments (a title is not a finding)
    if kind == "heading" {
        return heading_class(text).unwrap_or(FRAGMENT);
    }
    if let Some(c) = section.and_then(heading_class) {
        // a missed heading after the list: long prose without citation marks is body
        let w = words(text);
        let marks = citation_marks(text);
        let prose = w >= 60 && (marks as f64) < 0.05 * w as f64 && entry_lines(text) == 0;
        if c != REFERENCES && looks_like_references(text) {
            return REFERENCES;
        }
        return if prose { BODY } else { c };
    }
    if looks_like_glossary(text) {
        return GLOSSARY;
    }
    let structural = kind.starts_with("table") || kind == "caption" || kind == "figure";
    if !structural && looks_like_references(text) {
        REFERENCES
    } else if !structural && looks_like_contents(text) {
        CONTENTS
    } else if is_fragment(kind, text) {
        FRAGMENT
    } else {
        BODY
    }
}

/// The classes a query asks for, which then rank with the body: reference lists and contents
/// pages for a query about references or citations; glossary rows and fragments for a query about
/// acronyms, abbreviations or definitions.
pub fn ranked_with_body(query: &str) -> Vec<String> {
    static REF: OnceLock<Regex> = OnceLock::new();
    static GLOSS: OnceLock<Regex> = OnceLock::new();
    let mut out = Vec::new();
    if rx(
        &REF,
        r"(?i)\b(?:references|reference\s+lists?|referenced|bibliograph\w*|cited|cites?|citations?|citing|table\s+of\s+contents|list\s+of\s+(?:figures|tables))\b",
    )
    .is_match(query)
    {
        out.extend([REFERENCES.to_string(), CONTENTS.to_string()]);
    }
    if rx(
        &GLOSS,
        r"(?i)\b(?:acronyms?|abbreviat\w*|glossary|nomenclature|stands?\s+for|definitions?|define[sd]?|meaning\s+of)\b",
    )
    .is_match(query)
    {
        out.extend([GLOSSARY.to_string(), FRAGMENT.to_string()]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFS_BLOCK: &str = "1. Williams, F. A., Combustion Theory, 2nd ed., Benjamin/Cummings, Menlo Park, CA, 1985.\n\
2. Nayagam, V., Dietrich, D. L., Ferkul, P. V., Hicks, M. C., and Williams, F. A., \"Can cool flames support quasi-steady alkane droplet burning?\" Combust. Flame 159 (2012) 3583-3588.\n\
3. Ohtani, H., Hirano, T., Akita, K., Flammability limits of oxygen concentration for downward flame spread, Proc. Combust. Inst. 19 (1982) pp. 611-618.\n\
4. Olson, S. L., Ferkul, P. V., and T'ien, J. S., Near-limit flame spread over a thin solid fuel in microgravity, Proc. Combust. Inst. 22 (1988) 1213-1222.\n\
5. Dietrich, D. L., et al., Droplet combustion experiments aboard the International Space Station, Microgravity Sci. Technol. 26 (2014) 65-76, doi:10.1007/s12217-014-9372-2.";

    const AUTHOR_YEAR: &str = "Fujita, O., 2015. Solid combustion research in microgravity as a basis of fire safety in space. Proc. Combust. Inst. 35, 2487-2502.\n\
Olson, S.L., Miller, F.J., 2009. Experimental comparison of opposed and concurrent flame spread in a forced convective microgravity environment. Proc. Combust. Inst. 32, 2445-2452.\n\
T'ien, J.S., Shih, H.Y., Jiang, C.B., Ross, H.D., Miller, F.J., Fernandez-Pello, A.C., Torero, J.L., Walther, D., 2001. Mechanisms of flame spread and smolder wave propagation. In: Ross, H.D. (Ed.), Microgravity Combustion: Fire in Free Fall. Academic Press, pp. 299-418.";

    const PROSE: &str = "The limiting oxygen concentration for flame spread over the thin cellulose sample decreased \
from 21 % to 17 % as the opposed flow velocity increased from 2 to 10 cm/s, in agreement with the trend reported \
by Olson et al. (1988) [4]. Below 17 % oxygen the flame shrank to a small, dim leading edge and extinguished within \
about 30 s. The flammability boundary is therefore set by radiative loss at low flow and by finite-rate chemistry at \
high flow, which explains the U-shaped flammability map shown in Figure 6.";

    #[test]
    fn numbered_reference_list_is_references() {
        assert_eq!(classify(None, "text", REFS_BLOCK), REFERENCES);
    }

    #[test]
    fn author_year_reference_list_is_references() {
        assert_eq!(classify(None, "text", AUTHOR_YEAR), REFERENCES);
    }

    #[test]
    fn a_single_entry_block_is_references() {
        let one = "12. Ferkul, P. V., Olson, S. L., Zero-gravity centerline profiles of PMMA flames, Combust. Sci. Technol. 182 (2010) 1-18.";
        assert_eq!(classify(None, "text", one), REFERENCES);
    }

    #[test]
    fn prose_that_cites_stays_body() {
        assert_eq!(classify(None, "text", PROSE), BODY);
        let results = "3.2 Results. Burning rate constants of 0.62 mm2/s (1 atm, 21 % O2) and 0.48 mm2/s \
(0.5 atm) were measured for n-heptane droplets in 2012 and 2013, consistent with Nayagam et al. [2] and \
Dietrich et al. [5]; see Table 2 and pp. 4-5 of the test report.";
        assert_eq!(classify(None, "text", results), BODY);
        // a section heading with a number and a year
        assert_ne!(
            classify(None, "text", "3. Results of the 2012 FLEX-2 tests"),
            REFERENCES
        );
        // an instrument log is dense with dates and number ranges, not references
        let log = "2021-03-04 063 02:58:38:909~acme.ipsu~Success for IPSU 5 command IpsugCamerasStopTestPointArchive \
2021-03-04 063 02:58:39:831~acme.ipsu~Success for IPSU 5 command IpsugTestpointClose 2021-03-04 063 \
02:58:43:780~zt.script~Homing fuel 2021-03-04 063 02:58:44:100~acme.ipsu~Success for IPSU 5 command \
IpsugTestpointOpen 2021-03-04 063 02:58:45:000~zt.fcf~ PRIORITY INVERSION 2021-03-04 063 02:58:46:000 done";
        assert_ne!(classify(None, "text", log), REFERENCES);
    }

    #[test]
    fn the_section_heading_decides_for_list_items() {
        let item = "Hirano, T. and Saito, K., Fire spread phenomena: the role of observation in experiment.";
        assert_eq!(classify(Some("REFERENCES"), "text", item), REFERENCES);
        assert_eq!(classify(Some("7. References"), "text", item), REFERENCES);
        assert_eq!(classify(Some("Bibliography:"), "text", item), REFERENCES);
        assert_eq!(classify(Some("Literature Cited"), "text", item), REFERENCES);
        assert_eq!(
            classify(Some("References"), "heading", "References"),
            REFERENCES
        );
        assert_ne!(classify(Some("Results"), "text", item), REFERENCES);
        // a long paragraph without citation marks under a stale heading is body
        let para = "After the reference list the report continues with the operations log of the facility. \
The crew installed the chamber insert, checked the igniter wires, purged the chamber with the test gas and \
waited for the fan to settle before each test point. The cameras were focused on the sample holder and the \
radiometer was zeroed; the procedure was repeated for every sample card of the series without exception.";
        assert_eq!(classify(Some("References"), "text", para), BODY);
    }

    #[test]
    fn contents_pages_are_contents() {
        let toc = "1. Introduction .......... 1\n2. Apparatus .......... 3\n2.1 Combustion chamber .......... 4\n3. Results .......... 9\n4. Conclusions .......... 21";
        assert_eq!(classify(None, "text", toc), CONTENTS);
        assert_eq!(
            classify(Some("Table of Contents"), "text", "Flame spread over PMMA"),
            CONTENTS
        );
        assert_eq!(
            classify(
                Some("List of Figures"),
                "text",
                "Figure 3. Flame shape at 16 % O2"
            ),
            CONTENTS
        );
        assert_eq!(classify(Some("Contents"), "heading", "Contents"), CONTENTS);
    }

    #[test]
    fn data_table_rows_are_body() {
        let row = "Test 12 | 2012 | 21 % | 1.0 atm | 0.62 | 1-18";
        assert_eq!(classify(None, "table_row", row), BODY);
    }

    #[test]
    fn acronym_rows_and_short_pieces_are_not_body() {
        // Docling table rows under "1.2 ACRONYMS AND ABBREVIATIONS", PDFium lines without a heading
        let s = Some("1.2 ACRONYMS AND ABBREVIATIONS");
        assert_eq!(
            classify(s, "table_row", "LFL | Lower Flammability Limit"),
            GLOSSARY
        );
        assert_eq!(
            classify(None, "text", "UFL Upper Flammability Limit"),
            GLOSSARY
        );
        assert_eq!(
            classify(None, "table_row", "LFL | Lower Flammability Limit"),
            GLOSSARY
        );
        assert_eq!(
            classify(None, "text", "Enhanced flammability limits"),
            FRAGMENT
        );
        assert_eq!(
            classify(None, "heading", "4. Flammability Limits"),
            FRAGMENT
        );
        assert_eq!(
            classify(None, "caption", "Figure 6. Flammability map."),
            FRAGMENT
        );
        // a sentence that reports the finding is body
        let finding =
            "The flame extinguished below 17 % oxygen at 5 cm/s opposed flow, which sets \
the lower flammability limit of the thin cellulose sample in microgravity.";
        assert_eq!(classify(None, "text", finding), BODY);
    }

    #[test]
    fn queries_that_ask_for_references_or_acronyms() {
        let refs = |q: &str| ranked_with_body(q).contains(&REFERENCES.to_string());
        assert!(refs("references on oxygen flammability limit"));
        assert!(refs("which papers are cited for the burning rate"));
        assert!(refs("bibliography of FLEX"));
        assert!(ranked_with_body("oxygen concentration flammability limit").is_empty());
        assert!(!refs("reference temperature of the gas"));
        assert!(ranked_with_body("what does LFL stand for").contains(&GLOSSARY.to_string()));
        assert_eq!(tier(REFERENCES, &[]), 2);
        assert_eq!(tier(FRAGMENT, &[]), 1);
        assert_eq!(tier(REFERENCES, &ranked_with_body("cited papers")), 0);
    }
}
