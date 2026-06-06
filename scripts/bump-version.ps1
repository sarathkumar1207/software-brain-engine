param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$Version
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

function Update-File {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [Parameter(Mandatory = $true)]
        [scriptblock]$Update
    )

    $FullPath = Join-Path $RepoRoot $Path
    if (-not (Test-Path $FullPath)) {
        return
    }

    $Content = Get-Content $FullPath -Raw
    $Updated = & $Update $Content
    if ($Updated -ne $Content) {
        Set-Content -Path $FullPath -Value $Updated -NoNewline
    }
}

Get-ChildItem -Path (Join-Path $RepoRoot "crates") -Filter Cargo.toml -Recurse | ForEach-Object {
    $Content = Get-Content $_.FullName -Raw
    $Updated = $Content -replace '(?m)^version = "\d+\.\d+\.\d+"', "version = `"$Version`""
    if ($Updated -ne $Content) {
        Set-Content -Path $_.FullName -Value $Updated -NoNewline
    }
}

Update-File "scripts/build-msi.ps1" {
    param($Content)
    $Content `
        -replace '\[string\]\$Version = "\d+\.\d+\.\d+"', "[string]`$Version = `"$Version`"" `
        -replace 'MSI version must be numeric like \d+\.\d+\.\d+ or \d+\.\d+\.\d+\.0', "MSI version must be numeric like $Version or $Version.0"
}

Update-File ".github/workflows/release.yml" {
    param($Content)
    $Lines = $Content -split "\r?\n"
    $UpdatedLines = foreach ($Line in $Lines) {
        if ($Line.Contains('$version -notmatch')) {
            "          if (`$version -notmatch '^\d+\.\d+\.\d+$') { `$version = `"$Version`" }"
        } else {
            $Line
        }
    }
    $UpdatedLines -join "`n"
}

$VersionedFiles = @(
    "README.md",
    "docs/install.md",
    "website/index.html"
)

foreach ($File in $VersionedFiles) {
    Update-File $File {
        param($Content)
        $Content `
            -replace 'sbe-\d+\.\d+\.\d+-windows-x64\.msi', "sbe-$Version-windows-x64.msi" `
            -replace 'badge/version-\d+\.\d+\.\d+-blue\.svg', "badge/version-$Version-blue.svg" `
            -replace 'git tag v\d+\.\d+\.\d+', "git tag v$Version" `
            -replace 'git push origin v\d+\.\d+\.\d+', "git push origin v$Version" `
            -replace 'build-msi\.ps1 -Version \d+\.\d+\.\d+', "build-msi.ps1 -Version $Version" `
            -replace '`v\d+\.\d+\.\d+`', "``v$Version``" `
            -replace '`sbe-\d+\.\d+\.\d+-windows-x64\.msi`', "``sbe-$Version-windows-x64.msi``"
    }
}

cargo update --workspace

Write-Host "Updated SBE version to $Version"
