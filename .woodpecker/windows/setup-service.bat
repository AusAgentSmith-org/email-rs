@echo off
setlocal
:: Called by the MSI deferred custom action after files are installed.
:: %1 = INSTALLDIR (e.g. C:\Program Files\email-rs\)
:: %2 = DATADIRROOT (e.g. C:\ProgramData\email-rs\)
set "INSTALLDIR=%~1"
set "DATADIRROOT=%~2"

echo [email-rs] Installing Windows service via NSSM...
"%INSTALLDIR%nssm.exe" install email-rs "%INSTALLDIR%email-server.exe"
"%INSTALLDIR%nssm.exe" set email-rs AppDirectory "%DATADIRROOT%"
"%INSTALLDIR%nssm.exe" set email-rs AppEnvironmentExtra "FRONTEND_DIST=%INSTALLDIR%static"
"%INSTALLDIR%nssm.exe" set email-rs AppEnvironmentExtra +PORT=8585
"%INSTALLDIR%nssm.exe" set email-rs AppEnvironmentExtra +HOST=127.0.0.1
"%INSTALLDIR%nssm.exe" set email-rs DisplayName "email-rs"
"%INSTALLDIR%nssm.exe" set email-rs Description "Self-hosted email and calendar client"
"%INSTALLDIR%nssm.exe" set email-rs Start SERVICE_AUTO_START
"%INSTALLDIR%nssm.exe" start email-rs
echo [email-rs] Service installed and started.
endlocal
exit /b 0
