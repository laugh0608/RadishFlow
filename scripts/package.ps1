[CmdletBinding()]
param(
    [string]$Version,

    [ValidateSet('Debug', 'Release')]
    [string]$Configuration = 'Release',

    [string]$OutputRoot,

    [switch]$SkipBuild,

    [switch]$NoArchive,

    [switch]$Clean
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Get-RepositoryRoot {
    $current = $PSScriptRoot
    while ($null -ne $current) {
        if ((Test-Path -LiteralPath (Join-Path $current 'Cargo.toml')) -and
            (Test-Path -LiteralPath (Join-Path $current 'apps\radishflow-studio'))) {
            return (Resolve-Path -LiteralPath $current).Path
        }

        $parent = Split-Path -Parent $current
        if ($parent -eq $current) {
            break
        }

        $current = $parent
    }

    throw 'Could not locate repository root from scripts/package.ps1.'
}

function Get-DefaultVersion {
    $now = Get-Date
    return 'v{0}.{1}.1-dev' -f $now.ToString('yy'), [int]$now.Month
}

function Assert-PackageVersion {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Value
    )

    if ($Value -notmatch '^v\d{2}\.(?:[1-9]|1[0-2])\.\d+(?:\.\d{4})?-(?:dev|test|release)$') {
        throw "Package version must match vYY.M.RELEASE[-.DDXX]-(dev|test|release): $Value"
    }
}

function Get-PackagePlatform {
    $runsOnWindows = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
        [System.Runtime.InteropServices.OSPlatform]::Windows)
    if (-not $runsOnWindows) {
        throw 'MVP alpha packaging currently supports Windows artifacts only.'
    }

    switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
        'X64' { return 'windows-x64' }
        'Arm64' { return 'windows-arm64' }
        default { throw "Unsupported Windows architecture: $([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture)" }
    }
}

function Invoke-CheckedCommand {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Command,

        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    Write-Host ("==> {0} {1}" -f $Command, ($Arguments -join ' '))
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "command failed: $Command $($Arguments -join ' ')"
    }
}

function Assert-UnderDirectory {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Parent,

        [Parameter(Mandatory = $true)]
        [string]$Child
    )

    $parentPath = [System.IO.Path]::GetFullPath($Parent).TrimEnd('\', '/')
    $childPath = [System.IO.Path]::GetFullPath($Child).TrimEnd('\', '/')
    $comparison = [System.StringComparison]::OrdinalIgnoreCase

    if (-not $childPath.StartsWith($parentPath + [System.IO.Path]::DirectorySeparatorChar, $comparison)) {
        throw "Refusing to operate outside output root: $childPath"
    }
}

function Copy-RelativeFile {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RelativePath,

        [Parameter(Mandatory = $true)]
        [string]$SourceRoot,

        [Parameter(Mandatory = $true)]
        [string]$DestinationRoot
    )

    $source = Join-Path $SourceRoot $RelativePath
    if (-not (Test-Path -LiteralPath $source)) {
        throw "Required package input was not found: $source"
    }

    $destination = Join-Path $DestinationRoot $RelativePath
    $destinationParent = Split-Path -Parent $destination
    New-Item -ItemType Directory -Force -Path $destinationParent | Out-Null
    Copy-Item -LiteralPath $source -Destination $destination -Force
}

function Copy-DirectoryTree {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RelativePath,

        [Parameter(Mandatory = $true)]
        [string]$SourceRoot,

        [Parameter(Mandatory = $true)]
        [string]$DestinationRoot
    )

    $source = Join-Path $SourceRoot $RelativePath
    if (-not (Test-Path -LiteralPath $source)) {
        throw "Required package input was not found: $source"
    }

    $destination = Join-Path $DestinationRoot $RelativePath
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $destination) | Out-Null
    Copy-Item -LiteralPath $source -Destination $destination -Recurse -Force
}

$repoRoot = Get-RepositoryRoot
Set-Location -LiteralPath $repoRoot

if ([string]::IsNullOrWhiteSpace($Version)) {
    $Version = Get-DefaultVersion
}

Assert-PackageVersion -Value $Version

if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $repoRoot 'artifacts\packages'
}

$platform = Get-PackagePlatform
$packageName = 'RadishFlow-{0}-{1}' -f $Version, $platform
$outputRootPath = [System.IO.Path]::GetFullPath($OutputRoot)
$stagingDir = Join-Path $outputRootPath $packageName
$archivePath = Join-Path $outputRootPath ($packageName + '.zip')

New-Item -ItemType Directory -Force -Path $outputRootPath | Out-Null
Assert-UnderDirectory -Parent $outputRootPath -Child $stagingDir
Assert-UnderDirectory -Parent $outputRootPath -Child $archivePath

if ($Clean) {
    if (Test-Path -LiteralPath $stagingDir) {
        Remove-Item -LiteralPath $stagingDir -Recurse -Force
    }

    if (Test-Path -LiteralPath $archivePath) {
        Remove-Item -LiteralPath $archivePath -Force
    }
}

