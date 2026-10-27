; Standard NSIS 3 Unicode script. Compile through scripts/package-windows.ps1.
Unicode true
RequestExecutionLevel user
ManifestDPIAware true
SetCompressor /SOLID lzma
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "WinVer.nsh"
!include "WinMessages.nsh"

!ifndef APP_VERSION
 !error "APP_VERSION is required"
!endif
!ifndef PAYLOAD_DIR
 !error "PAYLOAD_DIR is required"
!endif
!ifndef OUTPUT_FILE
 !error "OUTPUT_FILE is required"
!endif
!ifdef TEST_PACKAGE
 !define PRODUCT_ID "LievePackagingSmoke"
 !define PRODUCT_NAME "Lieve (packaging test)"
 !define STARTUP_KEY "Software\Lieve\PackagingSmoke\Run"
!else
 !define PRODUCT_ID "Lieve"
 !define PRODUCT_NAME "Lieve"
 !define STARTUP_KEY "Software\Microsoft\Windows\CurrentVersion\Run"
!endif
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_ID}"
Name "${PRODUCT_NAME} ${APP_VERSION}"
OutFile "${OUTPUT_FILE}"
InstallDir "$LOCALAPPDATA\Programs\${PRODUCT_ID}"
InstallDirRegKey HKCU "${UNINSTALL_KEY}" "InstallLocation"
BrandingText "Lieve - native Windows alpha"
VIProductVersion "${APP_VERSION}.0"
VIAddVersionKey /LANG=1033 "ProductName" "${PRODUCT_NAME}"
VIAddVersionKey /LANG=1033 "FileDescription" "Lieve per-user setup"
VIAddVersionKey /LANG=1033 "FileVersion" "${APP_VERSION}"
VIAddVersionKey /LANG=1033 "LegalCopyright" "Lieve contributors"
ShowInstDetails show
ShowUninstDetails show

!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TEXT "Install Lieve for your current Windows account.$\r$\n$\r$\nSettings and personal data are kept separately in AppData and preserved during updates and uninstall."
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!define MUI_PAGE_CUSTOMFUNCTION_PRE DirectoryPre
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\lieve.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch Lieve"
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH
!insertmacro MUI_LANGUAGE "English"

!macro StopInstalled PREFIX
Function ${PREFIX}StopInstalled
 ; Never close another portable/development copy sharing the same window class.
 FindWindow $0 "LieveNativeIsland"
 StrCmp $0 0 done
 System::Call 'user32::GetWindowThreadProcessId(p r0, *i .r1)'
 System::Call 'kernel32::OpenProcess(i 0x101000, i 0, i r1) p.r2'
 StrCmp $2 0 done
 StrCpy $3 ${NSIS_MAX_STRLEN}
 System::Call 'kernel32::QueryFullProcessImageNameW(p r2, i 0, w .r4, *i r3) i.r5'
 StrCmp $5 0 close_handle
 StrCmp $4 "$INSTDIR\lieve.exe" 0 close_handle
 System::Call 'user32::IsWindowEnabled(p r0) i.r5'
 StrCmp $5 0 blocked
 SendMessage $0 ${WM_CLOSE} 0 0 /TIMEOUT=3000
 System::Call 'kernel32::WaitForSingleObject(p r2, i 3000) i.r5'
 StrCmp $5 0 close_handle
 blocked:
 System::Call 'kernel32::CloseHandle(p r2)'
 MessageBox MB_OK|MB_ICONEXCLAMATION "Close Lieve and its open dialogs before continuing." /SD IDOK
 SetErrorLevel 2
 Abort
 close_handle:
 System::Call 'kernel32::CloseHandle(p r2)'
 done:
FunctionEnd
!macroend
!insertmacro StopInstalled ""
!insertmacro StopInstalled "un."

Function .onInit
 SetShellVarContext current
 SetRegView 64
 ${IfNot} ${RunningX64}
  MessageBox MB_OK|MB_ICONSTOP "Lieve requires Windows x64." /SD IDOK
  SetErrorLevel 2
  Abort
 ${EndIf}
 ${IfNot} ${AtLeastWin10}
  MessageBox MB_OK|MB_ICONSTOP "Lieve requires Windows 10 (1809+) or Windows 11." /SD IDOK
  SetErrorLevel 2
  Abort
 ${EndIf}
 ${If} ${AtMostWin10}
  ${WinVerGetBuild} $0
  IntCmp $0 17763 supported unsupported supported
  unsupported:
  MessageBox MB_OK|MB_ICONSTOP "Lieve requires Windows 10 version 1809 or newer." /SD IDOK
  SetErrorLevel 2
  Abort
  supported:
 ${EndIf}
 ; Read explicitly after SetRegView so both x86 and x64 compilers find upgrades.
 ReadRegStr $0 HKCU "${UNINSTALL_KEY}" "InstallLocation"
 StrCmp $0 "" no_previous_install
 StrCpy $INSTDIR $0
 no_previous_install:
