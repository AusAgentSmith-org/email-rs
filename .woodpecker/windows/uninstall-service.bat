@echo off
setlocal
:: Called by the MSI deferred custom action before files are removed.
:: %1 = INSTALLDIR
set "INSTALLDIR=%~1"

echo [email-rs] Stopping and removing service...
"%INSTALLDIR%email-rs-svc.exe" stop 2>nul
"%INSTALLDIR%email-rs-svc.exe" uninstall 2>nul
echo [email-rs] Service removed.
endlocal
exit /b 0
