# Native shell validation

## Glass seam fix, white grin and keyboard OSD verification — 2026-10-26

- Diagnosed a mid-flight seam during reversed open↔close motions: `glass::configure` was keyed to the sampled Panel alpha, so the layered `UpdateLayeredWindow` shell silently fell back to the binary `SetWindowRgn` shell while the close spring was still running, then re-enabled during the next open — a visible backdrop flash. The shell now keeps the layered glass mode for the entire motion (expanded, OSD, compact, transition or unsettled content/pill) and only drops it at a fully settled idle.
- Automated probe with a fixture APPDATA (`transparency=45`, frosted): across open → natural close → mid-close reopen → final close, `WS_EX_LAYERED` toggled exactly twice (once when motion started, once when the shell settled back at its 144×36 idle); no style change was observed inside the motion. Repeat runs at other idle heights behaved the same.
- While re-checking, the idle height read 144×48 with grin-pose eyes: the probe scripts had not pointed `APPDATA` at the fixture and were reading the real developer settings (`devil_grin=true`). The scripts were corrected to set `APPDATA` to the fixture; a corrected run then read the expected **216×54 physical (144×36 logical)** idle with only the two fixture keys present. No application defect there.
- `mascot::draw_grin` no longer receives the eye colour: teeth keep their fixed near-white fill and the smile outline now uses a fixed light grey, so changing eye colours cannot tint the grin.
- Keyboard-equivalent OSD acceptance: posting `WM_APPCOMMAND` with `APPCOMMAND_VOLUME_UP` (the message Windows generates for the volume media key) moved the master volume of the default endpoint and raised the 300×56 OSD within the read-back latency (~80 ms audio worker + 100 ms UI timer), then auto-hid. This confirms the already-present endpoint read-back covers keyboard volume keys; per-monitor brightness hotkeys that bypass WMI remain out of reach, while WMI-reported changes are caught by the display worker.
- Windows GNU default tests: **44 passed, 0 failed, 5 ignored**. release build passed. `run-animation-check.ps1 -OnlySystem` regression: Control Center sliders and hardware round trip PASS, frosted/clear checker clear=77.99/frosted=1.03 with saved percentage PASS, external OSD and auto-hide PASS.
- Tests used isolated fixture APPDATA (except the documented misconfiguration above); the normal development instance was restored with web-spring motion afterwards. Remaining: reversal feel on other refresh rates/DPI scales, grin tint check with multiple colour presets, and brightness hotkeys on vendor drivers outside WMI.

## Control Center, external OSD and frosted glass — 2026-10-26

- Added `src/system_control.rs` with three worker threads on one update channel: a Core Audio loop (default-endpoint volume/mute), a WMI brightness loop (`root/WMI`; `Put` arguments must be VT_I4 — `u8`/`u32` variants failed with 0x80041005), and an on-demand network loop reading active SSID/adapter connection state and Bluetooth device names over the Windows Runtime, refreshed when the Control Center opens and around every 8 seconds. Device names are deduplicated by address. `src/system_ui.rs` renders the Control Center (tab 7), the Bluetooth list (tab 9) and the glass/effects view (tab 8), and `src/osd.rs` animates a 300×56 spring HUD with momentum-preserving retargets.
- External and panel-driven volume/brightness changes share the OSD. Drags route `DeviceCommand` directly, and read-back events are suppressed while a slider is actively dragged so panel changes are not echoed twice. A 2000 ms `WM_TIMER(8)` hides the HUD; timers occasionally fired a few ms before the spring expired and left a 56 px-tall ghost window, fixed by an expiry deadline plus a re-check that re-hides outside the timer tick.
- Added `src/glass.rs`: `WS_EX_LAYERED` plus premultiplied `UpdateLayeredWindow` compositing carry a 0–100% transparency (0% keeps the opaque painted shell, `transparency=0` disables layers), frosted mode requests accent state 3 through `SetWindowCompositionAttribute`, and unsupported Windows/display combinations fall back to clear transparency with a status caption. Opaque card masks now scale their strength with the content fade during transitions, so cards stay readable at partial transparency and idle zero-alpha frames skip layered presentation.
- Windows GNU default tests: **44 passed, 0 failed, 5 ignored** (`--include-ignored` runs are the live integrations). Existing settings/geometry/render tests still pass after the bisection; audio/brightness workers snapshot without new test surface. Temporary APPDATA use was kept for all runtime checks.
- Live hardware round trips passed: slider changes actually moved the real output volume and the real monitor brightness, both were restored afterwards, and read-back events matched. The Wi-Fi worker reported the machine's live SSID; the Bluetooth list showed adapter state without throwing on desktop machines without radios accepted.
- `run-animation-check.ps1 -OnlySystem` final run: Control Center sliders and brightness round trip **PASS (original values restored)**; frosted-vs-clear backdrop checker standard deviation **clear=48.76, frosted=3.04** with **PASS** for the saved transparency percentage; external OSD and auto-hide **PASS** (every sample included the height back inside the 36/48 idle bounds).
- Inspected captures of clear and frosted checker patterns, frosted over a real desktop, and the Bluetooth device list; controls and OSD read correctly at 150% DPI. The `-OnlySystem` regression tank left pinned state released and the normal development instance restored with web-spring motion.
- Remaining checks: OSD timing under multi-monitor rules, brightness fallback on laptops without PowerShell/WMI support, Bluetooth device names for different radio stacks (desktop radios currently list but can be sparse), control center layout on other DPI scales, and mixed mixed-DPI/layered interactions with floating mode like drag/blur conflicts. MSVC remains pending its toolchain prerequisites; no memory/FPS ceilings are claimed.

