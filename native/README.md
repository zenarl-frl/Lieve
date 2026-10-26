# Lieve native — Windows preview

## Control Center, external OSD and glass transparency

The control bubble right of the tab rail (or the menu's **Control Center** entry) opens a Wi-Fi/Bluetooth/Sound/Brightness panel. Three worker threads keep it current: one Core Audio loop (default-endpoint volume, mute), one WMI brightness loop (`root/WMI` monitors; `N/A` when unsupported), and one on-demand network loop refreshed when the panel opens and at about every 8 seconds that reads the active SSID, adapter status and connected Bluetooth device names (deduplicated by address). Wi-Fi and Bluetooth cards deep-link to `ms-settings:`; the Bluetooth card shows its own back/connected-list view.

Draggable Sound/Brightness sliders push values straight to hardware and reuse the same `src/osd.rs` spring HUD as external changes: a 2000 ms auto-hide timer (with an expiry re-check that catches timers firing a few ms early) wraps a spring that retains velocity when a new value arrives mid-flight. OSD is suppressed while a slider is being dragged. Read-back updates from Windows arrive as `Update::{Audio,Brightness,Devices,Error}` events; brightness feedback keeps a panel-initiated drag from being echoed. Keyboard media keys change the default audio endpoint, so the read-back loop shows the same OSD (about 80 ms worker poll + 100 ms UI poll); brightness keys that bypass WMI cannot be intercepted without a global hook, while monitor-driver changes reported through WMI show it.

**Appearance → Glass** adds a frosted backdrop and a 0–100% transparency slider (saved as `transparency`/`frosted` in `settings.txt`, defaults 25%/frosted). The shell enables `WS_EX_LAYERED` and composites premultiplied per-pixel alpha through `UpdateLayeredWindow`; frosted mode requests accent state 3 via `SetWindowCompositionAttribute`, falling back to plain transparency when unavailable. Layered presentation only runs while the shell is expanded/animating, so the zero-alpha idle notch is skipped when nothing is transparent. Local checker pixels measured clear=48.76 vs frosted=3.04 standard deviation, confirming visible blur.

Default tests (44 passed, 5 live integrations ignored), release build, slider/hardware round trips, frosted-vs-clear capture comparison and the OSD auto-hide check all passed. See `VALIDATION.md`.

## Latest update: clean anti-aliasing, aligned labels, restart and Devil grin

Rendering surfaces now use top-down 32-bit DIB sections. Icon/mascot supersampling is resolved by averaging pixel coverage after GdiFlush, then copied one-to-one; GDI HALFTONE no longer reduces those sprites. Artwork uses cached area filtering for downscaling, bilinear upscaling and anti-aliased rounded masks. Card borders use cached coverage masks. Thumbnail source resolution still bounds real photo detail.

Focus/Water labels use measured centered rectangles. Custom colour centers its combined swatch/text group; Glow values align with the slider's right edge; list Add labels and counts share consistent vertical alignment and trailing padding. Appearance's right column fits five settings using separate label/caption typography.

The shared tray/context menu provides **Restart Lieve**. It exits the old window/event loop before launching the current executable; settings/data reload normally. A private restart argument transfers paused Focus seconds or the running wall-clock deadline, accounting for startup time. This does not persist Focus through ordinary Exit.

**Devil grin** is an opt-in, saved Appearance/tray toggle. It animates a procedural sharp-toothed smile with squinting/lifted eyes: 320 ms opening, a short hold and 550 ms closing, about 1.9 seconds total. It repeats every 12 seconds while an eligible mascot is visible; enabling it previews immediately. Idle geometry is 144 × 48 to fit the mouth; default-off keeps the 144 × 36 idle size. Reduced motion shows a static grin. Teeth and outline are a fixed near-white expression and do not follow the eye colour. Teeth and face motion use the same true coverage-resolved sprite, including subpixel head movement.

Default tests, release build, pixel-based button-centering checks, grin on/off captures, native menu inspection and a real process-replacement restart with running Focus all passed. See `VALIDATION.md`.

## Balanced Music player with in-player Lyrics

The expanded notch is **400 × 220 logical pixels**. Its five-tab navigation now has centered labels, even cell spacing and a separate Settings gear aligned to the outer margin. **Lyrics lives only inside Music**, at the right of its playback row. Toggling it swaps the album card for the active lyric and neighboring lines while preserving progress and playback controls. Closing Settings restores Music's preceding artwork/lyrics state. Pin/power remain in the tray/context menu.

Music uses a framed album card with 72 px artwork, measured ellipsis for metadata, and restrained title/artist/caption hierarchy. A duplicate album/title line is suppressed. The progress rail spans the same left/right content edges, its times align to opposite edges, and the 36 px white play disc is centered at x=200; previous/next are equidistant at x=148/252. Status and Lyrics occupy balanced outer lanes. `src/layout.rs` shares control rectangles with hit testing. `src/icons.rs` draws cached 4x vector glyphs for playback, Lyrics and Settings; hover redraws only when the hovered target changes.

Windows SMTC thumbnails are now opened and decoded in a dedicated artwork worker, using an agile COM reference between threads. Windows imaging produces aspect-preserving BGRA thumbnails up to 256 px; GDI center-crops them into rounded covers in Music and compact playback. Cover state is keyed by player/title/artist/album/duration, clears on track changes, rejects late old-track results and retries missing images. A transient refresh failure retains the matching track's current image. Real sessions display Loading/No artwork when appropriate; only demo playback uses the generated illustration.

Unit tests, release build, Music-only Lyrics/Settings-return navigation and shell/To-do/compact-control checks passed. Seven panel captures were inspected at 150% DPI, including real album artwork. A pixel check confirmed the play disc is centered in both artwork and lyrics modes. See `VALIDATION.md` for details.

## Smoother frame pacing

Active animation now uses a high-resolution, one-shot waitable timer integrated into the UI message loop. Cadence follows the current monitor's refresh rate (up to 240 Hz, 60 Hz for unknown rates). Retargeting leaves an armed deadline intact. Missed deadlines are skipped without catch-up bursts; each tick paints after releasing the application-state borrow. The clock is cancelled when motion settles, with a WM_TIMER fallback if the high-resolution API is unavailable. Windows' global timer resolution is not changed.

Output/stage buffers grow in small capacity blocks and are reused instead of allocating DCs and bitmaps every frame. Unchanged window geometry and region shapes avoid redundant updates. Icon/mascot sprites use 4x coverage resolve and cached rounded cards use fractional edge coverage; typography stays native scale. GDI resource-lifetime tests cover buffer reuse, replacement and cleanup.

Local geometry-event measurements improved from **27.01 ms median / 36.45 ms p95** to **8.30 ms / 9.02 ms**, using `scripts/measure-motion.ps1`. These are UI geometry-event intervals rather than presented compositor FPS. Tests, release build and shell/To-do regression checks passed; see `VALIDATION.md` for sample counts, limits and the interrupted sleepy check.

## Cursor-aware mascot

Idle eyes follow the pointer using the web mascot's normalized translation and 1.4-degree tilt, relative to Lieve's current monitor. Coordinates outside that monitor are clamped, including negative-origin monitor layouts. Follow/tilt/lid tracking now uses a critically damped filter retaining both position and velocity across new targets, replacing the repeatedly restarted cubic-bezier tweens. It settles to an exact pose with a bounded lifetime.

After 26 seconds without Windows input, eyes return to the center, tilt 3.5 degrees and narrow to 26% height. `GetLastInputInfo` supplies activity age using wrapping tick arithmetic. A new input wakes them, with up to roughly 500 ms detection delay while sleepy. Blinking still narrows to 6% and recovers to the current eyelid pose. Opening the panel centers the eyes; compact/minimal/hidden mascot states disable idle tracking. Reduced motion disables tracking; an enabled Devil grin remains static.

Dormant cursor sampling runs every 50 ms only while idle eyes are visible, slowing to 500 ms when sleepy. During active eye animation, the cursor is sampled each animation frame for smoother live tracking. Unchanged targets do not restart the filter or frame deadline. No global input hook or extra worker thread is used.

Unit tests, release build, animation/To-do smoke checks and isolated visual checks of left/right tracking, sleep and wake passed. See `VALIDATION.md` for the measured silhouette changes and remaining checks.

## Web-matched motion

Shell width, height, position and corner radius now use the prototype's analytic spring (`stiffness=360`, `damping=32`, `mass=0.85`), retaining velocity on mid-flight retargets. Settling is threshold-based with a time cap rather than the old fixed 260 ms quintic resize. The docked center stays anchored.

Idle, compact and panel content are composited as centered, native-scale GDI layers with wait-mode opacity handoffs (180/140/160 ms), matching the web's `AnimatePresence` structure. Text and eyes no longer stretch with the shell. The idle content has a small vertical fade offset, corners animate, and the tab highlight uses its own spring (`480/38/1`). Eyes blink with a 100 ms close, short hold and 500 ms recovery; varied 3.5–7.8 second scheduling only requests frame updates during a blink.

Right-click the notch or tray icon to choose **Motion: follow Windows** (default), **Motion: web spring**, or **Motion: reduced**. The per-app `motion=system|full|reduced` setting persists; system mode also responds to Windows preference changes. Web spring can be enabled when Windows disables client-area animation. Reduced mode snaps shell, content and pill and suppresses blinking.

`scripts/smoke-animation.ps1 -ExpectAnimated` checks a running, docked, unpinned web-spring instance for intermediate frames, a centered anchor, reversal and final settling. Keep the pointer outside the notch. Optional `-CaptureDirectory <existing-folder>` captures Lieve-window stills. Tests and release build passed; see `VALIDATION.md` for the measured samples and remaining visual gaps.

## Windows battery status

The context/tray menu shows battery percentage and an AC indicator. Machines without a battery show `No battery`; unavailable readings show a neutral fallback. `GetSystemPowerStatus` is read locally on startup and about every 30 seconds, without another power worker thread or a continuous rendering loop.

A tray notification is requested once when a known battery is at or below 20% on battery power. AC power or recovery to at least 25% rearms the warning; unknown readings and 20/21% jitter do not repeat it. The warning latch resets on app restart. Windows notification preferences govern visibility. Unit tests and a Windows GNU release build passed. A later animation capture verified the header showing a real `79% · AC` label; real low-battery notification delivery still needs device acceptance testing.

## Native To-do editing

To-do provides a native input dialog (up to 200 characters), row clicks to toggle completion, `edit` to rename, `x` to remove, and scrolling to browse additional tasks. Editing prefills and selects the current text; Save/Enter accepts a non-empty value and preserves completion, while Cancel/Escape discards changes without rewriting the task file. Failed saves restore the previous in-memory state and report an error. Up to 100 tasks are saved in `%APPDATA%\Lieve\tasks.txt`. Reminder scheduling is not implemented. The release build and expanded UI smoke test passed using the Windows GNU toolchain and isolated APPDATA; see `VALIDATION.md` for remaining manual checks and MSVC status. See the repository root README for the current feature matrix.

Rust + Win32/GDI, without a browser runtime or GPU rendering loop. The React prototype remains in `../prototype` as the design reference.

## Build

Requires Rust (MSVC target) and Visual Studio C++ Build Tools with a Windows SDK.

```powershell
cargo build --release
.\target\release\lieve.exe
```

## Controls

- Hover to expand; move away to collapse.
- Expand/collapse uses the web spring and unscaled content fading described above. Leaving waits 180 ms and checks that the pointer is actually outside before closing. Returning cancels the close timer and preserves spring momentum. The display-rate frame clock runs only while shell/content/pill/eye motion is active; rendering under load may miss deadlines.
- Idle notch shows only the eyes, centred horizontally and vertically, without a clock. Expanded panels omit the old header. Playing music replaces idle eyes with artwork and compact lyrics/title.
- Use the tray's left click or the context menu's Pin open entry to pin/unpin.
- Right-click for eye colour presets, floating/minimal modes, Devil grin, Control Center, Restart Lieve and Exit. The centre of the tab rail has a control bubble that opens the Control Center panel (Wi-Fi, Bluetooth list, Sound and Brightness sliders; mute toggles and ms-settings deep-links).
- The same Control Center lives in the tray/context menu (command 16) — selecting it pins the panel open.
- Appearance's right-hand **Glass** caption opens the glass/effects view: frosted toggle plus a 0–100% transparency slider that previews live and persists in settings.
- Volume/brightness changes made in Windows (media keys, monitors) or Lieve's own sliders show the spring OSD below the notch, which hides itself after about 2 seconds; dragging a slider suppresses the OSD.

## Translucent glass backdrop

The shell enables `WS_EX_LAYERED` and composites premultiplied per-pixel alpha through `UpdateLayeredWindow` whenever it is expanded, showing the OSD, compact-playing or still in motion (spring transition, content or tab-pill fade). Frosted mode requests accent state 3 via `SetWindowCompositionAttribute`; if unsupported on the Windows/display combination the view reports it and the same fallback keeps a clearly transparent backdrop. The layered mode is deliberately kept until the shell settles back at its idle frame: switching it mid-flight visibly flashed the backdrop, which was the seam behind a misbehaving close/open animation. Transparency at 0% keeps the binary-region, painted-range behaviour and disables blur.
- The system tray icon provides the same menu. Left-click it to pin/unpin the panel.
- Custom eye colour opens the Windows colour picker.
- Music's lower-right quote-bubble button toggles its in-player Lyrics card. The navigation gear opens/closes Appearance and returns to the previous view. Preview eyes, seven colour presets, custom colour, a draggable glow slider, mascot visibility, compact lyrics, minimal mode, floating mode and Devil grin are persisted in settings. Disabling mascot shows a centred clock while idle. The tray's Appearance entry opens the same panel.
- In floating mode, drag the empty top-bar background/edge to move. Use the menu to pin.
- Eye colour and floating/minimal modes are stored in `%APPDATA%\Lieve\settings.txt`. The previous `eye-color.txt` is read for migration if no settings file exists.
- A second launch exits if the existing Lieve window is present.

## Design preview controls

The expanded shell uses a minimal Music layout. A Windows SMTC worker reads the current player's title, artist, timeline and playback status once per second. Play/pause, previous/next and seek are sent to that player; supported controls depend on the player. Position is extrapolated from the timeline timestamp. Thumbnail opening/decoding is isolated from that worker and the UI; decoded pixels are shared in memory rather than saved to user files. Streams above 8 MiB and excessive source dimensions are rejected before full decoding.

When no Windows media session is available, Music shows explicitly labelled demo playback. Real songs never use sample lyrics. LRCLIB supplies matching synced lyrics where available. The active line appears in the compact notch while playing and highlights in the Lyrics view opened by the quote-bubble button; the title is the fallback when lyrics are unavailable. Long compact lines currently truncate rather than scroll.

Focus provides a 25-minute start/pause/reset timer that continues with the panel closed and sends a Windows tray notification when complete (subject to OS notification preferences). Restart Lieve transfers its paused/running state, while ordinary Exit does not persist it. Water records +200/+400 ml with a 1.5 L goal and saves today's total to `%APPDATA%\Lieve\water.txt`. The local date is checked on launch and while running, resetting the total on a new day. Historical charts and scheduled reminders are not implemented. To-do supports local task management as described above; Shelf has reference storage and native file operations as described below.

This is not yet pixel-identical to the browser design: the outer window still uses a binary Windows region, and Gaussian glow, alert/curious expressions and view-specific layout need refinement. Cards/artwork/icons/mascot have coverage-resolved edges. Shell/tab springs, content fades, blinking, cursor-follow, sleepy expressions and optional grin are implemented natively.

Glow currently uses low-cost layered GDI strokes, not a Gaussian blur. The colour picker remains a standard Windows dialog; the surrounding settings panel is drawn natively inside the notch.

## File shelf

Drop files/folders from Explorer onto the notch, or use Shelf → Add file. Up to 100 absolute path references are saved in `%APPDATA%\Lieve\shelf.txt`. Contents are not copied. Click an item to open its Windows handler; open only files you trust. Click `x` or use the tray's Clear shelf references command to remove references without deleting files. Scroll to browse more than four items. Missing files display a warning on opening.

Drag-out, thumbnails, keyboard navigation and file-size metadata are not implemented. Explorer drag/drop and the native picker still require interactive acceptance testing. Run without elevation to avoid Windows drag/drop privilege restrictions.

## Current scope

This is an alpha, not feature parity with the React prototype. Complete file-shelf parity, water/task reminders, connection alerts, startup integration, fullscreen auto-hide, packaging and animation polish are not completed. Battery status and low-battery alerts are implemented with runtime acceptance checks pending as described above. Control Center covers Wi-Fi/Bluetooth/system volume/brightness basics; its hardware-control acceptance is described in `VALIDATION.md`. The tray icon is restored after Explorer restarts. Rendering occurs on window events and periodic state updates; no continuous frame loop. The tray currently uses the Windows default application icon. Mixed-DPI displays and fullscreen behaviour still require validation.

## Network and privacy

On track changes, title, artist and duration are sent over HTTPS to `lrclib.net` to retrieve lyrics. There are no accounts or analytics. Missing lyrics and network errors fall back to the song title. Network and media operations run off the UI thread. A slow network operation can delay later lyric requests; an explicit request timeout and retry UI are not implemented yet.

## Memory target

Target: around 30 MiB idle working set. This is a target, not a guaranteed cap. Measure the release build on the target machine, recording both working set and private committed memory. Do not trim working sets to obtain artificially low measurements.

```powershell
Get-Process lieve | Select-Object Id, CPU,
  @{n='WorkingSetMiB';e={[math]::Round($_.WorkingSet64/1MB,2)}},
  @{n='PrivateMiB';e={[math]::Round($_.PrivateMemorySize64/1MB,2)}}
```
