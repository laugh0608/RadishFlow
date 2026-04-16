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
    var initialReport = UnitOperationHostReportReader.Read(unitOperation);
    var initialPresentation = UnitOperationHostReportPresenter.Present(initialReport);
    var initialDocument = UnitOperationHostReportFormatter.Format(initialPresentation);
    EnsureCondition(
        initialReport.State == UnitOperationCalculationReportState.None,
        "unit operation should expose an empty calculation report before Calculate().");
    EnsureCondition(
        string.Equals(initialReport.Headline, "No calculation result is available.", StringComparison.Ordinal),
        "empty calculation report should expose the frozen empty headline.");
    EnsureCondition(
        initialReport.DetailKeyCount == 0,
        "empty calculation report should not expose detail keys.");
    EnsureCondition(
        initialReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Status) is null,
        "empty calculation report should not expose status detail values.");
    EnsureCondition(
        initialReport.ScalarLines.Count == 1,
        "empty calculation report should expose exactly one display line.");
    EnsureCondition(
        string.Equals(initialReport.ScalarLines[0], initialReport.Headline, StringComparison.Ordinal),
        "empty calculation report scalar line export should collapse to the headline only.");
    EnsureHostReportLineApisAgree(initialReport, "empty calculation report");
    EnsureCondition(
        string.Equals(initialReport.Text, initialReport.Headline, StringComparison.Ordinal),
        "empty calculation report text should match the headline.");
    EnsureCondition(
        string.Equals(initialPresentation.StateLabel, "NoResult", StringComparison.Ordinal) &&
        !initialPresentation.RequiresAttention &&
        !initialPresentation.HasStableDetails &&
        !initialPresentation.HasSupplementalLines,
        "empty host presentation should expose idle label without stable details or supplemental lines.");
    EnsureCondition(
        initialDocument.HasSections &&
        initialDocument.Sections.Count == 1 &&
        string.Equals(initialDocument.Sections[0].Title, "Overview", StringComparison.Ordinal) &&
        initialDocument.Sections[0].Lines.Count == 3,
        "empty host formatter should expose overview section only.");

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
    var validationFailureReport = UnitOperationHostReportReader.Read(unitOperation);
    var validationFailurePresentation = UnitOperationHostReportPresenter.Present(validationFailureReport);
    var validationFailureDocument = UnitOperationHostReportFormatter.Format(validationFailurePresentation);
    EnsureCondition(
        string.Equals(
            validationFailureReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Error),
            validationFailureError.ErrorName,
            StringComparison.Ordinal),
        "validation failure host report should preserve the CAPE-OPEN semantic error name.");
    EnsureCondition(
        string.Equals(
            validationFailureReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Operation),
            validationFailureError.Operation,
            StringComparison.Ordinal),
        "validation failure host report should preserve the failing operation name.");
    EnsureCondition(
        validationFailureReport.State == UnitOperationCalculationReportState.Failure,
        "validation failure should switch the host-visible report into failure state.");
    EnsureCondition(
        !string.IsNullOrWhiteSpace(validationFailureReport.Headline),
        "validation failure host report should expose a non-empty headline.");
    EnsureCondition(
        validationFailureReport.DetailKeys.SequenceEqual(
            UnitOperationCalculationReportDetailCatalog
                .GetStableKeyOrder(UnitOperationCalculationReportState.Failure)
                .Where(key => string.Equals(key, UnitOperationCalculationReportDetailCatalog.Error, StringComparison.Ordinal) ||
                              string.Equals(key, UnitOperationCalculationReportDetailCatalog.Operation, StringComparison.Ordinal) ||
                              string.Equals(key, UnitOperationCalculationReportDetailCatalog.RequestedOperation, StringComparison.Ordinal)),
            StringComparer.Ordinal),
        "validation failure host report detail key enumeration should follow the frozen failure key order.");
    EnsureCondition(
        string.Equals(
            validationFailureReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.RequestedOperation),
            nameof(RadishFlowCapeOpenUnitOperation.SelectPropertyPackage),
            StringComparison.Ordinal) &&
        validationFailureReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.NativeStatus) is null,
        "validation failure host report should expose requested operation without inventing native status.");
    EnsureCondition(
        validationFailureReport.VectorLines.Any(line => line.Contains("requestedOperation=SelectPropertyPackage", StringComparison.Ordinal)),
        "validation failure host report should expose the requested follow-up operation.");
    EnsureCondition(
        validationFailureReport.ScalarLines.Count == validationFailureReport.DetailKeyCount + 1,
        "validation failure host report should expose headline plus stable detail entries.");
    EnsureHostReportLineApisAgree(validationFailureReport, "validation failure host report");
    EnsureCondition(
        validationFailureReport.Text.Contains(validationFailureReport.Headline, StringComparison.Ordinal) &&
        validationFailureReport.Text.Contains("requestedOperation=SelectPropertyPackage", StringComparison.Ordinal),
        "validation failure host report text should include both the headline and requested operation.");
    EnsureCondition(
        string.Equals(validationFailurePresentation.StateLabel, "Failure", StringComparison.Ordinal) &&
        validationFailurePresentation.RequiresAttention &&
        validationFailurePresentation.HasStableDetails &&
        !validationFailurePresentation.HasSupplementalLines,
        "validation failure host presentation should expose failure label, attention hint and only stable detail rows.");
    EnsureCondition(
        validationFailureDocument.Sections.Count == 2 &&
        string.Equals(validationFailureDocument.Sections[0].Title, "Overview", StringComparison.Ordinal) &&
        string.Equals(validationFailureDocument.Sections[1].Title, "Stable Details", StringComparison.Ordinal),
        "validation failure host formatter should expose overview and stable detail sections.");

    packageIdParameter.value = "missing-package-for-smoke";
    var nativeFailureError = ExpectCapeInvalidArgument(
        () => unitOperation.Calculate(),
        "calculate with missing property package id");
    var nativeFailureReport = UnitOperationHostReportReader.Read(unitOperation);
    var nativeFailurePresentation = UnitOperationHostReportPresenter.Present(nativeFailureReport);
    var nativeFailureDocument = UnitOperationHostReportFormatter.Format(nativeFailurePresentation);
    EnsureCondition(
        string.Equals(
            nativeFailureReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Error),
            nativeFailureError.ErrorName,
            StringComparison.Ordinal),
        "native failure host report should preserve the CAPE-OPEN semantic error name.");
    EnsureCondition(
        string.Equals(
            nativeFailureReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Operation),
            nativeFailureError.Operation,
            StringComparison.Ordinal),
        "native failure host report should preserve the failing operation name.");
    EnsureCondition(
        nativeFailureReport.State == UnitOperationCalculationReportState.Failure,
        "native failure should keep the host-visible report in failure state.");
    EnsureCondition(
        !string.IsNullOrWhiteSpace(nativeFailureReport.Headline),
        "native failure host report should expose a non-empty headline.");
    EnsureCondition(
        nativeFailureReport.DetailKeys.SequenceEqual(
            UnitOperationCalculationReportDetailCatalog
                .GetStableKeyOrder(UnitOperationCalculationReportState.Failure)
                .Where(key => string.Equals(key, UnitOperationCalculationReportDetailCatalog.Error, StringComparison.Ordinal) ||
                              string.Equals(key, UnitOperationCalculationReportDetailCatalog.Operation, StringComparison.Ordinal) ||
                              string.Equals(key, UnitOperationCalculationReportDetailCatalog.NativeStatus, StringComparison.Ordinal)),
            StringComparer.Ordinal),
        "native failure host report detail key enumeration should follow the frozen failure key order.");
    EnsureCondition(
        nativeFailureReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.RequestedOperation) is null &&
        nativeFailureReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.DiagnosticCode) is null &&
        string.Equals(
            nativeFailureReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.NativeStatus),
            "MissingEntity",
            StringComparison.Ordinal),
        "native failure host report should expose native status without inventing optional failure details.");
    EnsureCondition(
        nativeFailureReport.VectorLines.Any(line => line.Contains("nativeStatus=MissingEntity", StringComparison.Ordinal)),
        "native failure host report should expose the mapped native status.");
    EnsureCondition(
        nativeFailureReport.ScalarLines.Count >= 3 &&
        string.Equals(nativeFailureReport.ScalarLines[0], nativeFailureReport.Headline, StringComparison.Ordinal),
        "native failure host report lines should start with the headline before detail lines.");
    EnsureHostReportLineApisAgree(nativeFailureReport, "native failure host report");
    EnsureCondition(
        string.Equals(nativeFailurePresentation.StateLabel, "Failure", StringComparison.Ordinal) &&
        nativeFailurePresentation.RequiresAttention &&
        nativeFailurePresentation.HasStableDetails &&
        !nativeFailurePresentation.HasSupplementalLines,
        "native failure host presentation should expose failure label, attention hint and only stable detail rows.");
    EnsureCondition(
        nativeFailureDocument.Sections.Count == 2 &&
        string.Equals(nativeFailureDocument.Sections[0].Title, "Overview", StringComparison.Ordinal) &&
        string.Equals(nativeFailureDocument.Sections[1].Title, "Stable Details", StringComparison.Ordinal),
        "native failure host formatter should expose overview and stable detail sections.");

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

    var successReport = UnitOperationHostReportReader.Read(unitOperation);
    var successPresentation = UnitOperationHostReportPresenter.Present(successReport);
    var successDocument = UnitOperationHostReportFormatter.Format(successPresentation);
    EnsureCondition(
        successReport.State == UnitOperationCalculationReportState.Success,
        "successful Calculate() should switch the host-visible report into success state.");
    EnsureCondition(
        !string.IsNullOrWhiteSpace(successReport.Headline),
        "success host report should expose a non-empty headline.");
    EnsureCondition(
        successReport.DetailKeys.SequenceEqual(
            UnitOperationCalculationReportDetailCatalog.SuccessStableKeyOrder,
            StringComparer.Ordinal),
        "success host report detail key enumeration should expose the frozen success key order.");
    EnsureCondition(
        string.Equals(
            successReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Status),
            "converged",
            StringComparison.Ordinal) &&
        string.Equals(
            successReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.HighestSeverity),
            "info",
            StringComparison.Ordinal),
        "success host report should expose stable status and highest severity detail values.");
    EnsureCondition(
        int.TryParse(
            successReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.DiagnosticCount),
            out var successDiagnosticCount) &&
        successDiagnosticCount > 0,
        "success host report should expose a positive diagnostic count.");
    EnsureCondition(
        successReport.ScalarLines.Count > successReport.DetailKeyCount + 1,
        "success host report should expose non-key diagnostic display lines in addition to stable detail entries.");
    EnsureCondition(
        successReport.Text.Contains(successReport.Headline, StringComparison.Ordinal) &&
        successReport.Text.Contains("diagnosticCount=", StringComparison.Ordinal) &&
        successReport.Text.Contains("[info]", StringComparison.Ordinal),
        "success host report text should include the headline, stable detail lines and diagnostic display lines.");
    EnsureHostReportLineApisAgree(successReport, "success host report");
    EnsureCondition(
        string.Equals(successPresentation.StateLabel, "Success", StringComparison.Ordinal) &&
        !successPresentation.RequiresAttention &&
        successPresentation.HasStableDetails &&
        successPresentation.HasSupplementalLines &&
        successPresentation.SupplementalLines.All(line => line.StartsWith("[", StringComparison.Ordinal)),
        "success host presentation should expose success label, stable details and diagnostic supplemental lines.");
    EnsureCondition(
        successDocument.Sections.Count == 3 &&
        string.Equals(successDocument.Sections[0].Title, "Overview", StringComparison.Ordinal) &&
        string.Equals(successDocument.Sections[1].Title, "Stable Details", StringComparison.Ordinal) &&
        string.Equals(successDocument.Sections[2].Title, "Supplemental", StringComparison.Ordinal),
        "success host formatter should expose overview, stable detail and supplemental sections.");
    EnsureCondition(
        successDocument.FormattedText.Contains("[Overview]", StringComparison.Ordinal) &&
        successDocument.FormattedText.Contains("[Stable Details]", StringComparison.Ordinal) &&
        successDocument.FormattedText.Contains("[Supplemental]", StringComparison.Ordinal),
        "success host formatter text should include all section headers.");

    Console.WriteLine("== Sectioned Host Report ==");
    foreach (var section in successDocument.Sections)
    {
        Console.WriteLine($"[{section.Title}]");
        foreach (var line in section.Lines)
        {
            Console.WriteLine($"- {line}");
        }
    }
    Console.WriteLine();

    unitOperation.Terminate();
    var terminatedReport = UnitOperationHostReportReader.Read(unitOperation);
    var terminatedPresentation = UnitOperationHostReportPresenter.Present(terminatedReport);
    var terminatedDocument = UnitOperationHostReportFormatter.Format(terminatedPresentation);
    EnsureCondition(
        terminatedReport.State == UnitOperationCalculationReportState.None,
        "terminate should reset the host-visible report to empty state.");
    EnsureCondition(
        string.Equals(terminatedReport.Headline, "No calculation result is available.", StringComparison.Ordinal),
        "terminate should reset the host-visible report headline to the empty state headline.");
    EnsureCondition(
        terminatedReport.DetailKeyCount == 0,
        "terminate should clear host-visible stable report detail keys.");
    EnsureCondition(
        terminatedReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Status) is null,
        "terminate should clear host-visible stable report detail values.");
    EnsureCondition(
        terminatedReport.ScalarLines.Count == 1 &&
        string.Equals(terminatedReport.ScalarLines[0], "No calculation result is available.", StringComparison.Ordinal),
        "terminate should reset the host-visible scalar report line access to the empty headline.");
    EnsureHostReportLineApisAgree(terminatedReport, "terminated host report");
    EnsureCondition(
        string.Equals(terminatedReport.Text, "No calculation result is available.", StringComparison.Ordinal),
        "terminate should reset the host-visible report text to the empty headline.");
    EnsureCondition(
        string.Equals(terminatedPresentation.StateLabel, "NoResult", StringComparison.Ordinal) &&
        !terminatedPresentation.RequiresAttention &&
        !terminatedPresentation.HasStableDetails &&
        !terminatedPresentation.HasSupplementalLines,
        "terminated host presentation should return to idle label without stable details or supplemental lines.");
    EnsureCondition(
        terminatedDocument.Sections.Count == 1 &&
        string.Equals(terminatedDocument.Sections[0].Title, "Overview", StringComparison.Ordinal),
        "terminated host formatter should return to overview-only section output.");
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

static void EnsureHostReportLineApisAgree(UnitOperationHostReportSnapshot report, string scenario)
{
    EnsureCondition(
        report.ScalarLines.SequenceEqual(report.VectorLines, StringComparer.Ordinal),
        $"{scenario} scalar and vector line exports should match.");
    EnsureCondition(
        string.Equals(report.Text, string.Join(Environment.NewLine, report.ScalarLines), StringComparison.Ordinal),
        $"{scenario} text export should match the scalar line export.");
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
