using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

internal sealed class UnitOperationSmokeSession : IDisposable
{
    private readonly UnitOperationSmokeHostDriver _driver;
    private readonly List<string> _timeline = [];

    public UnitOperationSmokeSession(SmokeOptions options, string projectJson)
    {
        _driver = new UnitOperationSmokeHostDriver(options, projectJson);
    }

    public IReadOnlyList<string> Timeline => _timeline;

    public UnitOperationSmokeHostDriver Driver => _driver;

    public void ExpectInvocationOrderBeforeInitialize(string roundLabel)
    {
        var attempt = _driver.Calculate();
        var error = attempt.ExpectFailure<CapeBadInvocationOrderException>(
            UnitOperationHostDriverFailureKind.InvocationOrder,
            $"{roundLabel} calculate before initialize");
        _timeline.Add(
            $"{roundLabel} invocation-order: operation={error.Operation}, requested={error.RequestedOperation}");
    }

    public void InitializeAndExpectIdle(string roundLabel)
    {
        _driver.Initialize();
        var configuration = _driver.ReadConfiguration();
        UnitOperationSmokeConfigurationAssertions.AssertState(
            configuration,
            UnitOperationHostConfigurationState.Incomplete,
            expectedReady: false,
            $"{roundLabel} configuration");
        UnitOperationSmokeConfigurationAssertions.AssertActionPlan(
            _driver.ReadActionPlan(),
            $"{roundLabel} configuration",
            UnitOperationSmokeConfigurationAssertions.Action(
                UnitOperationHostActionGroupKind.Parameters,
                UnitOperationHostActionTargetKind.Parameter,
                UnitOperationParameterCatalog.FlowsheetJson.ConfigurationOperationName,
                UnitOperationHostConfigurationIssueKind.RequiredParameterMissing,
                "Required parameter",
                UnitOperationParameterCatalog.FlowsheetJson.Name),
            UnitOperationSmokeConfigurationAssertions.Action(
                UnitOperationHostActionGroupKind.Parameters,
                UnitOperationHostActionTargetKind.Parameter,
                UnitOperationParameterCatalog.PropertyPackageId.ConfigurationOperationName,
                UnitOperationHostConfigurationIssueKind.RequiredParameterMissing,
                "Required parameter",
                UnitOperationParameterCatalog.PropertyPackageId.Name),
            UnitOperationSmokeConfigurationAssertions.Action(
                UnitOperationHostActionGroupKind.Ports,
                UnitOperationHostActionTargetKind.Port,
                UnitOperationPortCatalog.Feed.ConnectionOperationName,
                UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected,
                "Required port",
                UnitOperationPortCatalog.Feed.Name),
            UnitOperationSmokeConfigurationAssertions.Action(
                UnitOperationHostActionGroupKind.Ports,
                UnitOperationHostActionTargetKind.Port,
                UnitOperationPortCatalog.Product.ConnectionOperationName,
                UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected,
                "Required port",
                UnitOperationPortCatalog.Product.Name));
        var report = _driver.ReadReport().Snapshot;
        UnitOperationSmokeReportAssertions.EnsureCondition(
            report.State == UnitOperationCalculationReportState.None,
            $"{roundLabel} should enter idle report state immediately after Initialize().");
        _timeline.Add($"{roundLabel} initialized: reportState={report.State}");
    }

