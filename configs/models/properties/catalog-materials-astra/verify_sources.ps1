$ErrorActionPreference = 'Stop'
$catalogRoot = $PSScriptRoot
$catalogPath = Join-Path $catalogRoot 'materials.json'
$expectedCatalogHash = (Get-Content -LiteralPath (Join-Path $catalogRoot 'materials.sha256') -Raw).Trim()
$actualCatalogHash = (Get-FileHash -LiteralPath $catalogPath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actualCatalogHash -cne $expectedCatalogHash) { throw 'Material catalog hash mismatch' }
$catalog = Get-Content -LiteralPath $catalogPath -Raw | ConvertFrom-Json
$sourceIndex = Get-Content -LiteralPath (Join-Path $catalogRoot 'source-index.json') -Raw | ConvertFrom-Json -AsHashtable
foreach ($source in $catalog.sources) {
    if (-not $sourceIndex.ContainsKey($source.id)) { throw "Missing source file mapping: $($source.id)" }
    $sourcePath = [System.IO.Path]::GetFullPath((Join-Path $catalogRoot $sourceIndex[$source.id]))
    if (-not $sourcePath.StartsWith($catalogRoot + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Source outside catalog directory: $($source.id)"
    }
    $actualHash = (Get-FileHash -LiteralPath $sourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualHash -cne $source.sha256) { throw "Source hash mismatch: $($source.id)" }
    Write-Output "Verified $($source.id): $actualHash"
}
if ($sourceIndex.Count -ne $catalog.sources.Count) { throw 'Unreferenced source file mapping' }
Write-Output "Verified material catalog: $actualCatalogHash"
