using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

Environment.ExitCode = ContractTestExecutable.Run(args);

internal static class ContractTestExecutable
{
    public static int Run(string[] args)
    {
        try
        {
            return Execute(args);
        }
        catch (CapeOpenException error)
        {
            Console.Error.WriteLine($"Contract test bootstrap failed at CAPE-OPEN operation: {error.Operation}");
            if (!string.IsNullOrWhiteSpace(error.NativeStatus))
            {
                Console.Error.WriteLine($"Native Status: {error.NativeStatus}");
            }

            Console.Error.WriteLine($"Message: {error.Message}");
            Console.Error.WriteLine(error);
            return 2;
        }
        catch (Exception error)
        {
            Console.Error.WriteLine("Contract test bootstrap failed with an unhandled exception.");
            Console.Error.WriteLine(error);
            return 2;
        }
    }

    private static int Execute(string[] args)
    {
        var options = ContractTestOptions.Parse(args);
        var tests = new (string Name, Action<ContractTestContext> Execute)[]
        {
            ("collection-contract", static context => ContractTests.Collections_ExposeStableLookupAndRejectInvalidSelectors(context)),
            ("configuration-contract", static context => ContractTests.ConfigurationSnapshot_ExposesReadinessAndNextOperations(context)),
            ("parameter-contract", static context => ContractTests.Parameters_ResetValidationStateAndFreezeMetadata(context)),
            ("port-contract", static context => ContractTests.Ports_RequireDisconnectBeforeReplacingConnections(context)),
            ("validate-before-initialize", static context => ContractTests.ValidateBeforeInitialize_ReturnsInvalidAndEmptyReport(context)),
            ("validation-failure-report", static context => ContractTests.CalculateValidationFailure_PopulatesFailureReport(context)),
            ("companion-validation-report", static context => ContractTests.CalculateCompanionValidationFailure_PopulatesFailureReport(context)),
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
            return 1;
        }

        Console.WriteLine($"Executed {selectedTests.Length} contract test(s).");
        return 0;
    }
}

