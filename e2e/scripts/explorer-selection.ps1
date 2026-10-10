param(
  [Parameter(Mandatory = $true)][string]$Path,
  [switch]$Close,
  [switch]$Diagnose
)

[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
$targetPath = [System.IO.Path]::GetFullPath($Path)
$targetParent = [System.IO.Path]::GetDirectoryName($targetPath)
if ($Close) {
  $workspacePath = [System.IO.Path]::GetFullPath($env:NEOPAD_WORKSPACE).TrimEnd('\') + '\'
  if (-not $targetPath.StartsWith($workspacePath, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'Only Explorer windows for the isolated test workspace may be closed.'
  }
}
$shell = New-Object -ComObject Shell.Application
$result = $null
$observedWindows = @()
foreach ($window in @($shell.Windows())) {
  try {
    $parentPath = $window.Document.Folder.Self.Path
    $selectedPaths = @($window.Document.SelectedItems() | ForEach-Object { $_.Path })
    $observed = @{ folder = $parentPath; selected = $selectedPaths }
    if ($Diagnose) {
      $observed.selectedCount = $window.Document.SelectedItems().Count
      $observed.items = @($window.Document.Folder.Items() | ForEach-Object { $_.Path })
    }
    $observedWindows += $observed
    if ($parentPath -ne $targetParent) { continue }
    # A failed selection must not leave its test Explorer window behind.
    # The workspace guard above limits cleanup to the isolated test data.
    if ($Close) { $window.Quit(); continue }
    if ($selectedPaths -contains $targetPath) {
      $result = @{ folder = $parentPath; selected = $selectedPaths }
      break
    }
  } catch { $observedWindows += @{ error = $_.Exception.Message } }
}
if ($Diagnose) {
  ConvertTo-Json -Depth 5 -Compress -InputObject @{
    target = $targetPath
    windows = $observedWindows
    session = (Get-Process -Id $PID).SessionId
    explorer = @(Get-Process explorer -ErrorAction SilentlyContinue | Select-Object Id, SessionId)
  }
  exit
}
if ($null -eq $result) { Write-Output 'null' }
else { ConvertTo-Json -InputObject $result -Compress }
