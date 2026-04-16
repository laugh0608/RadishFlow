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
        EnsureCondition(
            string.Equals(
                preInitializeError.RequestedOperation,
                nameof(RadishFlowCapeOpenUnitOperation.Initialize),
                StringComparison.Ordinal),
            "pre-initialize calculate should be classified as invocation-order failure and request Initialize().");

        driver.Initialize();
        var initialBundle = driver.ReadReport();
        var initialReport = initialBundle.Snapshot;
        var initialPresentation = initialBundle.Presentation;
        var initialDocument = initialBundle.Document;
        EnsureCondition(
            initialReport.State == UnitOperationCalculationReportState.None,
            "unit operation should expose an empty calculation report after Initialize().");
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
        EnsureCondition(
            flowsheetParameter.ValueKind == UnitOperationParameterValueKind.StructuredJsonText,
            "flowsheet parameter should expose structured JSON metadata.");
        EnsureCondition(
            packageIdParameter.ValueKind == UnitOperationParameterValueKind.Identifier,
            "package parameter should expose identifier metadata.");
        EnsureCondition(
            manifestPathParameter.ValueKind == UnitOperationParameterValueKind.FilePath,
            "manifest parameter should expose file path metadata.");
        EnsureCondition(
            !flowsheetParameter.AllowsEmptyValue,
            "flowsheet parameter should not allow empty text.");
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

        driver.ConfigureMinimumInputs(includePackageId: false);
        driver.ConnectRequiredPorts();

        var validationFailureAttempt = driver.Calculate();
        var validationFailureError = validationFailureAttempt.ExpectFailure<CapeBadInvocationOrderException>(
            UnitOperationHostDriverFailureKind.Validation,
            "calculate without property package id");
        var validationFailureReport = validationFailureAttempt.Report.Snapshot;
        var validationFailurePresentation = validationFailureAttempt.Report.Presentation;
        var validationFailureDocument = validationFailureAttempt.Report.Document;
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
        var nativeFailureAttempt = driver.Calculate();
        var nativeFailureError = nativeFailureAttempt.ExpectFailure<CapeInvalidArgumentException>(
            UnitOperationHostDriverFailureKind.Native,
            "calculate with missing property package id");
        var nativeFailureReport = nativeFailureAttempt.Report.Snapshot;
        var nativeFailurePresentation = nativeFailureAttempt.Report.Presentation;
        var nativeFailureDocument = nativeFailureAttempt.Report.Document;
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
        var successPresentation = successAttempt.Report.Presentation;
        var successDocument = successAttempt.Report.Document;
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

        var repeatedSuccessAttempt = driver.Calculate();
        EnsureCondition(
            repeatedSuccessAttempt.Succeeded &&
            repeatedSuccessAttempt.Failure is null &&
            repeatedSuccessAttempt.FailureKind is null,
            "repeated Calculate() on a stable unit should continue to succeed.");
        var repeatedSuccessReport = repeatedSuccessAttempt.Report.Snapshot;
        EnsureCondition(
            repeatedSuccessReport.State == UnitOperationCalculationReportState.Success &&
            repeatedSuccessReport.DetailKeys.SequenceEqual(
                UnitOperationCalculationReportDetailCatalog.SuccessStableKeyOrder,
                StringComparer.Ordinal),
            "repeated Calculate() should preserve the frozen success report shape.");

        feedPort.Disconnect();
        var disconnectedPortValidation = driver.Validate();
        EnsureCondition(
            !disconnectedPortValidation.IsValid &&
            disconnectedPortValidation.Message.Contains("Required port `Feed` is not connected.", StringComparison.Ordinal),
            "disconnecting a required port should make Validate() fail with the required-port message.");
        var disconnectedPortReport = driver.ReadReport().Snapshot;
        EnsureCondition(
            disconnectedPortReport.State == UnitOperationCalculationReportState.None,
            "disconnecting a required port should clear the last calculation report until the unit is driven again.");
        feedPort.Connect(new SmokeConnectedObject("Reconnected Feed"));
        var reconnectedPortValidation = driver.Validate();
        EnsureCondition(
            reconnectedPortValidation.IsValid,
            "reconnecting the required port should restore a valid minimal host configuration.");

        payloadPathParameter.value = null;
        var companionValidation = driver.Validate();
        EnsureCondition(
            !companionValidation.IsValid &&
            companionValidation.Message.Contains("must be configured together", StringComparison.Ordinal),
            "breaking the manifest/payload pair should fail validation with the companion-parameter message.");
        var companionFailureAttempt = driver.Calculate();
        var companionFailureError = companionFailureAttempt.ExpectFailure<CapeBadInvocationOrderException>(
            UnitOperationHostDriverFailureKind.Validation,
            "calculate with incomplete manifest/payload pair");
        var companionFailureReport = companionFailureAttempt.Report.Snapshot;
        EnsureCondition(
            string.Equals(
                companionFailureError.RequestedOperation,
                nameof(RadishFlowCapeOpenUnitOperation.LoadPropertyPackageFiles),
                StringComparison.Ordinal) &&
            string.Equals(
                companionFailureReport.GetDetailValue(UnitOperationCalculationReportDetailCatalog.RequestedOperation),
                nameof(RadishFlowCapeOpenUnitOperation.LoadPropertyPackageFiles),
                StringComparison.Ordinal),
            "companion-parameter validation failure should point the host back to LoadPropertyPackageFiles().");
        driver.ConfigureMinimumInputs(includePackageId: true);
        var recoveredValidation = driver.Validate();
        EnsureCondition(
            recoveredValidation.IsValid,
            "restoring manifest/payload inputs should recover the unit to a valid host configuration.");
        var recoveredSuccessAttempt = driver.Calculate();
        EnsureCondition(
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
        var terminatedReport = terminatedBundle.Snapshot;
        var terminatedPresentation = terminatedBundle.Presentation;
        var terminatedDocument = terminatedBundle.Document;
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
        var postTerminateValidation = driver.Validate();
        EnsureCondition(
            !postTerminateValidation.IsValid &&
            postTerminateValidation.Message.Contains("Terminate has already been called", StringComparison.Ordinal),
            "Validate() after Terminate() should return an invalid termination message instead of reactivating the unit.");
        ExpectCapeBadInvOrder(() => driver.Initialize(), "initialize after terminate");
        var postTerminateCalculateAttempt = driver.Calculate();
        var postTerminateCalculateError = postTerminateCalculateAttempt.ExpectFailure<CapeBadInvocationOrderException>(
            UnitOperationHostDriverFailureKind.Validation,
            "calculate after terminate");
        EnsureCondition(
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

    private static void EnsureCondition(bool condition, string message)
    {
        if (!condition)
        {
            throw new InvalidOperationException(message);
        }
    }

    private static void EnsureHostReportLineApisAgree(UnitOperationHostReportSnapshot report, string scenario)
    {
        EnsureCondition(
            report.ScalarLines.SequenceEqual(report.VectorLines, StringComparer.Ordinal),
            $"{scenario} scalar and vector line exports should match.");
        EnsureCondition(
            string.Equals(report.Text, string.Join(Environment.NewLine, report.ScalarLines), StringComparison.Ordinal),
            $"{scenario} text export should match the scalar line export.");
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
