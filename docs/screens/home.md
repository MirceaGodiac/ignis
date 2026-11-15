# Home

[Back to the screen guide](README.md)

## Route

`#/` (also any unknown route). No parameters. The sidebar's **New question** and the wordmark lead
here.

## Purpose

Home is where a question is asked and where each way into NASA's data starts. Under the ask box it
shows the runs by investigation family, the three crew atmospheres, shortcuts to the main tools, and
the latest questions to continue. Every tile opens the target page already filtered.

## Layout

1. **Heading**: "What do you want to find out?"
2. **Ask box** ([`views/AskBox.tsx`](../../web/src/views/AskBox.tsx)): a two-line field with the
   placeholder "Ask IGNIS", focused on load. Enter sends; Shift+Enter adds a line. The arrow button
   (**Ask**) is disabled while the field is empty.
3. **Suggested questions**: four pills that ask at once:
   - Which tests come closest to Moon-base air?
   - How does oxygen change FLEX burning rates?
   - What did BASS find about flame spread at low airflow?
   - Which data could feed a flammability model?
4. **NASA runs** (shown once runs are loaded): heading with **All N runs ->** (to `#/runs`), then
   one tile per family in the order FLEX, ACME normal, ACME inverse, BASS, Other (only families
   present). Each tile shows the family mark and name, the run count, and up to three fuels or
   materials, most frequent first. A tile opens `#/runs?family=<family>`.
5. **Crew atmospheres**: heading with **Atmosphere map ->** (to `#/conditions?tab=map`), then one
   tile per atmosphere: *Moon base and rovers* (>= 30 % O2, <= 10.2 psi), *Spacesuit* (100 % O2,
   4.3 psi), *ISS and Earth* (21 % O2, 14.7 psi), each labelled "Runs tested here". A tile opens
   `#/runs?habitat=<moon|suit|iss>`.
6. **Go to**: four shortcuts: *Search NASA documents* (Sources), *Habitat relevance* (Conditions),
   *Forecast a run* (Forecast lab), *Compare runs* (Compare).
7. **Continue** (only when questions exist): heading with **All questions ->**, then cards for the
   four most recent questions, each with its goal and last update time (`YYYY-MM-DD HH:MM`).

## Data

| Element | Source |
|---|---|
| Family tiles | `GET /api/experiments` (`ExperimentList.experiments`), counted by `family`; fuels by `fuel_or_material` |
| Atmosphere tiles | fixed definitions `HABITAT_LIST` in [`research/habitats.ts`](../../web/src/research/habitats.ts) |
| Continue cards | `GET /api/v1/episodes`, first four |

## Flows

**Ask a question**

1. Type the question in the ask box (or click a suggested question).
2. Press Enter. The app goes to `#/research?new=1&q=<question>`.
3. The Questions page creates the episode (`POST /api/v1/episodes`), posts the question as its first
   message (`POST .../messages`) and opens `#/research/<episode id>`, where the answer streams in.
   See [questions.md](questions.md) and [answer.md](answer.md).

**Browse runs of one family**: click a family tile; Runs opens filtered to it.

**Find runs tested in a crew atmosphere**: click an atmosphere tile; Runs opens with that atmosphere
filter on (recorded ambient O2 and pressure inside the atmosphere).

## Links

Runs, Conditions (map tab), Sources, Forecast lab, Compare, Questions, one question.

## Developer notes

- Component: [`views/Home.tsx`](../../web/src/views/Home.tsx). Suggested questions are `EXAMPLES` in
  [`views/AskBox.tsx`](../../web/src/views/AskBox.tsx); the family order is `FAMILY_ORDER`.
- To add a shortcut, extend the array in the *Go to* section.
