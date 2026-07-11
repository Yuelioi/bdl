param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Executable
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$path = (Resolve-Path -LiteralPath $Executable).Path
$bytes = [System.IO.File]::ReadAllBytes($path)
if ($bytes.Length -lt 256 -or $bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) {
    throw "Not a valid Windows PE executable: $path"
}

$peOffset = [BitConverter]::ToInt32($bytes, 0x3C)
$optionalHeaderOffset = $peOffset + 24
$subsystemOffset = $optionalHeaderOffset + 68
if ($subsystemOffset + 2 -gt $bytes.Length) {
    throw "Truncated Windows PE optional header: $path"
}

$subsystem = [BitConverter]::ToUInt16($bytes, $subsystemOffset)
if ($subsystem -ne 2) {
    throw "Expected IMAGE_SUBSYSTEM_WINDOWS_GUI (2), found subsystem $subsystem in $path."
}

Write-Host "Windows GUI subsystem verified: $path"
