@echo off
setlocal
:: Called by the MSI deferred custom action before files are removed.
:: %1 = INSTALLDIR
set "INSTALLDIR=%~1"

echo [email-rs] Stopping and removing service...
"%INSTALLDIR%nssm.exe" stop email-rs 2>nul
"%INSTALLDIR%nssm.exe" remove email-rs confirm 2>nul
echo [email-rs] Service removed.
endlocal
exit /b 0
