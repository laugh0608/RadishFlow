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
        var session = _driver.ReadSession();
        UnitOperationSmokeHostSessionAssertions.AssertSummary(
            session,
            expectedState: UnitOperationHostSessionState.Incomplete,
            expectedReady: false,
            expectedBlockingActions: true,
            expectedCurrentMaterialResults: false,
            expectedCurrentExecution: false,
            expectedCurrentResults: false,
            expectedRefresh: false,
            expectedFailureReport: false,
            scenario: $"{roundLabel} host session",
            UnitOperationParameterCatalog.FlowsheetJson.ConfigurationOperationName,
            UnitOperationParameterCatalog.PropertyPackageId.ConfigurationOperationName,
            UnitOperationPortCatalog.Feed.ConnectionOperationName);
        _timeline.Add($"{roundLabel} initialized: sessionState={session.State}, reportState={report.State}");
    }

    public void ConfigureMinimumInputsAndConnect(string roundLabel)
    {
        var round = _driver.ExecuteRound(
            new UnitOperationHostRoundRequest(
                actionInputSet: _driver.CreateMinimumConfigurationInputSet(includePackageId: true),
                executeReadyActions: true,
                runValidation: true,
                supplementalMutationCommands: _driver.CreateOptionalPackageFileMutationCommands()));
        var actionResult = round.ActionExecution
            ?? throw new InvalidOperationException($"{roundLabel} should execute host actions during minimum-input round.");
        var validation = round.Validation
            ?? throw new InvalidOperationException($"{roundLabel} should validate during minimum-input round.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actionResult.Execution.AppliedMutationCount == 4 &&
            actionResult.Execution.InvalidatedValidation &&
            actionResult.Execution.InvalidatedCalculationReport,
            $"{roundLabel} should configure minimum inputs through action execution.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actionResult.FollowUp.Kind == UnitOperationHostFollowUpKind.Validate &&
            actionResult.FollowUp.CanValidate &&
            !actionResult.FollowUp.CanCalculate,
            $"{roundLabel} should recommend validation as the next formal host step after applying mutations.");
        UnitOperationSmokeConfigurationAssertions.AssertState(
            actionResult.Configuration,
            UnitOperationHostConfigurationState.Ready,
            expectedReady: true,
            $"{roundLabel} configuration");
        UnitOperationSmokeConfigurationAssertions.AssertActionPlan(
            actionResult.ActionPlan,
            $"{roundLabel} configuration");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            validation.IsValid,
            $"{roundLabel} should validate once minimum inputs and required ports are configured.");
        UnitOperationSmokeHostSessionAssertions.AssertSummary(
            round.Session,
            expectedState: UnitOperationHostSessionState.Ready,
            expectedReady: true,
            expectedBlockingActions: false,
            expectedCurrentMaterialResults: false,
            expectedCurrentExecution: false,
            expectedCurrentResults: false,
            expectedRefresh: false,
            expectedFailureReport: false,
            scenario: $"{roundLabel} host session");
        _timeline.Add($"{roundLabel} configured: sessionState={round.Session.State}, actions={actionResult.Execution.AppliedActionCount}, mutations={actionResult.Execution.AppliedMutationCount}, validation=valid");
    }

    public UnitOperationHostReportBundle ExpectCurrentReportToBeEmpty(string roundLabel)
    {
        var report = _driver.ReadReport();
        UnitOperationSmokeReportAssertions.AssertEmpty(report, roundLabel);
        var session = _driver.ReadSession();
        _timeline.Add($"{roundLabel} report-read: sessionState={session.State}, reportState={report.Snapshot.State}");
        return report;
    }

    public UnitOperationHostReportBundle ExpectCurrentReportToBeSuccessful(
        string roundLabel,
        Func<UnitOperationHostReportBundle, string> timelineDetail)
    {
        var report = _driver.ReadReport();
        UnitOperationSmokeReportAssertions.AssertSuccess(report, roundLabel);
        var session = _driver.ReadSession();
        UnitOperationSmokeHostSessionAssertions.AssertSummary(
            session,
            expectedState: UnitOperationHostSessionState.Available,
            expectedReady: true,
            expectedBlockingActions: false,
            expectedCurrentMaterialResults: true,
            expectedCurrentExecution: true,
            expectedCurrentResults: true,
            expectedRefresh: false,
            expectedFailureReport: false,
            scenario: $"{roundLabel} host session");
        _timeline.Add($"{roundLabel} report-read: sessionState={session.State}, {timelineDetail(report)}");
        return report;
    }

    public UnitOperationHostReportBundle ExpectSuccessRound(
        string roundLabel,
        Func<UnitOperationHostReportBundle, string> timelineDetail)
    {
        var round = EnsureSuccessfulHostRound(_driver, roundLabel);
        var report = round.ReportBundle;
        UnitOperationSmokeHostSessionAssertions.AssertSummary(
            round.Session,
            expectedState: UnitOperationHostSessionState.Available,
            expectedReady: true,
            expectedBlockingActions: false,
            expectedCurrentMaterialResults: true,
            expectedCurrentExecution: true,
            expectedCurrentResults: true,
            expectedRefresh: false,
            expectedFailureReport: false,
            scenario: $"{roundLabel} host session");
        _timeline.Add($"{roundLabel} success: sessionState={round.Session.State}, {timelineDetail(report)}");
        return report;
    }

    public void ExpectNativeFailureForMissingPackage(
        string roundLabel,
        string missingPackageId)
    {
        _driver.PackageIdParameter.value = missingPackageId;
        var round = _driver.ExecuteRound(
            new UnitOperationHostRoundRequest(
                executeReadyActions: false,
                runValidation: false,
                runCalculation: true,
                requireSuccessfulValidationForCalculation: false));
        var calculation = round.Calculation
            ?? throw new InvalidOperationException($"{roundLabel} should execute calculate during native-failure round.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            !calculation.Succeeded && calculation.Failure is CapeInvalidArgumentException,
            $"{roundLabel} should preserve CapeInvalidArgumentException during native-failure round.");
        var error = calculation.Failure as CapeInvalidArgumentException
            ?? throw new InvalidOperationException($"{roundLabel} should preserve CapeInvalidArgumentException instance.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            string.Equals(error.NativeStatus, "MissingEntity", StringComparison.Ordinal) &&
            round.Report.State == UnitOperationCalculationReportState.Failure,
            $"{roundLabel} should preserve MissingEntity classification and failure report state.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            round.StopKind == UnitOperationHostRoundStopKind.CalculationFailed &&
            round.Session.State == UnitOperationHostSessionState.Failure &&
            round.Session.Summary.HasFailureReport &&
            !round.Session.Summary.HasCurrentResults,
            $"{roundLabel} host session should expose failure state without current results after native failure.");
        _timeline.Add($"{roundLabel} native-failure: sessionState={round.Session.State}, nativeStatus={error.NativeStatus}");
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
        var round = _driver.ExecuteRound(
            new UnitOperationHostRoundRequest(
                executeReadyActions: false,
                runValidation: true));
        var validation = round.Validation
            ?? throw new InvalidOperationException($"{roundLabel} should validate during package-recovery round.");
        UnitOperationSmokeReportAssertions.EnsureCondition(validation.IsValid, $"{roundLabel} should restore a valid package configuration.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            round.StopKind == UnitOperationHostRoundStopKind.Completed &&
            (round.Session.State == UnitOperationHostSessionState.Ready || round.Session.State == UnitOperationHostSessionState.Stale) &&
            round.Session.Summary.IsReadyForCalculate &&
            !round.Session.Summary.HasFailureReport,
            $"{roundLabel} host session should restore a ready-or-stale non-failure configuration after package recovery.");
        _timeline.Add($"{roundLabel} package-restored: sessionState={round.Session.State}, validation=valid");
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
        UnitOperationSmokeReportAssertions.EnsureCondition(
            attempt.Session.State == UnitOperationHostSessionState.Failure &&
            attempt.Session.Summary.HasFailureReport &&
            attempt.Session.Summary.HasBlockingActions &&
            attempt.Session.ContainsRecommendedOperation(UnitOperationParameterCatalog.PropertyPackageManifestPath.ConfigurationOperationName),
            $"{roundLabel} host session should expose failure state and keep the companion recovery action.");
        _timeline.Add($"{roundLabel} validation-failure: sessionState={attempt.Session.State}, requested={error.RequestedOperation}");
    }

    public void RestoreMinimumInputsAndExpectValid(string roundLabel)
    {
        var round = _driver.ExecuteRound(
            new UnitOperationHostRoundRequest(
                actionInputSet: _driver.CreateMinimumConfigurationInputSet(includePackageId: true),
                executeReadyActions: true,
                runValidation: true,
                supplementalMutationCommands: _driver.CreateOptionalPackageFileMutationCommands()));
        var actionResult = round.ActionExecution
            ?? throw new InvalidOperationException($"{roundLabel} should execute host actions during minimum-input recovery.");
        var validation = round.Validation
            ?? throw new InvalidOperationException($"{roundLabel} should validate during minimum-input recovery.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actionResult.Execution.AppliedMutationCount >= 1,
            $"{roundLabel} should restore minimum inputs through action execution.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actionResult.FollowUp.Kind == UnitOperationHostFollowUpKind.Validate &&
            actionResult.FollowUp.CanValidate,
            $"{roundLabel} should recommend validation after restoring minimum inputs.");
        UnitOperationSmokeConfigurationAssertions.AssertState(
            actionResult.Configuration,
            UnitOperationHostConfigurationState.Ready,
            expectedReady: true,
            $"{roundLabel} configuration");
        UnitOperationSmokeReportAssertions.EnsureCondition(validation.IsValid, $"{roundLabel} should restore a valid minimum input set.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            round.StopKind == UnitOperationHostRoundStopKind.Completed &&
            (round.Session.State == UnitOperationHostSessionState.Ready || round.Session.State == UnitOperationHostSessionState.Stale) &&
            round.Session.Summary.IsReadyForCalculate &&
            !round.Session.Summary.HasFailureReport,
            $"{roundLabel} host session should restore a ready-or-stale non-failure configuration after input recovery.");
        _timeline.Add($"{roundLabel} inputs-restored: sessionState={round.Session.State}, validation=valid");
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
        UnitOperationSmokeHostSessionAssertions.AssertSummary(
            validation.Session,
            expectedState: UnitOperationHostSessionState.Terminated,
            expectedReady: false,
            expectedBlockingActions: true,
            expectedCurrentMaterialResults: false,
            expectedCurrentExecution: false,
            expectedCurrentResults: false,
            expectedRefresh: false,
            expectedFailureReport: false,
            scenario: $"{roundLabel} host session");
        _timeline.Add($"{roundLabel} terminated: sessionState={validation.Session.State}, reportState={report.State}, validation=invalid");
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
            $"{roundLabel} calculate-blocked: sessionState={attempt.Session.State}, kind={UnitOperationHostDriverFailureKind.Validation}, operation={error.Operation}");
    }

    public void Dispose()
    {
        _driver.Dispose();
    }

    private static UnitOperationSmokeRoundResult EnsureSuccessfulHostRound(
        UnitOperationSmokeHostDriver driver,
        string scenario)
    {
        var round = driver.ExecuteRound(
            new UnitOperationHostRoundRequest(
                executeReadyActions: false,
                runValidation: true,
                runCalculation: true));
        var validation = round.Validation
            ?? throw new InvalidOperationException($"{scenario} should validate during success round.");
        UnitOperationSmokeReportAssertions.EnsureCondition(validation.IsValid, $"{scenario} should validate before Calculate().");
        var calculation = round.Calculation
            ?? throw new InvalidOperationException($"{scenario} should calculate during success round.");
        if (!calculation.Succeeded)
        {
            throw new InvalidOperationException(
                $"{scenario} expected success, but received {calculation.Failure?.GetType().Name ?? "<unknown>"}.");
        }

        UnitOperationSmokeReportAssertions.AssertSuccess(round.ReportBundle, scenario);
        return round;
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
        UnitOperationSmokeHostSessionAssertions.AssertSummary(
            validation.Session,
            expectedState: UnitOperationHostSessionState.Stale,
            expectedReady: false,
            expectedBlockingActions: true,
            expectedCurrentMaterialResults: false,
            expectedCurrentExecution: false,
            expectedCurrentResults: false,
            expectedRefresh: true,
            expectedFailureReport: false,
            scenario: $"{roundLabel} host session",
            UnitOperationPortCatalog.GetByName(portName).ConnectionOperationName);
        _timeline.Add($"{roundLabel} disconnected-{portName.ToLowerInvariant()}: sessionState={validation.Session.State}, validation=invalid");
    }

    private void ReconnectRequiredPort(
        string roundLabel,
        UnitOperationPortPlaceholder port,
        string componentName,
        string portName)
    {
        var actionOutcome = _driver.ApplyRequiredPortAction(portName, componentName);
        UnitOperationSmokeReportAssertions.EnsureCondition(
            actionOutcome.Execution.Outcomes.Count == 1 &&
            actionOutcome.Execution.Outcomes[0].Disposition == UnitOperationHostActionExecutionDisposition.MutationApplied &&
            actionOutcome.Execution.AppliedMutationCount == 1,
            $"{roundLabel} should reconnect {portName} through action execution.");
        UnitOperationSmokeReportAssertions.EnsureCondition(
            port.IsConnected,
            $"{roundLabel} should leave {portName} connected after action execution.");
        UnitOperationSmokeConfigurationAssertions.AssertState(
            actionOutcome.Configuration,
            UnitOperationHostConfigurationState.Ready,
            expectedReady: true,
            $"{roundLabel} configuration");
        var validation = _driver.Validate();
        UnitOperationSmokeReportAssertions.EnsureCondition(
            validation.IsValid,
            $"{roundLabel} should restore a valid state after reconnecting {portName} port.");
        UnitOperationSmokeHostSessionAssertions.AssertSummary(
            validation.Session,
            expectedState: UnitOperationHostSessionState.Stale,
            expectedReady: true,
            expectedBlockingActions: false,
            expectedCurrentMaterialResults: false,
            expectedCurrentExecution: false,
            expectedCurrentResults: false,
            expectedRefresh: true,
            expectedFailureReport: false,
            scenario: $"{roundLabel} host session");
        _timeline.Add($"{roundLabel} reconnected-{portName.ToLowerInvariant()}: sessionState={validation.Session.State}, validation=valid");
    }

}
