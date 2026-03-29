[CmdletBinding()]
param(
    [switch]$SkipClippy
)

$ErrorActionPreference = "Stop"

function Invoke-CheckedCommand {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FilePath,

        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    Write-Host "==> $FilePath $($Arguments -join ' ')"
    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "command failed: $FilePath $($Arguments -join ' ')"
    }
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Push-Location $repoRoot

try {
    & (Join-Path $PSScriptRoot "check-text-files.ps1") -RepoRoot $repoRoot
    if ($LASTEXITCODE -ne 0) {
        throw "text file checks failed"
    }

    Invoke-CheckedCommand -FilePath "cargo" -Arguments @("fmt", "--all", "--check")
    Invoke-CheckedCommand -FilePath "cargo" -Arguments @("check", "--workspace")
    Invoke-CheckedCommand -FilePath "cargo" -Arguments @("test", "--workspace")

    if (-not $SkipClippy) {
        Invoke-CheckedCommand -FilePath "cargo" -Arguments @("clippy", "--workspace", "--all-targets", "--", "-D", "warnings")
    }
}
finally {
    Pop-Location
}

Write-Host "Repository checks passed."

