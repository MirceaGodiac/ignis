//! Local content-addressed artifact store (the deployment uses the R2 backend, [`crate::r2`], with the
//! same layout and identity).
//!
//! Layout under the root:
//! ```text
//! staging/<random>.part          bytes being written (never referenced)
//! objects/<hex[..2]>/<hex>       immutable bytes, named by their sha256
//! manifests/<id-hex>.json        the published ArtifactRef + owning workspace
//! ```
//! `put` streams the input into a staging file while hashing it, syncs it, renames it to its
//! content address (or drops it when the bytes already exist), then publishes the manifest with an
//! atomic rename. A returned [`ArtifactRef`] therefore never points at incomplete bytes, and a crash
//! leaves at worst an orphan in `staging/`.
//!
//! The artifact id is derived from the bytes and the declared metadata (not the time), so storing
//! the same output twice yields the same reference (the first `created_at` is kept).

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use ignis_core::contracts::{
    ArtifactId, ArtifactReadRequest, ArtifactRef, Digest, ErrorCode, ErrorEnvelope, ExecutionScope,
    InlinePayload, RetryPolicy, Timestamp, WorkspaceId,
};
use ignis_core::hashing::content_hash;
use ignis_core::services::{ArtifactDraft, ArtifactStore, ServiceResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::{internal, new_id, not_found};

#[derive(Serialize, Deserialize)]
pub(crate) struct Manifest {
    pub(crate) workspace_id: WorkspaceId,
    pub(crate) artifact: ArtifactRef,
}

/// The 64-hex identity of an artifact id (`art_<hex>`), or not-found for anything else.
pub(crate) fn manifest_hex(id: &ArtifactId) -> ServiceResult<String> {
    let hex = id.as_str().strip_prefix("art_").unwrap_or_default();
    if hex.len() != 64 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(not_found(format!("artifact {id}")));
    }
    Ok(hex.to_string())
}

/// The reference of `len` bytes with digest `sha` declared by `draft` in `scope`'s workspace. The id
/// is derived from the bytes and the declared metadata (never the time).
pub(crate) fn artifact_ref(
    scope: &ExecutionScope,
    draft: ArtifactDraft,
    sha: Digest,
    len: u64,
) -> ServiceResult<ArtifactRef> {
    let mut r = ArtifactRef {
        artifact_id: ArtifactId::new("art_pending").expect("literal id"),
        sha256: sha,
        media_type: draft.media_type,
        byte_length: len,
        contract: draft.contract,
        produced_by: draft.produced_by,
        inputs: draft.inputs,
        origin: draft.origin,
        created_at: Timestamp::now(),
    };
    r.validate()?;
    let identity = content_hash(
        &(
            &scope.workspace_id,
            &r.sha256,
            &r.media_type,
            &r.contract,
            &r.produced_by,
            &r.inputs,
            &r.origin,
        ),
        &[],
    );
    let hex = identity.strip_prefix("sha256:").unwrap_or(&identity);
    r.artifact_id = ArtifactId::new(format!("art_{hex}")).expect("generated id");
    Ok(r)
}

pub struct LocalArtifactStore {
    root: PathBuf,
}

impl LocalArtifactStore {
    pub fn open(root: impl AsRef<Path>) -> ServiceResult<Self> {
        let root = root.as_ref().to_path_buf();
        for d in ["staging", "objects", "manifests"] {
            std::fs::create_dir_all(root.join(d)).map_err(|e| internal("artifact store", e))?;
        }
        let store = LocalArtifactStore { root };
        store.collect_staging(std::time::Duration::from_secs(3600));
        Ok(store)
    }

    /// Remove staging files older than `age` (left by an interrupted `put`; never referenced, since
    /// a manifest is published only after its bytes are promoted). Returns how many were removed.
    pub fn collect_staging(&self, age: std::time::Duration) -> usize {
        let Ok(rd) = std::fs::read_dir(self.root.join("staging")) else {
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
            .filter(|e| std::fs::remove_file(e.path()).is_ok())
            .count()
    }

    fn object_path(&self, sha: &Digest) -> PathBuf {
        let h = sha.hex();
        self.root.join("objects").join(&h[..2]).join(h)
    }

    fn manifest_path(&self, id: &ArtifactId) -> ServiceResult<PathBuf> {
        let hex = manifest_hex(id)?;
        Ok(self.root.join("manifests").join(format!("{hex}.json")))
    }

    fn manifest(&self, scope: &ExecutionScope, id: &ArtifactId) -> ServiceResult<Manifest> {
        let p = self.manifest_path(id)?;
        let text = match std::fs::read_to_string(&p) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(not_found(format!("artifact {id}")))
            }
            Err(e) => return Err(internal("read manifest", e)),
        };
        let m: Manifest = serde_json::from_str(&text).map_err(|e| internal("manifest", e))?;
        if m.workspace_id != scope.workspace_id {
            return Err(not_found(format!("artifact {id}")));
        }
        Ok(m)
    }
}

