using RadishFlow.CapeOpen.Adapter;
using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;

var options = SmokeOptions.Parse(args);
if (options.ShowHelp)
{
    Console.WriteLine(SmokeOptions.HelpText);
    return;
}

try
{
    if (options.Mode == SmokeMode.UnitOperation)
    {
        RunUnitOperationSmoke(options);
    }
    else
    {
        RunAdapterSmoke(options);
    }
}
catch (CapeOpenException error)
{
    Console.Error.WriteLine($"CAPE-OPEN operation failed: {error.Operation}");
    if (!string.IsNullOrWhiteSpace(error.NativeStatus))
    {
        Console.Error.WriteLine($"Native Status: {error.NativeStatus}");
    }

    Console.Error.WriteLine($"Message: {error.Message}");

    if (!string.IsNullOrWhiteSpace(error.DiagnosticJson))
    {
        Console.Error.WriteLine("Error Json:");
        Console.Error.WriteLine(error.DiagnosticJson);
    }

    Environment.ExitCode = 1;
}
catch (Exception error)
{
    Console.Error.WriteLine(error);
    Environment.ExitCode = 2;
}

static void RunAdapterSmoke(SmokeOptions options)
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

static void RunUnitOperationSmoke(SmokeOptions options)
{
    var projectJson = File.ReadAllText(options.ProjectPath);
    UnitOperationSmokeBoundarySuite.Run(options, projectJson);
    RunUnitOperationSessionSmoke(options, projectJson);
    RunUnitOperationRecoverySessionSmoke(options, projectJson);
    RunUnitOperationShutdownSessionSmoke(options, projectJson);
}

static void RunUnitOperationSessionSmoke(SmokeOptions options, string projectJson)
{
    using var session = new UnitOperationSmokeSession(options, projectJson);
    session.ExpectInvocationOrderBeforeInitialize("round-0");
    session.InitializeAndExpectIdle("round-1");
    session.ConfigureMinimumInputsAndConnect("round-2");
    session.ExpectSuccessRound(
        "round-3",
        report => $"status={report.Snapshot.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Status)}, diagnostics={report.Snapshot.GetDetailValue(UnitOperationCalculationReportDetailCatalog.DiagnosticCount)}");
    session.ExpectNativeFailureForMissingPackage("round-4", "missing-package-for-session");
    session.RestorePackageAndExpectValid("round-5a", options.PackageId);
    session.ExpectSuccessRound(
        "round-5b",
        report => $"state={report.Snapshot.State}, highestSeverity={report.Snapshot.GetDetailValue(UnitOperationCalculationReportDetailCatalog.HighestSeverity)}");
    session.BreakCompanionInputsAndExpectValidationFailure("round-6");
    session.RestoreMinimumInputsAndExpectValid("round-7a");
    session.ExpectSuccessRound(
        "round-7b",
        report => $"relatedStreams={report.Snapshot.GetDetailValue(UnitOperationCalculationReportDetailCatalog.RelatedStreamIds)}");
    session.DisconnectProductPortAndExpectRecoveryWindow("round-8a");
    session.ReconnectProductPort("round-8b", "Session Product");
    session.ExpectSuccessRound(
        "round-8c",
        report => $"headline={report.Snapshot.Headline}");
    session.TerminateAndExpectClosed("round-9");

    Console.WriteLine("== Host Session Timeline ==");
    foreach (var line in session.Timeline)
    {
        Console.WriteLine($"- {line}");
    }
    Console.WriteLine();
}

