Lieve - Windows alpha preview
=============================

Native Rust + Win32/GDI Dynamic Island for Windows 10 (1809+) / Windows 11 x64.
Project: https://github.com/zenarl-frl/Lieve

PORTABLE
Extract the ZIP to a folder you can keep, then run lieve.exe. No installer,
browser runtime or administrator privileges are required. Settings are shared
with the installed edition under %APPDATA%\Lieve; portable refers to the program
files, not to a separate portable data profile.

Lieve uses one instance per desktop. To switch from a running portable copy to
the installed edition, Exit the current copy, then open the installed shortcut.

INSTALLER
Setup installs for the current user, by default under
%LOCALAPPDATA%\Programs\Lieve. It adds Start menu entries and offers an optional
desktop shortcut. An update uses the existing installation folder. Setup closes
only a running Lieve whose executable matches that installation folder.

USING LIEVE
- Hover the notch to expand; move away to collapse. The tray provides pin/exit.
- Ctrl+Alt+Space opens/focuses the panel. Press again to close it.
- Tab / Shift+Tab selects controls; Enter / Space activates the selection.
- Ctrl+1 through Ctrl+5 switches primary tabs while Lieve is focused.
- Escape closes a keyboard-opened panel and restores the previous application.
- Appearance -> System: fullscreen auto-hide and opt-in Start with Windows.
- Appearance -> System -> Spectrum: five live audio bars, sensitivity and colour.
- Appearance -> Glass: native frosted backdrop and transparency.

STARTUP
Start with Windows registers the current executable path for the current user.
After moving a portable folder, enable that setting again to update the path.
Setup itself does not enable or replace your startup choice.

UNINSTALL
Use Windows Installed apps / Apps & features, or Uninstall.exe in the install
folder. Uninstall removes program files and installer-owned shortcuts. It keeps
%APPDATA%\Lieve (settings, tasks, water totals and shelf references) and other
files you placed in the install folder. A startup entry is removed only when it
points to the executable being uninstalled; an entry for another copy is kept.

PRIVACY
Spectrum processing uses the Windows output mix locally in memory, never the
microphone. Title/artist/duration are sent to LRCLIB over HTTPS for lyric lookup.
There are no accounts or analytics. See the project README for current scope.

BUILD INFORMATION
build-info.json identifies the packaged executable, its SHA-256, imported DLLs,
source commit and working-tree state. The *-SHA256SUMS.txt next to the ZIP
and installer verifies their contents. These are unsigned alpha packages.
