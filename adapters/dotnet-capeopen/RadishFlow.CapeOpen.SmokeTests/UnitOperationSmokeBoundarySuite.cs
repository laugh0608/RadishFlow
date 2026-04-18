using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal static class UnitOperationSmokeBoundarySuite
{
    public static void Run(SmokeOptions options, string projectJson)
    {
        using var driver = new UnitOperationSmokeHostDriver(options, projectJson);

        var preInitializeAttempt = driver.Calculate();
        var preInitializeError = preInitializeAttempt.ExpectFailure<CapeBadInvocationOrderException>(
            UnitOperationHostDriverFailureKind.InvocationOrder,
            "calculate before initialize");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            string.Equals(
                preInitializeError.RequestedOperation,
                nameof(RadishFlowCapeOpenUnitOperation.Initialize),
                StringComparison.Ordinal),
            "pre-initialize calculate should be classified as invocation-order failure and request Initialize().");

        driver.Initialize();
        var initialBundle = driver.ReadReport();
        var initialReport = initialBundle.Snapshot;
        UnitOperationSmokeReportAssertions.AssertEmpty(initialBundle, "empty calculation report");

        var parameters = driver.Parameters;
        var ports = driver.Ports;
        var parameterCollection = driver.ParameterCollection;
        var portCollection = driver.PortCollection;
        Console.WriteLine("== Unit Collections ==");
        Console.WriteLine($"Parameters.Count(): {parameterCollection.Count()}");
        Console.WriteLine($"Ports.Count(): {portCollection.Count()}");
        Console.WriteLine();

        var flowsheetParameter = driver.FlowsheetParameter;
        var packageIdParameter = driver.PackageIdParameter;
        var manifestPathParameter = driver.ManifestPathParameter;
        var payloadPathParameter = driver.PayloadPathParameter;
        var feedPort = driver.FeedPort;
        var productPort = driver.ProductPort;

        EnsureSameReference(parameters[0], flowsheetParameter, "parameter collection name lookup");
        EnsureSameReference(ports[0], feedPort, "port collection name lookup");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            flowsheetParameter.ValueKind == UnitOperationParameterValueKind.StructuredJsonText,
            "flowsheet parameter should expose structured JSON metadata.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            packageIdParameter.ValueKind == UnitOperationParameterValueKind.Identifier,
            "package parameter should expose identifier metadata.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            manifestPathParameter.ValueKind == UnitOperationParameterValueKind.FilePath,
            "manifest parameter should expose file path metadata.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            !flowsheetParameter.AllowsEmptyValue,
            "flowsheet parameter should not allow empty text.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            string.Equals(
                manifestPathParameter.RequiredCompanionParameterName,
                payloadPathParameter.ComponentName,
                StringComparison.Ordinal),
            "manifest parameter should declare payload companion metadata.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            string.Equals(
                payloadPathParameter.RequiredCompanionParameterName,
                manifestPathParameter.ComponentName,
                StringComparison.Ordinal),
            "payload parameter should declare manifest companion metadata.");

        var invalidJsonMessage = string.Empty;
        flowsheetParameter.value = "{ invalid json";
        UnitOperationSmokeReportAssertions.EnsureCondition(
            !flowsheetParameter.Validate(ref invalidJsonMessage),
            "flowsheet parameter should reject invalid JSON text.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            invalidJsonMessage.Contains("valid JSON text", StringComparison.Ordinal),
            "invalid JSON validation should mention JSON text.");
        ExpectCapeInvalidArgument(() => feedPort.Connect(new object()), "port connect with plain object");
        ExpectCapeInvalidArgument(
            () => feedPort.Connect(new InvalidSmokeConnectedObject("   ")),
            "port connect with blank ComponentName");

        driver.ConfigureMinimumInputs(includePackageId: false);
        driver.ConnectRequiredPorts();

        var validationFailureAttempt = driver.Calculate();
        var validationFailureError = validationFailureAttempt.ExpectFailure<CapeBadInvocationOrderException>(
            UnitOperationHostDriverFailureKind.Validation,
            "calculate without property package id");
        var validationFailureReport = validationFailureAttempt.Report.Snapshot;
        UnitOperationSmokeReportAssertions.AssertFailure(
            validationFailureAttempt.Report,
            validationFailureError,
            "validation failure host report",
            UnitOperationCalculationReportDetailCatalog
                .GetStableKeyOrder(UnitOperationCalculationReportState.Failure)
                .Where(key => string.Equals(key, UnitOperationCalculationReportDetailCatalog.Error, StringComparison.Ordinal) ||
                              string.Equals(key, UnitOperationCalculationReportDetailCatalog.Operation, StringComparison.Ordinal) ||
                              string.Equals(key, UnitOperationCalculationReportDetailCatalog.RequestedOperation, StringComparison.Ordinal))
                .ToArray(),
            expectedRequestedOperation: UnitOperationParameterCatalog.PropertyPackageId.ConfigurationOperationName);
        UnitOperationSmokeReportAssertions.EnsureCondition(
            validationFailureReport.ScalarLines.Count == validationFailureReport.DetailKeyCount + 1,
            "validation failure host report should expose headline plus stable detail entries.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            validationFailureReport.Text.Contains(validationFailureReport.Headline, StringComparison.Ordinal) &&
            validationFailureReport.Text.Contains($"requestedOperation={UnitOperationParameterCatalog.PropertyPackageId.ConfigurationOperationName}", StringComparison.Ordinal),
            "validation failure host report text should include both the headline and requested operation.");

        packageIdParameter.value = "missing-package-for-smoke";
        var nativeFailureAttempt = driver.Calculate();
        var nativeFailureError = nativeFailureAttempt.ExpectFailure<CapeInvalidArgumentException>(
            UnitOperationHostDriverFailureKind.Native,
            "calculate with missing property package id");
        var nativeFailureReport = nativeFailureAttempt.Report.Snapshot;
        UnitOperationSmokeReportAssertions.AssertFailure(
            nativeFailureAttempt.Report,
            nativeFailureError,
            "native failure host report",
            UnitOperationCalculationReportDetailCatalog
                .GetStableKeyOrder(UnitOperationCalculationReportState.Failure)
                .Where(key => string.Equals(key, UnitOperationCalculationReportDetailCatalog.Error, StringComparison.Ordinal) ||
                              string.Equals(key, UnitOperationCalculationReportDetailCatalog.Operation, StringComparison.Ordinal) ||
                              string.Equals(key, UnitOperationCalculationReportDetailCatalog.NativeStatus, StringComparison.Ordinal))
                .ToArray(),
            expectedNativeStatus: "MissingEntity");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            nativeFailureReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.RequestedOperation) is null &&
            nativeFailureReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.DiagnosticCode) is null,
            "native failure host report should not invent optional failure details.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            nativeFailureReport.ScalarLines.Count >= 3 &&
            string.Equals(nativeFailureReport.ScalarLines[0], nativeFailureReport.Headline, StringComparison.Ordinal),
            "native failure host report lines should start with the headline before detail lines.");

        packageIdParameter.value = options.PackageId;

        var validationResult = driver.Validate();
        Console.WriteLine("== Unit Validation ==");
        Console.WriteLine($"Valid: {validationResult.IsValid}");
        Console.WriteLine($"Message: {validationResult.Message}");
        Console.WriteLine();
        Console.WriteLine("== Parameter Metadata ==");
        Console.WriteLine($"{flowsheetParameter.ComponentName}: kind={flowsheetParameter.ValueKind}, default={(flowsheetParameter.DefaultValue ?? "<null>")}, allowEmpty={flowsheetParameter.AllowsEmptyValue}, companion={(flowsheetParameter.RequiredCompanionParameterName ?? "<none>")}");
        Console.WriteLine($"{packageIdParameter.ComponentName}: kind={packageIdParameter.ValueKind}, default={(packageIdParameter.DefaultValue ?? "<null>")}, allowEmpty={packageIdParameter.AllowsEmptyValue}, companion={(packageIdParameter.RequiredCompanionParameterName ?? "<none>")}");
        Console.WriteLine($"{manifestPathParameter.ComponentName}: kind={manifestPathParameter.ValueKind}, default={(manifestPathParameter.DefaultValue ?? "<null>")}, allowEmpty={manifestPathParameter.AllowsEmptyValue}, companion={(manifestPathParameter.RequiredCompanionParameterName ?? "<none>")}");
        Console.WriteLine($"{payloadPathParameter.ComponentName}: kind={payloadPathParameter.ValueKind}, default={(payloadPathParameter.DefaultValue ?? "<null>")}, allowEmpty={payloadPathParameter.AllowsEmptyValue}, companion={(payloadPathParameter.RequiredCompanionParameterName ?? "<none>")}");
        Console.WriteLine();

        if (!validationResult.IsValid)
        {
            throw new InvalidOperationException("Unit operation validation failed before Calculate().");
        }

        var successAttempt = driver.Calculate();
        if (!successAttempt.Succeeded)
        {
            throw new InvalidOperationException(
                $"Expected successful unit operation calculation, but received {successAttempt.Failure?.GetType().Name ?? "<unknown>"}.");
        }

        var successReport = successAttempt.Report.Snapshot;
        var successDocument = successAttempt.Report.Document;
        UnitOperationSmokeReportAssertions.AssertSuccess(successAttempt.Report, "success host report");

        var repeatedSuccessAttempt = driver.Calculate();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            repeatedSuccessAttempt.Succeeded &&
            repeatedSuccessAttempt.Failure is null &&
            repeatedSuccessAttempt.FailureKind is null,
            "repeated Calculate() on a stable unit should continue to succeed.");
        UnitOperationSmokeReportAssertions.AssertRepeatedSuccessShape(
            repeatedSuccessAttempt.Report,
            "repeated Calculate()");

        feedPort.Disconnect();
        var disconnectedPortValidation = driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            !disconnectedPortValidation.IsValid &&
            disconnectedPortValidation.Message.Contains("Required port `Feed` is not connected.", StringComparison.Ordinal),
            "disconnecting a required port should make Validate() fail with the required-port message.");
        var disconnectedPortReport = driver.ReadReport().Snapshot;
        UnitOperationSmokeReportAssertions.EnsureCondition(
            disconnectedPortReport.State == UnitOperationCalculationReportState.None,
            "disconnecting a required port should clear the last calculation report until the unit is driven again.");
        feedPort.Connect(new SmokeConnectedObject("Reconnected Feed"));
        var reconnectedPortValidation = driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            reconnectedPortValidation.IsValid,
            "reconnecting the required port should restore a valid minimal host configuration.");

        payloadPathParameter.value = null;
        var companionValidation = driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            !companionValidation.IsValid &&
            companionValidation.Message.Contains("must be configured together", StringComparison.Ordinal),
            "breaking the manifest/payload pair should fail validation with the companion-parameter message.");
        var companionFailureAttempt = driver.Calculate();
        var companionFailureError = companionFailureAttempt.ExpectFailure<CapeBadInvocationOrderException>(
            UnitOperationHostDriverFailureKind.Validation,
            "calculate with incomplete manifest/payload pair");
        var companionFailureReport = companionFailureAttempt.Report.Snapshot;
        UnitOperationSmokeReportAssertions.EnsureCondition(
            string.Equals(
                companionFailureError.RequestedOperation,
                UnitOperationParameterCatalog.PropertyPackageManifestPath.ConfigurationOperationName,
                StringComparison.Ordinal) &&
            string.Equals(
                companionFailureReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.RequestedOperation),
                UnitOperationParameterCatalog.PropertyPackageManifestPath.ConfigurationOperationName,
                StringComparison.Ordinal),
            "companion-parameter validation failure should point the host back to LoadPropertyPackageFiles().");
        driver.ConfigureMinimumInputs(includePackageId: true);
        var recoveredValidation = driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            recoveredValidation.IsValid,
            "restoring manifest/payload inputs should recover the unit to a valid host configuration.");
        var recoveredSuccessAttempt = driver.Calculate();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            recoveredSuccessAttempt.Succeeded,
            "after restoring companion inputs, Calculate() should succeed again.");

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

        driver.Terminate();
        var terminatedBundle = driver.ReadReport();
        UnitOperationSmokeReportAssertions.AssertEmpty(terminatedBundle, "terminated host report");
        UnitOperationSmokeReportAssertions.EnsureCondition(!feedPort.IsConnected, "feed port should release its connected object during Terminate().");
        UnitOperationSmokeReportAssertions.EnsureCondition(!productPort.IsConnected, "product port should release its connected object during Terminate().");
        var postTerminateValidation = driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            !postTerminateValidation.IsValid &&
            postTerminateValidation.Message.Contains("Terminate has already been called", StringComparison.Ordinal),
            "Validate() after Terminate() should return an invalid termination message instead of reactivating the unit.");
        ExpectCapeBadInvOrder(() => driver.Initialize(), "initialize after terminate");
        var postTerminateCalculateAttempt = driver.Calculate();
        var postTerminateCalculateError = postTerminateCalculateAttempt.ExpectFailure<CapeBadInvocationOrderException>(
            UnitOperationHostDriverFailureKind.Validation,
            "calculate after terminate");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            string.Equals(postTerminateCalculateError.Operation, nameof(RadishFlowCapeOpenUnitOperation.Calculate), StringComparison.Ordinal),
            "calculate after terminate should fail at the Calculate() boundary.");
        ExpectCapeBadInvOrder(() => _ = parameterCollection.Count(), "parameter collection count after terminate");
        ExpectCapeBadInvOrder(() => _ = flowsheetParameter.value, "parameter value get after terminate");
        ExpectCapeBadInvOrder(() => feedPort.Connect(new SmokeConnectedObject("Late Feed")), "port connect after terminate");
        ExpectCapeBadInvOrder(() => _ = feedPort.connectedObject, "port connectedObject after terminate");
    }

    private static void EnsureSameReference<T>(T expected, T actual, string scenario)
        where T : class
    {
        if (!ReferenceEquals(expected, actual))
        {
            throw new InvalidOperationException($"Unexpected object instance returned for {scenario}.");
        }
    }

    private static CapeBadInvocationOrderException ExpectCapeBadInvOrder(Action action, string scenario)
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

    private static CapeInvalidArgumentException ExpectCapeInvalidArgument(Action action, string scenario)
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
}
