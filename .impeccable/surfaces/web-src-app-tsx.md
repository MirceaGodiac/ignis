---
version: 1
slug: "web-src-app-tsx"
primary_target: "web/src/App.tsx"
related_targets: []
---

# IGNIS research workspace (web app shell + all workspaces)

Scope: the whole dashboard (`web/src`). Mode: Operate.
Audience and job: combustion researchers and engineers turning NASA PSI microgravity-combustion investigations into defensible fire-safety answers (Space Apps 2026 "Flame in Freefall").
Constraints: typed contracts from generated.ts only; synthetic origin always visible (one neutral badge, never a coloured banner); observed / model output / revealed / cutoff stay distinct; refusals typed and visible; no partial-gravity data exists.
Unresolved: tool-call arguments, model rationales and entity graph are not in contracts yet (UI shows the gap).
Interview substitution: user asked not to be asked. User redirected twice: no yellow, compact, minimal, follow Harvey / Legora / Hebbia, use the taste skill's palettes, and a harness that leads the researcher.

## Direction contract

THESIS: A quiet professional research harness in the Harvey / Legora / Hebbia mould: the agent does the work and proposes the next step, the researcher reads, checks and decides. It refuses dashboards of tiles, decorative colour and pages the researcher must drive step by step.

OWN-WORLD: "Sheet and biro" (white, #111 ink, one grey, ink-blue #1a4be0) in light, "Night and sky blue" in dark (taste-skill software set); Geist and Geist Mono; colour spent only on data (blue = model output and selection, orange = revealed data, red = refusal); shape carries evidence class.

STORY: Ask one question on Home; the investigation page streams the agent's route, then a cited answer with keyed claims; the harness lists next steps derived from the answer (read the deciding passage, test the forecast on the run it used, compare its runs, resolve a stated limit, support an unsupported claim); every claim, source, step and run opens in the right panel with a back trail.

FIRST VIEWPORT: Left rail: data-source switcher, New investigation and search, tools (Home, Runs, Atmosphere map, Forecast lab, Sources, Models), investigations with the open one expanded into Thread / Evidence / Route. Home centre: one heading, one prompt, four starting questions, Continue list. Investigation: title, tabs, thread with margin claims and Next steps, Properties column right.

FORM: research-derived structure (Legora sidebar and prompt, Harvey object page with tabs and properties, Hebbia/Legora evidence grid with side panel), seed:a5ad1abf1bc19ec1 (original roll superseded by the user's explicit redirection; quoted: "follow Harvey, Legora, Hebbia", "professional mature sleek colours", "a minimal feeling harness that helps the researcher through itself").

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
