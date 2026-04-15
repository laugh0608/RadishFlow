using RadishFlow.CapeOpen.Adapter;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
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
    var initialReport = unitOperation.GetCalculationReport();
    var initialReportState = unitOperation.GetCalculationReportState();
    var initialReportHeadline = unitOperation.GetCalculationReportHeadline();
    var initialReportDetailKeyCount = unitOperation.GetCalculationReportDetailKeyCount();
    var initialReportStatusDetail = unitOperation.GetCalculationReportDetailValue("status");
    var initialReportLineCount = unitOperation.GetCalculationReportLineCount();
    var initialReportLines = unitOperation.GetCalculationReportLines();
    var initialReportText = unitOperation.GetCalculationReportText();
    EnsureCondition(
        initialReport.State == UnitOperationCalculationReportState.None,
        "unit operation should expose an empty calculation report before Calculate().");
    EnsureCondition(
        initialReportState == initialReport.State,
        "empty calculation report scalar state should match the DTO state.");
    EnsureCondition(
        string.Equals(initialReportHeadline, initialReport.Headline, StringComparison.Ordinal),
        "empty calculation report scalar headline should match the DTO headline.");
    EnsureCondition(
        initialReportDetailKeyCount == 0,
        "empty calculation report should not expose detail keys.");
    EnsureCondition(
        initialReportStatusDetail is null,
        "empty calculation report should not expose status detail values.");
    EnsureCondition(
        initialReportLineCount == 1,
        "empty calculation report should expose exactly one display line.");
    EnsureCondition(
        initialReportLines.Count == 1 && string.Equals(initialReportLines[0], initialReport.Headline, StringComparison.Ordinal),
        "empty calculation report lines should collapse to the headline only.");
    EnsureCondition(
        string.Equals(unitOperation.GetCalculationReportLine(0), initialReport.Headline, StringComparison.Ordinal),
        "empty calculation report line(0) should return the headline.");
    EnsureCondition(
        string.Equals(initialReportText, initialReport.Headline, StringComparison.Ordinal),
        "empty calculation report text should match the headline.");

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

    feedPort.Connect(new SmokeConnectedObject("Smoke Feed"));
    productPort.Connect(new SmokeConnectedObject("Smoke Product"));

    var validationFailureError = ExpectCapeBadInvOrder(
        () => unitOperation.Calculate(),
        "calculate without property package id");
    var validationFailure = unitOperation.LastCalculationFailure
        ?? throw new InvalidOperationException("unit operation should preserve the last validation failure after Calculate().");
    var validationFailureReport = unitOperation.GetCalculationReport();
    var validationFailureState = unitOperation.GetCalculationReportState();
    var validationFailureHeadline = unitOperation.GetCalculationReportHeadline();
    var validationFailureDetailKeyCount = unitOperation.GetCalculationReportDetailKeyCount();
    var validationFailureRequestedOperation = unitOperation.GetCalculationReportDetailValue("requestedOperation");
    var validationFailureNativeStatus = unitOperation.GetCalculationReportDetailValue("nativeStatus");
    var validationFailureLineCount = unitOperation.GetCalculationReportLineCount();
    var validationFailureText = unitOperation.GetCalculationReportText();
    EnsureCondition(unitOperation.LastCalculationResult is null, "failed Calculate() should not expose a successful calculation result.");
    EnsureCondition(
        string.Equals(validationFailure.ErrorName, validationFailureError.ErrorName, StringComparison.Ordinal),
        "validation failure contract should preserve the CAPE-OPEN semantic error name.");
    EnsureCondition(
        string.Equals(validationFailure.RequestedOperation, nameof(RadishFlowCapeOpenUnitOperation.SelectPropertyPackage), StringComparison.Ordinal),
        "validation failure contract should preserve the requested follow-up operation.");
    EnsureCondition(
        string.IsNullOrWhiteSpace(validationFailure.NativeStatus),
        "validation failure should not expose a native status.");
    EnsureCondition(
        validationFailureReport.State == UnitOperationCalculationReportState.Failure,
        "validation failure should switch the unified calculation report into failure state.");
    EnsureCondition(
        validationFailureState == validationFailureReport.State &&
        string.Equals(validationFailureHeadline, validationFailureReport.Headline, StringComparison.Ordinal),
        "validation failure scalar metadata should match the DTO report metadata.");
    EnsureCondition(
        validationFailureDetailKeyCount == validationFailureReport.DetailLines.Count &&
        string.Equals(unitOperation.GetCalculationReportDetailKey(0), "error", StringComparison.Ordinal) &&
        string.Equals(unitOperation.GetCalculationReportDetailKey(validationFailureDetailKeyCount - 1), "requestedOperation", StringComparison.Ordinal),
        "validation failure detail key enumeration should expose stable key order.");
    EnsureCondition(
        string.Equals(validationFailureRequestedOperation, "SelectPropertyPackage", StringComparison.Ordinal) &&
        validationFailureNativeStatus is null,
        "validation failure detail value access should expose requested operation without inventing native status.");
    EnsureCondition(
        validationFailureReport.DetailLines.Any(line => line.Contains("requestedOperation=SelectPropertyPackage", StringComparison.Ordinal)),
        "validation failure report should expose the requested follow-up operation.");
    EnsureCondition(
        validationFailureLineCount == validationFailureReport.DetailLines.Count + 1 &&
        string.Equals(unitOperation.GetCalculationReportLine(0), validationFailureReport.Headline, StringComparison.Ordinal),
        "validation failure report scalar line access should expose headline plus detail count.");
    EnsureCondition(
        validationFailureText.Contains(validationFailureReport.Headline, StringComparison.Ordinal) &&
        validationFailureText.Contains("requestedOperation=SelectPropertyPackage", StringComparison.Ordinal),
        "validation failure report text should include both the headline and requested operation.");

    packageIdParameter.value = "missing-package-for-smoke";
    var nativeFailureError = ExpectCapeInvalidArgument(
        () => unitOperation.Calculate(),
        "calculate with missing property package id");
    var nativeFailure = unitOperation.LastCalculationFailure
        ?? throw new InvalidOperationException("unit operation should preserve the last native failure after Calculate().");
    var nativeFailureReport = unitOperation.GetCalculationReport();
    var nativeFailureState = unitOperation.GetCalculationReportState();
    var nativeFailureHeadline = unitOperation.GetCalculationReportHeadline();
    var nativeFailureDetailKeyCount = unitOperation.GetCalculationReportDetailKeyCount();
    var nativeFailureRequestedOperation = unitOperation.GetCalculationReportDetailValue("requestedOperation");
    var nativeFailureNativeStatus = unitOperation.GetCalculationReportDetailValue("nativeStatus");
    var nativeFailureLineCount = unitOperation.GetCalculationReportLineCount();
    var nativeFailureLines = unitOperation.GetCalculationReportLines();
    EnsureCondition(
        string.Equals(nativeFailure.ErrorName, nativeFailureError.ErrorName, StringComparison.Ordinal),
        "native failure contract should preserve the CAPE-OPEN semantic error name.");
    EnsureCondition(
        string.Equals(nativeFailure.NativeStatus, "MissingEntity", StringComparison.Ordinal),
        "native failure contract should preserve the mapped native status.");
    EnsureCondition(
        nativeFailure.Summary.DiagnosticCode is null,
        "missing package smoke failure should not invent a diagnostic code.");
    EnsureCondition(
        nativeFailure.Summary.RelatedUnitIds.Count == 0,
        "missing package smoke failure should not invent related unit ids.");
    EnsureCondition(
        nativeFailureReport.State == UnitOperationCalculationReportState.Failure,
        "native failure should keep the unified calculation report in failure state.");
    EnsureCondition(
        nativeFailureState == nativeFailureReport.State &&
        string.Equals(nativeFailureHeadline, nativeFailureReport.Headline, StringComparison.Ordinal),
        "native failure scalar metadata should match the DTO report metadata.");
    EnsureCondition(
        nativeFailureDetailKeyCount == nativeFailureReport.DetailLines.Count &&
        string.Equals(unitOperation.GetCalculationReportDetailKey(0), "error", StringComparison.Ordinal) &&
        string.Equals(unitOperation.GetCalculationReportDetailKey(nativeFailureDetailKeyCount - 1), "nativeStatus", StringComparison.Ordinal),
        "native failure detail key enumeration should expose only stable key-value lines.");
    EnsureCondition(
        nativeFailureRequestedOperation is null &&
        string.Equals(nativeFailureNativeStatus, "MissingEntity", StringComparison.Ordinal),
        "native failure detail value access should expose native status without inventing requested operation.");
    EnsureCondition(
        nativeFailureReport.DetailLines.Any(line => line.Contains("nativeStatus=MissingEntity", StringComparison.Ordinal)),
        "native failure report should expose the mapped native status.");
    EnsureCondition(
        nativeFailureLineCount == nativeFailureLines.Count &&
        nativeFailureLines.Count >= 2 &&
        string.Equals(nativeFailureLines[0], nativeFailureReport.Headline, StringComparison.Ordinal),
        "native failure report lines should start with the headline before detail lines.");
    EnsureCondition(
        nativeFailureLines
            .Select((line, index) => string.Equals(line, unitOperation.GetCalculationReportLine(index), StringComparison.Ordinal))
            .All(static matches => matches),
        "native failure scalar line access should match the vector line export.");

    packageIdParameter.value = options.PackageId;

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
    var successReport = unitOperation.GetCalculationReport();
    var successReportState = unitOperation.GetCalculationReportState();
    var successReportHeadline = unitOperation.GetCalculationReportHeadline();
    var successReportDetailKeyCount = unitOperation.GetCalculationReportDetailKeyCount();
    var successReportStatus = unitOperation.GetCalculationReportDetailValue("status");
    var successReportHighestSeverity = unitOperation.GetCalculationReportDetailValue("highestSeverity");
    var successReportDiagnosticCount = unitOperation.GetCalculationReportDetailValue("diagnosticCount");
    var successReportLineCount = unitOperation.GetCalculationReportLineCount();
    var successReportLines = unitOperation.GetCalculationReportLines();
    var successReportText = unitOperation.GetCalculationReportText();
    EnsureCondition(unitOperation.LastCalculationFailure is null, "successful Calculate() should clear the last calculation failure.");
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
    EnsureCondition(
        successReport.State == UnitOperationCalculationReportState.Success,
        "successful Calculate() should switch the unified calculation report into success state.");
    EnsureCondition(
        successReportState == successReport.State &&
        string.Equals(successReportHeadline, successReport.Headline, StringComparison.Ordinal),
        "success scalar metadata should match the DTO report metadata.");
    EnsureCondition(
        successReportDetailKeyCount == 5 &&
        string.Equals(unitOperation.GetCalculationReportDetailKey(0), "status", StringComparison.Ordinal) &&
        string.Equals(unitOperation.GetCalculationReportDetailKey(1), "highestSeverity", StringComparison.Ordinal) &&
        string.Equals(unitOperation.GetCalculationReportDetailKey(2), "diagnosticCount", StringComparison.Ordinal) &&
        string.Equals(unitOperation.GetCalculationReportDetailKey(3), "relatedUnitIds", StringComparison.Ordinal) &&
        string.Equals(unitOperation.GetCalculationReportDetailKey(4), "relatedStreamIds", StringComparison.Ordinal),
        "success detail key enumeration should expose summary keys but skip diagnostic text lines.");
    EnsureCondition(
        string.Equals(successReportStatus, "converged", StringComparison.Ordinal) &&
        string.Equals(successReportHighestSeverity, "info", StringComparison.Ordinal) &&
        string.Equals(successReportDiagnosticCount, calculationResult.Summary.DiagnosticCount.ToString(), StringComparison.Ordinal),
        "success detail value access should expose stable summary keys without parsing report text.");
    EnsureCondition(
        string.Equals(successReport.Headline, calculationResult.Summary.PrimaryMessage, StringComparison.Ordinal),
        "success report headline should mirror the calculation primary message.");
    EnsureCondition(
        successReportLineCount == successReport.DetailLines.Count + 1 &&
        successReportLines.Count == successReportLineCount &&
        string.Equals(successReportLines[0], successReport.Headline, StringComparison.Ordinal),
        "success report lines should expose headline plus all detail lines.");
    EnsureCondition(
        string.Equals(unitOperation.GetCalculationReportLine(successReportLineCount - 1), successReportLines[^1], StringComparison.Ordinal),
        "success report scalar line access should expose the final detail line.");
    EnsureCondition(
        successReportText.Contains(successReport.Headline, StringComparison.Ordinal) &&
        successReportText.Contains("diagnosticCount=", StringComparison.Ordinal),
        "success report text should include both the headline and detail lines.");

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
    Console.WriteLine("== Unit Calculation Report ==");
    Console.WriteLine($"State: {successReport.State}");
    Console.WriteLine($"Headline: {successReport.Headline}");
    foreach (var detail in successReport.DetailLines)
    {
        Console.WriteLine($"- {detail}");
    }
    Console.WriteLine();

    unitOperation.Terminate();
    EnsureCondition(unitOperation.LastCalculationResult is null, "terminate should clear the last calculation result.");
    EnsureCondition(unitOperation.LastCalculationFailure is null, "terminate should clear the last calculation failure.");
    EnsureCondition(
        unitOperation.GetCalculationReport().State == UnitOperationCalculationReportState.None,
        "terminate should reset the unified calculation report to empty state.");
    EnsureCondition(
        unitOperation.GetCalculationReportState() == UnitOperationCalculationReportState.None &&
        string.Equals(unitOperation.GetCalculationReportHeadline(), "No calculation result is available.", StringComparison.Ordinal),
        "terminate should reset the scalar report metadata to the empty state and headline.");
    EnsureCondition(
        unitOperation.GetCalculationReportDetailKeyCount() == 0,
        "terminate should clear stable report detail keys.");
    EnsureCondition(
        unitOperation.GetCalculationReportDetailValue("status") is null,
        "terminate should clear stable report detail values.");
    EnsureCondition(
        unitOperation.GetCalculationReportLineCount() == 1 &&
        string.Equals(unitOperation.GetCalculationReportLine(0), "No calculation result is available.", StringComparison.Ordinal),
        "terminate should reset the scalar report line access to the empty headline.");
    EnsureCondition(
        string.Equals(unitOperation.GetCalculationReportText(), "No calculation result is available.", StringComparison.Ordinal),
        "terminate should reset the calculation report text to the empty headline.");
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

static CapeBadInvocationOrderException ExpectCapeBadInvOrder(Action action, string scenario)
{
    try
    {
        action();
    }
    catch (CapeBadInvocationOrderException error)
    {
        return error;
    }

    throw new InvalidOperationException($"Expected CapeBadInvocationOrderException for {scenario}.");
}

static CapeInvalidArgumentException ExpectCapeInvalidArgument(Action action, string scenario)
{
    try
    {
        action();
    }
    catch (CapeInvalidArgumentException error)
    {
        return error;
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
