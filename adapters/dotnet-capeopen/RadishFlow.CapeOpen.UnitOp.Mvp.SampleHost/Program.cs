using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;

Environment.ExitCode = SampleHostExecutable.Run(args);

internal static class SampleHostExecutable
{
    public static int Run(string[] args)
    {
        try
        {
            return Execute(args);
        }
        catch (CapeOpenException error)
        {
            Console.Error.WriteLine($"CAPE-OPEN operation failed: {error.Operation}");
            if (!string.IsNullOrWhiteSpace(error.NativeStatus))
            {
                Console.Error.WriteLine($"Native Status: {error.NativeStatus}");
            }

            Console.Error.WriteLine($"Message: {error.Message}");
            Console.Error.WriteLine(error);
            return 1;
        }
        catch (Exception error)
        {
            Console.Error.WriteLine("Sample host failed with an unhandled exception.");
            Console.Error.WriteLine(error);
            return 2;
        }
    }

    private static int Execute(string[] args)
    {
        var options = SampleHostOptions.Parse(args);
        if (options.ShowHelp)
        {
            Console.WriteLine(SampleHostOptions.HelpText);
            return 0;
        }

        var host = new PmeLikeUnitOperationHost(options.NativeLibraryDirectory);
        using var session = host.CreateSession();

        PrintViews("Constructed", session.ConstructedViews);
        PrintViews("Initialized", session.InitializedViews);

        var input = CreateInput(options);
        var result = session.ExecuteRound(input);
        PrintRequestPlan(result.RequestPlan);
        PrintSupplementalCommands(result.SupplementalMutationCommands);
        PrintRoundOutcome(result.Outcome);

        var terminatedSession = session.Terminate();
        Console.WriteLine();
        Console.WriteLine("== Terminated ==");
        Console.WriteLine(terminatedSession.Headline);

        return result.Outcome.Completed ? 0 : 3;
    }

    private static PmeLikeUnitOperationInput CreateInput(SampleHostOptions options)
    {
        return new PmeLikeUnitOperationInput(
            flowsheetJson: File.ReadAllText(options.ProjectPath),
            packageId: options.PackageId,
            manifestPath: options.ManifestPath,
            payloadPath: options.PayloadPath,
            feedMaterialObject: new PmeLikeMaterialObject("PME Feed Material"),
            productMaterialObject: new PmeLikeMaterialObject("PME Product Material"));
    }

    private static void PrintViews(string label, UnitOperationHostViewSnapshot views)
    {
        Console.WriteLine();
        Console.WriteLine($"== {label} ==");
        Console.WriteLine($"Session State: {views.Session.State}");
        Console.WriteLine($"Headline: {views.Headline}");
        Console.WriteLine($"Configuration State: {views.Configuration.State}");

        if (views.ActionPlan.ActionCount == 0)
        {
            Console.WriteLine("Action Plan: <none>");
            return;
        }

        Console.WriteLine("Action Plan:");
        foreach (var group in views.ActionPlan.Groups)
        {
            Console.WriteLine($"- {group.Title}");
            foreach (var action in group.Actions)
            {
                Console.WriteLine(
                    $"  [{action.RecommendedOrder}] {action.IssueKind} -> {string.Join(", ", action.Target.Names)} | blocking={action.IsBlocking.ToString().ToLowerInvariant()}");
            }
        }
    }

    private static void PrintRequestPlan(UnitOperationHostActionExecutionRequestPlan requestPlan)
    {
        Console.WriteLine();
        Console.WriteLine("== Request Plan ==");
        Console.WriteLine($"Entries: {requestPlan.EntryCount}");
        Console.WriteLine($"Ready Requests: {requestPlan.RequestCount}");
        Console.WriteLine($"Missing Inputs: {requestPlan.MissingInputCount}");
        foreach (var entry in requestPlan.Entries)
        {
            var missingInputs = entry.MissingInputNames.Count == 0
                ? "-"
                : string.Join(", ", entry.MissingInputNames);
            Console.WriteLine(
                $"- {entry.Action.IssueKind}: disposition={entry.Disposition}, target={string.Join(", ", entry.Action.Target.Names)}, missing={missingInputs}");
        }
    }

    private static void PrintSupplementalCommands(IReadOnlyList<UnitOperationHostObjectMutationCommand> commands)
    {
        Console.WriteLine();
        Console.WriteLine("== Supplemental Mutations ==");
        if (commands.Count == 0)
        {
            Console.WriteLine("<none>");
            return;
        }

        foreach (var command in commands)
        {
            Console.WriteLine($"- {command.Kind}: {command.TargetName}");
        }
    }

