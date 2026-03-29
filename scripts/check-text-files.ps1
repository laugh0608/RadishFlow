[CmdletBinding()]
param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
)

$ErrorActionPreference = "Stop"

Push-Location $RepoRoot

try {
    $arguments = @("run", "--quiet", "-p", "xtask", "--", "check-text-files")

    Write-Host "==> cargo $($arguments -join ' ')"
    & cargo @arguments
    if ($LASTEXITCODE -ne 0) {
        throw "command failed: cargo $($arguments -join ' ')"
    }
}
finally {
    Pop-Location
}
