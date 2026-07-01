$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$cargoText = Get-Content -LiteralPath (Join-Path $root 'Cargo.toml') -Raw
$cargoMatch = [regex]::Match($cargoText, '(?ms)^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"')

if (-not $cargoMatch.Success) {
  throw 'Could not read workspace.package.version from Cargo.toml'
}

$versions = [ordered]@{
  'Cargo workspace' = $cargoMatch.Groups[1].Value
  'Root package' = (Get-Content -LiteralPath (Join-Path $root 'package.json') -Raw | ConvertFrom-Json).version
  'Frontend package' = (Get-Content -LiteralPath (Join-Path $root 'app/package.json') -Raw | ConvertFrom-Json).version
  'Tauri config' = (Get-Content -LiteralPath (Join-Path $root 'app/src-tauri/tauri.conf.json') -Raw | ConvertFrom-Json).version
}

$expected = $versions['Cargo workspace']
$mismatches = $versions.GetEnumerator() | Where-Object { $_.Value -ne $expected }

if ($mismatches) {
  $details = ($versions.GetEnumerator() | ForEach-Object { "$($_.Key)=$($_.Value)" }) -join ', '
  throw "NeoPad version mismatch: $details"
}

Write-Output "NeoPad versions are synchronized at $expected"
