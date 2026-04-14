using RadishFlow.CapeOpen.Adapter;

var options = SmokeOptions.Parse(args);
if (options.ShowHelp)
{
    Console.WriteLine(SmokeOptions.HelpText);
    return;
}

try
{
    if (!string.IsNullOrWhiteSpace(options.NativeLibraryDirectory))
    {
        RfNativeLibraryLoader.ConfigureSearchDirectory(options.NativeLibraryDirectory);
    }

    using var engine = new RadishFlowNativeEngine();
    var projectJson = File.ReadAllText(options.ProjectPath);
    engine.LoadFlowsheetJson(projectJson);

    if (options.LoadPackageFiles)
    {
        engine.LoadPropertyPackageFiles(options.ManifestPath!, options.PayloadPath!);
    }

    Console.WriteLine("== Package List ==");
    Console.WriteLine(engine.GetPropertyPackageListJson());
    Console.WriteLine();

    engine.SolveFlowsheet(options.PackageId);

    Console.WriteLine("== Flowsheet Snapshot ==");
    Console.WriteLine(engine.GetFlowsheetSnapshotJson());
    Console.WriteLine();

    if (!string.IsNullOrWhiteSpace(options.StreamId))
    {
        Console.WriteLine($"== Stream Snapshot: {options.StreamId} ==");
        Console.WriteLine(engine.GetStreamSnapshotJson(options.StreamId));
        Console.WriteLine();
    }
}
catch (RadishFlowNativeException error)
{
    Console.Error.WriteLine($"Native operation failed: {error.Operation}");
    Console.Error.WriteLine($"Status: {error.Status}");
    if (!string.IsNullOrWhiteSpace(error.NativeMessage))
    {
        Console.Error.WriteLine($"Message: {error.NativeMessage}");
    }

    if (!string.IsNullOrWhiteSpace(error.NativeErrorJson))
    {
        Console.Error.WriteLine("Error Json:");
        Console.Error.WriteLine(error.NativeErrorJson);
    }

    Environment.ExitCode = 1;
}
catch (Exception error)
{
    Console.Error.WriteLine(error);
    Environment.ExitCode = 2;
}

file sealed class SmokeOptions
{
    private SmokeOptions(
        bool showHelp,
        string projectPath,
        string packageId,
        string? manifestPath,
        string? payloadPath,
        string? streamId,
        string? nativeLibraryDirectory)
    {
        ShowHelp = showHelp;
        ProjectPath = projectPath;
        PackageId = packageId;
        ManifestPath = manifestPath;
        PayloadPath = payloadPath;
        StreamId = streamId;
        NativeLibraryDirectory = nativeLibraryDirectory;
    }

    public bool ShowHelp { get; }

    public string ProjectPath { get; }

    public string PackageId { get; }

    public string? ManifestPath { get; }

    public string? PayloadPath { get; }

    public string? StreamId { get; }

    public string? NativeLibraryDirectory { get; }

    public bool LoadPackageFiles =>
        !string.IsNullOrWhiteSpace(ManifestPath) &&
        !string.IsNullOrWhiteSpace(PayloadPath);

    public static string HelpText =>
        """
        RadishFlow.CapeOpen.SmokeTests

        Options:
          --project <path>        Project json path. Default: examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json
          --package <id>          Package id to solve with. Default: binary-hydrocarbon-lite-v1
          --manifest <path>       Optional property package manifest path
          --payload <path>        Optional property package payload path
          --stream <id>           Optional stream id to export after solve. Default: stream-vapor
          --native-lib-dir <dir>  Optional directory that contains rf_ffi.dll
          --help                  Show this help text
        """;

    public static SmokeOptions Parse(string[] args)
    {
        var repoRoot = ResolveRepositoryRoot();
        var values = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
        var flags = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        for (var index = 0; index < args.Length; index++)
        {
            var current = args[index];
            if (string.Equals(current, "--help", StringComparison.OrdinalIgnoreCase))
            {
                flags.Add(current);
                continue;
            }

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

        return new SmokeOptions(
            showHelp: flags.Contains("--help"),
            projectPath: values.TryGetValue("--project", out var projectPath)
                ? Path.GetFullPath(projectPath)
                : Path.Combine(
                    repoRoot,
                    "examples",
                    "flowsheets",
                    "feed-heater-flash-binary-hydrocarbon.rfproj.json"),
            packageId: values.TryGetValue("--package", out var packageId)
                ? packageId
                : "binary-hydrocarbon-lite-v1",
            manifestPath: values.TryGetValue("--manifest", out var manifestPath)
                ? Path.GetFullPath(manifestPath)
                : Path.Combine(
                    repoRoot,
                    "examples",
                    "sample-components",
                    "property-packages",
                    "binary-hydrocarbon-lite-v1",
                    "manifest.json"),
            payloadPath: values.TryGetValue("--payload", out var payloadPath)
                ? Path.GetFullPath(payloadPath)
                : Path.Combine(
                    repoRoot,
                    "examples",
                    "sample-components",
                    "property-packages",
                    "binary-hydrocarbon-lite-v1",
                    "payload.rfpkg"),
            streamId: values.TryGetValue("--stream", out var streamId)
                ? streamId
                : "stream-vapor",
            nativeLibraryDirectory: values.TryGetValue("--native-lib-dir", out var nativeLibDir)
                ? Path.GetFullPath(nativeLibDir)
                : Path.Combine(repoRoot, "target", "debug"));
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
