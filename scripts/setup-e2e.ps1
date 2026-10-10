$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$toolsDir = Join-Path $repoRoot 'target\tools'
$edgeDriver = Join-Path $toolsDir 'msedgedriver.exe'

New-Item -ItemType Directory -Force -Path $toolsDir | Out-Null

# Match the driver version locally and in CI; Cargo skips an existing match.
cargo install tauri-driver --version 2.1.0 --locked
if ($LASTEXITCODE -ne 0) { throw 'Failed to install tauri-driver 2.1.0' }

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
& $edgeDriver --version
whoami /groups | Select-String 'S-1-16-'