## Coverage anti-aliasing, label alignment, restart and Devil grin — 2026-10-26

- Replaced compatible DDB surfaces with top-down 32-bit DIB sections. After GdiFlush, 4x icon/mascot samples are explicitly averaged into opaque BGRA rasters and copied one-to-one. This avoids HALFTONE's visible dither patterns. Artwork has cached area resampling for minification, bilinear enlargement and 4x rounded-edge coverage. Cached card masks also resolve border/fill coverage rather than using aliased GDI outlines. Source thumbnail resolution remains the limit on photo detail.
- Converted Focus/Water labels from manual offsets to centered text rectangles; Custom colour's swatch/text group is measured and centered together. Glow percentages and list counts use true trailing alignment. The Appearance column now fits five toggles with a caption hierarchy.
- Added `src/restart.rs` and the native tray/context **Restart Lieve** command. The old window/event loop is destroyed before spawning the executable. Settings reload and paused/running Focus transfer through a validated private argument; an absolute running deadline includes startup delay. Errors launching a child are reported. Ordinary Exit still does not persist Focus.
- Added saved `devil_grin` (default false), Appearance and tray controls, a bounded 1.9-second smile envelope and a 12-second visible-mascot schedule. Procedural upper/lower fangs clip to the animated mouth, with squint/head lift resolved at subpixel coverage. Idle height expands to 48 logical pixels only when the visible mascot's grin option is enabled; compact playback and expanded panel sizes remain unchanged. Reduced mode is static.
- Windows GNU default tests: **41 passed, 3 optional live integrations ignored**. New checks cover deterministic coverage/area averages and fractional rounded-edge pixels, grin lifecycle/settling, Focus restart deadlines including startup/expiry, and settings round-trip with the new option. Reusable DIB/GDI resource cleanup tests passed.
- Release build passed. Shell animation, Music-only Lyrics, centered play disc, To-do, Water, Shelf and Appearance regression checks passed. At 150% DPI, cropped text-ink bounds verified both Focus labels and all three Water labels centered within 3 physical pixels; inspected Custom colour/Glow and final UI captures visually.
- Grin UI checks passed: enabling preview produced white sharp-tooth pixels within the mascot region; disabling removed them; the setting persisted. The captured expression was inspected visually. The final subpixel head-lift refinement repeated these checks successfully.
- Inspected the app's shared native tray/context popup and found command 21 labeled **Restart Lieve**. Executing that command replaced PID **2264 → 19956** in the final run; the old process exited, exactly one new instance remained, `devil_grin=true` survived reload and the running Focus deadline was present in the child command line. The resumed Focus capture showed **24:58** with Pause, then the test reset it. No player commands were issued.
- Post-rendering-change geometry delivery sample: **132 intervals, median 8.35 ms, p95 9.45 ms, one interval >25 ms**. This measures changed-window geometry, not presented FPS. The later subpixel-expression-only refinement did not rerun that benchmark.
- Tests used temporary APPDATA; restart children were closed and the normal development instance restored. Remaining checks: actual tray-icon mouse interaction across shell themes, restart failure paths, grin on alternate DPI/refresh scales and source-thumbnail quality from other players. MSVC remains pending prerequisites; no memory/FPS ceiling is claimed.

