//! Lab/vault isolation. Every public lab entry point holds a [`LabSession`] while it runs; the
//! vault refuses to open (and logs the refusal of) any continuation while a session is active in
//! the process. Together with the [`crate::eval::VaultAccess`] token (mintable only inside
//! `crate::eval`) this makes held-out outcomes unreachable from lab code paths.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::util::refused;

static LAB_SESSIONS: AtomicUsize = AtomicUsize::new(0);

/// RAII marker: lab code is executing.
pub struct LabSession(());

impl LabSession {
    pub fn enter() -> LabSession {
        LAB_SESSIONS.fetch_add(1, Ordering::SeqCst);
        LabSession(())
    }
}

impl Drop for LabSession {
    fn drop(&mut self) {
        LAB_SESSIONS.fetch_sub(1, Ordering::SeqCst);
    }
}

pub fn lab_active() -> bool {
    LAB_SESSIONS.load(Ordering::SeqCst) > 0
}

pub fn assert_not_lab() -> anyhow::Result<()> {
    if lab_active() {
        return Err(refused("ignis lab code cannot read sealed continuations"));
    }
    Ok(())
}
