# Lieve native — Windows preview

## Bluetooth battery panel and charger OSD

Bluetooth now has its own rail icon and panel (tab 9), separate from the Control Center. It lists connected classic and LE devices with a per-device battery percentage when Windows has a cached report, a refresh action and a `ms-settings:bluetooth` deep-link. Battery data comes from the Microsoft stack's device property `{104EA319-6EE2-4701-BD47-8DDBF425BBE5}` PID 2 (`SetupDi*` + `CM_Get_DevNode_PropertyW`); values outside 0–100 are ignored, and devices without a report show **Battery not reported** rather than a clamped or invented number. Addresses fall back to the instance-id form when the Windows Runtime address is zero.

`src/power.rs` now tracks charging and emits plug/unplug events from `WM_POWERBROADCAST` subscriptions (`GUID_ACDC_POWER_SOURCE`, `GUID_BATTERY_PERCENTAGE_REMAINING`), with periodic sampling as a fallback. The OSD adds `PowerConnected`, `PowerDisconnected` and `LowBattery` kinds: a 3-second charging HUD animates fill plus a one-pass sweep, and unplugging shows a compact percentage before auto-hide. The **Power & battery** page (tab 12) shows percentage, charging state, refresh, and preview plug/unplug buttons that never change the real power source. The preview path and both OSD directions were captured and checked.

Default GNU release suite: **64 passed, 0 failed, 10 ignored**. Optional live checks passed separately: power notification registration on a hidden window, live connected-device battery read, and atomic layered-frame publishing. Geometry probe measured **12.08 ms median / 20.05 ms p95**, with no interval above 25 ms in that run; these remain geometry-event intervals, not compositor FPS. See `VALIDATION.md`.

## Opening-jitter correction


Docked shell position, layer placement, pointer routing and glass opacity masks now use the same physical monitor anchor. Previously, rounding window X independently and dividing a negative `(clientWidth-layerWidth)/2` could alternately shift content by one pixel as the opening width changed parity. Layers are now pixel-locked to their final screen origin while the shell clips them. Floating mode keeps its local-center behavior; the idle fade offset is rounded in physical pixels.

Layered windows use their alpha mask directly, removing repeated rectangular `SetWindowRgn` updates during resize. Vector icons have a bounded 48-entry raster cache keyed by physical size, glyph, colours and disc. Release builds use speed optimization (`opt-level=3`) for the per-frame compositor. The spring parameters are retained.

All 60 release tests passed (7 optional integrations ignored), including parity/negative-origin anchoring. The isolated 25%-glass geometry probe measured 8.88 ms median / 16.97 ms p95 before and 8.40 / 17.24 ms after; occasional missed intervals remain, and this is not presented compositor FPS. Opening/closing/reversal and post-change UI checks passed. See `VALIDATION.md`.

## Compact UI polish

Focus and Water now share framed hero cards, a reusable 32 px metric font, clear status/progress, and primary/secondary/quiet action styles. Existing action positions remain stable and shared layout rectangles drive both rendering and hover/click routing. Buttons, list actions and seek have hover affordances; interactive areas use a hand cursor, while inactive list slots/capped Add actions retain the default cursor.

To-do uses vector pencil/close/check glyphs, measured ellipsis and muted strike-through for completed rows. Shelf uses separate filename/parent-folder typography without filesystem access during paint. Lists share bordered rows, informative empty states and visible-range counts. Appearance aligns to the panel's 16 px outer margins; lightweight captions and stronger subpage headings clarify hierarchy. Antialiased switches/checkboxes tone their track colours to retain white-thumb/check contrast with light eye colours. The metric font is released on shutdown, and glass masks retain dark primary-button labels on both Focus and Water.

GNU tests: 59 passed, 7 optional integrations ignored. Inspected populated/empty views at 150% DPI and verified Focus/Water actions and white-accent switch contrast through an isolated instance. The active foreground game captured cursor position, so physical hover acceptance and a fresh modal To-do dialog smoke were deferred; action/hit-rectangle consistency is covered by a unit regression. See `VALIDATION.md`.

## Windows installer and portable ZIP

`scripts/package-windows.ps1` builds an x64 GUI release, audits direct DLL imports and produces a portable ZIP, a per-user NSIS installer and SHA-256 checksums under ignored `dist/`. Prebuilt binaries are supported via `-SkipBuild`; `-PortableOnly` needs no installer compiler. Both formats contain the same executable, usage guide and build metadata.

Setup supports Start menu/optional desktop shortcuts, in-place updates and graceful closure of only its own running executable. Uninstall retains AppData/unknown files and preserves startup registration for another copy. An isolated test-package identity exercised install, update, standalone launch and uninstall successfully. Packages are local unsigned alpha artifacts; see [PACKAGING.md](PACKAGING.md) for requirements, commands and remaining acceptance checks.

## Spectrum preferences and keyboard controls

