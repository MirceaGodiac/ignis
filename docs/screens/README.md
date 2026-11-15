# IGNIS web app: screen guide

This guide describes every screen of the IGNIS web app (`web/`), top down: first the application
shell that frames every page, then each section of the left rail in the order it appears, then the
Questions screens, the NASA Original viewer and Home. Each screen page has the same parts: route and
URL parameters, purpose, a layout walkthrough, the data and where it comes from, user flows, links to
other screens, what the UI states about the data, and developer notes.

The live staging UI is at <https://ignis-ardealu.app>. Everything here is read from the code under
[`web/src`](../../web/src); where a screen computes something in the browser, the page says how.

## How the app is organised

IGNIS is a single-page React app with hash routes (`#/runs`, `#/lab/<run>`, ...), so every view is
addressable, reloadable and shareable without a server route
([`shell/router.ts`](../../web/src/shell/router.ts)). The rail groups pages by what a researcher does:
ask; look at what NASA recorded; work with models on it.

| Rail section | Screen | Route | Page |
|---|---|---|---|
| (top) | Home | `#/` | [home.md](home.md) |
| NASA data | Sources | `#/sources[?q=]` | [sources.md](sources.md) |
| NASA data | Runs | `#/runs[?family=&habitat=&inv=]` | [runs.md](runs.md) |
| NASA data | Conditions: By atmosphere | `#/conditions[?habitat=]` | [conditions.md](conditions.md) |
| NASA data | Conditions: Atmosphere map | `#/conditions?tab=map[&inv=]` | [atmosphere-map.md](atmosphere-map.md) |
| Models | Forecast lab | `#/lab[/<run id>]` | [forecast-lab.md](forecast-lab.md) |
| Models | Compare | `#/compare` | [compare.md](compare.md) |
| Models | Models | `#/models` | [models.md](models.md) |
| Recent questions | Questions (history) | `#/research` | [questions.md](questions.md) |
| Recent questions | One question (answer) | `#/research/<episode id>` | [answer.md](answer.md) |
| (new tab) | NASA Original viewer | `#/original/<source id>?page=&bbox=&row=&col=&t=&path=&title=` | [original.md](original.md) |

Two legacy routes redirect: `#/map` opens `#/conditions?tab=map`, and `#/conditions?tab=runs`
opens `#/runs` (keeping `habitat`) ([`App.tsx`](../../web/src/App.tsx)).

## 1. Application shell

[shell.md](shell.md) covers what surrounds every page:

- **Sidebar (left rail)**: the Ignis wordmark, **New question**, **Search** (Ctrl K / Cmd K), the
  page links grouped under *NASA data* and *Models* with counts, **Recent questions** grouped by day
  with a per-question menu (rename, pin, copy link, remove), and the **theme switch**
  (system, light, dark).
- **Top bar and breadcrumbs**: section, then page, then the selected item.
- **Command menu**: jump to any page, question or run by name; switch between live and synthetic data.
- **Side panel**: the details pane that opens on the right for a run, a source passage, a claim, a
  figure, a step of the agent, a model release, an investigation or a NASA original, with a back trail.
