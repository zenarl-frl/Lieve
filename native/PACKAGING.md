# Windows distribution

Packages are local, unsigned **alpha** builds for Windows 10 1809+ / Windows 11 x64. Creating them does not publish a GitHub release.

## Build packages

Requirements: the native Rust build prerequisites in `README.md`, Windows PowerShell 5.1+, and NSIS 3 (3.13 recommended) for the installer. Portable-only builds do not require NSIS.

From `native/`:

```powershell
# For MSVC, keep the CRT self-contained. This affects this terminal only.
$env:RUSTFLAGS = "$env:RUSTFLAGS -C target-feature=+crt-static"
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\package-windows.ps1 `
  -NsisPath "C:\Program Files (x86)\NSIS\makensis.exe"
```

The script runs a locked release build. Close the running development executable when a rebuild needs to replace it. If the release binary is already built/tested, use `-SkipBuild`. `-Executable <path>` selects an explicit prebuilt binary; `-Toolchain <rustup-name>` and `-Target <triple>` select the Cargo build toolchain/target. NSIS can also be found through PATH or its standard Program Files location. All build paths are resolved from the script, so invoking it from another working directory works.

```powershell
# Use the existing release binary and build only a portable ZIP.
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\package-windows.ps1 `
  -SkipBuild -PortableOnly
```

Default outputs for version 0.1.0:

- `dist/Lieve-0.1.0-windows-x64.zip`
- `dist/Lieve-0.1.0-windows-x64-setup.exe`
- `dist/Lieve-0.1.0-windows-x64-SHA256SUMS.txt`

The ZIP contains only `lieve.exe`, a Windows usage guide, and `build-info.json`. Metadata records alpha status, source commit, dirty-tree state, executable SHA-256, NSIS version and direct DLL imports. Source metadata can be unknown when building without Git; it does not block packaging. The packager checks PE architecture/subsystem and rejects unexpected non-system DLL imports. This audit covers direct imports; runtime Windows/WinRT facilities still depend on the target OS/device. The current tested GNU release imports only Windows-supplied DLLs.

The installer contains the same executable bytes as the portable ZIP. Generated artifacts and temporary staging directories are under ignored `dist/`; only scripts and documentation belong in Git. Check integrity by comparing `Get-FileHash <artifact> -Algorithm SHA256` with the companion sums file. Compiler output is staged before replacing the finished installer.

## Install, update and uninstall

- Installation is **per-user**, requests no elevation, and defaults to `%LOCALAPPDATA%\Programs\Lieve`.
- Start menu launch/uninstall shortcuts are installed; the desktop shortcut is optional. The finish page offers launching the app. Silent setup (`/S`) does not launch it.
- Apps & features / Installed apps registration is under the current user's 64-bit Uninstall registry view.
- Updates reuse the registered installation directory. Setup can close an existing app only after verifying its process executable is exactly `$INSTDIR\lieve.exe`. Open modal dialogs prevent automatic closure; another portable/development copy is left running. No forced process termination is used.
- Lieve is single-instance per desktop. Exit a running portable copy before switching to the installed shortcut.
- Setup does not enable or replace autostart. Configure **Appearance → System → Start with Windows** from the copy you want to launch at sign-in.
- Uninstall removes the three package files, uninstaller, and installer-owned shortcuts/registration. Directory cleanup is non-recursive, preserving unknown files and `%APPDATA%\Lieve` data.
- A `Lieve` startup value is removed only when its command exactly names the executable being uninstalled (quoted or unquoted). Registration for another copy is retained.

Portable means portable program files: settings still live in `%APPDATA%\Lieve`, shared with the installed edition. Normal Exit/update does not transfer an in-memory Focus session; the app's explicit Restart command provides that transfer.

## Isolated packaging verification

Compile a distinct test identity; the script refuses the production installer identity:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\package-windows.ps1 `
  -SkipBuild -TestPackage -NsisPath "C:\Program Files (x86)\NSIS\makensis.exe"

powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\smoke-package.ps1 `
  -Setup .\dist\tests\Lieve-0.1.0-windows-x64-test-setup.exe `
  -PortableZip .\dist\tests\Lieve-0.1.0-windows-x64-test.zip `
  -WorkRoot "$env:TEMP"
```

`WorkRoot` must already exist. Tests create a fresh random child directory and refuse an existing test registry identity or Start menu folder. They use isolated APPDATA, a separate uninstall identity and a private simulated Run key, leaving actual startup registration untouched. Keep the UI idle during testing. A running development instance is temporarily closed for the standalone-launch check and restored afterward.

Checks cover ZIP/installer hash parity, install paths containing spaces, quoted uninstall commands, Start menu targets, standalone launch, in-place update while running, graceful closure, data and unknown-file preservation, owned startup cleanup and preservation of another copy/registration. See `VALIDATION.md` for the observed results. Actual interactive wizard acceptance on other Windows machines, signing, public release publication and a clean-machine matrix remain separate steps.
