[CmdletBinding()]
param(
    [ValidateSet('register', 'unregister')]
    [string]$Action = 'register',

    [ValidateSet('current-user', 'local-machine')]
    [string]$Scope = 'current-user',

    [switch]$Execute,

    [string]$ConfirmToken,

    [string]$ComHostPath,

    [string]$BackupDir,

    [switch]$Json,

    [switch]$SkipBuild,

    [ValidateSet('Debug', 'Release')]
    [string]$Configuration = 'Debug'
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

    throw 'Could not locate repository root from scripts/register-com.ps1.'
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

$env:DOTNET_CLI_HOME = Join-Path $repoRoot '.dotnet-cli'
$env:APPDATA = Join-Path $repoRoot '.dotnet-appdata'
$env:LOCALAPPDATA = Join-Path $repoRoot '.dotnet-localappdata'
$env:USERPROFILE = Join-Path $repoRoot '.dotnet-home'
$env:NUGET_PACKAGES = Join-Path $env:USERPROFILE '.nuget\packages'

$registrationProject = Join-Path $repoRoot 'adapters\dotnet-capeopen\RadishFlow.CapeOpen.Registration\RadishFlow.CapeOpen.Registration.csproj'
$registrationExe = Join-Path $repoRoot ("adapters\dotnet-capeopen\RadishFlow.CapeOpen.Registration\bin\{0}\net10.0\RadishFlow.CapeOpen.Registration.exe" -f $Configuration)

if (-not $SkipBuild) {
    & dotnet build $registrationProject -c $Configuration -v minimal
    if ($LASTEXITCODE -ne 0) {
        throw "dotnet build failed for $registrationProject"
    }
}

if (-not (Test-Path -LiteralPath $registrationExe)) {
    throw "Registration executable was not found: $registrationExe"
}

$expectedToken = Get-ConfirmationToken -ActionName $Action -ScopeName $Scope
if ($Execute -and [string]::IsNullOrWhiteSpace($ConfirmToken)) {
    throw "Execution requires -ConfirmToken '$expectedToken'."
}

$registrationArgs = @(
    '--action', $Action,
    '--scope', $Scope
)

if ($Execute) {
    $registrationArgs += '--execute'
    $registrationArgs += '--confirm'
    $registrationArgs += $ConfirmToken
}

if (-not [string]::IsNullOrWhiteSpace($ComHostPath)) {
    $registrationArgs += '--comhost'
    $registrationArgs += (Resolve-Path -LiteralPath $ComHostPath).Path
}

if (-not [string]::IsNullOrWhiteSpace($BackupDir)) {
    $registrationArgs += '--backup-dir'
    $registrationArgs += $BackupDir
}

if ($Json) {
    $registrationArgs += '--json'
}

Write-Host ("Repository root: {0}" -f $repoRoot)
Write-Host ("Registration exe: {0}" -f $registrationExe)
Write-Host ("Action: {0}" -f $Action)
Write-Host ("Scope: {0}" -f $Scope)
Write-Host ("Mode: {0}" -f ($(if ($Execute) { 'execute' } else { 'dry-run' })))
Write-Host ("Expected confirm token: {0}" -f $expectedToken)

& $registrationExe @registrationArgs
exit $LASTEXITCODE
