# Run against a docked, unpinned test instance with isolated APPDATA.
# Tests fullscreen policy/UI; never enables or disables real Windows startup.
param([string]$CaptureDirectory)
$ErrorActionPreference='Stop'
if ($CaptureDirectory -and -not (Test-Path -LiteralPath $CaptureDirectory -PathType Container)) {throw 'CaptureDirectory must already exist.'}
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class LieveDesktopSmoke {
 [StructLayout(LayoutKind.Sequential)] public struct Rect {public int L,T,R,B;}
 [StructLayout(LayoutKind.Sequential)] public struct Point {public int X,Y;}
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern IntPtr FindWindow(string c,string t);
 public static IntPtr Window() {return FindWindow("LieveNativeIsland",null);}
 [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h,out Rect r);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h,IntPtr dc,uint flags);
 [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr c);
 [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
 [DllImport("user32.dll")] public static extern bool GetCursorPos(out Point p);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(Point p);
 [DllImport("user32.dll")] public static extern void mouse_event(uint flags,uint x,uint y,uint data,UIntPtr extra);
}
'@
$window=[LieveDesktopSmoke]::Window()
if ($window -eq [IntPtr]::Zero) {throw 'Start the test instance first.'}
$context=[LieveDesktopSmoke]::SetThreadDpiAwarenessContext([IntPtr](-4))
$cursor=New-Object LieveDesktopSmoke+Point;[void][LieveDesktopSmoke]::GetCursorPos([ref]$cursor)
$foreground=[LieveDesktopSmoke]::GetForegroundWindow()
$scale=[LieveDesktopSmoke]::GetDpiForWindow($window)/96.0
$settings=Join-Path $env:APPDATA 'Lieve\settings.txt'
function Hide-Preference {if (Test-Path -LiteralPath $settings) {return -not ([IO.File]::ReadAllText($settings) -match '(?m)^fullscreen_auto_hide=false\r?$')};return $true}
function Command($id) {[void][LieveDesktopSmoke]::SendMessage($window,0x111,[IntPtr]$id,[IntPtr]::Zero)}
function Pump($milliseconds) {$watch=[Diagnostics.Stopwatch]::StartNew();while ($watch.ElapsedMilliseconds -lt $milliseconds) {[Windows.Forms.Application]::DoEvents();Start-Sleep -Milliseconds 20}}
function Expect-Visible($expected,$description) {
 $watch=[Diagnostics.Stopwatch]::StartNew()
 do {[Windows.Forms.Application]::DoEvents();Start-Sleep -Milliseconds 30;$visible=[LieveDesktopSmoke]::IsWindowVisible($window)}while ($visible -ne $expected -and $watch.ElapsedMilliseconds -lt 2500)
 if ($visible -ne $expected) {throw "$description failed: visible=$visible"}
 "$description`: PASS"
}
function Activate-TestForm {
 $form.BringToFront();$form.Activate();[void][LieveDesktopSmoke]::SetForegroundWindow($form.Handle);Pump 100
 if ([LieveDesktopSmoke]::GetForegroundWindow() -ne $form.Handle) {
  # Windows can deny background SetForegroundWindow calls. Only click after
  # verifying the target is the dedicated test form, never another application.
  $r=New-Object LieveDesktopSmoke+Rect;[void][LieveDesktopSmoke]::GetWindowRect($form.Handle,[ref]$r)
  $p=New-Object LieveDesktopSmoke+Point;$p.X=[int](($r.L+$r.R)/2);$p.Y=[int](($r.T+$r.B)/2)
  [void][LieveDesktopSmoke]::SetCursorPos($p.X,$p.Y)
  if ([LieveDesktopSmoke]::WindowFromPoint($p) -ne $form.Handle) {throw 'Test activation target is covered by another window.'}
  [LieveDesktopSmoke]::mouse_event(2,0,0,0,[UIntPtr]::Zero);[LieveDesktopSmoke]::mouse_event(4,0,0,0,[UIntPtr]::Zero);Pump 150
 }
 if ([LieveDesktopSmoke]::GetForegroundWindow() -ne $form.Handle) {throw 'Test window could not become foreground.'}
}
$initialHide=Hide-Preference;$form=$null;$pinned=$false
try {
 [void][LieveDesktopSmoke]::SetCursorPos(60,800)
 if (-not $initialHide) {Command 40}
 Command 23;$pinned=$true;Pump 900
 # Exercise the actual fullscreen row in System settings twice.
 $point=([int](157*$scale) -shl 16) -bor [int](352*$scale)
 [void][LieveDesktopSmoke]::SendMessage($window,0x201,[IntPtr]::Zero,[IntPtr]$point)
 if (Hide-Preference) {throw 'System settings fullscreen row did not disable/persist.'}
 [void][LieveDesktopSmoke]::SendMessage($window,0x201,[IntPtr]::Zero,[IntPtr]$point)
 if (-not (Hide-Preference)) {throw 'System settings fullscreen row did not restore/persist.'}
 'System settings fullscreen toggle/persistence: PASS'
 if ($CaptureDirectory) {
  $r=New-Object LieveDesktopSmoke+Rect;[void][LieveDesktopSmoke]::GetWindowRect($window,[ref]$r)
  $image=New-Object Drawing.Bitmap ($r.R-$r.L),($r.B-$r.T);$g=[Drawing.Graphics]::FromImage($image)
  try {$dc=$g.GetHdc();try {[void][LieveDesktopSmoke]::PrintWindow($window,$dc,2)}finally{$g.ReleaseHdc($dc)}}finally{$g.Dispose()}
  try {$image.Save((Join-Path $CaptureDirectory 'system-settings.png'))}finally{$image.Dispose()}
 }
 $screen=[Windows.Forms.Screen]::FromHandle($window)
 $form=New-Object Windows.Forms.Form
 $form.Text='Lieve fullscreen test';$form.StartPosition='Manual';$form.Bounds=$screen.WorkingArea
 $form.WindowState='Maximized';$form.Show();Activate-TestForm;Pump 800
 Expect-Visible $true 'Maximized captioned app keeps notch visible'
 $form.WindowState='Normal';$form.FormBorderStyle='None';$form.Bounds=$screen.Bounds
 Activate-TestForm
 Expect-Visible $false 'Borderless foreground fullscreen hides notch'
 [void][LieveDesktopSmoke]::SendMessage($window,0x312,[IntPtr]0x4c49,[IntPtr]::Zero);Pump 200
 Expect-Visible $false 'Keyboard activation message respects fullscreen auto-hide'
 if ([LieveDesktopSmoke]::GetForegroundWindow() -ne $form.Handle) {throw 'Keyboard activation stole fullscreen focus.'}
 Command 40
 Expect-Visible $true 'Disabling fullscreen policy restores notch'
 if ([LieveDesktopSmoke]::GetForegroundWindow() -ne $form.Handle) {throw 'Restoring the notch stole foreground focus.'}
 Command 40
 Expect-Visible $false 'Re-enabling policy hides notch without stealing focus'
 if ([LieveDesktopSmoke]::GetForegroundWindow() -ne $form.Handle) {throw 'Auto-hide changed foreground focus.'}
 $form.Close();$form.Dispose();$form=$null
 Expect-Visible $true 'Exiting fullscreen restores notch'
 Pump 500
 $r=New-Object LieveDesktopSmoke+Rect;[void][LieveDesktopSmoke]::GetWindowRect($window,[ref]$r)
 if ([math]::Abs(($r.R-$r.L)/$scale-400) -gt 1 -or [math]::Abs(($r.B-$r.T)/$scale-220) -gt 1) {throw 'Pinned panel geometry was not restored.'}
 'Pinned view restored after fullscreen: PASS'
}finally {
 if ($form) {$form.Close();$form.Dispose()}
 if ((Hide-Preference) -ne $initialHide) {Command 40}
 if ($pinned) {[void][LieveDesktopSmoke]::SendMessage($window,0x8001,[IntPtr]::Zero,[IntPtr]0x202)}
 [void][LieveDesktopSmoke]::SetCursorPos($cursor.X,$cursor.Y)
 [void][LieveDesktopSmoke]::SetForegroundWindow($foreground)
 [void][LieveDesktopSmoke]::SetThreadDpiAwarenessContext($context)
}
