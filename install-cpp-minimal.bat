@echo off
chcp 65001 >nul
echo ============================================================
echo 正在安装 Rust / Tauri 2 所需的最小 C++ 编译环境
echo 仅安装: MSVC 编译器(link.exe) + Windows SDK (无额外冗余组件)
echo ============================================================

:: 检查管理员权限并自动提权
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo 请求管理员权限中，请在弹出的 UAC 窗口点击“是”...
    powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Process -FilePath '%~f0' -Verb RunAs"
    exit /b
)

cd /d "%~dp0"

if exist "vs_BuildTools.exe" (
    echo 正在安装，请稍候（会显示极简进度条）...
    .\vs_BuildTools.exe --productId Microsoft.VisualStudio.Product.BuildTools --channelId VisualStudio.17.Release --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.22621 --passive --wait --norestart --nocache
) else (
    echo 正在从微软官方下载引导程序...
    powershell -Command "Invoke-WebRequest -Uri https://aka.ms/vs/17/release/vs_BuildTools.exe -OutFile vs_BuildTools.exe"
    echo 正在安装...
    .\vs_BuildTools.exe --productId Microsoft.VisualStudio.Product.BuildTools --channelId VisualStudio.17.Release --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.22621 --passive --wait --norestart --nocache
)

echo.
echo ============================================================
echo 安装完成！您可以返回终端继续进行开发或编译测试。
echo ============================================================
pause
