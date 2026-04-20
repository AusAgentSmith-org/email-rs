; email-rs Windows Installer
;
; Invoked from CI:
;   makensis -DAPP_VERSION=<ver> -DSTAGING=<dir> -DOUTPUT_DIR=<dir> installer.nsi
;
; What this installer does:
;   - Copies email-server.exe + static frontend files to Program Files
;   - Installs email-rs as an auto-start Windows service via NSSM
;   - Stores the SQLite database in C:\ProgramData\email-rs\ (service working dir)
;   - Adds a Start Menu shortcut that opens http://localhost:8585
;   - Registers with Add/Remove Programs

!ifndef APP_VERSION
  !define APP_VERSION "dev"
!endif
!ifndef STAGING
  !define STAGING "."
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

  ; Gracefully stop existing service if present (ignore errors on fresh install)
  ClearErrors
  ExecWait '"$INSTDIR\nssm.exe" stop ${SERVICE_NAME}'

  SetOutPath "$INSTDIR"
  File "${STAGING}${/}email-server.exe"
  File "${STAGING}${/}nssm.exe"

  SetOutPath "$INSTDIR\static"
  File /r "${STAGING}${/}static${/}*"

  ; Data directory — service runs as LocalSystem, so use ProgramData (not AppData)
  CreateDirectory "$PROGRAMDATA\email-rs"

  ; Register as a Windows service via NSSM (wraps the exe for SCM compatibility)
  ExecWait '"$INSTDIR\nssm.exe" install ${SERVICE_NAME} "$INSTDIR\email-server.exe"'
  ; AppDirectory becomes the working dir — SQLite default sqlite://email.db lands here
  ExecWait '"$INSTDIR\nssm.exe" set ${SERVICE_NAME} AppDirectory "$PROGRAMDATA\email-rs"'
  ; Inject runtime env vars expected by config.rs
  ExecWait '"$INSTDIR\nssm.exe" set ${SERVICE_NAME} AppEnvironmentExtra "FRONTEND_DIST=$INSTDIR\static"'
  ExecWait '"$INSTDIR\nssm.exe" set ${SERVICE_NAME} AppEnvironmentExtra +PORT=${APP_PORT}'
  ExecWait '"$INSTDIR\nssm.exe" set ${SERVICE_NAME} AppEnvironmentExtra +HOST=127.0.0.1'
  ExecWait '"$INSTDIR\nssm.exe" set ${SERVICE_NAME} DisplayName "${APP_NAME}"'
  ExecWait '"$INSTDIR\nssm.exe" set ${SERVICE_NAME} Description "Self-hosted email and calendar client"'
  ExecWait '"$INSTDIR\nssm.exe" set ${SERVICE_NAME} Start SERVICE_AUTO_START'
  ExecWait '"$INSTDIR\nssm.exe" start ${SERVICE_NAME}'

  ; Add/Remove Programs registration
  WriteRegStr   HKLM "${UNINSTALL_KEY}" "DisplayName"     "${APP_NAME}"
  WriteRegStr   HKLM "${UNINSTALL_KEY}" "DisplayVersion"  "${APP_VERSION}"
  WriteRegStr   HKLM "${UNINSTALL_KEY}" "Publisher"       "${PUBLISHER}"
  WriteRegStr   HKLM "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr   HKLM "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoRepair"  1

  ; Start Menu: URL shortcut opens the web UI in the default browser
  CreateDirectory "$SMPROGRAMS\email-rs"
  WriteIniStr "$SMPROGRAMS\email-rs\Open email-rs.url" "InternetShortcut" "URL" \
    "http://localhost:${APP_PORT}"
  CreateShortcut "$SMPROGRAMS\email-rs\Uninstall email-rs.lnk" "$INSTDIR\uninstall.exe"

  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

; ── Uninstall ─────────────────────────────────────────────────────────────────

Section "Uninstall"
  ExecWait '"$INSTDIR\nssm.exe" stop ${SERVICE_NAME}'
  ExecWait '"$INSTDIR\nssm.exe" remove ${SERVICE_NAME} confirm'

  RMDir /r "$INSTDIR"
  RMDir /r "$SMPROGRAMS\email-rs"
  DeleteRegKey HKLM "${UNINSTALL_KEY}"
  ; Note: $PROGRAMDATA\email-rs (database + config) is intentionally kept
  ;       on uninstall to preserve user data.
SectionEnd
