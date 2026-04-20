@echo off
setlocal
:: Called by the MSI deferred custom action after files are installed.
:: %1 = INSTALLDIR (e.g. "C:\Program Files\email-rs\")
:: %2 = DATADIRROOT (e.g. "C:\ProgramData\email-rs\")
set "INSTALLDIR=%~1"
set "DATADIRROOT=%~2"

echo [email-rs] Writing WinSW service config...

:: Generate email-rs-svc.xml alongside the WinSW executable.
:: WinSW v2 reads <executable-name>.xml from the same directory.
(
  echo ^<?xml version="1.0" encoding="UTF-8"?^>
  echo ^<service^>
  echo   ^<id^>email-rs^</id^>
  echo   ^<name^>email-rs^</name^>
  echo   ^<description^>Self-hosted email and calendar client^</description^>
  echo   ^<executable^>%INSTALLDIR%email-server.exe^</executable^>
  echo   ^<workdir^>%DATADIRROOT%^</workdir^>
  echo   ^<env name="FRONTEND_DIST" value="%INSTALLDIR%static"/^>
  echo   ^<env name="PORT" value="8585"/^>
  echo   ^<env name="HOST" value="127.0.0.1"/^>
  echo   ^<startmode^>Automatic^</startmode^>
  echo   ^<logmode^>rotate^</logmode^>
  echo ^</service^>
) > "%INSTALLDIR%email-rs-svc.xml"

echo [email-rs] Installing and starting service...
"%INSTALLDIR%email-rs-svc.exe" install
"%INSTALLDIR%email-rs-svc.exe" start
echo [email-rs] Service installed and started.
endlocal
exit /b 0
