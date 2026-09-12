[CmdletBinding()]
param(
    [ValidateSet('Debug', 'Release')]
    [string]$Configuration = 'Debug',

    [string]$NativeLibDir,

    [switch]$SkipNativeBuild,

    [switch]$SkipDotnetBuild,

    [switch]$SkipSmoke
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Get-RepositoryRoot {
    $current = $PSScriptRoot
    while ($null -ne $current) {
        if ((Test-Path -LiteralPath (Join-Path $current 'Cargo.toml')) -and
            (Test-Path -LiteralPath (Join-Path $current 'adapters\dotnet-capeopen'))) {
            return (Resolve-Path -LiteralPath $current).Path
        }

        $parent = Split-Path -Parent $current
        if ($parent -eq $current) {
            break
        }

        $current = $parent
    }

    throw 'Could not locate repository root from scripts/check-dotnet-capeopen.ps1.'
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

if (-not [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
        [System.Runtime.InteropServices.OSPlatform]::Windows)) {
    throw '.NET CAPE-OPEN baseline currently requires a Windows runner because ContractTests target net10.0-windows7.0 and validate COM-facing surfaces.'
}

$repoRoot = Get-RepositoryRoot
Set-Location -LiteralPath $repoRoot

$env:DOTNET_CLI_HOME = Join-Path $repoRoot '.dotnet-cli'
$env:APPDATA = Join-Path $repoRoot '.dotnet-appdata'
$env:LOCALAPPDATA = Join-Path $repoRoot '.dotnet-localappdata'
$env:USERPROFILE = Join-Path $repoRoot '.dotnet-home'
$env:NUGET_PACKAGES = Join-Path $env:USERPROFILE '.nuget\packages'

$targetProfile = $Configuration.ToLowerInvariant()
if ([string]::IsNullOrWhiteSpace($NativeLibDir)) {
    $NativeLibDir = Join-Path $repoRoot "target\$targetProfile"
}

$NativeLibDir = [System.IO.Path]::GetFullPath($NativeLibDir)
$solutionPath = Join-Path $repoRoot 'adapters\dotnet-capeopen\RadishFlow.CapeOpen.sln'
$contractProject = Join-Path $repoRoot 'adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests\RadishFlow.CapeOpen.UnitOp.Mvp.ContractTests.csproj'

if (-not $SkipNativeBuild) {
    $cargoArgs = @('build', '--locked', '-p', 'rf-ffi')
    if ($Configuration -eq 'Release') {
        $cargoArgs += '--release'
    }

    Invoke-CheckedCommand -Command 'cargo' -Arguments $cargoArgs
}

if (-not (Test-Path -LiteralPath $NativeLibDir)) {
    throw "Native library directory was not found: $NativeLibDir"
}

if (-not $SkipDotnetBuild) {
    Invoke-CheckedCommand -Command 'dotnet' -Arguments @('build', $solutionPath, '-c', $Configuration, '--nologo')
}

Invoke-CheckedCommand -Command 'dotnet' -Arguments @(
    'run',
    '--project',
    $contractProject,
    '-c',
    $Configuration,
    '--no-build',
    '--',
    '--native-lib-dir',
    $NativeLibDir
)

if (-not $SkipSmoke) {
    $smokeArgs = @('-Configuration', $Configuration, '-NativeLibDir', $NativeLibDir, '-SkipBuild')
    $pwshArgs = @(
        '-NoProfile',
        '-File',
        (Join-Path $repoRoot 'scripts\smoke-test.ps1')
    ) + $smokeArgs

    Invoke-CheckedCommand -Command 'pwsh' -Arguments $pwshArgs
}

Write-Host '.NET CAPE-OPEN baseline passed.'
