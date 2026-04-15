using RadishFlow.CapeOpen.Adapter;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

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
    using var unitOperation = new RadishFlowCapeOpenUnitOperation();
    if (!string.IsNullOrWhiteSpace(options.NativeLibraryDirectory))
    {
        unitOperation.ConfigureNativeLibraryDirectory(options.NativeLibraryDirectory);
    }

    var projectJson = File.ReadAllText(options.ProjectPath);
    unitOperation.Initialize();

    var parameters = unitOperation.Parameters;
    var ports = unitOperation.Ports;
    var parameterCollection = (ICapeCollection)parameters;
    var portCollection = (ICapeCollection)ports;
    Console.WriteLine("== Unit Collections ==");
    Console.WriteLine($"Parameters.Count(): {parameterCollection.Count()}");
    Console.WriteLine($"Ports.Count(): {portCollection.Count()}");
    Console.WriteLine();

    var flowsheetParameter = (UnitOperationParameterPlaceholder)parameterCollection.Item("Flowsheet Json");
    var packageIdParameter = (UnitOperationParameterPlaceholder)parameterCollection.Item("Property Package Id");
    var manifestPathParameter = (UnitOperationParameterPlaceholder)parameterCollection.Item("Property Package Manifest Path");
    var payloadPathParameter = (UnitOperationParameterPlaceholder)parameterCollection.Item(4);
    var feedPort = (UnitOperationPortPlaceholder)portCollection.Item("Feed");
    var productPort = (UnitOperationPortPlaceholder)portCollection.Item(2);

    EnsureSameReference(parameters[0], flowsheetParameter, "parameter collection name lookup");
    EnsureSameReference(ports[0], feedPort, "port collection name lookup");
    EnsureCondition(flowsheetParameter.ValueKind == UnitOperationParameterValueKind.StructuredJsonText, "flowsheet parameter should expose structured JSON metadata.");
    EnsureCondition(packageIdParameter.ValueKind == UnitOperationParameterValueKind.Identifier, "package parameter should expose identifier metadata.");
    EnsureCondition(manifestPathParameter.ValueKind == UnitOperationParameterValueKind.FilePath, "manifest parameter should expose file path metadata.");
    EnsureCondition(!flowsheetParameter.AllowsEmptyValue, "flowsheet parameter should not allow empty text.");
    EnsureCondition(
        string.Equals(
            manifestPathParameter.RequiredCompanionParameterName,
            payloadPathParameter.ComponentName,
            StringComparison.Ordinal),
        "manifest parameter should declare payload companion metadata.");
    EnsureCondition(
        string.Equals(
            payloadPathParameter.RequiredCompanionParameterName,
            manifestPathParameter.ComponentName,
            StringComparison.Ordinal),
        "payload parameter should declare manifest companion metadata.");

    var invalidJsonMessage = string.Empty;
    flowsheetParameter.value = "{ invalid json";
    EnsureCondition(
        !flowsheetParameter.Validate(ref invalidJsonMessage),
        "flowsheet parameter should reject invalid JSON text.");
    EnsureCondition(
        invalidJsonMessage.Contains("valid JSON text", StringComparison.Ordinal),
        "invalid JSON validation should mention JSON text.");
    ExpectCapeInvalidArgument(() => feedPort.Connect(new object()), "port connect with plain object");
    ExpectCapeInvalidArgument(
        () => feedPort.Connect(new InvalidSmokeConnectedObject("   ")),
        "port connect with blank ComponentName");

    flowsheetParameter.value = projectJson;

    if (options.LoadPackageFiles)
    {
        manifestPathParameter.value = options.ManifestPath;
        payloadPathParameter.value = options.PayloadPath;
    }

    packageIdParameter.value = options.PackageId;
    feedPort.Connect(new SmokeConnectedObject("Smoke Feed"));
    productPort.Connect(new SmokeConnectedObject("Smoke Product"));

    var validationMessage = string.Empty;
    var isValid = unitOperation.Validate(ref validationMessage);
    Console.WriteLine("== Unit Validation ==");
    Console.WriteLine($"Valid: {isValid}");
    Console.WriteLine($"Message: {validationMessage}");
    Console.WriteLine();
    Console.WriteLine("== Parameter Metadata ==");
    Console.WriteLine($"{flowsheetParameter.ComponentName}: kind={flowsheetParameter.ValueKind}, default={(flowsheetParameter.DefaultValue ?? "<null>")}, allowEmpty={flowsheetParameter.AllowsEmptyValue}, companion={(flowsheetParameter.RequiredCompanionParameterName ?? "<none>")}");
    Console.WriteLine($"{packageIdParameter.ComponentName}: kind={packageIdParameter.ValueKind}, default={(packageIdParameter.DefaultValue ?? "<null>")}, allowEmpty={packageIdParameter.AllowsEmptyValue}, companion={(packageIdParameter.RequiredCompanionParameterName ?? "<none>")}");
    Console.WriteLine($"{manifestPathParameter.ComponentName}: kind={manifestPathParameter.ValueKind}, default={(manifestPathParameter.DefaultValue ?? "<null>")}, allowEmpty={manifestPathParameter.AllowsEmptyValue}, companion={(manifestPathParameter.RequiredCompanionParameterName ?? "<none>")}");
    Console.WriteLine($"{payloadPathParameter.ComponentName}: kind={payloadPathParameter.ValueKind}, default={(payloadPathParameter.DefaultValue ?? "<null>")}, allowEmpty={payloadPathParameter.AllowsEmptyValue}, companion={(payloadPathParameter.RequiredCompanionParameterName ?? "<none>")}");
    Console.WriteLine();

    if (!isValid)
    {
        throw new InvalidOperationException("Unit operation validation failed before Calculate().");
    }

    unitOperation.Calculate();

    var calculationResult = unitOperation.LastCalculationResult
        ?? throw new InvalidOperationException("Unit operation should expose the last calculation result after Calculate().");
    EnsureCondition(
        string.Equals(calculationResult.Status, "converged", StringComparison.Ordinal),
        "unit operation calculation result should expose converged status for the smoke sample.");
    EnsureCondition(
        calculationResult.Summary.DiagnosticCount == calculationResult.Diagnostics.Count,
        "calculation summary diagnostic count should match the exported diagnostics.");
    EnsureCondition(
        !string.IsNullOrWhiteSpace(calculationResult.Summary.PrimaryMessage),
        "calculation summary should expose a primary message.");
    EnsureCondition(
        calculationResult.Diagnostics.Count > 0,
        "calculation result should expose at least one diagnostic.");

    Console.WriteLine("== Unit Calculation Result ==");
    Console.WriteLine($"Status: {calculationResult.Status}");
    Console.WriteLine($"Summary.HighestSeverity: {calculationResult.Summary.HighestSeverity}");
    Console.WriteLine($"Summary.PrimaryMessage: {calculationResult.Summary.PrimaryMessage}");
    Console.WriteLine($"Summary.DiagnosticCount: {calculationResult.Summary.DiagnosticCount}");
    Console.WriteLine($"Summary.RelatedUnitIds: {string.Join(", ", calculationResult.Summary.RelatedUnitIds)}");
    Console.WriteLine($"Summary.RelatedStreamIds: {string.Join(", ", calculationResult.Summary.RelatedStreamIds)}");
    Console.WriteLine("Diagnostics:");
    foreach (var diagnostic in calculationResult.Diagnostics)
    {
        Console.WriteLine(
            $"- [{diagnostic.Severity}] {diagnostic.Code}: {diagnostic.Message}");
    }
    Console.WriteLine();

    unitOperation.Terminate();
    EnsureCondition(unitOperation.LastCalculationResult is null, "terminate should clear the last calculation result.");
    EnsureCondition(!feedPort.IsConnected, "feed port should release its connected object during Terminate().");
    EnsureCondition(!productPort.IsConnected, "product port should release its connected object during Terminate().");
    ExpectCapeBadInvOrder(() => _ = parameterCollection.Count(), "parameter collection count after terminate");
    ExpectCapeBadInvOrder(() => _ = flowsheetParameter.value, "parameter value get after terminate");
    ExpectCapeBadInvOrder(() => feedPort.Connect(new SmokeConnectedObject("Late Feed")), "port connect after terminate");
    ExpectCapeBadInvOrder(() => _ = feedPort.connectedObject, "port connectedObject after terminate");
}

