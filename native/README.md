# Lieve native — Windows preview

## Latest update: native To-do

To-do is no longer a placeholder: Add task opens a native input dialog (up to 200 characters), click a row to toggle completion, `x` removes it, and scroll browses additional tasks. Up to 100 tasks are saved in `%APPDATA%\Lieve\tasks.txt`. Reminder scheduling and editing are not implemented. This update supersedes older To-do placeholder descriptions below. See the repository root README for the current feature matrix.

Rust + Win32/GDI, without a browser runtime or GPU rendering loop. The React prototype remains in `../prototype` as the design reference.

## Build

Requires Rust (MSVC target) and Visual Studio C++ Build Tools with a Windows SDK.

```powershell
cargo build --release
.\target\release\lieve.exe
```

## Controls

- Hover to expand; move away to collapse.
- Expand/collapse dimensions ease over 260 ms. Leaving waits 180 ms and checks that the pointer is actually outside before closing. Returning cancels the close timer. Animation frame timers run only during transitions; Windows' client-area animation preference disables motion on launch when requested. Contents scale during resize; this is not yet the browser's independent element choreography.
- Idle notch shows only the eyes, centred horizontally and vertically, without a clock. Expanded header keeps the eyes centred independently of song metadata; the clock appears at the right. Playing music replaces idle eyes with compact lyrics/title.
- Click the docked header or `P` to pin/unpin.
- Right-click for eye colour presets, floating/minimal modes, pin, and Exit.
- The system tray icon provides the same menu. Left-click it to pin/unpin the panel.
- Custom eye colour opens the Windows colour picker.
- The header `...` opens/closes the native Appearance panel. Preview eyes, seven colour presets, custom colour, a draggable glow slider, mascot visibility, compact lyrics, minimal mode and floating mode are persisted in settings. Disabling mascot shows a centred clock while idle. The tray's Appearance entry opens the same panel.
- In floating mode, left-drag the header to move. Use the menu to pin.
- Eye colour and floating/minimal modes are stored in `%APPDATA%\Lieve\settings.txt`. The previous `eye-color.txt` is read for migration if no settings file exists.
- A second launch exits if the existing Lieve window is present.

## Design preview controls

The expanded shell follows the React prototype's Music layout. A Windows SMTC worker reads the current player's title, artist, timeline and playback status once per second. Play/pause, previous/next and seek are sent to that player; supported controls depend on the player. Position is extrapolated from the timeline timestamp. Artwork is still a generated placeholder, not the actual album image.

When no Windows media session is available, Music shows explicitly labelled demo playback. Real songs never use sample lyrics. LRCLIB supplies matching synced lyrics where available. The active line appears in the compact notch while playing and highlights in the Lyrics tab; the title is the fallback when lyrics are unavailable. Long compact lines currently truncate rather than scroll.

Focus provides a 25-minute start/pause/reset timer that continues with the panel closed and sends a Windows tray notification when complete (subject to OS notification preferences). The timer does not persist after exit. Water records +200/+400 ml with a 1.5 L goal and saves today's total to `%APPDATA%\Lieve\water.txt`. The local date is checked on launch and while running, resetting the total on a new day. Historical charts and scheduled reminders are not implemented. To-do remains a placeholder; Shelf now has reference storage and native file operations as described below.

This is not yet pixel-identical to the browser design: glow, curve antialiasing and independent element transitions still need refinement. Resize easing is implemented, not a spring physics simulation. Rendering uses GDI for now.

Glow currently uses low-cost layered GDI strokes, not a Gaussian blur. The colour picker remains a standard Windows dialog; the surrounding settings panel is drawn natively inside the notch.

## File shelf

Drop files/folders from Explorer onto the notch, or use Shelf → Add file. Up to 100 absolute path references are saved in `%APPDATA%\Lieve\shelf.txt`. Contents are not copied. Click an item to open its Windows handler; open only files you trust. Click `x` or use the tray's Clear shelf references command to remove references without deleting files. Scroll to browse more than four items. Missing files display a warning on opening.

Drag-out, thumbnails, keyboard navigation and file-size metadata are not implemented. Explorer drag/drop and the native picker still require interactive acceptance testing. Run without elevation to avoid Windows drag/drop privilege restrictions.

## Current scope

This is an alpha, not feature parity with the React prototype. Tasks, complete file-shelf parity, water/task reminders, HUD overlays, connection/battery alerts, startup integration, fullscreen auto-hide, packaging and animation polish are not completed. The tray icon is restored after Explorer restarts. Rendering occurs on window events and periodic state updates; no continuous frame loop. The tray currently uses the Windows default application icon. Mixed-DPI displays and fullscreen behaviour still require validation.

## Network and privacy

On track changes, title, artist and duration are sent over HTTPS to `lrclib.net` to retrieve lyrics. There are no accounts or analytics. Missing lyrics and network errors fall back to the song title. Network and media operations run off the UI thread. A slow network operation can delay later lyric requests; an explicit request timeout and retry UI are not implemented yet.

## Memory target

Target: around 30 MiB idle working set. This is a target, not a guaranteed cap. Measure the release build on the target machine, recording both working set and private committed memory. Do not trim working sets to obtain artificially low measurements.

```powershell
Get-Process lieve | Select-Object Id, CPU,
  @{n='WorkingSetMiB';e={[math]::Round($_.WorkingSet64/1MB,2)}},
  @{n='PrivateMiB';e={[math]::Round($_.PrivateMemorySize64/1MB,2)}}
```
