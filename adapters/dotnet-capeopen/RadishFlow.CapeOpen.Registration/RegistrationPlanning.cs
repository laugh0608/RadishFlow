using System.Security.Principal;
using System.Runtime.InteropServices;
using Microsoft.Win32;

internal static class CapeOpenRegistrationConfirmationToken
{
    public static string Create(
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope,
        string classId)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(classId);

        var tokenScope = scope == CapeOpenRegistrationScope.CurrentUser
            ? "current-user"
            : "local-machine";
        var actionText = action == CapeOpenRegistrationAction.Register
            ? "register"
            : "unregister";
        var classIdPrefix = classId
            .Replace("-", string.Empty, StringComparison.Ordinal)
            .ToUpperInvariant()[..8];
        return $"{actionText}-{tokenScope}-{classIdPrefix}";
    }
}

internal static class CapeOpenComHostPathResolver
{
    public static string Resolve(
        Type componentType,
        string? explicitPath)
    {
        ArgumentNullException.ThrowIfNull(componentType);

        if (!string.IsNullOrWhiteSpace(explicitPath))
        {
            return Path.GetFullPath(explicitPath);
        }

        var fileName = "RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll";
        var candidates = CapeOpenUnitOperationOutputLocator.EnumerateCandidateOutputDirectories(componentType)
            .Select(directory => Path.Combine(directory, fileName))
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .ToArray();
        var resolved = candidates.FirstOrDefault(candidate =>
                File.Exists(candidate) && CapeOpenComHostRuntimeLayoutInspector.HasRequiredSidecars(candidate))
            ?? candidates.FirstOrDefault(File.Exists);

        return resolved is not null
            ? Path.GetFullPath(resolved)
            : Path.GetFullPath(candidates[0]);
    }
}

internal static class CapeOpenUnitOperationOutputLocator
{
    private const string CargoTomlFileName = "Cargo.toml";
    private const string UnitOperationProjectRelativePath = @"adapters\dotnet-capeopen\RadishFlow.CapeOpen.UnitOp.Mvp";

    public static IReadOnlyList<string> EnumerateCandidateOutputDirectories(Type componentType)
    {
        ArgumentNullException.ThrowIfNull(componentType);

        var directories = new List<string>();
        AddRepositoryOutputDirectory(directories, AppContext.BaseDirectory);
        AddRepositoryOutputDirectory(directories, componentType.Assembly.Location);
        AddDirectory(directories, Path.GetDirectoryName(componentType.Assembly.Location));
        AddDirectory(directories, AppContext.BaseDirectory);
        AddDirectory(directories, Environment.CurrentDirectory);

        return directories
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .ToArray();
    }

    private static void AddRepositoryOutputDirectory(
        List<string> directories,
        string? startPath)
    {
        if (!TryExtractBuildOutputCoordinates(startPath, out var configuration, out var targetFramework) ||
            !TryResolveRepositoryRoot(startPath, out var repositoryRoot))
        {
            return;
        }

        foreach (var targetFrameworkCandidate in EnumerateTargetFrameworkCandidates(targetFramework))
        {
            AddDirectory(
                directories,
                Path.Combine(
                    repositoryRoot,
                    UnitOperationProjectRelativePath,
                    "bin",
                    configuration,
                    targetFrameworkCandidate));
        }
    }

    private static void AddDirectory(
        List<string> directories,
        string? path)
    {
        if (string.IsNullOrWhiteSpace(path))
        {
            return;
        }

        directories.Add(Path.GetFullPath(path));
    }

    private static bool TryExtractBuildOutputCoordinates(
        string? startPath,
        out string configuration,
        out string targetFramework)
    {
        configuration = string.Empty;
        targetFramework = string.Empty;

        if (string.IsNullOrWhiteSpace(startPath))
        {
            return false;
        }

        var normalizedPath = Path.GetFullPath(startPath);
        var directory = File.Exists(normalizedPath)
            ? Path.GetDirectoryName(normalizedPath)
            : normalizedPath;
        if (string.IsNullOrWhiteSpace(directory))
        {
            return false;
        }

        var outputDirectory = new DirectoryInfo(directory);
        var configurationDirectory = outputDirectory.Parent;
        var binDirectory = configurationDirectory?.Parent;
        if (configurationDirectory is null ||
            binDirectory is null ||
            !string.Equals(binDirectory.Name, "bin", StringComparison.OrdinalIgnoreCase))
        {
            return false;
        }

        configuration = configurationDirectory.Name;
        targetFramework = outputDirectory.Name;
        return !string.IsNullOrWhiteSpace(configuration) && !string.IsNullOrWhiteSpace(targetFramework);
    }