## Balanced Music composition and Music-only Lyrics — 2026-10-26

- Removed Lyrics from global navigation. The only Lyrics action is now in Music's lower-right control lane. Artwork and Lyrics are two states of the same player, sharing the same timeline and playback row; the Settings gear returns to the prior state. Other tabs do not render the Lyrics action.
- Reworked navigation into five centered 64 px cells in a 320 px rail, with a separate Settings control and equal outer margins. Music's framed card aligns artwork, metadata and the full-width progress rail; the 36 px white playback disc centers at x=200 and previous/next at x=148/252. Time labels use left/right-aligned text rectangles rather than guessed offsets. Status and Lyrics occupy mirrored outer lanes.
- Added `src/icons.rs`: supersampled 4x native vector glyphs drawn through a reused surface, replacing ASCII playback symbols. Added a secondary caption font, subtle card borders, measured text ellipsis, and hover-target feedback. Duplicate album/title metadata is hidden and the two-line title/artist block recenters vertically.
- Shared Music control rectangles drive rendering, pointer routing and hover detection. Music/lyrics states both retain play/pause, skip and seek routing. The shell stays 400 × 220 logical pixels and artwork stays tied to the SMTC cache.
- Windows GNU default tests: **38 passed, 3 optional integrations ignored**. Release build passed. Shell reduced/full motion and To-do add/edit/toggle/cancel/delete smoke checks passed with the new nav coordinates.
- At 150% DPI, native pixel checks confirmed the white play disc is centered on the panel in both artwork and lyrics modes. Isolated UI checks passed for Music's Lyrics toggle, Settings open/return to Music Lyrics, and toggling back to artwork. The final post-caption-refinement build repeated the visual/control checks successfully.
- Inspected final Music, Lyrics, Appearance and To-do captures. Actual SMTC artwork displayed correctly; artwork/lyrics modes preserved the centered playback row and progress rail. Water persistence, Shelf reference removal, fourth-row To-do toggling, and Appearance mascot/glow controls passed in the seven-view check.
- Runtime fixtures used temporary APPDATA and the normal instance was restored with saved appearance and web-spring motion. Tests did not issue playback commands to the user's player. Physical playback-command acceptance, hover feel, alternate DPI scales and keyboard/accessibility navigation remain manual checks. No new FPS or memory ceiling is claimed.

## Headerless panel, grouped Lyrics/Settings and SMTC artwork — 2026-10-26

- Removed the expanded metadata/eyes/clock/power/pin header. Final panel dimensions are **400 × 220 logical / 600 × 330 physical pixels** at the tested 150% DPI. Five primary tabs share one top rail with adjacent quote-bubble Lyrics and Settings gear controls. Music stays selected for its Lyrics view; closing Settings restores the preceding view, including Lyrics.
- Updated tab mapping, spring targets, pointer routing and list smoke coordinates. Body rendering uses a shared 40 px viewport offset rather than scaling; cover clipping accounts for device-space viewport origin. Pin/power are available via the tray/context menu. Expanded panels without an eye preview no longer schedule mascot blink redraws.
- Added `src/artwork.rs` and enabled Windows Streams/Imaging bindings. The media worker requests the active player's SMTC Thumbnail; an AgileReference transfers it to a dedicated decode worker so provider stream access does not block metadata/playback command processing or painting. Queued obsolete work is dropped before decoding.
- Decoder caps compressed streams at 8 MiB, bounds source dimensions and creates aspect-preserving thumbnails up to 256 px. BGRA pixels are shared through Arc and drawn as rounded, center-cropped covers. Track keys include source app/title/artist/album/duration; track changes clear old covers, stale results are ignored, missing images retry after two seconds, and successful covers refresh after 30 seconds. Transient refresh failures retain a matching current cover. Real sessions use explicit Loading/No artwork states, with generated art restricted to demo playback.
- Windows GNU default tests: **38 passed, 3 optional live integrations ignored**. New tests cover thumbnail dimensions, track/cache transitions and stale results, delayed retries, retaining a cover through transient failure, BGRA channel order, bottom-up BMP conversion to top-down pixels, and invalid image data. The strengthened two-row orientation fixture passed separately after the full run.
- Optional live `media::tests::current_session_artwork_decodes`: passed against an active player, decoding a **120 × 120** thumbnail. It reads metadata/artwork without issuing playback commands.
- Release build passed. Shell smoke tests passed for 400 × 220 geometry, center anchoring, intermediate frames and reversal in reduced/full modes. To-do add/toggle/edit/save/blank rejection/cancel/delete tests passed with updated coordinates.
- Isolated seven-view capture/control check passed at 150% DPI: Water persistence, Shelf reference removal, fourth-row To-do toggle, Appearance mascot toggle/glow drag. Grouped controls were verified by active-icon pixels: Lyrics on, Settings on, Settings close back to Lyrics, and Lyrics off back to Music. Inspected the final Music image with actual album artwork and no expanded header, plus Lyrics, Appearance and To-do captures. The current playing track changed during development and the runtime cover matched the current song's metadata.
- Test fixtures used temporary APPDATA; the normal development instance was relaunched with saved appearance and web-spring motion. No player commands were sent by the artwork/UI checks. Remaining checks include more player-specific thumbnail formats, provider disconnects/slow streams, physical floating drag and keyboard/accessibility navigation. MSVC remains pending prerequisites; no new memory ceiling is claimed.