static void RunUnitOperationRecoverySessionSmoke(SmokeOptions options, string projectJson)
{
    using var session = new UnitOperationSmokeSession(options, projectJson);
    session.InitializeAndExpectIdle("recovery-0");
    session.ConfigureMinimumInputsAndConnect("recovery-1");
    session.BreakCompanionInputsAndExpectValidationFailure("recovery-2");
    session.RestoreMinimumInputsAndExpectValid("recovery-3");
    session.ExpectSuccessRound(
        "recovery-4",
        report => $"headline={report.Snapshot.Headline}");
    session.DisconnectFeedPortAndExpectRecoveryWindow("recovery-5");
    session.ReconnectFeedPort("recovery-6", "Recovery Feed");
    session.ExpectSuccessRound(
        "recovery-7",
        report => $"diagnosticCount={report.Snapshot.GetDetailValue(UnitOperationCalculationReportDetailCatalog.DiagnosticCount)}");
    session.ExpectNativeFailureForMissingPackage("recovery-8", "missing-package-for-recovery");
    session.RestorePackageAndExpectValid("recovery-9", options.PackageId);
    session.ExpectSuccessRound(
        "recovery-10",
        report => $"relatedUnits={report.Snapshot.GetDetailValue(UnitOperationCalculationReportDetailCatalog.RelatedUnitIds)}");
    session.TerminateAndExpectClosed("recovery-11");

    Console.WriteLine("== Host Recovery Timeline ==");
    foreach (var line in session.Timeline)
    {
        Console.WriteLine($"- {line}");
    }
    Console.WriteLine();
}

static void RunUnitOperationShutdownSessionSmoke(SmokeOptions options, string projectJson)
{
    using var session = new UnitOperationSmokeSession(options, projectJson);
    session.ExpectCurrentReportToBeEmpty("shutdown-0");
    session.InitializeAndExpectIdle("shutdown-1");
    session.ConfigureMinimumInputsAndConnect("shutdown-2");
    session.ExpectSuccessRound(
        "shutdown-3",
        report => $"headline={report.Snapshot.Headline}");
    session.ExpectCurrentReportToBeSuccessful(
        "shutdown-4",
        report => $"detailKeys={report.Snapshot.DetailKeyCount}, supplementalLines={report.Presentation.SupplementalLines.Count}");
    session.TerminateAndExpectClosed("shutdown-5");
    session.ExpectCurrentReportToBeEmpty("shutdown-6");
    session.ExpectPostTerminateCalculationFailure("shutdown-7");

    Console.WriteLine("== Host Shutdown Timeline ==");
    foreach (var line in session.Timeline)
    {
        Console.WriteLine($"- {line}");
    }
    Console.WriteLine();
}


internal sealed class SmokeOptions
{
    private SmokeOptions(
        bool showHelp,
        SmokeMode mode,
        string projectPath,
        string packageId,
        string? manifestPath,
        string? payloadPath,
        string? streamId,
        string? nativeLibraryDirectory)
    {
        ShowHelp = showHelp;
        Mode = mode;
        ProjectPath = projectPath;
        PackageId = packageId;
        ManifestPath = manifestPath;
        PayloadPath = payloadPath;
        StreamId = streamId;
        NativeLibraryDirectory = nativeLibraryDirectory;
    }

    public bool ShowHelp { get; }

    public SmokeMode Mode { get; }

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
          --mode <adapter|unitop> Run direct Adapter smoke or UnitOp.Mvp smoke. Default: adapter
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
            mode: values.TryGetValue("--mode", out var modeText)
                ? ParseMode(modeText)
                : SmokeMode.Adapter,
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

    private static SmokeMode ParseMode(string value)
    {
        return value.Trim().ToLowerInvariant() switch
        {
            "adapter" => SmokeMode.Adapter,
            "unitop" => SmokeMode.UnitOperation,
            _ => throw new ArgumentException($"Unsupported smoke mode `{value}`."),
        };
    }
}

internal enum SmokeMode
{
    Adapter,
    UnitOperation,
}

internal sealed class SmokeConnectedObject : ICapeIdentification
{
    public SmokeConnectedObject(string componentName)
    {
        ComponentName = componentName;
        ComponentDescription = "Smoke test placeholder connected object.";
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }
}

internal sealed class InvalidSmokeConnectedObject : ICapeIdentification
{
    public InvalidSmokeConnectedObject(string componentName)
    {
        ComponentName = componentName;
        ComponentDescription = "Smoke test invalid placeholder connected object.";
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }
}
