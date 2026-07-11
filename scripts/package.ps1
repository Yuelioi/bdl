param(
    [switch]$SkipCheck
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = Split-Path -Parent $PSScriptRoot
$DesktopDir = Join-Path $RepoRoot "apps/desktop"
$TargetDir = Join-Path $RepoRoot "target/release"
$BundleDir = Join-Path $TargetDir "bundle"

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,
        [Parameter(Mandatory = $true)]
        [scriptblock]$Command
    )

    Write-Host ""
    Write-Host "==> $Name"
    $global:LASTEXITCODE = 0
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE."
    }
}

function Get-PackageArtifacts {
    $artifacts = @()

    if (Test-Path $TargetDir) {
        $artifacts += Get-ChildItem -Path $TargetDir -File -Filter "*.exe"
    }

    if (Test-Path $BundleDir) {
        $artifacts += Get-ChildItem -Path $BundleDir -Recurse -File | Where-Object {
            $_.Extension -in @(".exe", ".msi", ".msix", ".zip", ".dmg", ".deb", ".rpm", ".AppImage") -or
            $_.Name.EndsWith(".app.tar.gz")
        }
    }

    $artifacts | Sort-Object FullName -Unique
}

Push-Location $RepoRoot
try {
    if (-not $SkipCheck) {
        Invoke-Step "Workspace checks" {
            & (Join-Path $PSScriptRoot "check.ps1")
        }
    }

    Invoke-Step "Tauri package" {
        pnpm --dir $DesktopDir tauri build
    }

    Invoke-Step "Windows GUI subsystem" {
        & (Join-Path $PSScriptRoot "verify-windows-gui.ps1") (Join-Path $TargetDir "bdl-desktop.exe")
    }

    $artifacts = @(Get-PackageArtifacts)
    if ($artifacts.Count -eq 0) {
        throw "Tauri build completed but no package artifacts were found under $TargetDir."
    }

    Write-Host ""
    Write-Host "Artifacts:"
    foreach ($artifact in $artifacts) {
        $sizeMb = [Math]::Round($artifact.Length / 1MB, 2)
        Write-Host ("- {0} ({1} MB)" -f $artifact.FullName, $sizeMb)
    }
}
finally {
    Pop-Location
}