## Compact expanded panel — 2026-10-26

- Reduced expanded geometry from **470 × 322** to **400 × 260 logical pixels**, approximately 31.3% less area. At the tested 150% DPI, the final window is **600 × 390 physical pixels**. The docked expanded corner radius is 22 logical pixels.
- Added `src/layout.rs` to share panel dimensions, tab origin/spacing, four-row list geometry, Add bounds and row action lanes. Native-scale layer compositing and pointer centering use the new width; spring targets and tab-pill positioning use the same constants.
- Reflowed the header and six-tab rail; Music artwork is 80 px with a 276 px seek bar. Lyrics/Focus/Water fit the shorter body. Four To-do/Shelf rows remain visible; counts moved to Add bars and task text clips before edit/delete actions. Appearance's two columns fit preview/presets/custom colour/glow and four toggles. Existing font sizes are retained and development footers removed.
- Windows GNU unit tests: **35 passed, 2 optional live integrations ignored**. Existing geometry/anchor tests were updated to the compact target. Release build passed.
- Updated `smoke-animation.ps1` and `smoke-todo.ps1` to the new dimensions and hit coordinates. Runtime shell checks passed in reduced and web-spring modes: centered anchor, intermediate sizes, reversal and final 400 × 260 bounds. To-do add, completion toggle, edit prefill/save/blank rejection/cancel, deletion and Add cancellation passed.
- Captured and visually inspected all seven panels (Music, Lyrics, Focus, Water, Shelf, To-do, Appearance) at 150% DPI. Confirmed readable text, separate header lanes, all four list rows and action buttons within the new frame. A Focus capture showed the running-state Pause button after Start; this was a button/layout check rather than a full timer-duration test.
- Additional isolated UI checks passed: Water +200/+400/reset persisted 200/600/0; Shelf removed the reference while retaining the original README file; clicking the fourth To-do row toggled only that fixture and was restored; Appearance mascot toggled off/on and glow dragged to 100% then restored to 50%.
- Runtime fixtures used temporary APPDATA. The normal development instance was relaunched with its saved appearance and web-spring motion. Remaining checks: compact layout on other DPI scales, long real media metadata, native colour/file pickers and physical touch/keyboard navigation. No new performance or memory ceiling is claimed.

## Frame-pacing and rendering smoothness — 2026-10-26

