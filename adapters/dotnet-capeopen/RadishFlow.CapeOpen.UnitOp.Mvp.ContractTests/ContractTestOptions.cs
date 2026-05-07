using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Guids;
using RadishFlow.CapeOpen.Interop.Ole;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Persistence;
using RadishFlow.CapeOpen.Interop.Thermo;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text.Json;

internal sealed class ContractTestOptions
{
    private ContractTestOptions(
        string projectPath,
        string packageId,
        string manifestPath,
        string payloadPath,
        string nativeLibraryDirectory,
        string testFilter)
    {
        ProjectPath = projectPath;
        PackageId = packageId;
        ManifestPath = manifestPath;
        PayloadPath = payloadPath;
        NativeLibraryDirectory = nativeLibraryDirectory;
        TestFilter = testFilter;
    }

    public string ProjectPath { get; }

    public string PackageId { get; }

    public string ManifestPath { get; }

    public string PayloadPath { get; }

    public string NativeLibraryDirectory { get; }

    public string TestFilter { get; }

    public static ContractTestOptions Parse(string[] args)
    {
        var repoRoot = ResolveRepositoryRoot();
        var values = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);

        for (var index = 0; index < args.Length; index++)
        {
            var current = args[index];
            if (!current.StartsWith("--", StringComparison.Ordinal))
            {
                throw new ArgumentException($"Unexpected argument `{current}`.");
            }

            if (index == args.Length - 1)
            {
                throw new ArgumentException($"Missing value for option `{current}`.");
            }

            values[current] = args[++index];
        }

        return new ContractTestOptions(
            projectPath: values.TryGetValue("--project", out var projectPath)
                ? Path.GetFullPath(projectPath)
                : Path.Combine(repoRoot, "examples", "flowsheets", "feed-heater-flash-binary-hydrocarbon.rfproj.json"),
            packageId: values.TryGetValue("--package", out var packageId)
                ? packageId
                : "binary-hydrocarbon-lite-v1",
            manifestPath: values.TryGetValue("--manifest", out var manifestPath)
                ? Path.GetFullPath(manifestPath)
                : Path.Combine(repoRoot, "examples", "sample-components", "property-packages", "binary-hydrocarbon-lite-v1", "manifest.json"),
            payloadPath: values.TryGetValue("--payload", out var payloadPath)
                ? Path.GetFullPath(payloadPath)
                : Path.Combine(repoRoot, "examples", "sample-components", "property-packages", "binary-hydrocarbon-lite-v1", "payload.rfpkg"),
            nativeLibraryDirectory: values.TryGetValue("--native-lib-dir", out var nativeLibraryDirectory)
                ? Path.GetFullPath(nativeLibraryDirectory)
                : Path.Combine(repoRoot, "target", "debug"),
            testFilter: values.TryGetValue("--test", out var testFilter)
                ? testFilter
                : "all");
    }

    private static string ResolveRepositoryRoot()
    {
        var current = new DirectoryInfo(AppContext.BaseDirectory);
        while (current is not null)
        {
            if (File.Exists(Path.Combine(current.FullName, "Cargo.toml")) &&
                Directory.Exists(Path.Combine(current.FullName, "adapters", "dotnet-capeopen")))
            {
                return current.FullName;
            }

            current = current.Parent;
        }

        throw new InvalidOperationException("Could not locate repository root from AppContext.BaseDirectory.");
    }
}
