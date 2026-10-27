# Only accepts a -TestPackage build; uses private registry identity and a new
# random test directory. It never installs the production package identity.
param(
 [Parameter(Mandatory=$true)][string]$Setup,
 [Parameter(Mandatory=$true)][string]$PortableZip,
 [Parameter(Mandatory=$true)][string]$WorkRoot
)
$ErrorActionPreference='Stop'
if (-not (Test-Path -LiteralPath $WorkRoot -PathType Container)) {throw 'WorkRoot must already exist.'}
$Setup=(Resolve-Path -LiteralPath $Setup).Path;$PortableZip=(Resolve-Path -LiteralPath $PortableZip).Path
if ([Diagnostics.FileVersionInfo]::GetVersionInfo($Setup).ProductName -ne 'Lieve (packaging test)') {throw 'Only a -TestPackage installer is accepted.'}
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class LievePackageSmoke {
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern IntPtr FindWindow(string c,string t);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h,out uint p);
 [DllImport("user32.dll")] static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 public static uint WindowPid() {uint p;GetWindowThreadProcessId(FindWindow("LieveNativeIsland",null),out p);return p;}
 public static bool Close(uint expected) {var h=FindWindow("LieveNativeIsland",null);uint p;GetWindowThreadProcessId(h,out p);return p==expected && PostMessage(h,0x10,IntPtr.Zero,IntPtr.Zero);}
}
'@
$registry=[Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::CurrentUser,[Microsoft.Win32.RegistryView]::Registry64)
$uninstallKey='Software\Microsoft\Windows\CurrentVersion\Uninstall\LievePackagingSmoke'
$privateKey='Software\Lieve\PackagingSmoke'
$startupKey=$privateKey+'\Run'
$realRun='Software\Microsoft\Windows\CurrentVersion\Run'
foreach ($path in @($uninstallKey,$privateKey)) {$key=$registry.OpenSubKey($path);if ($key) {$key.Dispose();throw "A previous/private test key already exists: $path"}}
function Value($path,$name) {$key=$registry.OpenSubKey($path);if (-not $key) {return $null};try {return $key.GetValue($name,$null,[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)}finally{$key.Dispose()}}
function Startup-Snapshot {return (Value $realRun 'Lieve' | ConvertTo-Json -Compress)}
function Invoke-Setup($arguments) {
 $process=Start-Process -FilePath $Setup -ArgumentList $arguments -PassThru -Wait
 if ($process.ExitCode -ne 0) {throw "Installer exited with $($process.ExitCode)."}
}
function Invoke-Uninstall {
 $uninstaller=Join-Path $install 'Uninstall.exe'
 $process=Start-Process -FilePath $uninstaller -ArgumentList '/S' -PassThru -Wait
 if ($process.ExitCode -ne 0) {throw "Uninstaller exited with $($process.ExitCode)."}
 $watch=[Diagnostics.Stopwatch]::StartNew()
 while ((Test-Path -LiteralPath $uninstaller) -and $watch.ElapsedMilliseconds -lt 5000) {Start-Sleep -Milliseconds 100}
}
function Check-Installed {
 foreach ($file in @('lieve.exe','README.txt','build-info.json','Uninstall.exe')) {if (-not (Test-Path -LiteralPath (Join-Path $install $file))) {throw "Installed file missing: $file"}}
 if ((Get-FileHash -LiteralPath (Join-Path $install 'lieve.exe') -Algorithm SHA256).Hash -ne $info.exeSha256) {throw 'Installed executable differs from the portable payload.'}
 if ((Value $uninstallKey 'InstallLocation') -ne $install) {throw 'Uninstall registration has wrong location.'}
 if ((Value $uninstallKey 'UninstallString') -ne ('"'+(Join-Path $install 'Uninstall.exe')+'"')) {throw 'Uninstall command is not correctly quoted.'}
 foreach ($file in @('Lieve.lnk','Uninstall.lnk')) {if (-not (Test-Path -LiteralPath (Join-Path $menu $file))) {throw "Start menu shortcut missing: $file"}}
 $shell=New-Object -ComObject WScript.Shell
 try {$shortcut=$shell.CreateShortcut((Join-Path $menu 'Lieve.lnk'));if ($shortcut.TargetPath -ne (Join-Path $install 'lieve.exe')) {throw 'Start menu shortcut target mismatch.'}}
 finally {if ($shortcut) {[void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($shortcut)};[void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($shell)}
}
function Start-Installed {
 $process=Start-Process -FilePath (Join-Path $install 'lieve.exe') -WorkingDirectory $install -PassThru
 $watch=[Diagnostics.Stopwatch]::StartNew()
 while ([LievePackageSmoke]::WindowPid() -ne $process.Id -and -not $process.HasExited -and $watch.ElapsedMilliseconds -lt 5000) {Start-Sleep -Milliseconds 100}
 if ($process.HasExited -or [LievePackageSmoke]::WindowPid() -ne $process.Id) {throw 'Installed executable did not start its window.'}
 return $process
}
function Set-TestStartup($text) {$key=$registry.CreateSubKey($startupKey);try {$key.SetValue('Lieve',$text,[Microsoft.Win32.RegistryValueKind]::String)}finally{$key.Dispose()}}
function Check-Preserved {
 foreach ($file in $dataHashes.Keys) {if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -ne $dataHashes[$file]) {throw "Fixture data changed: $file"}}
 if (-not (Test-Path -LiteralPath (Join-Path $install 'keep-me.txt'))) {throw 'Uninstaller deleted a non-package file.'}
 foreach ($file in @('lieve.exe','README.txt','build-info.json','Uninstall.exe')) {if (Test-Path -LiteralPath (Join-Path $install $file)) {throw "Program file remained after uninstall: $file"}}
 if ($null -ne (Value $uninstallKey 'InstallLocation')) {throw 'Uninstall registration remained.'}
 if (Test-Path -LiteralPath $menu) {throw 'Installer-owned Start menu shortcuts remained.'}
 if ((Startup-Snapshot) -ne $realStartup) {throw 'The actual Windows startup entry changed.'}
}
$test=Join-Path ([IO.Path]::GetFullPath($WorkRoot)) ('lieve-package-smoke-'+[guid]::NewGuid().ToString('N'))
[void](New-Item -ItemType Directory -Path $test)
$install=Join-Path $test 'Install with spaces'
$portable=Join-Path $test 'Portable'
$profile=Join-Path $test 'Profile'
$menu=Join-Path ([Environment]::GetFolderPath('Programs')) 'Lieve (packaging test)'
if (Test-Path -LiteralPath $menu) {throw 'The test Start menu folder already exists.'}
$original=$env:APPDATA;$owned=$null;$restored=$null;$closedExisting=$false;$realStartup=Startup-Snapshot
$existing=Get-Process lieve -ErrorAction SilentlyContinue
$development=Join-Path (Split-Path -Parent $PSScriptRoot) 'target\release\lieve.exe'
if ($existing -and (@($existing).Count -ne 1 -or $existing.Path -ne $development)) {throw 'Unexpected Lieve process; stop the test before installing.'}
try {
 Expand-Archive -LiteralPath $PortableZip -DestinationPath $portable
 $info=Get-Content -LiteralPath (Join-Path $portable 'build-info.json') -Raw | ConvertFrom-Json
 if (-not $info.testPackage) {throw 'Portable payload must also be a -TestPackage build.'}
 if ((Get-FileHash -LiteralPath (Join-Path $portable 'lieve.exe') -Algorithm SHA256).Hash -ne $info.exeSha256) {throw 'Portable hash mismatch.'}
 $data=Join-Path $profile 'Lieve';[void](New-Item -ItemType Directory -Force -Path $data)
 [IO.File]::WriteAllText((Join-Path $data 'settings.txt'),"motion=reduced`ntransparency=0`ncompact_lyrics=false`nspectrum_enabled=false`nfullscreen_auto_hide=false`n")
 foreach ($name in @('tasks.txt','shelf.txt','water.txt')) {[IO.File]::WriteAllText((Join-Path $data $name),"packaging-fixture-$name")}
 $dataHashes=@{};foreach ($file in Get-ChildItem -LiteralPath $data -File) {$dataHashes[$file.FullName]=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash}
 $env:APPDATA=$profile
 Invoke-Setup ('/S /D='+$install);Check-Installed
 if ($existing -and $existing.HasExited) {throw 'Installer closed a different development/portable copy.'}
 'Fresh per-user install, quoted paths, Start menu and portable/installed hash identity: PASS'
 if ($existing) {
  if (-not [LievePackageSmoke]::Close($existing.Id) -or -not $existing.WaitForExit(3000)) {throw 'Could not close the development instance for installed-launch testing.'}
  $closedExisting=$true
 }
 $owned=Start-Installed
 'Installed standalone executable launches with isolated APPDATA: PASS'
 $ownStartup='"'+(Join-Path $install 'lieve.exe')+'"';Set-TestStartup $ownStartup
 Invoke-Setup '/S';Check-Installed
 if (-not $owned.WaitForExit(3000)) {throw 'Update did not gracefully close its own running app.'};$owned=$null
 if ((Value $startupKey 'Lieve') -ne $ownStartup) {throw 'Update changed the startup choice.'}
 'Update reuses custom install path, closes only its own app, preserves data/startup: PASS'
 [IO.File]::WriteAllText((Join-Path $install 'keep-me.txt'),'not owned by installer')
 $owned=Start-Installed;Invoke-Uninstall
 if (-not $owned.WaitForExit(3000)) {throw 'Uninstall did not close its own app.'};$owned=$null
 Check-Preserved
 if ($null -ne (Value $startupKey 'Lieve')) {throw 'Owned startup entry was not removed.'}
 'Uninstall closes its app, removes owned files/registration/startup, preserves data and unknown files: PASS'
 Invoke-Setup ('/S /D='+$install);Check-Installed
 $otherStartup='"'+(Join-Path $test 'Another copy\lieve.exe')+'"';Set-TestStartup $otherStartup
 if ($existing) {$env:APPDATA=$original;$restored=Start-Process -FilePath $development -PassThru;$env:APPDATA=$profile;Start-Sleep -Milliseconds 1500}
 Invoke-Uninstall;Check-Preserved
 if ((Value $startupKey 'Lieve') -ne $otherStartup) {throw 'Uninstall removed another copy startup registration.'}
 if ($restored -and $restored.HasExited) {throw 'Uninstall closed another running copy.'}
 'Uninstall preserves another copy and its startup registration: PASS'
 'Actual Windows startup value and user APPDATA were not modified by the installer tests.'
}finally {
 try {
  if ($owned -and -not $owned.HasExited) {[void][LievePackageSmoke]::Close($owned.Id);[void]$owned.WaitForExit(3000)}
  if ((Value $uninstallKey 'InstallLocation') -eq $install -and (Test-Path -LiteralPath (Join-Path $install 'Uninstall.exe'))) {Invoke-Uninstall}
 }finally {
  $env:APPDATA=$original
  try {
   $keepTest=(Value $uninstallKey 'InstallLocation') -eq $install
   $registry.DeleteSubKeyTree($privateKey,$false)
  }finally {
   $registry.Dispose()
   if ($closedExisting -and (-not $restored -or $restored.HasExited)) {Start-Process -FilePath $development}
   if (-not $keepTest -and (Test-Path -LiteralPath $test)) {Remove-Item -LiteralPath $test -Recurse -Force}
  }
 }
}
