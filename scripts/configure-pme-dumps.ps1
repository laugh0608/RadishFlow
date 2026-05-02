[CmdletBinding()]
param(
    [ValidateSet('enable', 'disable', 'status')]
    [string]$Action = 'status',

    [ValidateSet('local-machine', 'current-user', 'both')]
    [string]$Scope = 'local-machine',

    [string[]]$ProcessName = @('DWSIM.exe', 'COFE.exe'),

    [string]$DumpFolder,

    [ValidateSet('mini', 'full')]
    [string]$DumpType = 'full',

    [ValidateRange(1, 100)]
    [int]$DumpCount = 10
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

    throw 'Could not locate repository root from scripts/configure-pme-dumps.ps1.'
}

function Get-DumpTypeValue {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    if ($Name -eq 'mini') {
        return 1
    }

    return 2
}

function Test-IsAdministrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Get-LocalDumpsProcessKeyPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ScopeName,

        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    if ($ScopeName -eq 'local-machine') {
        return 'HKLM:\Software\Microsoft\Windows\Windows Error Reporting\LocalDumps\{0}' -f $Name
    }

    return 'HKCU:\Software\Microsoft\Windows\Windows Error Reporting\LocalDumps\{0}' -f $Name
}

function Write-DumpStatus {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ScopeName,

        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    $keyPath = Get-LocalDumpsProcessKeyPath -ScopeName $ScopeName -Name $Name
    if (-not (Test-Path -LiteralPath $keyPath)) {
        Write-Host ("{0} {1}: disabled" -f $ScopeName, $Name)
        return
    }

    $properties = Get-ItemProperty -LiteralPath $keyPath
    Write-Host ("{0} {1}: enabled" -f $ScopeName, $Name)
    Write-Host ("  DumpFolder: {0}" -f $properties.DumpFolder)
    Write-Host ("  DumpType: {0}" -f $properties.DumpType)
    Write-Host ("  DumpCount: {0}" -f $properties.DumpCount)
}

function Get-TargetScopes {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ScopeName,

        [Parameter(Mandatory = $true)]
        [string]$ActionName
    )

    if ($ActionName -eq 'status') {
        return @('local-machine', 'current-user')
    }

    if ($ScopeName -eq 'both') {
        return @('local-machine', 'current-user')
    }

    return @($ScopeName)
}

function Write-RegistryValueStatus {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,

        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        Write-Host ("{0}\{1}: absent" -f $Path, $Name)
        return
    }

    $properties = Get-ItemProperty -LiteralPath $Path
    $property = $properties.PSObject.Properties[$Name]
    if ($null -eq $property) {
        Write-Host ("{0}\{1}: absent" -f $Path, $Name)
        return
    }

    Write-Host ("{0}\{1}: {2}" -f $Path, $Name, $property.Value)
}

function Write-WerStatus {
    Write-Host 'WER policy/status:'
    Write-RegistryValueStatus -Path 'HKCU:\Software\Microsoft\Windows\Windows Error Reporting' -Name 'Disabled'
    Write-RegistryValueStatus -Path 'HKLM:\Software\Microsoft\Windows\Windows Error Reporting' -Name 'Disabled'
    Write-RegistryValueStatus -Path 'HKCU:\Software\Policies\Microsoft\Windows\Windows Error Reporting' -Name 'Disabled'
    Write-RegistryValueStatus -Path 'HKLM:\Software\Policies\Microsoft\Windows\Windows Error Reporting' -Name 'Disabled'
    Write-RegistryValueStatus -Path 'HKCU:\Software\Microsoft\Windows\Windows Error Reporting' -Name 'DontShowUI'
    Write-RegistryValueStatus -Path 'HKLM:\Software\Microsoft\Windows\Windows Error Reporting' -Name 'DontShowUI'
}

$repoRoot = Get-RepositoryRoot
if ([string]::IsNullOrWhiteSpace($DumpFolder)) {
    $DumpFolder = Join-Path $repoRoot 'artifacts\pme-dumps'
}

$resolvedDumpFolder = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($DumpFolder)
$dumpTypeValue = Get-DumpTypeValue -Name $DumpType
$targetScopes = Get-TargetScopes -ScopeName $Scope -ActionName $Action

foreach ($targetScope in $targetScopes) {
    if (($Action -eq 'enable' -or $Action -eq 'disable') -and $targetScope -eq 'local-machine' -and -not (Test-IsAdministrator)) {
        throw 'Configuring effective WER LocalDumps under HKLM requires an elevated PowerShell session. Re-run PowerShell as Administrator, or use -Scope current-user only to remove legacy ineffective HKCU settings.'
    }

    if ($Action -eq 'enable' -and $targetScope -eq 'current-user') {
        Write-Warning 'WER LocalDumps settings under HKCU are not effective for dump capture. Use -Scope local-machine from an elevated PowerShell session for real dump collection.'
    }

    foreach ($name in $ProcessName) {
        if ([string]::IsNullOrWhiteSpace($name)) {
            continue
        }

        $keyPath = Get-LocalDumpsProcessKeyPath -ScopeName $targetScope -Name $name

        if ($Action -eq 'enable') {
            New-Item -ItemType Directory -Path $resolvedDumpFolder -Force | Out-Null
            New-Item -Path $keyPath -Force | Out-Null
            New-ItemProperty -LiteralPath $keyPath -Name 'DumpFolder' -Value $resolvedDumpFolder -PropertyType ExpandString -Force | Out-Null
            New-ItemProperty -LiteralPath $keyPath -Name 'DumpType' -Value $dumpTypeValue -PropertyType DWord -Force | Out-Null
            New-ItemProperty -LiteralPath $keyPath -Name 'DumpCount' -Value $DumpCount -PropertyType DWord -Force | Out-Null
            Write-Host ("{0} {1}: enabled dumps at {2}" -f $targetScope, $name, $resolvedDumpFolder)
            continue
        }

        if ($Action -eq 'disable') {
            if (Test-Path -LiteralPath $keyPath) {
                Remove-Item -LiteralPath $keyPath -Recurse -Force
                Write-Host ("{0} {1}: disabled" -f $targetScope, $name)
            }
            else {
                Write-Host ("{0} {1}: already disabled" -f $targetScope, $name)
            }
            continue
        }

        Write-DumpStatus -ScopeName $targetScope -Name $name
    }
}

if ($Action -eq 'status') {
    Write-Host 'Note: WER LocalDumps settings under HKCU are not effective for dump capture; use local-machine/HKLM for real dump collection.'
    Write-WerStatus
    Write-Host ("Default dump folder: {0}" -f $resolvedDumpFolder)
    Write-Host ("Default dump folder exists: {0}" -f (Test-Path -LiteralPath $resolvedDumpFolder))
}
