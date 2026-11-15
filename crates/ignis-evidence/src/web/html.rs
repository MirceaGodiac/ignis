//! HTML → plain text for a snapshot, plus the head metadata a citation needs (title, canonical URL,
//! declared licence). A small tag scanner, not a browser: scripts, styles and templates are dropped,
//! block elements become line breaks, character references are decoded. The text is data for the
//! reader, never instructions.

/// What an HTML page yields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Extracted {
    pub text: String,
    pub title: Option<String>,
    pub canonical: Option<String>,
    /// `<link rel="license" href>` or a licence/rights/copyright `<meta>`, as declared.
    pub license: Option<String>,
}

const SKIP: &[&str] = &["script", "style", "noscript", "template", "svg", "head"];
const BLOCK: &[&str] = &[
    "p",
    "div",
    "br",
    "li",
    "ul",
    "ol",
    "tr",
    "table",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "section",
    "article",
    "header",
    "footer",
    "pre",
    "blockquote",
    "dd",
    "dt",
    "hr",
    "td",
    "th",
    "main",
    "nav",
    "figure",
    "figcaption",
];
const LICENSE_META: &[&str] = &[
    "dcterms.license",
    "dc.rights",
    "dcterms.rights",
    "citation_license",
    "license",
    "copyright",
    "rights",
];

/// True when the content type (or, without one, the first bytes) says HTML.
pub fn is_html(content_type: Option<&str>, bytes: &[u8]) -> bool {
    match content_type {
        Some(ct) => {
            let ct = ct.to_ascii_lowercase();
            ct.starts_with("text/html") || ct.starts_with("application/xhtml")
        }
        None => {
            let head = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]).to_ascii_lowercase();
            let head = head.trim_start();
            head.starts_with("<!doctype html") || head.starts_with("<html")
        }
    }
}

/// Attribute `name` of a tag body (`a href="x" rel=y`), unquoted or quoted.
fn attr(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let mut from = 0;
    while let Some(i) = lower[from..].find(name) {
        let at = from + i;
        from = at + name.len();
        let before_ok = at == 0 || lower.as_bytes()[at - 1].is_ascii_whitespace();
        let rest = lower[from..].trim_start();
        if !before_ok || !rest.starts_with('=') {
            continue;
        }
        let vstart = tag.len() - rest.len() + 1;
        let v = tag[vstart..].trim_start();
        let value = match v.chars().next() {
            Some(q @ ('"' | '\'')) => v[1..].split(q).next().unwrap_or_default(),
            _ => v
                .split(|c: char| c.is_whitespace() || c == '>')
                .next()
                .unwrap_or_default(),
        };
        return Some(decode_entities(value).trim().to_string());
    }
    None
}

fn tag_name(tag: &str) -> String {
    tag.trim_start_matches('/')
        .split(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

pub fn extract(html: &str) -> Extracted {
    let mut out = Extracted::default();
    let mut text = String::new();
    let mut skip: Option<String> = None;
    let mut in_title = false;
    let mut title = String::new();
    let mut rest = html;
    while !rest.is_empty() {
        let Some(lt) = rest.find('<') else {
            if skip.is_none() {
                text.push_str(&decode_entities(rest));
            }
            break;
        };
        let chunk = &rest[..lt];
        if in_title {
            title.push_str(chunk);
        } else if skip.is_none() {
            text.push_str(&decode_entities(chunk));
        }
        rest = &rest[lt..];
        if rest.starts_with("<!--") {
            rest = rest.find("-->").map_or("", |i| &rest[i + 3..]);
            continue;
        }
        let Some(gt) = rest.find('>') else { break };
        let tag = &rest[1..gt];
        rest = &rest[gt + 1..];
        let name = tag_name(tag);
        let closing = tag.starts_with('/');
        if let Some(s) = skip.clone() {
            if closing && name == s {
                skip = None;
            }
            // head metadata is read even while its text is skipped
            if s != "head" {
                continue;
            }
        }
        match (name.as_str(), closing) {
            ("title", false) => in_title = true,
            ("title", true) => in_title = false,
            ("link", false) => {
                let rel = attr(tag, "rel").unwrap_or_default().to_ascii_lowercase();
                let rels: Vec<&str> = rel.split_whitespace().collect();
                if rels.contains(&"canonical") {
                    out.canonical = out.canonical.take().or_else(|| attr(tag, "href"));
                }
                if rels.contains(&"license") {
                    out.license = out.license.take().or_else(|| attr(tag, "href"));
                }
            }
            ("meta", false) => {
                let key = attr(tag, "name")
                    .or_else(|| attr(tag, "property"))
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                if LICENSE_META.contains(&key.as_str()) && out.license.is_none() {
                    out.license = attr(tag, "content").filter(|c| !c.is_empty());
                }
            }
            (n, false) if SKIP.contains(&n) && skip.is_none() && !tag.ends_with('/') => {
                skip = Some(n.to_string());
            }
            (n, _) if BLOCK.contains(&n) => text.push('\n'),
            _ => {}
        }
    }
    let t = decode_entities(&title)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    out.title = (!t.is_empty()).then_some(t);
    // collapse runs of spaces per line, keep at most one blank line
    let mut lines: Vec<String> = Vec::new();
    for l in text.lines() {
        let l = l.split_whitespace().collect::<Vec<_>>().join(" ");
        if l.is_empty() && lines.last().is_none_or(|p| p.is_empty()) {
            continue;
        }
        lines.push(l);
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    out.text = lines.join("\n");
    out
}

pub fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        let end = rest[1..].find(';').map(|j| j + 1).filter(|&j| j <= 12);
        let Some(end) = end else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let ent = &rest[1..end];
        let ch = match ent {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" | "#39" => Some('\''),
            "nbsp" => Some(' '),
            "copy" => Some('©'),
            "ndash" => Some('–'),
            "mdash" => Some('—'),
            e if e.starts_with("#x") || e.starts_with("#X") => u32::from_str_radix(&e[2..], 16)
                .ok()
                .and_then(char::from_u32),
            e if e.starts_with('#') => e[1..].parse().ok().and_then(char::from_u32),
            _ => None,
        };
        match ch {
            Some(c) => {
                out.push(c);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}
