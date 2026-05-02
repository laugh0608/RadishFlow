[CmdletBinding()]
param(
    [ValidateSet('current-user', 'local-machine')]
    [string]$Scope = 'current-user',

    [ValidateSet('Debug', 'Release')]
    [string]$Configuration = 'Debug',

    [string]$BackupDir,

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

    throw 'Could not locate repository root from scripts/pme-unregister.ps1.'
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

if ([string]::IsNullOrWhiteSpace($BackupDir)) {
    $BackupDir = Join-Path $repoRoot ("artifacts\registration-validation\unregister-{0}" -f $Scope)
}
else {
    $BackupDir = [System.IO.Path]::GetFullPath($BackupDir)
}

$registerScript = Join-Path $repoRoot 'scripts\register-com.ps1'
$confirmToken = Get-ConfirmationToken -ActionName 'unregister' -ScopeName $Scope
$unregisterParams = @{
    Action = 'unregister'
    Scope = $Scope
    Execute = $true
    ConfirmToken = $confirmToken
    BackupDir = $BackupDir
    Configuration = $Configuration
}

if ($SkipDotnetBuild) {
    $unregisterParams.SkipBuild = $true
}

if ($Json) {
    $unregisterParams.Json = $true
}

Write-Host ("Unregistering RadishFlow CAPE-OPEN UnitOp for {0}..." -f $Scope)
& $registerScript @unregisterParams
exit $LASTEXITCODE
