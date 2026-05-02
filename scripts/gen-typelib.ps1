[CmdletBinding()]
param(
    [string]$IdlPath,

    [string]$TypeLibPath,

    [string]$IntermediateDir,

    [string]$MidlPath,

    [string]$VcVarsPath,

    [ValidateSet('x64', 'x86', 'arm64')]
    [string]$Architecture = 'x64'
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

    throw 'Could not locate repository root from scripts/gen-typelib.ps1.'
}

function Resolve-OptionalPath {
    param(
        [string]$PathValue
    )

    if ([string]::IsNullOrWhiteSpace($PathValue)) {
        return $null
    }

    return [System.IO.Path]::GetFullPath($PathValue)
}

function Find-Midl {
    param(
        [string]$ExplicitPath,
        [string]$TargetArchitecture
    )

    if (-not [string]::IsNullOrWhiteSpace($ExplicitPath)) {
        $resolved = Resolve-Path -LiteralPath $ExplicitPath
        return $resolved.Path
    }

    $command = Get-Command 'midl.exe' -ErrorAction SilentlyContinue
    if ($null -ne $command) {
        return $command.Source
    }

    $candidateRoots = @()
    if (-not [string]::IsNullOrWhiteSpace($env:WindowsSdkDir)) {
        $candidateRoots += (Join-Path $env:WindowsSdkDir 'bin')
    }

    $candidateRoots += @(
        'D:\Windows Kits\10\bin',
        'C:\Program Files (x86)\Windows Kits\10\bin',
        'C:\Program Files\Windows Kits\10\bin'
    )

    foreach ($root in ($candidateRoots | Select-Object -Unique)) {
        if (-not (Test-Path -LiteralPath $root)) {
            continue
        }

        $versionDirs = Get-ChildItem -LiteralPath $root -Directory -ErrorAction SilentlyContinue |
            Sort-Object Name -Descending

        foreach ($versionDir in $versionDirs) {
            $candidate = Join-Path $versionDir.FullName (Join-Path $TargetArchitecture 'midl.exe')
            if (Test-Path -LiteralPath $candidate) {
                return $candidate
            }
        }
    }

    throw 'Could not locate midl.exe. Install Windows SDK, put midl.exe on PATH, or pass -MidlPath.'
}

function Find-SdkIncludeDirs {
    param(
        [string]$ResolvedMidlPath
    )

    $architectureDir = Split-Path -Parent $ResolvedMidlPath
    $versionDir = Split-Path -Parent $architectureDir
    $binDir = Split-Path -Parent $versionDir
    $sdkRoot = Split-Path -Parent $binDir
    $sdkVersion = Split-Path -Leaf $versionDir
    $includeRoot = Join-Path $sdkRoot (Join-Path 'Include' $sdkVersion)

    $includeDirs = @(
        (Join-Path $includeRoot 'um'),
        (Join-Path $includeRoot 'shared'),
        (Join-Path $includeRoot 'ucrt')
    )

    return @($includeDirs | Where-Object { Test-Path -LiteralPath $_ })
}

function Find-VcVars {
    param(
        [string]$ExplicitPath,
        [string]$TargetArchitecture
    )

    if (-not [string]::IsNullOrWhiteSpace($ExplicitPath)) {
        $resolved = Resolve-Path -LiteralPath $ExplicitPath
        return $resolved.Path
    }

    if ($null -ne (Get-Command 'cl.exe' -ErrorAction SilentlyContinue)) {
        return $null
    }

    $vswhereCandidates = @(
        'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe',
        'C:\Program Files\Microsoft Visual Studio\Installer\vswhere.exe'
    )

    foreach ($vswhere in $vswhereCandidates) {
        if (-not (Test-Path -LiteralPath $vswhere)) {
            continue
        }

        $installationPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($installationPath)) {
            continue
        }

        $scriptName = switch ($TargetArchitecture) {
            'x64' { 'vcvars64.bat' }
            'x86' { 'vcvars32.bat' }
            'arm64' { 'vcvarsamd64_arm64.bat' }
        }

        $candidate = Join-Path $installationPath (Join-Path 'VC\Auxiliary\Build' $scriptName)
        if (Test-Path -LiteralPath $candidate) {
            return $candidate
        }
    }

    throw 'Could not locate cl.exe or Visual Studio vcvars script. Install Visual Studio C++ tools, put cl.exe on PATH, or pass -VcVarsPath.'
}

function ConvertTo-CmdArgument {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Argument
    )

    return '"' + $Argument.Replace('"', '\"') + '"'
}