    private static bool TryResolveRepositoryRoot(
        string? startPath,
        out string repositoryRoot)
    {
        repositoryRoot = string.Empty;
        if (string.IsNullOrWhiteSpace(startPath))
        {
            return false;
        }

        var normalizedPath = Path.GetFullPath(startPath);
        var directory = File.Exists(normalizedPath)
            ? Path.GetDirectoryName(normalizedPath)
            : normalizedPath;
        if (string.IsNullOrWhiteSpace(directory))
        {
            return false;
        }

        var current = new DirectoryInfo(directory);
        while (current is not null)
        {
            if (File.Exists(Path.Combine(current.FullName, CargoTomlFileName)) &&
                Directory.Exists(Path.Combine(current.FullName, UnitOperationProjectRelativePath)))
            {
                repositoryRoot = current.FullName;
                return true;
            }

            current = current.Parent;
        }

        return false;
    }

    private static IEnumerable<string> EnumerateTargetFrameworkCandidates(string targetFramework)
    {
        yield return targetFramework;

        var separatorIndex = targetFramework.IndexOf('-');
        if (separatorIndex > 0)
        {
            yield return targetFramework[..separatorIndex];
        }
    }
}

internal static class CapeOpenComHostRuntimeLayoutInspector
{
    public static IReadOnlyList<string> GetMissingSidecars(string comHostPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(comHostPath);

        var directory = Path.GetDirectoryName(comHostPath) ?? Environment.CurrentDirectory;
        return new[]
        {
            Path.Combine(directory, "RadishFlow.CapeOpen.UnitOp.Mvp.runtimeconfig.json"),
            Path.Combine(directory, "RadishFlow.CapeOpen.UnitOp.Mvp.deps.json"),
        }
            .Where(path => !File.Exists(path))
            .ToArray();
    }

    public static bool HasRequiredSidecars(string comHostPath)
    {
        return GetMissingSidecars(comHostPath).Count == 0;
    }
}

internal static class CapeOpenRegistrationPreflightChecker
{
    public static IReadOnlyList<CapeOpenPreflightCheck> Check(
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope,
        CapeOpenRegistrationExecutionMode executionMode,
        string comHostPath,
        string typeLibraryPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(comHostPath);
        ArgumentException.ThrowIfNullOrWhiteSpace(typeLibraryPath);

        var checks = new List<CapeOpenPreflightCheck>
        {
            CheckComHostPath(comHostPath),
            CheckComHostArchitecture(comHostPath),
            CheckComHostRuntimeLayout(comHostPath),
            CheckTypeLibraryPath(typeLibraryPath),
            CheckTypeLibraryIdentity(typeLibraryPath),
            CheckProcessArchitecture(),
            CheckScopePermission(scope, executionMode),
        };

        checks.AddRange(CheckRegistryConflicts(action, scope));
        return checks;
    }

    private static CapeOpenPreflightCheck CheckComHostPath(string comHostPath)
    {
        return File.Exists(comHostPath)
            ? Pass("comhost path", $"Resolved comhost DLL: {comHostPath}")
            : Fail("comhost path", $"Generated comhost DLL was not found: {comHostPath}");
    }

    private static CapeOpenPreflightCheck CheckProcessArchitecture()
    {
        return Pass(
            "process architecture",
            $"Current process is {(Environment.Is64BitProcess ? "64-bit" : "32-bit")}; registry view must match the target PME bitness.");
    }

