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
    # Workspace integration tests rebuild xtask.exe; Windows locks the running executable.
    $arguments = @("run", "--locked", "--quiet", "--target-dir", "target/xtask", "-p", "xtask", "--", "check-repo")

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
