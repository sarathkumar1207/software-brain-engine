param(
    [Parameter(Mandatory = $true)]
    [string]$ProjectPath,

    [string]$Query = "jwt to passport"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$ProjectFullPath = (Resolve-Path $ProjectPath).Path
$Binary = Join-Path $RepoRoot "target\release\sbe.exe"

Push-Location $RepoRoot
try {
    cargo build --release -p sbe-cli

    & $Binary validate $ProjectFullPath --query $Query
    & $Binary benchmark $ProjectFullPath --query $Query --json
}
finally {
    Pop-Location
}
