<#
.SYNOPSIS
    Logzilla 一键编译脚本 (Windows / PowerShell)

.DESCRIPTION
    自动检测编译 Logzilla 所需的全部依赖，逐项报告缺失的依赖，
    询问用户是否自动安装（通过 winget），然后编译出 Windows .exe。

    检测的依赖：
      1. Node.js + npm      —— 前端构建 (Vite / React)
      2. Rust + Cargo       —— Tauri 后端 (rustc / cargo)
      3. MSVC C++ 生成工具   —— Rust MSVC 工具链所需的链接器 (link.exe)
      4. WebView2 运行时     —— Tauri 应用运行时 (Win11 通常自带)
      5. 项目 npm 依赖       —— node_modules (含本地 Tauri CLI)

.PARAMETER Yes
    对所有“是否自动安装 / 是否继续编译”的询问一律回答“是”（无人值守）。

.PARAMETER SkipDeps
    跳过依赖检测，直接编译（用于确定环境已就绪时）。

.PARAMETER Debug
    编译 debug 版本（默认编译 release 版本）。

.PARAMETER Arch
    目标架构：x86_64 或 i686（默认 i686，32 位）。

.EXAMPLE
    .\build.ps1
    交互式：检测依赖 → 缺失时询问是否安装 → 编译 release exe

.EXAMPLE
    .\build.ps1 -Yes
    无人值守：自动安装缺失依赖并编译
#>

[CmdletBinding()]
param(
    [switch]$Yes,
    [switch]$SkipDeps,
    [switch]$DebugBuild,
    [ValidateSet('x86_64','i686')][string]$Arch = 'i686'
)

$ErrorActionPreference = 'Stop'
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Definition
Set-Location $ProjectRoot

# ------------------------------------------------------------------ 输出辅助
function Write-Head($t) { Write-Host "`n=== $t ===" -ForegroundColor Cyan }
function Write-Ok($t)   { Write-Host "  [OK]   $t" -ForegroundColor Green }
function Write-Miss($t) { Write-Host "  [缺失] $t" -ForegroundColor Yellow }
function Write-Err($t)  { Write-Host "  [错误] $t" -ForegroundColor Red }
function Write-Info($t) { Write-Host "  $t" -ForegroundColor Gray }

function Confirm-Action($question) {
    if ($Yes) { return $true }
    $ans = Read-Host "$question [Y/n]"
    return ($ans -eq '' -or $ans -match '^[Yy]')
}

# winget 安装完成后，当前进程的 PATH 不会自动刷新 —— 从注册表重新拼接。
function Update-EnvPath {
    $machine = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $user    = [Environment]::GetEnvironmentVariable('Path', 'User')
    $env:Path = ($machine, $user | Where-Object { $_ } ) -join ';'
}

function Test-Cmd($name) {
    return [bool](Get-Command $name -ErrorAction SilentlyContinue)
}

function Ensure-Winget {
    if (Test-Cmd winget) { return $true }
    Write-Err "未找到 winget（Windows 程序包管理器），无法自动安装依赖。"
    Write-Info "请手动安装依赖，或从 Microsoft Store 安装“应用安装程序”后重试。"
    return $false
}

# 通过 winget 安装一个包，成功后刷新 PATH
function Install-Pkg($id, $extraArgs) {
    if (-not (Ensure-Winget)) { return $false }
    Write-Info "正在安装 $id ..."
    $args = @('install','--id',$id,'-e','--source','winget',
              '--accept-package-agreements','--accept-source-agreements')
    if ($extraArgs) { $args += $extraArgs }
    & winget @args
    $code = $LASTEXITCODE
    Update-EnvPath
    if ($code -ne 0) {
        Write-Err "$id 安装未成功（winget 退出码 $code）。可能需要手动安装。"
        return $false
    }
    Write-Ok "$id 安装完成。"
    return $true
}

