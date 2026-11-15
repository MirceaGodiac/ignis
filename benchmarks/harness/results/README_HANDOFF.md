# IGNIS benchmark: results for the final presentation

Hand-off for assembling the slides. Every number in these files is generated from the benchmark
results (`results.json`), the same data the charts are drawn from. Charts are in this folder.

## The one-line story

**The cheapest OpenAI model inside IGNIS (GPT-6 Luna, 77) matches the flagship raw model (GPT-6 Astra, 77) at 1/7 of the cost per question**, and lifts Luna from 43 to 77 (+34 points).

## Files

| File | What it is |
|---|---|
| `RESULTS.md` | The full leaderboard: every system, score, interval, cost, latency, tokens |
| `LIFT.md` | Each model raw vs inside IGNIS, and the cost of that lift |
| `COSTS.md` | Cost analysis and the most striking cheap-vs-expensive comparisons |
| `METHOD.md` | How the benchmark was run and graded, and the caveats to keep on the slides |
| `HARNESS.md` | What changed in IGNIS between the two harness versions, with measured savings |
| `QUESTIONS.md` | The 22 questions and their reference answers |

## Charts (PNG, 16:9-friendly, 200 dpi)

| Chart | Suggested slide |
|---|---|
| `headline.png` | Title/claim slide: cheapest model inside IGNIS vs flagships raw |
| `lift.png` | "Same model, with and without IGNIS" (raw vs IGNIS per model, cost under each pair) |
| `score_vs_cost.png` | Artificial Analysis-style score vs cost per question (log scale, Pareto line) |
| `score_vs_latency.png` | Score vs latency (the trade-off: IGNIS reads documents, ~30–40 s per answer) |
| `leaderboard.png` | Full ranked board with 95% intervals |
| `by_kind.png` | Where models fail: concept questions vs NASA-record facts vs traps |

## Suggested slide order

1. **Claim**: `headline.png`.
2. **Every model gets better inside IGNIS**: `lift.png` (weaker models gain most).
3. **Quality per dollar**: `score_vs_cost.png` (IGNIS points sit on the Pareto line).
4. **Honest trade-off**: `score_vs_latency.png` (slower, because it reads NASA documents and cites them).
5. **How it was measured**: one slide from `METHOD.md` (22 questions, blind 3-LLM committee, caveats).
