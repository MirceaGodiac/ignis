# Harness cost and latency per question

| | |
|---|---|
| **Scope** | What one research question costs in model input and wall time inside IGNIS, what this change removed, what is left, and what still needs a live measurement |
| **Baseline** | [COSTS.md](../../benchmarks/harness/results/COSTS.md) and [LIFT.md](../../benchmarks/harness/results/LIFT.md): 5–12 model requests per question, each re-sending the conversation; about 98K input tokens per question; a fixed prefix of about 10.9K tokens on the first request; p50 latency 24–42 s inside IGNIS against 5–9 s raw |
| **Measured with** | Codex 0.169.0 and Claude Code 2.1.303 run against a local capturing model provider and a stub IGNIS MCP server (no model, no tokens): [`tools/codex/prefix_capture.py`](../../tools/codex/prefix_capture.py); byte counts of the harness and tool definitions from code: `cargo test -p ignis-mcp --test prefix_size -- --nocapture` |

Bytes are exact. Tokens are estimates: the production first request (10.9K tokens) and the captured
first request (59.6 KB with every catalog tool attached) give about 5.5 bytes per token for this
mix of JSON and prose; plain prose is closer to 4. The live `usage.reported` events
(`input_tokens`, `cached_input_tokens`) are the truth.

## 1. Where a request's input goes

### Codex (production runtime)

The first request of an episode, as Codex 0.169 sends it (all 17 catalog tools attached):

