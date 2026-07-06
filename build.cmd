@echo off
REM Logzilla ���������� ���� ˫���������У��ƹ� PowerShell ִ�в������ơ�
REM Usage:  build.cmd [-Yes] [-DebugBuild] [-Arch (i686|x86_64)]
REM ������Ĭ�ϱ��� 32 λ (i686) �������� Arch=x86_64 ���� 64 λ��
setlocal
set "SCRIPT_DIR=%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%build.ps1" %*
set "RC=%ERRORLEVEL%"
echo.
if not "%RC%"=="0" (
  echo [ʧ��] ����ű��˳��� %RC%
)
echo ��������رմ���...
pause >nul
endlocal
exit /b %RC%
