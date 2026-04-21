using System.Text.Json;
using System.Text.Json.Serialization;
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
                options.Scope);
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
    IReadOnlyList<CapeOpenRegistryPlanEntry> RegistryPlan,
    bool WritesRegistry,
    bool RequiresComRegistration,
    bool RequiresPmeAutomation,
    bool SupportsThirdPartyCapeOpenModels)
{
    public static CapeOpenRegistrationDescriptor CreateUnitOperationMvp(
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope)
    {
        var unitOperationType = typeof(RadishFlowCapeOpenUnitOperation);
        var registryPlan = CapeOpenRegistryPlanBuilder.BuildUnitOperationMvpPlan(action, scope);
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
            RegistryPlan: registryPlan,
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

internal sealed record CapeOpenRegistryPlanEntry(
    CapeOpenRegistryPlanOperation Operation,
    string Hive,
    string KeyPath,
    string? ValueName,
    string? ValueData,
    string Reason);

internal static class CapeOpenRegistryPlanBuilder
{
    public static IReadOnlyList<CapeOpenRegistryPlanEntry> BuildUnitOperationMvpPlan(
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope)
    {
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
        const string unresolvedComHostPath = "<resolved RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll path>";

        return action == CapeOpenRegistrationAction.Register
            ? CreateRegisterPlan(hive, clsidKey, progIdKey, versionedProgIdKey, implementedCategoriesKey, unresolvedComHostPath)
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
}

internal sealed class RegistrationPreflightOptions
{
    private RegistrationPreflightOptions(
        bool showHelp,
        bool json,
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope)
    {
        ShowHelp = showHelp;
        Json = json;
        Action = action;
        Scope = scope;
    }

    public bool ShowHelp { get; }

    public bool Json { get; }

    public CapeOpenRegistrationAction Action { get; }

    public CapeOpenRegistrationScope Scope { get; }

    public static string HelpText =>
        """
        RadishFlow.CapeOpen.Registration

        Prints the dry-run registration plan for the MVP CAPE-OPEN Unit Operation PMC.
        This tool does not write the registry, register COM classes, start a PME, or load third-party CAPE-OPEN models.

        Options:
          --action <register|unregister>           Dry-run action. Default: register
          --scope <current-user|local-machine>     Registry scope to plan. Default: current-user
          --json                                   Print descriptor as JSON
          --help                                   Show this help text
        """;

    public static RegistrationPreflightOptions Parse(string[] args)
    {
        var showHelp = false;
        var json = false;
        var action = CapeOpenRegistrationAction.Register;
        var scope = CapeOpenRegistrationScope.CurrentUser;

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

            throw new ArgumentException($"Unknown option `{arg}`.");
        }

        return new RegistrationPreflightOptions(showHelp, json, action, scope);
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