    private static void PrintRoundOutcome(UnitOperationHostRoundOutcome outcome)
    {
        Console.WriteLine();
        Console.WriteLine("== Round Outcome ==");
        Console.WriteLine($"Stop Kind: {outcome.StopKind}");
        Console.WriteLine($"Completed: {outcome.Completed.ToString().ToLowerInvariant()}");
        Console.WriteLine($"Follow-up: {outcome.FollowUp.Kind}");
        Console.WriteLine($"Session State: {outcome.Session.State}");
        Console.WriteLine($"Session Headline: {outcome.Session.Headline}");

        if (outcome.ActionExecution is not null)
        {
            Console.WriteLine();
            Console.WriteLine("Action Execution:");
            Console.WriteLine($"- Planned Actions: {outcome.ActionExecution.PlannedActionCount}");
            Console.WriteLine($"- Ready Requests: {outcome.ActionExecution.ReadyRequestCount}");
            Console.WriteLine($"- Applied Mutations: {outcome.ActionExecution.Execution.AppliedMutationCount}");
        }

        if (outcome.SupplementalMutations is not null)
        {
            Console.WriteLine();
            Console.WriteLine("Supplemental Mutation Phase:");
            Console.WriteLine($"- Commands: {outcome.SupplementalMutations.Commands.Count}");
            Console.WriteLine($"- Applied: {outcome.SupplementalMutations.Batch.AppliedCount}");
        }

        if (outcome.Validation is not null)
        {
            Console.WriteLine();
            Console.WriteLine("Validation:");
            Console.WriteLine($"- Is Valid: {outcome.Validation.IsValid.ToString().ToLowerInvariant()}");
            Console.WriteLine($"- Headline: {outcome.Validation.Views.Configuration.Headline}");
        }

        if (outcome.Calculation is not null)
        {
            Console.WriteLine();
            Console.WriteLine("Calculation:");
            Console.WriteLine($"- Succeeded: {outcome.Calculation.Succeeded.ToString().ToLowerInvariant()}");
            Console.WriteLine($"- Execution State: {outcome.Execution.State}");
            Console.WriteLine($"- Step Count: {outcome.Execution.StepCount}");
        }

        PrintPortMaterial(outcome.PortMaterial);
        PrintExecution(outcome.Execution);
        PrintReport(outcome.Report);
    }

    private static void PrintPortMaterial(UnitOperationHostPortMaterialSnapshot portMaterial)
    {
        Console.WriteLine();
        Console.WriteLine("== Port Material ==");
        Console.WriteLine($"State: {portMaterial.State}");
        Console.WriteLine($"Headline: {portMaterial.Headline}");
        foreach (var port in portMaterial.PortEntries)
        {
            Console.WriteLine(
                $"- {port.Name}: connected={port.IsConnected.ToString().ToLowerInvariant()}, materialState={port.MaterialState}, streams={string.Join(", ", port.BoundStreamIds)}");
            foreach (var material in port.MaterialEntries)
            {
                var phases = string.Join(
                    ", ",
                    material.Phases.Select(phase => $"{phase.Label}={phase.PhaseFraction:0.###}"));
                Console.WriteLine(
                    $"  stream={material.StreamId}, T={material.TemperatureK:0.###} K, P={material.PressurePa:0.###} Pa, F={material.TotalMolarFlowMolS:0.###} mol/s, phases={phases}");
            }
        }
    }

    private static void PrintExecution(UnitOperationHostExecutionSnapshot execution)
    {
        Console.WriteLine();
        Console.WriteLine("== Execution ==");
        Console.WriteLine($"State: {execution.State}");
        Console.WriteLine($"Headline: {execution.Headline}");
        foreach (var step in execution.StepEntries)
        {
            Console.WriteLine(
                $"- step={step.Index}, unit={step.UnitId}, kind={step.UnitKind}, consumed={string.Join(", ", step.ConsumedStreamIds)}, produced={string.Join(", ", step.ProducedStreamIds)}");
        }
    }

    private static void PrintReport(UnitOperationHostReportSnapshot report)
    {
        Console.WriteLine();
        Console.WriteLine("== Report ==");
        var document = UnitOperationHostReportFormatter.Format(
            UnitOperationHostReportPresenter.Present(report));
        Console.WriteLine(document.FormattedText);
    }
}

internal sealed class SampleHostOptions
{
    private SampleHostOptions(
        bool showHelp,
        string projectPath,
        string packageId,
        string? manifestPath,
        string? payloadPath,
        string? nativeLibraryDirectory)
    {
        ShowHelp = showHelp;
        ProjectPath = projectPath;
        PackageId = packageId;
        ManifestPath = manifestPath;
        PayloadPath = payloadPath;
        NativeLibraryDirectory = nativeLibraryDirectory;
    }

    public bool ShowHelp { get; }

    public string ProjectPath { get; }

    public string PackageId { get; }

    public string? ManifestPath { get; }

    public string? PayloadPath { get; }

    public string? NativeLibraryDirectory { get; }

    public bool LoadPackageFiles =>
        !string.IsNullOrWhiteSpace(ManifestPath) &&
        !string.IsNullOrWhiteSpace(PayloadPath);

    public static string HelpText =>
        """
        RadishFlow.CapeOpen.UnitOp.Mvp.SampleHost

        A PME-like thin external host sample that consumes UnitOp.Mvp formal view/planner/round APIs.

        Options:
          --project <path>        Project json path. Default: examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json
          --package <id>          Package id to solve with. Default: binary-hydrocarbon-lite-v1
          --manifest <path>       Optional property package manifest path
          --payload <path>        Optional property package payload path
          --native-lib-dir <dir>  Optional directory that contains rf_ffi.dll
          --help                  Show this help text
        """;

    public static SampleHostOptions Parse(string[] args)
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

        var manifestPath = values.TryGetValue("--manifest", out var manifestValue)
            ? Path.GetFullPath(manifestValue)
            : Path.Combine(
                repoRoot,
                "examples",
                "sample-components",
                "property-packages",
                "binary-hydrocarbon-lite-v1",
                "manifest.json");
        var payloadPath = values.TryGetValue("--payload", out var payloadValue)
            ? Path.GetFullPath(payloadValue)
            : Path.Combine(
                repoRoot,
                "examples",
                "sample-components",
                "property-packages",
                "binary-hydrocarbon-lite-v1",
                "payload.rfpkg");

        if (string.IsNullOrWhiteSpace(manifestPath) != string.IsNullOrWhiteSpace(payloadPath))
        {
            throw new ArgumentException("`--manifest` and `--payload` must be provided together.");
        }

        return new SampleHostOptions(
            showHelp: flags.Contains("--help"),
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
            manifestPath: manifestPath,
            payloadPath: payloadPath,
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
}
