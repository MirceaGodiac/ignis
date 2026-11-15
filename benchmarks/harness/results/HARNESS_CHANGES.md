# What changed in IGNIS (old harness → new harness, PR #122, deployed 2026-11-15)

Found by using production IGNIS as a researcher and by the first benchmark round.

## Answers
- **Model knowledge allowed, labelled**: the agent answers from its own knowledge too, as claims labelled "model knowledge, not from the NASA corpus", while citing NASA documents wherever it can. Before, it could only state what it retrieved.
- **A real agentic loop**: plan → retrieve → check → iterate → verify with the tools → submit; no small call budget.
- **Tool map**: the agent now knows when to use the graph, document outlines, run conditions and datasets (before: only search and read).
- **NASA guidance**: uncertainty and credibility wording (NASA-STD-7009B), dataset citation (SPD-41a / PSI), AI disclosure, flammability terms (NASA-STD-6001B), reference crew atmospheres.

## Cost (measured)
- Codex's coding-agent base prompt replaced with a short research prompt: about 6K fewer input tokens per request.
- Skills load on demand instead of on every request; tool schemas 47% smaller; server-side job waits instead of polling; compact search results (−67% bytes).
- First request of a question: ~17.5K → ~10.9K tokens.

## Effect on GPT-6 Astra (same model, same questions)

| | Old harness | New harness |
|---|---|---|
| Score | 75 | **82** |
| Cost / question | $0.514 | **$0.426** |
| NASA-record score | 17 | 33 |
| Precision | 97 | 100 |