static void EnsureSameReference<T>(T expected, T actual, string scenario)
    where T : class
{
    if (!ReferenceEquals(expected, actual))
    {
        throw new InvalidOperationException($"Unexpected object instance returned for {scenario}.");
    }
}

static void EnsureCondition(bool condition, string message)
{
    if (!condition)
    {
        throw new InvalidOperationException(message);
    }
}

static void ExpectCapeBadInvOrder(Action action, string scenario)
{
    try
    {
        action();
    }
    catch (CapeBadInvocationOrderException)
    {
        return;
    }

    throw new InvalidOperationException($"Expected CapeBadInvocationOrderException for {scenario}.");
}

static void ExpectCapeInvalidArgument(Action action, string scenario)
{
    try
    {
        action();
    }
    catch (CapeInvalidArgumentException)
    {
        return;
    }

    throw new InvalidOperationException($"Expected CapeInvalidArgumentException for {scenario}.");
}

file sealed class SmokeOptions
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

file enum SmokeMode
{
    Adapter,
    UnitOperation,
}

file sealed class SmokeConnectedObject : ICapeIdentification
{
    public SmokeConnectedObject(string componentName)
    {
        ComponentName = componentName;
        ComponentDescription = "Smoke test placeholder connected object.";
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }
}

file sealed class InvalidSmokeConnectedObject : ICapeIdentification
{
    public InvalidSmokeConnectedObject(string componentName)
    {
        ComponentName = componentName;
        ComponentDescription = "Smoke test invalid placeholder connected object.";
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }
}