| Part | Before | After | Note |
|---|---:|---:|---|
| `additional_tools`: `functions` namespace (`exec` 7,620, `wait` 1,391, `request_user_input` 1,425, `request_user_input_async` 1,544) | 11,980 | 11,980 | `exec` carries the code-mode contract; the two `request_user_input` tools cannot be switched off in 0.169 (`tools.experimental_request_user_input` accepts no known field) |
| `additional_tools`: `collaboration` namespace (`spawn_agent`, `wait_agent`, `send_message`, `followup_task`, `interrupt_agent`, `list_agents`) | 5,529 | 0 | **removed**: `agents.enabled=false` |
| developer: IGNIS base instructions (`codex_base.md`) | 1,024 | 1,024 | |
| developer: harness bundle (investigator, tool section, skill index) | 36,037 | 35,743 | trims below; ends with Codex's own `Deferred tool namespaces: mcp__ignis` |
| developer: `<multi_agent_role>`, `<multi_agent_mode>` | 2,985 | 0 | **removed**: `agents.enabled=false` |
| user: `<environment_context>` (cwd, shell) | 602–746 | 0 | **removed**: `include_environment_context=false`; its cwd differs per episode |
| **whole request body** (incl. the user's question) | **59,583** | **50,378** | **−9,205 bytes, −15.4 %** |

`features.multi_agent=false`, which the profile already set, is deprecated in 0.169: Codex prints
"`[agents].enabled = false` disables subagent tools" and keeps sending the subagent tools and texts.
The two new switches are what the profile meant all along.

Codex 0.169 **always defers MCP tool definitions** (`tool_search_always_defer_mcp_tools` is fixed
on): the model sees only the namespace `mcp__ignis` and reaches a tool's TypeScript declaration
through `ALL_TOOLS` inside `exec`. The IGNIS input schemas (45.8 KB for the whole catalog) are
therefore not in the per-request prefix, and the harness tool section is the only place the model
reads what each tool is for. It stays complete on Codex.

### Claude Code

Claude Code 2.1.303, started the way `ClaudeBridge` starts it, sends every MCP tool definition with
each request (17 tools, 48,115 bytes, none deferred) plus a 38.2 KB system prompt that contains the
harness. The harness tool section repeated every description a second time. It is now an index
(name and traits) for Claude Code only:

| Part | Before | After |
|---|---:|---:|
| harness bundle | 34,954 | 20,104 |
| harness + tool definitions | 80,707 | 65,857 (−18 %) |

### Tool descriptions

The descriptions the model reads were trimmed where the investigator already says the same thing,
without dropping a rule:

| Tool | Before | After | What moved |
|---|---:|---:|---|
| `answer_submit` | 2,717 | 2,215 | the view catalog (kinds and fields) is in the input schema and in "Show, don't only tell" |
| `sources_search` | 1,969 | 1,897 | the lockbox rule, shortened; the investigator's "Lockbox" paragraph has it whole |
| `conditions_query` | 2,065 | 2,048 | the atmosphere bounds point at "Reference atmospheres" (also dropped from the "Which tool answers what" row) |

`artifacts_read` (284 bytes) is already short. The prefix-size test fails when the harness grows
past 35,500 bytes or the tool definitions past 46,500, so a longer description is a decision.

## 2. Prompt caching across episodes

Within an episode every request after the first hits the cache. The first one does not, and the
cause is now confirmed from the captured requests:

- Codex sets `prompt_cache_key` to the thread id, one per episode (and the same id in
  `client_metadata` and the `session-id`/`thread-id` headers). OpenAI routes a request by its
  prefix hash combined with that key, so each episode's first request is routed as a new
  conversation and misses, although its prefix is the same as every other episode's.
- The static prefix is first and identical across episodes: tools, base instructions, harness, then
  the episode's messages. Two captured episodes differ only in the item ids (`at_…`, `msg_…`), which
  are identifiers, not prompt text, and in the question itself. Removing the environment context
  also removed the one per-episode string that sat between the harness and the question.
- No Codex 0.169 setting changes the key: the config schema has none (`prompt_cache_key` exists only
  in the request type), and a model provider's `http_headers` or `query_params` cannot rewrite the
  body.

Claude Code needs nothing: Anthropic caches by exact prefix with `cache_control` on the system
blocks, the system prompt is the same for every episode (`--exclude-dynamic-system-prompt-sections`
already moves cwd and date out of it), and the tool order is the host's registration order.

### The relay this needs (designed, not built)

A shared cache key needs a relay, which is close to the "LLM gateway" AGENTS.md asks an ADR for, and
it sits on every model request. It is therefore designed here and left for an ADR and a live check:

1. A loopback HTTP listener in the host, registered as a Codex model provider:
   `model_provider="ignis_relay"`, `model_providers.ignis_relay.base_url="http://127.0.0.1:<port>/v1"`,
   `wire_api="responses"`, `env_key="CODEX_API_KEY"`. API-key deployments only; a ChatGPT login
   keeps the built-in provider.
2. For `POST /v1/responses` it replaces only `prompt_cache_key` with
   `ignis-<harness digest, 16 hex>-<model>` (optionally `-<n>` sharded by episode when more than
   about 15 requests per minute share it, OpenAI's overflow point) and forwards the body, headers and
   streamed SSE answer unchanged. Everything else passes through byte for byte.
3. Codex may compress request bodies (`enable_request_compression`, on by default): the relay
   decodes zstd before rewriting, or the profile sets `features.enable_request_compression=false`.
4. GPT-6 models introduce cache boundaries and cache-write billing (Codex's own model notes); the
   saving must be confirmed on the bill, not assumed.

Expected: the first request (about 9.2K tokens after this change; see below) moves from the input
price to the cached price. On GPT-6 Astra (10.00 vs 1.00 USD per million) that is about 0.08 USD
per question; on GPT-6 Luna about 0.0008.

## 3. Round trips and latency

- **Batched tool calls now run in parallel on Codex.** `codex_base.md` already told the model to
  batch independent calls in one `exec` with `Promise.allSettled`, but Codex 0.169 ran an MCP
  server's calls one after another unless the server is marked `supports_parallel_tool_calls`.
  Measured with a stub whose every call takes 2 s, three batched calls: **6.0 s before, 2.0 s
  after**. The host already serves concurrent calls (each on the blocking pool), as it does for
  Claude Code's parallel tool use of read-only tools.
- The investigator now says it for both runtimes (work loop step 2): issue independent calls in one
  step, keep dependent ones sequential, do not re-read a range already returned.
- **Jobs.** `jobs_get`, `models_forecast` and `analysis_execute` wait up to `JOB_WAIT` (25 s) for the
  job server-side instead of polling through the model. The wait polls the store with a backoff
  that now stops at 200 ms instead of 500 ms, so a finished job is seen sooner (one SQLite read per
  pause).
- `sources_read` results are not cached by the host. A re-read is a model decision, and the
  conversation already holds the earlier result; a host cache would save only the corpus read and
  need invalidation by corpus snapshot. The prompt asks the model not to re-read instead. Whether
  re-reads happen is a live measurement (below).
- `parallel_tool_calls` in the request stays `false`: Codex's parallelism is code mode (`exec`), not
  parallel function calls, and Codex sets the field itself.

## 4. Reasoning effort

Codex 0.169 sends `reasoning.effort = "medium"` (with `context: "all_turns"` and
`text.verbosity: "low"`) for the GPT-6 family when nothing is configured; Claude Code 2.1.303 sends
`output_config.effort = "medium"` with adaptive thinking. Neither was configurable from IGNIS.

`IGNIS_CODEX_REASONING_EFFORT` / `--codex-reasoning-effort` now sets `model_reasoning_effort` for
every Codex turn (`none`, `minimal`, `low`, `medium`, `high`, `xhigh`; an unknown value leaves the
runtime unavailable with the reason). It is recorded in the build manifest's `config_overrides`, and
the effort the runtime reports is pinned on each episode. **Unset keeps today's behaviour**; the
deployed model and effort are unchanged.

Trade-off: reasoning tokens are billed as output (GPT-6 Astra: 50.00 USD per million) and are
generated before every tool call, so a lower effort shortens each of the 5–12 rounds and costs
less; it may also plan fewer avenues or bind citations less carefully. Choose it with a benchmark
run per effort (the harness benchmark records the pin), not by expectation.

## 5. Expected savings

Per question, assuming 8 model requests (the middle of 5–12), all with the same fixed prefix:

| Change | Tokens per request | GPT-6 Astra | GPT-6 Luna |
|---|---:|---:|---:|
| `agents.enabled=false`, `include_environment_context=false`, description trims (Codex) | about −1.7K (−15 % of the 10.9K prefix) | about −0.03 USD (first request at 10.00, seven cached at 1.00 per million) | about −0.0003 USD |
| index tool section (Claude Code) | about −3.7K | n/a | n/a |
| parallel batched MCP calls (Codex) | 0 | latency: a batch of k calls takes the longest call instead of the sum | same |
| cache key relay (designed, not built) | 0 | about −0.08 USD | about −0.0008 USD |

On Claude Fable 5.2 (0.66 USD per question in LIFT.md) the index section removes about 3.7K tokens
from each request; most of them are cache reads, so the saving is a few cents per question.

## 6. Needs a live measurement

1. **Prefix tokens before and after**: one question on staging with each build, comparing the first
   `usage.reported.input_tokens` of the episode (expected about 10.9K → 9.2K on the same tool set).
2. **Parallel calls in practice**: from the event log, how many `tool.called` share one runtime item
   (one `exec`), and the turn's wall time against the sum of its tool latencies
   (`GET /api/v1/episodes/{id}/metrics`).
3. **Tool result size**: the distribution of result bytes per tool. A result is inlined up to
   `INLINE_RESULT_BYTES` (64 KiB, about 12–16K tokens) and is then re-sent on every later request of
   the episode; if large inline results are common, this is the largest remaining lever.
4. **Re-reads**: `sources_read` calls with the same `source_id` and range within one episode.
5. **Reasoning effort**: the benchmark per effort, with the score, cost and p50 latency.
6. **The relay**: first-request `cached_input_tokens` across episodes with a shared key, and the cache
   write charge on GPT-6.

## 7. Staging configuration

No variable is required: the new Codex switches are part of the public profile (they change the
runtime build's `config_overrides`, so new episodes pin the new build), and the Claude index section
follows from `IGNIS_AGENT_RUNTIME=claude`. Optional:

| Variable | Value | Effect |
|---|---|---|
| `IGNIS_CODEX_REASONING_EFFORT` | unset (default) | Codex default (`medium` for GPT-6) |
| `IGNIS_CODEX_REASONING_EFFORT` | `low` | only for a benchmark run of the trade-off above |
