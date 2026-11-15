# Application shell

[Back to the screen guide](README.md)

## Routes

The shell frames every route. Its own state lives in the URL hash (see
[`shell/router.ts`](../../web/src/shell/router.ts)):

| Hash | View |
|---|---|
| `#/` | Home |
| `#/research[/<episode>]` | Questions, one question |
| `#/sources[?q=]` | Sources |
| `#/runs[?family=&habitat=&inv=]` | Runs |
| `#/conditions[?tab=map&habitat=&inv=]` | Conditions |
| `#/compare` | Compare |
| `#/lab[/<run id>]` | Forecast lab |
| `#/models` | Models |
| `#/original/<source id>?...` | NASA Original viewer |

An unknown first segment opens Home. `#/map` redirects to `#/conditions?tab=map`;
`#/conditions?tab=runs` redirects to `#/runs` with its `habitat`.

One query parameter outside the hash is read: `?source=synthetic` (before the `#`) starts the app on
synthetic data ([`shell/data.tsx`](../../web/src/shell/data.tsx)).

## Purpose

The shell gives every page the same frame: a left rail for where to go, a top bar that says where you
are, the page in the centre, and a side panel on the right for the details of whatever you selected
(why a claim holds, where a value comes from). A command menu reaches any page, question or run from
the keyboard.

## Layout

### Sidebar (left rail)

From top to bottom ([`shell/Sidebar.tsx`](../../web/src/shell/Sidebar.tsx)):

1. **Ignis wordmark**: links to Home (`#/`).
2. **Menu button** (narrow screens only): opens and closes the rail as an overlay. Choosing any link
   closes it.
3. **New question** (primary button): links to Home, where questions are asked.
4. **Search** (magnifier icon, tooltip "Search Ctrl K"): opens the command menu.
5. **Home** link.
6. **NASA data** section: **Sources**, **Runs**, **Conditions**. *Runs* shows a count badge with the
   number of runs in the registry (`ExperimentList.count`). *Conditions* is highlighted on both of its
   tabs.
7. **Models** section: **Forecast lab**, **Compare**, **Models**. *Compare* shows a count badge with
   the number of runs selected for comparison (0 hides it).
8. **Recent questions** (only when at least one question exists): see below.
9. **Theme switch** (footer): three segments, *System theme*, *Light theme*, *Dark theme*.

The current page link carries `aria-current="page"`. A page whose backing service this deployment
does not serve stays reachable but is drawn dimmed, with the tooltip "Not available on this server".
The service of each page is `VIEW_SERVICE` in the router: Sources and Original read `knowledge`,
Conditions reads `datasets`, Models reads `models`, Questions reads `agent_runtime`, and Runs,
Compare and Forecast lab read the run `registry`. A page is dimmed only on a definite answer:
`GET /api/v1/capabilities` reports the service `available: false`, or (for the registry)
`GET /api/health` failed with no data.

### Recent questions

[`shell/ChatList.tsx`](../../web/src/shell/ChatList.tsx), [`shell/chatPrefs.ts`](../../web/src/shell/chatPrefs.ts).

- The **Recent questions** heading links to the full history (`#/research`).
- Questions are grouped: **Pinned** first, then by last activity (`updated_at`, else `created_at`):
  **Today**, **Yesterday**, **Previous 7 days**, **Previous 30 days**, **Older**.
- Up to 30 unpinned questions are listed; pinned ones always show. When more exist, a final row
  **All N questions** links to `#/research`.
- Each row shows a status mark and the question's name (its goal, or the name you gave it). Active
  questions show a dot: a running dot for `ready` and `running`, an attention dot for
  `waiting_tools`, `waiting_approval` and `waiting_user`. Finished ones show a thread icon. The
  tooltip shows the original goal (and your name for it, when renamed).
