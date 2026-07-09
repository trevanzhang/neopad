$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$releaseDir = Join-Path $repoRoot "target\release"
$source = Join-Path $releaseDir "neopad-mcp.exe"

if (!(Test-Path -LiteralPath $source)) {
  throw "Missing MCP release binary: $source. Run cargo build -p neopad-mcp --release first."
}

$rustcInfo = rustc -vV
$hostLine = $rustcInfo | Where-Object { $_ -like "host: *" } | Select-Object -First 1
if (!$hostLine) {
  throw "Could not determine Rust host target from rustc -vV."
}

$targetTriple = $hostLine.Substring("host: ".Length).Trim()
$target = Join-Path $releaseDir "neopad-mcp-$targetTriple.exe"

Copy-Item -LiteralPath $source -Destination $target -Force
Write-Host "Prepared MCP sidecar: $target"