    public void ConfigureMinimumInputsAndConnect(string roundLabel)
    {
        _driver.ConfigureMinimumInputs(includePackageId: true);
        _driver.ConnectRequiredPorts();
        var configuration = _driver.ReadConfiguration();
        UnitOperationSmokeConfigurationAssertions.AssertState(
            configuration,
            UnitOperationHostConfigurationState.Ready,
            expectedReady: true,
            $"{roundLabel} configuration");
        UnitOperationSmokeConfigurationAssertions.AssertActionPlan(
            _driver.ReadActionPlan(),
            $"{roundLabel} configuration");
        var validation = _driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            validation.IsValid,
            $"{roundLabel} should validate once minimum inputs and required ports are configured.");
        _timeline.Add($"{roundLabel} configured: validation=valid");
    }

    public UnitOperationHostReportBundle ExpectCurrentReportToBeEmpty(string roundLabel)
    {
        var report = _driver.ReadReport();
        UnitOperationSmokeReportAssertions.AssertEmpty(report, roundLabel);
        _timeline.Add($"{roundLabel} report-read: state={report.Snapshot.State}");
        return report;
    }

    public UnitOperationHostReportBundle ExpectCurrentReportToBeSuccessful(
        string roundLabel,
        Func<UnitOperationHostReportBundle, string> timelineDetail)
    {
        var report = _driver.ReadReport();
        UnitOperationSmokeReportAssertions.AssertSuccess(report, roundLabel);
        _timeline.Add($"{roundLabel} report-read: {timelineDetail(report)}");
        return report;
    }

    public UnitOperationHostReportBundle ExpectSuccessRound(
        string roundLabel,
        Func<UnitOperationHostReportBundle, string> timelineDetail)
    {
        var report = EnsureSuccessfulHostRound(_driver, roundLabel);
        _timeline.Add($"{roundLabel} success: {timelineDetail(report)}");
        return report;
    }

    public void ExpectNativeFailureForMissingPackage(
        string roundLabel,
        string missingPackageId)
    {
        _driver.PackageIdParameter.value = missingPackageId;
        var attempt = _driver.Calculate();
        var error = attempt.ExpectFailure<CapeInvalidArgumentException>(
            UnitOperationHostDriverFailureKind.Native,
            $"{roundLabel} missing property package");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            string.Equals(error.NativeStatus, "MissingEntity", StringComparison.Ordinal) &&
            attempt.Report.Snapshot.State == UnitOperationCalculationReportState.Failure,
            $"{roundLabel} should preserve MissingEntity classification and failure report state.");
        _timeline.Add($"{roundLabel} native-failure: nativeStatus={error.NativeStatus}");
    }

    public void RestorePackageAndExpectValid(string roundLabel, string packageId)
    {
        _driver.PackageIdParameter.value = packageId;
        var configuration = _driver.ReadConfiguration();
        UnitOperationSmokeConfigurationAssertions.AssertState(
            configuration,
            UnitOperationHostConfigurationState.Ready,
            expectedReady: true,
            $"{roundLabel} configuration");
        var validation = _driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(validation.IsValid, $"{roundLabel} should restore a valid package configuration.");
        _timeline.Add($"{roundLabel} package-restored: validation=valid");
    }

    public void BreakCompanionInputsAndExpectValidationFailure(string roundLabel)
    {
        _driver.ManifestPathParameter.value = null;
        var configuration = _driver.ReadConfiguration();
        UnitOperationSmokeConfigurationAssertions.AssertState(
            configuration,
            UnitOperationHostConfigurationState.Incomplete,
            expectedReady: false,
            $"{roundLabel} configuration");
        UnitOperationSmokeConfigurationAssertions.AssertBlockingIssueKinds(
            configuration,
            $"{roundLabel} configuration",
            UnitOperationHostConfigurationIssueKind.CompanionParameterMismatch);
        UnitOperationSmokeConfigurationAssertions.AssertActionPlan(
            _driver.ReadActionPlan(),
            $"{roundLabel} configuration",
            UnitOperationSmokeConfigurationAssertions.Action(
                UnitOperationHostActionGroupKind.Parameters,
                UnitOperationHostActionTargetKind.Parameter,
                UnitOperationParameterCatalog.PropertyPackageManifestPath.ConfigurationOperationName,
                UnitOperationHostConfigurationIssueKind.CompanionParameterMismatch,
                "must be configured together",
                UnitOperationParameterCatalog.PropertyPackageManifestPath.Name,
                UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name));
        var validation = _driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            !validation.IsValid &&
            validation.Message.Contains("must be configured together", StringComparison.Ordinal),
            $"{roundLabel} should fail validation when companion inputs diverge.");
        var attempt = _driver.Calculate();
        var error = attempt.ExpectFailure<CapeBadInvocationOrderException>(
            UnitOperationHostDriverFailureKind.Validation,
            $"{roundLabel} broken companion inputs");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            string.Equals(
                error.RequestedOperation,
                UnitOperationParameterCatalog.PropertyPackageManifestPath.ConfigurationOperationName,
                StringComparison.Ordinal),
            $"{roundLabel} should request LoadPropertyPackageFiles().");
        _timeline.Add($"{roundLabel} validation-failure: requested={error.RequestedOperation}");
    }

    public void RestoreMinimumInputsAndExpectValid(string roundLabel)
    {
        _driver.ConfigureMinimumInputs(includePackageId: true);
        var configuration = _driver.ReadConfiguration();
        UnitOperationSmokeConfigurationAssertions.AssertState(
            configuration,
            UnitOperationHostConfigurationState.Ready,
            expectedReady: true,
            $"{roundLabel} configuration");
        var validation = _driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(validation.IsValid, $"{roundLabel} should restore a valid minimum input set.");
        _timeline.Add($"{roundLabel} inputs-restored: validation=valid");
    }

    public void DisconnectProductPortAndExpectRecoveryWindow(string roundLabel)
    {
        DisconnectRequiredPortAndExpectRecoveryWindow(roundLabel, _driver.ProductPort, "Product");
    }

    public void ReconnectProductPort(string roundLabel, string componentName)
    {
        ReconnectRequiredPort(roundLabel, _driver.ProductPort, componentName, "Product");
    }

    public void DisconnectFeedPortAndExpectRecoveryWindow(string roundLabel)
    {
        DisconnectRequiredPortAndExpectRecoveryWindow(roundLabel, _driver.FeedPort, "Feed");
    }

    public void ReconnectFeedPort(string roundLabel, string componentName)
    {
        ReconnectRequiredPort(roundLabel, _driver.FeedPort, componentName, "Feed");
    }

    public void TerminateAndExpectClosed(string roundLabel)
    {
        _driver.Terminate();
        var configuration = _driver.ReadConfiguration();
        UnitOperationSmokeConfigurationAssertions.AssertState(
            configuration,
            UnitOperationHostConfigurationState.Terminated,
            expectedReady: false,
            $"{roundLabel} configuration");
        UnitOperationSmokeConfigurationAssertions.AssertBlockingIssueKinds(
            configuration,
            $"{roundLabel} configuration",
            UnitOperationHostConfigurationIssueKind.Terminated);
        var report = _driver.ReadReport().Snapshot;
        UnitOperationSmokeReportAssertions.EnsureCondition(
            report.State == UnitOperationCalculationReportState.None,
            $"{roundLabel} should end in the empty report state after Terminate().");
        var validation = _driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            !validation.IsValid &&
            validation.Message.Contains("Terminate has already been called", StringComparison.Ordinal),
            $"{roundLabel} should keep Validate() invalid after Terminate().");
        _timeline.Add($"{roundLabel} terminated: reportState={report.State}, validation=invalid");
    }

    public void ExpectPostTerminateCalculationFailure(string roundLabel)
    {
        var attempt = _driver.Calculate();
        var error = attempt.ExpectFailure<CapeBadInvocationOrderException>(
            UnitOperationHostDriverFailureKind.Validation,
            $"{roundLabel} calculate after terminate");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            string.Equals(error.Operation, nameof(RadishFlowCapeOpenUnitOperation.Calculate), StringComparison.Ordinal),
            $"{roundLabel} should fail at the Calculate() boundary after Terminate().");
        UnitOperationSmokeReportAssertions.AssertEmpty(attempt.Report, roundLabel);
        _timeline.Add(
            $"{roundLabel} calculate-blocked: kind={UnitOperationHostDriverFailureKind.Validation}, operation={error.Operation}");
    }

    public void Dispose()
    {
        _driver.Dispose();
    }

    private static UnitOperationHostReportBundle EnsureSuccessfulHostRound(
        UnitOperationSmokeHostDriver driver,
        string scenario)
    {
        var validation = driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(validation.IsValid, $"{scenario} should validate before Calculate().");

        var attempt = driver.Calculate();
        if (!attempt.Succeeded)
        {
            throw new InvalidOperationException(
                $"{scenario} expected success, but received {attempt.Failure?.GetType().Name ?? "<unknown>"}.");
        }

        UnitOperationSmokeReportAssertions.AssertSuccess(attempt.Report, scenario);

        return attempt.Report;
    }

    private void DisconnectRequiredPortAndExpectRecoveryWindow(
        string roundLabel,
        UnitOperationPortPlaceholder port,
        string portName)
    {
        port.Disconnect();
        var configuration = _driver.ReadConfiguration();
        UnitOperationSmokeConfigurationAssertions.AssertState(
            configuration,
            UnitOperationHostConfigurationState.Incomplete,
            expectedReady: false,
            $"{roundLabel} configuration");
        UnitOperationSmokeConfigurationAssertions.AssertBlockingIssueKinds(
            configuration,
            $"{roundLabel} configuration",
            UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected);
        UnitOperationSmokeConfigurationAssertions.AssertActionPlan(
            _driver.ReadActionPlan(),
            $"{roundLabel} configuration",
            UnitOperationSmokeConfigurationAssertions.Action(
                UnitOperationHostActionGroupKind.Ports,
                UnitOperationHostActionTargetKind.Port,
                UnitOperationPortCatalog.GetByName(portName).ConnectionOperationName,
                UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected,
                "Required port",
                portName));
        var validation = _driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            !validation.IsValid &&
            validation.Message.Contains($"Required port `{portName}` is not connected.", StringComparison.Ordinal),
            $"{roundLabel} should fail validation when {portName} port is disconnected.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            _driver.ReadReport().Snapshot.State == UnitOperationCalculationReportState.None,
            $"{roundLabel} should clear the cached host report while disconnected.");
        _timeline.Add($"{roundLabel} disconnected-{portName.ToLowerInvariant()}: validation=invalid");
    }

    private void ReconnectRequiredPort(
        string roundLabel,
        UnitOperationPortPlaceholder port,
        string componentName,
        string portName)
    {
        port.Connect(new SmokeConnectedObject(componentName));
        var configuration = _driver.ReadConfiguration();
        UnitOperationSmokeConfigurationAssertions.AssertState(
            configuration,
            UnitOperationHostConfigurationState.Ready,
            expectedReady: true,
            $"{roundLabel} configuration");
        var validation = _driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            validation.IsValid,
            $"{roundLabel} should restore a valid state after reconnecting {portName} port.");
        _timeline.Add($"{roundLabel} reconnected-{portName.ToLowerInvariant()}: validation=valid");
    }
}
