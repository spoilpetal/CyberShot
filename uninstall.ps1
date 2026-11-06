$ErrorActionPreference = 'Stop'
$InstallDir = if ($env:CYBERSHOT_INSTALL_DIR) { $env:CYBERSHOT_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Programs\CyberShot' }
$Choice = Read-Host 'Remove 1) cybershot  2) csb  3) both  4) cancel'
switch ($Choice) {
  '1' { Remove-Item (Join-Path $InstallDir 'cybershot.exe') -Force -ErrorAction SilentlyContinue }
  '2' { Remove-Item (Join-Path $InstallDir 'csb.exe') -Force -ErrorAction SilentlyContinue }
  '3' { Remove-Item (Join-Path $InstallDir 'cybershot.exe'), (Join-Path $InstallDir 'csb.exe') -Force -ErrorAction SilentlyContinue }
  default { Write-Host 'Cancelled.'; exit 0 }
}
if (-not (Test-Path (Join-Path $InstallDir 'cybershot.exe')) -and -not (Test-Path (Join-Path $InstallDir 'csb.exe'))) {
  $UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
  $Parts = @($UserPath -split ';' | Where-Object { $_ -and $_ -ne $InstallDir })
  [Environment]::SetEnvironmentVariable('Path', ($Parts -join ';'), 'User')
  if (Test-Path $InstallDir) {
    if (@(Get-ChildItem -Force $InstallDir).Count -eq 0) { Remove-Item $InstallDir -Force }
  }
}
Write-Host 'Selected command(s) removed. User settings and screenshots were preserved.'