- **Double-click** a row to rename it inline.
- The **...** button opens a menu:
  - **Rename**: inline field, at most 120 characters. Enter or leaving the field saves, Escape
    cancels; an empty name restores the original question.
  - **Use the original question** (only when renamed).
  - **Pin to top** / **Unpin**.
  - **Copy link**: copies the full URL of `#/research/<episode id>`.
  - **Remove from list**: hides the question from the rail. A status bar **Removed from this list**
    with **Undo** shows for 3 seconds. Afterwards a **Show N removed** button brings every removed
    question back.
- The menu opens upward when it would pass the bottom of the rail, and closes on Escape or a click
  outside. Opened from the keyboard, focus moves into the menu.

Names, pins and removals are this browser's own (localStorage key `ignis.chats`), shared across tabs
through the `storage` event. The server keeps every question; removing one only hides it here.

### Theme switch

[`shell/theme.ts`](../../web/src/shell/theme.ts). *System* follows `prefers-color-scheme`; *Light*
and *Dark* pin the scheme by setting `<html data-theme>`. The choice is kept in localStorage
(`ignis.theme`). Charts re-render on a change through `useScheme()`.

### Top bar and breadcrumbs

[`App.tsx`](../../web/src/App.tsx). Shown on every page except Home and an open question (which draws
its own bar). The breadcrumb reads *section > page*, for example *NASA data > Runs* or
*Models > Compare*. In the Forecast lab with a run open it reads
*Models > Forecast lab > FLEX-093 · n-heptane* (the run's label from `runLabel`), and *Forecast lab*
links back to the lab. The window title is `<page> · IGNIS`, or `<question goal> · IGNIS` on an
open question.

### Command menu (Ctrl K / Cmd K)

[`shell/CommandMenu.tsx`](../../web/src/shell/CommandMenu.tsx). Toggled by Ctrl K (Cmd K on macOS)
anywhere, or by the Search button.

- A search field: "Search views, runs and investigations...".
- Results grouped as **Actions** (*New question*; *Switch to synthetic data* or *Switch to live
  data*), **Go to** (every rail page), **Questions** (every question, with its status as a hint) and
  **Runs** (every run, with family and fuel as a hint).
- With an empty query the Runs group is left out; typing matches label, hint and group. At most 60
  results show.
- Arrow keys move the selection, Enter runs it, Escape closes. A click outside closes it. Focus
  returns to the control that opened it.
- Picking a run opens it in the side panel; picking a question opens it.

### Side panel

[`shell/Panel.tsx`](../../web/src/shell/Panel.tsx), [`shell/inspector.tsx`](../../web/src/shell/inspector.tsx).

The panel opens on the right when you select something on a page. It keeps a trail: a link inside the
panel pushes a new item, and the **Back** arrow (tooltip "Back to `<previous>`") returns. **Close**
(the X, or Escape) closes it and returns focus to the control that opened it. Changing page closes
it. Item kinds:

| Item | Opened from | Shows |
|---|---|---|
| **Run** | Runs row, map, Compare header, command menu, answer figure | family tag; fuel, configuration, investigation; **Add to compare** / **Remove from compare**; **Forecast** (when the run has a fitted release); **Ask about this run** (starts a question naming the run by NASA's alias); *Conditions* (named, in display units, from `conditionRows`); *Gravity*; *NASA originals* (every file the record cites, each with **Original ↗**); *Measured series* (points and time span per quantity); *Models* (releases with state); *Not available* (registry gaps in plain words). Source: `GET /api/experiments/{id}`. |
| **Source** (evidence) | citation chip, search hit, step result | evidence class tag; the quote, or the cited span cut from the parsed page; *Where it is* (source id, parse view, locator, character span and unit, asset hash); the original (PDF page with region, or video at time) through `SourceAccess`; *Parsed text* of the page with the cited region boxed. |
| **Claim** | claim reference | claim kind tag and text with a one-line note on what that kind needs; *Support* (each cited passage, or the value read from an artifact field); *Support check* (verdict, method, model, confidence, when checked). |
| **Figure / Artifact** | answer figure, step output | contract or *Model output* tag; typed view for known contracts (execution record, dataset bundle, evaluation, analysis, video contours, answer refusal); id, size, media type, created, job, release, sha256; *Computed from* (runs link to the run item); *Payload* as JSON. |
| **Step** | the steps trail of an answer | the step as a sentence; for a search: query, extra terms, filters, the passages returned (each opens its source); for a read: the source, range, **Open the original at page N**, the text read; for a conditions lookup: what was asked and how many runs came back; job status; outputs; *Why, in the agent's words*; *Technical details* (tool, call id, event, execution and scientific status, origin, contract, error, arguments, result). |
| **Model release** | Models row, answer release card | state tag; family, trained on, lineage, features, baseline, id; *Construction* (method, uncertainty kind, author, forward, tensors), *Build checks*, *What the author saw*, *Hashes*. |
| **Sources** | **Sources N** button on a question | every source the question's answers cite, numbered in order of first citation, each passage opening its evidence item. |
| **Investigation** | **About** on a Conditions card | the atlas card: title, id, sub-area, hardware; **On the map**, **Documents**; *Fire safety* needs with NASA's words; *Why it matters*; *How the tests differ*; *Findings* with their sources; *Instruments*; *Not in the documents*; *Related* investigations. |
| **NASA original** | **Open the original** in a read step | the original inside the panel, with **Originals tab ↗** and **Open at NASA ↗**. |

## Data source: live and synthetic

[`shell/data.tsx`](../../web/src/shell/data.tsx). The app reads one data source at a time:

- **Live** (default): the registry at `/api` and the research runtime at `/api/v1`.
- **Synthetic**: the fixture registry at `/api/fixtures` and the research fixtures at
  `/api/fixtures/v1` when the server runs with `--dev`; otherwise the fixtures are replayed in the
  browser ("mock" mode, read only).

Switch with the command menu (*Switch to synthetic data* / *Switch to live data*) or with
`?source=synthetic`. The choice is kept per browser (`ignis.source`). Every query key includes the
source, the Compare selection is cleared on a switch, and items from fixtures carry a **Synthetic**
tag.

The run list and the reduced conditions view are kept in IndexedDB (`ignis-cache`) so a returning
visit draws at once while a fresh copy loads ([`shell/persist.ts`](../../web/src/shell/persist.ts)).
The registry is retried with back-off (4 retries, up to 8 s) and refetched when the tab regains focus.

## Errors shown in place

[`ui/common.tsx`](../../web/src/ui/common.tsx): a server refusal or error is shown as a note with its
code, HTTP status, message and retry advice ("Do not retry.", "Retry after N s.", ...). A 401
`scope_denied` "not authenticated" from the origin shows **Sign in again** (Cloudflare Access logout).
After a deploy, a tab on the previous build reloads itself once when a lazy chunk fails to load
([`main.tsx`](../../web/src/main.tsx)).

## Developer notes

- Add a page: add the view to `View`, `VIEWS`, `VIEW_TITLE`, `VIEW_SERVICE` and a `SECTIONS` group in
  [`router.ts`](../../web/src/shell/router.ts), an icon in `ICON` in
  [`Sidebar.tsx`](../../web/src/shell/Sidebar.tsx), and the render line in
  [`App.tsx`](../../web/src/App.tsx). The rail, breadcrumbs, command menu and window title read these
  tables.
- Add a side panel item: extend `PanelItem` and `itemKey` in
  [`inspector.tsx`](../../web/src/shell/inspector.tsx), then `TITLE`, `itemTitle` and the body switch
  in [`Panel.tsx`](../../web/src/shell/Panel.tsx). Open it with `usePanel().open(item)` from a page, or
  `push(item)` from inside the panel.
- Dev builds expose `window.__ignisPanel.open(item)` to open any panel item from the console.
- Shared queries: `useEpisodes()` and `useReleases()` in
  [`shell/queries.ts`](../../web/src/shell/queries.ts); `useData()` gives `client`, `origin`,
  `experiments`, `caps`, `health`.
