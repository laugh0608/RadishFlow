using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

var options = ContractTestOptions.Parse(args);
var tests = new (string Name, Action<ContractTestContext> Execute)[]
{
    ("validate-before-initialize", static context => ContractTests.ValidateBeforeInitialize_ReturnsInvalidAndEmptyReport(context)),
    ("validation-failure-report", static context => ContractTests.CalculateValidationFailure_PopulatesFailureReport(context)),
    ("native-failure-report", static context => ContractTests.CalculateNativeFailure_PopulatesFailureReport(context)),
    ("success-report", static context => ContractTests.SuccessfulCalculate_PopulatesSuccessReport(context)),
    ("configuration-invalidation", static context => ContractTests.ConfigurationChange_ClearsReportAndMarksNotValidated(context)),
    ("terminate-report", static context => ContractTests.Terminate_ResetsReportAndBlocksCalculate(context)),
};

var selectedTests = tests
    .Where(test => string.Equals(options.TestFilter, "all", StringComparison.OrdinalIgnoreCase) ||
                   string.Equals(test.Name, options.TestFilter, StringComparison.OrdinalIgnoreCase))
    .ToArray();
if (selectedTests.Length == 0)
{
    throw new ArgumentException(
        $"Unsupported contract test `{options.TestFilter}`. Supported values: all|{string.Join("|", tests.Select(test => test.Name))}.");
}

var failures = new List<string>();
foreach (var (name, execute) in selectedTests)
{
    using var context = new ContractTestContext(options);
    try
    {
        execute(context);
        Console.WriteLine($"[PASS] {name}");
    }
    catch (Exception error)
    {
        failures.Add($"{name}: {error.Message}");
        Console.WriteLine($"[FAIL] {name}");
        Console.WriteLine(error);
    }
}

if (failures.Count > 0)
{
    Environment.ExitCode = 1;
    return;
}

Console.WriteLine($"Executed {selectedTests.Length} contract test(s).");

internal static class ContractTests
{
    public static void ValidateBeforeInitialize_ReturnsInvalidAndEmptyReport(ContractTestContext context)
    {
        var message = string.Empty;
        var isValid = context.UnitOperation.Validate(ref message);

        ContractAssert.False(isValid, "Validate() before Initialize() should be invalid.");
        ContractAssert.Equal(CapeValidationStatus.Invalid, context.UnitOperation.ValStatus, "Validate() should set ValStatus to Invalid.");
        ContractAssert.Contains(message, "Initialize must be called before Validate.", "Validate() before Initialize() should explain the required lifecycle step.");
        ContractAssert.Equal(UnitOperationCalculationReportState.None, context.UnitOperation.GetCalculationReportState(), "Report state should stay empty before any calculation.");
        ContractAssert.Equal("No calculation result is available.", context.UnitOperation.GetCalculationReportHeadline(), "Empty report headline should remain frozen.");
        ContractAssert.Equal(1, context.UnitOperation.GetCalculationReportLineCount(), "Empty report should still expose one display line.");
    }

    public static void CalculateValidationFailure_PopulatesFailureReport(ContractTestContext context)
    {
        context.Initialize();
        context.LoadFlowsheet();
        context.LoadPackageFiles();
        context.ConnectRequiredPorts();

        var error = ContractAssert.Throws<CapeBadInvocationOrderException>(
            static unitOperation => unitOperation.Calculate(),
            context.UnitOperation,
            "Calculate() without selected package should fail at the validation boundary.");

        ContractAssert.Equal(CapeValidationStatus.Invalid, context.UnitOperation.ValStatus, "Validation failure should set ValStatus to Invalid.");
        ContractAssert.Equal(UnitOperationCalculationReportState.Failure, context.UnitOperation.GetCalculationReportState(), "Validation failure should publish failure report state.");
        ContractAssert.Equal(error.ErrorName, context.UnitOperation.GetCalculationReportDetailValue(UnitOperationCalculationReportDetailCatalog.Error), "Failure report should preserve semantic error name.");
        ContractAssert.Equal(nameof(RadishFlowCapeOpenUnitOperation.SelectPropertyPackage), context.UnitOperation.GetCalculationReportDetailValue(UnitOperationCalculationReportDetailCatalog.RequestedOperation), "Validation failure should point back to SelectPropertyPackage().");
        ContractAssert.Null(context.UnitOperation.GetCalculationReportDetailValue(UnitOperationCalculationReportDetailCatalog.NativeStatus), "Validation failure should not invent native status.");
        ContractAssert.Null(context.UnitOperation.LastCalculationResult, "Validation failure should not preserve a stale success result.");
        ContractAssert.NotNull(context.UnitOperation.LastCalculationFailure, "Validation failure should preserve failure summary.");
    }

