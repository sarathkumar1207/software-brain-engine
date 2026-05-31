param(
    [string]$Configuration = "release",
    [string]$Version = "0.1.0",
    [string]$OutputDir = "dist"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if ($Version -notmatch '^\d+\.\d+\.\d+(\.\d+)?$') {
    throw "MSI version must be numeric like 0.1.0 or 0.1.0.0. Received: $Version"
}

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$sbeExe = Join-Path $root "target\$Configuration\sbe.exe"
$wxs = Join-Path $root "packaging\windows\Product.wxs"
$outDir = Join-Path $root $OutputDir
$wixObj = Join-Path $outDir "sbe.wixobj"
$msi = Join-Path $outDir "sbe-$Version-windows-x64.msi"

if (-not (Get-Command candle.exe -ErrorAction SilentlyContinue)) {
    throw "WiX Toolset is required to build the MSI. Install WiX v3.x and ensure candle.exe is on PATH."
}

if (-not (Get-Command light.exe -ErrorAction SilentlyContinue)) {
    throw "WiX Toolset is required to build the MSI. Install WiX v3.x and ensure light.exe is on PATH."
}

cargo build --release -p sbe-cli

if (-not (Test-Path $sbeExe)) {
    throw "Release binary was not found at $sbeExe"
}

New-Item -ItemType Directory -Force -Path $outDir | Out-Null

& candle.exe `
    "-dProductVersion=$Version" `
    "-dSbeExePath=$sbeExe" `
    -out "$wixObj" `
    "$wxs"

if ($LASTEXITCODE -ne 0) {
    throw "candle.exe failed with exit code $LASTEXITCODE"
}

& light.exe `
    -ext WixUIExtension `
    -out "$msi" `
    "$wixObj"

if ($LASTEXITCODE -ne 0) {
    throw "light.exe failed with exit code $LASTEXITCODE"
}

Write-Host "MSI created: $msi"
