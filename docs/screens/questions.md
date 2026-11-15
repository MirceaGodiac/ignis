# Questions

[Back to the screen guide](README.md)

## Routes

| Hash | Behaviour |
|---|---|
| `#/research` | the history of questions |
| `#/research?new=1&q=<question>` | creates a question from `q` and opens it (used by Home and by **Ask about this run**) |
| `#/research?new=1` (no `q`) | redirects to Home, where questions are asked |
| `#/research/<episode id>` | one question: see [answer.md](answer.md) |

## Purpose

Questions is the history of every question asked on this server, by day. It finds a past question by
its words or its status and reopens it. It is also the page that starts a new question handed over
from Home.

## Layout

1. **Heading** "Questions", subtitle "Every question you asked, by day."
2. **Starting note**: while a question from `?q=` is being created, "Starting "`<question>`"...". If
   creation fails, an error note "The question was not sent" with the server's code and message.
3. **Search** field "Search your questions": filters by words in the goal (case-insensitive).
4. **Filter chips** (each with a count): **All**, **Answered** (`completed`), **In progress**
   (`ready`, `running`, `paused`), **Stopped** (`failed`, `cancelled`). A chip whose count is zero is
   hidden, except *All*.
5. **Groups**: Pinned, Today, Yesterday, Previous 7 days, Previous 30 days, Older, the same grouping
   and pins as the rail (removals from the rail do not hide questions here).
6. **Rows**: thread icon, the goal in bold, the status in words (*Answered*, *Working...*, *Ready*,
   *Paused*, *Failed*, *Cancelled*, ...), and the time of last activity, for example "15 Nov, 10:28"
   (the year is added when it is not this year). A row opens the question.

### States

- Loading: "Loading your questions...".
- No questions: "No questions yet. Ask one on Home."
- Nothing matches the search or filter: "No question matches."
- Load error: "Questions could not be loaded" with the server's error.
- Before the data source is ready: "Connecting...".

## Data

`GET /api/v1/episodes` returns `ResearchEpisode[]` (`episode_id`, `goal`, `status`, `created_at`,
`updated_at`, pins, runtime). Grouping is `groupChats` in
[`shell/chatPrefs.ts`](../../web/src/shell/chatPrefs.ts), with this browser's pins.

## Flows

**Start a question from another page**

1. Navigate to `#/research?new=1&q=<text>` (Home's ask box does this).
2. The page calls `POST /api/v1/episodes` with `goal` and an idempotency key (`client_request_id`).
3. If the new episode is `ready` with no events, the page posts the goal as the first message
   (`POST /api/v1/episodes/{id}/messages`), which starts the agent's turn.
4. The new episode is added to the cached list and the page opens `#/research/<id>`.

In mock mode (synthetic data without a server) nothing is created: mock mode replays fixed fixtures.

**Find a past question**: type words in the search field or pick a status chip, then open the row.

## Links

One question ([answer.md](answer.md)); Home. The rail's *Recent questions* heading and *All N
questions* link here.

## Developer notes

- Component: `Index` in [`views/Research.tsx`](../../web/src/views/Research.tsx). Filters are
  `QFILTER`, labels `QFILTER_LABEL` and `STATUS_LABEL`.
- The idempotency key is a `DraftKey` ([`research/episodes.ts`](../../web/src/research/episodes.ts)):
  a retry of the same draft reuses the key; a new key is made after a success or an edit.
- `QFILTER` names `waiting_for_user` and `interrupted`, while the contract's `EpisodeStatus` uses
  `waiting_tools`, `waiting_approval` and `waiting_user`. Episodes in those three statuses fall
  outside every filter set, including *All*; aligning the sets with `EpisodeStatus` lists them.