# ------------------------------------------------------------------ 依赖检测器
# vswhere 检测 MSVC C++ 生成工具（Rust 的 MSVC 工具链依赖 link.exe）
function Test-MsvcTools {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path $vswhere)) { return $false }
    $path = & $vswhere -latest -products * `
        -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
        -property installationPath 2>$null
    return [bool]$path
}

# WebView2 运行时：检测 EdgeUpdate 注册表中的版本号（pv）
function Test-WebView2 {
    $guid = '{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
    $keys = @(
        "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\$guid",
        "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\$guid",
        "HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients\$guid"
    )
    foreach ($k in $keys) {
        try {
            $pv = (Get-ItemProperty -Path $k -Name pv -ErrorAction Stop).pv
            if ($pv -and $pv -ne '0.0.0.0') { return $true }
        } catch {}
    }
    return $false
}

# 每个依赖：名称 / 检测 / winget 包 id / 额外参数 / 是否致命
$Deps = @(
    @{ Name='Node.js + npm'; Check={ (Test-Cmd node) -and (Test-Cmd npm) };
       Id='OpenJS.NodeJS.LTS'; Args=$null; Fatal=$true;
       Hint='前端构建 (Vite/React)' }

    @{ Name='Rust + Cargo'; Check={ (Test-Cmd rustc) -and (Test-Cmd cargo) };
       Id='Rustlang.Rustup'; Args=$null; Fatal=$true;
       Hint='Tauri 后端编译' }

    @{ Name='MSVC C++ 生成工具'; Check={ Test-MsvcTools };
       Id='Microsoft.VisualStudio.2022.BuildTools';
       Args=@('--override','--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended');
       Fatal=$true; Hint='Rust MSVC 工具链所需链接器 link.exe（安装体积较大，约数 GB）' }

    @{ Name='WebView2 运行时'; Check={ Test-WebView2 };
       Id='Microsoft.EdgeWebView2Runtime'; Args=$null; Fatal=$false;
       Hint='应用运行时（Win11 通常自带；缺失不影响编译，但影响运行）' }
)

# ------------------------------------------------------------------ 主流程
Write-Host ""
Write-Host "  Logzilla 编译脚本" -ForegroundColor Magenta
Write-Host "  项目目录: $ProjectRoot" -ForegroundColor DarkGray

if (-not $SkipDeps) {
    Write-Head "1/3 检测编译依赖"

    $missing = @()
    foreach ($d in $Deps) {
        if (& $d.Check) {
            Write-Ok "$($d.Name)"
        } else {
            Write-Miss "$($d.Name)  —— $($d.Hint)"
            $missing += $d
        }
    }

    # 项目 npm 依赖（node_modules，含本地 Tauri CLI）—— 依赖 npm 存在
    $nodeModulesMissing = -not (Test-Path (Join-Path $ProjectRoot 'node_modules'))
    if ($nodeModulesMissing) { Write-Miss "项目 npm 依赖 (node_modules) —— 尚未安装" }
    else                     { Write-Ok  "项目 npm 依赖 (node_modules)" }

    if ($missing.Count -gt 0) {
        Write-Head "检测到 $($missing.Count) 项缺失依赖"
        foreach ($d in $missing) {
            $tag = if ($d.Fatal) { '必需' } else { '可选' }
            Write-Host "   • [$tag] $($d.Name) —— $($d.Hint)" -ForegroundColor Yellow
        }

        if (Confirm-Action "是否尝试用 winget 自动安装以上缺失依赖？") {
            foreach ($d in $missing) {
                Write-Head "安装：$($d.Name)"
                $installed = Install-Pkg $d.Id $d.Args
                if (-not $installed -and $d.Fatal) {
                    Write-Err "必需依赖 $($d.Name) 未安装成功，无法继续编译。"
                    Write-Info "请手动安装后重试，或参考 README 的环境搭建说明。"
                    exit 1
                }
            }
            # 重新检测必需依赖是否已就绪
            Update-EnvPath
            $stillMissing = $Deps | Where-Object { $_.Fatal -and -not (& $_.Check) }
            if ($stillMissing) {
                Write-Err "以下必需依赖安装后仍不可用（可能需重开终端刷新 PATH）："
                $stillMissing | ForEach-Object { Write-Info " - $($_.Name)" }
                Write-Info "请关闭并重新打开 PowerShell，再次运行本脚本。"
                exit 1
            }
        } else {
            $fatalMissing = $missing | Where-Object { $_.Fatal }
            if ($fatalMissing) {
                Write-Err "存在未安装的必需依赖，无法编译。已退出。"
                exit 1
            }
            Write-Info "仅缺失可选依赖，继续编译。"
        }
    } else {
        Write-Ok "全部依赖已就绪。"
    }

    # 安装项目 npm 依赖
    if ($nodeModulesMissing) {
        Write-Head "安装项目 npm 依赖 (npm install)"
        npm install
        if ($LASTEXITCODE -ne 0) { Write-Err "npm install 失败。"; exit 1 }
        Write-Ok "npm 依赖安装完成。"
    }
} else {
    Write-Info "已跳过依赖检测 (-SkipDeps)。"
}

# ------------------------------------------------------------------ 编译
Write-Head "2/3 编译应用"
if (-not (Confirm-Action "开始编译 Logzilla$(if($DebugBuild){' (debug)'}else{' (release)'})？该过程可能需要数分钟")) {
    Write-Info "已取消。"
    exit 0
}

$target = if ($Arch -eq 'i686') { 'i686-pc-windows-msvc' } else { 'x86_64-pc-windows-msvc' }
$profile = if ($DebugBuild) { 'debug' } else { 'release' }
$cargoFlag = if ($DebugBuild) { '' } else { '--release' }

$sw = [System.Diagnostics.Stopwatch]::StartNew()

# 步骤 1：构建前端
Write-Head "2a/3 构建前端 (Vite + TypeScript)"
Write-Info "执行: npm run build"
& npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Err "前端构建失败（退出码 $LASTEXITCODE）。"
    exit $LASTEXITCODE
}
Write-Ok "前端构建完成"

# 步骤 2：编译 Rust 后端（跳过 Tauri CLI 的打包步骤，避免下载 NSIS/WiX 超时）
Write-Head "2b/3 编译后端 (Cargo)"
$cargoArgs = @('build', '--target', $target, '--manifest-path', (Join-Path $ProjectRoot 'src-tauri\Cargo.toml'))
if ($cargoFlag) { $cargoArgs += $cargoFlag }
Write-Info "执行: cargo $($cargoArgs -join ' ')"
& cargo @cargoArgs
$rustCode = $LASTEXITCODE
$sw.Stop()

if ($rustCode -ne 0) {
    Write-Err "Rust 编译失败（退出码 $rustCode），用时 $([int]$sw.Elapsed.TotalSeconds)s。"
    Write-Info "请检查上方 cargo 的错误输出。"
    exit $rustCode
}

# ------------------------------------------------------------------ 产物
Write-Head "3/3 编译完成"
Write-Ok "用时 $([int]$sw.Elapsed.TotalSeconds)s"

$exe = Join-Path $ProjectRoot "src-tauri\target\$target\$profile\logzilla.exe"
if (Test-Path $exe) {
    Write-Host "`n  可执行文件：" -ForegroundColor Green
    Write-Host "    $exe" -ForegroundColor White
}

Write-Host ""
Write-Ok "全部完成 ✔"
Write-Host ""
