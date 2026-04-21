using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.Win32;
using RadishFlow.CapeOpen.Interop.Guids;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

Environment.ExitCode = RegistrationPreflightExecutable.Run(args);

internal static class RegistrationPreflightExecutable
{
    public static int Run(string[] args)
    {
        try
        {
            var options = RegistrationPreflightOptions.Parse(args);
            if (options.ShowHelp)
            {
                Console.WriteLine(RegistrationPreflightOptions.HelpText);
                return 0;
            }

            var descriptor = CapeOpenRegistrationDescriptor.CreateUnitOperationMvp(
                options.Action,
                options.Scope,
                options.ComHostPath);
            if (options.Json)
            {
                Console.WriteLine(JsonSerializer.Serialize(
                    descriptor,
                    new JsonSerializerOptions
                    {
                        WriteIndented = true,
                        Converters = { new JsonStringEnumConverter() },
                    }));
            }
            else
            {
                Console.WriteLine(CapeOpenRegistrationPlanFormatter.Format(descriptor));
            }

            return 0;
        }
        catch (Exception error)
        {
            Console.Error.WriteLine("Registration preflight failed.");
            Console.Error.WriteLine(error.Message);
            return 1;
        }
    }
}

internal sealed record CapeOpenRegistrationDescriptor(
    string ComponentName,
    string Description,
    string ClassId,
    string ProgId,
    string VersionedProgId,
    string AssemblyName,
    string TypeName,
    CapeOpenRegistrationAction Action,
    CapeOpenRegistrationScope Scope,
    IReadOnlyList<CapeOpenRegistrationCategory> Categories,
    IReadOnlyList<CapeOpenImplementedInterface> ImplementedInterfaces,
    IReadOnlyList<CapeOpenPreflightCheck> PreflightChecks,
    IReadOnlyList<CapeOpenRegistryPlanEntry> RegistryPlan,
    IReadOnlyList<CapeOpenRegistryBackupPlanEntry> BackupPlan,
    bool WritesRegistry,
    bool RequiresComRegistration,
    bool RequiresPmeAutomation,
    bool SupportsThirdPartyCapeOpenModels)
{
    public static CapeOpenRegistrationDescriptor CreateUnitOperationMvp(
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope,
        string? comHostPath)
    {
        var unitOperationType = typeof(RadishFlowCapeOpenUnitOperation);
        var resolvedComHostPath = CapeOpenComHostPathResolver.Resolve(unitOperationType, comHostPath);
        var preflightChecks = CapeOpenRegistrationPreflightChecker.Check(
            action,
            scope,
            resolvedComHostPath);
        var registryPlan = CapeOpenRegistryPlanBuilder.BuildUnitOperationMvpPlan(
            action,
            scope,
            resolvedComHostPath);
        var backupPlan = CapeOpenRegistryBackupPlanBuilder.BuildUnitOperationMvpPlan(scope);
        return new CapeOpenRegistrationDescriptor(
            ComponentName: UnitOperationComIdentity.DisplayName,
            Description: UnitOperationComIdentity.Description,
            ClassId: UnitOperationComIdentity.ClassId,
            ProgId: UnitOperationComIdentity.ProgId,
            VersionedProgId: UnitOperationComIdentity.VersionedProgId,
            AssemblyName: unitOperationType.Assembly.GetName().Name ?? "RadishFlow.CapeOpen.UnitOp.Mvp",
            TypeName: unitOperationType.FullName ?? unitOperationType.Name,
            Action: action,
            Scope: scope,
            Categories:
            [
                new CapeOpenRegistrationCategory(
                    Name: "CAPE-OPEN Object",
                    CategoryId: CapeOpenCategoryIds.CapeOpenObject),
                new CapeOpenRegistrationCategory(
                    Name: "CAPE-OPEN Unit Operation",
                    CategoryId: CapeOpenCategoryIds.UnitOperation),
            ],
            ImplementedInterfaces:
            [
                new CapeOpenImplementedInterface(
                    Name: "ICapeIdentification",
                    InterfaceId: CapeOpenInterfaceIds.ICapeIdentification),
                new CapeOpenImplementedInterface(
                    Name: "ICapeUtilities",
                    InterfaceId: CapeOpenInterfaceIds.ICapeUtilities),
                new CapeOpenImplementedInterface(
                    Name: "ICapeUnit",
                    InterfaceId: CapeOpenInterfaceIds.ICapeUnit),
            ],
            PreflightChecks: preflightChecks,
            RegistryPlan: registryPlan,
            BackupPlan: backupPlan,
            WritesRegistry: false,
            RequiresComRegistration: false,
            RequiresPmeAutomation: false,
            SupportsThirdPartyCapeOpenModels: false);
    }
}

