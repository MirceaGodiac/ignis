---
name: IGNIS
description: A calm research harness for NASA microgravity-combustion data in graphite and ember, where colour is spent only on evidence.
colors:
  paper: "#f4f2ee"
  rail: "#ebe8e2"
  sheet: "#fbfaf8"
  ink: "#1b1a18"
  ink-2: "#48453f"
  ink-3: "#68635c"
  rule: "#e2ded7"
  rule-2: "#d3cec6"
  wash: "#ebe8e2"
  hover: "#e8e5df"
  flame: "#b14716"
  flame-wash: "#f6e6dc"
  soot: "#6a4bc0"
  ok: "#2b7046"
  bad: "#b3261e"
  bad-wash: "#f8e5e2"
  grid: "#e7e3dd"
  ev2: "#d27848"
  ev3: "#e6a984"
  ev4: "#f3d5c2"
  paper-dark: "#1a1918"
  rail-dark: "#151413"
  sheet-dark: "#211f1e"
  ink-dark: "#eeebe6"
  ink-2-dark: "#c8c4bd"
  ink-3-dark: "#979189"
  rule-dark: "#2b2927"
  rule-2-dark: "#3a3835"
  wash-dark: "#292725"
  hover-dark: "#262422"
  flame-dark: "#f0895a"
  flame-wash-dark: "#3a2a21"
  soot-dark: "#b7a2ef"
  ok-dark: "#7fc492"
  bad-dark: "#f08a80"
  bad-wash-dark: "#3a2220"
  grid-dark: "#262422"
  ev2-dark: "#c06a43"
  ev3-dark: "#8c4f33"
  ev4-dark: "#5a3525"
typography:
  display:
    fontFamily: "Hanken Grotesk, Segoe UI Variable Text, Segoe UI, system-ui, -apple-system, Roboto, Helvetica Neue, Arial, sans-serif"
    fontSize: "42px"
    fontWeight: 400
    lineHeight: 1.12
    letterSpacing: "-0.032em"
  headline:
    fontFamily: "Hanken Grotesk, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "24px"
    fontWeight: 500
    lineHeight: 1.28
    letterSpacing: "-0.022em"
  title:
    fontFamily: "Hanken Grotesk, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 500
    lineHeight: 1.35
    letterSpacing: "-0.01em"
  title-sm:
    fontFamily: "Hanken Grotesk, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "24px"
    fontWeight: 500
    lineHeight: 1.5
    letterSpacing: "-0.022em"
  heading:
    fontFamily: "Hanken Grotesk, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 500
    lineHeight: 1.5
  reading:
    fontFamily: "Hanken Grotesk, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "15.5px"
    fontWeight: 400
    lineHeight: 1.7
  body:
    fontFamily: "Hanken Grotesk, Segoe UI Variable Text, Segoe UI, system-ui, -apple-system, Roboto, Helvetica Neue, Arial, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.55
    letterSpacing: "-0.006em"
    fontFeature: "tnum"
  body-sm:
    fontFamily: "Hanken Grotesk, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "Hanken Grotesk, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "12.5px"
    fontWeight: 500
    lineHeight: 1.5
  data:
    fontFamily: "Geist Mono, ui-monospace, Cascadia Mono, SF Mono, Menlo, Consolas, monospace"
    fontSize: "12.5px"
    fontWeight: 400
    lineHeight: 1.5
rounded:
  tag: "6px"
  control: "8px"
  surface: "12px"
  float: "13px"
  pill: "999px"
