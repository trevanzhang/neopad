$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$toolsDir = Join-Path $repoRoot 'target\tools'
$edgeDriver = Join-Path $toolsDir 'msedgedriver.exe'

New-Item -ItemType Directory -Force -Path $toolsDir | Out-Null

if (-not (Get-Command tauri-driver -ErrorAction SilentlyContinue)) {
    cargo install tauri-driver --locked
}

if (-not (Get-Command msedgedriver-tool -ErrorAction SilentlyContinue)) {
    cargo install --git https://github.com/chippers/msedgedriver-tool --locked
}

if (-not (Test-Path -LiteralPath $edgeDriver)) {
    Push-Location $toolsDir
    try {
        msedgedriver-tool
    }
    finally {
        Pop-Location
    }
}

Write-Output "Tauri driver: $((Get-Command tauri-driver).Source)"
Write-Output "Edge driver: $edgeDriver"
