# Build the analysis runner image and print the id to pin (IGNIS_RUNNER_IMAGE).
param([string]$Docker = "docker")
$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
& $Docker build --pull=false -t ignis-runner:local (Join-Path $root "infra\runner") | Out-Host
if ($LASTEXITCODE -ne 0) { throw "docker build failed" }
$id = & $Docker image inspect ignis-runner:local --format '{{.Id}}'
"IGNIS_RUNNER_IMAGE=$id"
