# Full results

22 questions, graded blind by a committee of 3 LLM jurors (claude-opus-5-5, claude-fable-5-2, gpt-6.1-sol); majority vote; correct = 1, partly correct = 0.5. The committee was unanimous on 87% of answers.

| # | System | Type | Score | 95% CI | Concepts | NASA records | Traps | Precision | Abstained | Cost / question | Cost basis | Latency p50 | Output tokens (all 22) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | IGNIS v2 · GPT-6 Astra | IGNIS (new harness) | **82** | 64–95 | 100 | 33 | 100 | 100 | 18% | $0.4256 | list | 37.3 s | 21,851 |
| 2 | IGNIS v2 · GPT-6.1 Sol | IGNIS (new harness) | **82** | 64–95 | 100 | 33 | 100 | 100 | 18% | $0.0804 | list | 24.1 s | 21,934 |
| 3 | IGNIS v2 · Claude Fable 5.2 | IGNIS (new harness) | **82** | 64–95 | 100 | 33 | 100 | 100 | 18% | $0.6645 | reported | 42.2 s | 102,151 |
| 4 | Claude Fable 5.2 | raw model | **80** | 61–95 | 100 | 25 | 100 | 97 | 18% | $0.0248 | reported | 5.7 s | 7,584 |
| 5 | IGNIS v2 · GPT-6.1 Terra | IGNIS (new harness) | **80** | 61–95 | 100 | 25 | 100 | 97 | 18% | $0.0463 | list | 28.1 s | 24,559 |
| 6 | GPT-6 Astra | raw model | **77** | 59–95 | 100 | 17 | 100 | 100 | 23% | $0.0651 | list | 8.2 s | 2,074 |
| 7 | IGNIS v2 · GPT-6 Sol | IGNIS (new harness) | **77** | 59–93 | 96 | 25 | 100 | 94 | 18% | $0.1287 | list | 30.1 s | 37,566 |
| 8 | IGNIS v2 · GPT-6 Luna | IGNIS (new harness) | **77** | 59–93 | 96 | 25 | 100 | 94 | 18% | $0.0093 | list | 38.1 s | 61,225 |
| 9 | Claude Opus 5.5 | raw model | **75** | 57–91 | 100 | 17 | 75 | 97 | 23% | $0.0129 | reported | 6.6 s | 8,128 |
| 10 | IGNIS (GPT-6 Astra + harness) | IGNIS (old harness) | **75** | 57–91 | 96 | 17 | 100 | 97 | 23% | $0.5141 | list | 33.4 s | 19,427 |
| 11 | GPT-6.1 Sol | raw model | **73** | 55–89 | 93 | 17 | 100 | 94 | 23% | $0.0097 | list | 9.1 s | 2,663 |
| 12 | IGNIS (GPT-6.1 Terra + harness) | IGNIS (old harness) | **73** | 55–89 | 93 | 17 | 100 | 94 | 23% | $0.0467 | list | 28.8 s | 18,691 |
| 13 | GPT-5.6 Sol | raw model | **70** | 50–89 | 96 | 0 | 100 | 91 | 23% | $0.0139 | list | 8.6 s | 3,598 |
| 14 | GPT-6 Sol | raw model | **66** | 45–84 | 89 | 0 | 100 | 91 | 27% | $0.0091 | list | 6.9 s | 3,275 |
| 15 | Claude Sonnet 5.5 | raw model | **57** | 39–75 | 79 | 0 | 75 | 69 | 18% | $0.0060 | reported | 5.2 s | 7,074 |
| 16 | GPT-6.1 Terra | raw model | **57** | 39–75 | 75 | 0 | 100 | 83 | 32% | $0.0038 | list | 5.5 s | 1,291 |
| 17 | Claude Haiku 5.5 | raw model | **52** | 32–70 | 68 | 0 | 100 | 88 | 41% | $0.0004 | reported | 6.6 s | 9,371 |
| 18 | GPT-5.6 Terra | raw model | **52** | 34–70 | 68 | 0 | 100 | 64 | 18% | $0.0179 | list | 26.9 s | 2,525 |
| 19 | GPT-5.6 Luna | raw model | **50** | 30–68 | 64 | 0 | 100 | 55 | 9% | $0.0014 | list | 8.2 s | 5,604 |
| 20 | GPT-6 Luna | raw model | **43** | 27–59 | 54 | 0 | 100 | 59 | 27% | $0.0007 | list | 5.0 s | 1,039 |

Column meanings:
- **Score**: mean committee score over all 22 questions, 0–100.
- **95% CI**: bootstrap over questions. With 22 questions the intervals are wide, so neighbours whose intervals overlap are not reliably different.
- **Concepts / NASA records / Traps**: score per question kind (see `QUESTIONS.md`).
- **Precision**: mean score over the questions the system actually answered (abstentions excluded).
- **Abstained**: share of questions where it declined to answer.
- **Cost basis**: `reported` = cost reported by Claude Code; `list` = reported tokens × OpenAI's list price (2026-11-15). The OpenAI runs used a ChatGPT login and the Claude IGNIS runs a plan token, so these are API-equivalent costs, not bills.
- **IGNIS rows**: cost and latency cover the whole agent turn (every search, read and model request).
