using System.Security.Principal;
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

        var assemblyPath = componentType.Assembly.Location;
        if (string.IsNullOrWhiteSpace(assemblyPath))
        {
            return Path.GetFullPath("RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll");
        }

        var assemblyDirectory = Path.GetDirectoryName(assemblyPath) ?? Environment.CurrentDirectory;
        return Path.Combine(assemblyDirectory, "RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll");
    }
}

internal static class CapeOpenRegistrationPreflightChecker
{
    public static IReadOnlyList<CapeOpenPreflightCheck> Check(
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope,
        CapeOpenRegistrationExecutionMode executionMode,
        string comHostPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(comHostPath);

        var checks = new List<CapeOpenPreflightCheck>
        {
            CheckComHostPath(comHostPath),
            CheckComHostArchitecture(comHostPath),
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
    string VersionedProgIdKey)
{
    public IReadOnlyList<string> AllTopLevelKeys => [ClassIdKey, ProgIdKey, VersionedProgIdKey];

    public static CapeOpenRegistryKeySet CreateUnitOperationMvp()
    {
        return new CapeOpenRegistryKeySet(
            ClassIdKey: $@"Software\Classes\CLSID\{{{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ClassId}}}",
            ProgIdKey: $@"Software\Classes\{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ProgId}",
            VersionedProgIdKey: $@"Software\Classes\{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.VersionedProgId}");
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
    public static IReadOnlyList<CapeOpenRegistryPlanEntry> BuildUnitOperationMvpPlan(
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope,
        string comHostPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(comHostPath);

        var hive = CapeOpenRegistryHiveAccessor.GetHiveName(scope);
        const string classRoot = @"Software\Classes";
        var clsidKey = $@"{classRoot}\CLSID\{{{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ClassId}}}";
        var progIdKey = $@"{classRoot}\{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ProgId}";
        var versionedProgIdKey = $@"{classRoot}\{RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.VersionedProgId}";
        var implementedCategoriesKey = $@"{clsidKey}\Implemented Categories";

        return action == CapeOpenRegistrationAction.Register
            ? CreateRegisterPlan(hive, clsidKey, progIdKey, versionedProgIdKey, implementedCategoriesKey, comHostPath)
            : CreateUnregisterPlan(hive, clsidKey, progIdKey, versionedProgIdKey);
    }

    private static IReadOnlyList<CapeOpenRegistryPlanEntry> CreateRegisterPlan(
        string hive,
        string clsidKey,
        string progIdKey,
        string versionedProgIdKey,
        string implementedCategoriesKey,
        string unresolvedComHostPath)
    {
        return
        [
            Verify(hive, $@"{clsidKey}\InprocServer32", "Resolve the generated .NET comhost DLL path before any real registry write."),
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
            SetDefaultValue(hive, $@"{progIdKey}\CLSID", RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ClassId, "Bind stable ProgID to CLSID."),
            SetDefaultValue(hive, $@"{progIdKey}\CurVer", RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.VersionedProgId, "Bind stable ProgID to its current versioned ProgID."),
            SetDefaultValue(hive, versionedProgIdKey, RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ClassDisplayName, "Expose the versioned ProgID display name."),
            SetDefaultValue(hive, $@"{versionedProgIdKey}\CLSID", RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ClassId, "Bind versioned ProgID to CLSID."),
            SetDefaultValue(hive, $@"{implementedCategoriesKey}\{{{RadishFlow.CapeOpen.Interop.Guids.CapeOpenCategoryIds.CapeOpenObject}}}", string.Empty, "Advertise CAPE-OPEN object category."),
            SetDefaultValue(hive, $@"{implementedCategoriesKey}\{{{RadishFlow.CapeOpen.Interop.Guids.CapeOpenCategoryIds.UnitOperation}}}", string.Empty, "Advertise CAPE-OPEN Unit Operation category."),
        ];
    }

    private static IReadOnlyList<CapeOpenRegistryPlanEntry> CreateUnregisterPlan(
        string hive,
        string clsidKey,
        string progIdKey,
        string versionedProgIdKey)
    {
        return
        [
            DeleteTree(hive, clsidKey, "Remove the COM class registration tree."),
            DeleteTree(hive, progIdKey, "Remove the stable ProgID registration tree."),
            DeleteTree(hive, versionedProgIdKey, "Remove the versioned ProgID registration tree."),
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