internal static class ContractTests
{
    public static void Collections_ExposeStableLookupAndRejectInvalidSelectors(ContractTestContext context)
    {
        context.Initialize();

        ContractAssert.Equal(4, context.ParameterCollection.Count(), "Parameter collection Count() should remain stable.");
        ContractAssert.Equal(2, context.PortCollection.Count(), "Port collection Count() should remain stable.");
        ContractAssert.Equal(4, context.UnitOperation.Parameters.Count, "Parameter collection IReadOnlyList.Count should stay aligned with ICapeCollection.Count().");
        ContractAssert.Equal(2, context.UnitOperation.Ports.Count, "Port collection IReadOnlyList.Count should stay aligned with ICapeCollection.Count().");
        ContractAssert.SequenceEqual(
            UnitOperationParameterCatalog.OrderedNames,
            context.UnitOperation.Parameters.Select(static parameter => parameter.ComponentName),
            "Parameter collection order should match the frozen public catalog.");
        ContractAssert.SequenceEqual(
            UnitOperationPortCatalog.OrderedNames,
            context.UnitOperation.Ports.Select(static port => port.ComponentName),
            "Port collection order should match the frozen public catalog.");
        ContractAssert.True(context.UnitOperation.Parameters.ContainsName(UnitOperationParameterCatalog.FlowsheetJson.Name), "Typed parameter collection should report known canonical names.");
        ContractAssert.True(context.UnitOperation.Ports.ContainsName(UnitOperationPortCatalog.Product.Name), "Typed port collection should report known canonical names.");
        ContractAssert.False(context.UnitOperation.Parameters.ContainsName("missing-parameter"), "Typed parameter collection should reject unknown names.");
        ContractAssert.True(
            context.UnitOperation.Parameters.TryGetByName("flowsheet json", out var typedFlowsheetParameter) &&
            ReferenceEquals(context.FlowsheetParameter, typedFlowsheetParameter),
            "Typed parameter collection should support case-insensitive lookup.");
        ContractAssert.True(
            context.UnitOperation.Ports.TryGetByName("product", out var typedProductPort) &&
            ReferenceEquals(context.ProductPort, typedProductPort),
            "Typed port collection should support case-insensitive lookup.");
        ContractAssert.False(
            context.UnitOperation.Parameters.TryGetByName("   ", out var blankParameterLookup) &&
            blankParameterLookup is not null,
            "Typed parameter collection should reject blank lookup names.");
        ContractAssert.SameReference(
            context.PayloadPathParameter,
            context.UnitOperation.Parameters.GetByOneBasedIndex(4),
            "Typed parameter collection should support 1-based numeric lookup.");
        ContractAssert.SameReference(
            context.ProductPort,
            context.UnitOperation.Ports.GetByName(UnitOperationPortCatalog.Product.Name),
            "Typed port collection should support canonical name lookup.");
        foreach (var parameter in context.UnitOperation.Parameters)
        {
            var definition = UnitOperationParameterCatalog.GetByName(parameter.ComponentName);
            ContractAssert.Equal(definition.Description, parameter.ComponentDescription, "Runtime parameter description should match the frozen catalog definition.");
            ContractAssert.Equal(definition.IsRequired, parameter.IsRequired, "Runtime parameter required flag should match the frozen catalog definition.");
            ContractAssert.Equal(definition.ValueKind, parameter.ValueKind, "Runtime parameter value kind should match the frozen catalog definition.");
            ContractAssert.Equal(definition.AllowsEmptyValue, parameter.AllowsEmptyValue, "Runtime parameter allow-empty flag should match the frozen catalog definition.");
            ContractAssert.Equal(definition.RequiredCompanionParameterName, parameter.RequiredCompanionParameterName, "Runtime parameter companion contract should match the frozen catalog definition.");
            ContractAssert.Equal(definition.Mode, parameter.Mode, "Runtime parameter mode should match the frozen catalog definition.");
            ContractAssert.Equal(definition.DefaultValue, parameter.DefaultValue, "Runtime parameter default value should match the frozen catalog definition.");
        }

        foreach (var port in context.UnitOperation.Ports)
        {
            var definition = UnitOperationPortCatalog.GetByName(port.ComponentName);
            ContractAssert.Equal(definition.Description, port.ComponentDescription, "Runtime port description should match the frozen catalog definition.");
            ContractAssert.Equal(definition.Direction, port.direction, "Runtime port direction should match the frozen catalog definition.");
            ContractAssert.Equal(definition.PortType, port.portType, "Runtime port type should match the frozen catalog definition.");
            ContractAssert.Equal(definition.IsRequired, port.IsRequired, "Runtime port required flag should match the frozen catalog definition.");
        }

        ContractAssert.SameReference(
            context.FlowsheetParameter,
            context.ParameterCollection.Item(1),
            "Parameter collection should support 1-based numeric lookup.");
        ContractAssert.SameReference(
            context.PayloadPathParameter,
            context.ParameterCollection.Item(4.0d),
            "Parameter collection should accept whole-number floating selectors from COM callers.");
        ContractAssert.SameReference(
            context.ProductPort,
            context.PortCollection.Item("Product"),
            "Port collection should support case-insensitive name lookup.");

        var blankNameError = ContractAssert.Throws<CapeInvalidArgumentException>(
            () => context.ParameterCollection.Item("   "),
            "Blank collection names should be rejected.");
        ContractAssert.Contains(blankNameError.Description, "requires a non-empty component name", "Blank collection names should produce a clear validation message.");

        var outOfRangeError = ContractAssert.Throws<CapeInvalidArgumentException>(
            () => context.PortCollection.Item(0),
            "Out-of-range collection selectors should be rejected.");
        ContractAssert.Contains(outOfRangeError.Description, "out of range", "Out-of-range collection selectors should mention the stable 1-based bounds.");

        var fractionalError = ContractAssert.Throws<CapeInvalidArgumentException>(
            () => context.ParameterCollection.Item(1.5d),
            "Fractional collection selectors should be rejected.");
        ContractAssert.Contains(fractionalError.Description, "1-based integer index", "Fractional selectors should keep the collection contract explicit.");

        var collectionMutationError = ContractAssert.Throws<CapeInvalidArgumentException>(
            () => ((ICapeIdentification)context.UnitOperation.Parameters).ComponentName = "Mutable Parameters",
            "Collection identification should remain immutable in the MVP runtime.");
        ContractAssert.Contains(collectionMutationError.Description, "does not allow ComponentName mutation", "Collection immutability failures should stay explicit.");
        var typedMissingParameterError = ContractAssert.Throws<CapeInvalidArgumentException>(
            () => context.UnitOperation.Parameters.GetByName("missing-parameter"),
            "Typed parameter collection should reject unknown names.");
        ContractAssert.Contains(typedMissingParameterError.Description, "does not contain an item named", "Typed parameter collection missing-name failures should stay explicit.");
        var typedOutOfRangePortError = ContractAssert.Throws<CapeInvalidArgumentException>(
            () => context.UnitOperation.Ports.GetByOneBasedIndex(0),
            "Typed port collection should reject out-of-range indices.");
        ContractAssert.Contains(typedOutOfRangePortError.Description, "out of range", "Typed port collection out-of-range failures should stay explicit.");

        ContractAssert.True(
            UnitOperationParameterCatalog.TryGetByName("flowsheet json", out var flowsheetDefinition) &&
            ReferenceEquals(UnitOperationParameterCatalog.FlowsheetJson, flowsheetDefinition),
            "Parameter catalog should support case-insensitive lookup.");
        ContractAssert.Equal(
            nameof(RadishFlowCapeOpenUnitOperation.LoadFlowsheetJson),
            UnitOperationParameterCatalog.FlowsheetJson.ConfigurationOperationName,
            "Flowsheet parameter should point hosts back to LoadFlowsheetJson().");
        ContractAssert.Equal(
            nameof(RadishFlowCapeOpenUnitOperation.SelectPropertyPackage),
            UnitOperationParameterCatalog.PropertyPackageId.ConfigurationOperationName,
            "Package id parameter should point hosts back to SelectPropertyPackage().");
        ContractAssert.Equal(
            nameof(RadishFlowCapeOpenUnitOperation.LoadPropertyPackageFiles),
            UnitOperationParameterCatalog.PropertyPackageManifestPath.ConfigurationOperationName,
            "Manifest parameter should point hosts back to LoadPropertyPackageFiles().");
        ContractAssert.Equal(
            UnitOperationParameterCatalog.PropertyPackageManifestPath.ConfigurationOperationName,
            UnitOperationParameterCatalog.PropertyPackagePayloadPath.ConfigurationOperationName,
            "Companion parameters should share the same configuration operation.");
        ContractAssert.True(
            UnitOperationPortCatalog.TryGetByName("product", out var productDefinition) &&
            ReferenceEquals(UnitOperationPortCatalog.Product, productDefinition),
            "Port catalog should support case-insensitive lookup.");
        ContractAssert.Equal(
            nameof(RadishFlowCapeOpenUnitOperation.SetPortConnected),
            UnitOperationPortCatalog.Feed.ConnectionOperationName,
            "Feed port should point hosts back to SetPortConnected().");
        ContractAssert.Equal(
            UnitOperationPortCatalog.Feed.ConnectionOperationName,
            UnitOperationPortCatalog.Product.ConnectionOperationName,
            "Required ports should share the same connection operation in the current MVP runtime.");
        var missingParameterDefinitionError = ContractAssert.Throws<ArgumentException>(
            () => UnitOperationParameterCatalog.GetByName("missing-parameter"),
            "Unknown parameter definitions should be rejected by the catalog.");
        ContractAssert.Contains(missingParameterDefinitionError.Message, "Unknown unit operation parameter definition", "Missing parameter definition failures should stay explicit.");
        var missingPortDefinitionError = ContractAssert.Throws<ArgumentException>(
            () => UnitOperationPortCatalog.GetByName("missing-port"),
            "Unknown port definitions should be rejected by the catalog.");
        ContractAssert.Contains(missingPortDefinitionError.Message, "Unknown unit operation port definition", "Missing port definition failures should stay explicit.");
    }

