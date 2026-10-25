# Run against an already-running Lieve alpha. Adds/removes one uniquely named task.
# Does not restart Lieve or send playback commands. Keep the pointer away during the test.
$ErrorActionPreference = 'Stop'
Add-Type @'
using System;
using System.Runtime.InteropServices;
public class LieveTodoSmokeTest {
 [StructLayout(LayoutKind.Sequential)] public struct Rect { public int L,T,R,B; }
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c,string t);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowEx(IntPtr p,IntPtr a,string c,string t);
 [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll",CharSet=CharSet.Unicode,EntryPoint="SendMessageW")] public static extern IntPtr SetText(IntPtr h,uint m,IntPtr w,string text);
 [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h,out Rect r);
 [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c);
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr c);
}
'@
[void][LieveTodoSmokeTest]::SetProcessDpiAwarenessContext([IntPtr](-4))
$previousDpi = [LieveTodoSmokeTest]::SetThreadDpiAwarenessContext([IntPtr](-4))
$window = [LieveTodoSmokeTest]::FindWindow('LieveNativeIsland', $null)
if ($window -eq [IntPtr]::Zero) { throw 'Start Lieve first.' }
$dialog = [IntPtr]::Zero
$task = 'Lieve smoke test ' + [guid]::NewGuid().ToString('N')
$store = Join-Path $env:APPDATA 'Lieve\tasks.txt'
$added = $false
$pinnedByTest = $false

function Get-Bounds {
    $rect = New-Object LieveTodoSmokeTest+Rect
    [void][LieveTodoSmokeTest]::GetWindowRect($window, [ref]$rect)
    return $rect
}
function Click-Point($x, $y, [switch]$Async) {
    $point = [IntPtr](([int]($y * $script:scale) -shl 16) -bor [int]($x * $script:scale))
    if ($Async) { [void][LieveTodoSmokeTest]::PostMessage($window, 0x201, [IntPtr]::Zero, $point) }
    else { [void][LieveTodoSmokeTest]::SendMessage($window, 0x201, [IntPtr]::Zero, $point) }
}
function Open-TaskDialog {
    Click-Point 220 267 -Async
    Start-Sleep -Milliseconds 400
    $found = [LieveTodoSmokeTest]::FindWindow('LieveTaskInput', $null)
    if ($found -eq [IntPtr]::Zero) { throw 'Task dialog did not open.' }
    return $found
}

try {
    # Settle auto-close first. If still expanded, do not disturb the user's existing pin.
    [void][LieveTodoSmokeTest]::SendMessage($window, 0x2A3, [IntPtr]::Zero, [IntPtr]::Zero)
    Start-Sleep -Milliseconds 600
    $rect = Get-Bounds
    if (($rect.B - $rect.T) -lt 150) {
        [void][LieveTodoSmokeTest]::SendMessage($window, 0x8001, [IntPtr]::Zero, [IntPtr]0x202)
        $pinnedByTest = $true
        Start-Sleep -Milliseconds 350
    }
    $rect = Get-Bounds
    # Coordinates must use physical window geometry, not a DPI-virtualized caller's units.
    $script:scale = ($rect.R - $rect.L) / 470.0
    Click-Point 420 65
    $dialog = Open-TaskDialog
    $edit = [LieveTodoSmokeTest]::FindWindowEx($dialog, [IntPtr]::Zero, 'EDIT', $null)
    if ($edit -eq [IntPtr]::Zero) { throw 'Task input control not found.' }
    [void][LieveTodoSmokeTest]::SetText($edit, 0xC, [IntPtr]::Zero, $task)
    $added = $true
    [void][LieveTodoSmokeTest]::SendMessage($dialog, 0x111, [IntPtr]1, [IntPtr]::Zero)
    # A synchronously sent modal command may destroy the dialog while its GetMessage
    # still waits; queue a message so the modal loop can return to the owner.
    [void][LieveTodoSmokeTest]::PostMessage($window, 0, [IntPtr]::Zero, [IntPtr]::Zero)
    $dialog = [IntPtr]::Zero
    Start-Sleep -Milliseconds 200
    $text = [IO.File]::ReadAllText($store)
    if (-not $text.Contains("0`t$task")) { throw 'New task not persisted.' }
    $added = $true
    'Add task and persist: PASS'
    Click-Point 100 112
    if (-not [IO.File]::ReadAllText($store).Contains("1`t$task")) { throw 'Completion not persisted.' }
    'Toggle completion: PASS'
    Click-Point 439 112
    if ([IO.File]::ReadAllText($store).Contains($task)) { throw 'Task not removed.' }
    $added = $false
    'Delete task: PASS'
    $beforeCancel = [IO.File]::ReadAllText($store)
    $dialog = Open-TaskDialog
    [void][LieveTodoSmokeTest]::SendMessage($dialog, 0x10, [IntPtr]::Zero, [IntPtr]::Zero)
    [void][LieveTodoSmokeTest]::PostMessage($window, 0, [IntPtr]::Zero, [IntPtr]::Zero)
    $dialog = [IntPtr]::Zero
    Start-Sleep -Milliseconds 200
    if ([IO.File]::ReadAllText($store) -ne $beforeCancel) { throw 'Cancel changed tasks.' }
    'Cancel preserves tasks: PASS'
} finally {
    if ($dialog -ne [IntPtr]::Zero) { [void][LieveTodoSmokeTest]::SendMessage($dialog, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) }
    if ($added) { Write-Warning "Test task may remain; remove only this task manually: $task" }
    if ($pinnedByTest) { [void][LieveTodoSmokeTest]::SendMessage($window, 0x8001, [IntPtr]::Zero, [IntPtr]0x202) }
    [void][LieveTodoSmokeTest]::SetThreadDpiAwarenessContext($previousDpi)
}
