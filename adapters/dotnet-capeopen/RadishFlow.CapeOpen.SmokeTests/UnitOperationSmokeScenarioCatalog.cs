using RadishFlow.CapeOpen.UnitOp.Mvp.Results;

internal sealed record UnitOperationSmokeScenario(
    string Id,
    string Title,
    Action<UnitOperationSmokeSession, SmokeOptions> Execute);

internal static class UnitOperationSmokeScenarioCatalog
{
    public static string ScenarioOptionHelpText => "all|session|recovery|shutdown";

    public static IReadOnlyList<UnitOperationSmokeScenario> CreateScenarios(string scenarioId)
    {
        UnitOperationSmokeScenario[] scenarios =
        [
            CreateHostSessionScenario(),
            CreateHostRecoveryScenario(),
            CreateHostShutdownScenario(),
        ];

        if (string.IsNullOrWhiteSpace(scenarioId) ||
            string.Equals(scenarioId, "all", StringComparison.OrdinalIgnoreCase))
        {
            return scenarios;
        }

        var matchedScenario = scenarios.FirstOrDefault(
            scenario => string.Equals(scenario.Id, scenarioId, StringComparison.OrdinalIgnoreCase));
        if (matchedScenario is null)
        {
            throw new ArgumentException(
                $"Unsupported unitop scenario `{scenarioId}`. Supported values: {ScenarioOptionHelpText}.");
        }

        return new[] { matchedScenario };
    }

    private static UnitOperationSmokeScenario CreateHostSessionScenario()
    {
        return new UnitOperationSmokeScenario(
            "session",
            "Host Session Timeline",
            static (session, options) =>
            {
                session.ExpectInvocationOrderBeforeInitialize("round-0");
                session.InitializeAndExpectIdle("round-1");
                session.ConfigureMinimumInputsAndConnect("round-2");
                session.ExpectSuccessRound(
                    "round-3",
                    static report => $"status={report.Snapshot.GetDetailValue(UnitOperationCalculationReportDetailCatalog.Status)}, diagnostics={report.Snapshot.GetDetailValue(UnitOperationCalculationReportDetailCatalog.DiagnosticCount)}");
                session.ExpectNativeFailureForMissingPackage("round-4", "missing-package-for-session");
                session.RestorePackageAndExpectValid("round-5a", options.PackageId);
                session.ExpectSuccessRound(
                    "round-5b",
                    static report => $"reportState={report.Snapshot.State}, highestSeverity={report.Snapshot.GetDetailValue(UnitOperationCalculationReportDetailCatalog.HighestSeverity)}");
                session.BreakCompanionInputsAndExpectValidationFailure("round-6");
                session.RestoreMinimumInputsAndExpectValid("round-7a");
                session.ExpectSuccessRound(
                    "round-7b",
                    static report => $"relatedStreams={report.Snapshot.GetDetailValue(UnitOperationCalculationReportDetailCatalog.RelatedStreamIds)}");
                session.DisconnectProductPortAndExpectRecoveryWindow("round-8a");
                session.ReconnectProductPort("round-8b", "Session Product");
                session.ExpectSuccessRound(
                    "round-8c",
                    static report => $"headline={report.Snapshot.Headline}");
                session.TerminateAndExpectClosed("round-9");
            });
    }

    private static UnitOperationSmokeScenario CreateHostRecoveryScenario()
    {
        return new UnitOperationSmokeScenario(
            "recovery",
            "Host Recovery Timeline",
            static (session, options) =>
            {
                session.InitializeAndExpectIdle("recovery-0");
                session.ConfigureMinimumInputsAndConnect("recovery-1");
                session.BreakCompanionInputsAndExpectValidationFailure("recovery-2");
                session.RestoreMinimumInputsAndExpectValid("recovery-3");
                session.ExpectSuccessRound(
                    "recovery-4",
                    static report => $"headline={report.Snapshot.Headline}");
                session.DisconnectFeedPortAndExpectRecoveryWindow("recovery-5");
                session.ReconnectFeedPort("recovery-6", "Recovery Feed");
                session.ExpectSuccessRound(
                    "recovery-7",
                    static report => $"diagnosticCount={report.Snapshot.GetDetailValue(UnitOperationCalculationReportDetailCatalog.DiagnosticCount)}");
                session.ExpectNativeFailureForMissingPackage("recovery-8", "missing-package-for-recovery");
                session.RestorePackageAndExpectValid("recovery-9", options.PackageId);
                session.ExpectSuccessRound(
                    "recovery-10",
                    static report => $"relatedUnits={report.Snapshot.GetDetailValue(UnitOperationCalculationReportDetailCatalog.RelatedUnitIds)}");
                session.TerminateAndExpectClosed("recovery-11");
            });
    }

    private static UnitOperationSmokeScenario CreateHostShutdownScenario()
    {
        return new UnitOperationSmokeScenario(
            "shutdown",
            "Host Shutdown Timeline",
            static (session, _) =>
            {
                session.ExpectCurrentReportToBeEmpty("shutdown-0");
                session.InitializeAndExpectIdle("shutdown-1");
                session.ConfigureMinimumInputsAndConnect("shutdown-2");
                session.ExpectSuccessRound(
                    "shutdown-3",
                    static report => $"headline={report.Snapshot.Headline}");
                session.ExpectCurrentReportToBeSuccessful(
                    "shutdown-4",
                    static report => $"detailKeys={report.Snapshot.DetailKeyCount}, supplementalLines={report.Presentation.SupplementalLines.Count}");
                session.TerminateAndExpectClosed("shutdown-5");
                session.ExpectCurrentReportToBeEmpty("shutdown-6");
                session.ExpectPostTerminateCalculationFailure("shutdown-7");
            });
    }
}

internal static class UnitOperationSmokeScenarioRunner
{
    public static void RunAll(
        SmokeOptions options,
        string projectJson,
        IReadOnlyList<UnitOperationSmokeScenario> scenarios)
    {
        foreach (var scenario in scenarios)
        {
            Run(options, projectJson, scenario);
        }
    }

    private static void Run(
        SmokeOptions options,
        string projectJson,
        UnitOperationSmokeScenario scenario)
    {
        using var session = new UnitOperationSmokeSession(options, projectJson);
        scenario.Execute(session, options);

        Console.WriteLine($"== {scenario.Title} ==");
        foreach (var line in session.Timeline)
        {
            Console.WriteLine($"- {line}");
        }

        Console.WriteLine();
    }
}