    public static void Parameters_ResetValidationStateAndFreezeMetadata(ContractTestContext context)
    {
        context.Initialize();

        var specification = (ICapeParameterSpec)context.ManifestPathParameter.Specification;
        ContractAssert.NotSameReference(context.ManifestPathParameter, specification, "Specification should no longer alias the parameter instance.");
        ContractAssert.SameReference(specification, context.ManifestPathParameter.Specification, "Specification should remain a stable object across repeated reads.");
        ContractAssert.Equal(CapeParamType.CAPE_OPTION, specification.Type, "Manifest parameter spec should expose CAPE_OPTION.");
        ContractAssert.Equal(0, specification.Dimensionality.Length, "Manifest parameter spec should expose empty dimensionality for option values.");
        ContractAssert.Equal(CapeParamMode.CAPE_INPUT, context.ManifestPathParameter.Mode, "Manifest parameter should keep its frozen input mode.");
        context.ManifestPathParameter.Mode = CapeParamMode.CAPE_INPUT;

        var modeMutationError = ContractAssert.Throws<CapeInvalidArgumentException>(
            () => context.ManifestPathParameter.Mode = CapeParamMode.CAPE_OUTPUT,
            "Parameter mode mutation should be rejected in the MVP runtime.");
        ContractAssert.Contains(modeMutationError.Description, "does not allow Mode mutation", "Parameter mode mutation failures should stay explicit.");

        context.ManifestPathParameter.value = context.ManifestPath;
        var message = string.Empty;
        var valid = context.ManifestPathParameter.Validate(ref message);

        ContractAssert.True(valid, "Configured manifest path parameter should validate.");
        ContractAssert.Equal(CapeValidationStatus.Valid, context.ManifestPathParameter.ValStatus, "Successful parameter validation should set ValStatus to Valid.");
        ContractAssert.True(context.ManifestPathParameter.IsConfigured, "Configured manifest path should report IsConfigured.");
        ContractAssert.Equal(context.ManifestPath, context.ManifestPathParameter.Value, "Configured manifest path should round-trip through Value.");

        context.ManifestPathParameter.Reset();

        ContractAssert.Equal(CapeValidationStatus.NotValidated, context.ManifestPathParameter.ValStatus, "Reset() should always return parameter validation state to NotValidated.");
        ContractAssert.False(context.ManifestPathParameter.IsConfigured, "Reset() should restore the default unconfigured state for optional parameters.");
        ContractAssert.Null(context.ManifestPathParameter.Value, "Reset() should restore the default null value for optional parameters.");

        var parameterMutationError = ContractAssert.Throws<CapeInvalidArgumentException>(
            () => ((ICapeIdentification)context.ManifestPathParameter).ComponentDescription = "Mutated manifest parameter",
            "Parameter identification should remain immutable.");
        ContractAssert.Contains(parameterMutationError.Description, "does not allow ComponentDescription mutation", "Parameter immutability failures should stay explicit.");

        context.UnitOperation.Terminate();

        var postTerminateValueError = ContractAssert.Throws<CapeBadInvocationOrderException>(
            () => _ = context.ManifestPathParameter.Value,
            "Value access after Terminate() should be blocked.");
        ContractAssert.Contains(postTerminateValueError.Description, "Terminate has already been called", "Post-terminate Value access should preserve lifecycle guidance.");

        var postTerminateConfiguredError = ContractAssert.Throws<CapeBadInvocationOrderException>(
            () => _ = context.ManifestPathParameter.IsConfigured,
            "IsConfigured access after Terminate() should be blocked.");
        ContractAssert.Contains(postTerminateConfiguredError.Description, "Terminate has already been called", "Post-terminate IsConfigured access should preserve lifecycle guidance.");

        var postTerminateSpecError = ContractAssert.Throws<CapeBadInvocationOrderException>(
            () => _ = specification.Type,
            "Specification access after Terminate() should remain lifecycle-guarded.");
        ContractAssert.Contains(postTerminateSpecError.Description, "Terminate has already been called", "Post-terminate spec access should preserve lifecycle guidance.");
    }

