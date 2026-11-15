# Same model, raw vs inside IGNIS (new harness)

| Model | Raw score | IGNIS score | Lift | Raw cost / question | IGNIS cost / question | Raw latency p50 | IGNIS latency p50 |
|---|---|---|---|---|---|---|---|
| GPT-6 Astra | 77 | **82** | **+5** | $0.0651 | $0.4256 | 8.2 s | 37.3 s |
| GPT-6.1 Sol | 73 | **82** | **+9** | $0.0097 | $0.0804 | 9.1 s | 24.1 s |
| Claude Fable 5.2 | 80 | **82** | **+2** | $0.0248 | $0.6645 | 5.7 s | 42.2 s |
| GPT-6.1 Terra | 57 | **80** | **+23** | $0.0038 | $0.0463 | 5.5 s | 28.1 s |
| GPT-6 Sol | 66 | **77** | **+11** | $0.0091 | $0.1287 | 6.9 s | 30.1 s |
| GPT-6 Luna | 43 | **77** | **+34** | $0.0007 | $0.0093 | 5.0 s | 38.1 s |

- Every model scores at least as high inside IGNIS. The largest gain: **GPT-6 Luna (+34)**; the smallest: Claude Fable 5.2 (+2).
- The weaker the model, the more IGNIS adds: the harness and the NASA corpus supply what a small model lacks.
- IGNIS is slower (it searches and reads NASA documents and cites them) and uses more tokens per question.
