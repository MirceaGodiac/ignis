//! Minimal S3 client for Cloudflare R2: path-style requests signed with AWS Signature V4
//! (`region = auto`, `service = s3`), blocking over `ureq`. Only the calls the artifact store needs:
//! HEAD/GET (with Range), PUT (optionally `If-None-Match: *`), and multipart upload.
//!
//! Every PUT is signed over the SHA-256 of its body (`x-amz-content-sha256`), so R2 rejects bytes
//! that do not match the hash IGNIS computed while staging them.

use std::fs::File;
use std::io::Read;
use std::time::Duration;

use sha2::{Digest as _, Sha256};

/// `sha256("")`, the payload hash of bodiless requests.
pub const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

#[derive(Clone)]
pub struct S3Config {
    /// `https://<account>.r2.cloudflarestorage.com` (no bucket, no trailing slash).
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

impl std::fmt::Debug for S3Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("S3Config")
            .field("endpoint", &self.endpoint)
            .field("bucket", &self.bucket)
            .field("access_key_id", &self.access_key_id)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum S3Error {
    /// 404 / NoSuchKey.
    NotFound,
    /// 412: a conditional write found the key already present.
    Exists,
    /// Network failure or 5xx/429: may succeed later.
    Transient(String),
    /// Anything else (auth, bad request): retrying the same call will not help.
    Fatal(String),
}

impl std::fmt::Display for S3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S3Error::NotFound => write!(f, "not found"),
            S3Error::Exists => write!(f, "already exists"),
            S3Error::Transient(m) | S3Error::Fatal(m) => write!(f, "{m}"),
        }
    }
}

pub struct S3Client {
    cfg: S3Config,
    host: String,
    agent: ureq::Agent,
}