- **Data source**: live (the deployment's own data) or synthetic (development fixtures).

## 2. NASA data

- [Sources](sources.md): full-text and hybrid search over the NASA documents IGNIS holds, with each
  hit opened at its page in the original beside the result list.
- [Runs](runs.md): the run index, one row per NASA run, sortable by family, investigation, date,
  fuel, run name, oxygen, pressure and flow; filter by family, text, crew atmosphere and
  investigation; select runs for Compare.
- [Conditions, By atmosphere](conditions.md): for a chosen crew atmosphere (Moon base and rovers,
  Spacesuit, ISS and Earth), which investigations tested inside it, which came nearest and by how
  much, and which cannot be placed from NASA's tables.
- [Conditions, Atmosphere map](atmosphere-map.md): every test point on an oxygen-pressure map with
  the crew atmospheres drawn on it, zoomable, filterable by investigation, with a nearest-tests list
  and a CSV export.

## 3. Models

- [Forecast lab](forecast-lab.md): pick a run (or type conditions), let the model written for it
  forecast from the first seconds, read the forecast with its uncertainty, and for fitted releases
  seal the prediction and reveal the recorded continuation.
- [Compare](compare.md): selected runs side by side, with the server's verdict on whether a numeric
  comparison of a quantity is permitted.
- [Models](models.md): the model releases the research service serves, with their state, weights and
  construction.

## 4. Questions (research episodes)

- [Questions](questions.md): the history of every question asked, searchable, filterable by status,
  grouped by day.
- [One question and its answer](answer.md): the conversation with the agent: the steps trail, the
  answer with citation chips, claims, limits, figures and embedded views, the follow-ups, the sources
  panel and the composer.

## 5. NASA Original viewer

[original.md](original.md): a NASA original opened at the cited place in its own browser tab: the
PDF page with the cited region marked, the video at the cited time, or the table record with its
neighbours.

## 6. Home

[home.md](home.md): the ask box with suggested questions, and the ways into the data: runs by
family, crew atmospheres, shortcuts and the questions to continue.

## Screenshots

Screenshots live in [`docs/images/`](../images/): [runs](../images/runs.png),
[conditions](../images/conditions.png), [atmosphere map](../images/atmosphere-map.png),
[forecast lab](../images/forecast-lab.png), [compare](../images/compare.png),
[models](../images/models.png), [a research answer](../images/research-answer.png).

## API surfaces the screens read

The web app talks to two server surfaces (see [`api.ts`](../../web/src/api.ts) and
[`research/http.ts`](../../web/src/research/http.ts)):

| Surface | Base (live) | Base (synthetic) | Used by |
|---|---|---|---|
| Run registry | `/api` | `/api/fixtures` | Runs, Compare, Forecast lab (fitted releases), run side panel, Home run counts |
| Research runtime | `/api/v1` | `/api/fixtures/v1`, or an in-browser replay ("mock") | Questions, answers, Sources, Original, Conditions, Models, hand-written models in the Forecast lab |

| Endpoint | Screen |
|---|---|
| `GET /api/health` | sidebar (whether the registry is served) |
| `GET /api/experiments` | Runs, Home, Forecast lab run picker, command menu |
| `GET /api/experiments?split=&partition=` | Forecast lab (partition of a run) |
| `GET /api/experiments/{id}` | run side panel, Forecast lab |
| `GET /api/experiments/{id}/observations?cutoff_s=` | Forecast lab |
| `POST /api/forecast`, `GET /api/jobs/{id}` | Forecast lab (fitted releases) |
| `POST /api/acquisition`, `/api/seal`, `/api/reveal` | Forecast lab steps 3 to 5 |
| `POST /api/compare` | Compare |
| `GET /api/v1/capabilities` | every page that depends on a service |
| `GET/POST /api/v1/episodes`, `GET /api/v1/episodes/{id}`, `GET .../events?after=` (SSE), `POST .../messages`, `POST .../{pause,resume,cancel}`, `POST .../approvals/{id}` | Questions, answer |
| `GET /api/v1/artifacts/{id}[?pointer=]` | answer, side panel |
| `POST /api/v1/search` | Sources |
| `GET /api/v1/sources/{id}/pages/{n}`, `GET /api/v1/sources/{id}/content` | side panel, Sources, Original |
| `POST /api/v1/conditions` | Conditions (both tabs), answer views |
| `GET /api/v1/models`, `GET /api/v1/models/{id}` | Models, answer release cards, Forecast lab |
| `GET /api/v1/model-cards`, `POST /api/v1/models/{id}/scenarios`, `POST /api/v1/analyses`, `GET /api/v1/jobs/{id}` | Forecast lab (hand-written models) |

Contract types come from [`contracts/generated.ts`](../../web/src/contracts/generated.ts), generated
from `schemas/`; the registry envelopes are declared in [`api.ts`](../../web/src/api.ts).