    private static CapeOpenPreflightCheck CheckComHostArchitecture(string comHostPath)
    {
        if (!File.Exists(comHostPath))
        {
            return Fail("comhost architecture", "Cannot inspect comhost architecture because the file does not exist.");
        }

        try
        {
            var architecture = PortableExecutableArchitectureReader.ReadMachineArchitecture(comHostPath);
            var processArchitecture = Environment.Is64BitProcess
                ? PortableExecutableMachineArchitecture.X64
                : PortableExecutableMachineArchitecture.X86;
            if (architecture == PortableExecutableMachineArchitecture.Unknown)
            {
                return Warn("comhost architecture", $"Could not classify comhost machine type in `{comHostPath}`.");
            }

            if (architecture != processArchitecture &&
                architecture is PortableExecutableMachineArchitecture.X64 or PortableExecutableMachineArchitecture.X86)
            {
                return Warn(
                    "comhost architecture",
                    $"Comhost is {architecture}, current process is {processArchitecture}; target PME bitness must match the registered in-proc server.");
            }

            return Pass(
                "comhost architecture",
                $"Comhost architecture is {architecture}; target PME bitness must match this in-proc server.");
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException or InvalidDataException)
        {
            return Fail("comhost architecture", $"Failed to inspect comhost PE header: {error.Message}");
        }
    }

    private static CapeOpenPreflightCheck CheckComHostRuntimeLayout(string comHostPath)
    {
        if (!File.Exists(comHostPath))
        {
            return Fail("comhost runtime layout", "Cannot inspect runtime sidecars because the comhost file does not exist.");
        }

        var missingSidecars = CapeOpenComHostRuntimeLayoutInspector.GetMissingSidecars(comHostPath);
        return missingSidecars.Count == 0
            ? Pass("comhost runtime layout", "Resolved comhost directory contains UnitOp.Mvp runtimeconfig/deps sidecars required for .NET COM activation.")
            : Fail(
                "comhost runtime layout",
                $"Resolved comhost directory is missing required UnitOp.Mvp runtime sidecars: {string.Join(", ", missingSidecars)}");
    }

    private static CapeOpenPreflightCheck CheckScopePermission(
        CapeOpenRegistrationScope scope,
        CapeOpenRegistrationExecutionMode executionMode)
    {
        if (scope == CapeOpenRegistrationScope.CurrentUser)
        {
            return Pass("scope permission", "Current-user scope targets HKCU and does not require elevation.");
        }

        if (executionMode == CapeOpenRegistrationExecutionMode.DryRun)
        {
            return Warn("scope permission", "Local-machine dry-run targets HKLM; real execution will require elevation.");
        }

        return CapeOpenRegistrationElevationChecker.IsProcessElevated()
            ? Pass("scope permission", "Local-machine execution is running with elevation.")
            : Fail("scope permission", "Local-machine execution requires elevation before any HKLM write.");
    }

    private static CapeOpenPreflightCheck CheckTypeLibraryPath(string typeLibraryPath)
    {
        return File.Exists(typeLibraryPath)
            ? Pass("type library path", $"Resolved type library: {typeLibraryPath}")
            : Fail("type library path", $"Type library file was not found: {typeLibraryPath}");
    }

    private static CapeOpenPreflightCheck CheckTypeLibraryIdentity(string typeLibraryPath)
    {
        if (!File.Exists(typeLibraryPath))
        {
            return Fail("type library identity", "Cannot inspect type library identity because the file does not exist.");
        }

        try
        {
            var identity = CapeOpenTypeLibraryInspector.Inspect(typeLibraryPath);
            var expectedGuid = Guid.Parse(RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.TypeLibraryId);
            var expectedVersion = CapeOpenTypeLibraryVersionParser.Parse(
                RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.TypeLibraryVersion);
            if (identity.Guid != expectedGuid)
            {
                return Fail(
                    "type library identity",
                    $"Type library GUID mismatch. Expected {expectedGuid:B}, actual {identity.Guid:B}.");
            }

            if (identity.Version != expectedVersion)
            {
                return Fail(
                    "type library identity",
                    $"Type library version mismatch. Expected {expectedVersion}, actual {identity.Version}.");
            }

            return Pass(
                "type library identity",
                $"Type library GUID/version match expected identity; lcid={identity.LocaleId}, syskind={identity.SysKind}.");
        }
        catch (Exception error) when (error is COMException or IOException or UnauthorizedAccessException or InvalidOperationException)
        {
            return Fail("type library identity", $"Failed to inspect type library: {error.Message}");
        }
    }

