# NASA Original viewer

[Back to the screen guide](README.md)

## Route

`#/original/<source id>?page=&bbox=&row=&col=&t=&path=&title=`

| Parameter | Meaning |
|---|---|
| `<source id>` | `sha256:<hex>` of the NASA asset |
| `page` | PDF page to open (1-based) |
| `bbox` | cited region on the page, `x0,y0,x1,y1` as fractions of the page, origin top-left |
| `row` | cited record of a table (the header is record 1) |
| `col` | cited column name |
| `t` | video time, seconds |
| `path` | file path inside the source |
| `title` | page heading and window title |

Links are built by `originalHref` in [`shell/OpenOriginal.tsx`](../../web/src/shell/OpenOriginal.tsx)
from a `SourceLocator`. The link carries only the source id and the locator, never a delivery URL.

## Purpose

The viewer opens a NASA original at exactly the place a citation, search hit, condition value or run
record points to: the PDF page with the cited region marked, the video at the cited time, or the
table record with its neighbours. Every **Original ↗** button in the app opens it in one shared
browser tab (named `ignis-original`), so you flip between IGNIS and the original while each later
click reuses that tab.

## Layout

1. **Heading**: the `title` (else the path, else the source id).
2. **Delivery mode** and **Open at NASA ↗** (the provider's landing page, http(s) only).
3. **Locator line**: source id, *page N*, *record N*, *column "X"*, *N s*.
4. **The original**, by locator kind:
   - **PDF** (a page locator, or a `.pdf` title): a pager **<**, "Page N of M", **>**, **Back to cited
     page** (when you have paged away), and the bytes fetched with the method ("1.2 MiB by range" or
     "whole"). The page is rendered at the frame width with a selectable text layer and the cited
     region boxed on the cited page.
   - **Video** (a time locator): the video with controls, seeked to the cited time, and "Cited time:
     N s".
   - **Table** (row, column or path): the header and the records around the cited one (12 before and
     12 after), the cited record outlined and the cited cell highlighted. A line states which records
     are shown and how many KiB were read ("stopped after the shown records", or "whole file").

### States

- No source id: "No source named. Open an original from a citation, a search hit, a condition or a
  run."
- "Finding the original..." while access resolves.
- "The original cannot be opened from here" with the error.
- "The access link expired." with **Resolve again**.
- A `link_out` source (or one without CORS): "Not streamed by IGNIS. This source opens at the
  provider. Use "Open at NASA"."
- PDF: "The PDF could not be opened here: `<reason>`". Video: "The video could not be played here.
  Open it at NASA." Table: "The cited record is past the end of the file".

## Data

`GET /api/v1/sources/{id}/content` returns `SourceAccess`. Delivery modes: *direct* streams from
NASA's storage, *proxy* passes through the IGNIS Worker, *link_out* links to the provider. IGNIS
stores no copy.

How bytes are read ([`shell/PdfPage.tsx`](../../web/src/shell/PdfPage.tsx)):

- PDFs load lazily (PDF.js is fetched only when an original opens). When the access says ranges work,
  a custom range transport fetches only the 256 KiB chunks PDF.js asks for. The file length comes
  from a 1-byte `Range` probe (`Content-Range`), or from the `Content-Length` of a GET dropped after
  its headers; otherwise the file is read whole.
- Tables stream the CSV from the start and stop after the last shown record
  ([`research/csvRecords.ts`](../../web/src/research/csvRecords.ts)).
- Videos load metadata only, then seek.

## Flows

**Open a cited page**: click **Original ↗** on a citation card, a sources list, a run's *NASA
originals*, a conditions cell, or a search preview. The originals tab opens at the page with the
region boxed.

**Read around the citation**: page with **<** and **>**; **Back to cited page** returns.

## Links

Opened from the answer, side panel (Source, Run, NASA original items), Sources, conditions matrix.
**Open at NASA ↗** leaves IGNIS.

## Developer notes

- Components: `Original`, `OriginalBody`, `TableOriginal`, `locatorFromParams` in
  [`views/Original.tsx`](../../web/src/views/Original.tsx); `VideoOriginal`, `SourceOriginal`,
  `httpUrl` in [`shell/SourceOriginal.tsx`](../../web/src/shell/SourceOriginal.tsx); `PdfPage` in
  [`shell/PdfPage.tsx`](../../web/src/shell/PdfPage.tsx); `viewerFor`, `accessExpired`,
  `videoTimeS` in [`research/sourceAccess.ts`](../../web/src/research/sourceAccess.ts).
- `OriginalBody` is shared by this page, the Sources preview and the panel's *NASA original* item.
- To support a new medium, add a `locatorKind` in
  [`research/units.ts`](../../web/src/research/units.ts) and a branch in `OriginalBody`.