- Baseline diagnosis: repeating `SetTimer(...,16)` produced coarse geometry delivery; gaze retargets also reset that timer, and every paint allocated fresh DCs/bitmaps. Eye coordinates were rounded directly to whole physical pixels.
- Added `src/frame_clock.rs`: a high-resolution one-shot waitable timer, integrated with `MsgWaitForMultipleObjectsEx` on the UI thread. Monitor refresh sets cadence, capped at 240 Hz; invalid/unknown rates default to 60 Hz. Retargeting does not reset an armed deadline. Late frames advance to the next future deadline instead of bursting. Painting is forced after the application borrow is released. Settling cancels the clock. An unavailable high-resolution timer falls back to WM_TIMER. No global timeBeginPeriod or additional worker is used.
- Added reusable grow-only output/stage/eye surfaces in `src/render.rs`, skipping redundant SetWindowPos/region work. The 4x eye sprite is downsampled while text and other content remain at native scale. Layer DCs release selected custom fonts after drawing; buffers own and release their GDI objects.
- Replaced repeated zero-velocity gaze tweens with critically damped tracking that preserves velocity through new targets. Dormant cursor polling is 50 ms (500 ms while sleepy); active eye frames sample the current pointer. Position and velocity continuity are unit-tested.
- Added `scripts/measure-motion.ps1`, using out-of-context window-location events restricted to the Lieve process/window. It collects changed-geometry intervals over the first 300 ms of four opening transitions, with message-driven event delivery and no screenshot capture during measurement. The script releases its test pin afterwards.
- Before: **40 intervals, median 27.01 ms, p95 36.45 ms, 23 intervals >25 ms**. After: **138 intervals, median 8.30 ms, p95 9.02 ms, 0 intervals >25 ms**, on the same local monitor at 150% DPI. The benchmark observes geometry-event delivery, not GPU/compositor presentation or sustained loaded FPS; counts can differ because cadence changed.
- Windows GNU tests: **35 passed, 2 optional live integrations ignored**. New tests cover refresh-rate cadence/deadline skipping, gaze velocity continuity, and repeated buffer growth/reuse/drop returning the GDI object count to baseline after flushing. Float continuity assertions use numerical tolerance; stale-handle GetObjectType checks were replaced by resource counts because GDI can defer/cache handle deletion.
- Release build passed. Reduced/full motion shell smoke tests passed for anchor, intermediate sizes, reversal and settling. To-do add/toggle/edit/cancel/delete smoke tests passed. Left/right eye response passed with the supersampled renderer, and native stills were inspected.
- The later live sleepy check was **interrupted by desktop input**, so it is not counted as a passed sleep/wake test for this build. Threshold/blink/filter logic remains unit-tested, and the previous build's sleep/wake captures remain historical results. The normal development instance was restored with web spring active.
- Latest point-in-time process observation after measurement: **17.75 MiB working set, 2.57 MiB private memory**. This is not a peak, an idle CPU measurement or a guaranteed cap; reusable buffers retain their capacity.
- Remaining checks: user-perceived smoothness, frame presentation under CPU/GPU load, mixed refresh/DPI display changes, the WM_TIMER fallback path on older systems, and an uninterrupted physical sleep/wake check. MSVC remains unverified without its prerequisites.

## Cursor-follow and sleepy mascot update — 2026-10-26

- Added `src/mascot.rs` for monitor-relative cursor normalization, web-scaled translation/tilt, cubic-bezier timing, interruption-safe pose transitions and sleepy/blink composition. No new dependency was added.
- Idle eyes track the cursor with a 600 ms `cubic-bezier(.22,1,.36,1)` response. Coordinates clamp to the active monitor, including negative monitor origins. The group rotates around its center without shifting the notch geometry.
- `GetLastInputInfo` and `GetTickCount` determine Windows-session inactivity with wrapping 32-bit arithmetic. At 26 seconds, eyes center, tilt 3.5 degrees and narrow to 26%; input wakes them. Blink narrowing still bottoms at 6% and recovers to the current lid pose.
- An 80 ms cursor/activity poll runs only for visible idle mascots, slows to 500 ms in sleepy mode and stops for panel/compact/minimal/disabled mascot or reduced motion. Polls with identical targets do not invalidate or restart the tween. Pose motion joins the existing short-lived frame timer.
- Windows GNU unit tests: **31 passed, 2 optional live integrations ignored**. Four new tests cover monitor normalization/clamping, invalid bounds, inactivity threshold and clock wrap, interruption continuity, reduced-motion neutral state, and sleepy/blink composition.
- Windows GNU release build passed. Existing shell animation smoke tests passed in reduced and full modes, including intermediate frames, anchored center, reversal and settling. To-do add/toggle/edit/cancel/delete regression smoke test passed.
- Isolated desktop check passed at 150% DPI: cursor moved to opposite monitor sides changed the eye silhouette; after at least 26 seconds idle, the most-open eye silhouette narrowed from **30 physical pixels** to **12 physical pixels**; input restored the awake silhouette. Multiple captures selected the most-open sample to avoid confusing a transient blink with sleep. Left/right/sleepy/awake window images were inspected visually.
- Runtime checks used temporary APPDATA and restored the original pointer position in cleanup. The normal development instance was relaunched with web spring active. A PowerShell-only diagnostic syntax error (`26_000`, unsupported numeric separators in PowerShell 5.1) was corrected before the successful mascot check; no Rust change was needed for that failure.
- Remaining manual checks: real multi-monitor/mixed-DPI movement, pointer-follow under sustained CPU load, physical keyboard-only wake and user acceptance of the subtle tracking amplitude. No new memory or CPU ceiling is claimed. The idle source is desktop-session input rather than the web page's own events.

