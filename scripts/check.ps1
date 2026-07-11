param(
    [switch]$SkipClippy
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = Split-Path -Parent $PSScriptRoot

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

Push-Location $RepoRoot
try {
    Invoke-Step "Rust format" {
        cargo fmt --all --check
    }

    if (-not $SkipClippy) {
        Invoke-Step "Rust clippy" {
            cargo clippy --workspace --all-targets -- -D warnings
        }
    }

    Invoke-Step "Rust tests" {
        cargo test --workspace
    }

    Invoke-Step "Desktop frontend build" {
        pnpm --dir apps/desktop build
    }

    Invoke-Step "Desktop code lint" {
        pnpm --dir apps/desktop lint
    }

    Invoke-Step "Desktop style lint" {
        pnpm --dir apps/desktop lint:styles
    }

    Invoke-Step "Desktop style format" {
        pnpm --dir apps/desktop format:styles:check
    }

    Invoke-Step "Desktop frontend tests" {
        pnpm --dir apps/desktop test
    }

    Invoke-Step "Desktop visual regression tests" {
        pnpm --dir apps/desktop test:visual
    }

    Invoke-Step "Git whitespace check" {
        git diff --check
    }

    Write-Host ""
    Write-Host "All checks passed."
}
finally {
    Pop-Location
}
