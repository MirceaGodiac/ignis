---
name: IGNIS
description: A quiet research harness for NASA microgravity-combustion data, where colour is spent only on evidence.
colors:
  paper: "#ffffff"
  rail: "#fafafa"
  sheet: "#ffffff"
  ink: "#111111"
  ink-2: "#3a3a3a"
  ink-3: "#6b6b6b"
  rule: "#ededed"
  rule-2: "#dedede"
  wash: "#f4f4f4"
  hover: "#f2f2f2"
  flame: "#1a4be0"
  flame-wash: "#eef2fd"
  soot: "#c2410c"
  ok: "#167a45"
  bad: "#c62828"
  bad-wash: "#fdf1f1"
  grid: "#f1f1f1"
  ev2: "#5b7fe9"
  ev3: "#9db2f1"
  ev4: "#d6e0fb"
  paper-dark: "#0a0d14"
  rail-dark: "#0d1119"
  sheet-dark: "#131824"
  ink-dark: "#e9edf3"
  ink-2-dark: "#c3cad6"
  ink-3-dark: "#8891a1"
  rule-dark: "#1d2331"
  rule-2-dark: "#2a3142"
  wash-dark: "#171d2a"
  hover-dark: "#1a2130"
  flame-dark: "#5ab9fc"
  flame-wash-dark: "#12263a"
  soot-dark: "#fb923c"
  ok-dark: "#4ade80"
  bad-dark: "#f87171"
  bad-wash-dark: "#2a1416"
  grid-dark: "#1a2030"
  ev2-dark: "#3b8fd0"
  ev3-dark: "#2a6698"
  ev4-dark: "#1d4466"
typography:
  display:
    fontFamily: "Geist, Segoe UI Variable Text, Segoe UI, system-ui, -apple-system, Roboto, Helvetica Neue, Arial, sans-serif"
    fontSize: "26px"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "-0.025em"
  headline:
    fontFamily: "Geist, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "20px"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "-0.02em"
  title:
    fontFamily: "Geist, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "17px"
    fontWeight: 600
    lineHeight: 1.35
    letterSpacing: "-0.012em"
  title-sm:
    fontFamily: "Geist, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 600
    lineHeight: 1.5
    letterSpacing: "-0.01em"
  heading:
    fontFamily: "Geist, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 600
    lineHeight: 1.5
  reading:
    fontFamily: "Geist, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.65
  body:
    fontFamily: "Geist, Segoe UI Variable Text, Segoe UI, system-ui, -apple-system, Roboto, Helvetica Neue, Arial, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "-0.003em"
    fontFeature: "tnum"
  body-sm:
    fontFamily: "Geist, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "12.5px"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "Geist, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "11.5px"
    fontWeight: 600
    lineHeight: 1.5
  data:
    fontFamily: "Geist Mono, ui-monospace, Cascadia Mono, SF Mono, Menlo, Consolas, monospace"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.5
rounded:
  tag: "5px"
  control: "6px"
  surface: "8px"
  float: "10px"
  pill: "999px"
components:
  button-primary:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.paper}"
    rounded: "{rounded.control}"
    height: "28px"
    padding: "0 10px"
  button-secondary:
    backgroundColor: "{colors.sheet}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    height: "28px"
    padding: "0 10px"
  button-secondary-hover:
    backgroundColor: "{colors.hover}"
  button-quiet:
    backgroundColor: "transparent"
    textColor: "{colors.ink-2}"
    rounded: "{rounded.control}"
    height: "28px"
    padding: "0 10px"
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

**Creative North Star: "Sheet and Biro"**

IGNIS is a research harness, not a dashboard. The page is a white sheet with black type, one grey family and one biro-blue ink. In dark mode the same system becomes "Night and Sky Blue": a blue-black ground with a sky-blue accent. The agent does the work and the researcher reads, checks and decides, so the chrome stays quiet and the evidence gets the colour.

