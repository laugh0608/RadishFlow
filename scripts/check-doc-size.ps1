[CmdletBinding()]
param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path,
    [switch]$FailOnExceeded,
    [switch]$IncludeAdvisory
)

$ErrorActionPreference = "Stop"

function Get-DocSizeRule {
    param([string]$RelativePath)

    $path = $RelativePath.Replace("\", "/")

    if ($path -in @("AGENTS.md", "CLAUDE.md")) {
        return [pscustomobject]@{ Limit = 14000; Scope = "entry"; Enforced = $true }
    }
    if ($path -eq "docs/status/current.md") {
        return [pscustomobject]@{ Limit = 8000; Scope = "status"; Enforced = $true }
    }
    if ($path -eq "docs/README.md") {
        return [pscustomobject]@{ Limit = 10000; Scope = "docs-index"; Enforced = $true }
    }
    if ($path -like "docs/adr/*.md") {
        return [pscustomobject]@{ Limit = 12000; Scope = "adr"; Enforced = $true }
    }
    if ($path -like "docs/guides/*.md" -or $path -eq "docs/capeopen/pme-validation.md") {
        return [pscustomobject]@{ Limit = 15000; Scope = "guide"; Enforced = $true }
    }
    if ($path -like "docs/reference/*.md") {
        return [pscustomobject]@{ Limit = 25000; Scope = "reference"; Enforced = $true }
    }
    if ($path -like "docs/architecture/*.md" -or $path -eq "docs/capeopen/boundary.md" -or $path -like "docs/thermo/*.md" -or $path -like "docs/mvp/*.md" -or $path -like "docs/mvp/roadmap/*.md" -or $path -eq "docs/radishflow-mvp-roadmap.md") {
        return [pscustomobject]@{ Limit = 30000; Scope = "topic"; Enforced = $true }
    }
    if ($path -like "docs/devlogs/*.md" -or $path -like "docs/devlogs/*/*.md" -or $path -like "docs/*draft*.md" -or $path -like "docs/*checklist*.md") {
        return [pscustomobject]@{ Limit = 30000; Scope = "history"; Enforced = $false }
    }

    return [pscustomobject]@{ Limit = 25000; Scope = "other"; Enforced = $true }
}

$root = (Resolve-Path $RepoRoot).Path
$rootFiles = @("AGENTS.md", "CLAUDE.md", "README.md") |
    ForEach-Object { Join-Path $root $_ } |
    Where-Object { Test-Path -LiteralPath $_ } |
    ForEach-Object { Get-Item -LiteralPath $_ }

$docFiles = Get-ChildItem -LiteralPath (Join-Path $root "docs") -Recurse -File -Filter "*.md"
$files = @($rootFiles) + @($docFiles)

$results = foreach ($file in $files) {
    $relativePath = [System.IO.Path]::GetRelativePath($root, $file.FullName)
    $rule = Get-DocSizeRule -RelativePath $relativePath
    $content = Get-Content -Raw -LiteralPath $file.FullName
    $chars = $content.Length

    [pscustomobject]@{
        Path = $relativePath
        Chars = $chars
        Limit = $rule.Limit
        Scope = $rule.Scope
        Enforced = $rule.Enforced
        Status = if ($chars -gt $rule.Limit) { "over" } else { "ok" }
    }
}

$overLimit = @(
    $results |
        Where-Object { $_.Status -eq "over" -and ($_.Enforced -or $IncludeAdvisory) } |
        Sort-Object Enforced, Chars -Descending
)
$enforcedOverLimit = @($overLimit | Where-Object { $_.Enforced })

if ($overLimit.Count -eq 0) {
    if ($IncludeAdvisory) {
        Write-Host "doc size check: all markdown files are within target limits"
    }
    else {
        Write-Host "doc size check: all enforced markdown files are within target limits"
    }
}
else {
    if ($IncludeAdvisory) {
        Write-Host "doc size check: files over target limits"
    }
    else {
        Write-Host "doc size check: enforced files over target limits"
    }
    $overLimit |
        Select-Object Path, Chars, Limit, Scope, Enforced |
        Format-Table -AutoSize
}

if ($FailOnExceeded -and $enforcedOverLimit.Count -gt 0) {
    throw "doc size check failed: $($enforcedOverLimit.Count) enforced file(s) exceed target limits"
}