    private static IEnumerable<CapeOpenPreflightCheck> CheckRegistryConflicts(
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope)
    {
        var hive = CapeOpenRegistryHiveAccessor.OpenRoot(scope);
        var prefix = scope == CapeOpenRegistrationScope.CurrentUser
            ? "HKCU"
            : "HKLM";
        var keys = CapeOpenRegistryKeySet.CreateUnitOperationMvp();

        foreach (var keyPath in keys.AllTopLevelKeys)
        {
            var exists = CapeOpenRegistryHiveAccessor.RegistryKeyExists(hive, keyPath);
            if (action == CapeOpenRegistrationAction.Register)
            {
                yield return exists
                    ? Warn("registry conflict", $"{prefix}\\{keyPath} already exists and would need backup/review before registration.")
                    : Pass("registry conflict", $"{prefix}\\{keyPath} is currently absent.");
                continue;
            }

            yield return exists
                ? Pass("registry removal target", $"{prefix}\\{keyPath} exists and is a candidate for unregister.")
                : Warn("registry removal target", $"{prefix}\\{keyPath} is absent; unregister would be a no-op for this key.");
        }
    }

    private static CapeOpenPreflightCheck Pass(string name, string detail)
    {
        return new CapeOpenPreflightCheck(name, CapeOpenPreflightCheckStatus.Pass, detail);
    }

    private static CapeOpenPreflightCheck Warn(string name, string detail)
    {
        return new CapeOpenPreflightCheck(name, CapeOpenPreflightCheckStatus.Warning, detail);
    }

    private static CapeOpenPreflightCheck Fail(string name, string detail)
    {
        return new CapeOpenPreflightCheck(name, CapeOpenPreflightCheckStatus.Fail, detail);
    }
}

internal static class CapeOpenRegistrationElevationChecker
{
    public static bool IsProcessElevated()
    {
        if (!OperatingSystem.IsWindows())
        {
            return false;
        }

        using var identity = WindowsIdentity.GetCurrent();
        var principal = new WindowsPrincipal(identity);
        return principal.IsInRole(WindowsBuiltInRole.Administrator);
    }
}

internal sealed record CapeOpenRegistryKeySet(
    string ClassIdKey,
    string ProgIdKey,
    string VersionedProgIdKey,
    string TypeLibraryKey)
{
    public IReadOnlyList<string> AllTopLevelKeys => [ClassIdKey, ProgIdKey, VersionedProgIdKey, TypeLibraryKey];

    public static CapeOpenRegistryKeySet CreateUnitOperationMvp()
    {
        return new CapeOpenRegistryKeySet(
            ClassIdKey: $@"Software\Classes\CLSID\{{{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ClassId}}}",
            ProgIdKey: $@"Software\Classes\{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ProgId}",
            VersionedProgIdKey: $@"Software\Classes\{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.VersionedProgId}",
            TypeLibraryKey: $@"Software\Classes\TypeLib\{{{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.TypeLibraryId}}}");
    }
}

internal static class CapeOpenRegistryHiveAccessor
{
    public static RegistryKey OpenRoot(CapeOpenRegistrationScope scope)
    {
        return scope == CapeOpenRegistrationScope.CurrentUser
            ? Registry.CurrentUser
            : Registry.LocalMachine;
    }

    public static string GetHiveName(CapeOpenRegistrationScope scope)
    {
        return scope == CapeOpenRegistrationScope.CurrentUser
            ? "HKCU"
            : "HKLM";
    }

    public static bool RegistryKeyExists(
        RegistryKey hive,
        string keyPath)
    {
        using var key = hive.OpenSubKey(keyPath, writable: false);
        return key is not null;
    }
}

