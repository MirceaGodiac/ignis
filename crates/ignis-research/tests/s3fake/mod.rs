//! In-process S3 stand-in that behaves like R2 where it matters: it recomputes the SigV4 signature
//! from the request as received (so what goes on the wire is what was signed), refuses a body whose
//! SHA-256 differs from `x-amz-content-sha256`, refuses chunked uploads, honours `If-None-Match: *`,
//! `Range` and multipart upload.
#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::Router;
use ignis_research::s3::{canonical_query_of, sign_v4, S3Config, SigningInput};
use sha2::{Digest as _, Sha256};

pub const KEY_ID: &str = "test-key-id";
pub const SECRET: &str = "test-secret";

#[derive(Default)]
pub struct Bucket {
    pub objects: BTreeMap<String, Vec<u8>>,
    pub uploads: HashMap<String, BTreeMap<u32, Vec<u8>>>,
    /// (method, key, query) of every request, for assertions.
    pub log: Vec<(String, String, String)>,
    /// Requests refused by the checks (should stay empty).
    pub refused: Vec<String>,
}

pub type Shared = Arc<Mutex<Bucket>>;

fn check(method: &Method, uri: &Uri, h: &HeaderMap, body: &[u8]) -> Result<(), String> {
    if h.get("transfer-encoding").is_some() {
        return Err("chunked body".into());
    }
    let claimed = h
        .get("x-amz-content-sha256")
        .and_then(|v| v.to_str().ok())
        .ok_or("no payload hash")?;
    if claimed != hex::encode(Sha256::digest(body)) {
        return Err("payload hash mismatch".into());
    }
    let auth = h
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or("no authorization")?;
    let signed = auth
        .split("SignedHeaders=")
        .nth(1)
        .and_then(|s| s.split(',').next())
        .ok_or("no signed headers")?;
    let headers: Vec<(String, String)> = signed
        .split(';')
        .map(|k| {
            let v = h.get(k).and_then(|v| v.to_str().ok()).unwrap_or_default();
            (k.to_string(), v.to_string())
        })
        .collect();
    let amz_date = h
        .get("x-amz-date")
        .and_then(|v| v.to_str().ok())
        .ok_or("no date")?;
    let expected = sign_v4(&SigningInput {
        method: method.as_str(),
        path: uri.path(),
        query: &canonical_query_of(uri.query().unwrap_or("")),
        headers: &headers,
        payload_sha: claimed,
        amz_date,
        region: "auto",
        service: "s3",
        access_key_id: KEY_ID,
        secret: SECRET,
    });
    if expected != auth {
        return Err(format!("signature mismatch\n got {auth}\nwant {expected}"));
    }
    for k in ["host", "x-amz-date", "x-amz-content-sha256"] {
        if !signed.split(';').any(|s| s == k) {
            return Err(format!("{k} not signed"));
        }
    }
    Ok(())
}

async fn handle(
    State(b): State<Shared>,
    method: Method,
    uri: Uri,
    Path((bucket, key)): Path<(String, String)>,
    Query(q): Query<HashMap<String, String>>,
    h: HeaderMap,
    body: Bytes,
) -> Response {
    let mut b = b.lock().unwrap();
    b.log.push((
        method.to_string(),
        key.clone(),
        uri.query().unwrap_or("").to_string(),
    ));
    if bucket != "ignis-test" {
        return StatusCode::NOT_FOUND.into_response();
    }
    if let Err(e) = check(&method, &uri, &h, &body) {
        b.refused.push(format!("{method} {key}: {e}"));
        return (StatusCode::FORBIDDEN, e).into_response();
    }
    match (
        method.as_str(),
        q.get("uploadId"),
        q.contains_key("uploads"),
    ) {
        ("POST", None, true) => {
            let id = format!("up-{}", b.uploads.len() + 1);
            b.uploads.insert(id.clone(), BTreeMap::new());
            format!("<InitiateMultipartUploadResult><UploadId>{id}</UploadId></InitiateMultipartUploadResult>").into_response()
        }
        ("PUT", Some(id), _) => {
            let n: u32 = q["partNumber"].parse().unwrap();
            let etag = format!("\"{}\"", hex::encode(&Sha256::digest(&body)[..8]));
            match b.uploads.get_mut(id) {
                Some(u) => {
                    u.insert(n, body.to_vec());
                    ([("etag", etag)], "").into_response()
                }
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
        ("POST", Some(id), _) => {
            let Some(parts) = b.uploads.remove(id) else {
                return StatusCode::NOT_FOUND.into_response();
            };
            let text = String::from_utf8_lossy(&body);
            assert_eq!(text.matches("<Part>").count(), parts.len(), "{text}");
            let bytes: Vec<u8> = parts.into_values().flatten().collect();
            b.objects.insert(key, bytes);
            "<CompleteMultipartUploadResult/>".into_response()
        }
        ("DELETE", Some(id), _) => {
            b.uploads.remove(id);
            StatusCode::NO_CONTENT.into_response()
        }
        ("PUT", None, _) => {
            if h.get("if-none-match").is_some_and(|v| v == "*") && b.objects.contains_key(&key) {
                return StatusCode::PRECONDITION_FAILED.into_response();
            }
            b.objects.insert(key, body.to_vec());
            StatusCode::OK.into_response()
        }
        ("HEAD", None, _) => match b.objects.get(&key) {
            Some(o) => ([("content-length", o.len().to_string())], "").into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        },
        ("GET", None, _) => {
            let Some(o) = b.objects.get(&key) else {
                return StatusCode::NOT_FOUND.into_response();
            };
            match h.get("range").and_then(|v| v.to_str().ok()) {
                Some(r) => {
                    let (a, z) = r.strip_prefix("bytes=").unwrap().split_once('-').unwrap();
                    let (a, z): (usize, usize) = (a.parse().unwrap(), z.parse().unwrap());
                    let z = z.min(o.len() - 1);
                    (StatusCode::PARTIAL_CONTENT, o[a..=z].to_vec()).into_response()
                }
                None => o.clone().into_response(),
            }
        }
        _ => StatusCode::METHOD_NOT_ALLOWED.into_response(),
    }
}

pub struct Fake {
    pub bucket: Shared,
    pub endpoint: String,
}

pub fn fake() -> Fake {
    let bucket: Shared = Default::default();
    let app = Router::new()
        .route("/{bucket}/{*key}", axum::routing::any(handle))
        .with_state(bucket.clone());
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                let l = tokio::net::TcpListener::from_std(listener).unwrap();
                axum::serve(l, app).await.unwrap();
            })
    });
    Fake {
        bucket,
        endpoint: format!("http://127.0.0.1:{port}"),
    }
}

pub fn cfg(f: &Fake) -> S3Config {
    S3Config {
        endpoint: f.endpoint.clone(),
        bucket: "ignis-test".into(),
        region: "auto".into(),
        access_key_id: KEY_ID.into(),
        secret_access_key: SECRET.into(),
    }
}
