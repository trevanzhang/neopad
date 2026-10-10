param(
  [Parameter(Mandatory = $true)][string]$Workspace,
  [switch]$Close
)

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$expected = [System.IO.Path]::GetFullPath((Join-Path $repoRoot 'target\e2e-workspace')).TrimEnd('\')
$workspacePath = [System.IO.Path]::GetFullPath($Workspace).TrimEnd('\')
if ($workspacePath -ne $expected) { throw 'Explorer fixtures must stay in target/e2e-workspace.' }
$shell = New-Object -ComObject Shell.Application

if ($Close) {
  foreach ($window in @($shell.Windows())) {
    try {
      if ($window.Document.Folder.Self.Path -eq $workspacePath) { $window.Quit() }
    } catch { }
  }
  exit
}

# Hosted runners can start without a desktop Shell. Initialize its folder view
# using an unrelated fixture directory before testing reveal-and-select once.
# Keep this view alive for the suite, then close only this fixture in teardown.
$before = @(Get-Process explorer -ErrorAction SilentlyContinue | Select-Object Id, SessionId)
Start-Process -FilePath (Join-Path $env:WINDIR 'explorer.exe') -ArgumentList ('"{0}"' -f $workspacePath) -WindowStyle Hidden
$deadline = [DateTime]::UtcNow.AddSeconds(30)
do {
  foreach ($window in @($shell.Windows())) {
    try {
      if ($window.Document.Folder.Self.Path -eq $workspacePath -and -not $window.Busy) {
        ConvertTo-Json -Compress -Depth 4 -InputObject @{
          folder = $workspacePath
          session = (Get-Process -Id $PID).SessionId
          explorerBefore = $before
        }
        exit
      }
    } catch { }
  }
  Start-Sleep -Milliseconds 200
} while ([DateTime]::UtcNow -lt $deadline)
throw 'Explorer did not initialize the isolated test workspace view.'