internal static class CapeOpenRegistryBackupPlanBuilder
{
    public static IReadOnlyList<CapeOpenRegistryBackupPlanEntry> BuildUnitOperationMvpPlan(
        CapeOpenRegistrationScope scope)
    {
        var hive = CapeOpenRegistryHiveAccessor.OpenRoot(scope);
        var hiveName = CapeOpenRegistryHiveAccessor.GetHiveName(scope);
        var keySet = CapeOpenRegistryKeySet.CreateUnitOperationMvp();

        return keySet.AllTopLevelKeys
            .Select(keyPath => new CapeOpenRegistryBackupPlanEntry(
                Hive: hiveName,
                KeyPath: keyPath,
                Exists: CapeOpenRegistryHiveAccessor.RegistryKeyExists(hive, keyPath),
                Reason: "Capture existing registration tree before any future write/delete operation."))
            .ToArray();
    }
}

internal static class CapeOpenRegistryPlanBuilder
{
    private static readonly string UnitOperationClassIdValue =
        $"{{{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ClassId}}}";

    public static IReadOnlyList<CapeOpenRegistryPlanEntry> BuildUnitOperationMvpPlan(
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope,
        string comHostPath,
        string typeLibraryPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(comHostPath);
        ArgumentException.ThrowIfNullOrWhiteSpace(typeLibraryPath);

        var hive = CapeOpenRegistryHiveAccessor.GetHiveName(scope);
        const string classRoot = @"Software\Classes";
        var clsidKey = $@"{classRoot}\CLSID\{{{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ClassId}}}";
        var progIdKey = $@"{classRoot}\{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ProgId}";
        var versionedProgIdKey = $@"{classRoot}\{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.VersionedProgId}";
        var typeLibraryKey = $@"{classRoot}\TypeLib\{{{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.TypeLibraryId}}}";
        var implementedCategoriesKey = $@"{clsidKey}\Implemented Categories";

        return action == CapeOpenRegistrationAction.Register
            ? CreateRegisterPlan(hive, clsidKey, progIdKey, versionedProgIdKey, typeLibraryKey, implementedCategoriesKey, comHostPath, typeLibraryPath)
            : CreateUnregisterPlan(hive, clsidKey, progIdKey, versionedProgIdKey, typeLibraryKey, typeLibraryPath);
    }

