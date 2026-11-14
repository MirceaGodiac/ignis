<#
Sign the IGNIS runtime in with your Codex (ChatGPT) account, once per machine (Windows).
Local development only: deployments use an API key.

  tools\codex\login.ps1           # opens the browser sign-in of `codex login`
  tools\codex\login.ps1 -Status   # only show whether it is signed in
  tools\codex\login.ps1 -Force    # sign in again (starting a login first clears the current one)

The login lives in runs\codex\login (its own CODEX_HOME, ignored by git), not in your personal
~\.codex: IGNIS never shares or refreshes the login your own Codex uses. serve.ps1 picks it up.
Run setup.ps1 first.
#>
param([switch]$Status, [switch]$Force)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$EnvFile = Join-Path $Root 'runs\codex\codex.env'
if (-not (Test-Path $EnvFile)) { throw "missing ${EnvFile}: run tools\codex\setup.ps1 first" }
$Bin = (Get-Content $EnvFile | Where-Object { $_ -like 'IGNIS_CODEX_BIN=*' } | Select-Object -First 1).Substring(16)
$LoginHome = Join-Path $Root 'runs\codex\login'
New-Item -ItemType Directory -Force $LoginHome | Out-Null
$env:CODEX_HOME = $LoginHome
# Keep the credentials a file in that home (not the OS keyring), as the runtime expects.
$store = 'cli_auth_credentials_store="file"'
if (-not $Status -and -not $Force -and (Test-Path (Join-Path $LoginHome 'auth.json'))) {
    # `codex login` clears the existing login as soon as it starts; never do that by accident.
    Write-Host 'already signed in (use -Force to sign in again)'
    $Status = $true
}
if (-not $Status) {
    & $Bin -c $store login
    if ($LASTEXITCODE -ne 0) { throw 'codex login failed' }
}
& $Bin -c $store login status
if (Test-Path (Join-Path $LoginHome 'auth.json')) {
    Write-Host "signed in: tools\codex\serve.ps1 will use $LoginHome"
} else {
    Write-Host 'not signed in'
    exit 1
}
