# IGNIS web dashboard

React 18 + TypeScript + [uPlot](https://github.com/leeoniya/uPlot) client for the IGNIS API, built with
Vite. Three workspaces:

| Workspace | What it does |
|---|---|
| **Explore** | browse experiments by family, inspect observations and evidence search results |
| **Compare** | put several runs side by side (`/api/compare`) |
| **Experiment Lab** | choose a release and a cutoff, forecast from the prefix, list candidate next observations, seal, then reveal the continuation |

A switch selects the data origin: the **registry** (`/api`, real records after `ignis data ingest`) or
the **development fixture** (`/api/fixtures`, synthetic, shown with a fixture banner; available only
when the server runs with `--dev`). Refusals from the API are typed values shown in the UI, not
swallowed errors.

## Run

```bash
npm ci --prefix web
ignis serve --dev                  # terminal 1: API on 127.0.0.1:8787
npm --prefix web run dev           # terminal 2: http://localhost:5173 (proxies /api → IGNIS_PORT, default 8787)
```

Or use the `ignis-web` and `ignis-server` entries in [`.claude/launch.json`](../.claude/launch.json).

## Scripts

| Script | Does |
|---|---|
| `npm run dev` | Vite dev server (port `PORT`, default 5173) |
| `npm run build` | typecheck + production build to `web/dist` (served by `ignis serve` at `/`) |
| `npm run typecheck` | `tsc --noEmit` |
| `npm run gen:types` | regenerate `src/contracts/generated.ts` from `../schemas/*.json` |

## Layout

```text
src/
├── App.tsx                origin switch, tabs
├── api.ts                 typed client: endpoints, envelopes, RefusalError
├── hooks.ts               react-query hooks
├── contracts/generated.ts generated from schemas/ (do not edit by hand)
├── components/            ForecastChart (observed points, quantile bands, revealed continuation), Panels
└── workspaces/            Explore, Compare, Lab
```

After a contract change in `ignis-core`: `ignis schemas export`, then `npm --prefix web run gen:types`.

See also: [ignis-server](../crates/ignis-server/README.md) · [GUIDE §11](../docs/GUIDE.md#11-api-and-dashboard)
