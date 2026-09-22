$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = Split-Path -Parent $PSScriptRoot
$RustSourceRoot = Join-Path $RepoRoot "crates"
$CommandHelper = Join-Path $RustSourceRoot "bdl-core/src/process.rs"
$RawLaunchPattern = "Command::new\s*\("

$RuntimeSourceRoots = @(
    Get-ChildItem -Path $RustSourceRoot -Directory |
        ForEach-Object { Join-Path $_.FullName "src" } |
        Where-Object { Test-Path -LiteralPath $_ }
    Join-Path $RepoRoot "apps/desktop/src-tauri/src"
)

$rawLaunches = Get-ChildItem -Path $RuntimeSourceRoots -Recurse -File -Filter "*.rs" |
    Select-String -Pattern $RawLaunchPattern |
    Where-Object { $_.Path -ne $CommandHelper }

if ($rawLaunches) {
    $locations = $rawLaunches | ForEach-Object {
        "{0}:{1}" -f $_.Path, $_.LineNumber
    }
    throw "Runtime child processes bypass the hidden-window command helper:`n$($locations -join "`n")"
}

if (-not (Test-Path -LiteralPath $CommandHelper)) {
    throw "Missing hidden-window command helper: $CommandHelper"
}

$helperSource = Get-Content -LiteralPath $CommandHelper -Raw
if ($helperSource -notmatch "CREATE_NO_WINDOW" -or $helperSource -notmatch "\.creation_flags\s*\(") {
    throw "The Windows command helper must apply CREATE_NO_WINDOW via creation_flags."
}

Write-Output "Windows child-process policy verified: all runtime commands use CREATE_NO_WINDOW."
