# Sources

[Back to the screen guide](README.md)

## Route

`#/sources[?q=<query>]`. `q` is the submitted search; the page searches only when `q` is set, so a
search is a shareable link. Rail: *NASA data > Sources*.

## Purpose

Sources searches the NASA reports, papers and test documents IGNIS holds. Each hit is a passage with
its place in the document; the selected hit opens beside the list as the NASA original at that page,
with the passage marked.

## Layout

1. **Heading** "Sources".
2. **Search form**: field "Search NASA documents" and **Search**. Submitting sets `?q=`.
3. **Try:** three starter searches: *oxygen concentration flammability limit*, *soot formation in
   microgravity flames*, *flame size of gas jet flames in microgravity*. The active one is
   highlighted.
4. **Empty state** (no query): "Search the NASA reports, papers and test documents IGNIS holds. Each
   hit opens at its page in the original."
5. **Result line**: search mode (for example *hybrid search*), corpus snapshot id, total results (or
   "total not counted"), and a *Synthetic* tag for fixture results.
6. **Degraded ranking** note, when a search backend was not used: names each backend and its status.
7. **Split view**:
   - **Result list** (left): each hit shows its rank, title (or source id), the quote or snippet, and
     the evidence class with the locator. Click to select; the first hit is selected by default.
     **Load more (N shown)** fetches the next page (25 passages per page).
   - **Preview** (right), for the selected hit: title, **Passage** (opens the Source item in the side
     panel), **Originals tab ↗**, **Open at NASA ↗** (the provider's landing page), the delivery
     mode, and the original itself: the PDF page with the cited region marked, the video at its
     time, or the table record (see [original.md](original.md)).

### States

- "Searching..." while a search runs.
- "Search is not available on this server" with the capability detail, when the knowledge service is
  off (the field and button are disabled).
- "Search is not configured" (code `capability_unavailable`) or "The search failed" with the error.
- No hits: "No hits. An empty page does not show that the corpus lacks the information."
- Preview: "Finding the original...", or "The original cannot be opened from here; the passage is
  still readable", or "This hit has no page to open."

## Data

| What | Source |
|---|---|
| Hits | `POST /api/v1/search` with `{ schema_version: "evidence.v1", query, mode: "hybrid", page_size: 25, cursor? }` → `SearchPage` (`hits[]` with `rank`, `title`, `snippet`, `evidence`; `mode`, `corpus_snapshot`, `total_count`, `backends`, `next_cursor`, `origin`) |
| Original | `GET /api/v1/sources/{source id}/content` → `SourceAccess` (`mode` direct/proxy/link_out, `url`, `range`, `cors`, `expires_at`, `landing_url`) |

The access answer is cached only until 30 s before its `expires_at`. Delivery modes read:
*Streamed from NASA; IGNIS stores no copy* (direct), *Through the IGNIS pass-through proxy; nothing
is stored* (proxy), *Opens at the provider* (link_out).

## Flows

**Find a passage and read it in the original**

1. Type a query and press **Search** (or pick a starter).
2. Scan the hits; click one to preview its original at the cited page.
3. Use the PDF pager to read around it; **Back to cited page** returns.
4. Click **Originals tab ↗** to open it full size in the IGNIS originals tab, or **Open at NASA ↗**
   for the provider's page.

**Cite-check a passage**: click **Passage** to open the Source item in the side panel, with the
locator, character span and parsed page text.

## Links

Side panel Source item; [NASA Original viewer](original.md). The Investigation panel's
**Documents** button opens Sources with the investigation's acronym as the query.

## Developer notes

- Components: `Sources`, `SourcePreview` in [`views/Sources.tsx`](../../web/src/views/Sources.tsx);
  `OriginalBody` and `MODE_TEXT` in [`views/Original.tsx`](../../web/src/views/Original.tsx).
- Paging uses `useInfiniteQuery` over `next_cursor`. Change the page size with `PAGE`, the starters
  with `STARTERS`.
- The view is keyed by `q` in [`App.tsx`](../../web/src/App.tsx), so a new query starts a fresh
  result list.
