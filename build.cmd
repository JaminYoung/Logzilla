@echo off
REM Logzilla 编译启动器 —— 双击即可运行，绕过 PowerShell 执行策略限制。
REM 透传参数示例:  build.cmd -Yes   /   build.cmd -Debug
setlocal
set "SCRIPT_DIR=%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%build.ps1" %*
set "RC=%ERRORLEVEL%"
echo.
if not "%RC%"=="0" (
  echo [失败] 编译脚本退出码 %RC%
)
echo 按任意键关闭窗口...
pause >nul
endlocal
exit /b %RC%