components:
  button-primary:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.paper}"
    rounded: "{rounded.control}"
    height: "32px"
    padding: "0 12px"
  button-secondary:
    backgroundColor: "{colors.sheet}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    height: "32px"
    padding: "0 12px"
  button-secondary-hover:
    backgroundColor: "{colors.hover}"
  button-quiet:
    backgroundColor: "transparent"
    textColor: "{colors.ink-2}"
    rounded: "{rounded.control}"
    height: "32px"
    padding: "0 12px"
  button-quiet-hover:
    backgroundColor: "{colors.hover}"
    textColor: "{colors.ink}"
  button-small:
    height: "24px"
    padding: "0 8px"
  field:
    backgroundColor: "{colors.sheet}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    height: "28px"
    padding: "0 8px"
  composer:
    backgroundColor: "{colors.sheet}"
    textColor: "{colors.ink}"
    rounded: "{rounded.float}"
    padding: "6px"
  prompt:
    backgroundColor: "{colors.sheet}"
    textColor: "{colors.ink}"
    rounded: "14px"
    padding: "12px 12px 8px"
  nav-item:
    backgroundColor: "transparent"
    textColor: "{colors.ink-2}"
    rounded: "{rounded.control}"
    height: "28px"
    padding: "0 8px"
  nav-item-hover:
    backgroundColor: "{colors.hover}"
    textColor: "{colors.ink}"
  nav-item-current:
    backgroundColor: "{colors.wash}"
    textColor: "{colors.ink}"
  tag:
    backgroundColor: "{colors.wash}"
    textColor: "{colors.ink-2}"
    rounded: "{rounded.tag}"
    height: "20px"
    padding: "0 7px"
  tag-model:
    backgroundColor: "{colors.flame-wash}"
    textColor: "{colors.flame}"
  tag-bad:
    backgroundColor: "{colors.bad-wash}"
    textColor: "{colors.bad}"
  pill-synthetic:
    backgroundColor: "transparent"
    textColor: "{colors.ink-3}"
    rounded: "{rounded.pill}"
    height: "22px"
    padding: "0 8px"
  card:
    backgroundColor: "{colors.sheet}"
    rounded: "{rounded.surface}"
    padding: "12px"
  note:
    backgroundColor: "{colors.wash}"
    textColor: "{colors.ink-2}"
    rounded: "{rounded.surface}"
    padding: "9px 11px"
  note-refusal:
    backgroundColor: "{colors.bad-wash}"
    textColor: "{colors.bad}"
  claim-key:
    backgroundColor: "{colors.wash}"
    textColor: "{colors.ink-2}"
    height: "17px"
    padding: "0 4px"
  claim-key-active:
    backgroundColor: "{colors.flame-wash}"
    textColor: "{colors.flame}"
  table-row-selected:
    backgroundColor: "{colors.flame-wash}"
  tray:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.paper}"
    rounded: "{rounded.float}"
---

# Design System: IGNIS

## Overview

**Creative North Star: "Graphite and Ember"** (replaces "Sheet and Biro / Night and Sky Blue", 15 Nov 2026)

IGNIS is a research harness, not a dashboard. Dark is a neutral graphite night with no blue in the
ground; light is warm stone paper. Ink and one warm grey family carry the structure; a single ember
accent marks what the model did and what is selected. The references are Corpus (graphite, roomy
rail, a serif wordmark, a light display question), SubQ (stone paper, ruled cells, quiet numerals)
and TypeSafe (plain, exact chart chrome).

The density is calm, not cramped: 14px body, 32px controls, 38px rail items, 1px hairlines,
12px surfaces. The layout keeps the Linear / Harvey / Legora mould: a left rail, a centre object with
tabs, a properties column and a right side panel with a back trail.

Colour has a meaning or it does not appear. Ember is model output, the current selection, a running
state and a cited region; violet is revealed data; red is a refusal or a failure; green is a complete
or qualified status. Shape carries the evidence class, so colour is never the only signal.