    public static void ConfigurationSnapshot_ExposesReadinessAndNextOperations(ContractTestContext context)
    {
        var constructedSnapshot = context.ReadConfiguration();
        ContractAssert.Equal(UnitOperationHostConfigurationState.Constructed, constructedSnapshot.State, "Configuration snapshot should expose constructed state before Initialize().");
        ContractAssert.False(constructedSnapshot.IsReadyForCalculate, "Configuration snapshot should not report ready before Initialize().");
        ContractAssert.Equal(4, constructedSnapshot.ParameterEntries.Count, "Configuration snapshot should expose parameter entries in frozen catalog order.");
        ContractAssert.Equal(2, constructedSnapshot.PortEntries.Count, "Configuration snapshot should expose port entries in frozen catalog order.");
        ContractAssert.Equal(UnitOperationHostConfigurationIssueKind.InitializeRequired, constructedSnapshot.BlockingIssues[0].Kind, "Constructed configuration snapshot should lead with InitializeRequired.");
        ContractAssert.True(constructedSnapshot.ContainsNextOperation(nameof(RadishFlowCapeOpenUnitOperation.Initialize)), "Constructed configuration snapshot should direct the host to Initialize().");
        ContractAssert.True(constructedSnapshot.ContainsNextOperation(UnitOperationParameterCatalog.FlowsheetJson.ConfigurationOperationName), "Constructed configuration snapshot should already expose the flowsheet configuration operation.");
        ContractAssert.True(constructedSnapshot.ContainsNextOperation(UnitOperationParameterCatalog.PropertyPackageId.ConfigurationOperationName), "Constructed configuration snapshot should already expose the package selection operation.");
        ContractAssert.True(constructedSnapshot.ContainsNextOperation(UnitOperationPortCatalog.Feed.ConnectionOperationName), "Constructed configuration snapshot should already expose the port connection operation.");
        ContractAssert.False(constructedSnapshot.GetParameter(UnitOperationParameterCatalog.FlowsheetJson.Name).IsConfigured, "Flowsheet parameter should start unconfigured in configuration snapshot.");
        ContractAssert.False(constructedSnapshot.GetPort(UnitOperationPortCatalog.Product.Name).IsConnected, "Product port should start disconnected in configuration snapshot.");

        context.Initialize();
        context.LoadFlowsheet();
        context.SelectPackage();
        context.ConnectRequiredPorts();

        var readySnapshot = context.ReadConfiguration();
        ContractAssert.Equal(UnitOperationHostConfigurationState.Ready, readySnapshot.State, "Configuration snapshot should expose ready state once minimum calculate inputs are present.");
        ContractAssert.True(readySnapshot.IsReadyForCalculate, "Configuration snapshot should report ready once minimum calculate inputs are present.");
        ContractAssert.Equal(0, readySnapshot.BlockingIssueCount, "Ready configuration snapshot should not expose blocking issues.");
        ContractAssert.Equal(0, readySnapshot.NextOperations.Count, "Ready configuration snapshot should not expose follow-up operations.");
        ContractAssert.True(readySnapshot.GetParameter(UnitOperationParameterCatalog.FlowsheetJson.Name).IsConfigured, "Flowsheet parameter should report configured in ready configuration snapshot.");
        ContractAssert.True(readySnapshot.GetParameter(UnitOperationParameterCatalog.PropertyPackageId.Name).IsConfigured, "Package id parameter should report configured in ready configuration snapshot.");
        ContractAssert.False(readySnapshot.GetParameter(UnitOperationParameterCatalog.PropertyPackageManifestPath.Name).IsConfigured, "Optional manifest parameter should remain unconfigured until explicitly loaded.");
        ContractAssert.True(readySnapshot.GetPort(UnitOperationPortCatalog.Feed.Name).IsConnected, "Feed port should report connected in ready configuration snapshot.");
        ContractAssert.True(readySnapshot.GetPort(UnitOperationPortCatalog.Product.Name).IsConnected, "Product port should report connected in ready configuration snapshot.");

        context.ManifestPathParameter.value = context.ManifestPath;
        var companionMismatchSnapshot = context.ReadConfiguration();
        ContractAssert.Equal(UnitOperationHostConfigurationState.Incomplete, companionMismatchSnapshot.State, "Configuration snapshot should downgrade to incomplete when companion parameters diverge.");
        ContractAssert.False(companionMismatchSnapshot.IsReadyForCalculate, "Companion mismatch should clear configuration readiness.");
        ContractAssert.Equal(UnitOperationHostConfigurationIssueKind.CompanionParameterMismatch, companionMismatchSnapshot.BlockingIssues[0].Kind, "Companion mismatch should expose the matching issue kind.");
        ContractAssert.True(companionMismatchSnapshot.ContainsNextOperation(UnitOperationParameterCatalog.PropertyPackageManifestPath.ConfigurationOperationName), "Companion mismatch should direct the host to the shared property package file operation.");

        context.UnitOperation.Terminate();

        var terminatedSnapshot = context.ReadConfiguration();
        ContractAssert.Equal(UnitOperationHostConfigurationState.Terminated, terminatedSnapshot.State, "Configuration snapshot should expose terminated state after Terminate().");
        ContractAssert.False(terminatedSnapshot.IsReadyForCalculate, "Terminated configuration snapshot should not report ready.");
        ContractAssert.Equal(1, terminatedSnapshot.BlockingIssueCount, "Terminated configuration snapshot should collapse to a single terminal issue.");
        ContractAssert.Equal(UnitOperationHostConfigurationIssueKind.Terminated, terminatedSnapshot.BlockingIssues[0].Kind, "Terminated configuration snapshot should expose the terminal issue kind.");
        ContractAssert.Equal(0, terminatedSnapshot.NextOperations.Count, "Terminated configuration snapshot should not suggest recovery operations.");
        ContractAssert.Equal(0, terminatedSnapshot.ParameterEntries.Count, "Terminated configuration snapshot should not bypass lifecycle guards to expose parameter entries.");
        ContractAssert.Equal(0, terminatedSnapshot.PortEntries.Count, "Terminated configuration snapshot should not bypass lifecycle guards to expose port entries.");
    }

