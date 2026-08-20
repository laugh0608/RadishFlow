[CmdletBinding()]
param(
    [switch]$SkipClippy,
    [switch]$SkipTextFiles,
    [string]$BaseRef
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Push-Location $repoRoot

try {
    $arguments = @("run", "--quiet", "-p", "xtask", "--", "check-repo")

    if ($SkipClippy) {
        $arguments += "--skip-clippy"
    }

    if ($SkipTextFiles) {
        $arguments += "--skip-text-files"
    }

    if ($BaseRef) {
        $arguments += @("--base-ref", $BaseRef)
    }

    Write-Host "==> cargo $($arguments -join ' ')"
    & cargo @arguments
    if ($LASTEXITCODE -ne 0) {
        throw "command failed: cargo $($arguments -join ' ')"
    }
}
finally {
    Pop-Location
}