The density is compact. Body text is 13px, controls are 28px high, and hairlines (1px) divide the surfaces. The layout follows the Linear, Harvey, Legora and Hebbia mould: a left rail, a centre object with tabs, a properties column and a right side panel with a back trail. The system refuses tile dashboards, decorative colour and pages that the researcher must drive step by step.

Colour has a meaning or it does not appear. Blue is model output, the current data selection and a running state, orange is revealed data, red is a refusal or a failure, and green is a complete or qualified status. Shape carries the evidence class, so colour is never the only signal.

**Key Characteristics:**
- White sheet, black ink, one grey family, one blue accent (light); blue-black night and sky blue (dark).
- One sans family (Geist) for all text; Geist Mono only for ids, hashes, paths, times and code.
- Compact: 13px body, 28px controls, 1px hairlines, 8px surface radius.
- Flat at rest; shadow only on layers that float above the content.
- Evidence classes drawn as CSS shapes: disc, square, diamond, dashed band, hollow ring, dashed ring.
- Synthetic origin is one neutral outlined pill, never a coloured banner.

## Colors

The palette is achromatic ink on white, with four hues that each carry one meaning.

### Primary
- **Biro Blue** (flame): model output, the current data selection (selected table row, active claim key, selected map run, selected route stop), a running state (the pulse dot and the run route node), the cited region box on a source page, the planned Moon base and rover region on the atmosphere map, the focus ring, text selection and inline action links. The light tint **Biro Blue Wash** (flame-wash) fills the selected row, the selected stop and the active claim key. The build has no solid blue fill.
- **Evidence Blues** (flame, ev2, ev3, ev4): the four-step stack in the Forecast lab evidence bar. It is a ramp of the primary, not new hues.

### Secondary
- **Reveal Orange** (soot): revealed (sealed, then opened) data only: the revealed line in the forecast chart and the hollow revealed ring.

### Tertiary
- **Refusal Red** (bad) with **Refusal Wash** (bad-wash): typed refusals, failed status tags, bad route stops, and the rail dot of an investigation that waits on the researcher.
- **Qualified Green** (ok): complete and qualified status tags, ok route stops, live service dots. Its tag fill is the green at 10% on transparent.

### Neutral
- **Sheet White** (paper): the page ground. **Sheet** (sheet) is the surface of cards, tables, fields and the side panel; in light mode it is the same white, in dark mode it is one step lighter than the ground.
- **Rail Grey** (rail): the left rail, the properties column and the source page frame.
- **Biro Black** (ink): primary text, the primary button, the active tab underline, the cutoff line and the observed series.
- **Graphite** (ink-2): secondary text, agent messages, tag text.
- **Pencil Grey** (ink-3): labels, metadata, counts, axis text, icons at rest. It holds 5.3:1 on white.
- **Hairline** (rule): every divider and surface border. **Control Hairline** (rule-2): borders on buttons, fields, the prompt and dashed empty states; also the scrollbar thumb.
- **Wash** (wash): neutral note fill, tag fill, current nav item, user message bubble. **Hover Tint** (hover): the hover fill on rows and controls.
- **Chart Grid** (grid): chart gridlines only.

### Dark scheme
The dark scheme comes from `prefers-color-scheme: dark`. Each `-dark` token replaces its light sibling with the same role. The chart aliases follow the same tokens: observed and cutoff use ink, model uses flame, revealed uses soot, the muted axis uses ink-3. Chart colours stay hex, because the chart appends an alpha suffix to them.

### Named Rules
**The Colour Is Data Rule.** The chrome is ink and grey. Blue, orange, red and green appear only when they mean model output, selection, a running state, a cited region or the target atmosphere; revealed data; refusal, failure or a wait on the researcher; and complete or qualified status.

**The No Yellow Rule.** Caution has no hue. A caution note carries an ink dot, a warn route stop is a dashed ink ring, and a "not qualified" tag stays neutral grey.

