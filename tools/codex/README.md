# tools/codex — set up and check the agent runtime on a machine

IGNIS runs a pinned Codex App Server (`0.169.0`) as a private child process. These scripts find or
install that exact build, record its path and hash, start the server with it, and send one message
through the HTTP API to check the whole path. Bash versions are for Linux, macOS and Git Bash; the
`.ps1` versions are for Windows PowerShell 5.1 and pwsh 7.

| Step | Linux / macOS / Git Bash | Windows PowerShell |
|---|---|---|
| 1. Find or install Codex 0.169.0, write `runs/codex/codex.env` | `tools/codex/setup.sh [--install]` | `tools\codex\setup.ps1 [-Install]` |
| 1b. Sign in with your Codex account (local dev, once) | `tools/codex/login.sh` | `tools\codex\login.ps1` |
| 2a. Start IGNIS with the real Codex | `tools/codex/serve.sh` | `tools\codex\serve.ps1` |
| 2b. …or with the free fake runtime | `tools/codex/serve.sh --fake` | `tools\codex\serve.ps1 -Fake` |
| 3. Send one message and follow it | `tools/codex/smoke.sh [--approve] [--spend-tokens]` | `tools\codex\smoke.ps1 [-Approve] [-SpendTokens]` |

- `--install` runs `npm install @openai/codex@0.169.0` into `runs/codex/npm` (needs Node.js ≥ 16;
  ~100 MB). Without it, setup looks for `IGNIS_CODEX_BIN`, that local install, then the global npm
  install. It records the **native** binary (`…/vendor/<target>/bin/codex[.exe]`), not the npm shim,
  and refuses any other version.
- `runs/codex/codex.env` (`IGNIS_CODEX_BIN`, `IGNIS_CODEX_SHA256`) is machine-specific and ignored by
  git. `serve` passes both to `ignis serve`, so the server refuses a binary that changed since setup.
- **Credentials.** Two ways:
  - **Your Codex (ChatGPT) account, local development:** run `login` once. It signs in through the
    browser into `runs/codex/login`, a Codex home that belongs to IGNIS (your personal `~/.codex`
    login is never copied or refreshed by IGNIS, so your own Codex stays signed in). `serve` then
    starts the server with `--codex-login-home`; every workspace shares that home, so the server
    allows it only with `--auth local`. Turns count against your Codex plan's usage.
  - **The login of your own Codex / ChatGPT app:** `serve --use-app-login` (`-UseCodexAppLogin`)
    points the runtime at `~/.codex` in place (nothing copied, so nothing is signed out). Your
    `config.toml` applies too (e.g. its default model; override with `--codex-model`) and the
    runtime's conversations appear in that app's history.
  - **API key (deployments):** `CODEX_API_KEY` (or `OPENAI_API_KEY`) in the environment of `serve`;
    each workspace gets its own empty `CODEX_HOME`.

  With neither, the server still starts and a turn fails visibly with `401 Unauthorized`.
- **Cost.** The fake runtime (`--fake`) and every `cargo test` use no model. `smoke` refuses to send a
  message to a real Codex unless you pass `--spend-tokens` / `-SpendTokens` (with an account login it
  uses your plan's usage instead of billed tokens, but it is still a real model call).
- Default port 8787 (`serve … -- --port N` / `-ServeArgs '--port','N'`; then `smoke --base
  http://127.0.0.1:N/api/v1`). With `--dev` the fixture mode at `/api/fixtures/v1` uses its own runtime
  state.

Expected `smoke` output with the fake runtime and a message containing `tool` and `approval`:

```text
  [1] message.user: …
  [2] status -> running  runtime turn started
  [3] tool: sources_search
  [5] approval requested: write files: write notes.md -> approved
  [6] status -> waiting_approval …
  [7] approval.resolved
  [10] message.completed: item/fileChange/requestApproval -> accept
  [11] status -> completed  turn completed
OK: the turn completed
```

## Models on ChatGPT accounts

With a ChatGPT (account) login, OpenAI refused every model for Codex 0.141.0 (`400 model_not_found
... not supported when using Codex with a ChatGPT account`), although `model/list` offered them. The
pin moved to 0.169.0, with which a real turn through the desktop app's login (`--use-app-login`, no
model override) ran and called an IGNIS MCP tool. If a later client hits the same error, pin a newer
release (the preflight regenerates and checks its protocol schema) or use an API key.