/// Body of a request: nothing, bytes, or a staged file (sized, so never chunked).
pub enum Payload<'a> {
    Empty,
    Bytes(&'a [u8]),
    File(&'a File),
}

impl S3Client {
    pub fn new(cfg: S3Config) -> Result<Self, String> {
        let host = cfg
            .endpoint
            .strip_prefix("https://")
            .or_else(|| cfg.endpoint.strip_prefix("http://"))
            .ok_or_else(|| format!("S3 endpoint must be an http(s) URL: {}", cfg.endpoint))?
            .trim_end_matches('/')
            .to_string();
        if host.is_empty() || host.contains('/') {
            return Err(format!(
                "S3 endpoint must be scheme://host[:port]: {}",
                cfg.endpoint
            ));
        }
        if cfg.bucket.is_empty() || cfg.bucket.contains('/') {
            return Err(format!("bad bucket name {:?}", cfg.bucket));
        }
        let agent = ureq::Agent::new_with_config(
            ureq::Agent::config_builder()
                .http_status_as_error(false)
                // Objects are stored as-is; never let a proxy re-encode a ranged read.
                .accept_encoding(ureq::config::AutoHeaderValue::None)
                .timeout_connect(Some(Duration::from_secs(10)))
                // Per read/write stall, not per transfer: large objects stream for as long as needed.
                .timeout_recv_body(Some(Duration::from_secs(120)))
                .timeout_send_body(Some(Duration::from_secs(120)))
                .build(),
        );
        Ok(S3Client { cfg, host, agent })
    }

    fn url(&self, key: &str, query: &[(&str, String)]) -> (String, String, String) {
        let path = format!("/{}/{}", self.cfg.bucket, uri_encode(key, false));
        let q = canonical_query(query);
        let scheme = if self.cfg.endpoint.starts_with("http://") {
            "http"
        } else {
            "https"
        };
        let url = if q.is_empty() {
            format!("{scheme}://{}{path}", self.host)
        } else {
            format!("{scheme}://{}{path}?{q}", self.host)
        };
        (url, path, q)
    }

    /// Send one signed request. `extra` headers are signed too.
    fn call(
        &self,
        method: &str,
        key: &str,
        query: &[(&str, String)],
        extra: &[(&str, String)],
        payload: Payload<'_>,
        payload_sha: &str,
    ) -> Result<ureq::http::Response<ureq::Body>, S3Error> {
        let (url, path, q) = self.url(key, query);
        let amz_date = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let mut headers: Vec<(String, String)> = vec![
            ("host".into(), self.host.clone()),
            ("x-amz-content-sha256".into(), payload_sha.to_string()),
            ("x-amz-date".into(), amz_date.clone()),
        ];
        headers.extend(
            extra
                .iter()
                .map(|(k, v)| (k.to_ascii_lowercase(), v.clone())),
        );
        let auth = sign_v4(&SigningInput {
            method,
            path: &path,
            query: &q,
            headers: &headers,
            payload_sha,
            amz_date: &amz_date,
            region: &self.cfg.region,
            service: "s3",
            access_key_id: &self.cfg.access_key_id,
            secret: &self.cfg.secret_access_key,
        });
        let mut req = ureq::http::Request::builder().method(method).uri(&url);
        for (k, v) in headers.iter().filter(|(k, _)| k != "host") {
            req = req.header(k, v);
        }
        req = req.header("authorization", auth);
        let res = match payload {
            // GET/HEAD carry no body; any other method gets an explicit `content-length: 0`
            // (an empty unsized body would go out chunked, which S3/R2 refuse).
            Payload::Empty if matches!(method, "GET" | "HEAD") => {
                self.agent.run(req.body(()).map_err(fatal)?)
            }
            Payload::Empty => self.agent.run(req.body(&[][..]).map_err(fatal)?),
            Payload::Bytes(b) => self.agent.run(req.body(b).map_err(fatal)?),
            Payload::File(f) => self.agent.run(req.body(f).map_err(fatal)?),
        };
        let resp = res.map_err(|e| S3Error::Transient(format!("{method} {key}: {e}")))?;
        let st = resp.status().as_u16();
        match st {
            200..=299 => Ok(resp),
            404 => Err(S3Error::NotFound),
            412 => Err(S3Error::Exists),
            429 | 500..=599 => Err(S3Error::Transient(format!("{method} {key}: HTTP {st}"))),
            _ => {
                let mut resp = resp;
                let body = resp
                    .body_mut()
                    .with_config()
                    .limit(16 << 10)
                    .read_to_string()
                    .unwrap_or_default();
                Err(S3Error::Fatal(format!("{method} {key}: HTTP {st} {body}")))
            }
        }
    }

    /// Size of the object, or `NotFound`.
    pub fn head(&self, key: &str) -> Result<u64, S3Error> {
        let r = self.call("HEAD", key, &[], &[], Payload::Empty, EMPTY_SHA256)?;
        r.headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .ok_or_else(|| S3Error::Fatal(format!("HEAD {key}: no content-length")))
    }

    /// The whole (small) object.
    pub fn get_bytes(&self, key: &str, limit: u64) -> Result<Vec<u8>, S3Error> {
        let mut r = self.call("GET", key, &[], &[], Payload::Empty, EMPTY_SHA256)?;
        r.body_mut()
            .with_config()
            .limit(limit)
            .read_to_vec()
            .map_err(|e| S3Error::Transient(format!("GET {key}: {e}")))
    }

    /// Stream `[offset, offset + len)` of the object.
    pub fn get_range(
        &self,
        key: &str,
        offset: u64,
        len: u64,
    ) -> Result<Box<dyn Read + Send>, S3Error> {
        if len == 0 {
            return Ok(Box::new(std::io::empty()));
        }
        let range = format!("bytes={offset}-{}", offset + len - 1);
        let r = self.call(
            "GET",
            key,
            &[],
            &[("range", range)],
            Payload::Empty,
            EMPTY_SHA256,
        )?;
        Ok(Box::new(r.into_body().into_reader().take(len)))
    }

    /// PUT one body. With `if_absent`, an existing key yields `Exists` and is left untouched.
    pub fn put(
        &self,
        key: &str,
        body: Payload<'_>,
        sha_hex: &str,
        content_type: &str,
        if_absent: bool,
    ) -> Result<(), S3Error> {
        let mut extra = vec![("content-type", content_type.to_string())];
        if if_absent {
            extra.push(("if-none-match", "*".to_string()));
        }
        self.call("PUT", key, &[], &extra, body, sha_hex)
            .map(|_| ())
    }

    /// Multipart upload of already-staged parts (`(file, sha256 hex)` each, in order).
    pub fn put_multipart(
        &self,
        key: &str,
        parts: &[(File, String)],
        content_type: &str,
    ) -> Result<(), S3Error> {
        let mut r = self.call(
            "POST",
            key,
            &[("uploads", String::new())],
            &[("content-type", content_type.to_string())],
            Payload::Empty,
            EMPTY_SHA256,
        )?;
        let xml = r
            .body_mut()
            .with_config()
            .limit(64 << 10)
            .read_to_string()
            .map_err(|e| S3Error::Transient(format!("create multipart {key}: {e}")))?;
        let upload_id = xml_tag(&xml, "UploadId")
            .ok_or_else(|| S3Error::Fatal(format!("create multipart {key}: no UploadId")))?;
        let result = (|| {
            let mut etags = Vec::with_capacity(parts.len());
            for (i, (f, sha)) in parts.iter().enumerate() {
                let n = (i + 1).to_string();
                let r = self.call(
                    "PUT",
                    key,
                    &[("partNumber", n), ("uploadId", upload_id.clone())],
                    &[],
                    Payload::File(f),
                    sha,
                )?;
                let etag = r
                    .headers()
                    .get("etag")
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| S3Error::Fatal(format!("part {} of {key}: no ETag", i + 1)))?;
                etags.push(etag.to_string());
            }
            let mut body = String::from("<CompleteMultipartUpload>");
            for (i, e) in etags.iter().enumerate() {
                body.push_str(&format!(
                    "<Part><PartNumber>{}</PartNumber><ETag>{}</ETag></Part>",
                    i + 1,
                    xml_escape(e)
                ));
            }
            body.push_str("</CompleteMultipartUpload>");
            let sha = hex::encode(Sha256::digest(body.as_bytes()));
            let mut r = self.call(
                "POST",
                key,
                &[("uploadId", upload_id.clone())],
                &[("content-type", "application/xml".into())],
                Payload::Bytes(body.as_bytes()),
                &sha,
            )?;
            // S3 may answer 200 with an <Error> body when completion fails late.
            let text = r
                .body_mut()
                .with_config()
                .limit(64 << 10)
                .read_to_string()
                .unwrap_or_default();
            if text.contains("<Error>") {
                return Err(S3Error::Transient(format!(
                    "complete multipart {key}: {text}"
                )));
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = self.call(
                "DELETE",
                key,
                &[("uploadId", upload_id)],
                &[],
                Payload::Empty,
                EMPTY_SHA256,
            );
        }
        result
    }
}

fn fatal(e: impl std::fmt::Display) -> S3Error {
    S3Error::Fatal(e.to_string())
}

fn xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&format!("</{tag}>"))? + start;
    Some(xml[start..end].to_string())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ------------------------------------------------------------------------------ SigV4

/// RFC 3986 encoding as SigV4 wants it (`/` kept in paths, encoded in query values).
pub fn uri_encode(s: &str, encode_slash: bool) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b'/' if !encode_slash => out.push('/'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn canonical_query(q: &[(&str, String)]) -> String {
    let mut pairs: Vec<(String, String)> = q
        .iter()
        .map(|(k, v)| (uri_encode(k, true), uri_encode(v, true)))
        .collect();
    pairs.sort();
    pairs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&")
}

/// Canonical form of a raw (already percent-encoded) query string: pairs sorted, `k` → `k=`.
pub fn canonical_query_of(raw: &str) -> String {
    let mut pairs: Vec<(String, String)> = raw
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| match p.split_once('=') {
            Some((k, v)) => (k.to_string(), v.to_string()),
            None => (p.to_string(), String::new()),
        })
        .collect();
    pairs.sort();
    pairs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&")
}

pub struct SigningInput<'a> {
    pub method: &'a str,
    /// Already URI-encoded absolute path.
    pub path: &'a str,
    /// Already canonical query string.
    pub query: &'a str,
    /// Every header to sign, lowercase names (must include `host` and `x-amz-date`).
    pub headers: &'a [(String, String)],
    pub payload_sha: &'a str,
    /// `YYYYMMDDTHHMMSSZ`.
    pub amz_date: &'a str,
    pub region: &'a str,
    pub service: &'a str,
    pub access_key_id: &'a str,
    pub secret: &'a str,
}

