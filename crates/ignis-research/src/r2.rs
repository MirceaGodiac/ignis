//! Cloudflare R2 backend of [`ArtifactStore`] (S3 API), same layout and identity as
//! [`crate::LocalArtifactStore`]:
//! ```text
//! <prefix>objects/<hex[..2]>/<hex>     immutable bytes, named by their sha256
//! <prefix>manifests/<id-hex>.json      the published ArtifactRef + owning workspace
//! ```
//! `put` streams the input into local staging parts (64 MiB) while hashing the whole object and
//! each part, uploads the bytes under their content address unless already present (one PUT, or a
//! multipart upload above one part; every body is signed over its own SHA-256, so R2 refuses bytes
//! that differ from what was hashed), and only then publishes the manifest with
//! `If-None-Match: *`. A manifest therefore never names incomplete bytes; a crash leaves at worst
//! an unreferenced object or a local staging directory. Manifests are immutable, so the local copy
//! under `<local>/manifests` is a cache that never needs invalidation.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    ArtifactId, ArtifactRef, Digest, ErrorCode, ErrorEnvelope, ExecutionScope, RetryPolicy,
};
use ignis_core::services::{ArtifactDraft, ArtifactStore, ServiceResult};
use sha2::{Digest as _, Sha256};

use crate::artifacts::{artifact_ref, manifest_hex, publish_file, Manifest};
use crate::s3::{Payload, S3Client, S3Config, S3Error};
use crate::{internal, new_id, not_found};

/// Size of one staged part (and of one multipart part). R2 needs ≥ 5 MiB for all parts but the last.
pub const PART_BYTES: u64 = 64 << 20;
const MANIFEST_MAX_BYTES: u64 = 1 << 20;

pub struct R2ArtifactStore {
    s3: S3Client,
    prefix: String,
    local: PathBuf,
    part_bytes: u64,
}

fn upstream(what: &str, e: S3Error) -> ErrorEnvelope {
    match e {
        S3Error::Transient(m) => ErrorEnvelope::new(
            ErrorCode::UpstreamUnavailable,
            format!("artifact storage: {what}: {m}"),
            RetryPolicy::AfterDelay { seconds: 30 },
        ),
        other => internal(&format!("artifact storage: {what}"), other),
    }
}

impl R2ArtifactStore {
    /// `prefix` separates instances sharing a bucket (e.g. `registry/`); `local` holds staging parts
    /// and the manifest cache (local NVMe, never the only copy of anything).
    pub fn open(cfg: S3Config, prefix: &str, local: impl AsRef<Path>) -> ServiceResult<Self> {
        Self::with_part_size(cfg, prefix, local, PART_BYTES)
    }

    /// As [`open`](Self::open) with another part size (tests exercise multipart with small parts).
    pub fn with_part_size(
        cfg: S3Config,
        prefix: &str,
        local: impl AsRef<Path>,
        part_bytes: u64,
    ) -> ServiceResult<Self> {
        let local = local.as_ref().to_path_buf();
        for d in ["staging", "manifests"] {
            std::fs::create_dir_all(local.join(d)).map_err(|e| internal("artifact store", e))?;
        }
        let prefix = match prefix.trim_matches('/') {
            "" => String::new(),
            p => format!("{p}/"),
        };
        let s3 = S3Client::new(cfg).map_err(|e| internal("artifact storage", e))?;
        let store = R2ArtifactStore {
            s3,
            prefix,
            local,
            part_bytes: part_bytes.max(1),
        };
        store.collect_staging(std::time::Duration::from_secs(3600));
        Ok(store)
    }

