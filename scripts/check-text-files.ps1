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

$utf8Strict = [System.Text.UTF8Encoding]::new($false, $true)
$errors = [System.Collections.Generic.List[string]]::new()

$trackedFiles = git -C $RepoRoot ls-files --cached --others --exclude-standard
if ($LASTEXITCODE -ne 0) {
    throw "failed to enumerate tracked files"
}

foreach ($relativePath in $trackedFiles) {
    if ($relativePath.StartsWith("target/")) {
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

    if (
        $bytes.Length -ge 3 -and
        $bytes[0] -eq 0xEF -and
        $bytes[1] -eq 0xBB -and
        $bytes[2] -eq 0xBF
    ) {
        $errors.Add("${relativePath}: contains UTF-8 BOM")
    }

    if ([Array]::IndexOf($bytes, [byte]0) -ge 0) {
        $errors.Add("${relativePath}: contains NUL byte")
        continue
    }

    try {
        $null = $utf8Strict.GetString($bytes)
    }
    catch {
        $errors.Add("${relativePath}: is not valid UTF-8")
        continue
    }

    $content = [System.Text.Encoding]::UTF8.GetString($bytes)

    if ($content.Contains("`r")) {
        $errors.Add("${relativePath}: contains CRLF or CR line endings; expected LF")
    }

    if (-not $content.EndsWith("`n")) {
        $errors.Add("${relativePath}: missing trailing newline")
    }
}

if ($errors.Count -gt 0) {
    $errors | ForEach-Object { Write-Error $_ }
    throw "text file checks failed"
}

Write-Host "Text file checks passed."