$repoRoot = Get-RepositoryRoot
Set-Location -LiteralPath $repoRoot

if ([string]::IsNullOrWhiteSpace($IdlPath)) {
    $IdlPath = Join-Path $repoRoot 'adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp\typelib\RadishFlow.CapeOpen.UnitOp.Mvp.idl'
}
else {
    $IdlPath = Resolve-OptionalPath -PathValue $IdlPath
}

if ([string]::IsNullOrWhiteSpace($TypeLibPath)) {
    $TypeLibPath = Join-Path $repoRoot 'adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp\typelib\RadishFlow.CapeOpen.UnitOp.Mvp.tlb'
}
else {
    $TypeLibPath = Resolve-OptionalPath -PathValue $TypeLibPath
}

if ([string]::IsNullOrWhiteSpace($IntermediateDir)) {
    $IntermediateDir = Join-Path $repoRoot 'artifacts\typelib\RadishFlow.CapeOpen.UnitOp.Mvp'
}
else {
    $IntermediateDir = Resolve-OptionalPath -PathValue $IntermediateDir
}

$IdlPath = (Resolve-Path -LiteralPath $IdlPath).Path
$TypeLibDirectory = Split-Path -Parent $TypeLibPath
New-Item -ItemType Directory -Force -Path $TypeLibDirectory | Out-Null
New-Item -ItemType Directory -Force -Path $IntermediateDir | Out-Null

$resolvedMidlPath = Find-Midl -ExplicitPath $MidlPath -TargetArchitecture $Architecture
$includeDirs = Find-SdkIncludeDirs -ResolvedMidlPath $resolvedMidlPath
if ($includeDirs.Count -eq 0) {
    throw "Could not locate Windows SDK include directories for $resolvedMidlPath."
}

$resolvedVcVarsPath = Find-VcVars -ExplicitPath $VcVarsPath -TargetArchitecture $Architecture
$midlEnvironment = switch ($Architecture) {
    'x64' { 'x64' }
    'x86' { 'win32' }
    'arm64' { 'arm64' }
}

$headerPath = Join-Path $IntermediateDir 'RadishFlow.CapeOpen.UnitOp.Mvp.h'
$iidPath = Join-Path $IntermediateDir 'RadishFlow.CapeOpen.UnitOp.Mvp_i.c'
$proxyPath = Join-Path $IntermediateDir 'RadishFlow.CapeOpen.UnitOp.Mvp_p.c'
$dlldataPath = Join-Path $IntermediateDir 'RadishFlow.CapeOpen.UnitOp.Mvp_dlldata.c'

$midlArgs = @(
    '/nologo',
    '/char', 'signed',
    '/env', $midlEnvironment,
    '/client', 'none',
    '/server', 'none',
    '/out', $IntermediateDir,
    '/tlb', $TypeLibPath,
    '/h', $headerPath,
    '/iid', $iidPath,
    '/proxy', $proxyPath,
    '/dlldata', $dlldataPath
)

foreach ($includeDir in $includeDirs) {
    $midlArgs += '/I'
    $midlArgs += $includeDir
}

$midlArgs += $IdlPath

Write-Host ("Repository root: {0}" -f $repoRoot)
Write-Host ("IDL: {0}" -f $IdlPath)
Write-Host ("TLB: {0}" -f $TypeLibPath)
Write-Host ("MIDL: {0}" -f $resolvedMidlPath)
Write-Host ("Architecture: {0}" -f $Architecture)
Write-Host ("Intermediate dir: {0}" -f $IntermediateDir)

if ($null -eq $resolvedVcVarsPath) {
    & $resolvedMidlPath @midlArgs
}
else {
    Write-Host ("Visual C++ environment: {0}" -f $resolvedVcVarsPath)
    $quotedMidl = ConvertTo-CmdArgument -Argument $resolvedMidlPath
    $quotedArgs = $midlArgs | ForEach-Object { ConvertTo-CmdArgument -Argument $_ }
    $cmdLine = 'call ' + (ConvertTo-CmdArgument -Argument $resolvedVcVarsPath) + ' >nul && ' + $quotedMidl + ' ' + ($quotedArgs -join ' ')
    & cmd.exe /d /s /c $cmdLine
}

if ($LASTEXITCODE -ne 0) {
    throw "midl.exe failed with exit code $LASTEXITCODE."
}

if (-not (Test-Path -LiteralPath $TypeLibPath)) {
    throw "MIDL completed but the type library was not found: $TypeLibPath"
}

Write-Host ("Generated type library: {0}" -f $TypeLibPath)
