# Build from any working directory. Windows PowerShell 5.1 or later.
param(
 [string]$NsisPath,
 [string]$Toolchain,
 [string]$Target,
 [string]$Executable,
 [string]$OutputDirectory,
 [switch]$SkipBuild,
 [switch]$PortableOnly,
 [switch]$TestPackage
)
$ErrorActionPreference='Stop'
$nsisVersion=$null
if (-not $Target -and $env:CARGO_BUILD_TARGET) {$Target=$env:CARGO_BUILD_TARGET}
$native=Split-Path -Parent $PSScriptRoot
$manifest=Join-Path $native 'Cargo.toml'
$versionMatch=[regex]::Match([IO.File]::ReadAllText($manifest),'(?m)^version\s*=\s*"(\d+\.\d+\.\d+)"')
if (-not $versionMatch.Success) {throw 'Cargo.toml needs a numeric three-part version.'}
$version=$versionMatch.Groups[1].Value
if (-not $OutputDirectory) {
 $OutputDirectory=Join-Path $native 'dist'
 if ($TestPackage) {
  if (-not (Test-Path -LiteralPath $OutputDirectory)) {[void](New-Item -ItemType Directory -Path $OutputDirectory)}
  $OutputDirectory=Join-Path $OutputDirectory 'tests'
 }
}
$OutputDirectory=[IO.Path]::GetFullPath($OutputDirectory)
if (-not (Test-Path -LiteralPath (Split-Path -Parent $OutputDirectory) -PathType Container)) {throw 'Output parent directory must exist.'}
if (-not $PortableOnly) {
 if (-not $NsisPath) {
  $command=Get-Command makensis.exe -ErrorAction SilentlyContinue
  if ($command) {$NsisPath=$command.Source}
  else {
   foreach ($folder in @(${env:ProgramFiles(x86)},$env:ProgramFiles)) {
    if ($folder -and (Test-Path -LiteralPath (Join-Path $folder 'NSIS\makensis.exe'))) {$NsisPath=Join-Path $folder 'NSIS\makensis.exe';break}
   }
  }
 }
 if (-not $NsisPath -or -not (Test-Path -LiteralPath $NsisPath -PathType Leaf)) {throw 'NSIS 3 is required. Pass -NsisPath <makensis.exe>, or use -PortableOnly.'}
 $NsisPath=(Resolve-Path -LiteralPath $NsisPath).Path
 $nsisVersion=(& $NsisPath /VERSION | Out-String).Trim()
 if ($LASTEXITCODE -ne 0) {throw 'NSIS compiler could not start.'}
}
if (-not $SkipBuild) {
 $cargo=@();if ($Toolchain) {$cargo+='+'+$Toolchain};$cargo+=@('build','--release','--locked','--manifest-path',$manifest)
 if ($Target) {$cargo+=@('--target',$Target)}
 & cargo @cargo
 if ($LASTEXITCODE -ne 0) {throw 'Release build failed. Close the running build first, or package an already-built executable with -SkipBuild.'}
}
if (-not $Executable) {
 $targetDirectory=if ($env:CARGO_TARGET_DIR) {$env:CARGO_TARGET_DIR}else{Join-Path $native 'target'}
 if ($Target) {$targetDirectory=Join-Path $targetDirectory $Target}
 $Executable=Join-Path $targetDirectory 'release\lieve.exe'
}
$Executable=(Resolve-Path -LiteralPath $Executable).Path