## Web-matched motion update — 2026-10-26

- Compared native motion with `prototype/src/components/Notch.tsx`, `TabBar.tsx` and `Mascot.tsx`. Replaced the fixed 260 ms quintic resize with the shell's analytic damped spring (`360/32/0.85`), preserving velocity when retargeted. Width, height, position and radius settle together; the tab highlight uses `480/38/1`.
- Replaced whole-panel StretchBlt with native-size, centered layers and constant-alpha blending. Idle/compact/panel use sequential outgoing/incoming fades (180/140/160 ms); the shell clips content rather than scaling typography. Hit testing uses the same centered layout and ignores invisible incoming controls.
- Added short, scheduled eye blinks (100 ms close, 120 ms blink state, 500 ms recovery). Frame updates stop on settling; blink scheduling uses varied intervals and skips hidden/minimal/disabled mascots and reduced motion.
- Found Windows client-area animation disabled on this machine. Verified the instant-resize behavior in system/reduced modes. Added persisted per-app System/Full/Reduced motion choices to the tray/context menu; system remains the default. `WM_SETTINGCHANGE` refreshes the effective preference.
- Windows GNU unit tests: **27 passed, 2 optional live integrations ignored**. Coverage includes spring reference response, velocity/position continuity on reversal, finite/anchored geometry, exact settling, interrupted wait-mode fades, tab retargets, blink phases, motion overrides and legacy settings.
- Release build: passed. An attempted rebuild while the existing executable was active hit Windows' executable lock; the development instance was then closed gracefully before rebuilding and restored afterwards.
- Added `scripts/smoke-animation.ps1`: runtime checks passed at 150% DPI for intermediate opening sizes, center anchoring, close/open reversal, and final open/closed sizes. Reduced mode also passed endpoint checks. Example logical samples from the final animated run: `144×36` at 0 ms, `267×144` at 62 ms, `391×253` at 123 ms, `436×293` at 185 ms, `467×319` at 263 ms, and `470×322` at 401 ms. Capture work can delay sample timestamps; these are not frame-pacing measurements.
- Inspected native window stills at an outgoing-content frame and the settled panel. Confirmed centered eyes, unscaled content and the final layout; the live power label showed `79% · AC`. This is a visual check, not an automated pixel comparison with the browser.
- To-do UI smoke test passed again with web spring active: add, toggle, edit prefill/save/empty rejection/cancel, delete and Add cancellation. Runtime fixtures used isolated APPDATA. The normal Lieve instance was relaunched and **Motion: web spring** was enabled and persisted for the requested animation treatment; Windows' global preference was not modified.
- Remaining manual checks: frame pacing under load, mixed-DPI/floating transitions, cursor-follow eye expressions, long compact lyric motion and pixel-level antialiasing/glow parity. MSVC build remains pending its toolchain prerequisites. No new RAM/CPU ceiling is claimed.

## Windows battery status update — 2026-10-26

- Added local `GetSystemPowerStatus` sampling at startup and about every 30 seconds using the existing one-second timer. No extra worker thread or dependency package was added; `windows-sys` enables its Power API feature.
- The expanded header places a clipped battery/AC label below the clock. Missing batteries and unavailable readings have explicit fallback labels.
- A known battery at ≤20% on battery power requests one tray warning. The latch rearms on AC or known recovery to ≥25%, avoiding repeated warnings from threshold jitter. Unknown readings retain the latch. The latch is session-only and notification display depends on Windows settings.
- Four new tests cover raw Windows sentinel values (including flags/percent 255), absent batteries, invalid percentages, warning thresholds, jitter, recharging/recovery, and unknown samples preserving the latch.
- Windows GNU `cargo +stable-x86_64-pc-windows-gnu test --locked`: **21 passed, 2 optional live integrations ignored**.
- Windows GNU `cargo +stable-x86_64-pc-windows-gnu build --release --locked`: passed, using the same temporary toolchain and `RUSTFLAGS='-C link-self-contained=yes'` as the To-do update below.
- An isolated runtime/header-capture check was blocked by an already-running Lieve instance. The check stopped before launching another instance or modifying its pin/data. No new UI screenshot or live low-battery notification was verified.
- Remaining acceptance checks: updated header layout at different DPI scales; charger connect/disconnect refresh; a real low-battery notification and Windows notification suppression; resume from sleep. MSVC build remains pending installation of C++ Build Tools/Windows SDK. No new RAM measurement was taken.