/// Write `bytes` to `path` through a sibling temp file and an atomic rename.
pub(crate) fn publish_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4().simple()));
    {
        let mut f = File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

impl ArtifactStore for LocalArtifactStore {
    fn put(
        &self,
        scope: &ExecutionScope,
        draft: ArtifactDraft,
        bytes: &mut dyn Read,
    ) -> ServiceResult<ArtifactRef> {
        let staging = self
            .root
            .join("staging")
            .join(format!("{}.part", new_id("put")));
        let result = (|| {
            let mut f = File::create(&staging).map_err(|e| internal("staging", e))?;
            let mut hasher = Sha256::new();
            let mut len = 0u64;
            let mut buf = vec![0u8; 64 * 1024];
            loop {
                let n = bytes.read(&mut buf).map_err(|e| {
                    ErrorEnvelope::new(
                        ErrorCode::InvalidRequest,
                        format!("reading artifact input: {e}"),
                        RetryPolicy::Never,
                    )
                })?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
                f.write_all(&buf[..n]).map_err(|e| internal("staging", e))?;
                len += n as u64;
            }
            f.sync_all().map_err(|e| internal("staging", e))?;
            drop(f);
            let sha = Digest::parse(&format!("sha256:{}", hex::encode(hasher.finalize())))
                .map_err(|e| internal("digest", e))?;
            let obj = self.object_path(&sha);
            if obj.exists() {
                let _ = std::fs::remove_file(&staging);
            } else {
                std::fs::create_dir_all(obj.parent().unwrap())
                    .map_err(|e| internal("objects", e))?;
                std::fs::rename(&staging, &obj).map_err(|e| internal("promote", e))?;
            }
            let r = artifact_ref(scope, draft, sha, len)?;
            if let Ok(existing) = self.manifest(scope, &r.artifact_id) {
                return Ok(existing.artifact);
            }
            let m = Manifest {
                workspace_id: scope.workspace_id.clone(),
                artifact: r.clone(),
            };
            let text = serde_json::to_vec_pretty(&m).map_err(|e| internal("manifest", e))?;
            publish_file(&self.manifest_path(&r.artifact_id)?, &text)
                .map_err(|e| internal("publish manifest", e))?;
            Ok(r)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&staging);
        }
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
        let mut f =
            File::open(self.object_path(&a.sha256)).map_err(|e| internal("open object", e))?;
        f.seek(SeekFrom::Start(offset))
            .map_err(|e| internal("seek object", e))?;
        let take = len
            .unwrap_or(a.byte_length - offset)
            .min(a.byte_length - offset);
        Ok(Box::new(f.take(take)))
    }
}

// ---------------------------------------------------------------------------- reads

/// Largest JSON artifact resolved server-side for a JSON Pointer read (the parse holds it in memory;
/// larger artifacts are read by byte range).
pub const POINTER_MAX_BYTES: u64 = 8 << 20;

/// Bytes returned inline by one `artifacts_read` page when the caller gives no length, and the
/// budget of a default array slice (a context budget, not a size limit: the page says where the
/// next one starts). Kept well under the tool's inline limit, since JSON escaping of the text
/// grows it on the wire.
pub const INLINE_PAGE_BYTES: u64 = 24 << 10;

/// `true` for `application/json` and `+json` media types.
pub fn is_json(media_type: &str) -> bool {
    let m = media_type.split(';').next().unwrap_or("").trim();
    m == "application/json" || m.ends_with("+json")
}

fn refused(code: ErrorCode, msg: impl Into<String>) -> ErrorEnvelope {
    ErrorEnvelope::new(code, msg, RetryPolicy::Never)
}

/// The value at an RFC 6901 `pointer` of a stored JSON artifact. One implementation for the HTTP
/// route and the `artifacts_read` tool.
pub fn read_json_pointer(
    store: &dyn ArtifactStore,
    scope: &ExecutionScope,
    meta: &ArtifactRef,
    pointer: &str,
) -> ServiceResult<InlinePayload> {
    if !is_json(&meta.media_type) {
        return Err(refused(
            ErrorCode::Unsupported,
            format!("{} is not JSON; read it by byte range", meta.media_type),
        ));
    }
    if meta.byte_length > POINTER_MAX_BYTES {
        return Err(refused(
            ErrorCode::Unsupported,
            "artifact too large for a server-side pointer read; read it by byte range",
        ));
    }
    if !(pointer.is_empty() || pointer.starts_with('/')) {
        return Err(refused(
            ErrorCode::InvalidRequest,
            format!("{pointer:?} is not a JSON Pointer"),
        ));
    }
    let mut text = Vec::new();
    store
        .open(scope, &meta.artifact_id, 0, None)?
        .read_to_end(&mut text)
        .map_err(|e| internal("read artifact", e))?;
    let value: serde_json::Value = serde_json::from_slice(&text)
        .map_err(|e| internal("stored JSON artifact does not parse", e))?;
    let found = value.pointer(pointer).cloned().ok_or_else(|| {
        refused(
            ErrorCode::NotFound,
            format!("pointer {pointer:?} not in the artifact"),
        )
    })?;
    Ok(InlinePayload {
        value: found,
        partial: false,
        full: Some(meta.clone()),
    })
}

/// One slice of an array read by pointer: `{items, item_offset, of, next_item}`. Without a count,
/// as many items from `offset` as fit [`INLINE_PAGE_BYTES`] (at least one, so a read always
/// advances). Partial unless the slice is the whole array.
fn slice_items(
    items: Vec<serde_json::Value>,
    offset: Option<u64>,
    count: Option<u64>,
    meta: &ArtifactRef,
) -> ServiceResult<InlinePayload> {
    let len = items.len();
    let start = offset.unwrap_or(0);
    if start > len as u64 {
        return Err(refused(
            ErrorCode::InvalidRequest,
            format!("item_offset {start} is past the end of the array ({len} items)"),
        ));
    }
    let start = start as usize;
    let end = match count {
        Some(n) => start
            .saturating_add(usize::try_from(n).unwrap_or(usize::MAX))
            .min(len),
        None => {
            let budget = INLINE_PAGE_BYTES as usize;
            let mut used = 2usize;
            let mut end = start;
            for it in &items[start..] {
                let n = serde_json::to_vec(it).map_or(usize::MAX, |b| b.len()) + 1;
                if end > start && used.saturating_add(n) > budget {
                    break;
                }
                used = used.saturating_add(n);
                end += 1;
            }
            end
        }
    };
    let next = (end < len).then_some(end);
    let slice: Vec<serde_json::Value> = items.into_iter().skip(start).take(end - start).collect();
    Ok(InlinePayload {
        value: serde_json::json!({
            "items": slice,
            "item_offset": start,
            "of": len,
            "next_item": next,
        }),
        partial: start > 0 || end < len,
        full: Some(meta.clone()),
    })
}

/// `artifacts_read` for an agent: a JSON Pointer, or one page of text by byte range. A pointer to
/// an array answers one slice of it when the caller asks for one (`item_offset`/`item_count`) or
/// when the array does not fit one page. A page or slice that does not reach the end is `partial`
/// and says where the next one starts. Binary bytes are not inlined into a model context (the HTTP
/// route serves them).
pub fn read_for_tool(
    store: &dyn ArtifactStore,
    scope: &ExecutionScope,
    req: &ArtifactReadRequest,
) -> ServiceResult<InlinePayload> {
    let meta = store.stat(scope, &req.artifact_id)?;
    let slicing = req.item_offset.is_some() || req.item_count.is_some();
    if let Some(p) = &req.json_pointer {
        if req.byte_offset.is_some() || req.byte_length.is_some() {
            return Err(refused(
                ErrorCode::InvalidRequest,
                "json_pointer and byte_offset/byte_length are exclusive",
            ));
        }
        let found = read_json_pointer(store, scope, &meta, p)?;
        let slice = found.value.is_array()
            && (slicing
                || serde_json::to_vec(&found.value)
                    .map_or(true, |b| b.len() as u64 > INLINE_PAGE_BYTES));
        return match found.value {
            serde_json::Value::Array(items) if slice => {
                slice_items(items, req.item_offset, req.item_count, &meta)
            }
            _ if slicing => Err(refused(
                ErrorCode::InvalidRequest,
                format!(
                    "item_offset/item_count slice an array; pointer {p:?} does not resolve to one"
                ),
            )),
            value => Ok(InlinePayload { value, ..found }),
        };
    }
    if slicing {
        return Err(refused(
            ErrorCode::InvalidRequest,
            "item_offset/item_count need a json_pointer to an array",
        ));
    }
    let offset = req.byte_offset.unwrap_or(0);
    if offset > meta.byte_length {
        return Err(refused(
            ErrorCode::InvalidRequest,
            format!(
                "byte_offset {offset} is past the end ({} bytes)",
                meta.byte_length
            ),
        ));
    }
    let len = req
        .byte_length
        .unwrap_or(INLINE_PAGE_BYTES)
        .min(meta.byte_length - offset);
    let mut bytes = Vec::with_capacity(len as usize);
    store
        .open(scope, &meta.artifact_id, offset, Some(len))?
        .read_to_end(&mut bytes)
        .map_err(|e| internal("read artifact", e))?;
    // A page boundary may split a UTF-8 sequence: end the page at the last complete character.
    let text = match std::str::from_utf8(&bytes) {
        Ok(t) => t,
        Err(e) if e.error_len().is_none() && e.valid_up_to() > 0 => {
            std::str::from_utf8(&bytes[..e.valid_up_to()]).expect("checked prefix")
        }
        Err(_) => {
            return Err(refused(
                ErrorCode::Unsupported,
                format!(
                    "{} is not text; open it through GET /api/v1/artifacts/{}",
                    meta.media_type, meta.artifact_id
                ),
            ))
        }
    };
    let end = offset + text.len() as u64;
    let partial = offset > 0 || end < meta.byte_length;
    Ok(InlinePayload {
        value: serde_json::json!({
            "media_type": meta.media_type,
            "byte_offset": offset,
            "byte_length": text.len(),
            "total_bytes": meta.byte_length,
            "next_offset": (end < meta.byte_length).then_some(end),
            "text": text,
        }),
        partial,
        full: Some(meta),
    })
}