    /// Readiness: the bucket answers with these credentials (a missing probe key is fine).
    pub fn probe(&self) -> Result<(), String> {
        match self.s3.head(&format!("{}manifests/.probe", self.prefix)) {
            Ok(_) | Err(S3Error::NotFound) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Remove local staging directories older than `age` (interrupted `put`s).
    pub fn collect_staging(&self, age: std::time::Duration) -> usize {
        let Ok(rd) = std::fs::read_dir(self.local.join("staging")) else {
            return 0;
        };
        rd.filter_map(Result::ok)
            .filter(|e| {
                e.metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.elapsed().ok())
                    .is_some_and(|el| el >= age)
            })
            .filter(|e| std::fs::remove_dir_all(e.path()).is_ok())
            .count()
    }

    fn object_key(&self, sha: &Digest) -> String {
        let h = sha.hex();
        format!("{}objects/{}/{}", self.prefix, &h[..2], h)
    }

    fn manifest_key(&self, hex: &str) -> String {
        format!("{}manifests/{hex}.json", self.prefix)
    }

    fn manifest(&self, scope: &ExecutionScope, id: &ArtifactId) -> ServiceResult<Manifest> {
        let hex = manifest_hex(id)?;
        let cached = self.local.join("manifests").join(format!("{hex}.json"));
        let bytes = match std::fs::read(&cached) {
            Ok(b) => b,
            Err(_) => {
                let b = match self
                    .s3
                    .get_bytes(&self.manifest_key(&hex), MANIFEST_MAX_BYTES)
                {
                    Ok(b) => b,
                    Err(S3Error::NotFound) => return Err(not_found(format!("artifact {id}"))),
                    Err(e) => return Err(upstream("read manifest", e)),
                };
                let _ = publish_file(&cached, &b);
                b
            }
        };
        let m: Manifest = serde_json::from_slice(&bytes).map_err(|e| internal("manifest", e))?;
        if m.workspace_id != scope.workspace_id {
            return Err(not_found(format!("artifact {id}")));
        }
        Ok(m)
    }

    /// Stream `bytes` into part files under `dir`; returns the parts (file + sha256 hex), the
    /// whole-object digest and length.
    fn stage(
        &self,
        dir: &Path,
        bytes: &mut dyn Read,
    ) -> ServiceResult<(Vec<(PathBuf, String)>, Digest, u64)> {
        let mut whole = Sha256::new();
        let mut parts = Vec::new();
        let mut len = 0u64;
        let mut buf = vec![0u8; 64 * 1024];
        let mut eof = false;
        while !eof {
            let path = dir.join(format!("{:06}.part", parts.len() + 1));
            let mut f = File::create(&path).map_err(|e| internal("staging", e))?;
            let mut part = Sha256::new();
            let mut in_part = 0u64;
            while in_part < self.part_bytes {
                let want = (self.part_bytes - in_part).min(buf.len() as u64) as usize;
                let n = bytes.read(&mut buf[..want]).map_err(|e| {
                    ErrorEnvelope::new(
                        ErrorCode::InvalidRequest,
                        format!("reading artifact input: {e}"),
                        RetryPolicy::Never,
                    )
                })?;
                if n == 0 {
                    eof = true;
                    break;
                }
                whole.update(&buf[..n]);
                part.update(&buf[..n]);
                f.write_all(&buf[..n]).map_err(|e| internal("staging", e))?;
                in_part += n as u64;
            }
            f.sync_all().map_err(|e| internal("staging", e))?;
            if in_part == 0 && !parts.is_empty() {
                let _ = std::fs::remove_file(&path);
                break;
            }
            len += in_part;
            parts.push((path, hex::encode(part.finalize())));
        }
        let sha = Digest::parse(&format!("sha256:{}", hex::encode(whole.finalize())))
            .map_err(|e| internal("digest", e))?;
        Ok((parts, sha, len))
    }

    fn upload(
        &self,
        key: &str,
        parts: &[(PathBuf, String)],
        sha: &Digest,
        len: u64,
        media_type: &str,
    ) -> ServiceResult<()> {
        match self.s3.head(key) {
            Ok(n) if n == len => return Ok(()), // same address, same bytes
            Ok(n) => {
                return Err(internal(
                    "artifact storage",
                    format!("object {key} has {n} bytes, its address says {len}"),
                ))
            }
            Err(S3Error::NotFound) => {}
            Err(e) => return Err(upstream("stat object", e)),
        }
        let open = |p: &PathBuf| File::open(p).map_err(|e| internal("staging", e));
        if parts.len() == 1 {
            let f = open(&parts[0].0)?;
            // The body hash is the object hash: R2 verifies it.
            debug_assert_eq!(parts[0].1, sha.hex());
            self.s3
                .put(key, Payload::File(&f), &sha.hex(), media_type, false)
                .map_err(|e| upstream("upload object", e))
        } else {
            let files = parts
                .iter()
                .map(|(p, h)| Ok((open(p)?, h.clone())))
                .collect::<ServiceResult<Vec<_>>>()?;
            self.s3
                .put_multipart(key, &files, media_type)
                .map_err(|e| upstream("multipart upload", e))
        }
    }
}

impl ArtifactStore for R2ArtifactStore {
    fn put(
        &self,
        scope: &ExecutionScope,
        draft: ArtifactDraft,
        bytes: &mut dyn Read,
    ) -> ServiceResult<ArtifactRef> {
        let dir = self.local.join("staging").join(new_id("put"));
        std::fs::create_dir_all(&dir).map_err(|e| internal("staging", e))?;
        let result = (|| {
            let (parts, sha, len) = self.stage(&dir, bytes)?;
            let media_type = draft.media_type.clone();
            let r = artifact_ref(scope, draft, sha.clone(), len)?;
            if let Ok(existing) = self.manifest(scope, &r.artifact_id) {
                return Ok(existing.artifact);
            }
            self.upload(&self.object_key(&sha), &parts, &sha, len, &media_type)?;
            let m = Manifest {
                workspace_id: scope.workspace_id.clone(),
                artifact: r.clone(),
            };
            let text = serde_json::to_vec_pretty(&m).map_err(|e| internal("manifest", e))?;
            let hex = manifest_hex(&r.artifact_id)?;
            let mhash = hex::encode(Sha256::digest(&text));
            match self.s3.put(
                &self.manifest_key(&hex),
                Payload::Bytes(&text),
                &mhash,
                "application/json",
                true,
            ) {
                Ok(()) => {
                    let _ = publish_file(
                        &self.local.join("manifests").join(format!("{hex}.json")),
                        &text,
                    );
                    Ok(r)
                }
                // Published concurrently by another writer: theirs is the reference (first wins).
                Err(S3Error::Exists) => Ok(self.manifest(scope, &r.artifact_id)?.artifact),
                Err(e) => Err(upstream("publish manifest", e)),
            }
        })();
        let _ = std::fs::remove_dir_all(&dir);
        result
    }

    fn stat(&self, scope: &ExecutionScope, id: &ArtifactId) -> ServiceResult<ArtifactRef> {
        Ok(self.manifest(scope, id)?.artifact)
    }

    fn open(
        &self,
        scope: &ExecutionScope,
        id: &ArtifactId,
        offset: u64,
        len: Option<u64>,
    ) -> ServiceResult<Box<dyn Read + Send>> {
        let a = self.manifest(scope, id)?.artifact;
        if offset > a.byte_length {
            return Err(ErrorEnvelope::new(
                ErrorCode::InvalidRequest,
                format!(
                    "offset {offset} is past the end of artifact {id} ({} bytes)",
                    a.byte_length
                ),
                RetryPolicy::Never,
            ));
        }
        let take = len
            .unwrap_or(a.byte_length - offset)
            .min(a.byte_length - offset);
        self.s3
            .get_range(&self.object_key(&a.sha256), offset, take)
            .map_err(|e| match e {
                S3Error::NotFound => internal(
                    "artifact storage",
                    format!("object of published artifact {id} is missing"),
                ),
                e => upstream("read object", e),
            })
    }
}
