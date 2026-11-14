<#
Set up the pinned Codex App Server for IGNIS on Windows.

  tools\codex\setup.ps1            # find an existing codex install of the pinned version
  tools\codex\setup.ps1 -Install   # install it with npm into runs\codex\npm first (downloads ~100 MB)

Writes runs\codex\codex.env with IGNIS_CODEX_BIN (the native codex.exe, not the npm shim) and
IGNIS_CODEX_SHA256. tools\codex\serve.ps1 reads it. Nothing here calls a model.
#>
param([switch]$Install)
$ErrorActionPreference = 'Stop'

$Pin = '0.169.0'   # keep equal to ignis_research::runtime::PINNED_CODEX_VERSION
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$Out = Join-Path $Root 'runs\codex'
$NpmDir = Join-Path $Out 'npm'
New-Item -ItemType Directory -Force $Out | Out-Null

if ($Install) {
    if (-not (Get-Command npm -ErrorAction SilentlyContinue)) { throw 'npm is required for -Install (Node.js >= 16)' }
    Write-Host "installing @openai/codex@$Pin into $NpmDir"
    npm install --prefix $NpmDir --no-audit --no-fund "@openai/codex@$Pin" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'npm install failed' }
}

# The npm package ships the real executable at <platform package>\vendor\<target triple>\bin\codex.exe.
function Find-Native([string]$NodeModules) {
    $scope = Join-Path $NodeModules '@openai'
    if (-not (Test-Path $scope)) { return $null }
    Get-ChildItem -Path $scope -Recurse -File -Filter 'codex.exe' -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match '\\vendor\\[^\\]+\\bin\\codex\.exe$' } |
        Select-Object -First 1 -ExpandProperty FullName
}

$Bin = $env:IGNIS_CODEX_BIN
if (-not $Bin) { $Bin = Find-Native (Join-Path $NpmDir 'node_modules') }
if (-not $Bin -and (Get-Command npm -ErrorAction SilentlyContinue)) { $Bin = Find-Native (npm root -g) }
if (-not $Bin -or -not (Test-Path $Bin)) {
    throw "no codex $Pin binary found. Run: tools\codex\setup.ps1 -Install  (or set IGNIS_CODEX_BIN)"
}

$Version = ((& $Bin --version) | Select-Object -First 1).Split(' ')[-1]
if ($Version -ne $Pin) { throw "$Bin is codex $Version, IGNIS is pinned to $Pin. Run: tools\codex\setup.ps1 -Install" }

$Sha = (Get-FileHash -Algorithm SHA256 $Bin).Hash.ToLower()
@(
    '# Written by tools\codex\setup.ps1. Machine-specific; not committed (runs\ is ignored).'
    "IGNIS_CODEX_BIN=$Bin"
    "IGNIS_CODEX_SHA256=$Sha"
) | Set-Content -Encoding utf8 (Join-Path $Out 'codex.env')

Write-Host "codex ${Version}: $Bin"
Write-Host "sha256:   $Sha"
Write-Host "wrote     $(Join-Path $Out 'codex.env')"
if ($env:CODEX_API_KEY -or $env:OPENAI_API_KEY) {
    Write-Host 'credentials: an API key is set in this shell (forwarded to the runtime)'
} else {
    Write-Host 'credentials: none set. Real turns need CODEX_API_KEY (or OPENAI_API_KEY) in the server''s environment;'
    Write-Host '             the fake runtime (tools\codex\serve.ps1 -Fake) needs none and costs nothing.'
}
