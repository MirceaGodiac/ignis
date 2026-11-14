//! Per-field interpretation strategy, declared and inspectable.
//!
//! Each family ingest module declares a [`FieldRule`] for every quantity/label it produces: which
//! method interprets it (deterministic / numerical / semantic / human review), why, and its version.
//! `ignis data rules <family>` prints the table, so the choice is never implicit.
//!
//! Semantic rules never run an LLM inside the pipeline. They emit a *review item* (cited source text
//! plus protocol excerpt) to `data/review/<family>/`; an adjudication file written by a person or a
//! local agent session is read back with its agent/reviewer recorded in `Derivation`. Until then the
//! value stays `unresolved_conflict` or `unknown`.

use serde::Serialize;

use crate::contracts::Method;

#[derive(Debug, Clone, Serialize)]
pub struct FieldRule {
    pub family: &'static str,
    pub field: &'static str,
    pub method: Method,
    pub method_id: &'static str,
    pub version: &'static str,
    pub rationale: &'static str,
}