**Key Characteristics:**
- Graphite night (#1a1918 ground, #151413 rail, #211f1e sheet) and stone paper (#f4f2ee ground, #ebe8e2 rail, #fbfaf8 sheet). No blue anywhere in the chrome.
- Hanken Grotesk for all text, including the "Ignis" lockup (flame in an ink square, 18px 600); Geist Mono only for ids, hashes, paths, times and code.
- Display and titles at weight 400-500 (never bold), tight negative tracking; labels at 500 in ink-3, sentence case.
- Flat at rest; shadow only on layers that float above the content and on the Home prompt.
- Theme: System, Light or Dark from the rail footer; a pinned choice is `<html data-theme>`, read before `prefers-color-scheme`.
- Synthetic origin is one neutral outlined pill, never a coloured banner.

## Colors

### Primary
- **Ember** (flame): light #b14716, dark #f0895a. Model output, the current data selection (selected row, active claim key, selected map run, selected route stop), a running state, the cited region box, the planned Moon base and rover region on the atmosphere map, the focus ring, text selection, inline action links. **Ember Wash** (flame-wash: #f6e6dc / #3a2a21) fills the selected row, stop and claim key. No solid ember fill on chrome.
- **Evidence ramp** (ev1-ev4): ember steps for the Forecast lab evidence bar.
- **On video** (#ff9a62): the BASS contour drawn over video, the same in both themes.

### Secondary
- **Reveal Violet** (soot): light #6a4bc0, dark #b7a2ef. Revealed (sealed, then opened) data only.

### Tertiary
- **Refusal Red** (bad #b3261e / #f08a80) with **Refusal Wash**: typed refusals, failed status, bad route stops.
- **Qualified Green** (ok #2b7046 / #7fc492): complete and qualified status, ok route stops, live service dots.

### Neutral
- **Paper** (paper), **Rail** (rail), **Sheet** (sheet): ground, left rail and properties column, surfaces. In dark the rail is darker than the ground and the sheet lighter, as in Corpus.
- **Ink** (ink #1b1a18 / #eeebe6): primary text, the primary button (solid ink, paper text), the active tab underline, the cutoff line, the observed series.
- **Ink 2** (#48453f / #c8c4bd): secondary text, agent messages. **Ink 3** (#68635c / #979189): labels, metadata, axis text, icons at rest (>= 4.5:1 on paper in both themes).
- **Rule** and **Rule 2**: hairlines; control borders and dashed empty states. **Wash**, **Hover**: neutral fills.

### Named Rules
**The Colour Is Data Rule.** Chrome is ink and warm grey. Ember, violet, red and green appear only with the meanings above.

**The No Blue Rule.** No blue or blue-tinted grey in either theme: grounds, rails and accents stay neutral or warm.

**The No Yellow Rule.** Caution has no hue: an ink dot, a dashed ink ring, a neutral tag.

**The Ink Navigation Rule.** Navigation state is ink, not ember: the current nav item is a wash fill, the active tab a 2px ink underline, the primary button solid ink.

## Typography

**Text font:** Hanken Grotesk 400/500/600 (Segoe UI Variable Text, system-ui stand in offline)
**Lockup:** the flame mark in a 26px ink square and "Ignis" in Hanken Grotesk 600, 18px
**Data font:** Geist Mono 400/500

### Hierarchy
- **Display** (400, 42px, 1.12, -0.032em, centred): the Home question only.
- **Page title** (500, 24px, 1.28, -0.022em): an investigation, max 60ch, 3 lines; also list view headings.
- **Card title** (500, 15px, -0.01em): cards, sheets, the side panel head.
- **Reading** (400, 15.5px, 1.7): the answer text, 66ch. Agent messages 15px / 1.65, 70ch.
- **Body** (400, 14px, 1.55, -0.006em): default UI text; rail items 14.5px. Tabular numerals everywhere.
- **Body small** (400, 13px): tables, key-value lists, notes, the side panel.
- **Label** (500, 12.5px, ink-3, sentence case): section labels, properties, panel sections. Never uppercase.
- **Caption** (400, 12px, ink-3): legends, captions, counts. **Data** (Geist Mono, 12.5px): ids, hashes, paths, times.

### Named Rules
**The One Family Rule.** Hanken Grotesk sets all text; the serif is the wordmark and nothing else; mono only for machine values.

**The Light Display Rule.** Large text is never bold: 400 for the Home question, 500 for titles. Weight 600 is for small headings and emphasis only.

## Layout

The shell is a grid: a 256px rail, a flexible main column, and a 400px side panel when an item is open. The main column has a sticky 60px topbar without a rule (crumbs, the synthetic pill, a 300px search field), 90% paper with a 10px blur. An object page has a main column and a 280px properties column on the rail tone. Home is one centred 820px column starting at 11vh.

Information architecture (one registry, `web/src/shell/router.ts` SECTIONS / VIEW_SERVICE): the rail
has the serif wordmark, "New question" (Home's prompt) and search, then Home; **NASA data**: Sources
(knowledge), Runs (registry), Conditions (datasets: Habitats, Atmosphere map and Runs as tabs of one
page); **Models**: Forecast lab, Compare (registry), Models (models); then Recent questions (agent
runtime) and the theme switch. Breadcrumbs read "section › page". A page whose service the deployment
does not serve stays reachable, is dimmed in the rail, and says so in place; there is no service
footer and no global server banner. Every page opens with its title (24px), then tabs or a toolbar,
then its content: no explanatory sentence under a title, no helper copy under a field. The data source
is live; fixtures are a development choice (command menu or `?source=synthetic`).

Spacing: 6-10px inside a group, 16-22px between blocks (20px view gap), 32-36px between sections. View padding 12px 32px 56px. Control heights 26 (small), 32 (default), 38 (rail items), 40 (rail actions), 42 (table rows, tabs, list rows), 60 (topbar, panel head).

Responsive behaviour is unchanged: at 1280px the claims margin moves under the answer; at 1100px the side panel overlays and the properties column moves under the page; at 820px the shell is one column with a menu toggle.

## Elevation & Depth

The system is flat at rest. A surface is a sheet with a 1px hairline (rule) on the rail or paper ground, not a lifted card. Depth comes from the tone steps paper, rail, sheet and wash, and from hairlines. A shadow appears only on a layer that floats above the content: the tooltip, hovercard, suggestion list, workspace menu, command menu, the selection tray and the side panel when it overlays. The Home prompt carries a faint resting lift so that the place to type reads first.

### Shadow Vocabulary
- **Float** (`box-shadow: 0 1px 2px rgb(0 0 0 / 0.05), 0 8px 24px rgb(0 0 0 / 0.08)`; dark: `0 1px 2px rgb(0 0 0 / 0.5), 0 12px 32px rgb(0 0 0 / 0.5)`): popovers, menus, tooltip, tray, overlay panel.
- **Prompt rest** (`box-shadow: 0 1px 2px rgb(0 0 0 / 0.04), 0 6px 20px rgb(0 0 0 / 0.04)`): the Home prompt.
- **Segment pressed** (`box-shadow: 0 1px 2px rgb(0 0 0 / 0.08), 0 0 0 1px var(--rule)`): the pressed button in a segmented control.
- **Focus glow** (`box-shadow: 0 0 0 3px color-mix(in srgb, var(--flame) 15%, transparent)`): the thread composer when focus is inside it, together with a blue border.

### Named Rules
**The Flat At Rest Rule.** A card, table, sheet or note has no shadow. Use the float shadow only for a layer that sits above the content.

## Shapes

Corners are small and consistent. Controls (buttons, fields, menu options) use 8px and rail items 9px. Surfaces (cards, tables, sheets, figure blocks) use 12px; notes 10px. Menus and the hovercard use 13px, the composer and command menu 16px, the Home prompt 20px. Tags use 6px. The pill (999px) is only for the small synthetic badge.

Borders are 1px hairlines. A dashed 1px border marks an empty state. A dashed ring marks something that is not a fact: interpretation, synthetic origin, a warn stop. An outlined (inset 1px) note marks a gap.

### Named Rules
**The Shape Carries Evidence Rule.** Each evidence class has a CSS shape that reads without colour: observed is a filled disc, source is a filled square, computed is a diamond, predicted is a dashed blue band, revealed is a hollow orange ring, interpretation is a dashed grey ring, and the cutoff is a 2px ink bar.

## Components

### Buttons
Quiet and exact.
- **Shape:** gently rounded (6px), 28px high, 13px weight 500, 6px icon gap.
- **Primary:** solid ink with paper text. Hover mixes ink 85% with paper.
- **Secondary (default):** sheet fill, control hairline, ink text. Hover takes the hover tint.
- **Quiet:** no border and no fill, graphite text. Hover takes the hover tint and ink text.
- **Small / Icon:** 24px high with 8px padding; icon buttons are square (28px or 24px).
- **Disabled:** 45% opacity and a not-allowed cursor.
- **Inline link (`linkish`):** blue text with no fill; hover adds an underline at 3px offset.

### Tags
- **Tag:** 20px high, 5px radius, 11.5px weight 500 on wash. The model tag is blue on blue wash. The ok tag is green on a 10% green fill. The bad tag is red on refusal wash. The line tag and the synthetic tag are outlined with an inset 1px control hairline. The caution tag (`tag-warn`: "fit only", "not qualified") is ink text outlined in pencil grey, never a yellow fill.
- **Synthetic pill:** outlined 999px pill in pencil grey with a dashed 6px ring before the text. It lives in the topbar, and on Home (which has no topbar) at the right of the prompt foot. A value that comes from a fixture carries the inline synthetic tag.

### Cards / Containers
- **Corner Style:** 8px.
- **Background:** sheet on the paper ground.
- **Shadow Strategy:** none (see the Flat At Rest Rule).
- **Border:** 1px hairline.
- **Internal Padding:** 12px; card heads are 10px 12px with a hairline under them. A sheet in the lab uses 12px 14px.

### Inputs / Fields
- **Field:** 28px, 6px radius, control hairline, sheet fill, 8px side padding. Focus is the global 2px blue outline at 0 offset.
- **Home prompt:** a 14px-radius textarea card with a control hairline and the prompt rest shadow. Its footer row holds a muted hint and a 30px square primary send button. With focus inside, the border darkens toward ink; it does not turn blue.
- **Composer:** the follow-up input under a thread. It sticks to the bottom over a paper fade, has a 10px radius and a control hairline. With focus inside, the border turns blue and the focus glow appears.
- **Range and checkbox:** accent colour is biro blue.

### Navigation
- **Rail:** rail tone with a hairline at its right edge. The workspace switcher sits at the top (34px), then New investigation and search, then the tools, then the investigations tree.
- **Nav item:** 28px, 6px radius, graphite text with a pencil-grey 16px icon. Hover takes the hover tint. The current item takes the wash fill, ink text, weight 600 and an ink icon. A sub-tree indents 15px behind a 1px control-hairline guide; its items are 26px at 12.5px.
- **Tabs:** 34px, pencil-grey text, ink on hover. The active tab is ink with a 2px ink underline that sits on the hairline.
- **Breadcrumbs:** pencil-grey links, the current item in ink weight 600 with an ellipsis.
- **Command menu:** a 580px dialog at 14vh from the top on a 25% black scrim. It has a 46px input, group labels and 32px options; the selected option takes the hover tint.

### Notes
Neutral by default, red only for refusals.
- **Note:** wash fill, 8px radius, 9px 11px padding, 12.5px graphite text with an ink bold heading.
- **Refusal:** refusal wash fill and a red heading. It shows the typed code and the next action.
- **Caution:** neutral note with a 6px ink dot before the heading.
- **Ok:** neutral note with a 6px green dot before the heading.
- **Gap:** transparent with an inset 1px hairline and pencil-grey text. It names data that the contracts do not carry yet.

### Evidence glyphs (signature)
8px CSS shapes that sit before a label in tags, legends, claim keys and the evidence grid. They use `currentColor` where the class has no hue of its own. See the Shape Carries Evidence Rule.

### Claim keys and margin (signature)
A claim key is a 17px inline chip in the answer text (4px radius, wash fill, 10.5px weight 600) with the claim's glyph. On hover or when active, it turns blue on blue wash and the matching margin row takes the hover tint. The margin lists each claim as a 14px key column and a two-line clamped text.

### Route stops (signature)
A vertical timeline of the agent's steps. A 1.5px control-hairline line joins 9px ring nodes. The node fill and border show the state: ok is solid green, bad is a red ring, warn is a dashed ink ring, run is a blue ring, and the answer is a solid ink square. Times are Geist Mono 11px.

### Next steps
A bordered list under each answer. Each row is a 36px minimum button with a weight-600 label, a pencil-grey description and an arrow that moves 2px right on hover.

### Selection tray
A sticky ink bar at the bottom centre of a table (10px radius, float shadow) with paper text. Its buttons invert: paper fill with ink text.

## Do's and Don'ts

### Do:
- **Do** use biro blue (#1a4be0; dark #5ab9fc) only for model output, the current data selection, a running state, a cited region, the target atmosphere region, focus and inline links.
- **Do** keep every surface flat at rest with a 1px hairline (#ededed), and keep the float shadow for layers above the content.
- **Do** mark every evidence class with its CSS shape (disc, square, diamond, dashed band, hollow ring, dashed ring), so the class reads without colour.
- **Do** show synthetic origin as one outlined pill with a dashed ring in the topbar, and as the inline synthetic tag on fixture values.
- **Do** show a refusal as a red-headed note on refusal wash, with the typed code and the next action; show a gap as an outlined neutral note.
- **Do** set controls at 28px (24px small) with a 6px radius, and surfaces at an 8px radius.
- **Do** set ids, hashes, paths and times in Geist Mono 12px, and everything else in Geist.
- **Do** keep reading text at 66 to 72ch, and keep tabular numerals on.
- **Do** theme the browser surfaces: selection at 22% blue, a 2px blue focus outline at 1px offset, thin scrollbars in the control hairline tone.
- **Do** draw icons from the authored set: 16px grid, 1.5px stroke, round joins.

### Don't:
- **Don't** use yellow or amber for caution or warnings; caution is ink.
- **Don't** use blue for navigation state; the current nav item is a wash fill and the active tab is an ink underline.
- **Don't** use orange for anything except revealed data.
- **Don't** show synthetic origin as a coloured banner.
- **Don't** put a label, tag or pill above a heading as an eyebrow.
- **Don't** build a view from same-size stat tiles or a hero metric; the system is a harness, not a tile dashboard.
- **Don't** add a shadow to a card, table, sheet or note at rest.
- **Don't** use Unicode characters or emoji as icons or as evidence marks.
- **Don't** use a coloured left or right border on notes or cards; notes use a fill.
