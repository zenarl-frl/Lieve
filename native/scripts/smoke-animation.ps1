# Requires a running, docked, unpinned Lieve and an idle pointer outside the notch.
# Optional captures are still images of Lieve's own window, not the desktop.
param([string]$CaptureDirectory,[switch]$ExpectAnimated)
$ErrorActionPreference = 'Stop'
if ($CaptureDirectory -and -not (Test-Path -LiteralPath $CaptureDirectory -PathType Container)) {
    throw 'CaptureDirectory must already exist.'
}
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class LieveAnimationSmoke {
 [StructLayout(LayoutKind.Sequential)] public struct Rect { public int L,T,R,B; }
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern IntPtr FindWindow(string c,string t);
 public static IntPtr Island() { return FindWindow("LieveNativeIsland",null); }
 [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h,out Rect r);
 [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr c);
 [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h,IntPtr dc,uint flags);
 [DllImport("user32.dll")] public static extern bool SystemParametersInfo(uint action,uint param,out int value,uint flags);
}
'@
$window = [LieveAnimationSmoke]::Island()
if ($window -eq [IntPtr]::Zero) { throw 'Start Lieve first.' }
$previousDpi = [LieveAnimationSmoke]::SetThreadDpiAwarenessContext([IntPtr](-4))
$scale = [LieveAnimationSmoke]::GetDpiForWindow($window) / 96.0
$motion = 1
[void][LieveAnimationSmoke]::SystemParametersInfo(0x1042,0,[ref]$motion,0)
if ($ExpectAnimated) { $motion=1 }
if ($motion -eq 0) { 'Windows reduced motion: intermediate-frame checks skipped (use -ExpectAnimated for web spring mode).' }
function Get-Bounds {
    $r = New-Object LieveAnimationSmoke+Rect
    if (-not [LieveAnimationSmoke]::GetWindowRect($window,[ref]$r)) { throw 'Cannot read Lieve bounds.' }
    return $r
}
function Toggle-Pin {
    [void][LieveAnimationSmoke]::SendMessage($window,0x8001,[IntPtr]::Zero,[IntPtr]0x202)
}
function Capture-Window($name) {
    if (-not $CaptureDirectory) { return }
    $r = Get-Bounds
    $image = New-Object Drawing.Bitmap ($r.R-$r.L),($r.B-$r.T)
    $graphics = [Drawing.Graphics]::FromImage($image)
    try {
        $dc = $graphics.GetHdc()
        try { if (-not [LieveAnimationSmoke]::PrintWindow($window,$dc,2)) { throw 'Window capture failed.' } }
        finally { $graphics.ReleaseHdc($dc) }
        $image.Save((Join-Path $CaptureDirectory "$name.png"))
    } finally { $graphics.Dispose(); $image.Dispose() }
}
$pinnedByTest = $false
try {
    [void][LieveAnimationSmoke]::SendMessage($window,0x2A3,[IntPtr]::Zero,[IntPtr]::Zero)
    Start-Sleep -Milliseconds 900
    $baseline = Get-Bounds
    if (($baseline.B-$baseline.T) / $scale -gt 50) { throw 'Unpin Lieve and move the pointer outside before testing.' }
    $center = $baseline.L + $baseline.R
    $watch = [Diagnostics.Stopwatch]::StartNew()
    Toggle-Pin
    $pinnedByTest = $true
    $samples = @()
    foreach ($time in @(0,60,120,180,260,400,750)) {
        while ($watch.ElapsedMilliseconds -lt $time) { Start-Sleep -Milliseconds 1 }
        $r = Get-Bounds
        $samples += [pscustomobject]@{Milliseconds=$watch.ElapsedMilliseconds; Width=($r.R-$r.L)/$scale; Height=($r.B-$r.T)/$scale}
        if ([math]::Abs(($r.L+$r.R)-$center) -gt 2) { throw 'Docked animation drifted from the horizontal center.' }
        if ($time -eq 120 -or $time -eq 750) { Capture-Window "open-$time" }
    }
    $samples | Format-Table -AutoSize
    $r = Get-Bounds
    if ([math]::Abs(($r.R-$r.L)/$scale-400) -gt 1 -or [math]::Abs(($r.B-$r.T)/$scale-220) -gt 1) { throw 'Open did not settle at panel size.' }
    if ($motion -ne 0 -and @($samples | Where-Object { $_.Height -gt 50 -and $_.Height -lt 218 }).Count -eq 0) { throw 'No intermediate opening frames observed.' }
    'Open geometry and centered anchor: PASS'

    # Close, then reverse before settling. The next open must finish normally.
    Toggle-Pin
    $pinnedByTest = $false
    Start-Sleep -Milliseconds 90
    $before = Get-Bounds
    Toggle-Pin
    $pinnedByTest = $true
    $after = Get-Bounds
    if ($motion -ne 0 -and [math]::Abs(($after.B-$after.T)-($before.B-$before.T)) -gt 8*$scale) { throw 'Reversal jumped instead of continuing from the current frame.' }
    Start-Sleep -Milliseconds 800
    $r = Get-Bounds
    if ([math]::Abs(($r.B-$r.T)/$scale-220) -gt 1) { throw 'Reversal did not settle open.' }
    'Mid-flight close/open reversal: PASS'

    Toggle-Pin
    $pinnedByTest = $false
    Start-Sleep -Milliseconds 850
    $r = Get-Bounds
    if (($r.B-$r.T)/$scale -gt 50) { throw 'Close did not settle.' }
    'Close settles and releases test pin: PASS'
} finally {
    if ($pinnedByTest) { Toggle-Pin }
    [void][LieveAnimationSmoke]::SetThreadDpiAwarenessContext($previousDpi)
}