    public static void CalculateNativeFailure_PopulatesFailureReport(ContractTestContext context)
    {
        context.ConfigureMinimumValidInputs();
        context.UnitOperation.SelectPropertyPackage("missing-package-for-contract");

        var error = ContractAssert.Throws<CapeInvalidArgumentException>(
            static unitOperation => unitOperation.Calculate(),
            context.UnitOperation,
            "Calculate() with missing package should fail at the native boundary.");

        ContractAssert.Equal(CapeValidationStatus.Invalid, context.UnitOperation.ValStatus, "Native failure should set ValStatus to Invalid.");
        ContractAssert.Equal(UnitOperationCalculationReportState.Failure, context.UnitOperation.GetCalculationReportState(), "Native failure should publish failure report state.");
        ContractAssert.Equal(error.ErrorName, context.UnitOperation.GetCalculationReportDetailValue(UnitOperationCalculationReportDetailCatalog.Error), "Native failure report should preserve semantic error name.");
        ContractAssert.Equal("MissingEntity", context.UnitOperation.GetCalculationReportDetailValue(UnitOperationCalculationReportDetailCatalog.NativeStatus), "Native failure should expose native status.");
        ContractAssert.Null(context.UnitOperation.GetCalculationReportDetailValue(UnitOperationCalculationReportDetailCatalog.RequestedOperation), "Native failure should not invent requested operation.");
        ContractAssert.Null(context.UnitOperation.LastCalculationResult, "Native failure should clear the last success result.");
        ContractAssert.NotNull(context.UnitOperation.LastCalculationFailure, "Native failure should preserve failure summary.");
    }

    public static void SuccessfulCalculate_PopulatesSuccessReport(ContractTestContext context)
    {
        context.ConfigureMinimumValidInputs();
        context.UnitOperation.Calculate();

        ContractAssert.Equal(CapeValidationStatus.Valid, context.UnitOperation.ValStatus, "Successful Calculate() should set ValStatus to Valid.");
        ContractAssert.Equal(UnitOperationCalculationReportState.Success, context.UnitOperation.GetCalculationReportState(), "Success should publish success report state.");
        ContractAssert.Equal("converged", context.UnitOperation.GetCalculationReportDetailValue(UnitOperationCalculationReportDetailCatalog.Status), "Success report should expose converged status.");
        ContractAssert.True(context.UnitOperation.GetCalculationReportLineCount() > context.UnitOperation.GetCalculationReportDetailKeyCount(), "Success report should expose supplemental lines beyond stable details.");
    }

    public static void ConfigurationChange_ClearsReportAndMarksNotValidated(ContractTestContext context)
    {
        context.ConfigureMinimumValidInputs();
        context.UnitOperation.Calculate();

        ContractAssert.True(context.IsProductPortConnected(), "Product port should be connected before the invalidation step.");
        context.DisconnectProductPort();
        ContractAssert.False(context.IsProductPortConnected(), "Product port should be disconnected after the invalidation step.");

        ContractAssert.Equal(CapeValidationStatus.NotValidated, context.UnitOperation.ValStatus, "Configuration changes after success should reset ValStatus to NotValidated.");
        ContractAssert.Equal(UnitOperationCalculationReportState.None, context.UnitOperation.GetCalculationReportState(), "Configuration changes should clear the last calculation report.");
        ContractAssert.Null(context.UnitOperation.LastCalculationResult, "Configuration changes should clear the last success result.");
        ContractAssert.Null(context.UnitOperation.LastCalculationFailure, "Configuration changes should clear the last failure result.");
    }