internal sealed record CapeOpenRegistrationCategory(
    string Name,
    string CategoryId);

internal sealed record CapeOpenImplementedInterface(
    string Name,
    string InterfaceId);

internal sealed record CapeOpenPreflightCheck(
    string Name,
    CapeOpenPreflightCheckStatus Status,
    string Detail);

internal sealed record CapeOpenRegistryPlanEntry(
    CapeOpenRegistryPlanOperation Operation,
    string Hive,
    string KeyPath,
    string? ValueName,
    string? ValueData,
    string Reason);

internal sealed record CapeOpenRegistryBackupPlanEntry(
    string Hive,
    string KeyPath,
    bool Exists,
    string Reason);

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
        string comHostPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(comHostPath);

        var checks = new List<CapeOpenPreflightCheck>
        {
            CheckComHostPath(comHostPath),
            CheckComHostArchitecture(comHostPath),
            CheckProcessArchitecture(),
            CheckScopePermission(scope),
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

    private static CapeOpenPreflightCheck CheckScopePermission(CapeOpenRegistrationScope scope)
    {
        return scope == CapeOpenRegistrationScope.CurrentUser
            ? Pass("scope permission", "Current-user dry-run targets HKCU and does not require elevation.")
            : Warn("scope permission", "Local-machine registration will require elevation before any real HKLM write.");
    }

    private static IEnumerable<CapeOpenPreflightCheck> CheckRegistryConflicts(
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope)
    {
        var hive = scope == CapeOpenRegistrationScope.CurrentUser
            ? Registry.CurrentUser
            : Registry.LocalMachine;
        var prefix = scope == CapeOpenRegistrationScope.CurrentUser
            ? "HKCU"
            : "HKLM";
        var keys = CapeOpenRegistryKeySet.CreateUnitOperationMvp();

        foreach (var keyPath in keys.AllTopLevelKeys)
        {
            var exists = RegistryKeyExists(hive, keyPath);
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

    private static bool RegistryKeyExists(
        RegistryKey hive,
        string keyPath)
    {
        using var key = hive.OpenSubKey(keyPath, writable: false);
        return key is not null;
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

internal sealed record CapeOpenRegistryKeySet(
    string ClassIdKey,
    string ProgIdKey,
    string VersionedProgIdKey)
{
    public IReadOnlyList<string> AllTopLevelKeys => [ClassIdKey, ProgIdKey, VersionedProgIdKey];

    public static CapeOpenRegistryKeySet CreateUnitOperationMvp()
    {
        return new CapeOpenRegistryKeySet(
            ClassIdKey: $@"Software\Classes\CLSID\{{{UnitOperationComIdentity.ClassId}}}",
            ProgIdKey: $@"Software\Classes\{UnitOperationComIdentity.ProgId}",
            VersionedProgIdKey: $@"Software\Classes\{UnitOperationComIdentity.VersionedProgId}");
    }
}

internal static class CapeOpenRegistryBackupPlanBuilder
{
    public static IReadOnlyList<CapeOpenRegistryBackupPlanEntry> BuildUnitOperationMvpPlan(
        CapeOpenRegistrationScope scope)
    {
        var hive = scope == CapeOpenRegistrationScope.CurrentUser
            ? Registry.CurrentUser
            : Registry.LocalMachine;
        var hiveName = scope == CapeOpenRegistrationScope.CurrentUser
            ? "HKCU"
            : "HKLM";
        var keySet = CapeOpenRegistryKeySet.CreateUnitOperationMvp();

        return keySet.AllTopLevelKeys
            .Select(keyPath => new CapeOpenRegistryBackupPlanEntry(
                Hive: hiveName,
                KeyPath: keyPath,
                Exists: RegistryKeyExists(hive, keyPath),
                Reason: "Capture existing registration tree before any future write/delete operation."))
            .ToArray();
    }

    private static bool RegistryKeyExists(
        RegistryKey hive,
        string keyPath)
    {
        using var key = hive.OpenSubKey(keyPath, writable: false);
        return key is not null;
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

        var hive = scope == CapeOpenRegistrationScope.CurrentUser
            ? "HKCU"
            : "HKLM";
        var classRoot = scope == CapeOpenRegistrationScope.CurrentUser
            ? @"Software\Classes"
            : @"Software\Classes";
        var clsidKey = $@"{classRoot}\CLSID\{{{UnitOperationComIdentity.ClassId}}}";
        var progIdKey = $@"{classRoot}\{UnitOperationComIdentity.ProgId}";
        var versionedProgIdKey = $@"{classRoot}\{UnitOperationComIdentity.VersionedProgId}";
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
            SetDefaultValue(hive, clsidKey, UnitOperationComIdentity.DisplayName, "Expose the COM class display name."),
            SetValue(hive, clsidKey, "ProgID", UnitOperationComIdentity.ProgId, "Bind CLSID to the stable ProgID."),
            SetValue(hive, clsidKey, "VersionIndependentProgID", UnitOperationComIdentity.ProgId, "Bind CLSID to the version-independent ProgID."),
            SetValue(hive, clsidKey, "VersionedProgID", UnitOperationComIdentity.VersionedProgId, "Bind CLSID to the current versioned ProgID."),
            SetDefaultValue(hive, $@"{clsidKey}\InprocServer32", unresolvedComHostPath, "Host the .NET class through the generated native comhost DLL."),
            SetDefaultValue(hive, progIdKey, UnitOperationComIdentity.DisplayName, "Expose the stable ProgID display name."),
            SetValue(hive, progIdKey, "CLSID", UnitOperationComIdentity.ClassId, "Bind stable ProgID to CLSID."),
            SetDefaultValue(hive, versionedProgIdKey, UnitOperationComIdentity.DisplayName, "Expose the versioned ProgID display name."),
            SetValue(hive, versionedProgIdKey, "CLSID", UnitOperationComIdentity.ClassId, "Bind versioned ProgID to CLSID."),
            SetDefaultValue(hive, $@"{implementedCategoriesKey}\{{{CapeOpenCategoryIds.CapeOpenObject}}}", string.Empty, "Advertise CAPE-OPEN object category."),
            SetDefaultValue(hive, $@"{implementedCategoriesKey}\{{{CapeOpenCategoryIds.UnitOperation}}}", string.Empty, "Advertise CAPE-OPEN Unit Operation category."),
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

internal static class CapeOpenRegistrationPlanFormatter
{
    public static string Format(CapeOpenRegistrationDescriptor descriptor)
    {
        ArgumentNullException.ThrowIfNull(descriptor);

        var lines = new List<string>
        {
            "RadishFlow CAPE-OPEN registration preflight",
            string.Empty,
            "Component:",
            $"  Name: {descriptor.ComponentName}",
            $"  Description: {descriptor.Description}",
            $"  CLSID: {descriptor.ClassId}",
            $"  ProgID: {descriptor.ProgId}",
            $"  Versioned ProgID: {descriptor.VersionedProgId}",
            $"  Assembly: {descriptor.AssemblyName}",
            $"  Type: {descriptor.TypeName}",
            $"  Action: {descriptor.Action}",
            $"  Scope: {descriptor.Scope}",
            string.Empty,
            "CAPE-OPEN Categories:",
        };

        lines.AddRange(descriptor.Categories.Select(
            category => $"  - {category.Name}: {category.CategoryId}"));

        lines.Add(string.Empty);
        lines.Add("Implemented Interfaces:");
        lines.AddRange(descriptor.ImplementedInterfaces.Select(
            implementedInterface => $"  - {implementedInterface.Name}: {implementedInterface.InterfaceId}"));

        lines.Add(string.Empty);
        lines.Add("Preflight checks:");
        lines.AddRange(descriptor.PreflightChecks.Select(FormatPreflightCheck));

        lines.Add(string.Empty);
        lines.Add("Backup plan:");
        lines.AddRange(descriptor.BackupPlan.Select(FormatBackupPlanEntry));

        lines.Add(string.Empty);
        lines.Add("Dry-run registry plan:");
        lines.AddRange(descriptor.RegistryPlan.Select(FormatRegistryPlanEntry));

        lines.Add(string.Empty);
        lines.Add("Boundary:");
        lines.Add($"  Writes registry: {FormatBoolean(descriptor.WritesRegistry)}");
        lines.Add($"  Requires COM registration now: {FormatBoolean(descriptor.RequiresComRegistration)}");
        lines.Add($"  Requires PME automation now: {FormatBoolean(descriptor.RequiresPmeAutomation)}");
        lines.Add($"  Supports third-party CAPE-OPEN models: {FormatBoolean(descriptor.SupportsThirdPartyCapeOpenModels)}");

        return string.Join(Environment.NewLine, lines);
    }

    private static string FormatBoolean(bool value)
    {
        return value ? "yes" : "no";
    }

    private static string FormatRegistryPlanEntry(CapeOpenRegistryPlanEntry entry)
    {
        var valueName = entry.ValueName is null ? "(Default)" : entry.ValueName;
        var valueData = entry.ValueData is null ? string.Empty : $" = {entry.ValueData}";
        return entry.Operation == CapeOpenRegistryPlanOperation.SetValue
            ? $"  - Set {entry.Hive}\\{entry.KeyPath}\\{valueName}{valueData} | {entry.Reason}"
            : entry.Operation == CapeOpenRegistryPlanOperation.DeleteTree
                ? $"  - DeleteTree {entry.Hive}\\{entry.KeyPath} | {entry.Reason}"
                : $"  - Verify {entry.Hive}\\{entry.KeyPath} | {entry.Reason}";
    }

    private static string FormatPreflightCheck(CapeOpenPreflightCheck check)
    {
        return $"  - {check.Status}: {check.Name} | {check.Detail}";
    }

    private static string FormatBackupPlanEntry(CapeOpenRegistryBackupPlanEntry entry)
    {
        var state = entry.Exists ? "exists" : "absent";
        return $"  - {entry.Hive}\\{entry.KeyPath}: {state} | {entry.Reason}";
    }
}

internal sealed class RegistrationPreflightOptions
{
    private RegistrationPreflightOptions(
        bool showHelp,
        bool json,
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope,
        string? comHostPath)
    {
        ShowHelp = showHelp;
        Json = json;
        Action = action;
        Scope = scope;
        ComHostPath = comHostPath;
    }

    public bool ShowHelp { get; }

    public bool Json { get; }

    public CapeOpenRegistrationAction Action { get; }

    public CapeOpenRegistrationScope Scope { get; }

    public string? ComHostPath { get; }

    public static string HelpText =>
        """
        RadishFlow.CapeOpen.Registration

        Prints the dry-run registration plan for the MVP CAPE-OPEN Unit Operation PMC.
        This tool does not write the registry, register COM classes, start a PME, or load third-party CAPE-OPEN models.

        Options:
          --action <register|unregister>           Dry-run action. Default: register
          --scope <current-user|local-machine>     Registry scope to plan. Default: current-user
          --comhost <path>                         Optional explicit RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll path
          --json                                   Print descriptor as JSON
          --help                                   Show this help text
        """;

    public static RegistrationPreflightOptions Parse(string[] args)
    {
        var showHelp = false;
        var json = false;
        var action = CapeOpenRegistrationAction.Register;
        var scope = CapeOpenRegistrationScope.CurrentUser;
        string? comHostPath = null;

        for (var index = 0; index < args.Length; index++)
        {
            var arg = args[index];
            if (string.Equals(arg, "--help", StringComparison.OrdinalIgnoreCase))
            {
                showHelp = true;
                continue;
            }

            if (string.Equals(arg, "--json", StringComparison.OrdinalIgnoreCase))
            {
                json = true;
                continue;
            }

            if (string.Equals(arg, "--action", StringComparison.OrdinalIgnoreCase))
            {
                action = ParseAction(ReadOptionValue(args, ref index, arg));
                continue;
            }

            if (string.Equals(arg, "--scope", StringComparison.OrdinalIgnoreCase))
            {
                scope = ParseScope(ReadOptionValue(args, ref index, arg));
                continue;
            }

            if (string.Equals(arg, "--comhost", StringComparison.OrdinalIgnoreCase))
            {
                comHostPath = Path.GetFullPath(ReadOptionValue(args, ref index, arg));
                continue;
            }

            throw new ArgumentException($"Unknown option `{arg}`.");
        }

        return new RegistrationPreflightOptions(showHelp, json, action, scope, comHostPath);
    }

    private static string ReadOptionValue(
        string[] args,
        ref int index,
        string option)
    {
        if (index == args.Length - 1)
        {
            throw new ArgumentException($"Missing value for option `{option}`.");
        }

        return args[++index];
    }

    private static CapeOpenRegistrationAction ParseAction(string value)
    {
        return value.ToLowerInvariant() switch
        {
            "register" => CapeOpenRegistrationAction.Register,
            "unregister" => CapeOpenRegistrationAction.Unregister,
            _ => throw new ArgumentException($"Unknown registration action `{value}`."),
        };
    }

    private static CapeOpenRegistrationScope ParseScope(string value)
    {
        return value.ToLowerInvariant() switch
        {
            "current-user" => CapeOpenRegistrationScope.CurrentUser,
            "local-machine" => CapeOpenRegistrationScope.LocalMachine,
            _ => throw new ArgumentException($"Unknown registration scope `{value}`."),
        };
    }
}

internal enum CapeOpenRegistrationAction
{
    Register,
    Unregister,
}

internal enum CapeOpenRegistrationScope
{
    CurrentUser,
    LocalMachine,
}

internal enum CapeOpenRegistryPlanOperation
{
    Verify,
    SetValue,
    DeleteTree,
}

internal enum CapeOpenPreflightCheckStatus
{
    Pass,
    Warning,
    Fail,
}

internal enum PortableExecutableMachineArchitecture
{
    Unknown,
    X86,
    X64,
    Arm64,
}
