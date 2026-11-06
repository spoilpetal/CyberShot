$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Bin = Join-Path $Root 'target\release\cybershot.exe'
$InstallDir = if ($env:CYBERSHOT_INSTALL_DIR) { $env:CYBERSHOT_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Programs\CyberShot' }
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'Rust/Cargo is required. Install it from https://rustup.rs/.' }
Push-Location $Root
try { cargo build --locked --release } finally { Pop-Location }
if (-not (Test-Path $Bin)) { throw "Build did not produce $Bin" }

$Choice = Read-Host 'Install command names: 1) cybershot  2) csb  3) both [3]'
if ([string]::IsNullOrWhiteSpace($Choice)) { $Choice = '3' }
if ($Choice -notin @('1','2','3')) { Write-Host 'Cancelled.'; exit 0 }
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$Targets = @()
if ($Choice -in @('1','3')) { $Targets += Join-Path $InstallDir 'cybershot.exe' }
if ($Choice -in @('2','3')) { $Targets += Join-Path $InstallDir 'csb.exe' }
$Conflicts = @($Targets | Where-Object { Test-Path $_ })
if ($Conflicts.Count -gt 0) {
  Write-Host 'Existing command files found:'
  $Conflicts | ForEach-Object { Write-Host "  $_" }
  $Confirm = Read-Host 'Replace these files? [y/N]'
  if ($Confirm -notmatch '^[Yy]$') { Write-Host 'Cancelled; existing files preserved.'; exit 0 }
}
foreach ($Target in $Targets) { Copy-Item $Bin $Target -Force }
$ShortcutExe = if ($Choice -eq '1') { Join-Path $InstallDir 'cybershot.exe' } else { Join-Path $InstallDir 'csb.exe' }
$UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$UserPathParts = @()
if (-not [string]::IsNullOrWhiteSpace($UserPath)) { $UserPathParts = @($UserPath -split ';' | Where-Object { $_ }) }
if ($UserPathParts -notcontains $InstallDir) {
  [Environment]::SetEnvironmentVariable('Path', (($UserPathParts + $InstallDir) -join ';'), 'User')
  Write-Host 'Added install directory to your user PATH. Open a new terminal to use it.'
}
Write-Host "Installed to $InstallDir. Create a Windows shortcut with Target: $ShortcutExe and Arguments: hotkey; assign its Shortcut key."
Write-Host "Enable the trigger with: `"$ShortcutExe`" keybind enable"
