//! Read-only HTTP (blocking `ureq`): GET/HEAD/POST JSON and verified streaming downloads.

use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use anyhow::Context;
use md5::Md5;
use sha2::{Digest, Sha256};

pub struct Http {
    agent: ureq::Agent,
}

pub struct Got {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
    /// Response headers, names lower-cased (e.g. a provider's `x-ratelimit-*`).
    pub headers: Vec<(String, String)>,
}

impl Got {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

pub struct Streamed {
    pub sha256: String,
    pub md5: String,
    pub size: u64,
    pub content_type: Option<String>,
}

impl Http {
    pub fn new(user_agent: &str, timeout_s: u64) -> Http {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .user_agent(user_agent)
            .timeout_global(Some(Duration::from_secs(timeout_s)))
            .build()
            .into();
        Http { agent }
    }

    fn headers(h: &ureq::http::HeaderMap) -> Vec<(String, String)> {
        h.iter()
            .filter_map(|(k, v)| {
                Some((
                    k.as_str().to_ascii_lowercase(),
                    v.to_str().ok()?.to_string(),
                ))
            })
            .collect()
    }

    fn ctype(h: &ureq::http::HeaderMap) -> Option<String> {
        h.get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
    }

    pub fn get(&self, url: &str, query: &[(&str, &str)]) -> anyhow::Result<Got> {
        let mut req = self.agent.get(url);
        for (k, v) in query {
            req = req.query(*k, *v);
        }
        let mut r = req.call().with_context(|| format!("GET {url}"))?;
        let content_type = Self::ctype(r.headers());
        let headers = Self::headers(r.headers());
        let bytes = r.body_mut().with_config().limit(u64::MAX).read_to_vec()?;
        Ok(Got {
            bytes,
            content_type,
            headers,
        })
    }

    pub fn post_json(&self, url: &str, body: &serde_json::Value) -> anyhow::Result<Got> {
        let mut r = self
            .agent
            .post(url)
            .header("Content-Type", "application/json")
            .send(serde_json::to_string(body)?)
            .with_context(|| format!("POST {url}"))?;
        let content_type = Self::ctype(r.headers());
        let headers = Self::headers(r.headers());
        let bytes = r.body_mut().with_config().limit(u64::MAX).read_to_vec()?;
        Ok(Got {
            bytes,
            content_type,
            headers,
        })
    }

    /// Exact size from a HEAD `content-length`, when the server sends one.
    pub fn head_size(&self, url: &str) -> anyhow::Result<Option<u64>> {
        let r = self
            .agent
            .head(url)
            .call()
            .with_context(|| format!("HEAD {url}"))?;
        Ok(r.headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok()))
    }

    /// Stream `url` to `dest` via `<dest>.part`, hashing as it goes; the part file is removed on error.
    pub fn stream_to(
        &self,
        url: &str,
        dest: &Path,
        expected_size: Option<u64>,
    ) -> anyhow::Result<Streamed> {
        if let Some(d) = dest.parent() {
            std::fs::create_dir_all(d)?;
        }
        let mut tmp = dest.as_os_str().to_owned();
        tmp.push(".part");
        let tmp = std::path::PathBuf::from(tmp);
        let res = (|| -> anyhow::Result<Streamed> {
            let mut r = self
                .agent
                .get(url)
                .call()
                .with_context(|| format!("GET {url}"))?;
            let content_type = Self::ctype(r.headers());
            let mut reader = r.body_mut().with_config().limit(u64::MAX).reader();
            let mut f = std::io::BufWriter::new(std::fs::File::create(&tmp)?);
            let (mut h, mut m, mut n) = (Sha256::new(), Md5::new(), 0u64);
            let mut buf = vec![0u8; 1 << 20];
            loop {
                let k = reader.read(&mut buf)?;
                if k == 0 {
                    break;
                }
                f.write_all(&buf[..k])?;
                h.update(&buf[..k]);
                m.update(&buf[..k]);
                n += k as u64;
            }
            f.flush()?;
            drop(f);
            if let Some(e) = expected_size {
                anyhow::ensure!(n == e, "{url}: got {n} B, listing says {e} B");
            }
            Ok(Streamed {
                sha256: hex::encode(h.finalize()),
                md5: hex::encode(m.finalize()),
                size: n,
                content_type,
            })
        })();
        match res {
            Ok(s) => {
                std::fs::rename(&tmp, dest)?;
                Ok(s)
            }
            Err(e) => {
                let _ = std::fs::remove_file(&tmp);
                Err(e)
            }
        }
    }
}

pub fn sha256_bytes(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}