**The Ink Navigation Rule.** Navigation state is ink, not blue. The current nav item is a wash fill, the active tab is a 2px ink underline, and the primary button is solid ink.

## Typography

**Display Font:** Geist (with Segoe UI Variable Text, Segoe UI, system-ui)
**Body Font:** Geist (same stack)
**Label/Mono Font:** Geist Mono (with ui-monospace, Cascadia Mono, SF Mono, Menlo, Consolas)

**Character:** One neutral grotesque does every job, from the Home question to the table labels. Hierarchy comes from size steps and weight 600 against 400, not from a second family. Geist Mono marks a value that a machine wrote.

### Hierarchy
- **Display** (600, 26px, 1.2, -0.025em): the Home question only. A second line in ink-3 can continue it.
- **Headline** (600, 20px, 1.3, -0.02em): the object page title (an investigation). Max 64ch. At 820px and below it is 17px and clamps to 3 lines.
- **Title** (600, 17px, 1.35, -0.012em): the thread goal. Max 72ch.
- **Title small** (600, 15px, -0.01em): the view heading on list views (Runs, Sources, Models).
- **Heading** (600, 13px): card, sheet and panel headings.
- **Reading** (400, 14px, 1.65): the answer text. Measure 66ch. Agent messages use 13.5px at 1.6, max 72ch.
- **Body** (400, 13px, 1.5, -0.003em): the default for all UI text. Tabular numerals are on everywhere.
- **Body small** (400, 12.5px): tables, key-value lists, notes, the side panel.
- **Label** (600, 11.5px, ink-3, sentence case): section labels over lists, the properties heading, panel section headings. The source writes 550; Geist as loaded renders it at 600. Never uppercase.
- **Table header** (500, 11.5px, ink-3): sticky table column headers.
- **Caption** (400, 11.5px, ink-3): legends, table captions, counts and metadata.
- **Data** (Geist Mono 400, 12px): run ids, hashes, file paths, times, JSON. Inline `code` is 0.9em in ink-2.

### Named Rules
**The One Family Rule.** Geist sets all text. Geist Mono sets only ids, hashes, paths, times and code, never a heading or a label for effect.

**The 20px Ceiling Rule.** Inside the workspace no heading is above 20px. The Home question (26px) is the only exception.

## Layout

The shell is a three-column grid: a 232px left rail, a flexible main column, and a 380px right side panel when an item is open. The main column has a sticky 44px topbar with breadcrumbs, the synthetic pill and the search button. The topbar is 88% paper with an 8px backdrop blur, so content stays legible under it.

An object page (an investigation) has a main column and a 280px properties column on the rail tone. The thread is max 980px wide. The answer uses a 66ch text column and a claims margin column at its right. Home is one centred 760px column that starts at 9vh from the top.

Spacing is literal, not a token scale. Use 4 to 8px inside a group, 10 to 16px between blocks (14px is the default view and panel gap), and 24 to 28px between sections. View padding is 16px 24px 40px. Control heights are 24px (small), 28px (default), 32px (table rows and menu options), 34px (tabs and list rows) and 44px (topbar and panel head).

Responsive behaviour:
- At 1280px and below, the claims margin moves under the answer.
- At 1100px and below, the side panel becomes an overlay at the right edge with the float shadow, the properties column moves under the page, and the map and lab grids become one column.
- At 820px and below, the shell is one column. The rail becomes a header with a menu toggle and opens as a full-screen layer. View padding is 12px 14px 32px. Tables hide their low-priority columns (`.narrow-hide`).

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

Corners are small and consistent. Controls (buttons, fields, nav items, menu options) use 6px. Surfaces (cards, tables, sheets, notes, figure blocks) use 8px. The composer, menus, hovercard and tray use 10px. The command menu uses 12px and the Home prompt uses 14px. Tags use 5px. The pill (999px) is only for the small synthetic badge.

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