    private static IReadOnlyList<CapeOpenRegistryPlanEntry> CreateRegisterPlan(
        string hive,
        string clsidKey,
        string progIdKey,
        string versionedProgIdKey,
        string typeLibraryKey,
        string implementedCategoriesKey,
        string unresolvedComHostPath,
        string unresolvedTypeLibraryPath)
    {
        return
        [
            Verify(hive, $@"{clsidKey}\InprocServer32", "Resolve the generated .NET comhost DLL path before any real registry write."),
            RegisterTypeLibrary(hive, typeLibraryKey, unresolvedTypeLibraryPath, "Register the frozen COM type library required by late-bound IDispatch hosts."),
            SetDefaultValue(hive, clsidKey, RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ClassDisplayName, "Expose the COM class display name."),
            SetDefaultValue(hive, $@"{clsidKey}\InprocServer32", unresolvedComHostPath, "Host the .NET class through the generated native comhost DLL."),
            SetValue(
                hive,
                $@"{clsidKey}\InprocServer32",
                "ThreadingModel",
                RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ThreadingModel,
                "Declare the COM threading model expected by classic CAPE-OPEN hosts."),
            SetDefaultValue(
                hive,
                $@"{clsidKey}\ProgID",
                RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.VersionedProgId,
                "Bind CLSID to the versioned ProgID subkey expected by classic COM registration."),
            SetDefaultValue(
                hive,
                $@"{clsidKey}\VersionIndependentProgID",
                RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ProgId,
                "Bind CLSID to the version-independent ProgID subkey expected by classic COM registration."),
            SetDefaultValue(
                hive,
                $@"{clsidKey}\Version",
                RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ComVersion,
                "Expose the component version under the CLSID registration tree."),
            SetDefaultValue(
                hive,
                $@"{clsidKey}\TypeLib",
                $"{{{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.TypeLibraryId}}}",
                "Bind CLSID to the registered type library for classic late-bound COM hosts."),
            SetDefaultValue(
                hive,
                $@"{clsidKey}\Programmable",
                string.Empty,
                "Mark the COM class as programmable for legacy COM/CAPE-OPEN hosts."),
            SetValue(
                hive,
                $@"{clsidKey}\CapeDescription",
                "Name",
                RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.DisplayName,
                "Expose CAPE-OPEN display name metadata for host discovery UIs."),
            SetValue(
                hive,
                $@"{clsidKey}\CapeDescription",
                "Description",
                RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.Description,
                "Expose CAPE-OPEN description metadata for host discovery UIs."),
            SetValue(
                hive,
                $@"{clsidKey}\CapeDescription",
                "CapeVersion",
                RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.CapeVersion,
                "Expose the CAPE-OPEN version metadata used by host registries."),
            SetValue(
                hive,
                $@"{clsidKey}\CapeDescription",
                "ComponentVersion",
                RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ComponentVersion,
                "Expose the component version metadata used by host registries."),
            SetValue(
                hive,
                $@"{clsidKey}\CapeDescription",
                "VendorURL",
                RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.VendorUrl,
                "Expose vendor metadata for CAPE-OPEN host discovery UIs."),
            SetValue(
                hive,
                $@"{clsidKey}\CapeDescription",
                "About",
                RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.About,
                "Expose about-text metadata for CAPE-OPEN host discovery UIs."),
            SetDefaultValue(hive, progIdKey, RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ClassDisplayName, "Expose the stable ProgID display name."),
            SetDefaultValue(hive, $@"{progIdKey}\CLSID", UnitOperationClassIdValue, "Bind stable ProgID to CLSID."),
            SetDefaultValue(hive, $@"{progIdKey}\CurVer", RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.VersionedProgId, "Bind stable ProgID to its current versioned ProgID."),
            SetDefaultValue(hive, versionedProgIdKey, RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ClassDisplayName, "Expose the versioned ProgID display name."),
            SetDefaultValue(hive, $@"{versionedProgIdKey}\CLSID", UnitOperationClassIdValue, "Bind versioned ProgID to CLSID."),
            SetDefaultValue(hive, $@"{implementedCategoriesKey}\{{{RadishFlow.CapeOpen.Interop.Guids.CapeOpenCategoryIds.CapeOpenObject}}}", string.Empty, "Advertise CAPE-OPEN object category."),
            SetDefaultValue(hive, $@"{implementedCategoriesKey}\{{{RadishFlow.CapeOpen.Interop.Guids.CapeOpenCategoryIds.UnitOperation}}}", string.Empty, "Advertise CAPE-OPEN Unit Operation category."),
            SetDefaultValue(hive, $@"{implementedCategoriesKey}\{{{RadishFlow.CapeOpen.Interop.Guids.CapeOpenCategoryIds.ConsumesThermodynamics}}}", string.Empty, "Advertise CAPE-OPEN thermodynamics consumption for PME canvas compatibility."),
            SetDefaultValue(hive, $@"{implementedCategoriesKey}\{{{RadishFlow.CapeOpen.Interop.Guids.CapeOpenCategoryIds.SupportsThermodynamics10}}}", string.Empty, "Advertise CAPE-OPEN 1.0 thermodynamics compatibility for material-port PMEs."),
            SetDefaultValue(hive, $@"{implementedCategoriesKey}\{{{RadishFlow.CapeOpen.Interop.Guids.CapeOpenCategoryIds.SupportsThermodynamics11}}}", string.Empty, "Advertise CAPE-OPEN 1.1 thermodynamics compatibility for material-port PMEs."),
        ];
    }

    private static IReadOnlyList<CapeOpenRegistryPlanEntry> CreateUnregisterPlan(
        string hive,
        string clsidKey,
        string progIdKey,
        string versionedProgIdKey,
        string typeLibraryKey,
        string unresolvedTypeLibraryPath)
    {
        return
        [
            UnregisterTypeLibrary(hive, typeLibraryKey, unresolvedTypeLibraryPath, "Unregister the frozen COM type library used by late-bound CAPE-OPEN hosts."),
            DeleteTree(hive, clsidKey, "Remove the COM class registration tree."),
            DeleteTree(hive, progIdKey, "Remove the stable ProgID registration tree."),
            DeleteTree(hive, versionedProgIdKey, "Remove the versioned ProgID registration tree."),
            DeleteTree(hive, typeLibraryKey, "Remove the registered TypeLib tree after COM typelib unregistration."),
        ];
    }

