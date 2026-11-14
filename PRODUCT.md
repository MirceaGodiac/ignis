# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

- **Primary:** researchers who work with NASA microgravity-combustion data (the challenge names
  professors, graduate and undergraduate students, aerospace researchers and aerospace engineers).
  They ask documentary, analytic and predictive questions about PSI investigations (FLEX, ACME,
  BASS, and related CIR/MSG experiments) and must defend every answer with sources.
- **Secondary:** NASA Space Apps judges, who see the same workflow in a short demo *(inferred:
  demo shows the researcher workflow, not a separate demo mode)*.
- **Devices:** desktop and laptop first, keyboard and mouse *(inferred)*; phone stays readable.

## Product Purpose

IGNIS is an SI-powered research dashboard that finds, summarizes, ranks, compares and interprets
NASA PSI combustion investigations to give fire-safety insight for crewed spacecraft, stations,
lunar and Martian bases and pressurized rovers. Success: a researcher reaches a defensible,
cited answer (or an exact statement of what evidence is missing) faster than by reading PSI by
hand, and can verify every claim at its source.

## Positioning

Every claim is bound to an exact source passage, data row or model artifact, typed by what it
asserts (source reports, observed, computed, predicted, interpretation). Forecasts are made from a
causal prefix and checked against a sealed continuation. Missing, conflicting and synthetic data
are shown as such, never as zero or as NASA results.

## Operating Context

- Challenge objectives (Space Apps 2026, "Flame in Freefall"): identify investigations relevant to
  habitat fire safety; extract data that supports predictive modeling of flammability; highlight
  research for prevention, detection and suppression equipment; optional Earth applications.
- Attributes the challenge asks to summarize: objectives; fuels (sample/burner shape and size);
  conditions (oxygen concentration, airflow, pressure); how tests in one investigation differ;
  sensors and measurement types; similarities and differences across investigations; hardware
  (CIR/MDCA, MSG/SPICE flow duct, ACME modules).
- Reference atmospheres: Earth 21% O2 at 14.7 psi; ISS ~21% at 14.7 psi; spacesuit 100% at
  4.3 psi; planned Moon base / rovers ~30% or more at 10.2 psi or less. No partial-gravity data in
  PSI yet (LUCI, FM2 expected).
- Judging criteria: impact, creativity, validity, relevance, presentation.
- Research runtime: an LLM agent (Codex App Server) runs an RLM-style loop with IGNIS tools over
  MCP; the UI follows an ordered, resumable event log (SSE) per episode.

## Capabilities and Constraints

- Contracts live in `crates/ignis-core/src/contracts/` and reach the UI only through
  `web/src/contracts/generated.ts`; the UI never hand-writes a cross-lane type.
- Episodes, events (`message.*`, `tool.*`, `job.updated`, `artifact.created`, `approval.*`,
  `answer.completed`, `runtime.opaque`), `ResearchAnswer` (blocks, typed claims, evidence,
  artifacts, limitations), `EvidenceRef` + `SourceLocator` (page, bbox, row/column, frame).
- Three data modes: browser mock (C0 fixtures), server fixtures, live. Synthetic origin must be
  unmistakable everywhere.
- Not yet in contracts: tool-call arguments, model rationales ("why"), entity graph, document
  classification labels. The UI shows these gaps; it never fabricates them.
- Legacy routes: experiments registry, compare, forecast with seal/reveal.

## Brand Commitments

Name: IGNIS. No logo asset exists. The user requires Linear / Slack / Vercel-grade navigation and
craft, one settled palette, and colour used on principle, not decoration.

## Evidence on Hand

- C0 contract fixtures (`fixtures/contracts/`), a 4-run dev fixture, and a local 35-run synthetic
  demo fixture (`runs/demo-fixture/`, git-ignored). No model trained on NASA data yet. No real
  user testimonials or benchmarks: none may be invented.

## Product Principles

1. Every number is one step from its source.
2. Honesty without noise: evidence class and synthetic origin are always visible, compactly.
3. No dead ends: a refusal or gap says why and what to do next.
4. The scientist leaves with something they can cite, export or continue.
5. Speed and keyboard reach for experts; visible controls for newcomers.

## Accessibility & Inclusion

WCAG 2.1 AA *(inferred standard)*: keyboard-complete navigation, visible focus, no meaning by
colour alone, reduced-motion support.
