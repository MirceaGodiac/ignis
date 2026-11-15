# Method and caveats

## Questions
- 22 questions about NASA microgravity-combustion investigations (FLEX, FLEX-2, ACME s-Flame / BRE / E-FIELD / CLD / CFI-G, BASS, Saffire, SPICE, SLICE, SAME, CFI, DAFT).
- Three kinds: **concept** (what an investigation studied), **NASA record** (facts only in NASA's PSI records: DOIs, dataset sizes, PSI numbers, hardware), **trap** (a made-up test run and a false premise: the right answer is to abstain or reject it).
- This is a pilot set written for the presentation, **not** the IGNIS gold benchmark (T10). Record references come from NASA PSI metadata; concept references are short key points from public NASA descriptions.

## Systems
- **Raw models**: one call, no tools, no documents, same short prompt ("answer from your own knowledge; say if you are not sure"). OpenAI models through the Codex CLI, Claude models through Claude Code headless.
- **IGNIS**: the production research agent with its harness, tools and the NASA document corpus. Old harness = before 2026-11-15 12:39 UTC; new harness = commit a31cc97 (see `HARNESS.md`). GPT-6 Astra ran on production, the other models on the identical staging instance.

## Grading
- A committee of 3 jurors (claude-opus-5-5, claude-fable-5-2, gpt-6.1-sol) graded every answer against the reference, without being told which system wrote it, in shuffled order.
- Labels: correct (1), partly correct (0.5), incorrect (0), abstained (0). Majority vote; a three-way split is left out.
- The jurors agreed unanimously on 87% of answers.
- Self-preference check: each score is also computed with only the jurors from another model family (`index_other_families` in results.json); it moves by a few points at most.

## Caveats to keep on the slides
- **22 questions**: intervals are wide (about ±15 points). Use "matches" / "on par" for close scores, not "beats".
- **IGNIS answers carry citations**, so jurors can tell them apart from raw answers; grading of IGNIS is not fully blind.
- **Costs are API-equivalent**: the OpenAI runs used a ChatGPT login and the Claude IGNIS runs a plan token; costs are tokens × list price (OpenAI) or Claude Code's reported cost.
- **Latency**: IGNIS ~25–40 s per answer vs ~5–9 s raw, because it searches and reads documents before answering.
- **NASA record questions** (DOIs, dataset sizes) are not in IGNIS's document corpus yet; IGNIS abstains or answers from model knowledge (labelled), which caps its record score.