function Get-WindowsImports([string]$Path) {
 $stream=[IO.File]::OpenRead($Path);$reader=New-Object IO.BinaryReader $stream
 try {
  if ($reader.ReadUInt16() -ne 0x5a4d) {throw 'Not a Windows executable.'}
  [void]$stream.Seek(0x3c,0);$pe=$reader.ReadUInt32();[void]$stream.Seek($pe,0)
  if ($reader.ReadUInt32() -ne 0x4550 -or $reader.ReadUInt16() -ne 0x8664) {throw 'Packaging requires a Windows x64 executable.'}
  $sectionCount=$reader.ReadUInt16();[void]$stream.Seek($pe+20,0);$optionalSize=$reader.ReadUInt16();$optional=$pe+24
  [void]$stream.Seek($optional,0);if ($reader.ReadUInt16() -ne 0x20b) {throw 'Expected a PE32+ image.'}
  [void]$stream.Seek($optional+68,0);if ($reader.ReadUInt16() -ne 2) {throw 'Expected a Windows GUI executable.'}
  [void]$stream.Seek($optional+120,0);$importRva=$reader.ReadUInt32();$importSize=$reader.ReadUInt32()
  $sections=@()
  for ($i=0;$i -lt $sectionCount;$i++) {
   [void]$stream.Seek($optional+$optionalSize+40*$i+8,0)
   $virtual=$reader.ReadUInt32();$address=$reader.ReadUInt32();$rawSize=$reader.ReadUInt32();$raw=$reader.ReadUInt32()
   $sections+=@{Address=$address;Size=[math]::Max($virtual,$rawSize);Raw=$raw}
  }
  function Offset($rva) {
   foreach ($section in $sections) {if ($rva -ge $section.Address -and $rva -lt $section.Address+$section.Size) {return [long]($section.Raw+$rva-$section.Address)}}
   throw ('Unmapped PE address 0x{0:x}.' -f $rva)
  }
  $imports=@()
  if ($importRva -ne 0) {
   $base=Offset $importRva
   for ($i=0;$i -lt [math]::Min(1024,[math]::Ceiling($importSize/20.0));$i++) {
    [void]$stream.Seek($base+20*$i,0);$original=$reader.ReadUInt32();[void]$reader.ReadUInt32();[void]$reader.ReadUInt32();$name=$reader.ReadUInt32();$thunk=$reader.ReadUInt32()
    if ($original -eq 0 -and $name -eq 0 -and $thunk -eq 0) {break}
    [void]$stream.Seek((Offset $name),0);$text=New-Object Text.StringBuilder
    for ($j=0;$j -lt 512;$j++) {$byte=$reader.ReadByte();if ($byte -eq 0) {break};[void]$text.Append([char]$byte)}
    if ($j -eq 512) {throw 'Invalid DLL import name.'};$imports+=$text.ToString().ToLowerInvariant()
   }
  }
  return @($imports | Sort-Object -Unique)
 }finally {$reader.Dispose();$stream.Dispose()}
}
$imports=@(Get-WindowsImports $Executable)
$systemDlls=@('advapi32.dll','avrt.dll','bcrypt.dll','bcryptprimitives.dll','cfgmgr32.dll','combase.dll','comctl32.dll','comdlg32.dll','crypt32.dll','dbghelp.dll','dwmapi.dll','gdi32.dll','imm32.dll','iphlpapi.dll','kernel32.dll','kernelbase.dll','msimg32.dll','msvcrt.dll','ncrypt.dll','normaliz.dll','ntdll.dll','ole32.dll','oleaut32.dll','powrprof.dll','propsys.dll','psapi.dll','rpcrt4.dll','secur32.dll','setupapi.dll','shell32.dll','shlwapi.dll','ucrtbase.dll','user32.dll','userenv.dll','version.dll','winhttp.dll','wininet.dll','winmm.dll','winspool.drv','wintrust.dll','ws2_32.dll','wtsapi32.dll')
$unexpected=@($imports | Where-Object {$_ -notmatch '^(api|ext)-ms-' -and $_ -notin $systemDlls})
if ($unexpected.Count -gt 0) {throw "Non-system DLL dependencies: $($unexpected -join ', '). Use a self-contained/static-CRT build before packaging."}
$commit='unknown';$dirty=$null
if (Get-Command git -ErrorAction SilentlyContinue) {
 try {
  $revision=(& git -C $native rev-parse HEAD 2>$null)
  if ($LASTEXITCODE -eq 0) {
   $commit="$revision";$changes=(& git -C $native status --porcelain 2>$null)
   if ($LASTEXITCODE -eq 0) {$dirty=[bool]$changes}
  }
 }catch {} # Source ZIPs need no Git installation or repository metadata.
}
$name="Lieve-$version-windows-x64"
if ($TestPackage) {$name+='-test'}
[void](New-Item -ItemType Directory -Force -Path $OutputDirectory)
$stage=Join-Path $OutputDirectory ('.stage-'+[guid]::NewGuid().ToString('N'))
[void](New-Item -ItemType Directory -Path $stage)
try {
 Copy-Item -LiteralPath $Executable -Destination (Join-Path $stage 'lieve.exe')
 Copy-Item -LiteralPath (Join-Path $native 'packaging\README-WINDOWS.txt') -Destination (Join-Path $stage 'README.txt')
 $info=[ordered]@{product='Lieve';version=$version;channel='alpha';architecture='x64';builtUtc=[DateTime]::UtcNow.ToString('o');commit="$commit";dirty=$dirty;testPackage=[bool]$TestPackage;installerCompiler=$nsisVersion;exeSha256=(Get-FileHash -LiteralPath (Join-Path $stage 'lieve.exe') -Algorithm SHA256).Hash.ToLowerInvariant();imports=$imports}
 $info | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $stage 'build-info.json') -Encoding UTF8
 $zip=Join-Path $OutputDirectory ($name+'.zip')
 Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip -Force
 $artifacts=@($zip)
 if (-not $PortableOnly) {
  $installer=Join-Path $OutputDirectory ($name+'-setup.exe')
  $stagedInstaller=Join-Path $stage ($name+'-setup.exe')
  $args=@('/V3',"/DAPP_VERSION=$version","/DPAYLOAD_DIR=$stage","/DOUTPUT_FILE=$stagedInstaller")
  if ($TestPackage) {$args+='/DTEST_PACKAGE'}
  $args+=Join-Path $native 'packaging\lieve.nsi'
  & $NsisPath @args
  if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $stagedInstaller)) {throw 'NSIS compilation failed.'}
  Move-Item -LiteralPath $stagedInstaller -Destination $installer -Force
  $artifacts+=$installer
 }
 $sums=foreach ($file in $artifacts) {"$((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant())  $([IO.Path]::GetFileName($file))"}
 $checksum=Join-Path $OutputDirectory ($name+'-SHA256SUMS.txt');$sums | Set-Content -LiteralPath $checksum -Encoding ASCII
 $artifacts+=$checksum
 foreach ($file in $artifacts) {Get-Item -LiteralPath $file | Select-Object Name,Length,FullName}
}finally {if (Test-Path -LiteralPath $stage) {Remove-Item -LiteralPath $stage -Recurse -Force}}
