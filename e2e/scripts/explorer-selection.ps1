param(
  [Parameter(Mandatory = $true)][string]$Path,
  [switch]$Close
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
foreach ($window in @($shell.Windows())) {
  try {
    $parentPath = $window.Document.Folder.Self.Path
    if ($parentPath -ne $targetParent) { continue }
    $selectedPaths = @($window.Document.SelectedItems() | ForEach-Object { $_.Path })
    if ($selectedPaths -contains $targetPath) {
      $result = @{ folder = $parentPath; selected = $selectedPaths }
      if ($Close) { $window.Quit() }
      break
    }
  } catch { }
}
if ($null -eq $result) { Write-Output 'null' }
else { ConvertTo-Json -InputObject $result -Compress }