## To-do editing fixes — 2026-10-26

- Added an explicit `edit` action to each task row, reusing the native input dialog with prefilled/selected text and a Save button. Updating the text retains the task's completion flag and list position; empty input is rejected and cancellation retains the old value.
- Restricted task actions to visible row bounds so clicks in the margins or row gaps do not toggle/delete/edit a task.
- Extended `scripts/smoke-todo.ps1` to check edit prefill, blank-input rejection, persistence of edited text and completed status, and cancellation after changing the input.
- Fixed cancellation and unchanged input rewriting the task file. A failed save restores the previous tasks and scroll offset before displaying the error.
- Clipped task text to its lane so wide text cannot paint over the edit/delete actions.
- Let `DefWindowProcW` handle `WM_NCCREATE` after attaching dialog state, preserving the dialog caption. The modal dialog now posts its own wake-up message on destruction; the test no longer supplies a wake-up workaround.
- Fixed the test driver's null title filters with C# wrappers: PowerShell's string conversion could turn `$null` into an empty title filter and fail to locate a prefilled EDIT control. Read edit text through cross-process `WM_GETTEXT`.
- Installed temporary Rust and GNU compiler tools under the pre-approved agent temp directory, without changing the repository's default target or the user's PATH. Rust 1.99.1, `stable-x86_64-pc-windows-gnu`, with `RUSTFLAGS='-C link-self-contained=yes'` and w64devkit on the process PATH.
- `cargo +stable-x86_64-pc-windows-gnu test --locked`: **17 passed, 2 ignored** (optional live integrations).
- `cargo +stable-x86_64-pc-windows-gnu build --release --locked`: passed.
- Updated UI smoke test passed against the release executable: add/persist, completion toggle, edit prefill, blank rejection, edited text retaining completion, cancelling a changed edit, delete, and cancelling Add. Cancellation checks both contents and the file's last-write time. Test data used isolated APPDATA in the agent temp directory; personal task storage was not used.
- `git diff --check`: passed. Temporary diagnostic instrumentation was removed before the final build/test run.
- MSVC verification remains pending: this machine lacks Visual Studio C++ Build Tools/Windows SDK; its PATH resolved `link.exe` to a non-MSVC utility. Physical keyboard navigation, visual layout, save-failure UI/rollback, restart persistence, screen readers and multi-monitor positioning still require acceptance testing. No new RAM measurement was taken.

## To-do interaction verification

- Added repeatable Windows UI smoke test: `scripts/smoke-todo.ps1` (requires Lieve already running).
- Corrected the test driver's DPI context and physical-coordinate conversion. The previous driver could send clicks to the wrong control.
- For synthetic synchronous modal commands, the driver queues a wake-up message before checking owner persistence. Real queued user input and synchronously injected commands do not have identical message-loop timing.
- Repeated runtime checks passed: Add dialog → enter unique task → save file; completion toggle → save completed flag; delete → remove persisted task; cancel → preserve task data.
- All uniquely named fixtures created by failed and successful runs were removed through Lieve's UI. No original user task was intentionally changed. The test released the pin it acquired.
- This closes the earlier unresolved dialog-open/add/cancel smoke-test finding. Physical keyboard navigation, screen readers, restart persistence, and multiple-monitor modal positioning still need manual acceptance testing.

## Native To-do update

- `cargo test`: **17 passed**, 2 optional live integration tests skipped. Added task-state parsing, invalid-input handling, count and length limits.
- `cargo build --release`: passed; new executable launched.
- Added native input dialog, completion toggles, deletion, wheel navigation and local task persistence. Dialog input/cancel, restart persistence, keyboard focus and full interaction still require manual acceptance testing; unit tests are not an end-to-end UI guarantee.
- A synthetic pointer-message smoke test did not find the task dialog; this check is unresolved and must not be counted as a passed UI test.
- The root README now describes build prerequisites, current features, privacy, data paths, known gaps and cross-device continuation.

## File shelf update

- Unit tests: 14 passed, 2 live checks skipped. New tests reject relative/missing references, reject duplicate entries, and verify that removing a reference leaves the original file intact.
- Release build passed and the updated application was launched.
- A synthetic cross-process WM_DROPFILES test did not validate drop persistence. Its locally allocated HDROP payload cannot be treated as a reliable Explorer drag/drop simulation. No claim of a passed end-to-end drag/drop test is made.
- Real Explorer drop, native picker, item opening, wheel navigation and app-restart persistence need interactive acceptance testing.
- Latest point-in-time working set observed during smoke-test attempts: approximately 29.32 MiB; not a memory ceiling.

