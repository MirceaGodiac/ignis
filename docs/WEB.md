# IGNIS web application

| | |
|---|---|
| **Audience** | Engineers working on `web/` |
| **Stack** | React 18, TypeScript (strict), Vite 6, TanStack Query 5, uPlot (charts), PDF.js (originals) |
| **Served by** | the edge Worker `ignis-edge` in production ([`docs/DEPLOYMENT.md`](DEPLOYMENT.md)); `ignis serve` serves `web/dist` at `/` locally when it exists |
| **Design system** | [`DESIGN.md`](../DESIGN.md) ("Graphite and Ember") |
| **Related** | [`web/HARNESS_CONTRACT.md`](../web/HARNESS_CONTRACT.md), [`crates/ignis-server`](../crates/ignis-server/) (`/api/v1`), [`CONTRIBUTING.md`](../CONTRIBUTING.md) |

`web/README.md` predates the current shell (it describes an older Explore / Compare / Lab layout and
a `src/workspaces/` directory that no longer exists). This document describes the code as it is.

---

## 1. Source layout

```text
web/
├── index.html              theme pinned before first paint; Hanken Grotesk + Geist Mono from Google Fonts
├── vite.config.ts          dev server, /api proxy, fs allow for ../fixtures/contracts
├── public/data/            static tables fetched at runtime (e.g. bass5_inputs.json for the forecast lab)
├── scripts/gen-types.mjs   schemas/*.json -> src/contracts/generated.ts
├── tests/                  node --test suites (*.test.ts)
└── src/
    ├── main.tsx            React root, QueryClient, reload-once on stale lazy chunks after a deploy
    ├── App.tsx             the shell: rail, top bar, centre view, side panel, command menu
    ├── api.ts              legacy registry client (/api, /api/fixtures)
    ├── hooks.ts, format.ts shared hooks and formatting
    ├── styles.css          tokens and global styles (views add lab.css, trail.css)
    ├── contracts/          generated.ts (generated; never edit by hand)
    ├── research/           research client (mock / fixtures / live), SSE, event log, units, view specs
    ├── shell/              router, data provider, sidebar, side panel, command menu, theme, persistence
    ├── views/              one component per page, plus answer/trail rendering
    ├── components/         ForecastChart, SeriesChart (uPlot)
    ├── data/               investigations.json (static investigation metadata)
    └── ui/common.tsx       small shared UI primitives
```

---

## 2. Shell

`App.tsx` composes three providers and the shell:

| Piece | File | Role |
|---|---|---|
| `DataProvider` | `shell/data.tsx` | one data source for the whole app (§4) |
| `CompareProvider` | `shell/compare.tsx` | the run selection shared by Runs, the map and Compare |
| `PanelProvider` | `shell/inspector.tsx` | the side panel's stack of items (run, claim, evidence, artifact, step, ...) with a back trail |
| `Sidebar` | `shell/Sidebar.tsx` | wordmark, new question, search, sections from `SECTIONS`, recent questions (`ChatList.tsx`), theme switch |
| Top bar | `App.tsx` | breadcrumbs "section > page"; hidden on Home and inside a question |
| `Panel` | `shell/Panel.tsx` | the right side panel; Escape closes it; it closes on every view change |
| `CommandMenu` | `shell/CommandMenu.tsx` | Ctrl/Cmd+K: every view, run, investigation, source, and the live/synthetic switch |

The window title is `<page title or question goal> · IGNIS`.

### 2.1 Information architecture

`shell/router.ts` is the single registry for the rail, breadcrumbs, command menu and window title:

| Section | Views | Service each view reads (`VIEW_SERVICE`) |
|---|---|---|
| (none) | Home | none |
| NASA data | Sources, Runs, Conditions | knowledge, registry, datasets |
| Models | Forecast lab, Compare, Models | registry, registry, models |
| (rail, Recent questions) | Questions (`research`) | agent runtime |

A page whose service the deployment does not serve stays reachable, is dimmed in the rail, and says
so in place (from `/api/v1/capabilities`). There is no global service banner.

---

## 3. Routing

