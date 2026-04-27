[CmdletBinding()]
param(
    [ValidateSet('current-user', 'local-machine')]
    [string]$Scope = 'current-user',

    [ValidateSet('Debug', 'Release')]
    [string]$Configuration = 'Debug',

    [string]$BackupDir,

    [switch]$SkipRustBuild,

    [switch]$SkipTypeLibGeneration,

    [switch]$SkipDotnetBuild,

    [switch]$Json
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Get-RepositoryRoot {
    $current = $PSScriptRoot
    while ($null -ne $current) {
        if ((Test-Path -LiteralPath (Join-Path $current 'Cargo.toml')) -and
            (Test-Path -LiteralPath (Join-Path $current 'adapters\dotnet-capeopen'))) {
            return $current
        }

        $parent = Split-Path -Parent $current
        if ($parent -eq $current) {
            break
        }

        $current = $parent
    }

    throw 'Could not locate repository root from scripts/pme-register-latest.ps1.'
}

function Get-ConfirmationToken {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ActionName,

        [Parameter(Mandatory = $true)]
        [string]$ScopeName
    )

    return '{0}-{1}-2F0E4C8F' -f $ActionName, $ScopeName
}

$repoRoot = Get-RepositoryRoot
Set-Location -LiteralPath $repoRoot

if (-not $SkipRustBuild) {
    $cargoArgs = @('build', '-p', 'rf-ffi')
    if ($Configuration -eq 'Release') {
        $cargoArgs += '--release'
    }

    Write-Host ("Building native rf-ffi ({0})..." -f $Configuration)
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) {
        throw 'cargo build failed for rf-ffi.'
    }
}

if (-not $SkipTypeLibGeneration) {
    $typeLibScript = Join-Path $repoRoot 'scripts\gen-typelib.ps1'
    Write-Host 'Generating CAPE-OPEN UnitOp type library...'
    & $typeLibScript -Architecture x64
    if ($LASTEXITCODE -ne 0) {
        throw 'type library generation failed.'
    }
}

if ([string]::IsNullOrWhiteSpace($BackupDir)) {
    $BackupDir = Join-Path $repoRoot ("artifacts\registration-validation\register-{0}" -f $Scope)
}
else {
    $BackupDir = [System.IO.Path]::GetFullPath($BackupDir)
}

$registerScript = Join-Path $repoRoot 'scripts\register-com.ps1'
$confirmToken = Get-ConfirmationToken -ActionName 'register' -ScopeName $Scope
$registerParams = @{
    Scope = $Scope
    Execute = $true
    ConfirmToken = $confirmToken
    BackupDir = $BackupDir
    Configuration = $Configuration
}

if ($SkipDotnetBuild) {
    $registerParams.SkipBuild = $true
}

if ($Json) {
    $registerParams.Json = $true
}

Write-Host ("Registering RadishFlow CAPE-OPEN UnitOp for {0}..." -f $Scope)
& $registerScript @registerParams
exit $LASTEXITCODE