**Appearance → System → Spectrum** (or tray **Spectrum settings...**) configures the five live bars. Saved keys are `spectrum_enabled` (default true), `spectrum_sensitivity` (50–200, default 100) and `spectrum_color=accent|white` (default accent). Sensitivity applies audio gain in the normalized decibel domain and preserves exact silence; it does not synthesize movement. White affects only the bars. Disabling the visualizer releases capture, restores the PLAYING badge in Music and expands the compact text lane. Both pointer and keyboard edits persist and reload.

`src/keyboard.rs` registers **Ctrl+Alt+Space** using `RegisterHotKey` with no-repeat, without a low-level hook. It opens/pins/focuses the panel and toggles closed when pressed again while Lieve owns focus. Registration failure is displayed in System settings; the binding is released on destruction. Fullscreen auto-hide takes precedence. Opening records the prior foreground window and process; closing restores it only if that window is still valid and Lieve still owns foreground. Losing focus ends the keyboard session.

- **Tab / Shift+Tab**, **Left / Right**: move the visible focus cue across primary navigation and the Appearance/System/Spectrum settings controls exposed to keyboard input.
- **Enter / Space**: activate the focused action (auto-repeat does not toggle it repeatedly).
- **Ctrl+1–5**: jump directly to Music, Focus, Water, Shelf or To-do while Lieve is focused.
- **Escape**: close/unpin and restore the shortcut caller. A stationary pointer cannot reopen the panel through geometry-generated mouse messages; actual pointer movement restores hover behavior.
- Spectrum colour: Left/Right choose Accent/White. Sensitivity: arrows adjust 5%, Home/End choose 50/200%.

Default tests: 58 passed, 7 optional integrations ignored. Real keyboard input verified shortcut focus/close, navigation, spectrum edits, previous-foreground restoration, stationary-pointer close, shortcut cleanup and settings reload in a new process. Existing animation/fullscreen checks also passed. This is basic navigation, not complete keyboard or accessibility parity for every Music/list/slider action.

## Transition polish, fullscreen policy and Windows startup

Shell retargeting now cancels a pending destination even when a reversal happens before the first changed geometry frame. Returning to content that is still visible raises its opacity rather than fading it out again. Repeated volume/brightness changes retain the OSD's original fade-in timestamp while extending expiry; reduced motion displays it immediately. The runtime animation smoke test accounts for legitimate spring travel between cross-process geometry samples instead of treating every display frame as a jump.

**Appearance → System** (also **System settings...** in the tray) opens two toggles:

