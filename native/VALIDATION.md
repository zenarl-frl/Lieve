# Native shell validation

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