/// The `Authorization` header value for `i`.
pub fn sign_v4(i: &SigningInput<'_>) -> String {
    use ring::hmac;
    let mut hs: Vec<(String, String)> = i
        .headers
        .iter()
        .map(|(k, v)| {
            (
                k.to_ascii_lowercase(),
                v.split_whitespace().collect::<Vec<_>>().join(" "),
            )
        })
        .collect();
    hs.sort();
    let canonical_headers: String = hs.iter().map(|(k, v)| format!("{k}:{v}\n")).collect();
    let signed_headers = hs
        .iter()
        .map(|(k, _)| k.as_str())
        .collect::<Vec<_>>()
        .join(";");
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        i.method, i.path, i.query, canonical_headers, signed_headers, i.payload_sha
    );
    let date = &i.amz_date[..8];
    let scope = format!("{date}/{}/{}/aws4_request", i.region, i.service);
    let to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{scope}\n{}",
        i.amz_date,
        hex::encode(Sha256::digest(canonical_request.as_bytes()))
    );
    let mac = |key: &[u8], data: &str| {
        hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, key), data.as_bytes())
            .as_ref()
            .to_vec()
    };
    let k = mac(format!("AWS4{}", i.secret).as_bytes(), date);
    let k = mac(&k, i.region);
    let k = mac(&k, i.service);
    let k = mac(&k, "aws4_request");
    let sig = hex::encode(mac(&k, &to_sign));
    format!(
        "AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={sig}",
        i.access_key_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AWS documentation, "Signature Calculations for the Authorization Header", GET Object
    /// example (`examplebucket`, `test.txt`, `Range: bytes=0-9`).
    #[test]
    fn sigv4_matches_the_aws_s3_get_object_example() {
        let headers = vec![
            (
                "host".to_string(),
                "examplebucket.s3.amazonaws.com".to_string(),
            ),
            ("range".to_string(), "bytes=0-9".to_string()),
            ("x-amz-content-sha256".to_string(), EMPTY_SHA256.to_string()),
            ("x-amz-date".to_string(), "20130524T000000Z".to_string()),
        ];
        let auth = sign_v4(&SigningInput {
            method: "GET",
            path: "/test.txt",
            query: "",
            headers: &headers,
            payload_sha: EMPTY_SHA256,
            amz_date: "20130524T000000Z",
            region: "us-east-1",
            service: "s3",
            access_key_id: "AKIAIOSFODNN7EXAMPLE",
            secret: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        });
        assert_eq!(
            auth,
            "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20130524/us-east-1/s3/aws4_request, \
             SignedHeaders=host;range;x-amz-content-sha256;x-amz-date, \
             Signature=f0e8bdb87c964420e857bd35b5d6ed310bd44f0170aba48dd91039c6036bdb41"
        );
    }

    #[test]
    fn query_and_path_encoding() {
        assert_eq!(
            uri_encode("objects/ab/x y+z", false),
            "objects/ab/x%20y%2Bz"
        );
        assert_eq!(
            canonical_query(&[("uploadId", "a/b=".into()), ("partNumber", "2".into())]),
            "partNumber=2&uploadId=a%2Fb%3D"
        );
        assert_eq!(canonical_query(&[("uploads", String::new())]), "uploads=");
    }
}