    private static CapeOpenRegistryPlanEntry SetDefaultValue(
        string hive,
        string keyPath,
        string valueData,
        string reason)
    {
        return new CapeOpenRegistryPlanEntry(
            Operation: CapeOpenRegistryPlanOperation.SetValue,
            Hive: hive,
            KeyPath: keyPath,
            ValueName: null,
            ValueData: valueData,
            Reason: reason);
    }

    private static CapeOpenRegistryPlanEntry SetValue(
        string hive,
        string keyPath,
        string valueName,
        string valueData,
        string reason)
    {
        return new CapeOpenRegistryPlanEntry(
            Operation: CapeOpenRegistryPlanOperation.SetValue,
            Hive: hive,
            KeyPath: keyPath,
            ValueName: valueName,
            ValueData: valueData,
            Reason: reason);
    }

    private static CapeOpenRegistryPlanEntry DeleteTree(
        string hive,
        string keyPath,
        string reason)
    {
        return new CapeOpenRegistryPlanEntry(
            Operation: CapeOpenRegistryPlanOperation.DeleteTree,
            Hive: hive,
            KeyPath: keyPath,
            ValueName: null,
            ValueData: null,
            Reason: reason);
    }

    private static CapeOpenRegistryPlanEntry RegisterTypeLibrary(
        string hive,
        string keyPath,
        string typeLibraryPath,
        string reason)
    {
        return new CapeOpenRegistryPlanEntry(
            Operation: CapeOpenRegistryPlanOperation.RegisterTypeLibrary,
            Hive: hive,
            KeyPath: keyPath,
            ValueName: null,
            ValueData: typeLibraryPath,
            Reason: reason);
    }

    private static CapeOpenRegistryPlanEntry UnregisterTypeLibrary(
        string hive,
        string keyPath,
        string typeLibraryPath,
        string reason)
    {
        return new CapeOpenRegistryPlanEntry(
            Operation: CapeOpenRegistryPlanOperation.UnregisterTypeLibrary,
            Hive: hive,
            KeyPath: keyPath,
            ValueName: null,
            ValueData: typeLibraryPath,
            Reason: reason);
    }

    private static CapeOpenRegistryPlanEntry Verify(
        string hive,
        string keyPath,
        string reason)
    {
        return new CapeOpenRegistryPlanEntry(
            Operation: CapeOpenRegistryPlanOperation.Verify,
            Hive: hive,
            KeyPath: keyPath,
            ValueName: null,
            ValueData: null,
            Reason: reason);
    }
}

internal static class PortableExecutableArchitectureReader
{
    public static PortableExecutableMachineArchitecture ReadMachineArchitecture(string path)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(path);

        using var stream = File.OpenRead(path);
        using var reader = new BinaryReader(stream);

        if (stream.Length < 0x40)
        {
            throw new InvalidDataException("File is too small to contain a PE header.");
        }

        if (reader.ReadUInt16() != 0x5A4D)
        {
            throw new InvalidDataException("Missing MZ signature.");
        }

        stream.Position = 0x3C;
        var peHeaderOffset = reader.ReadInt32();
        if (peHeaderOffset <= 0 || peHeaderOffset + 6 > stream.Length)
        {
            throw new InvalidDataException("Invalid PE header offset.");
        }

        stream.Position = peHeaderOffset;
        if (reader.ReadUInt32() != 0x00004550)
        {
            throw new InvalidDataException("Missing PE signature.");
        }

        var machine = reader.ReadUInt16();
        return machine switch
        {
            0x014C => PortableExecutableMachineArchitecture.X86,
            0x8664 => PortableExecutableMachineArchitecture.X64,
            0xAA64 => PortableExecutableMachineArchitecture.Arm64,
            _ => PortableExecutableMachineArchitecture.Unknown,
        };
    }
}
