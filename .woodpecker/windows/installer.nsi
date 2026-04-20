; email-rs Windows Installer — NSIS
;
; Build:
;   makensis -DAPP_VERSION=<ver> -DSTAGING=<dir> -DSCRIPTS=<dir> -DOUTPUT_DIR=<dir> installer.nsi
;
; Silent install:  email-rs-<ver>-installer.exe /S
; Silent uninstall: $INSTDIR\uninstall.exe /S
;
; Service: WinSW (email-rs-svc.exe) wraps email-server.exe. Config is written
; by setup-service.bat at install time. WinSW reads email-rs-svc.xml from $INSTDIR.

!ifndef APP_VERSION
  !define APP_VERSION "dev"
!endif
!ifndef STAGING
  !define STAGING "."
!endif
!ifndef SCRIPTS
  !define SCRIPTS "."
!endif
!ifndef OUTPUT_DIR
  !define OUTPUT_DIR "."
!endif

!define APP_NAME      "email-rs"
!define SERVICE_NAME  "email-rs"
!define PUBLISHER     "sprooty"
!define APP_PORT      "8585"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\email-rs"

Name          "${APP_NAME} ${APP_VERSION}"
OutFile       "${OUTPUT_DIR}${/}email-rs-${APP_VERSION}-installer.exe"
InstallDir    "$PROGRAMFILES64\email-rs"
RequestExecutionLevel admin
SetCompressor /SOLID lzma

!include "MUI2.nsh"
!define MUI_ABORTWARNING

!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

; ── Install ───────────────────────────────────────────────────────────────────

Section "email-rs" SecMain
  SectionIn RO

  ; Stop and uninstall existing service if present (upgrade path)
  IfFileExists "$INSTDIR\email-rs-svc.exe" 0 no_prev_install
    ExecWait '"$INSTDIR\uninstall-service.bat" "$INSTDIR\\"'
  no_prev_install:

  SetOutPath "$INSTDIR"
  File "${STAGING}${/}email-server.exe"
  File "${STAGING}${/}email-rs-svc.exe"
  File "${SCRIPTS}${/}setup-service.bat"
  File "${SCRIPTS}${/}uninstall-service.bat"

  SetOutPath "$INSTDIR\static"
  File /r "${STAGING}${/}static${/}*.*"

  ; Create data directory for SQLite DB (WinSW service workdir)
  CreateDirectory "$PROGRAMDATA\email-rs"

  ; Install and start the Windows service
  ExecWait '"$INSTDIR\setup-service.bat" "$INSTDIR\\" "$PROGRAMDATA\email-rs\\"'

  ; Add/Remove Programs
  WriteRegStr   HKLM "${UNINSTALL_KEY}" "DisplayName"     "${APP_NAME}"
  WriteRegStr   HKLM "${UNINSTALL_KEY}" "DisplayVersion"  "${APP_VERSION}"
  WriteRegStr   HKLM "${UNINSTALL_KEY}" "Publisher"       "${PUBLISHER}"
  WriteRegStr   HKLM "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr   HKLM "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr   HKLM "${UNINSTALL_KEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoRepair"  1

  ; Start Menu: URL shortcut opens web UI in default browser
  CreateDirectory "$SMPROGRAMS\email-rs"
  WriteIniStr "$SMPROGRAMS\email-rs\Open email-rs.url" "InternetShortcut" "URL" \
    "http://localhost:${APP_PORT}"
  CreateShortcut "$SMPROGRAMS\email-rs\Uninstall email-rs.lnk" "$INSTDIR\uninstall.exe"

  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

; ── Uninstall ─────────────────────────────────────────────────────────────────

Section "Uninstall"
  ExecWait '"$INSTDIR\uninstall-service.bat" "$INSTDIR\\"'

  RMDir /r "$INSTDIR"
  RMDir /r "$SMPROGRAMS\email-rs"
  DeleteRegKey HKLM "${UNINSTALL_KEY}"
  ; $PROGRAMDATA\email-rs (database) is intentionally kept to preserve user data.
SectionEnd