- **Hide in fullscreen** defaults on and persists as `fullscreen_auto_hide` in settings. Every 500 ms, the app checks whether a visible foreground borderless window covers the notch's monitor. Captioned maximized windows, desktop/taskbar windows and other-monitor windows do not hide it. While hidden, spectrum capture and animation timers stop, new OSDs are suppressed, and Focus/media state continue updating. Leaving fullscreen restores the pinned panel or normal compact/idle state without activation or stale OSD frames. Menus and active slider drags defer the visibility change.
- **Start with Windows** reads/writes the `Lieve` value in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`. It stores a quoted absolute path to the current executable and does not require elevation. Registration only changes on an explicit toggle; no duplicate settings-file flag is used. Moving the binary requires re-enabling it. Windows Startup Apps policy can override launch, and actual sign-in delivery remains an acceptance check. Setup preserves the startup choice; uninstall removes it only when it names the uninstalled copy.

Default tests: 54 passed, 7 optional integrations ignored. An optional registry round-trip passed using a private test key, leaving the actual Run entry untouched. Runtime checks passed for open/close reversal, maximized-vs-fullscreen behavior, disabling/re-enabling the policy, foreground-focus preservation and pinned-view restoration. `scripts/smoke-desktop.ps1` repeats the desktop checks against an isolated test instance and never toggles startup registration; it temporarily activates its own test form and restores the pointer/foreground in cleanup.

## Live music spectrum

`src/spectrum.rs` captures the default Windows render endpoint using shared-mode WASAPI loopback while a real SMTC session is playing and Music/Lyrics or compact playback is visible. Five frequency bands span 40 Hz–16 kHz, with boundaries at 40/160/630/2500/8000/16000 Hz. A 2048-point Hann-windowed FFT analyzes channels independently, preserving anti-phase stereo; band peaks use a fixed decibel scale. PCM 8/16/24/32-bit, float32 and matching extensible formats are accepted, with up to eight channels. Unsupported formats/device errors leave quiet bars and retry; default-output changes are checked every two seconds.

The compact spectrum replaces the decorative pause mark; its title/lyric is clipped before the bars. Music/Lyrics replaces the PLAYING badge with a larger spectrum while retaining all playback controls. New readings arrive around 30 Hz and attack/release smoothing uses the existing frame clock; silence settles the frame animation. Pausing or hiding the visualizer releases the capture client. This reads the system output mix, including other applications on that endpoint, rather than isolating the current player. No microphone, audio files, external DSP package or network service is involved.

Windows GNU tests: 48 passed, 6 optional integrations ignored. The live loopback test separately passed on stereo 48 kHz float output and observed a nonzero spectrum. Compact and expanded Music captures showed changing bar pixels during existing playback; the checks sent no playback commands or volume changes. See `VALIDATION.md`.

## Control Center, external OSD and glass transparency

The control bubble right of the tab rail (or the menu's **Control Center** entry) opens a Wi-Fi/Bluetooth/Sound/Brightness panel. Worker threads keep it current: one Core Audio loop (default-endpoint volume, mute), one WMI brightness loop (`root/WMI` monitors; `N/A` when unsupported), and an on-demand network loop refreshed when the panel opens and at about every 8 seconds that reads the active SSID, adapter status and connected Bluetooth devices. Wi-Fi and Bluetooth cards deep-link to `ms-settings:`; the Bluetooth card and the dedicated Bluetooth icon in the rail open the same connected-device panel. Bluetooth device addresses are resolved from the Windows Runtime device id, falling back to the instance-id form (`BLUETOOTHDEVICE_`/`DEV_` + 12 hex digits) when the runtime address is zero.

Connected Bluetooth battery percentages come from the Microsoft stack's cached device property `{104EA319-6EE2-4701-BD47-8DDBF425BBE5}` PID 2, read through `SetupDi*` plus `CM_Get_DevNode_PropertyW`. Values outside 0–100 are ignored rather than clamped, and devices without a driver-cached report show **Battery not reported** instead of a fabricated number. The refresh button and the panel-open path re-query the list.

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

## Windows battery status and charger OSD

The context/tray menu shows battery percentage and an AC indicator. Machines without a battery show `No battery`; unavailable readings show a neutral fallback. `GetSystemPowerStatus` is sampled at startup, on every Windows power broadcast (`WM_POWERBROADCAST`), and about every 30 seconds as a fallback. `src/power.rs` registers `GUID_ACDC_POWER_SOURCE` and `GUID_BATTERY_PERCENTAGE_REMAINING` notifications on the Lieve window; startup duplicates are skipped and unknown readings bridge instead of producing false plug events.

Plugging in shows a 300 × 56 OSD with a charging battery icon, animated fill and a one-pass sweep; unplugging shows the percentage in a compact indicator before auto-hiding. The **Power & battery** page (tab 12) shows percentage, charging state, a refresh control and preview plug/unplug buttons. Previews only exercise the same OSD/animation path and never change the real power source. Low-battery notifications remain as before.

A tray notification is requested once when a known battery is at or below 20% on battery power. AC power or recovery to at least 25% rearms the warning; unknown readings and 20/21% jitter do not repeat it. The warning latch resets on app restart. Windows notification preferences govern visibility. Unit tests and a Windows GNU release build passed; live notification registration, both OSD directions and a real capture of the preview path passed on the development machine. Real low-battery notification delivery still needs device acceptance testing.

Layered glass notes: expanded frames are published atomically. The renderer stores the next surface in `pending_frame` and `render::upload_layered(hwnd, surface, position)` pushes geometry and premultiplied pixels in one `UpdateLayeredWindow` call, so the old bitmap is never shown at the new rectangle.


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
- Right-click for eye colour presets, floating/minimal modes, Devil grin, Control Center, Bluetooth devices, Power & battery, Restart Lieve and Exit. The centre of the tab rail has a control bubble that opens the Control Center panel (Wi-Fi, Bluetooth, Sound and Brightness sliders; mute toggles and ms-settings deep-links). The Bluetooth icon beside it opens the connected-device list with per-device battery percentages. The tray's **Power: …** entry opens Power & battery with preview plug/unplug controls.
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

This is an alpha, not feature parity with the React prototype. Complete file-shelf parity, water/task reminders, connection alerts and further animation polish remain. Local installer/portable packaging is implemented; signing, a public release and clean-machine acceptance are pending. Per-user startup registration and fullscreen auto-hide are implemented as described above; sign-in delivery and game-specific fullscreen behavior still need acceptance checks. Battery status, charger OSD and low-battery alerts have runtime acceptance checks on the development machine; Bluetooth battery percentages depend on driver-cached reports and need more device coverage. Control Center covers Wi-Fi/Bluetooth/system volume/brightness basics; its hardware-control acceptance is described in `VALIDATION.md`. The tray icon is restored after Explorer restarts. Rendering occurs on window events and periodic state updates, with active spectrum frames while audio levels change. The tray currently uses the Windows default application icon. Mixed-DPI displays still require validation.

## Network and privacy

On track changes, title, artist and duration are sent over HTTPS to `lrclib.net` to retrieve lyrics. There are no accounts or analytics. Missing lyrics and network errors fall back to the song title. Network and media operations run off the UI thread. A slow network operation can delay later lyric requests; an explicit request timeout and retry UI are not implemented yet.

## Memory target

Target: around 30 MiB idle working set. This is a target, not a guaranteed cap. Measure the release build on the target machine, recording both working set and private committed memory. Do not trim working sets to obtain artificially low measurements.

```powershell
Get-Process lieve | Select-Object Id, CPU,
  @{n='WorkingSetMiB';e={[math]::Round($_.WorkingSet64/1MB,2)}},
  @{n='PrivateMiB';e={[math]::Round($_.PrivateMemorySize64/1MB,2)}}
```
