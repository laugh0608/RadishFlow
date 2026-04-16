using RadishFlow.CapeOpen.Interop.Errors;
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
        var report = _driver.ReadReport().Snapshot;
        EnsureCondition(
            report.State == UnitOperationCalculationReportState.None,
            $"{roundLabel} should enter idle report state immediately after Initialize().");
        _timeline.Add($"{roundLabel} initialized: reportState={report.State}");
    }

    public void ConfigureMinimumInputsAndConnect(string roundLabel)
    {
        _driver.ConfigureMinimumInputs(includePackageId: true);
        _driver.ConnectRequiredPorts();
        var validation = _driver.Validate();
        EnsureCondition(
            validation.IsValid,
            $"{roundLabel} should validate once minimum inputs and required ports are configured.");
        _timeline.Add($"{roundLabel} configured: validation=valid");
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
        EnsureCondition(
            string.Equals(error.NativeStatus, "MissingEntity", StringComparison.Ordinal) &&
            attempt.Report.Snapshot.State == UnitOperationCalculationReportState.Failure,
            $"{roundLabel} should preserve MissingEntity classification and failure report state.");
        _timeline.Add($"{roundLabel} native-failure: nativeStatus={error.NativeStatus}");
    }

    public void RestorePackageAndExpectValid(string roundLabel, string packageId)
    {
        _driver.PackageIdParameter.value = packageId;
        var validation = _driver.Validate();
        EnsureCondition(validation.IsValid, $"{roundLabel} should restore a valid package configuration.");
        _timeline.Add($"{roundLabel} package-restored: validation=valid");
    }

    public void BreakCompanionInputsAndExpectValidationFailure(string roundLabel)
    {
        _driver.ManifestPathParameter.value = null;
        var validation = _driver.Validate();
        EnsureCondition(
            !validation.IsValid &&
            validation.Message.Contains("must be configured together", StringComparison.Ordinal),
            $"{roundLabel} should fail validation when companion inputs diverge.");
        var attempt = _driver.Calculate();
        var error = attempt.ExpectFailure<CapeBadInvocationOrderException>(
            UnitOperationHostDriverFailureKind.Validation,
            $"{roundLabel} broken companion inputs");
        EnsureCondition(
            string.Equals(error.RequestedOperation, nameof(RadishFlowCapeOpenUnitOperation.LoadPropertyPackageFiles), StringComparison.Ordinal),
            $"{roundLabel} should request LoadPropertyPackageFiles().");
        _timeline.Add($"{roundLabel} validation-failure: requested={error.RequestedOperation}");
    }

    public void RestoreMinimumInputsAndExpectValid(string roundLabel)
    {
        _driver.ConfigureMinimumInputs(includePackageId: true);
        var validation = _driver.Validate();
        EnsureCondition(validation.IsValid, $"{roundLabel} should restore a valid minimum input set.");
        _timeline.Add($"{roundLabel} inputs-restored: validation=valid");
    }

    public void DisconnectProductPortAndExpectRecoveryWindow(string roundLabel)
    {
        _driver.ProductPort.Disconnect();
        var validation = _driver.Validate();
        EnsureCondition(
            !validation.IsValid &&
            validation.Message.Contains("Required port `Product` is not connected.", StringComparison.Ordinal),
            $"{roundLabel} should fail validation when product port is disconnected.");
        EnsureCondition(
            _driver.ReadReport().Snapshot.State == UnitOperationCalculationReportState.None,
            $"{roundLabel} should clear the cached host report while disconnected.");
        _timeline.Add($"{roundLabel} disconnected: validation=invalid");
    }

    public void ReconnectProductPort(string roundLabel, string componentName)
    {
        _driver.ProductPort.Connect(new SmokeConnectedObject(componentName));
        var validation = _driver.Validate();
        EnsureCondition(validation.IsValid, $"{roundLabel} should restore a valid state after reconnecting product port.");
        _timeline.Add($"{roundLabel} reconnected: validation=valid");
    }

    public void TerminateAndExpectClosed(string roundLabel)
    {
        _driver.Terminate();
        var report = _driver.ReadReport().Snapshot;
        EnsureCondition(
            report.State == UnitOperationCalculationReportState.None,
            $"{roundLabel} should end in the empty report state after Terminate().");
        var validation = _driver.Validate();
        EnsureCondition(
            !validation.IsValid &&
            validation.Message.Contains("Terminate has already been called", StringComparison.Ordinal),
            $"{roundLabel} should keep Validate() invalid after Terminate().");
        _timeline.Add($"{roundLabel} terminated: reportState={report.State}, validation=invalid");
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
        EnsureCondition(validation.IsValid, $"{scenario} should validate before Calculate().");

        var attempt = driver.Calculate();
        if (!attempt.Succeeded)
        {
            throw new InvalidOperationException(
                $"{scenario} expected success, but received {attempt.Failure?.GetType().Name ?? "<unknown>"}.");
        }

        EnsureCondition(
            attempt.Report.Snapshot.State == UnitOperationCalculationReportState.Success,
            $"{scenario} should expose success report state.");
        EnsureCondition(
            string.Equals(
                attempt.Report.Snapshot.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Status),
                "converged",
                StringComparison.Ordinal),
            $"{scenario} should expose converged status detail.");

        return attempt.Report;
    }

    private static void EnsureCondition(bool condition, string message)
    {
        if (!condition)
        {
            throw new InvalidOperationException(message);
        }
    }
}