    public static void Ports_RequireDisconnectBeforeReplacingConnections(ContractTestContext context)
    {
        context.Initialize();

        var firstConnection = new ContractConnectedObject("Contract Feed A");
        var replacementConnection = new ContractConnectedObject("Contract Feed B");

        context.FeedPort.Connect(firstConnection);
        ContractAssert.SameReference(firstConnection, context.FeedPort.connectedObject, "Port should expose the connected object that was just attached.");

        context.FeedPort.Connect(firstConnection);
        ContractAssert.SameReference(firstConnection, context.FeedPort.connectedObject, "Reconnecting the same object should be a no-op.");

        var replacementError = ContractAssert.Throws<CapeBadInvocationOrderException>(
            () => context.FeedPort.Connect(replacementConnection),
            "Replacing a connected object without Disconnect() should be rejected.");
        ContractAssert.Equal(nameof(ICapeUnitPort.Disconnect), replacementError.RequestedOperation, "Port replacement failure should direct the host to Disconnect() first.");

        context.FeedPort.Disconnect();
        ContractAssert.Null(context.FeedPort.connectedObject, "Disconnect() should clear the connected object.");

        context.FeedPort.Connect(replacementConnection);
        ContractAssert.SameReference(replacementConnection, context.FeedPort.connectedObject, "Port should accept a new connection after Disconnect().");

        var portMutationError = ContractAssert.Throws<CapeInvalidArgumentException>(
            () => ((ICapeIdentification)context.FeedPort).ComponentName = "Mutated Feed",
            "Port identification should remain immutable.");
        ContractAssert.Contains(portMutationError.Description, "does not allow ComponentName mutation", "Port immutability failures should stay explicit.");
    }

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
        ContractAssert.Equal(UnitOperationParameterCatalog.PropertyPackageId.ConfigurationOperationName, context.UnitOperation.GetCalculationReportDetailValue(UnitOperationCalculationReportDetailCatalog.RequestedOperation), "Validation failure should point back to the package-id configuration operation frozen in the catalog.");
        ContractAssert.Null(context.UnitOperation.GetCalculationReportDetailValue(UnitOperationCalculationReportDetailCatalog.NativeStatus), "Validation failure should not invent native status.");
        ContractAssert.Null(context.UnitOperation.LastCalculationResult, "Validation failure should not preserve a stale success result.");
        ContractAssert.NotNull(context.UnitOperation.LastCalculationFailure, "Validation failure should preserve failure summary.");
    }

    public static void CalculateCompanionValidationFailure_PopulatesFailureReport(ContractTestContext context)
    {
        context.Initialize();
        context.LoadFlowsheet();
        context.SelectPackage();
        context.ManifestPathParameter.value = context.ManifestPath;
        context.ConnectRequiredPorts();

        var error = ContractAssert.Throws<CapeBadInvocationOrderException>(
            static unitOperation => unitOperation.Calculate(),
            context.UnitOperation,
            "Calculate() with only one companion file path should fail at the validation boundary.");

        ContractAssert.Equal(CapeValidationStatus.Invalid, context.UnitOperation.ValStatus, "Companion validation failure should set ValStatus to Invalid.");
        ContractAssert.Equal(UnitOperationCalculationReportState.Failure, context.UnitOperation.GetCalculationReportState(), "Companion validation failure should publish failure report state.");
        ContractAssert.Equal(error.ErrorName, context.UnitOperation.GetCalculationReportDetailValue(UnitOperationCalculationReportDetailCatalog.Error), "Companion validation failure report should preserve semantic error name.");
        ContractAssert.Equal(UnitOperationParameterCatalog.PropertyPackageManifestPath.ConfigurationOperationName, context.UnitOperation.GetCalculationReportDetailValue(UnitOperationCalculationReportDetailCatalog.RequestedOperation), "Companion validation failure should point back to the shared configuration operation frozen in the catalog.");
        ContractAssert.Null(context.UnitOperation.GetCalculationReportDetailValue(UnitOperationCalculationReportDetailCatalog.NativeStatus), "Companion validation failure should not invent native status.");
        ContractAssert.Null(context.UnitOperation.LastCalculationResult, "Companion validation failure should not preserve a stale success result.");
        ContractAssert.NotNull(context.UnitOperation.LastCalculationFailure, "Companion validation failure should preserve failure summary.");
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
        ParameterCollection = (ICapeCollection)UnitOperation.Parameters;
        PortCollection = (ICapeCollection)UnitOperation.Ports;
        FlowsheetParameter = UnitOperation.Parameters.GetByName(UnitOperationParameterCatalog.FlowsheetJson.Name);
        PackageIdParameter = UnitOperation.Parameters.GetByName(UnitOperationParameterCatalog.PropertyPackageId.Name);
        ManifestPathParameter = UnitOperation.Parameters.GetByName(UnitOperationParameterCatalog.PropertyPackageManifestPath.Name);
        PayloadPathParameter = UnitOperation.Parameters.GetByOneBasedIndex(4);
        FeedPort = UnitOperation.Ports.GetByName(UnitOperationPortCatalog.Feed.Name);
        ProductPort = UnitOperation.Ports.GetByName(UnitOperationPortCatalog.Product.Name);
    }

    public RadishFlowCapeOpenUnitOperation UnitOperation { get; }

    public string ManifestPath => _options.ManifestPath;

    public ICapeCollection ParameterCollection { get; }

    public ICapeCollection PortCollection { get; }

    public UnitOperationParameterPlaceholder FlowsheetParameter { get; }

    public UnitOperationParameterPlaceholder PackageIdParameter { get; }

    public UnitOperationParameterPlaceholder ManifestPathParameter { get; }

    public UnitOperationParameterPlaceholder PayloadPathParameter { get; }

    public UnitOperationPortPlaceholder FeedPort { get; }

    public UnitOperationPortPlaceholder ProductPort { get; }

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
        UnitOperation.Ports.GetByName(UnitOperationPortCatalog.Feed.Name).Connect(new ContractConnectedObject("Contract Feed"));
        UnitOperation.Ports.GetByName(UnitOperationPortCatalog.Product.Name).Connect(new ContractConnectedObject("Contract Product"));
    }

    public void DisconnectProductPort()
    {
        UnitOperation.Ports.GetByName(UnitOperationPortCatalog.Product.Name).Disconnect();
    }

    public bool IsProductPortConnected()
    {
        return UnitOperation.Ports.GetByName(UnitOperationPortCatalog.Product.Name).connectedObject is not null;
    }

    public UnitOperationHostConfigurationSnapshot ReadConfiguration()
    {
        return UnitOperationHostConfigurationReader.Read(UnitOperation);
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

    public static void SameReference(object? expected, object? actual, string message)
    {
        if (!ReferenceEquals(expected, actual))
        {
            throw new InvalidOperationException(message);
        }
    }

    public static void SequenceEqual<T>(
        IEnumerable<T> expected,
        IEnumerable<T> actual,
        string message)
    {
        if (!expected.SequenceEqual(actual))
        {
            throw new InvalidOperationException(message);
        }
    }

    public static void NotSameReference(object? unexpected, object? actual, string message)
    {
        if (ReferenceEquals(unexpected, actual))
        {
            throw new InvalidOperationException(message);
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

    public static TException Throws<TException>(Action action, string message)
        where TException : Exception
    {
        try
        {
            action();
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
