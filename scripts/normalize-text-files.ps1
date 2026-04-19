[CmdletBinding()]
param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
)

$ErrorActionPreference = "Stop"

$textExtensions = @(
    ".md",
    ".toml",
    ".rs",
    ".ps1",
    ".sh",
    ".yml",
    ".yaml",
    ".json",
    ".props",
    ".targets",
    ".sln",
    ".txt",
    ".cs",
    ".csproj"
)

$textFileNames = @(
    ".editorconfig",
    ".gitattributes",
    ".gitignore",
    "AGENTS.md",
    "Cargo.lock",
    "Cargo.toml",
    "LICENSE",
    "README.md"
)

$excludedPrefixes = @(
    "adapters/reference/"
)

$utf8Strict = [System.Text.UTF8Encoding]::new($false, $true)
$utf8NoBom = [System.Text.UTF8Encoding]::new($false)

$files = git -C $RepoRoot ls-files --cached --others --exclude-standard
if ($LASTEXITCODE -ne 0) {
    throw "failed to enumerate repository files"
}

foreach ($relativePath in $files) {
    if ($relativePath.StartsWith("target/")) {
        continue
    }

    $normalizedRelativePath = $relativePath.Replace("\", "/")
    $isExcluded = $false
    foreach ($prefix in $excludedPrefixes) {
        if ($normalizedRelativePath.StartsWith($prefix)) {
            $isExcluded = $true
            break
        }
    }

    if ($isExcluded) {
        continue
    }

    $fullPath = Join-Path $RepoRoot $relativePath
    if (-not (Test-Path $fullPath -PathType Leaf)) {
        continue
    }

    $extension = [System.IO.Path]::GetExtension($relativePath).ToLowerInvariant()
    $fileName = [System.IO.Path]::GetFileName($relativePath)

    if (($textExtensions -notcontains $extension) -and ($textFileNames -notcontains $fileName)) {
        continue
    }

    $bytes = [System.IO.File]::ReadAllBytes($fullPath)
    if ($bytes.Length -eq 0) {
        continue
    }

    if ([Array]::IndexOf($bytes, [byte]0) -ge 0) {
        continue
    }

    $content = $null
    try {
        $content = $utf8Strict.GetString($bytes)
    }
    catch {
        $content = [System.Text.Encoding]::Default.GetString($bytes)
    }

    if ($content.Length -gt 0 -and $content[0] -eq [char]0xFEFF) {
        $content = $content.Substring(1)
    }

    $normalized = $content.Replace("`r`n", "`n").Replace("`r", "`n")
    if (-not $normalized.EndsWith("`n")) {
        $normalized += "`n"
    }

    [System.IO.File]::WriteAllText($fullPath, $normalized, $utf8NoBom)
}

Write-Host "Text files normalized to UTF-8 without BOM and LF line endings."