## Native Appearance panel update

- `cargo test`: 12 passed, 2 live integration tests skipped. Settings round-trip includes mascot/compact-lyrics/glow and tests clamp invalid glow values.
- Release build passed.
- Opened Appearance through the header button with a synthetic click and captured the live window; visually verified the two-column preview/presets/slider/toggles layout at 150% DPI.
- The test unpinned the panel afterwards. No preset/toggle changes were made to user settings by the screenshot test.
- Working set after the UI check: **29.27 MiB**, not a guaranteed cap.
- Full pointer-drag, colour-picker, settings-restart and accessibility/keyboard acceptance checks remain manual.

## Resize animation update

- Unit tests: 11 passed, 2 live integration checks skipped. Added interpolation endpoint, clamping, monotonic-size and no-overshoot checks.
- Release build passed.
- Runtime samples (logical window coordinates) confirmed intermediate dimensions on open: `144x36 → 379x243 → 433x289 → 463x315 → 470x322`; close: `470x322 → 321x191 → 181x68 → 157x48 → 145x37 → 144x36`.
- The test opened via tray-pin, then explicitly unpinned/closed afterwards. It did not leave the window pinned open.
- Point-in-time working set after the animation test: **29.21 MiB**. This does not establish a peak or sustained ceiling.
- Frame pacing under load, reduced-motion launch and cursor leave/return debounce still need interactive acceptance testing. Frame timer interval is 16 ms, not a guarantee of 60 FPS.

## Latest alpha: Windows music and lyrics

### Centred-eyes and daily-water update

- `cargo test`: 9 passed, 2 live integration tests skipped (these had passed in the prior build). New tests cover same-day water totals, daily rollover and malformed values.
- `cargo build --release`: passed.
- Captured the expanded native window and visually checked that the eyes stay at the header centre while actual Windows song metadata occupies the left lane. Idle geometry is now 144 × 36 logical pixels with a centred 68 × 20 eye group and no clock.
- Point-in-time working set with expanded panel: **29.22 MiB**. Still not a guaranteed maximum.
- Water file persistence across a real app restart and midnight rollover still require interactive acceptance testing; parser behaviour is unit-tested.

- Release build passed; 8 tests passed, including live Windows media-manager access (1 session found) and a real LRCLIB lookup for a public test track.
- Latest point-in-time release measurement: **29.34 MiB working set**, **4.88 MiB private committed memory**, **277.5 KiB executable**. An earlier run measured **30.30 MiB working set**, so this is not a proven <=30 MiB cap. HTTP/WinRT system components can increase the working set further when used.
- Tests do not send playback commands to the user's player. Actual play/pause/seek, full-song lyric timing and unsupported-player behaviour still require interactive validation.
- Focus/water UI is implemented but needs interactive acceptance testing, including the timer notification under Windows notification settings.
- No sustained memory soak test or complete-feature acceptance test has passed. The app is not yet a finished release.

## Historical shell-only results

Local release build, 2026-10-25. These results cover the native shell only, not all planned Lieve features.

## Automated checks

- `cargo check`: passed, no warnings.
- `cargo test`: 3 passed (settings round-trip, invalid colours, unknown/missing fields).
- `cargo build --release`: passed.
- Window message smoke test: 196 × 36 collapsed, 380 × 194 pinned/expanded, then 196 × 36 after unpin/leave (100% display scale).
- Second executable launch: exited, original process remained the only Lieve instance.

## Observed process memory

After launch and the expand/collapse smoke test:

| Metric | Observed |
|---|---:|
| Working set | 9.43 MiB |
| Private committed memory | 1.59 MiB |
| Accumulated CPU time | 0.046875 s |
| Release executable size | 139 KiB |

No working-set trimming was used. Values are a point-in-time observation, not a sustained ceiling or an idle CPU utilisation measurement. The executable uses Windows system libraries; its small file size does not represent all memory usage.

## Needs interactive validation

- Tray icon visibility and context menu on the user's desktop.
- Custom colour picker acceptance/cancellation and settings surviving restart.
- Floating drag, minimal mode and Explorer restart recovery.
- Multiple monitors, different DPI scales and fullscreen applications.

Rustfmt was not available in the installed minimal Rust toolchain; formatting check was not run.
