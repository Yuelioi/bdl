param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateNotNullOrEmpty()]
    [string]$Version
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = Split-Path -Parent $PSScriptRoot
$VersionPattern = '^\d+\.\d+\.\d+$'
$ManifestPaths = @(
    "crates/bdl-core/Cargo.toml",
    "crates/bdl-tauri/Cargo.toml",
    "crates/bdl-mobile-credentials/Cargo.toml",
    "crates/bdl-mobile-execution/Cargo.toml",
    "crates/bdl-mobile-media/Cargo.toml",
    "crates/bdl-mobile-storage/Cargo.toml",
    "crates/bdl-cli/Cargo.toml",
    "apps/desktop/src-tauri/Cargo.toml"
)

function Read-Utf8File {
    param([Parameter(Mandatory = $true)][string]$Path)
    return [System.IO.File]::ReadAllText($Path)
}

function Write-Utf8File {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Content
    )
    [System.IO.File]::WriteAllText($Path, $Content, [System.Text.UTF8Encoding]::new($false))
}

function Get-ManifestVersion {
    param([Parameter(Mandatory = $true)][string]$Path)
    $content = Read-Utf8File $Path
    $match = [regex]::Match($content, '(?m)^version = "(?<version>\d+\.\d+\.\d+)"')
    if (-not $match.Success) {
        throw "No package version found in $Path."
    }
    return $match.Groups['version'].Value
}

function Resolve-NextVersion {
    param(
        [Parameter(Mandatory = $true)][string]$Current,
        [Parameter(Mandatory = $true)][string]$Requested
    )
    $parts = @($Current.Split('.') | ForEach-Object { [int]$_ })
    switch ($Requested.ToLowerInvariant()) {
        'major' { return "{0}.0.0" -f ($parts[0] + 1) }
        'minor' { return "{0}.{1}.0" -f $parts[0], ($parts[1] + 1) }
        'patch' { return "{0}.{1}.{2}" -f $parts[0], $parts[1], ($parts[2] + 1) }
        default {
            if ($Requested -notmatch $VersionPattern) {
                throw "Version must be patch, minor, major, or an exact x.y.z version."
            }
            return $Requested
        }
    }
}

function Replace-FirstVersion {
    param(
        [Parameter(Mandatory = $true)][string]$RelativePath,
        [Parameter(Mandatory = $true)][string]$NextVersion
    )
    $path = Join-Path $RepoRoot $RelativePath
    $content = Read-Utf8File $path
    $updated = [regex]::Replace(
        $content,
        '(?m)^version = "\d+\.\d+\.\d+"',
        "version = `"$NextVersion`"",
        1
    )
    if ($updated -eq $content) {
        throw "Failed to update $RelativePath."
    }
    Write-Utf8File $path $updated
}

$manifestVersions = @{}
foreach ($relativePath in $ManifestPaths) {
    $manifestVersions[$relativePath] = Get-ManifestVersion (Join-Path $RepoRoot $relativePath)
}

$packagePath = Join-Path $RepoRoot "apps/desktop/package.json"
$tauriConfigPath = Join-Path $RepoRoot "apps/desktop/src-tauri/tauri.conf.json"
$packageVersion = (Get-Content -Raw $packagePath | ConvertFrom-Json).version
$tauriVersion = (Get-Content -Raw $tauriConfigPath | ConvertFrom-Json).version
$versions = @($manifestVersions.Values) + @($packageVersion, $tauriVersion)
$currentVersion = $versions[0]
if (@($versions | Where-Object { $_ -ne $currentVersion }).Count -gt 0) {
    throw "Project versions are inconsistent: $($versions -join ', '). Fix them before bumping."
}

$nextVersion = Resolve-NextVersion $currentVersion $Version
if ($nextVersion -eq $currentVersion) {
    throw "The requested version is already $currentVersion."
}

foreach ($relativePath in $ManifestPaths) {
    Replace-FirstVersion $relativePath $nextVersion
}

$jsonFiles = @($packagePath, $tauriConfigPath)
foreach ($path in $jsonFiles) {
    $content = Read-Utf8File $path
    $updated = [regex]::Replace(
        $content,
        '(?m)^(\s*"version"\s*:\s*)"\d+\.\d+\.\d+"',
        "`${1}`"$nextVersion`"",
        1
    )
    if ($updated -eq $content) {
        throw "Failed to update $path."
    }
    Write-Utf8File $path $updated
}

$readmePath = Join-Path $RepoRoot "README.md"
$readme = Read-Utf8File $readmePath
$readme = [regex]::Replace($readme, '当前版本：`\d+\.\d+\.\d+`', "当前版本：``$nextVersion``", 1)
Write-Utf8File $readmePath $readme

$lockPath = Join-Path $RepoRoot "Cargo.lock"
$lock = Read-Utf8File $lockPath
foreach ($packageName in @(
    'bdl-core',
    'bdl-tauri',
    'bdl-mobile-credentials',
    'bdl-mobile-execution',
    'bdl-mobile-media',
    'bdl-mobile-storage',
    'bdl-cli',
    'bdl-desktop'
)) {
    $pattern = "(?ms)(\[\[package\]\]\r?\nname = `"$([regex]::Escape($packageName))`"\r?\nversion = `")\d+\.\d+\.\d+(`")"
    $updated = [regex]::Replace($lock, $pattern, "`${1}$nextVersion`${2}", 1)
    if ($updated -eq $lock) {
        throw "Failed to update $packageName in Cargo.lock."
    }
    $lock = $updated
}
Write-Utf8File $lockPath $lock

Write-Host "Version updated: $currentVersion -> $nextVersion"
Write-Host "Run ./scripts/check.ps1 before creating the release tag."
