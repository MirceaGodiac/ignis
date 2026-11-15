# Cost analysis

## Headline comparisons

- **GPT-6 Luna inside IGNIS: 77 at $0.0093 per question.**
  - Raw GPT-6 Astra (flagship) scores the same 77 at $0.0651: **7× the price**.
  - Raw GPT-6 Luna: 43 at $0.0007. IGNIS adds +34 points for $0.0086 more per question.
  - IGNIS on GPT-6 Astra reaches 82 (+5 over IGNIS-Luna) at $0.4256: 46× IGNIS-Luna's cost.

## IGNIS (new harness) vs raw models it matches or beats, ranked by how much cheaper IGNIS is

| IGNIS system | Score | Cost / q | Raw system | Score | Cost / q | IGNIS is | Points |
|---|---|---|---|---|---|---|---|
| IGNIS v2 · GPT-6 Luna | 77 | $0.0093 | GPT-6 Astra | 77 | $0.0651 | 7.0× cheaper | +0 |
| IGNIS v2 · GPT-6 Luna | 77 | $0.0093 | GPT-5.6 Terra | 52 | $0.0179 | 1.9× cheaper | +25 |
| IGNIS v2 · GPT-6 Luna | 77 | $0.0093 | GPT-5.6 Sol | 70 | $0.0139 | 1.5× cheaper | +7 |
| IGNIS v2 · GPT-6.1 Terra | 80 | $0.0463 | GPT-6 Astra | 77 | $0.0651 | 1.4× cheaper | +2 |
| IGNIS v2 · GPT-6 Luna | 77 | $0.0093 | Claude Opus 5.5 | 75 | $0.0129 | 1.4× cheaper | +2 |
| IGNIS v2 · GPT-6 Luna | 77 | $0.0093 | GPT-6.1 Sol | 73 | $0.0097 | 1.0× cheaper | +5 |
| IGNIS v2 · GPT-6 Luna | 77 | $0.0093 | GPT-6 Sol | 66 | $0.0091 | 1.0× the price | +11 |
| IGNIS v2 · GPT-6.1 Sol | 82 | $0.0804 | GPT-6 Astra | 77 | $0.0651 | 1.2× the price | +5 |
| IGNIS v2 · GPT-6 Luna | 77 | $0.0093 | Claude Sonnet 5.5 | 57 | $0.0060 | 1.6× the price | +20 |
| IGNIS v2 · GPT-6.1 Terra | 80 | $0.0463 | Claude Fable 5.2 | 80 | $0.0248 | 1.9× the price | +0 |
| IGNIS v2 · GPT-6 Sol | 77 | $0.1287 | GPT-6 Astra | 77 | $0.0651 | 2.0× the price | +0 |
| IGNIS v2 · GPT-6 Luna | 77 | $0.0093 | GPT-6.1 Terra | 57 | $0.0038 | 2.4× the price | +20 |

## Where IGNIS's cost goes (measured on production)

- One question = 5–12 model requests (one per tool round); each request re-sends the conversation so far.
- Output is small (about 500–1,500 tokens per question); almost all cost is input tokens.
- The new harness cut the fixed per-request prefix from ~17.5K to ~10.9K tokens (first request, measured), and a question's total input from ~146K to ~98K tokens on the same question.
- Still open: the first request of every episode is not prompt-cached (Codex ties the cache to each conversation); fixing it needs a small relay and would save roughly a further $0.10 per question on GPT-6 Astra.

## Pricing used

OpenAI standard tier, short context, USD per million tokens (developers.openai.com/api/docs/pricing, read 2026-11-15):

| Model | Input | Cached input | Output |
|---|---|---|---|
| gpt-6-astra | 10.00 | 1.00 | 50.00 |
| gpt-6.1-sol | 2.00 | 0.10 | 10.00 |
| gpt-6.1-terra | 1.00 | 0.05 | 5.00 |
| gpt-6-sol | 2.00 | 0.20 | 10.00 |
| gpt-6-luna | 0.10 | 0.01 | 0.50 |
| gpt-5.6-sol | 4.00 | 0.40 | 20.00 |
| gpt-5.6-terra | 2.00 | 0.20 | 12.00 |
| gpt-5.6-luna | 0.20 | 0.02 | 1.20 |

Claude costs are as reported by Claude Code. Committee (judging) spend is not included in any system's cost (Claude jurors reported $8.50 in total).