Hash routes, so every view is addressable, reloadable and shareable without a server route (the
Worker's SPA fallback serves `index.html` for every non-API path). Parsing and building live in
`shell/router.ts` (`parse`, `href`, `go`, `useRoute`); an unknown view falls back to Home.

| Route | View | Component |
|---|---|---|
| `#/` | Home: ask, suggested questions, ways into the data | `views/Home.tsx` |
| `#/research` | Questions; `?q=<text>` prefills, `?new=1` starts a new one | `views/Research.tsx` |
| `#/research/<episode id>` | one question as a conversation; `?tab=thread\|evidence\|route` | `views/Research.tsx` |
| `#/runs` | run index; `?family=`, `?habitat=`, `?inv=` | `views/Runs.tsx` |
| `#/conditions` | Conditions by crew atmosphere; `?tab=map` for the atmosphere map; `?habitat=`, `?inv=` | `views/Conditions.tsx` (+ `ByAtmosphere`, `MapView`, `ConditionsMap`) |
| `#/map` | old link: redirects to `#/conditions?tab=map` | (redirect in `App.tsx`) |
| `#/conditions?tab=runs` | old link: redirects to `#/runs` (keeps `habitat`) | (redirect in `App.tsx`) |
| `#/compare` | side-by-side conditions for the selected runs | `views/Compare.tsx` |
| `#/lab` | forecast lab: pick a run | `views/Lab.tsx` (+ `RunPicker`, `LabModels`) |
| `#/lab/<run id>` | forecast lab on one run | `views/Lab.tsx` |
| `#/sources` | source search; `?q=` | `views/Sources.tsx` |
| `#/models` | model releases and their qualification state | `views/Models.tsx` |
| `#/original/<source id>` | a NASA original at a locator, opened in a new tab; `?page=&bbox=x0,y0,x1,y1&row=&col=&t=&title=` | `views/Original.tsx` |

Route parsing is covered by `web/tests/route.test.ts`.

### 3.1 Views at a glance

| File | What it renders |
|---|---|
| `Home.tsx`, `AskBox.tsx` | the prompt and entry strips (runs by family, crew atmospheres, recent questions) |
| `Research.tsx`, `turns.ts` | a question: messages, agent steps grouped into turns from the ordered event log, answers, composer |
| `Answer.tsx` | a `ResearchAnswer`: text with citation chips, quoted passages on hover, sources, follow-ups |
| `AnswerViews.tsx` | registry for `AnswerBlock::View` (closed `ViewSpec` catalog); refuses views that reference uncited data |
| `Sources.tsx` | search over the pinned corpus snapshot, with the original at the hit's page beside the list |
| `Original.tsx` | the PDF page (or table cell, or video time) behind a locator, bytes via `SourceAccess` |
| `Runs.tsx`, `FamilyLegend.tsx` | one row per run, every recorded condition as a sortable column |
| `Conditions.tsx`, `ByAtmosphere.tsx`, `MapView.tsx`, `ConditionsMap.tsx` | the conditions table by crew atmosphere and on the oxygen/pressure map |
| `Compare.tsx` | selected runs side by side; the server decides whether a numeric comparison is permitted |
| `Lab.tsx`, `RunPicker.tsx`, `LabModels.tsx` | prefix forecast, seal, reveal; hand-written model releases served from `/api/v1` |
| `Models.tsx` | releases with "not qualified" stated on every experimental row |

---

## 4. Data loading

### 4.1 Source: live or synthetic

`shell/data.tsx` holds one source for the whole app, so a link always resolves in the source it came
from and the two never mix on one screen.

| Source | Chosen by | Registry routes (`api.ts`) | Research client (`research/client.ts`) |
|---|---|---|---|
| **live** (default) | default; command menu | `/api` | `live` mode: `/api/v1` |
| **synthetic** | command menu, or `?source=synthetic` in the URL; remembered in `localStorage` key `ignis.source` | `/api/fixtures` | `fixtures` mode if the server answers `/api/fixtures/v1/capabilities`, otherwise `mock` |

Synthetic data is shown with a neutral outlined pill (DESIGN.md), and its origin is
`synthetic_fixture`; it is never presented as NASA data.

### 4.2 Research client modes

All three modes implement the same `ResearchClient` interface (`research/http.ts`) over the same
routes, with contract types from `contracts/generated.ts`.

| Mode | Origin | Needs | Behaviour |
|---|---|---|---|
| `mock` | in the browser | nothing | replays the C0 fixtures from `fixtures/contracts/` (imported at build time) as real SSE wire text; creating, steering or changing episodes returns a typed `unsupported` error; condition filters are not applied. Test faults (`MockStreamFaults`: dropped connection, 410 on resume, resume overlap, pacing) exercise the resume path. |
| `fixtures` | `/api/fixtures/v1` | `ignis serve --dev` | real server routes over C0 fixture data |
| `live` | `/api/v1` | `ignis serve` with configured services | the research runtime with whatever services the deployment has |

Errors are typed `ErrorEnvelope`s (`ApiError`), shown in the UI, never swallowed. A non-envelope HTTP
error is converted into one that names the likely cause (for example, the server not running with
`--dev`).

### 4.3 Caching and resilience

- TanStack Query defaults: `staleTime` 30 s, no refetch on window focus. Query keys include the
  research mode and replay counter, so a source switch never shows stale rows.
- Registry queries (`/health`, `/experiments`) retry with back-off (up to 4 times, at most 8 s
  apart) and refetch on focus, so the few seconds of a server restart during a deploy do not leave
  Runs empty.
- `shell/persist.ts` keeps a per-browser IndexedDB copy (`ignis-cache`) of the run list and computed
  test points, shown immediately on return while a fresh copy loads. Every read and write may fail;
  the page then waits for the network.
- Theme choice is `localStorage` key `ignis.theme` (`system` when absent), applied in `index.html`
  before first paint.

---

## 5. Design system notes

[`DESIGN.md`](../DESIGN.md) is the reference (tokens in its front matter; rules in the body).
The points that most often matter in review:

- **Colour is data.** Chrome is ink and warm grey. Ember marks model output, the current selection,
  a running state and a cited region; violet is revealed data only; red is a refusal or failure;
  green is complete or qualified. No blue anywhere; caution has no hue.
- **Navigation state is ink, not ember**: wash fill for the current nav item, 2px ink underline for
  the active tab, solid ink primary button.
- **Type**: Hanken Grotesk for all text; Geist Mono only for ids, hashes, paths, times and code.
  Large text is never bold (400 display, 500 titles). Labels are sentence case, never uppercase.
- **Layout**: 256px rail, flexible main column, 400px side panel when an item is open; sticky 60px
  top bar; object pages have a 280px properties column. Every page opens with its 24px title, then
  tabs or a toolbar, then content; no explanatory sentence under a title.
- **Themes**: light (stone paper) and dark (graphite), with a pinned choice on `<html data-theme>`
  read before `prefers-color-scheme`.
- **Agent markup is never rendered.** Answer views come from the closed `ViewSpec` catalog rendered
  by trusted components (`AnswerViews.tsx`); a new kind is a contract change.

---

## 6. Running locally

Prerequisites: Node.js per [`web/.nvmrc`](../web/.nvmrc) (24.18.1), and Rust for the server.

```bash
npm ci --prefix web
```

### 6.1 UI only (mock mode, no server)

```bash
npm --prefix web run dev                 # http://localhost:5173
```

Open `http://localhost:5173/?source=synthetic`. With no server, the synthetic source falls back to
`mock` for research data (questions, answers, sources, the C0 conditions table). The run registry
(`/api/fixtures/experiments`, used by Runs, Compare and the forecast lab) needs a server.

### 6.2 UI against a local server

```bash
cargo run -p ignis-cli -- serve --dev --port 8787   # terminal 1: API on 127.0.0.1:8787, fixtures at /api/fixtures
npm --prefix web run dev                            # terminal 2: Vite on 5173, proxies /api -> 127.0.0.1:$IGNIS_PORT
```

The same two entries exist in [`.claude/launch.json`](../.claude/launch.json) (`ignis-server`,
`ignis-web`), plus `ignis-server-fake-codex` on port 8788 for runtime work without a real model.

### 6.3 Environment variables

The app reads no `VITE_*` variables. The only variables are read by `vite.config.ts` at dev time:

| Variable | Default | Effect |
|---|---|---|
| `PORT` | `5173` | Vite dev server port (`strictPort: false`: the next free port is used if taken) |
| `IGNIS_PORT` | `8787` | port of the local `ignis serve` that `/api` is proxied to |

The dev proxy covers `/api` only. The NTRS download proxy (`/ntrs/*`) exists only on the edge
Worker; the URL an original opens at comes from the server's `SourceAccess` answer, so how an NTRS
original behaves locally depends on what the local server returns there.

---

## 7. Building

```bash
npm --prefix web run build      # tsc --noEmit, then vite build -> web/dist (target es2022, sourcemaps)
npm --prefix web run preview    # serve web/dist locally
```

`web/dist` is what the Worker serves as static assets and what `ignis serve` serves at `/` when
present. In production, the build runs on the VM in a `node:22-bookworm-slim` container at every
deploy ([`docs/DEPLOYMENT.md`](DEPLOYMENT.md) §4.1 step 9).

---

## 8. Contract types

`src/contracts/generated.ts` is generated from `schemas/*.json`, which are exported from the Rust
contracts in `crates/ignis-core`. `web/scripts/gen-types.mjs` merges every type listed in
`schemas/index.json` into one schema (shared `$defs` declared once), compiles it with
`json-schema-to-typescript`, and appends `SCHEMA_VERSION`.

```bash
cargo run -p ignis-cli -- schemas export   # after any contract change
npm --prefix web run gen:types             # regenerate generated.ts
npm --prefix web run check:types           # fails if generated.ts is stale (run before merging)
```

Never hand-write a TypeScript interface for a type that crosses the API; import it from
`generated.ts`.

---

## 9. Tests

```bash
npm --prefix web test           # node --test "tests/**/*.test.ts"
npm --prefix web run typecheck
```

Tests run TypeScript directly with Node's type stripping (no bundler, no test framework), which is
why modules under test such as `research/http.ts` avoid runtime-only constructs like parameter
properties. Suites in `web/tests/`:

| Suite | Covers |
|---|---|
| `research.test.ts`, `live.test.ts` | research client, event log and stream; the live client and episode controls |
| `route.test.ts` | hash route parsing and building |
| `views.test.ts`, `artifacts.test.ts`, `richParts.test.ts` | answer views render only what the answer cites; artifact views; agent text parts (bold, safe links, run names) |
| `sources.test.ts`, `runSources.test.ts` | opening a citation's original from `SourceAccess`; the NASA files a run cites |
| `contours.test.ts`, `habitats.test.ts`, `testpoints.test.ts`, `csvRecords.test.ts` | BASS contours, crew atmospheres, test-point grouping, CSV records |
| `chatPrefs.test.ts` | the chat list: pinned first, grouping by activity, viewer's own names and removals |

Before merging a web change, run `check:types`, `typecheck`, `test` and `build`
([`CONTRIBUTING.md`](../CONTRIBUTING.md) §3).