    public static void Terminate_ResetsReportAndBlocksCalculate(ContractTestContext context)
    {
        context.ConfigureMinimumValidInputs();
        context.UnitOperation.Calculate();

        context.UnitOperation.Terminate();

        ContractAssert.Equal(CapeValidationStatus.NotValidated, context.UnitOperation.ValStatus, "Terminate() should reset ValStatus to NotValidated.");
        ContractAssert.Equal(UnitOperationCalculationReportState.None, context.UnitOperation.GetCalculationReportState(), "Terminate() should reset report state to empty.");

        var message = string.Empty;
        var isValid = context.UnitOperation.Validate(ref message);
        ContractAssert.False(isValid, "Validate() after Terminate() should stay invalid.");
        ContractAssert.Contains(message, "Terminate has already been called", "Validate() after Terminate() should explain the terminal state.");

        var error = ContractAssert.Throws<CapeBadInvocationOrderException>(
            static unitOperation => unitOperation.Calculate(),
            context.UnitOperation,
            "Calculate() after Terminate() should be blocked.");
        ContractAssert.Equal(nameof(RadishFlowCapeOpenUnitOperation.Calculate), error.Operation, "Calculate() after Terminate() should fail at the Calculate() boundary.");
    }
}

internal sealed class ContractTestContext : IDisposable
{
    private readonly ContractTestOptions _options;

    public ContractTestContext(ContractTestOptions options)
    {
        _options = options;
        UnitOperation = new RadishFlowCapeOpenUnitOperation();
        UnitOperation.ConfigureNativeLibraryDirectory(options.NativeLibraryDirectory);
    }

    public RadishFlowCapeOpenUnitOperation UnitOperation { get; }

    public void Initialize()
    {
        UnitOperation.Initialize();
    }

    public void LoadFlowsheet()
    {
        UnitOperation.LoadFlowsheetJson(File.ReadAllText(_options.ProjectPath));
    }

    public void LoadPackageFiles()
    {
        UnitOperation.LoadPropertyPackageFiles(_options.ManifestPath, _options.PayloadPath);
    }

    public void SelectPackage()
    {
        UnitOperation.SelectPropertyPackage(_options.PackageId);
    }

    public void ConnectRequiredPorts()
    {
        var ports = (ICapeCollection)UnitOperation.Ports;
        ((ICapeUnitPort)ports.Item("Feed")).Connect(new ContractConnectedObject("Contract Feed"));
        ((ICapeUnitPort)ports.Item("Product")).Connect(new ContractConnectedObject("Contract Product"));
    }

    public void DisconnectProductPort()
    {
        var ports = (ICapeCollection)UnitOperation.Ports;
        ((ICapeUnitPort)ports.Item("Product")).Disconnect();
    }

    public bool IsProductPortConnected()
    {
        var ports = (ICapeCollection)UnitOperation.Ports;
        return ((ICapeUnitPort)ports.Item("Product")).connectedObject is not null;
    }

    public void ConfigureMinimumValidInputs()
    {
        Initialize();
        LoadFlowsheet();
        LoadPackageFiles();
        SelectPackage();
        ConnectRequiredPorts();
    }

    public void Dispose()
    {
        UnitOperation.Dispose();
    }
}

internal static class ContractAssert
{
    public static void True(bool condition, string message)
    {
        if (!condition)
        {
            throw new InvalidOperationException(message);
        }
    }

    public static void False(bool condition, string message)
    {
        if (condition)
        {
            throw new InvalidOperationException(message);
        }
    }

    public static void Equal<T>(T expected, T actual, string message)
    {
        if (!EqualityComparer<T>.Default.Equals(expected, actual))
        {
            throw new InvalidOperationException($"{message} Expected `{expected}`, got `{actual}`.");
        }
    }

    public static void Contains(string actual, string expectedFragment, string message)
    {
        if (!actual.Contains(expectedFragment, StringComparison.Ordinal))
        {
            throw new InvalidOperationException($"{message} Missing fragment `{expectedFragment}` in `{actual}`.");
        }
    }

    public static void Null(object? value, string message)
    {
        if (value is not null)
        {
            throw new InvalidOperationException(message);
        }
    }

    public static void NotNull(object? value, string message)
    {
        if (value is null)
        {
            throw new InvalidOperationException(message);
        }
    }

    public static TException Throws<TException>(
        Action<RadishFlowCapeOpenUnitOperation> action,
        RadishFlowCapeOpenUnitOperation unitOperation,
        string message)
        where TException : Exception
    {
        try
        {
            action(unitOperation);
        }
        catch (TException error)
        {
            return error;
        }

        throw new InvalidOperationException(message);
    }
}

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

internal sealed class ContractConnectedObject : ICapeIdentification
{
    public ContractConnectedObject(string componentName)
    {
        ComponentName = componentName;
        ComponentDescription = "UnitOp.Mvp contract test connected object.";
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }
}
