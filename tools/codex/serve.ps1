<#
Start `ignis serve --dev` with the agent runtime (Windows).

  tools\codex\serve.ps1                      # real Codex from runs\codex\codex.env (run setup.ps1 first)
  tools\codex\serve.ps1 -Fake                # the offline fake app-server: no model, no credentials, no cost
  tools\codex\serve.ps1 -UseCodexAppLogin    # reuse the login of your own Codex/ChatGPT app (~\.codex)
  tools\codex\serve.ps1 -ServeArgs '--port','9000'

Real turns use your Codex account (tools\codex\login.ps1, or -UseCodexAppLogin) or
CODEX_API_KEY / OPENAI_API_KEY.
#>
param([switch]$Fake, [switch]$UseCodexAppLogin, [string[]]$ServeArgs = @())
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Set-Location $Root

if ($Fake) {
    cargo build -q -p ignis-research --bin ignis-fake-codex
    if ($LASTEXITCODE -ne 0) { throw 'build of the fake app-server failed' }
    $Target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }
    $env:IGNIS_CODEX_BIN = Join-Path $Target 'debug\ignis-fake-codex.exe'
    Remove-Item Env:IGNIS_CODEX_SHA256 -ErrorAction SilentlyContinue
    $State = 'runs/server-fake-codex'
} else {
    $EnvFile = Join-Path $Root 'runs\codex\codex.env'
    if (-not (Test-Path $EnvFile)) { throw "missing ${EnvFile}: run tools\codex\setup.ps1 first" }
    foreach ($line in Get-Content $EnvFile) {
        if ($line -match '^\s*([A-Z_][A-Z0-9_]*)=(.*)$') { Set-Item "Env:$($Matches[1])" $Matches[2] }
    }
    $State = 'runs/server'
    $LoginHome = Join-Path $Root 'runs\codex\login'
    $AppHome = if ($env:CODEX_HOME) { $env:CODEX_HOME } else { Join-Path $env:USERPROFILE '.codex' }
    if ($UseCodexAppLogin) {
        # The login of your own Codex / ChatGPT app, used in place (never copied): its config.toml
        # applies too and the runtime's conversations appear in that app's history.
        if (-not (Test-Path (Join-Path $AppHome 'auth.json'))) { throw "no Codex login in $AppHome" }
        $env:IGNIS_CODEX_LOGIN_HOME = $AppHome
        Write-Host "login: your Codex app's login at $AppHome (shared, not copied)"
    } elseif (Test-Path (Join-Path $LoginHome 'auth.json')) {
        # Signed in with tools\codex\login.ps1 (local development).
        $env:IGNIS_CODEX_LOGIN_HOME = $LoginHome
        Write-Host "login: Codex account signed in at $LoginHome"
    } elseif (-not ($env:CODEX_API_KEY -or $env:OPENAI_API_KEY)) {
        Write-Warning 'not signed in (tools\codex\login.ps1) and no CODEX_API_KEY/OPENAI_API_KEY: real turns will fail to authenticate'
    }
}

Write-Host "codex: $env:IGNIS_CODEX_BIN"
cargo run -q -p ignis-cli -- serve --dev --state-dir $State @ServeArgs
exit $LASTEXITCODE
