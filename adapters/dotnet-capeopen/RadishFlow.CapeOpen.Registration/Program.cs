using System.Text.Json;
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

            var descriptor = CapeOpenRegistrationDescriptor.CreateUnitOperationMvp();
            if (options.Json)
            {
                Console.WriteLine(JsonSerializer.Serialize(
                    descriptor,
                    new JsonSerializerOptions { WriteIndented = true }));
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
    IReadOnlyList<CapeOpenRegistrationCategory> Categories,
    IReadOnlyList<CapeOpenImplementedInterface> ImplementedInterfaces,
    bool WritesRegistry,
    bool RequiresComRegistration,
    bool RequiresPmeAutomation,
    bool SupportsThirdPartyCapeOpenModels)
{
    public static CapeOpenRegistrationDescriptor CreateUnitOperationMvp()
    {
        var unitOperationType = typeof(RadishFlowCapeOpenUnitOperation);
        return new CapeOpenRegistrationDescriptor(
            ComponentName: UnitOperationComIdentity.DisplayName,
            Description: UnitOperationComIdentity.Description,
            ClassId: UnitOperationComIdentity.ClassId,
            ProgId: UnitOperationComIdentity.ProgId,
            VersionedProgId: UnitOperationComIdentity.VersionedProgId,
            AssemblyName: unitOperationType.Assembly.GetName().Name ?? "RadishFlow.CapeOpen.UnitOp.Mvp",
            TypeName: unitOperationType.FullName ?? unitOperationType.Name,
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
}

internal sealed class RegistrationPreflightOptions
{
    private RegistrationPreflightOptions(bool showHelp, bool json)
    {
        ShowHelp = showHelp;
        Json = json;
    }

    public bool ShowHelp { get; }

    public bool Json { get; }

    public static string HelpText =>
        """
        RadishFlow.CapeOpen.Registration

        Prints the dry-run registration descriptor for the MVP CAPE-OPEN Unit Operation PMC.
        This tool does not write the registry, register COM classes, start a PME, or load third-party CAPE-OPEN models.

        Options:
          --json   Print descriptor as JSON
          --help   Show this help text
        """;

    public static RegistrationPreflightOptions Parse(string[] args)
    {
        var showHelp = false;
        var json = false;
        foreach (var arg in args)
        {
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

            throw new ArgumentException($"Unknown option `{arg}`.");
        }

        return new RegistrationPreflightOptions(showHelp, json);
    }
}