FunctionEnd
Function DirectoryPre
 ReadRegStr $0 HKCU "${UNINSTALL_KEY}" "InstallLocation"
 StrCmp $0 "" done
 ; Updates stay in the registered directory, avoiding an orphaned older copy.
 Abort
 done:
FunctionEnd

Section "Lieve (required)" MainSection
 SectionIn RO
 ReadRegStr $0 HKCU "${UNINSTALL_KEY}" "InstallLocation"
 StrCmp $0 "" directory_ok
 StrCmp $0 $INSTDIR directory_ok
 MessageBox MB_OK|MB_ICONEXCLAMATION "Update Lieve in its existing installation folder, or uninstall it before changing folders." /SD IDOK
 SetErrorLevel 2
 Abort
 directory_ok:
 Call StopInstalled
 SetOutPath "$INSTDIR"
 File "${PAYLOAD_DIR}\lieve.exe"
 File "${PAYLOAD_DIR}\README.txt"
 File "${PAYLOAD_DIR}\build-info.json"
 WriteUninstaller "$INSTDIR\Uninstall.exe"
 CreateDirectory "$SMPROGRAMS\${PRODUCT_NAME}"
 CreateShortcut "$SMPROGRAMS\${PRODUCT_NAME}\Lieve.lnk" "$INSTDIR\lieve.exe"
 CreateShortcut "$SMPROGRAMS\${PRODUCT_NAME}\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
 WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${PRODUCT_NAME}"
 WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${APP_VERSION}"
 WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "Lieve contributors"
 WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
 WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\lieve.exe"
 WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
 WriteRegStr HKCU "${UNINSTALL_KEY}" "QuietUninstallString" '$\"$INSTDIR\Uninstall.exe$\" /S'
 WriteRegStr HKCU "${UNINSTALL_KEY}" "URLInfoAbout" "https://github.com/zenarl-frl/Lieve"
 WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
 WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
SectionEnd
Section /o "Desktop shortcut" DesktopSection
 CreateShortcut "$DESKTOP\${PRODUCT_NAME}.lnk" "$INSTDIR\lieve.exe"
 WriteRegDWORD HKCU "${UNINSTALL_KEY}" "DesktopShortcut" 1
SectionEnd

Function un.onInit
 SetShellVarContext current
 SetRegView 64
 ReadRegStr $0 HKCU "${UNINSTALL_KEY}" "InstallLocation"
 StrCmp $0 $INSTDIR own_installation
 MessageBox MB_OK|MB_ICONSTOP "Run the uninstaller from its registered Lieve installation." /SD IDOK
 SetErrorLevel 2
 Abort
 own_installation:
FunctionEnd
Section "Uninstall"
 Call un.StopInstalled
 ReadRegStr $0 HKCU "${STARTUP_KEY}" "Lieve"
 StrCmp $0 '$\"$INSTDIR\lieve.exe$\"' remove_startup
 StrCmp $0 "$INSTDIR\lieve.exe" remove_startup keep_startup
 remove_startup:
 DeleteRegValue HKCU "${STARTUP_KEY}" "Lieve"
 keep_startup:
 ReadRegDWORD $0 HKCU "${UNINSTALL_KEY}" "DesktopShortcut"
 StrCmp $0 1 0 no_desktop
 Delete "$DESKTOP\${PRODUCT_NAME}.lnk"
 no_desktop:
 Delete "$SMPROGRAMS\${PRODUCT_NAME}\Lieve.lnk"
 Delete "$SMPROGRAMS\${PRODUCT_NAME}\Uninstall.lnk"
 RMDir "$SMPROGRAMS\${PRODUCT_NAME}"
 Delete "$INSTDIR\lieve.exe"
 Delete "$INSTDIR\README.txt"
 Delete "$INSTDIR\build-info.json"
 Delete "$INSTDIR\Uninstall.exe"
 ; Non-recursive removal deliberately preserves unknown files and AppData.
 RMDir "$INSTDIR"
 DeleteRegKey HKCU "${UNINSTALL_KEY}"
SectionEnd
