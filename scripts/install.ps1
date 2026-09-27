# CUV (C-Ultra-Velocity) Universal Installer for Windows PowerShell
# Usage: powershell -c "irm https://raw.githubusercontent.com/ARASKOVA-labs/CUV/main/scripts/install.ps1 | iex"

$ErrorActionPreference = "Stop"

Write-Host "⚡ CUV (C-Ultra-Velocity) - Windows Installer" -ForegroundColor Cyan
Write-Host "The 'uv' and 'bun' for C and C++" -ForegroundColor DarkGray

$target = "x86_64-pc-windows-msvc"
$installDir = Join-Path $HOME ".cuv\bin"
$repo = "ARASKOVA-labs/CUV"
$tag = if ($env:CUV_VERSION) { $env:CUV_VERSION } else { "latest" }

if ($tag -eq "latest") {
    $url = "https://github.com/$repo/releases/latest/download/cuv-$target.tar.gz"
} else {
    $url = "https://github.com/$repo/releases/download/$tag/cuv-$target.tar.gz"
}

if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$tmpDir = Join-Path $env:TEMP ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

try {
    Write-Host "Downloading CUV archive from $url..." -ForegroundColor Yellow
    $archivePath = Join-Path $tmpDir "cuv.tar.gz"
    Invoke-WebRequest -Uri $url -OutFile $archivePath -UseBasicParsing

    tar -xzf $archivePath -C $tmpDir
    Copy-Item (Join-Path $tmpDir "cuv.exe") (Join-Path $installDir "cuv.exe") -Force

    Write-Host "✔ Successfully installed cuv.exe to $installDir" -ForegroundColor Green

    # Add to User PATH if not present
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$installDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$installDir", "User")
        Write-Host "Added $installDir to User PATH." -ForegroundColor Cyan
    }

    Write-Host ""
    Write-Host "Restart your terminal or run:" -ForegroundColor White
    Write-Host "  `$env:Path += `";$installDir`"" -ForegroundColor Yellow
    Write-Host "  cuv --help" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Welcome to ultra-velocity C/C++ development! ⚡" -ForegroundColor Green
} finally {
    Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
}