if (-not $SkipBuild) {
    $buildArgs = @('build', '-p', 'radishflow-studio', '--bin', 'radishflow-studio')
    if ($Configuration -eq 'Release') {
        $buildArgs += '--release'
    }

    Invoke-CheckedCommand -Command 'cargo' -Arguments $buildArgs
}

$targetProfile = $Configuration.ToLowerInvariant()
$studioExe = Join-Path $repoRoot "target\$targetProfile\radishflow-studio.exe"
if (-not (Test-Path -LiteralPath $studioExe)) {
    throw "Studio executable was not found: $studioExe"
}

New-Item -ItemType Directory -Force -Path $stagingDir | Out-Null
Copy-Item -LiteralPath $studioExe -Destination (Join-Path $stagingDir 'radishflow-studio.exe') -Force

$rootFiles = @(
    'README.md',
    'LICENSE',
    'docs\guides\studio-quick-start.md',
    'docs\guides\run-first-flowsheet.md',
    'docs\guides\review-solve-results.md',
    'docs\reference\units-and-conventions.md',
    'docs\reference\solve-snapshot-results.md',
    'docs\mvp\alpha-acceptance-checklist.md',
    'docs\architecture\versioning.md'
)

foreach ($relativePath in $rootFiles) {
    Copy-RelativeFile -RelativePath $relativePath -SourceRoot $repoRoot -DestinationRoot $stagingDir
}

$releaseNotesRelativePath = "docs\releases\$Version.md"
$releaseNotesSourcePath = Join-Path $repoRoot $releaseNotesRelativePath
$releaseNotesPackagePath = 'not-included'
if (Test-Path -LiteralPath $releaseNotesSourcePath) {
    Copy-RelativeFile -RelativePath $releaseNotesRelativePath -SourceRoot $repoRoot -DestinationRoot $stagingDir
    $releaseNotesPackagePath = $releaseNotesRelativePath.Replace('\', '/')
}

New-Item -ItemType Directory -Force -Path (Join-Path $stagingDir 'examples\flowsheets') | Out-Null
Get-ChildItem -LiteralPath (Join-Path $repoRoot 'examples\flowsheets') -File |
    ForEach-Object {
        $destination = Join-Path $stagingDir ('examples\flowsheets\' + $_.Name)
        Copy-Item -LiteralPath $_.FullName -Destination $destination -Force
    }

Copy-DirectoryTree -RelativePath 'examples\sample-components' -SourceRoot $repoRoot -DestinationRoot $stagingDir

$gitCommit = 'unknown'
try {
    $gitCommit = (& git rev-parse --short HEAD).Trim()
}
catch {
    $gitCommit = 'unknown'
}

$gitDirty = 'unknown'
try {
    $gitStatus = ((& git status --porcelain) -join [Environment]::NewLine).Trim()
    if ([string]::IsNullOrWhiteSpace($gitStatus)) {
        $gitDirty = 'false'
    }
    else {
        $gitDirty = 'true'
    }
}
catch {
    $gitDirty = 'unknown'
}

$manifest = @(
    "name=$packageName",
    "version=$Version",
    "platform=$platform",
    "configuration=$Configuration",
    "gitCommit=$gitCommit",
    "gitDirty=$gitDirty",
    "createdAt=$((Get-Date).ToString('o'))",
    'entrypoint=radishflow-studio.exe',
    'examples=examples/flowsheets',
    'sampleComponents=examples/sample-components',
    "releaseNotes=$releaseNotesPackagePath",
    'notes=This MVP alpha package is a portable zip/staging artifact, not an installer.'
)
$manifest | Set-Content -LiteralPath (Join-Path $stagingDir 'PACKAGE-MANIFEST.txt') -Encoding UTF8

$packageReadme = @'
# RadishFlow MVP Alpha Package

This package is a portable MVP alpha artifact for local review.

Start:

```powershell
.\radishflow-studio.exe
```

Included:

- RadishFlow Studio executable
- positive example flowsheets under `examples/flowsheets`
- sample property package payloads under `examples/sample-components`
- quick start, result review, acceptance, versioning, and license documents
- release notes under `docs/releases` when a matching version note exists

Not included:

- installer
- COM registration
- PME automation
- third-party CAPE-OPEN model loading

Run repository validation before publishing a release tag:

```powershell
pwsh ./scripts/check-repo.ps1
```
'@
$packageReadme | Set-Content -LiteralPath (Join-Path $stagingDir 'PACKAGE-README.md') -Encoding UTF8

if (-not $NoArchive) {
    if (Test-Path -LiteralPath $archivePath) {
        Remove-Item -LiteralPath $archivePath -Force
    }

    Compress-Archive -LiteralPath $stagingDir -DestinationPath $archivePath
}

Write-Host ("Package staging directory: {0}" -f $stagingDir)
if (-not $NoArchive) {
    Write-Host ("Package archive: {0}" -f $archivePath)
}
