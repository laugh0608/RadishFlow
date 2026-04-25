using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Persistence;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;
using System.Runtime.InteropServices;

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
            ("assembly-com-identity-contract", static _ => ContractTests.AssemblyComIdentity_StaysAlignedWithTypeLibraryRegistration()),
            ("registration-plan-contract", static _ => ContractTests.RegistrationPlan_ExposesGuardedExecutionBoundary()),
            ("registration-execute-confirm-contract", static _ => ContractTests.RegistrationExecution_RequiresMatchingConfirmToken()),
            ("registration-execute-preflight-contract", static _ => ContractTests.RegistrationExecution_StopsOnPreflightFailure()),
            ("pme-activation-probe-contract", static context => ContractTests.PmeActivationProbe_ExposesStandardActivationSurface(context)),
            ("pme-persistence-probe-contract", static context => ContractTests.PmePersistenceProbe_ExposesNoOpPersistStreamInit(context)),
            ("collection-contract", static context => ContractTests.Collections_ExposeStableLookupAndRejectInvalidSelectors(context)),
            ("object-definition-contract", static _ => ContractTests.ObjectDefinitionSnapshot_ExposesFrozenCatalogShape()),
            ("object-runtime-contract", static context => ContractTests.ObjectRuntimeSnapshot_ExposesFrozenObjectMetadata(context)),
            ("object-mutation-contract", static context => ContractTests.ObjectMutationDispatcher_AppliesCanonicalMutations(context)),
            ("object-mutation-batch-contract", static context => ContractTests.ObjectMutationDispatcher_DispatchesCanonicalBatch(context)),
            ("action-definition-contract", static context => ContractTests.ActionDefinitionCatalog_StaysAlignedWithIssueKinds(context)),
            ("configuration-contract", static context => ContractTests.ConfigurationSnapshot_ExposesReadinessAndNextOperations(context)),
            ("action-plan-contract", static context => ContractTests.ActionPlan_ExposesCanonicalChecklistShape(context)),
            ("action-mutation-bridge-contract", static context => ContractTests.ActionMutationBridge_TranslatesCanonicalHostActions(context)),
            ("action-execution-request-plan-contract", static context => ContractTests.ActionExecutionRequestPlanner_PlansRequestsFromHostInputs(context)),
            ("action-execution-contract", static context => ContractTests.ActionExecutionDispatcher_AppliesCanonicalHostActions(context)),
            ("action-execution-orchestration-contract", static context => ContractTests.ActionExecutionOrchestrator_RefreshesHostViews(context)),
            ("validation-round-contract", static context => ContractTests.ValidationRound_RefreshesHostViews(context)),
            ("calculation-round-contract", static context => ContractTests.CalculationRound_RefreshesHostViews(context)),
            ("host-round-contract", static context => ContractTests.HostRound_OrchestratesCanonicalHostPath(context)),
            ("port-material-contract", static context => ContractTests.PortMaterialSnapshot_ExposesBoundaryStreamsAndLifecycleState(context)),
            ("execution-snapshot-contract", static context => ContractTests.ExecutionSnapshot_ExposesStepAndDiagnosticShape(context)),
            ("session-snapshot-contract", static context => ContractTests.SessionSnapshot_ExposesUnifiedHostView(context)),
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
    public static void AssemblyComIdentity_StaysAlignedWithTypeLibraryRegistration()
    {
        AssertAssemblyTypeLibraryIdentity(
            typeof(RadishFlowCapeOpenUnitOperation).Assembly,
            "UnitOp.Mvp assembly should expose a stable COM type library identity.");
        AssertAssemblyTypeLibraryIdentity(
            typeof(ICapeUtilities).Assembly,
            "Interop assembly should expose the same COM type library identity used by the frozen MVP TLB.");

        AssertComDefaultInterface(
            typeof(RadishFlowCapeOpenUnitOperation),
            typeof(ICapeUtilities),
            "Unit operation COM default interface should expose ICapeUtilities as the first late-bound surface.");
        AssertComDefaultInterface(
            typeof(UnitOperationParameterCollection),
            typeof(ICapeCollection),
            "Parameter collection COM default interface should expose ICapeCollection for Count/Item late binding.");
        AssertComDefaultInterface(
            typeof(UnitOperationPortCollection),
            typeof(ICapeCollection),
            "Port collection COM default interface should expose ICapeCollection for Count/Item late binding.");
        AssertComDefaultInterface(
            typeof(UnitOperationParameterPlaceholder),
            typeof(ICapeParameter),
            "Parameter placeholder COM default interface should expose ICapeParameter for value/specification late binding.");
        AssertComDefaultInterface(
            typeof(UnitOperationPortPlaceholder),
            typeof(ICapeUnitPort),
            "Port placeholder COM default interface should expose ICapeUnitPort for connection late binding.");

        ContractAssert.True(
            typeof(ICapeUnitReport).IsAssignableFrom(typeof(RadishFlowCapeOpenUnitOperation)),
            "Unit operation should expose ICapeUnitReport as an optional PME activation/reporting surface.");
        ContractAssert.True(
            typeof(IPersistStreamInit).IsAssignableFrom(typeof(RadishFlowCapeOpenUnitOperation)),
            "Unit operation should expose IPersistStreamInit for PME canvas object persistence probing.");
    }

    private static void AssertComDefaultInterface(Type classType, Type expectedInterface, string context)
    {
        var defaultInterface = classType
            .GetCustomAttributes(typeof(System.Runtime.InteropServices.ComDefaultInterfaceAttribute), inherit: false)
            .OfType<System.Runtime.InteropServices.ComDefaultInterfaceAttribute>()
            .SingleOrDefault();

        ContractAssert.NotNull(defaultInterface, $"{context} Missing ComDefaultInterface.");
        ContractAssert.Equal(expectedInterface, defaultInterface!.Value, context);
    }

    private static void AssertAssemblyTypeLibraryIdentity(System.Reflection.Assembly assembly, string context)
    {
        var typeLibraryGuid = assembly.GetCustomAttributes(typeof(System.Runtime.InteropServices.GuidAttribute), inherit: false)
            .OfType<System.Runtime.InteropServices.GuidAttribute>()
            .SingleOrDefault();
        var typeLibraryVersion = assembly.GetCustomAttributes(typeof(System.Runtime.InteropServices.TypeLibVersionAttribute), inherit: false)
            .OfType<System.Runtime.InteropServices.TypeLibVersionAttribute>()
            .SingleOrDefault();

        ContractAssert.NotNull(typeLibraryGuid, $"{context} Missing GUID.");
        ContractAssert.Equal(UnitOperationComIdentity.TypeLibraryId, typeLibraryGuid!.Value, $"{context} GUID should stay aligned with Registration and IDL identity.");
        ContractAssert.NotNull(typeLibraryVersion, $"{context} Missing TypeLibVersion.");
        ContractAssert.Equal(1, typeLibraryVersion!.MajorVersion, $"{context} Major version should stay aligned with IDL version 1.0.");
        ContractAssert.Equal(0, typeLibraryVersion.MinorVersion, $"{context} Minor version should stay aligned with IDL version 1.0.");
    }

    public static void RegistrationPlan_ExposesGuardedExecutionBoundary()
    {
        var explicitComHostPath = ResolveFrozenComHostPath();
        var explicitTypeLibraryPath = ResolveFrozenTypeLibraryPath();
        var dryRunDescriptor = CapeOpenRegistrationDescriptor.CreateUnitOperationMvp(
            CapeOpenRegistrationAction.Register,
            CapeOpenRegistrationScope.CurrentUser,
            CapeOpenRegistrationExecutionMode.DryRun,
            explicitComHostPath,
            explicitTypeLibraryPath);

        ContractAssert.Equal(CapeOpenRegistrationExecutionMode.DryRun, dryRunDescriptor.ExecutionMode, "Dry-run descriptor should preserve execution mode.");
        ContractAssert.Equal("register-current-user-2F0E4C8F", dryRunDescriptor.RequiredConfirmToken, "Registration descriptor should expose the stable confirmation token.");
        ContractAssert.False(dryRunDescriptor.WritesRegistry, "Dry-run descriptor should not claim registry writes.");
        ContractAssert.Equal(4, dryRunDescriptor.BackupPlan.Count, "Backup plan should cover CLSID, ProgID, versioned ProgID and TypeLib roots.");
        ContractAssert.True(
            dryRunDescriptor.RegistryPlan.Any(static entry => entry.Operation == CapeOpenRegistryPlanOperation.Verify),
            "Register plan should preserve the pre-write comhost verification step.");
        ContractAssert.True(
            dryRunDescriptor.RegistryPlan.Any(static entry =>
                entry.Operation == CapeOpenRegistryPlanOperation.RegisterTypeLibrary &&
                entry.KeyPath.EndsWith(@"\TypeLib\{9D9E5F0D-5E28-4A45-9E2A-70A39D4C8D11}", StringComparison.Ordinal)),
            "Register plan should include the frozen TypeLib registration step for late-bound hosts.");
        ContractAssert.True(
            dryRunDescriptor.RegistryPlan.Any(static entry => entry.Operation == CapeOpenRegistryPlanOperation.SetValue && entry.KeyPath.Contains(@"Implemented Categories", StringComparison.Ordinal)),
            "Register plan should advertise CAPE-OPEN categories.");
        ContractAssert.True(
            dryRunDescriptor.ImplementedInterfaces.Any(static implementedInterface =>
                string.Equals(implementedInterface.Name, "ICapeUnitReport", StringComparison.Ordinal) &&
                string.Equals(implementedInterface.InterfaceId, "678C099B-0093-11D2-A67D-00105A42887F", StringComparison.OrdinalIgnoreCase)),
            "Register descriptor should advertise ICapeUnitReport as an implemented activation/reporting interface.");
        ContractAssert.True(
            dryRunDescriptor.ImplementedInterfaces.Any(static implementedInterface =>
                string.Equals(implementedInterface.Name, "IPersistStreamInit", StringComparison.Ordinal) &&
                string.Equals(implementedInterface.InterfaceId, ComPersistenceInterfaceIds.IPersistStreamInit, StringComparison.OrdinalIgnoreCase)),
            "Register descriptor should advertise IPersistStreamInit as an implemented PME canvas persistence interface.");
        ContractAssert.True(
            dryRunDescriptor.RegistryPlan.Any(static entry =>
                entry.Operation == CapeOpenRegistryPlanOperation.SetValue &&
                string.Equals(entry.KeyPath, @"Software\Classes\CLSID\{2F0E4C8F-7C89-4DA7-A5D3-5F8C987D6718}\InprocServer32", StringComparison.Ordinal) &&
                string.Equals(entry.ValueName, "ThreadingModel", StringComparison.Ordinal) &&
                string.Equals(entry.ValueData, "Apartment", StringComparison.Ordinal)),
            "Register plan should expose classic COM ThreadingModel metadata.");
        ContractAssert.True(
            dryRunDescriptor.RegistryPlan.Any(static entry =>
                entry.Operation == CapeOpenRegistryPlanOperation.SetValue &&
                entry.KeyPath.Contains(@"\CapeDescription", StringComparison.Ordinal) &&
                string.Equals(entry.ValueName, "Name", StringComparison.Ordinal)),
            "Register plan should expose CAPE-OPEN description metadata for discovery UIs.");
        ContractAssert.True(
            dryRunDescriptor.RegistryPlan.Any(static entry =>
                entry.Operation == CapeOpenRegistryPlanOperation.SetValue &&
                entry.KeyPath.EndsWith(@"\Programmable", StringComparison.Ordinal) &&
                entry.ValueName is null),
            "Register plan should mark the component as programmable for legacy hosts.");
        ContractAssert.True(
            dryRunDescriptor.RegistryPlan.Any(static entry =>
                entry.Operation == CapeOpenRegistryPlanOperation.SetValue &&
                entry.KeyPath.EndsWith(@"\TypeLib", StringComparison.Ordinal) &&
                string.Equals(entry.ValueData, "{9D9E5F0D-5E28-4A45-9E2A-70A39D4C8D11}", StringComparison.Ordinal)),
            "Register plan should bind the CLSID tree back to the registered type library GUID.");
        ContractAssert.True(
            dryRunDescriptor.RegistryPlan.Any(static entry =>
                entry.Operation == CapeOpenRegistryPlanOperation.SetValue &&
                entry.KeyPath.EndsWith(@"\CurVer", StringComparison.Ordinal) &&
                string.Equals(entry.ValueData, "RadishFlow.CapeOpen.UnitOp.Mvp.1", StringComparison.Ordinal)),
            "Register plan should bind the stable ProgID to the current versioned ProgID.");
        ContractAssert.True(
            dryRunDescriptor.RegistryPlan.Any(static entry =>
                entry.Operation == CapeOpenRegistryPlanOperation.SetValue &&
                entry.KeyPath.EndsWith(@"RadishFlow.CapeOpen.UnitOp.Mvp\CLSID", StringComparison.Ordinal) &&
                string.Equals(entry.ValueData, "{2F0E4C8F-7C89-4DA7-A5D3-5F8C987D6718}", StringComparison.Ordinal)),
            "Register plan should write the stable ProgID CLSID mapping using the canonical braced GUID string.");
        ContractAssert.True(
            dryRunDescriptor.RegistryPlan.Any(static entry =>
                entry.Operation == CapeOpenRegistryPlanOperation.SetValue &&
                entry.KeyPath.EndsWith(@"RadishFlow.CapeOpen.UnitOp.Mvp.1\CLSID", StringComparison.Ordinal) &&
                string.Equals(entry.ValueData, "{2F0E4C8F-7C89-4DA7-A5D3-5F8C987D6718}", StringComparison.Ordinal)),
            "Register plan should write the versioned ProgID CLSID mapping using the canonical braced GUID string.");
        ContractAssert.True(
            dryRunDescriptor.PreflightChecks.Any(static check =>
                check.Status == CapeOpenPreflightCheckStatus.Pass &&
                string.Equals(check.Name, "type library identity", StringComparison.Ordinal) &&
                check.Detail.Contains("GUID/version match", StringComparison.Ordinal)),
            "Register preflight should confirm that the frozen TLB identity is ready for registration.");
        ContractAssert.True(
            dryRunDescriptor.PreflightChecks.Any(static check =>
                check.Status == CapeOpenPreflightCheckStatus.Pass &&
                string.Equals(check.Name, "comhost runtime layout", StringComparison.Ordinal)),
            "Register preflight should confirm that the resolved comhost directory contains the required .NET runtime sidecars.");

        var defaultDescriptor = CapeOpenRegistrationDescriptor.CreateUnitOperationMvp(
            CapeOpenRegistrationAction.Register,
            CapeOpenRegistrationScope.CurrentUser,
            CapeOpenRegistrationExecutionMode.DryRun,
            null,
            null);
        ContractAssert.Contains(
            defaultDescriptor.ResolvedComHostPath,
            @"RadishFlow.CapeOpen.UnitOp.Mvp\bin\",
            "Default resolver should prefer the UnitOp.Mvp project output directory in repository builds.");
        ContractAssert.True(
            defaultDescriptor.PreflightChecks.Any(static check =>
                check.Status == CapeOpenPreflightCheckStatus.Pass &&
                string.Equals(check.Name, "comhost runtime layout", StringComparison.Ordinal)),
            "Default resolver should land on a comhost directory that is activation-ready for .NET COM hosting.");

        var executeDescriptor = CapeOpenRegistrationDescriptor.CreateUnitOperationMvp(
            CapeOpenRegistrationAction.Unregister,
            CapeOpenRegistrationScope.CurrentUser,
            CapeOpenRegistrationExecutionMode.Execute,
            explicitComHostPath,
            explicitTypeLibraryPath);
        ContractAssert.True(executeDescriptor.WritesRegistry, "Execute descriptor should explicitly mark registry writes.");
        ContractAssert.Equal(5, executeDescriptor.RegistryPlan.Count, "Unregister plan should cover TypeLib API unregistration plus the four top-level registration roots.");
        ContractAssert.True(
            executeDescriptor.RegistryPlan.Any(static entry => entry.Operation == CapeOpenRegistryPlanOperation.UnregisterTypeLibrary),
            "Unregister plan should explicitly include the TypeLib unregistration step.");
        ContractAssert.Equal(
            4,
            executeDescriptor.RegistryPlan.Count(static entry => entry.Operation == CapeOpenRegistryPlanOperation.DeleteTree),
            "Unregister plan should still constrain tree deletion to the four frozen registration roots.");
    }

    public static void RegistrationExecution_RequiresMatchingConfirmToken()
    {
        var explicitComHostPath = ResolveFrozenComHostPath();
        var explicitTypeLibraryPath = ResolveFrozenTypeLibraryPath();
        var backupDirectory = Path.Combine(Path.GetTempPath(), "radishflow-registration-confirm-" + Guid.NewGuid().ToString("N"));
        var options = RegistrationOptions.Parse(
        [
            "--execute",
            "--confirm",
            "wrong-token",
            "--comhost",
            explicitComHostPath,
            "--typelib",
            explicitTypeLibraryPath,
            "--backup-dir",
            backupDirectory,
        ]);
        var descriptor = CapeOpenRegistrationDescriptor.CreateUnitOperationMvp(
            options.Action,
            options.Scope,
            options.ExecutionMode,
            options.ComHostPath,
            options.TypeLibraryPath);

        var error = ContractAssert.Throws<InvalidOperationException>(
            () => CapeOpenRegistrationExecutor.Execute(descriptor, options),
            "Registration execution should reject a missing or mismatched confirmation token before writing.");

        ContractAssert.Contains(error.Message, descriptor.RequiredConfirmToken, "Confirmation failures should report the required token.");
        ContractAssert.False(Directory.Exists(backupDirectory), "Rejected execution should not create backup output.");
    }

    public static void RegistrationExecution_StopsOnPreflightFailure()
    {
        var missingComHostPath = Path.Combine(Path.GetTempPath(), "radishflow-registration-missing-" + Guid.NewGuid().ToString("N") + ".dll");
        var explicitTypeLibraryPath = ResolveFrozenTypeLibraryPath();
        var backupDirectory = Path.Combine(Path.GetTempPath(), "radishflow-registration-preflight-" + Guid.NewGuid().ToString("N"));
        var options = RegistrationOptions.Parse(
        [
            "--execute",
            "--confirm",
            "register-current-user-2F0E4C8F",
            "--comhost",
            missingComHostPath,
            "--typelib",
            explicitTypeLibraryPath,
            "--backup-dir",
            backupDirectory,
        ]);
        var descriptor = CapeOpenRegistrationDescriptor.CreateUnitOperationMvp(
            options.Action,
            options.Scope,
            options.ExecutionMode,
            options.ComHostPath,
            options.TypeLibraryPath);

        ContractAssert.True(
            descriptor.PreflightChecks.Any(static check =>
                check.Status == CapeOpenPreflightCheckStatus.Fail &&
                string.Equals(check.Name, "comhost path", StringComparison.Ordinal)),
            "Execute descriptor should surface failing preflight checks before any registry write.");

        var error = ContractAssert.Throws<InvalidOperationException>(
            () => CapeOpenRegistrationExecutor.Execute(descriptor, options),
            "Registration execution should stop when preflight reports Fail.");

        ContractAssert.Contains(error.Message, "preflight failures", "Preflight-blocked execution should explain why no write happened.");
        ContractAssert.False(Directory.Exists(backupDirectory), "Preflight-blocked execution should not create backup output.");
    }

    private static string ResolveFrozenTypeLibraryPath()
    {
        var fileName = UnitOperationComIdentity.TypeLibraryFileName;
        var baseDirectory = AppContext.BaseDirectory;
        var candidates = new[]
        {
            Path.Combine(baseDirectory, fileName),
            Path.Combine(baseDirectory, "typelib", fileName),
            Path.GetFullPath(Path.Combine(baseDirectory, @"..\..\..\..\RadishFlow.CapeOpen.UnitOp.Mvp\typelib", fileName)),
            Path.GetFullPath(Path.Combine(baseDirectory, @"..\..\..\..\..\RadishFlow.CapeOpen.UnitOp.Mvp\typelib", fileName)),
        };

        var resolved = candidates.FirstOrDefault(File.Exists);
        return resolved
               ?? throw new InvalidOperationException(
                   $"Failed to locate frozen type library fixture `{fileName}`. Candidates: {string.Join(", ", candidates)}");
    }

    private static string ResolveFrozenComHostPath()
    {
        const string fileName = "RadishFlow.CapeOpen.UnitOp.Mvp.comhost.dll";
        var baseDirectory = AppContext.BaseDirectory;
        var candidates = new[]
        {
            Path.GetFullPath(Path.Combine(baseDirectory, @"..\..\..\..\RadishFlow.CapeOpen.UnitOp.Mvp\bin\Debug\net10.0", fileName)),
            Path.GetFullPath(Path.Combine(baseDirectory, @"..\..\..\..\..\RadishFlow.CapeOpen.UnitOp.Mvp\bin\Debug\net10.0", fileName)),
            Path.Combine(baseDirectory, fileName),
        };

        var resolved = candidates.FirstOrDefault(static path =>
                File.Exists(path) &&
                File.Exists(Path.Combine(Path.GetDirectoryName(path)!, "RadishFlow.CapeOpen.UnitOp.Mvp.runtimeconfig.json")) &&
                File.Exists(Path.Combine(Path.GetDirectoryName(path)!, "RadishFlow.CapeOpen.UnitOp.Mvp.deps.json")))
            ?? candidates.FirstOrDefault(File.Exists);
        return resolved
               ?? throw new InvalidOperationException(
                   $"Failed to locate generated comhost fixture `{fileName}`. Candidates: {string.Join(", ", candidates)}");
    }

    public static void PmeActivationProbe_ExposesStandardActivationSurface(ContractTestContext context)
    {
        var identity = (ICapeIdentification)context.UnitOperation;
        ContractAssert.Equal(UnitOperationComIdentity.DisplayName, identity.ComponentName, "Activation probe should read ICapeIdentification.ComponentName.");
        ContractAssert.Contains(identity.ComponentDescription, "CAPE-OPEN", "Activation probe should read ICapeIdentification.ComponentDescription.");

        var utilities = (ICapeUtilities)context.UnitOperation;
        utilities.Initialize();
        ContractAssert.NotNull(utilities.Parameters, "Activation probe should read ICapeUtilities.Parameters.");

        var unit = (ICapeUnit)context.UnitOperation;
        ContractAssert.NotNull(unit.Ports, "Activation probe should read ICapeUnit.Ports.");

        var validationMessage = string.Empty;
        ContractAssert.False(unit.Validate(ref validationMessage), "Activation probe should tolerate early Validate() before PME inputs are provided.");
        ContractAssert.Contains(validationMessage, "Required parameter", "Early activation Validate() should return a diagnostic message instead of crashing.");

        var report = (ICapeUnitReport)context.UnitOperation;
        var availableReports = report.reports as string[];
        ContractAssert.True(
            availableReports is { Length: 1 },
            "Activation probe should expose a stable report name array.");
        ContractAssert.Equal(report.selectedReport, availableReports![0], "Default selected report should be one of the advertised reports.");

        var reportContent = "stale report text";
        report.ProduceReport(ref reportContent);
        ContractAssert.Equal(context.UnitOperation.GetCalculationReportText(), reportContent, "ICapeUnitReport.ProduceReport should reuse the canonical calculation report text.");

        var reportInterfacePointer = IntPtr.Zero;
        try
        {
            reportInterfacePointer = Marshal.GetComInterfaceForObject(context.UnitOperation, typeof(ICapeUnitReport));
            ContractAssert.True(
                reportInterfacePointer != IntPtr.Zero,
                "COM QueryInterface for ICapeUnitReport should succeed for the unit operation.");
        }
        finally
        {
            if (reportInterfacePointer != IntPtr.Zero)
            {
                Marshal.Release(reportInterfacePointer);
            }
        }

        utilities.Terminate();
    }

    public static void PmePersistenceProbe_ExposesNoOpPersistStreamInit(ContractTestContext context)
    {
        var persistence = (IPersistStreamInit)context.UnitOperation;

        ContractAssert.Equal(
            ComHResults.SOk,
            persistence.GetClassID(out var classId),
            "IPersistStreamInit.GetClassID should return S_OK.");
        ContractAssert.Equal(
            Guid.Parse(UnitOperationComIdentity.ClassId),
            classId,
            "IPersistStreamInit.GetClassID should return the unit operation CLSID.");
        ContractAssert.Equal(
            ComHResults.SFalse,
            persistence.IsDirty(),
            "IPersistStreamInit.IsDirty should report clean no-op persistence state.");
        ContractAssert.Equal(
            ComHResults.SOk,
            persistence.InitNew(),
            "IPersistStreamInit.InitNew should accept PME canvas creation probing.");
        ContractAssert.Equal(
            ComHResults.SOk,
            persistence.Load(null),
            "IPersistStreamInit.Load should no-op successfully for the MVP stateless persistence surface.");
        ContractAssert.Equal(
            ComHResults.SOk,
            persistence.Save(null, clearDirty: true),
            "IPersistStreamInit.Save should no-op successfully for the MVP stateless persistence surface.");
        ContractAssert.Equal(
            ComHResults.SOk,
            persistence.GetSizeMax(out var size),
            "IPersistStreamInit.GetSizeMax should return S_OK.");
        ContractAssert.Equal(0L, size, "IPersistStreamInit.GetSizeMax should report zero bytes for no-op persistence.");

        var persistenceInterfacePointer = IntPtr.Zero;
        try
        {
            persistenceInterfacePointer = Marshal.GetComInterfaceForObject(context.UnitOperation, typeof(IPersistStreamInit));
            ContractAssert.True(
                persistenceInterfacePointer != IntPtr.Zero,
                "COM QueryInterface for IPersistStreamInit should succeed for the unit operation.");
        }
        finally
        {
            if (persistenceInterfacePointer != IntPtr.Zero)
            {
                Marshal.Release(persistenceInterfacePointer);
            }
        }
    }

    public static void Collections_ExposeStableLookupAndRejectInvalidSelectors(ContractTestContext context)
    {
        context.Initialize();

        ContractAssert.Equal(4, context.ParameterCollection.Count(), "Parameter collection Count() should remain stable.");
        ContractAssert.Equal(2, context.PortCollection.Count(), "Port collection Count() should remain stable.");
        ContractAssert.Equal(4, context.UnitOperation.Parameters.Count, "Parameter collection IReadOnlyList.Count should stay aligned with ICapeCollection.Count().");
        ContractAssert.Equal(2, context.UnitOperation.Ports.Count, "Port collection IReadOnlyList.Count should stay aligned with ICapeCollection.Count().");
        ContractAssert.Equal(UnitOperationParameterCatalog.CollectionDefinition.Name, context.UnitOperation.Parameters.ComponentName, "Parameter collection name should come from the frozen collection definition.");
        ContractAssert.Equal(UnitOperationParameterCatalog.CollectionDefinition.Description, context.UnitOperation.Parameters.ComponentDescription, "Parameter collection description should come from the frozen collection definition.");
        ContractAssert.Equal(UnitOperationPortCatalog.CollectionDefinition.Name, context.UnitOperation.Ports.ComponentName, "Port collection name should come from the frozen collection definition.");
        ContractAssert.Equal(UnitOperationPortCatalog.CollectionDefinition.Description, context.UnitOperation.Ports.ComponentDescription, "Port collection description should come from the frozen collection definition.");
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
            var specification = (ICapeParameterSpec)parameter.Specification;
            ContractAssert.Equal(definition.SpecificationType, specification.Type, "Runtime parameter spec type should match the frozen catalog definition.");
            ContractAssert.SequenceEqual(definition.SpecificationDimensionality, specification.Dimensionality, "Runtime parameter dimensionality should match the frozen catalog definition.");
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
        ContractAssert.Equal(
            UnitOperationPortBoundaryMaterialRole.BoundaryInputs,
            UnitOperationPortCatalog.Feed.BoundaryMaterialRole,
            "Feed port should freeze the boundary-input material role.");
        ContractAssert.Equal(
            UnitOperationPortBoundaryMaterialRole.BoundaryOutputs,
            UnitOperationPortCatalog.Product.BoundaryMaterialRole,
            "Product port should freeze the boundary-output material role.");
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

    public static void ObjectDefinitionSnapshot_ExposesFrozenCatalogShape()
    {
        var snapshot = UnitOperationHostObjectDefinitionReader.Read();

        ContractAssert.Equal(UnitOperationParameterCatalog.CollectionDefinition.Name, snapshot.ParameterCollection.Name, "Object definition snapshot should preserve parameter collection name.");
        ContractAssert.Equal(UnitOperationParameterCatalog.CollectionDefinition.Description, snapshot.ParameterCollection.Description, "Object definition snapshot should preserve parameter collection description.");
        ContractAssert.Equal(UnitOperationPortCatalog.CollectionDefinition.Name, snapshot.PortCollection.Name, "Object definition snapshot should preserve port collection name.");
        ContractAssert.Equal(UnitOperationPortCatalog.CollectionDefinition.Description, snapshot.PortCollection.Description, "Object definition snapshot should preserve port collection description.");
        ContractAssert.Equal(4, snapshot.ParameterCollection.Count, "Object definition snapshot should expose parameter collection count.");
        ContractAssert.Equal(2, snapshot.PortCollection.Count, "Object definition snapshot should expose port collection count.");
        ContractAssert.SequenceEqual(UnitOperationParameterCatalog.OrderedNames, snapshot.ParameterEntries.Select(static entry => entry.Name), "Object definition snapshot should expose parameter entries in catalog order.");
        ContractAssert.SequenceEqual(UnitOperationPortCatalog.OrderedNames, snapshot.PortEntries.Select(static entry => entry.Name), "Object definition snapshot should expose port entries in catalog order.");

        var flowsheet = snapshot.GetParameter(UnitOperationParameterCatalog.FlowsheetJson.Name);
        ContractAssert.SameReference(flowsheet, snapshot.ParameterCollection.GetEntry(UnitOperationParameterCatalog.FlowsheetJson.Name), "Object definition parameter collection lookup should return the same entry instance.");
        ContractAssert.Equal(UnitOperationParameterCatalog.FlowsheetJson.Description, flowsheet.Description, "Object definition snapshot should preserve parameter description.");
        ContractAssert.Equal(UnitOperationParameterCatalog.FlowsheetJson.IsRequired, flowsheet.IsRequired, "Object definition snapshot should preserve parameter required flag.");
        ContractAssert.Equal(UnitOperationParameterCatalog.FlowsheetJson.ValueKind, flowsheet.ValueKind, "Object definition snapshot should preserve parameter value kind.");
        ContractAssert.Equal(UnitOperationParameterCatalog.FlowsheetJson.AllowsEmptyValue, flowsheet.AllowsEmptyValue, "Object definition snapshot should preserve parameter allow-empty flag.");
        ContractAssert.Equal(UnitOperationParameterCatalog.FlowsheetJson.ConfigurationOperationName, flowsheet.ConfigurationOperationName, "Object definition snapshot should preserve parameter configuration operation.");
        ContractAssert.Equal(UnitOperationParameterCatalog.FlowsheetJson.Mode, flowsheet.Mode, "Object definition snapshot should preserve parameter mode.");
        ContractAssert.Equal(UnitOperationParameterCatalog.FlowsheetJson.DefaultValue, flowsheet.DefaultValue, "Object definition snapshot should preserve parameter default value.");
        ContractAssert.Equal(UnitOperationParameterCatalog.FlowsheetJson.SpecificationType, flowsheet.SpecificationType, "Object definition snapshot should preserve parameter spec type.");
        ContractAssert.SequenceEqual(UnitOperationParameterCatalog.FlowsheetJson.SpecificationDimensionality, flowsheet.SpecificationDimensionality, "Object definition snapshot should preserve parameter spec dimensionality.");
        ContractAssert.True(flowsheet.Capabilities.CanWriteValue, "Object definition parameter should expose value write capability.");
        ContractAssert.True(flowsheet.Capabilities.CanResetValue, "Object definition parameter should expose reset capability.");
        ContractAssert.False(flowsheet.Capabilities.CanMutateMode, "Object definition parameter should expose immutable mode capability.");
        ContractAssert.False(flowsheet.Capabilities.CanMutateIdentity, "Object definition parameter should expose immutable identity capability.");

        var product = snapshot.GetPort(UnitOperationPortCatalog.Product.Name);
        ContractAssert.SameReference(product, snapshot.PortCollection.GetEntry(UnitOperationPortCatalog.Product.Name), "Object definition port collection lookup should return the same entry instance.");
        ContractAssert.Equal(UnitOperationPortCatalog.Product.Description, product.Description, "Object definition snapshot should preserve port description.");
        ContractAssert.Equal(UnitOperationPortCatalog.Product.IsRequired, product.IsRequired, "Object definition snapshot should preserve port required flag.");
        ContractAssert.Equal(UnitOperationPortCatalog.Product.Direction, product.Direction, "Object definition snapshot should preserve port direction.");
        ContractAssert.Equal(UnitOperationPortCatalog.Product.PortType, product.PortType, "Object definition snapshot should preserve port type.");
        ContractAssert.Equal(UnitOperationPortCatalog.Product.ConnectionOperationName, product.ConnectionOperationName, "Object definition snapshot should preserve port connection operation.");
        ContractAssert.Equal(UnitOperationPortCatalog.Product.BoundaryMaterialRole, product.BoundaryMaterialRole, "Object definition snapshot should preserve port boundary-material role.");
        ContractAssert.True(product.Capabilities.CanConnect, "Object definition port should expose connect capability.");
        ContractAssert.True(product.Capabilities.CanDisconnect, "Object definition port should expose disconnect capability.");
        ContractAssert.False(product.Capabilities.CanReplaceConnectionWithoutDisconnect, "Object definition port should expose explicit-disconnect replacement capability.");
        ContractAssert.False(product.Capabilities.CanMutateIdentity, "Object definition port should expose immutable identity capability.");

        var missingParameterError = ContractAssert.Throws<ArgumentException>(
            () => snapshot.ParameterCollection.GetEntry("missing-parameter"),
            "Unknown object definition parameter lookups should be rejected.");
        ContractAssert.Contains(missingParameterError.Message, "Unknown unit operation host parameter definition", "Missing object definition parameter failures should stay explicit.");
        var missingPortError = ContractAssert.Throws<ArgumentException>(
            () => snapshot.PortCollection.GetEntry("missing-port"),
            "Unknown object definition port lookups should be rejected.");
        ContractAssert.Contains(missingPortError.Message, "Unknown unit operation host port definition", "Missing object definition port failures should stay explicit.");
    }

    public static void ObjectRuntimeSnapshot_ExposesFrozenObjectMetadata(ContractTestContext context)
    {
        var constructedSnapshot = context.ReadObjectRuntime();
        ContractAssert.Equal(UnitOperationHostObjectRuntimeState.Constructed, constructedSnapshot.LifecycleState, "Object runtime snapshot should preserve constructed lifecycle state.");
        ContractAssert.Equal(UnitOperationParameterCatalog.CollectionDefinition.Name, constructedSnapshot.ParameterCollection.Name, "Object runtime snapshot should preserve parameter collection name.");
        ContractAssert.Equal(UnitOperationParameterCatalog.CollectionDefinition.Description, constructedSnapshot.ParameterCollection.Description, "Object runtime snapshot should preserve parameter collection description.");
        ContractAssert.Equal(UnitOperationPortCatalog.CollectionDefinition.Name, constructedSnapshot.PortCollection.Name, "Object runtime snapshot should preserve port collection name.");
        ContractAssert.Equal(UnitOperationPortCatalog.CollectionDefinition.Description, constructedSnapshot.PortCollection.Description, "Object runtime snapshot should preserve port collection description.");
        ContractAssert.Equal(4, constructedSnapshot.ParameterCollection.Count, "Object runtime snapshot should expose parameter collection count.");
        ContractAssert.Equal(2, constructedSnapshot.PortCollection.Count, "Object runtime snapshot should expose port collection count.");
        ContractAssert.Equal(4, constructedSnapshot.ParameterEntries.Count, "Object runtime snapshot should expose parameter entries in frozen catalog order.");
        ContractAssert.Equal(2, constructedSnapshot.PortEntries.Count, "Object runtime snapshot should expose port entries in frozen catalog order.");

        var constructedFlowsheet = constructedSnapshot.GetParameter(UnitOperationParameterCatalog.FlowsheetJson.Name);
        ContractAssert.SameReference(constructedFlowsheet, constructedSnapshot.ParameterCollection.GetEntry(UnitOperationParameterCatalog.FlowsheetJson.Name), "Object runtime parameter collection lookup should return the same entry instance.");
        ContractAssert.False(constructedFlowsheet.IsConfigured, "Constructed flowsheet parameter should start unconfigured in runtime snapshot.");
        ContractAssert.Equal(UnitOperationParameterCatalog.FlowsheetJson.Mode, constructedFlowsheet.Mode, "Runtime snapshot should preserve parameter mode.");
        ContractAssert.Equal(UnitOperationParameterCatalog.FlowsheetJson.SpecificationType, constructedFlowsheet.SpecificationType, "Runtime snapshot should preserve parameter spec type.");
        ContractAssert.SequenceEqual(UnitOperationParameterCatalog.FlowsheetJson.SpecificationDimensionality, constructedFlowsheet.SpecificationDimensionality, "Runtime snapshot should preserve parameter spec dimensionality.");
        ContractAssert.True(constructedFlowsheet.Capabilities.CanWriteValue, "Runtime parameter should expose value write capability.");
        ContractAssert.True(constructedFlowsheet.Capabilities.CanResetValue, "Runtime parameter should expose reset capability.");
        ContractAssert.False(constructedFlowsheet.Capabilities.CanMutateMode, "Runtime parameter should expose immutable mode capability.");
        ContractAssert.False(constructedFlowsheet.Capabilities.CanMutateIdentity, "Runtime parameter should expose immutable identity capability.");

        var constructedProduct = constructedSnapshot.GetPort(UnitOperationPortCatalog.Product.Name);
        ContractAssert.SameReference(constructedProduct, constructedSnapshot.PortCollection.GetEntry(UnitOperationPortCatalog.Product.Name), "Object runtime port collection lookup should return the same entry instance.");
        ContractAssert.False(constructedProduct.IsConnected, "Constructed product port should start disconnected in runtime snapshot.");
        ContractAssert.Equal(UnitOperationPortCatalog.Product.BoundaryMaterialRole, constructedProduct.BoundaryMaterialRole, "Runtime snapshot should preserve port boundary-material role.");
        ContractAssert.True(constructedProduct.Capabilities.CanConnect, "Runtime port should expose connect capability.");
        ContractAssert.True(constructedProduct.Capabilities.CanDisconnect, "Runtime port should expose disconnect capability.");
        ContractAssert.False(constructedProduct.Capabilities.CanReplaceConnectionWithoutDisconnect, "Runtime port should expose explicit-disconnect replacement capability.");
        ContractAssert.False(constructedProduct.Capabilities.CanMutateIdentity, "Runtime port should expose immutable identity capability.");

        context.ConfigureMinimumValidInputs();

        var readySnapshot = context.ReadObjectRuntime();
        ContractAssert.Equal(UnitOperationHostObjectRuntimeState.Initialized, readySnapshot.LifecycleState, "Object runtime snapshot should preserve initialized lifecycle state.");
        ContractAssert.True(readySnapshot.GetParameter(UnitOperationParameterCatalog.FlowsheetJson.Name).IsConfigured, "Configured flowsheet parameter should appear configured in runtime snapshot.");
        ContractAssert.True(readySnapshot.GetParameter(UnitOperationParameterCatalog.PropertyPackageId.Name).IsConfigured, "Configured package parameter should appear configured in runtime snapshot.");
        ContractAssert.True(readySnapshot.GetPort(UnitOperationPortCatalog.Feed.Name).IsConnected, "Connected feed port should appear connected in runtime snapshot.");
        ContractAssert.True(readySnapshot.GetPort(UnitOperationPortCatalog.Product.Name).IsConnected, "Connected product port should appear connected in runtime snapshot.");

        context.UnitOperation.Terminate();

        var terminatedSnapshot = context.ReadObjectRuntime();
        ContractAssert.Equal(UnitOperationHostObjectRuntimeState.Terminated, terminatedSnapshot.LifecycleState, "Object runtime snapshot should preserve terminated lifecycle state.");
        ContractAssert.Equal(0, terminatedSnapshot.ParameterCollection.Count, "Terminated runtime snapshot should expose empty parameter collection.");
        ContractAssert.Equal(0, terminatedSnapshot.PortCollection.Count, "Terminated runtime snapshot should expose empty port collection.");
        ContractAssert.Equal(0, terminatedSnapshot.ParameterEntries.Count, "Terminated runtime snapshot should not bypass lifecycle guards to expose parameters.");
        ContractAssert.Equal(0, terminatedSnapshot.PortEntries.Count, "Terminated runtime snapshot should not bypass lifecycle guards to expose ports.");
    }

    public static void ObjectMutationDispatcher_AppliesCanonicalMutations(ContractTestContext context)
    {
        context.Initialize();

        var setParameterOutcome = UnitOperationHostObjectMutationDispatcher.SetParameterValue(
            context.UnitOperation,
            UnitOperationParameterCatalog.PropertyPackageId.Name,
            context.PackageId);
        AssertMutationOutcome(
            setParameterOutcome,
            UnitOperationHostObjectMutationKind.SetParameterValue,
            UnitOperationHostActionTargetKind.Parameter,
            UnitOperationParameterCatalog.PropertyPackageId.Name);
        ContractAssert.Equal(context.PackageId, context.PackageIdParameter.Value, "SetParameterValue mutation should write the target parameter value.");
        ContractAssert.True(context.PackageIdParameter.IsConfigured, "SetParameterValue mutation should configure the target parameter.");
        ContractAssert.Equal(CapeValidationStatus.NotValidated, context.UnitOperation.ValStatus, "SetParameterValue mutation should invalidate validation state.");

        var resetParameterOutcome = UnitOperationHostObjectMutationDispatcher.ResetParameter(
            context.UnitOperation,
            UnitOperationParameterCatalog.PropertyPackageId.Name);
        AssertMutationOutcome(
            resetParameterOutcome,
            UnitOperationHostObjectMutationKind.ResetParameter,
            UnitOperationHostActionTargetKind.Parameter,
            UnitOperationParameterCatalog.PropertyPackageId.Name);
        ContractAssert.Null(context.PackageIdParameter.Value, "ResetParameter mutation should restore the target parameter default value.");
        ContractAssert.False(context.PackageIdParameter.IsConfigured, "ResetParameter mutation should clear configured state for optional-null default values.");

        var connectPortOutcome = UnitOperationHostObjectMutationDispatcher.ConnectPort(
            context.UnitOperation,
            UnitOperationPortCatalog.Feed.Name,
            new ContractConnectedObject("Dispatcher Feed"));
        AssertMutationOutcome(
            connectPortOutcome,
            UnitOperationHostObjectMutationKind.ConnectPort,
            UnitOperationHostActionTargetKind.Port,
            UnitOperationPortCatalog.Feed.Name);
        ContractAssert.True(context.FeedPort.IsConnected, "ConnectPort mutation should connect the target port.");

        var replacementError = ContractAssert.Throws<CapeBadInvocationOrderException>(
            () => UnitOperationHostObjectMutationDispatcher.ConnectPort(
                context.UnitOperation,
                UnitOperationPortCatalog.Feed.Name,
                new ContractConnectedObject("Replacement Dispatcher Feed")),
            "ConnectPort mutation should preserve explicit-disconnect replacement semantics.");
        ContractAssert.Contains(replacementError.Description, "Disconnect it before replacing", "ConnectPort replacement failures should preserve port guidance.");

        var disconnectPortOutcome = UnitOperationHostObjectMutationDispatcher.DisconnectPort(
            context.UnitOperation,
            UnitOperationPortCatalog.Feed.Name);
        AssertMutationOutcome(
            disconnectPortOutcome,
            UnitOperationHostObjectMutationKind.DisconnectPort,
            UnitOperationHostActionTargetKind.Port,
            UnitOperationPortCatalog.Feed.Name);
        ContractAssert.False(context.FeedPort.IsConnected, "DisconnectPort mutation should disconnect the target port.");

        context.UnitOperation.Terminate();

        var postTerminateMutationError = ContractAssert.Throws<CapeBadInvocationOrderException>(
            () => UnitOperationHostObjectMutationDispatcher.SetParameterValue(
                context.UnitOperation,
                UnitOperationParameterCatalog.PropertyPackageId.Name,
                context.PackageId),
            "Object mutation dispatcher should preserve lifecycle guard after Terminate().");
        ContractAssert.Contains(postTerminateMutationError.Description, "Terminate has already been called", "Post-terminate mutation failures should preserve lifecycle guidance.");
    }

    public static void ObjectMutationDispatcher_DispatchesCanonicalBatch(ContractTestContext context)
    {
        context.Initialize();

        var batchResult = UnitOperationHostObjectMutationDispatcher.DispatchBatch(
            context.UnitOperation,
            [
                UnitOperationHostObjectMutationCommand.SetParameterValue(UnitOperationParameterCatalog.FlowsheetJson.Name, context.FlowsheetJsonText),
                UnitOperationHostObjectMutationCommand.SetParameterValue(UnitOperationParameterCatalog.PropertyPackageId.Name, context.PackageId),
                UnitOperationHostObjectMutationCommand.ConnectPort(UnitOperationPortCatalog.Feed.Name, new ContractConnectedObject("Batch Feed")),
                UnitOperationHostObjectMutationCommand.ConnectPort(UnitOperationPortCatalog.Product.Name, new ContractConnectedObject("Batch Product")),
            ]);

        ContractAssert.Equal(4, batchResult.AppliedCount, "Mutation batch should report the number of applied commands.");
        ContractAssert.Equal(4, batchResult.Outcomes.Count, "Mutation batch should preserve ordered outcomes.");
        ContractAssert.True(batchResult.InvalidatedValidation, "Mutation batch should report validation invalidation.");
        ContractAssert.True(batchResult.InvalidatedCalculationReport, "Mutation batch should report calculation report invalidation.");
        AssertMutationOutcome(
            batchResult.Outcomes[0],
            UnitOperationHostObjectMutationKind.SetParameterValue,
            UnitOperationHostActionTargetKind.Parameter,
            UnitOperationParameterCatalog.FlowsheetJson.Name);
        AssertMutationOutcome(
            batchResult.Outcomes[1],
            UnitOperationHostObjectMutationKind.SetParameterValue,
            UnitOperationHostActionTargetKind.Parameter,
            UnitOperationParameterCatalog.PropertyPackageId.Name);
        AssertMutationOutcome(
            batchResult.Outcomes[2],
            UnitOperationHostObjectMutationKind.ConnectPort,
            UnitOperationHostActionTargetKind.Port,
            UnitOperationPortCatalog.Feed.Name);
        AssertMutationOutcome(
            batchResult.Outcomes[3],
            UnitOperationHostObjectMutationKind.ConnectPort,
            UnitOperationHostActionTargetKind.Port,
            UnitOperationPortCatalog.Product.Name);

        ContractAssert.True(context.FlowsheetParameter.IsConfigured, "Mutation batch should configure flowsheet parameter.");
        ContractAssert.True(context.PackageIdParameter.IsConfigured, "Mutation batch should configure package parameter.");
        ContractAssert.True(context.FeedPort.IsConnected, "Mutation batch should connect feed port.");
        ContractAssert.True(context.ProductPort.IsConnected, "Mutation batch should connect product port.");

        var readyConfiguration = context.ReadConfiguration();
        ContractAssert.Equal(UnitOperationHostConfigurationState.Ready, readyConfiguration.State, "Mutation batch should be able to produce a ready configuration state.");

        var batchFailureError = ContractAssert.Throws<CapeBadInvocationOrderException>(
            () => UnitOperationHostObjectMutationDispatcher.DispatchBatch(
                context.UnitOperation,
                [
                    UnitOperationHostObjectMutationCommand.ConnectPort(UnitOperationPortCatalog.Feed.Name, new ContractConnectedObject("Replacement Batch Feed")),
                ]),
            "Mutation batch should preserve command failure semantics.");
        ContractAssert.Contains(batchFailureError.Description, "Disconnect it before replacing", "Mutation batch failures should preserve connect replacement guidance.");
    }

    public static void ActionDefinitionCatalog_StaysAlignedWithIssueKinds(ContractTestContext context)
    {
        var expectedIssueKinds = Enum.GetValues<UnitOperationHostConfigurationIssueKind>();
        ContractAssert.SequenceEqual(
            expectedIssueKinds,
            UnitOperationHostActionDefinitionCatalog.OrderedDefinitions.Select(static definition => definition.IssueKind),
            "Action definition catalog should cover every host configuration issue kind in stable order.");

        foreach (var issueKind in expectedIssueKinds)
        {
            var definition = UnitOperationHostActionDefinitionCatalog.GetByIssueKind(issueKind);
            ContractAssert.Equal(issueKind, definition.IssueKind, "Action definition lookup should preserve issue kind.");
            ContractAssert.True(definition.GroupOrder >= 0, "Action definition should expose a non-negative group order.");
            ContractAssert.False(string.IsNullOrWhiteSpace(definition.GroupTitle), "Action definition should expose a non-empty group title.");
        }

        ContractAssert.Equal(UnitOperationHostActionGroupKind.Lifecycle, UnitOperationHostActionDefinitionCatalog.InitializeRequired.GroupKind, "InitializeRequired action should stay in Lifecycle group.");
        ContractAssert.Equal(UnitOperationHostActionTargetKind.Unit, UnitOperationHostActionDefinitionCatalog.InitializeRequired.TargetKind, "InitializeRequired action should target the unit.");
        ContractAssert.Equal("Lifecycle", UnitOperationHostActionDefinitionCatalog.InitializeRequired.GroupTitle, "InitializeRequired action should preserve Lifecycle title.");

        ContractAssert.Equal(UnitOperationHostActionGroupKind.Parameters, UnitOperationHostActionDefinitionCatalog.RequiredParameterMissing.GroupKind, "RequiredParameterMissing action should stay in Parameters group.");
        ContractAssert.Equal(UnitOperationHostActionTargetKind.Parameter, UnitOperationHostActionDefinitionCatalog.RequiredParameterMissing.TargetKind, "RequiredParameterMissing action should target parameters.");

        ContractAssert.Equal(UnitOperationHostActionGroupKind.Parameters, UnitOperationHostActionDefinitionCatalog.CompanionParameterMismatch.GroupKind, "CompanionParameterMismatch action should stay in Parameters group.");
        ContractAssert.Equal(UnitOperationHostActionTargetKind.Parameter, UnitOperationHostActionDefinitionCatalog.CompanionParameterMismatch.TargetKind, "CompanionParameterMismatch action should target parameters.");

        ContractAssert.Equal(UnitOperationHostActionGroupKind.Ports, UnitOperationHostActionDefinitionCatalog.RequiredPortDisconnected.GroupKind, "RequiredPortDisconnected action should stay in Ports group.");
        ContractAssert.Equal(UnitOperationHostActionTargetKind.Port, UnitOperationHostActionDefinitionCatalog.RequiredPortDisconnected.TargetKind, "RequiredPortDisconnected action should target ports.");

        ContractAssert.Equal(UnitOperationHostActionGroupKind.Terminal, UnitOperationHostActionDefinitionCatalog.Terminated.GroupKind, "Terminated action should stay in Terminal group.");
        ContractAssert.Equal(UnitOperationHostActionTargetKind.Unit, UnitOperationHostActionDefinitionCatalog.Terminated.TargetKind, "Terminated action should target the unit.");

        var missingDefinitionError = ContractAssert.Throws<ArgumentException>(
            () => UnitOperationHostActionDefinitionCatalog.GetByIssueKind((UnitOperationHostConfigurationIssueKind)999),
            "Unknown action definition lookups should be rejected.");
        ContractAssert.Contains(missingDefinitionError.Message, "Unknown unit operation host action definition", "Missing action definition failures should stay explicit.");
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

    public static void ActionPlan_ExposesCanonicalChecklistShape(ContractTestContext context)
    {
        var constructedPlan = context.ReadActionPlan();
        AssertActionPlan(
            constructedPlan,
            "constructed action plan",
            Action(
                UnitOperationHostActionGroupKind.Lifecycle,
                UnitOperationHostActionTargetKind.Unit,
                nameof(RadishFlowCapeOpenUnitOperation.Initialize),
                UnitOperationHostConfigurationIssueKind.InitializeRequired,
                "Initialize must be called",
                context.UnitOperation.ComponentName),
            Action(
                UnitOperationHostActionGroupKind.Parameters,
                UnitOperationHostActionTargetKind.Parameter,
                UnitOperationParameterCatalog.FlowsheetJson.ConfigurationOperationName,
                UnitOperationHostConfigurationIssueKind.RequiredParameterMissing,
                "Required parameter",
                UnitOperationParameterCatalog.FlowsheetJson.Name),
            Action(
                UnitOperationHostActionGroupKind.Parameters,
                UnitOperationHostActionTargetKind.Parameter,
                UnitOperationParameterCatalog.PropertyPackageId.ConfigurationOperationName,
                UnitOperationHostConfigurationIssueKind.RequiredParameterMissing,
                "Required parameter",
                UnitOperationParameterCatalog.PropertyPackageId.Name),
            Action(
                UnitOperationHostActionGroupKind.Ports,
                UnitOperationHostActionTargetKind.Port,
                UnitOperationPortCatalog.Feed.ConnectionOperationName,
                UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected,
                "Required port",
                UnitOperationPortCatalog.Feed.Name),
            Action(
                UnitOperationHostActionGroupKind.Ports,
                UnitOperationHostActionTargetKind.Port,
                UnitOperationPortCatalog.Product.ConnectionOperationName,
                UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected,
                "Required port",
                UnitOperationPortCatalog.Product.Name));

        context.Initialize();
        context.LoadFlowsheet();
        context.LoadPackageFiles();
        context.ConnectRequiredPorts();

        var missingRequiredParameterPlan = context.ReadActionPlan();
        AssertActionPlan(
            missingRequiredParameterPlan,
            "missing required parameter action plan",
            Action(
                UnitOperationHostActionGroupKind.Parameters,
                UnitOperationHostActionTargetKind.Parameter,
                UnitOperationParameterCatalog.PropertyPackageId.ConfigurationOperationName,
                UnitOperationHostConfigurationIssueKind.RequiredParameterMissing,
                "Required parameter",
                UnitOperationParameterCatalog.PropertyPackageId.Name));

        context.SelectPackage();
        var readyPlan = context.ReadActionPlan();
        AssertActionPlan(readyPlan, "ready action plan");
        ContractAssert.False(readyPlan.HasBlockingActions, "Ready action plan should not expose blocking actions.");

        context.ManifestPathParameter.value = context.ManifestPath;
        context.ManifestPathParameter.value = context.ManifestPath;
        context.PayloadPathParameter.value = null;
        var companionMismatchPlan = context.ReadActionPlan();
        AssertActionPlan(
            companionMismatchPlan,
            "companion mismatch action plan",
            Action(
                UnitOperationHostActionGroupKind.Parameters,
                UnitOperationHostActionTargetKind.Parameter,
                UnitOperationParameterCatalog.PropertyPackageManifestPath.ConfigurationOperationName,
                UnitOperationHostConfigurationIssueKind.CompanionParameterMismatch,
                "must be configured together",
                UnitOperationParameterCatalog.PropertyPackageManifestPath.Name,
                UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name));

        context.PayloadPathParameter.value = context.PayloadPath;
        context.DisconnectProductPort();
        var disconnectedRequiredPortPlan = context.ReadActionPlan();
        AssertActionPlan(
            disconnectedRequiredPortPlan,
            "disconnected required port action plan",
            Action(
                UnitOperationHostActionGroupKind.Ports,
                UnitOperationHostActionTargetKind.Port,
                UnitOperationPortCatalog.Product.ConnectionOperationName,
                UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected,
                "Required port",
                UnitOperationPortCatalog.Product.Name));

        context.UnitOperation.Terminate();
        var terminatedPlan = context.ReadActionPlan();
        AssertActionPlan(
            terminatedPlan,
            "terminated action plan",
            Action(
                UnitOperationHostActionGroupKind.Terminal,
                UnitOperationHostActionTargetKind.Unit,
                null,
                UnitOperationHostConfigurationIssueKind.Terminated,
                "Terminate has already been called",
                context.UnitOperation.ComponentName));
        ContractAssert.False(terminatedPlan.ContainsCanonicalOperation(nameof(RadishFlowCapeOpenUnitOperation.Initialize)), "Terminated action plan should not suggest Initialize().");
    }

    public static void ActionMutationBridge_TranslatesCanonicalHostActions(ContractTestContext context)
    {
        var constructedPlan = context.ReadActionPlan();
        var constructedBindings = UnitOperationHostActionMutationBridge.Describe(constructedPlan);
        ContractAssert.Equal(5, constructedBindings.Count, "Constructed action plan should translate every action into a mutation binding.");

        var initializeBinding = constructedBindings[0];
        ContractAssert.Equal(UnitOperationHostActionMutationBindingKind.LifecycleOperation, initializeBinding.Kind, "InitializeRequired action should be classified as lifecycle-only.");
        ContractAssert.False(initializeBinding.CanCreateMutationCommands, "InitializeRequired action should not produce mutation commands.");
        ContractAssert.Equal(0, initializeBinding.CommandCount, "InitializeRequired action should not report mutation commands.");

        var flowsheetBinding = constructedBindings[1];
        ContractAssert.Equal(UnitOperationHostActionMutationBindingKind.ParameterValues, flowsheetBinding.Kind, "Missing flowsheet action should require parameter values.");
        ContractAssert.True(flowsheetBinding.CanCreateMutationCommands, "Missing flowsheet action should support mutation command creation.");
        ContractAssert.SequenceEqual(
            [UnitOperationHostObjectMutationKind.SetParameterValue],
            flowsheetBinding.MutationKinds,
            "Missing flowsheet action should map to SetParameterValue.");

        var flowsheetCommands = UnitOperationHostActionMutationBridge.CreateParameterCommandBatch(
            constructedPlan.Actions[1],
            new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
            {
                [UnitOperationParameterCatalog.FlowsheetJson.Name] = context.FlowsheetJsonText,
            });
        ContractAssert.Equal(1, flowsheetCommands.CommandCount, "Single-parameter action should produce one mutation command.");
        AssertMutationCommand(
            flowsheetCommands.Commands[0],
            UnitOperationHostObjectMutationKind.SetParameterValue,
            UnitOperationParameterCatalog.FlowsheetJson.Name,
            context.FlowsheetJsonText);

        var packageFileAction = new UnitOperationHostActionItem(
            RecommendedOrder: 1,
            GroupKind: UnitOperationHostActionGroupKind.Parameters,
            Target: new UnitOperationHostActionTarget(
                UnitOperationHostActionTargetKind.Parameter,
                [
                    UnitOperationParameterCatalog.PropertyPackageManifestPath.Name,
                    UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name,
                ]),
            Reason: "Optional parameters must be configured together.",
            IsBlocking: true,
            CanonicalOperationName: UnitOperationParameterCatalog.PropertyPackageManifestPath.ConfigurationOperationName,
            IssueKind: UnitOperationHostConfigurationIssueKind.CompanionParameterMismatch);
        var companionBinding = UnitOperationHostActionMutationBridge.Describe(packageFileAction);
        ContractAssert.Equal(UnitOperationHostActionMutationBindingKind.ParameterValues, companionBinding.Kind, "Companion mismatch action should require parameter values.");
        ContractAssert.Equal(2, companionBinding.CommandCount, "Companion mismatch action should produce two parameter commands.");
        ContractAssert.SequenceEqual(
            [UnitOperationHostObjectMutationKind.SetParameterValue, UnitOperationHostObjectMutationKind.SetParameterValue],
            companionBinding.MutationKinds,
            "Companion mismatch action should map both targets to SetParameterValue.");

        var companionCommands = UnitOperationHostActionMutationBridge.CreateParameterCommandBatch(
            packageFileAction,
            new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
            {
                [UnitOperationParameterCatalog.PropertyPackageManifestPath.Name] = context.ManifestPath,
                [UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name] = context.PayloadPath,
            });
        ContractAssert.Equal(2, companionCommands.CommandCount, "Companion mismatch action should create one command per companion target.");
        AssertMutationCommand(
            companionCommands.Commands[0],
            UnitOperationHostObjectMutationKind.SetParameterValue,
            UnitOperationParameterCatalog.PropertyPackageManifestPath.Name,
            context.ManifestPath);
        AssertMutationCommand(
            companionCommands.Commands[1],
            UnitOperationHostObjectMutationKind.SetParameterValue,
            UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name,
            context.PayloadPath);

        var productPortAction = new UnitOperationHostActionItem(
            RecommendedOrder: 1,
            GroupKind: UnitOperationHostActionGroupKind.Ports,
            Target: new UnitOperationHostActionTarget(
                UnitOperationHostActionTargetKind.Port,
                [UnitOperationPortCatalog.Product.Name]),
            Reason: "Required port is not connected.",
            IsBlocking: true,
            CanonicalOperationName: UnitOperationPortCatalog.Product.ConnectionOperationName,
            IssueKind: UnitOperationHostConfigurationIssueKind.RequiredPortDisconnected);
        var portBinding = UnitOperationHostActionMutationBridge.Describe(productPortAction);
        ContractAssert.Equal(UnitOperationHostActionMutationBindingKind.PortConnection, portBinding.Kind, "Disconnected required port action should require a port connection object.");
        ContractAssert.Equal(1, portBinding.CommandCount, "Disconnected required port action should produce one connect command.");
        ContractAssert.SequenceEqual(
            [UnitOperationHostObjectMutationKind.ConnectPort],
            portBinding.MutationKinds,
            "Disconnected required port action should map to ConnectPort.");

        var portObject = new ContractConnectedObject("Bridge Product");
        var portCommands = UnitOperationHostActionMutationBridge.CreatePortConnectionCommandBatch(productPortAction, portObject);
        ContractAssert.Equal(1, portCommands.CommandCount, "Port connection action should create one connect command.");
        AssertMutationCommand(
            portCommands.Commands[0],
            UnitOperationHostObjectMutationKind.ConnectPort,
            UnitOperationPortCatalog.Product.Name,
            portObject);

        context.UnitOperation.Terminate();
        var terminatedBinding = UnitOperationHostActionMutationBridge.Describe(context.ReadActionPlan().Actions[0]);
        ContractAssert.Equal(UnitOperationHostActionMutationBindingKind.Unsupported, terminatedBinding.Kind, "Terminated action should remain explicitly unsupported for mutation translation.");
        ContractAssert.False(terminatedBinding.CanCreateMutationCommands, "Terminated action should not create mutation commands.");

        var wrongBindingError = ContractAssert.Throws<InvalidOperationException>(
            () => UnitOperationHostActionMutationBridge.CreatePortConnectionCommandBatch(constructedPlan.Actions[1], new ContractConnectedObject("Wrong Target")),
            "Parameter action should reject port-connection translation.");
        ContractAssert.Contains(wrongBindingError.Message, "does not accept port-connection mutation translation", "Wrong bridge usage should stay explicit.");

        var missingValueError = ContractAssert.Throws<ArgumentException>(
            () => UnitOperationHostActionMutationBridge.CreateParameterCommandBatch(
                packageFileAction,
                new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
                {
                    [UnitOperationParameterCatalog.PropertyPackageManifestPath.Name] = context.ManifestPath,
                }),
            "Companion action should reject incomplete parameter payloads.");
        ContractAssert.Contains(missingValueError.Message, "Missing parameter value", "Incomplete parameter payload errors should stay explicit.");
    }

    public static void ActionExecutionRequestPlanner_PlansRequestsFromHostInputs(ContractTestContext context)
    {
        var emptyConstructedPlan = UnitOperationHostActionExecutionRequestPlanner.Plan(context.ReadActionPlan());
        ContractAssert.Equal(5, emptyConstructedPlan.EntryCount, "Constructed request plan should include every action.");
        ContractAssert.Equal(1, emptyConstructedPlan.RequestCount, "Constructed request plan with no inputs should only produce lifecycle request.");
        ContractAssert.True(emptyConstructedPlan.HasLifecycleOperations, "Constructed request plan should surface lifecycle-only action.");
        ContractAssert.True(emptyConstructedPlan.HasMissingInputs, "Constructed request plan should report missing parameter and port inputs.");
        ContractAssert.False(emptyConstructedPlan.HasUnsupportedActions, "Constructed request plan should not mark active actions as unsupported.");
        ContractAssert.Equal(
            UnitOperationHostActionExecutionRequestPlanningDisposition.LifecycleOperationRequired,
            emptyConstructedPlan.Entries[0].Disposition,
            "Initialize action should be planned as lifecycle-only.");
        ContractAssert.NotNull(emptyConstructedPlan.Entries[0].Request, "Lifecycle action should still produce an execution request.");
        ContractAssert.Equal(
            UnitOperationHostActionExecutionRequestPlanningDisposition.MissingInputs,
            emptyConstructedPlan.Entries[1].Disposition,
            "Missing parameter action should wait for host input.");
        ContractAssert.SequenceEqual(
            [UnitOperationParameterCatalog.FlowsheetJson.Name],
            emptyConstructedPlan.Entries[1].MissingInputNames,
            "Missing parameter entry should name the required parameter input.");

        context.Initialize();
        var initializedPlan = context.ReadActionPlan();
        var feedObject = new ContractConnectedObject("Planned Feed");
        var plannedInputs = new UnitOperationHostActionExecutionInputSet(
            parameterValues: new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
            {
                [UnitOperationParameterCatalog.FlowsheetJson.Name] = context.FlowsheetJsonText,
                [UnitOperationParameterCatalog.PropertyPackageId.Name] = context.PackageId,
            },
            portObjects: new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase)
            {
                [UnitOperationPortCatalog.Feed.Name] = feedObject,
            });
        var partialPlan = UnitOperationHostActionExecutionRequestPlanner.Plan(initializedPlan, plannedInputs);
        ContractAssert.Equal(4, partialPlan.EntryCount, "Initialized request plan should include all blocking configuration actions.");
        ContractAssert.Equal(3, partialPlan.RequestCount, "Partial host inputs should produce ready requests and skip the missing product port.");
        ContractAssert.Equal(1, partialPlan.MissingInputCount, "Partial host inputs should report one missing port object.");
        ContractAssert.SequenceEqual(
            [UnitOperationPortCatalog.Product.Name],
            partialPlan.Entries[3].MissingInputNames,
            "Partial request plan should report the missing product port object.");
        ContractAssert.Equal(
            UnitOperationHostActionExecutionRequestPlanningDisposition.RequestReady,
            partialPlan.Entries[0].Disposition,
            "Flowsheet parameter action should be ready when its value is present.");
        ContractAssert.Equal(
            UnitOperationHostActionExecutionRequestPlanningDisposition.RequestReady,
            partialPlan.Entries[2].Disposition,
            "Feed port action should be ready when its object is present.");
        ContractAssert.SameReference(
            feedObject,
            partialPlan.Entries[2].Request?.PortObject,
            "Port request should preserve the supplied host object instance.");

        var productObject = new ContractConnectedObject("Planned Product");
        var completePlan = UnitOperationHostActionExecutionRequestPlanner.Plan(
            initializedPlan,
            new UnitOperationHostActionExecutionInputSet(
                parameterValues: new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
                {
                    [UnitOperationParameterCatalog.FlowsheetJson.Name] = context.FlowsheetJsonText,
                    [UnitOperationParameterCatalog.PropertyPackageId.Name] = context.PackageId,
                },
                portObjects: new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase)
                {
                    [UnitOperationPortCatalog.Feed.Name] = feedObject,
                    [UnitOperationPortCatalog.Product.Name] = productObject,
                }));
        ContractAssert.Equal(4, completePlan.RequestCount, "Complete host inputs should produce one request per initialized action.");
        ContractAssert.False(completePlan.HasMissingInputs, "Complete host inputs should not report missing inputs.");
        ContractAssert.SequenceEqual(
            [
                UnitOperationHostActionExecutionRequestPlanningDisposition.RequestReady,
                UnitOperationHostActionExecutionRequestPlanningDisposition.RequestReady,
                UnitOperationHostActionExecutionRequestPlanningDisposition.RequestReady,
                UnitOperationHostActionExecutionRequestPlanningDisposition.RequestReady,
            ],
            completePlan.Entries.Select(static entry => entry.Disposition),
            "Complete request plan should mark every initialized action as request-ready.");

        var batchResult = UnitOperationHostActionExecutionDispatcher.ApplyActionBatch(
            context.UnitOperation,
            completePlan.Requests);
        ContractAssert.Equal(4, batchResult.AppliedActionCount, "Request plan should feed directly into action execution batch.");
        ContractAssert.Equal(4, batchResult.AppliedMutationCount, "Request plan execution should apply all required object mutations.");
        ContractAssert.Equal(UnitOperationHostConfigurationState.Ready, context.ReadConfiguration().State, "Planned requests should drive the unit into ready state.");

        context.ManifestPathParameter.value = context.ManifestPath;
        context.PayloadPathParameter.value = null;
        var companionPlan = UnitOperationHostActionExecutionRequestPlanner.Plan(
            context.ReadActionPlan(),
            new UnitOperationHostActionExecutionInputSet(
                parameterValues: new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
                {
                    [UnitOperationParameterCatalog.PropertyPackageManifestPath.Name] = context.ManifestPath,
                }));
        ContractAssert.Equal(1, companionPlan.EntryCount, "Companion mismatch should expose one action.");
        ContractAssert.True(companionPlan.HasMissingInputs, "Companion mismatch plan should reject incomplete companion inputs.");
        ContractAssert.SequenceEqual(
            [UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name],
            companionPlan.Entries[0].MissingInputNames,
            "Companion mismatch plan should report the missing companion parameter.");

        var completeCompanionPlan = UnitOperationHostActionExecutionRequestPlanner.Plan(
            context.ReadActionPlan(),
            new UnitOperationHostActionExecutionInputSet(
                parameterValues: new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
                {
                    [UnitOperationParameterCatalog.PropertyPackageManifestPath.Name] = context.ManifestPath,
                    [UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name] = context.PayloadPath,
                }));
        ContractAssert.Equal(1, completeCompanionPlan.RequestCount, "Complete companion inputs should produce one request.");
        ContractAssert.Equal(
            2,
            completeCompanionPlan.Requests[0].ParameterValues?.Count ?? 0,
            "Companion request should carry both parameter values.");

        context.UnitOperation.Terminate();
        var terminatedPlan = UnitOperationHostActionExecutionRequestPlanner.Plan(context.ReadActionPlan());
        ContractAssert.True(terminatedPlan.HasUnsupportedActions, "Terminated request plan should surface unsupported terminal action.");
        ContractAssert.Equal(0, terminatedPlan.RequestCount, "Unsupported terminal action should not produce an executable request.");
        ContractAssert.Equal(
            UnitOperationHostActionExecutionRequestPlanningDisposition.Unsupported,
            terminatedPlan.Entries[0].Disposition,
            "Terminated action should remain unsupported for request planning.");
    }

    public static void ActionExecutionDispatcher_AppliesCanonicalHostActions(ContractTestContext context)
    {
        var constructedPlan = context.ReadActionPlan();
        var initializeOutcome = UnitOperationHostActionExecutionDispatcher.ApplyAction(
            context.UnitOperation,
            UnitOperationHostActionExecutionRequest.ForAction(constructedPlan.Actions[0]));
        ContractAssert.Equal(UnitOperationHostActionExecutionDisposition.LifecycleOperationRequired, initializeOutcome.Disposition, "Initialize action should remain a lifecycle-only execution outcome.");
        ContractAssert.Equal(nameof(RadishFlowCapeOpenUnitOperation.Initialize), initializeOutcome.LifecycleOperationName, "Initialize action should surface canonical lifecycle operation name.");
        ContractAssert.False(initializeOutcome.AppliedMutations, "Initialize action should not apply object mutations.");
        ContractAssert.Equal(0, initializeOutcome.ExecutedCommands.Count, "Initialize action should not execute mutation commands.");

        context.Initialize();

        var parameterOutcome = UnitOperationHostActionExecutionDispatcher.ApplyAction(
            context.UnitOperation,
            UnitOperationHostActionExecutionRequest.ForParameterValues(
                context.ReadActionPlan().Actions[0],
                new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
                {
                    [UnitOperationParameterCatalog.FlowsheetJson.Name] = context.FlowsheetJsonText,
                }));
        ContractAssert.Equal(UnitOperationHostActionExecutionDisposition.MutationApplied, parameterOutcome.Disposition, "Required parameter action should apply object mutations.");
        ContractAssert.True(parameterOutcome.AppliedMutations, "Required parameter action should report applied mutations.");
        ContractAssert.Equal(1, parameterOutcome.AppliedMutationCount, "Single required parameter action should apply one mutation.");
        ContractAssert.True(parameterOutcome.InvalidatedValidation, "Required parameter action should invalidate validation state.");
        ContractAssert.True(context.FlowsheetParameter.IsConfigured, "Required parameter action should configure the flowsheet parameter.");
        AssertMutationCommand(
            parameterOutcome.ExecutedCommands[0],
            UnitOperationHostObjectMutationKind.SetParameterValue,
            UnitOperationParameterCatalog.FlowsheetJson.Name,
            context.FlowsheetJsonText);
        AssertMutationOutcome(
            parameterOutcome.MutationOutcomes[0],
            UnitOperationHostObjectMutationKind.SetParameterValue,
            UnitOperationHostActionTargetKind.Parameter,
            UnitOperationParameterCatalog.FlowsheetJson.Name);

        var currentPlan = context.ReadActionPlan();
        var batchResult = UnitOperationHostActionExecutionDispatcher.ApplyActionBatch(
            context.UnitOperation,
            [
                UnitOperationHostActionExecutionRequest.ForParameterValues(
                    currentPlan.Actions[0],
                    new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
                    {
                        [UnitOperationParameterCatalog.PropertyPackageId.Name] = context.PackageId,
                    }),
                UnitOperationHostActionExecutionRequest.ForPortConnection(
                    currentPlan.Actions[1],
                    new ContractConnectedObject("Execution Feed")),
                UnitOperationHostActionExecutionRequest.ForPortConnection(
                    currentPlan.Actions[2],
                    new ContractConnectedObject("Execution Product")),
            ]);
        ContractAssert.Equal(3, batchResult.AppliedActionCount, "Action batch should report action count.");
        ContractAssert.Equal(3, batchResult.AppliedMutationCount, "Action batch should sum applied mutation commands.");
        ContractAssert.False(batchResult.HasLifecycleOperations, "Pure mutation action batch should not report lifecycle operations.");
        ContractAssert.False(batchResult.HasUnsupportedActions, "Pure mutation action batch should not report unsupported actions.");
        ContractAssert.True(batchResult.InvalidatedValidation, "Action batch should report validation invalidation.");
        ContractAssert.True(batchResult.InvalidatedCalculationReport, "Action batch should report calculation report invalidation.");
        ContractAssert.True(context.PackageIdParameter.IsConfigured, "Action batch should configure package id parameter.");
        ContractAssert.True(context.FeedPort.IsConnected, "Action batch should connect feed port.");
        ContractAssert.True(context.ProductPort.IsConnected, "Action batch should connect product port.");
        ContractAssert.Equal(UnitOperationHostConfigurationState.Ready, context.ReadConfiguration().State, "Action batch should be able to drive configuration into ready state.");

        context.UnitOperation.Terminate();

        var terminatedOutcome = UnitOperationHostActionExecutionDispatcher.ApplyAction(
            context.UnitOperation,
            UnitOperationHostActionExecutionRequest.ForAction(context.ReadActionPlan().Actions[0]));
        ContractAssert.Equal(UnitOperationHostActionExecutionDisposition.Unsupported, terminatedOutcome.Disposition, "Terminated action should remain unsupported for execution.");
        ContractAssert.False(terminatedOutcome.AppliedMutations, "Terminated action should not apply mutations.");

        var missingPayloadError = ContractAssert.Throws<InvalidOperationException>(
            () => UnitOperationHostActionExecutionDispatcher.ApplyAction(
                new RadishFlowCapeOpenUnitOperation(),
                UnitOperationHostActionExecutionRequest.ForAction(constructedPlan.Actions[1])),
            "Required parameter execution should reject missing parameter values.");
        ContractAssert.Contains(missingPayloadError.Message, "requires parameter values", "Missing parameter values should stay explicit.");
    }

    public static void ActionExecutionOrchestrator_RefreshesHostViews(ContractTestContext context)
    {
        var constructedOrchestration = UnitOperationHostActionExecutionOrchestrator.ExecutePlannedActions(context.UnitOperation);
        ContractAssert.Equal(5, constructedOrchestration.PlannedActionCount, "Constructed orchestration should include all blocking actions.");
        ContractAssert.Equal(1, constructedOrchestration.ReadyRequestCount, "Constructed orchestration should only auto-carry the lifecycle request.");
        ContractAssert.True(constructedOrchestration.HasMissingInputs, "Constructed orchestration should report missing inputs.");
        ContractAssert.True(constructedOrchestration.HasLifecycleOperations, "Constructed orchestration should preserve lifecycle-only action visibility.");
        ContractAssert.False(constructedOrchestration.HasUnsupportedActions, "Constructed orchestration should not report unsupported actions before terminate.");
        ContractAssert.Equal(UnitOperationHostConfigurationState.Constructed, constructedOrchestration.Configuration.State, "Constructed orchestration should leave configuration unchanged.");
        ContractAssert.Equal(UnitOperationHostSessionState.Constructed, constructedOrchestration.Session.State, "Constructed orchestration should leave session unchanged.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.LifecycleOperation, constructedOrchestration.FollowUp.Kind, "Constructed orchestration should recommend lifecycle follow-up.");
        ContractAssert.False(constructedOrchestration.FollowUp.CanValidate, "Constructed lifecycle follow-up should not allow validate.");
        ContractAssert.False(constructedOrchestration.FollowUp.CanCalculate, "Constructed lifecycle follow-up should not allow calculate.");
        ContractAssert.SequenceEqual([nameof(RadishFlowCapeOpenUnitOperation.Initialize)], constructedOrchestration.FollowUp.RecommendedOperations, "Constructed lifecycle follow-up should recommend Initialize().");
        ContractAssert.Equal(UnitOperationHostActionExecutionDisposition.LifecycleOperationRequired, constructedOrchestration.Execution.Outcomes[0].Disposition, "Constructed orchestration should keep initialize as lifecycle-only outcome.");

        context.Initialize();

        var feedObject = new ContractConnectedObject("Orchestration Feed");
        var productObject = new ContractConnectedObject("Orchestration Product");
        var configuredOrchestration = UnitOperationHostActionExecutionOrchestrator.ExecutePlannedActions(
            context.UnitOperation,
            new UnitOperationHostActionExecutionInputSet(
                parameterValues: new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
                {
                    [UnitOperationParameterCatalog.FlowsheetJson.Name] = context.FlowsheetJsonText,
                    [UnitOperationParameterCatalog.PropertyPackageId.Name] = context.PackageId,
                },
                portObjects: new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase)
                {
                    [UnitOperationPortCatalog.Feed.Name] = feedObject,
                    [UnitOperationPortCatalog.Product.Name] = productObject,
                }));
        ContractAssert.Equal(4, configuredOrchestration.PlannedActionCount, "Initialized orchestration should include all remaining blocking actions.");
        ContractAssert.Equal(4, configuredOrchestration.ReadyRequestCount, "Initialized orchestration should produce one ready request per action when inputs are complete.");
        ContractAssert.False(configuredOrchestration.HasMissingInputs, "Complete initialized orchestration should not report missing inputs.");
        ContractAssert.True(configuredOrchestration.AppliedMutations, "Initialized orchestration should apply mutations.");
        ContractAssert.True(configuredOrchestration.RequiresValidationRefresh, "Initialized orchestration should report validation refresh after applying mutations.");
        ContractAssert.True(configuredOrchestration.RequiresCalculationRefresh, "Initialized orchestration should report calculation refresh after applying mutations.");
        ContractAssert.Equal(UnitOperationHostConfigurationState.Ready, configuredOrchestration.Configuration.State, "Initialized orchestration should refresh configuration into ready state.");
        ContractAssert.Equal(0, configuredOrchestration.ActionPlan.ActionCount, "Initialized orchestration should refresh action plan to empty once ready.");
        ContractAssert.Equal(UnitOperationHostSessionState.Ready, configuredOrchestration.Session.State, "Initialized orchestration should refresh session into ready state.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.Validate, configuredOrchestration.FollowUp.Kind, "Mutation-applied orchestration should recommend validate before calculate.");
        ContractAssert.True(configuredOrchestration.FollowUp.CanValidate, "Validate follow-up should allow validation.");
        ContractAssert.False(configuredOrchestration.FollowUp.CanCalculate, "Validate follow-up should not allow calculate yet.");

        context.ManifestPathParameter.value = context.ManifestPath;
        context.PayloadPathParameter.value = null;
        var companionOrchestration = UnitOperationHostActionExecutionOrchestrator.ExecutePlannedActions(
            context.UnitOperation,
            new UnitOperationHostActionExecutionInputSet(
                parameterValues: new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
                {
                    [UnitOperationParameterCatalog.PropertyPackageManifestPath.Name] = context.ManifestPath,
                }));
        ContractAssert.Equal(1, companionOrchestration.PlannedActionCount, "Companion mismatch orchestration should focus on one blocking action.");
        ContractAssert.Equal(0, companionOrchestration.ReadyRequestCount, "Incomplete companion inputs should not produce ready requests.");
        ContractAssert.True(companionOrchestration.HasMissingInputs, "Incomplete companion inputs should surface missing inputs.");
        ContractAssert.Equal(UnitOperationHostConfigurationState.Incomplete, companionOrchestration.Configuration.State, "Companion mismatch orchestration should preserve incomplete configuration state.");
        ContractAssert.Equal(UnitOperationHostSessionState.Incomplete, companionOrchestration.Session.State, "Companion mismatch orchestration should refresh session to incomplete when configuration is broken before any current results exist.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.ProvideInputs, companionOrchestration.FollowUp.Kind, "Companion mismatch orchestration should recommend providing inputs.");
        ContractAssert.SequenceEqual([UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name], companionOrchestration.FollowUp.MissingInputNames, "Companion mismatch follow-up should report the missing payload input.");
        ContractAssert.False(companionOrchestration.FollowUp.CanValidate, "Provide-inputs follow-up should not allow validate.");
        ContractAssert.False(companionOrchestration.FollowUp.CanCalculate, "Provide-inputs follow-up should not allow calculate.");

        context.UnitOperation.Terminate();
        var terminatedOrchestration = UnitOperationHostActionExecutionOrchestrator.ExecutePlannedActions(context.UnitOperation);
        ContractAssert.True(terminatedOrchestration.HasUnsupportedActions, "Terminated orchestration should surface unsupported terminal action.");
        ContractAssert.Equal(0, terminatedOrchestration.ReadyRequestCount, "Terminated orchestration should not produce executable requests.");
        ContractAssert.Equal(UnitOperationHostConfigurationState.Terminated, terminatedOrchestration.Configuration.State, "Terminated orchestration should refresh configuration to terminated.");
        ContractAssert.Equal(UnitOperationHostSessionState.Terminated, terminatedOrchestration.Session.State, "Terminated orchestration should refresh session to terminated.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.Terminated, terminatedOrchestration.FollowUp.Kind, "Terminated orchestration should report terminated follow-up.");
        ContractAssert.False(terminatedOrchestration.FollowUp.CanValidate, "Terminated follow-up should not allow validate.");
        ContractAssert.False(terminatedOrchestration.FollowUp.CanCalculate, "Terminated follow-up should not allow calculate.");
    }

    public static void ValidationRound_RefreshesHostViews(ContractTestContext context)
    {
        var constructedValidation = context.ValidateRound();
        ContractAssert.False(constructedValidation.IsValid, "Constructed validation round should stay invalid.");
        ContractAssert.Equal(CapeValidationStatus.Invalid, constructedValidation.ValidationStatus, "Constructed validation round should preserve invalid ValStatus.");
        ContractAssert.Equal(UnitOperationHostSessionState.Constructed, constructedValidation.Session.State, "Constructed validation round should expose constructed session state.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.LifecycleOperation, constructedValidation.FollowUp.Kind, "Constructed validation round should recommend Initialize().");
        ContractAssert.SequenceEqual([nameof(RadishFlowCapeOpenUnitOperation.Initialize)], constructedValidation.FollowUp.RecommendedOperations, "Constructed validation round should preserve Initialize() recommendation.");

        context.ConfigureMinimumValidInputs();

        var readyValidation = context.ValidateRound();
        ContractAssert.True(readyValidation.IsValid, "Ready validation round should succeed.");
        ContractAssert.Equal(CapeValidationStatus.Valid, readyValidation.ValidationStatus, "Ready validation round should preserve valid ValStatus.");
        ContractAssert.Equal(UnitOperationHostSessionState.Ready, readyValidation.Session.State, "Ready validation round should expose ready session state.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.Calculate, readyValidation.FollowUp.Kind, "Ready validation round should recommend Calculate().");
        ContractAssert.True(readyValidation.FollowUp.CanValidate, "Ready validation round should still allow Validate().");
        ContractAssert.True(readyValidation.FollowUp.CanCalculate, "Ready validation round should allow Calculate().");

        context.DisconnectProductPort();

        var staleValidation = context.ValidateRound();
        ContractAssert.False(staleValidation.IsValid, "Broken required-port validation round should fail.");
        ContractAssert.Equal(UnitOperationHostSessionState.Incomplete, staleValidation.Session.State, "Broken required-port validation round should expose incomplete session state before any current results exist.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.ProvideInputs, staleValidation.FollowUp.Kind, "Broken required-port validation round should recommend providing inputs.");
        ContractAssert.True(staleValidation.FollowUp.MissingInputNames.Contains(UnitOperationPortCatalog.Product.Name), "Broken required-port validation round should surface the missing product port input.");

        context.UnitOperation.Terminate();

        var terminatedValidation = context.ValidateRound();
        ContractAssert.False(terminatedValidation.IsValid, "Terminated validation round should stay invalid.");
        ContractAssert.Equal(UnitOperationHostSessionState.Terminated, terminatedValidation.Session.State, "Terminated validation round should expose terminated session state.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.Terminated, terminatedValidation.FollowUp.Kind, "Terminated validation round should report terminated follow-up.");
        ContractAssert.False(terminatedValidation.FollowUp.CanCalculate, "Terminated validation round should not allow Calculate().");
    }

    public static void CalculationRound_RefreshesHostViews(ContractTestContext context)
    {
        var constructedCalculation = context.CalculateRound();
        ContractAssert.False(constructedCalculation.Succeeded, "Constructed calculation round should fail before Initialize().");
        ContractAssert.NotNull(constructedCalculation.Failure, "Constructed calculation round should preserve the invocation-order failure.");
        ContractAssert.True(constructedCalculation.Failure is CapeBadInvocationOrderException, "Constructed calculation round should preserve CapeBadInvocationOrderException.");
        ContractAssert.Equal(UnitOperationHostSessionState.Constructed, constructedCalculation.Session.State, "Constructed calculation round should expose constructed session state.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.LifecycleOperation, constructedCalculation.FollowUp.Kind, "Constructed calculation round should recommend Initialize().");
        ContractAssert.Equal(UnitOperationCalculationReportState.None, constructedCalculation.Report.State, "Constructed calculation round should preserve empty report state before any calculation result exists.");

        context.ConfigureMinimumValidInputs();

        var successCalculation = context.CalculateRound();
        ContractAssert.True(successCalculation.Succeeded, "Ready calculation round should succeed.");
        ContractAssert.Equal(UnitOperationHostSessionState.Available, successCalculation.Session.State, "Successful calculation round should expose available session state.");
        ContractAssert.Equal(UnitOperationHostExecutionState.Available, successCalculation.Execution.State, "Successful calculation round should expose available execution snapshot.");
        ContractAssert.Equal(UnitOperationCalculationReportState.Success, successCalculation.Report.State, "Successful calculation round should expose success report state.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.CurrentResults, successCalculation.FollowUp.Kind, "Successful calculation round should report current results as the next host state.");
        ContractAssert.True(successCalculation.FollowUp.CanCalculate, "Successful calculation round should still allow Calculate().");

        context.UnitOperation.SelectPropertyPackage("missing-package-for-calculation-round");
        var nativeFailureCalculation = context.CalculateRound();
        ContractAssert.False(nativeFailureCalculation.Succeeded, "Native-failure calculation round should fail.");
        ContractAssert.NotNull(nativeFailureCalculation.Failure, "Native-failure calculation round should preserve the native failure.");
        ContractAssert.True(nativeFailureCalculation.Failure is CapeInvalidArgumentException, "Native-failure calculation round should preserve CapeInvalidArgumentException.");
        ContractAssert.Equal(UnitOperationHostSessionState.Failure, nativeFailureCalculation.Session.State, "Native-failure calculation round should expose failure session state.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.Calculate, nativeFailureCalculation.FollowUp.Kind, "Native-failure calculation round should recommend calculate retry after recovery.");
        ContractAssert.True(nativeFailureCalculation.FollowUp.CanCalculate, "Native-failure calculation round should still allow Calculate() after recovery.");

        context.UnitOperation.Terminate();
        var terminatedCalculation = context.CalculateRound();
        ContractAssert.False(terminatedCalculation.Succeeded, "Terminated calculation round should fail.");
        ContractAssert.Equal(UnitOperationHostSessionState.Terminated, terminatedCalculation.Session.State, "Terminated calculation round should expose terminated session state.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.Terminated, terminatedCalculation.FollowUp.Kind, "Terminated calculation round should report terminated follow-up.");
        ContractAssert.False(terminatedCalculation.FollowUp.CanValidate, "Terminated calculation round should not allow Validate().");
    }

    public static void HostRound_OrchestratesCanonicalHostPath(ContractTestContext context)
    {
        var constructedRound = context.ExecuteRound();
        ContractAssert.True(constructedRound.ExecutedActions, "Default constructed round should evaluate current host actions.");
        ContractAssert.False(constructedRound.ExecutedValidation, "Constructed round should stop before Validate() when lifecycle is still required.");
        ContractAssert.False(constructedRound.ExecutedCalculation, "Constructed round should stop before Calculate() when lifecycle is still required.");
        ContractAssert.Equal(UnitOperationHostRoundStopKind.LifecycleOperationRequired, constructedRound.StopKind, "Constructed round should classify lifecycle gating explicitly.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.LifecycleOperation, constructedRound.FollowUp.Kind, "Constructed round should recommend Initialize().");
        ContractAssert.SequenceEqual([nameof(RadishFlowCapeOpenUnitOperation.Initialize)], constructedRound.FollowUp.RecommendedOperations, "Constructed round should preserve Initialize() recommendation.");

        context.Initialize();
        var configuredRound = context.ExecuteRound(
            new UnitOperationHostRoundRequest(
                actionInputSet: context.CreateMinimumConfigurationInputSet(
                    includePackageId: true,
                    includePackageFiles: false),
                executeReadyActions: true,
                runValidation: true,
                runCalculation: true,
                supplementalMutationCommands: context.CreateOptionalPackageFileMutationCommands()));
        ContractAssert.True(configuredRound.ExecutedActions, "Configured round should execute blocking host actions.");
        ContractAssert.True(configuredRound.ExecutedSupplementalMutations, "Configured round should apply supplemental package-file mutations.");
        ContractAssert.True(configuredRound.ExecutedValidation, "Configured round should validate current configuration.");
        ContractAssert.True(configuredRound.ExecutedCalculation, "Configured round should calculate after successful validation.");
        ContractAssert.Equal(UnitOperationHostRoundStopKind.Completed, configuredRound.StopKind, "Configured round should complete the canonical host path.");
        ContractAssert.Equal(UnitOperationHostSessionState.Available, configuredRound.Session.State, "Configured round should reach available session state.");
        ContractAssert.Equal(UnitOperationCalculationReportState.Success, configuredRound.Report.State, "Configured round should expose success report state.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.CurrentResults, configuredRound.FollowUp.Kind, "Configured round should end at current results.");
        ContractAssert.True(configuredRound.FollowUp.CanCalculate, "Current-results round should still allow Calculate().");
        ContractAssert.True(configuredRound.ActionExecution!.AppliedMutations, "Configured round should preserve action-execution mutation summary.");
        ContractAssert.Equal(2, configuredRound.SupplementalMutations!.Batch.AppliedCount, "Configured round should apply both package-file supplemental mutations.");
        ContractAssert.True(configuredRound.Configuration.GetParameter(UnitOperationParameterCatalog.PropertyPackageManifestPath.Name).IsConfigured, "Configured round should materialize manifest path through supplemental mutations.");
        ContractAssert.True(configuredRound.Configuration.GetParameter(UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name).IsConfigured, "Configured round should materialize payload path through supplemental mutations.");

        var successRound = context.ExecuteRound(
            new UnitOperationHostRoundRequest(
                executeReadyActions: false,
                runValidation: true,
                runCalculation: true));
        ContractAssert.False(successRound.ExecutedActions, "Ready round should be able to skip host actions when configuration is already present.");
        ContractAssert.False(successRound.ExecutedSupplementalMutations, "Ready round should not require supplemental mutations once optional package files are already configured.");
        ContractAssert.True(successRound.ExecutedValidation, "Ready round should validate current configuration.");
        ContractAssert.True(successRound.ExecutedCalculation, "Ready round should calculate after successful validation.");
        ContractAssert.Equal(UnitOperationHostRoundStopKind.Completed, successRound.StopKind, "Ready round should complete the canonical validate/calculate path.");
        ContractAssert.Equal(UnitOperationHostSessionState.Available, successRound.Session.State, "Ready round should reach available session state.");
        ContractAssert.Equal(UnitOperationCalculationReportState.Success, successRound.Report.State, "Ready round should expose success report state.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.CurrentResults, successRound.FollowUp.Kind, "Ready round should end at current results.");
        ContractAssert.True(successRound.FollowUp.CanCalculate, "Current-results round should still allow Calculate().");

        context.PayloadPathParameter.value = null;
        var missingCompanionRound = context.ExecuteRound(
            new UnitOperationHostRoundRequest(
                actionInputSet: new UnitOperationHostActionExecutionInputSet(
                    parameterValues: new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
                    {
                        [UnitOperationParameterCatalog.PropertyPackageManifestPath.Name] = context.ManifestPath,
                    }),
                executeReadyActions: true,
                runValidation: true,
                runCalculation: true));
        ContractAssert.True(missingCompanionRound.ExecutedActions, "Companion-input round should still evaluate current host actions.");
        ContractAssert.False(missingCompanionRound.ExecutedValidation, "Companion-input round should stop before Validate() when companion values are incomplete.");
        ContractAssert.False(missingCompanionRound.ExecutedCalculation, "Companion-input round should stop before Calculate() when companion values are incomplete.");
        ContractAssert.Equal(UnitOperationHostRoundStopKind.MissingInputs, missingCompanionRound.StopKind, "Companion-input round should classify missing inputs explicitly.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.ProvideInputs, missingCompanionRound.FollowUp.Kind, "Companion-input round should recommend additional inputs.");
        ContractAssert.True(missingCompanionRound.FollowUp.MissingInputNames.Contains(UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name), "Companion-input round should surface the missing payload path.");
        context.PayloadPathParameter.value = context.PayloadPath;

        context.UnitOperation.SelectPropertyPackage("missing-package-for-host-round");
        var nativeFailureRound = context.ExecuteRound(
            new UnitOperationHostRoundRequest(
                executeReadyActions: false,
                runValidation: false,
                runCalculation: true,
                requireSuccessfulValidationForCalculation: false));
        ContractAssert.False(nativeFailureRound.Completed, "Native-failure round should not report completion.");
        ContractAssert.True(nativeFailureRound.ExecutedCalculation, "Native-failure round should still execute Calculate().");
        ContractAssert.Equal(UnitOperationHostRoundStopKind.CalculationFailed, nativeFailureRound.StopKind, "Native-failure round should classify calculate failure explicitly.");
        ContractAssert.Equal(UnitOperationHostSessionState.Failure, nativeFailureRound.Session.State, "Native-failure round should expose failure session state.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.Calculate, nativeFailureRound.FollowUp.Kind, "Native-failure round should allow calculate retry after recovery.");

        context.UnitOperation.Terminate();
        var terminatedRound = context.ExecuteRound(
            new UnitOperationHostRoundRequest(
                executeReadyActions: false,
                runValidation: true,
                runCalculation: true));
        ContractAssert.False(terminatedRound.Completed, "Terminated round should not report completion.");
        ContractAssert.True(terminatedRound.ExecutedValidation, "Terminated round should still expose validation outcome.");
        ContractAssert.False(terminatedRound.ExecutedCalculation, "Terminated round should stop before Calculate().");
        ContractAssert.Equal(UnitOperationHostRoundStopKind.Terminated, terminatedRound.StopKind, "Terminated round should classify terminal state explicitly.");
        ContractAssert.Equal(UnitOperationHostSessionState.Terminated, terminatedRound.Session.State, "Terminated round should preserve terminated session state.");
        ContractAssert.Equal(UnitOperationHostFollowUpKind.Terminated, terminatedRound.FollowUp.Kind, "Terminated round should preserve terminated follow-up.");
    }

    public static void PortMaterialSnapshot_ExposesBoundaryStreamsAndLifecycleState(ContractTestContext context)
    {
        var constructedSnapshot = context.ReadPortMaterial();
        ContractAssert.Equal(UnitOperationHostPortMaterialState.None, constructedSnapshot.State, "Constructed port/material snapshot should start in the empty state.");
        ContractAssert.Equal(2, constructedSnapshot.PortCount, "Constructed port/material snapshot should expose host ports in frozen catalog order.");
        AssertPortMaterialEntry(
            constructedSnapshot.GetPort(UnitOperationPortCatalog.Feed.Name),
            UnitOperationPortCatalog.Feed,
            UnitOperationHostPortMaterialState.None,
            isConnected: false,
            connectedTargetName: null,
            expectedBoundStreamIds: [],
            expectedMaterialStreamIds: []);
        AssertPortMaterialEntry(
            constructedSnapshot.GetPort(UnitOperationPortCatalog.Product.Name),
            UnitOperationPortCatalog.Product,
            UnitOperationHostPortMaterialState.None,
            isConnected: false,
            connectedTargetName: null,
            expectedBoundStreamIds: [],
            expectedMaterialStreamIds: []);

        context.ConfigureMinimumValidInputs();

        var readySnapshot = context.ReadPortMaterial();
        ContractAssert.Equal(UnitOperationHostPortMaterialState.None, readySnapshot.State, "Configured but not yet calculated port/material snapshot should remain empty.");
        AssertPortMaterialEntry(
            readySnapshot.GetPort(UnitOperationPortCatalog.Feed.Name),
            UnitOperationPortCatalog.Feed,
            UnitOperationHostPortMaterialState.None,
            isConnected: true,
            connectedTargetName: "Contract Feed",
            expectedBoundStreamIds: ["stream-feed"],
            expectedMaterialStreamIds: []);
        AssertPortMaterialEntry(
            readySnapshot.GetPort(UnitOperationPortCatalog.Product.Name),
            UnitOperationPortCatalog.Product,
            UnitOperationHostPortMaterialState.None,
            isConnected: true,
            connectedTargetName: "Contract Product",
            expectedBoundStreamIds: ["stream-liquid", "stream-vapor"],
            expectedMaterialStreamIds: []);

        context.UnitOperation.Calculate();

        var availableSnapshot = context.ReadPortMaterial();
        ContractAssert.Equal(UnitOperationHostPortMaterialState.Available, availableSnapshot.State, "Successful calculate should publish available port/material snapshot state.");
        var availableFeed = availableSnapshot.GetPort(UnitOperationPortCatalog.Feed.Name);
        AssertPortMaterialEntry(
            availableFeed,
            UnitOperationPortCatalog.Feed,
            UnitOperationHostPortMaterialState.Available,
            isConnected: true,
            connectedTargetName: "Contract Feed",
            expectedBoundStreamIds: ["stream-feed"],
            expectedMaterialStreamIds: ["stream-feed"]);
        ContractAssert.True(availableFeed.MaterialEntries[0].TemperatureK > 0.0d, "Available feed material entry should expose positive temperature.");
        ContractAssert.True(availableFeed.MaterialEntries[0].PressurePa > 0.0d, "Available feed material entry should expose positive pressure.");
        var availableProduct = availableSnapshot.GetPort(UnitOperationPortCatalog.Product.Name);
        AssertPortMaterialEntry(
            availableProduct,
            UnitOperationPortCatalog.Product,
            UnitOperationHostPortMaterialState.Available,
            isConnected: true,
            connectedTargetName: "Contract Product",
            expectedBoundStreamIds: ["stream-liquid", "stream-vapor"],
            expectedMaterialStreamIds: ["stream-liquid", "stream-vapor"]);
        ContractAssert.True(
            availableProduct.MaterialEntries.All(static entry => entry.TotalMolarFlowMolS >= 0.0d && entry.PressurePa > 0.0d),
            "Available product material entries should expose non-negative flow and positive pressure.");

        context.DisconnectProductPort();

        var staleSnapshot = context.ReadPortMaterial();
        ContractAssert.Equal(UnitOperationHostPortMaterialState.Stale, staleSnapshot.State, "Configuration invalidation after success should mark port/material snapshot stale.");
        AssertPortMaterialEntry(
            staleSnapshot.GetPort(UnitOperationPortCatalog.Feed.Name),
            UnitOperationPortCatalog.Feed,
            UnitOperationHostPortMaterialState.Stale,
            isConnected: true,
            connectedTargetName: "Contract Feed",
            expectedBoundStreamIds: ["stream-feed"],
            expectedMaterialStreamIds: []);
        AssertPortMaterialEntry(
            staleSnapshot.GetPort(UnitOperationPortCatalog.Product.Name),
            UnitOperationPortCatalog.Product,
            UnitOperationHostPortMaterialState.Stale,
            isConnected: false,
            connectedTargetName: null,
            expectedBoundStreamIds: ["stream-liquid", "stream-vapor"],
            expectedMaterialStreamIds: []);

        context.UnitOperation.Terminate();

        var terminatedSnapshot = context.ReadPortMaterial();
        ContractAssert.Equal(UnitOperationHostPortMaterialState.Terminated, terminatedSnapshot.State, "Terminated unit should expose terminal port/material snapshot state.");
        ContractAssert.Equal(0, terminatedSnapshot.PortCount, "Terminated port/material snapshot should not bypass lifecycle guards to expose ports.");
    }

    public static void ExecutionSnapshot_ExposesStepAndDiagnosticShape(ContractTestContext context)
    {
        var constructedSnapshot = context.ReadExecution();
        ContractAssert.Equal(UnitOperationHostExecutionState.None, constructedSnapshot.State, "Constructed execution snapshot should start empty.");
        ContractAssert.False(constructedSnapshot.IsCurrentConfigurationExecution, "Constructed execution snapshot should not be current.");
        ContractAssert.Equal(0, constructedSnapshot.StepCount, "Constructed execution snapshot should not expose steps.");
        ContractAssert.Equal(0, constructedSnapshot.DiagnosticCount, "Constructed execution snapshot should not expose diagnostics.");

        context.ConfigureMinimumValidInputs();

        var readySnapshot = context.ReadExecution();
        ContractAssert.Equal(UnitOperationHostExecutionState.None, readySnapshot.State, "Ready-but-not-calculated execution snapshot should remain empty.");
        ContractAssert.False(readySnapshot.IsCurrentConfigurationExecution, "Ready-but-not-calculated execution snapshot should not be current.");
        ContractAssert.Equal(0, readySnapshot.StepCount, "Ready-but-not-calculated execution snapshot should not expose steps.");

        context.UnitOperation.Calculate();

        var availableSnapshot = context.ReadExecution();
        ContractAssert.Equal(UnitOperationHostExecutionState.Available, availableSnapshot.State, "Successful calculate should expose available execution snapshot state.");
        ContractAssert.True(availableSnapshot.IsCurrentConfigurationExecution, "Successful calculate should expose a current execution snapshot.");
        ContractAssert.Equal("converged", availableSnapshot.CalculationStatus, "Execution snapshot should preserve calculation status.");
        ContractAssert.NotNull(availableSnapshot.Summary, "Available execution snapshot should expose summary.");
        ContractAssert.Equal(4, availableSnapshot.DiagnosticCount, "Execution snapshot should preserve diagnostic count.");
        ContractAssert.Equal(3, availableSnapshot.StepCount, "Execution snapshot should preserve three solve steps for the sample flowsheet.");
        ContractAssert.SequenceEqual(
            ["feed-1", "heater-1", "flash-1"],
            availableSnapshot.StepEntries.Select(static step => step.UnitId),
            "Execution snapshot should preserve stable step unit order.");
        ContractAssert.SequenceEqual(
            ["stream-feed", "stream-heated", "stream-liquid", "stream-vapor"],
            availableSnapshot.Summary!.RelatedStreamIds,
            "Execution snapshot summary should preserve related stream ids.");
        var flashStep = availableSnapshot.GetStep(2);
        ContractAssert.Equal(2, flashStep.Index, "Execution snapshot should preserve zero-based native step index.");
        ContractAssert.Equal("flash-1", flashStep.UnitId, "Execution snapshot should preserve flash step unit id.");
        ContractAssert.Equal("flash_drum", flashStep.UnitKind, "Execution snapshot should preserve flash step unit kind.");
        ContractAssert.SequenceEqual(["stream-heated"], flashStep.ConsumedStreamIds, "Execution snapshot should preserve flash-step consumed streams.");
        ContractAssert.SequenceEqual(["stream-liquid", "stream-vapor"], flashStep.ProducedStreamIds, "Execution snapshot should preserve flash-step produced streams.");
        ContractAssert.Contains(flashStep.Summary, "flash-1", "Execution snapshot should preserve step summary text.");

        context.DisconnectProductPort();

        var staleSnapshot = context.ReadExecution();
        ContractAssert.Equal(UnitOperationHostExecutionState.Stale, staleSnapshot.State, "Configuration invalidation after success should mark execution snapshot stale.");
        ContractAssert.False(staleSnapshot.IsCurrentConfigurationExecution, "Stale execution snapshot should not be current.");
        ContractAssert.Equal(0, staleSnapshot.StepCount, "Stale execution snapshot should not expose old steps as current data.");
        ContractAssert.Equal(0, staleSnapshot.DiagnosticCount, "Stale execution snapshot should not expose old diagnostics as current data.");

        context.UnitOperation.Terminate();

        var terminatedSnapshot = context.ReadExecution();
        ContractAssert.Equal(UnitOperationHostExecutionState.Terminated, terminatedSnapshot.State, "Terminated execution snapshot should expose terminal state.");
        ContractAssert.False(terminatedSnapshot.IsCurrentConfigurationExecution, "Terminated execution snapshot should not be current.");
        ContractAssert.Equal(0, terminatedSnapshot.StepCount, "Terminated execution snapshot should not expose steps.");
        ContractAssert.Equal(0, terminatedSnapshot.DiagnosticCount, "Terminated execution snapshot should not expose diagnostics.");
    }

    public static void SessionSnapshot_ExposesUnifiedHostView(ContractTestContext context)
    {
        var constructedSnapshot = context.ReadSession();
        ContractAssert.Equal(UnitOperationHostSessionState.Constructed, constructedSnapshot.State, "Constructed host session should expose constructed session state.");
        ContractAssert.Equal(UnitOperationHostConfigurationState.Constructed, constructedSnapshot.Configuration.State, "Constructed host session should preserve constructed configuration state.");
        ContractAssert.True(constructedSnapshot.Summary.HasBlockingActions, "Constructed host session should report blocking actions.");
        ContractAssert.False(constructedSnapshot.Summary.IsReadyForCalculate, "Constructed host session should not be ready for Calculate().");
        ContractAssert.False(constructedSnapshot.Summary.HasFailureReport, "Constructed host session should not expose failure report state.");
        ContractAssert.False(constructedSnapshot.Summary.HasCurrentResults, "Constructed host session should not expose current results.");
        ContractAssert.False(constructedSnapshot.Summary.RequiresCalculateRefresh, "Constructed host session should not be stale.");
        ContractAssert.SequenceEqual(
            [
                nameof(RadishFlowCapeOpenUnitOperation.Initialize),
                UnitOperationParameterCatalog.FlowsheetJson.ConfigurationOperationName,
                UnitOperationParameterCatalog.PropertyPackageId.ConfigurationOperationName,
                UnitOperationPortCatalog.Feed.ConnectionOperationName,
            ],
            constructedSnapshot.Summary.RecommendedOperations,
            "Constructed host session should expose distinct recommended operations in action order.");
        ContractAssert.True(
            constructedSnapshot.ContainsRecommendedOperation(nameof(RadishFlowCapeOpenUnitOperation.Initialize)),
            "Constructed host session should recommend Initialize().");

        context.ConfigureMinimumValidInputs();

        var readySnapshot = context.ReadSession();
        ContractAssert.Equal(UnitOperationHostSessionState.Ready, readySnapshot.State, "Ready host session should expose ready session state.");
        ContractAssert.True(readySnapshot.Summary.IsReadyForCalculate, "Ready host session should report ready-for-calculate.");
        ContractAssert.False(readySnapshot.Summary.HasBlockingActions, "Ready host session should not expose blocking actions.");
        ContractAssert.False(readySnapshot.Summary.HasFailureReport, "Ready host session should not expose failure report state.");
        ContractAssert.False(readySnapshot.Summary.HasCurrentResults, "Ready host session should not expose current results before Calculate().");
        ContractAssert.False(readySnapshot.Summary.RequiresCalculateRefresh, "Ready host session should not be stale before Calculate().");
        ContractAssert.Equal(0, readySnapshot.Summary.RecommendedOperations.Count, "Ready host session should not recommend follow-up operations.");
        ContractAssert.Equal(readySnapshot.Configuration.Headline, readySnapshot.Headline, "Ready host session should default to configuration headline.");

        context.UnitOperation.SelectPropertyPackage("missing-package-for-session-contract");
        var nativeFailure = ContractAssert.Throws<CapeInvalidArgumentException>(
            static unitOperation => unitOperation.Calculate(),
            context.UnitOperation,
            "Calculate() with missing package should fail for host-session contract.");
        ContractAssert.Equal("MissingEntity", nativeFailure.NativeStatus, "Session contract native failure should preserve MissingEntity.");

        var failureSnapshot = context.ReadSession();
        ContractAssert.Equal(UnitOperationHostSessionState.Failure, failureSnapshot.State, "Failure host session should expose failure session state.");
        ContractAssert.True(failureSnapshot.Summary.IsReadyForCalculate, "Failure host session should preserve ready configuration state.");
        ContractAssert.False(failureSnapshot.Summary.HasBlockingActions, "Failure host session should not invent blocking actions.");
        ContractAssert.True(failureSnapshot.Summary.HasFailureReport, "Failure host session should expose failure report state.");
        ContractAssert.False(failureSnapshot.Summary.HasCurrentResults, "Failure host session should not expose current results.");
        ContractAssert.False(failureSnapshot.Summary.RequiresCalculateRefresh, "Failure host session should not be stale.");
        ContractAssert.Equal(UnitOperationCalculationReportState.Failure, failureSnapshot.Report.State, "Failure host session should preserve failure report snapshot.");
        ContractAssert.Equal(failureSnapshot.Report.Headline, failureSnapshot.Headline, "Failure host session should prefer report headline.");

        context.SelectPackage();
        context.UnitOperation.Calculate();

        var availableSnapshot = context.ReadSession();
        ContractAssert.Equal(UnitOperationHostSessionState.Available, availableSnapshot.State, "Successful host session should expose available session state.");
        ContractAssert.True(availableSnapshot.Summary.IsReadyForCalculate, "Successful host session should preserve ready configuration state.");
        ContractAssert.False(availableSnapshot.Summary.HasBlockingActions, "Successful host session should not expose blocking actions.");
        ContractAssert.True(availableSnapshot.Summary.HasCurrentMaterialResults, "Successful host session should expose current material results.");
        ContractAssert.True(availableSnapshot.Summary.HasCurrentExecution, "Successful host session should expose current execution.");
        ContractAssert.True(availableSnapshot.Summary.HasCurrentResults, "Successful host session should expose current combined results.");
        ContractAssert.False(availableSnapshot.Summary.HasFailureReport, "Successful host session should clear failure report state.");
        ContractAssert.False(availableSnapshot.Summary.RequiresCalculateRefresh, "Successful host session should not be stale.");
        ContractAssert.Equal(UnitOperationCalculationReportState.Success, availableSnapshot.Report.State, "Successful host session should preserve success report state.");
        ContractAssert.Equal(availableSnapshot.Execution.Headline, availableSnapshot.Headline, "Successful host session should prefer execution headline.");

        context.DisconnectProductPort();

        var staleSnapshot = context.ReadSession();
        ContractAssert.Equal(UnitOperationHostSessionState.Stale, staleSnapshot.State, "Stale host session should expose stale session state.");
        ContractAssert.False(staleSnapshot.Summary.IsReadyForCalculate, "Stale host session should not remain ready when configuration is broken.");
        ContractAssert.True(staleSnapshot.Summary.HasBlockingActions, "Stale host session should expose blocking actions.");
        ContractAssert.False(staleSnapshot.Summary.HasCurrentResults, "Stale host session should not expose current combined results.");
        ContractAssert.True(staleSnapshot.Summary.RequiresCalculateRefresh, "Stale host session should request Calculate() refresh after recovery.");
        ContractAssert.True(
            staleSnapshot.ContainsRecommendedOperation(UnitOperationPortCatalog.Product.ConnectionOperationName),
            "Stale host session should recommend reconnecting the required product port.");
        ContractAssert.Equal(staleSnapshot.Execution.Headline, staleSnapshot.Headline, "Stale host session should prefer stale execution headline.");

        context.UnitOperation.Terminate();

        var terminatedSnapshot = context.ReadSession();
        ContractAssert.Equal(UnitOperationHostSessionState.Terminated, terminatedSnapshot.State, "Terminated host session should expose terminated session state.");
        ContractAssert.Equal(UnitOperationHostConfigurationState.Terminated, terminatedSnapshot.Configuration.State, "Terminated host session should preserve terminated configuration state.");
        ContractAssert.True(terminatedSnapshot.Summary.HasBlockingActions, "Terminated host session should still expose terminal blocking action.");
        ContractAssert.False(terminatedSnapshot.Summary.HasCurrentResults, "Terminated host session should not expose current results.");
        ContractAssert.False(terminatedSnapshot.Summary.HasFailureReport, "Terminated host session should not expose failure report state.");
        ContractAssert.False(terminatedSnapshot.Summary.RequiresCalculateRefresh, "Terminated host session should not report refreshable stale results.");
        ContractAssert.Equal(0, terminatedSnapshot.Summary.RecommendedOperations.Count, "Terminated host session should not recommend executable follow-up operations.");
        ContractAssert.Equal(terminatedSnapshot.Configuration.Headline, terminatedSnapshot.Headline, "Terminated host session should use configuration headline.");
    }

    private static void AssertActionPlan(
        UnitOperationHostActionPlan actionPlan,
        string scenario,
        params ContractExpectedAction[] expectedActions)
    {
        ContractAssert.Equal(expectedActions.Length, actionPlan.ActionCount, $"{scenario} should expose the expected number of actions.");
        ContractAssert.True(
            actionPlan.Groups.All(static group => !string.IsNullOrWhiteSpace(group.Title) && group.Actions.Count > 0),
            $"{scenario} should expose non-empty action-plan groups.");

        var expectedGroupKinds = expectedActions
            .Select(static action => action.GroupKind)
            .Distinct()
            .ToArray();
        ContractAssert.SequenceEqual(
            expectedGroupKinds,
            actionPlan.Groups.Select(static group => group.Kind),
            $"{scenario} should expose the expected action-plan group order.");

        for (var index = 0; index < expectedActions.Length; index++)
        {
            expectedActions[index].AssertMatches(actionPlan.Actions[index], scenario, index + 1);
        }
    }

    private static ContractExpectedAction Action(
        UnitOperationHostActionGroupKind groupKind,
        UnitOperationHostActionTargetKind targetKind,
        string? canonicalOperationName,
        UnitOperationHostConfigurationIssueKind issueKind,
        string reasonFragment,
        params string[] targetNames)
    {
        return new ContractExpectedAction(
            GroupKind: groupKind,
            TargetKind: targetKind,
            TargetNames: targetNames,
            CanonicalOperationName: canonicalOperationName,
            IssueKind: issueKind,
            ReasonFragment: reasonFragment,
            IsBlocking: true);
    }

    private static void AssertPortMaterialEntry(
        UnitOperationHostPortMaterialEntry entry,
        UnitOperationPortDefinition definition,
        UnitOperationHostPortMaterialState expectedState,
        bool isConnected,
        string? connectedTargetName,
        IReadOnlyList<string> expectedBoundStreamIds,
        IReadOnlyList<string> expectedMaterialStreamIds)
    {
        ContractAssert.Equal(definition.Name, entry.Name, "Port/material entry should preserve canonical port name.");
        ContractAssert.Equal(definition.Description, entry.Description, "Port/material entry should preserve canonical port description.");
        ContractAssert.Equal(definition.Direction, entry.Direction, "Port/material entry should preserve port direction.");
        ContractAssert.Equal(definition.PortType, entry.PortType, "Port/material entry should preserve port type.");
        ContractAssert.Equal(definition.IsRequired, entry.IsRequired, "Port/material entry should preserve required flag.");
        ContractAssert.Equal(definition.BoundaryMaterialRole, entry.BoundaryMaterialRole, "Port/material entry should preserve boundary material role.");
        ContractAssert.Equal(expectedState, entry.MaterialState, "Port/material entry should expose the expected material state.");
        ContractAssert.Equal(isConnected, entry.IsConnected, "Port/material entry should expose the expected connection state.");
        ContractAssert.Equal(connectedTargetName, entry.ConnectedTargetName, "Port/material entry should expose the expected connected target name.");
        ContractAssert.SequenceEqual(expectedBoundStreamIds, entry.BoundStreamIds, "Port/material entry should expose the expected bound stream ids.");
        ContractAssert.SequenceEqual(expectedMaterialStreamIds, entry.MaterialEntries.Select(static entry => entry.StreamId), "Port/material entry should expose the expected current material stream ids.");
    }

    private static void AssertMutationOutcome(
        UnitOperationHostObjectMutationOutcome outcome,
        UnitOperationHostObjectMutationKind expectedOperation,
        UnitOperationHostActionTargetKind expectedTargetKind,
        string expectedTargetName)
    {
        ContractAssert.True(outcome.Succeeded, "Mutation outcome should report success.");
        ContractAssert.Equal(expectedOperation, outcome.Operation, "Mutation outcome should preserve operation kind.");
        ContractAssert.Equal(expectedTargetKind, outcome.Target.Kind, "Mutation outcome should preserve target kind.");
        ContractAssert.SequenceEqual([expectedTargetName], outcome.Target.Names, "Mutation outcome should preserve target name.");
        ContractAssert.True(outcome.InvalidatesValidation, "Mutation outcome should report validation invalidation.");
        ContractAssert.True(outcome.InvalidatesCalculationReport, "Mutation outcome should report calculation report invalidation.");
    }

    private static void AssertMutationCommand(
        UnitOperationHostObjectMutationCommand command,
        UnitOperationHostObjectMutationKind expectedKind,
        string expectedTargetName,
        object? expectedPayload)
    {
        ContractAssert.Equal(expectedKind, command.Kind, "Mutation command should preserve command kind.");
        ContractAssert.Equal(expectedTargetName, command.TargetName, "Mutation command should preserve target name.");
        if (expectedPayload is null)
        {
            ContractAssert.Null(command.Payload, "Mutation command should preserve null payload.");
            return;
        }

        if (expectedPayload is string expectedString)
        {
            ContractAssert.Equal(expectedString, (string?)command.Payload, "Mutation command should preserve string payload.");
            return;
        }

        ContractAssert.SameReference(expectedPayload, command.Payload, "Mutation command should preserve object payload by reference.");
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
        ContractAssert.NotNull(context.UnitOperation.LastCalculationResult, "Successful Calculate() should preserve the last success result.");
        ContractAssert.Equal(3, context.UnitOperation.LastCalculationResult!.Steps.Count, "Successful Calculate() should materialize native solve steps into the calculation result contract.");
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

    public string PayloadPath => _options.PayloadPath;

    public string PackageId => _options.PackageId;

    public string FlowsheetJsonText => File.ReadAllText(_options.ProjectPath);

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
        UnitOperation.LoadFlowsheetJson(FlowsheetJsonText);
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

    public UnitOperationHostObjectRuntimeSnapshot ReadObjectRuntime()
    {
        return UnitOperationHostObjectRuntimeReader.Read(UnitOperation);
    }

    public UnitOperationHostActionPlan ReadActionPlan()
    {
        return UnitOperationHostActionPlanReader.Read(ReadConfiguration());
    }

    public UnitOperationHostActionExecutionInputSet CreateMinimumConfigurationInputSet(
        bool includePackageId,
        bool includePackageFiles = true)
    {
        var values = new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
        {
            [UnitOperationParameterCatalog.FlowsheetJson.Name] = FlowsheetJsonText,
        };

        if (includePackageId)
        {
            values[UnitOperationParameterCatalog.PropertyPackageId.Name] = PackageId;
        }

        if (includePackageFiles)
        {
            values[UnitOperationParameterCatalog.PropertyPackageManifestPath.Name] = ManifestPath;
            values[UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name] = PayloadPath;
        }

        return new UnitOperationHostActionExecutionInputSet(
            parameterValues: values,
            portObjects: new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase)
            {
                [UnitOperationPortCatalog.Feed.Name] = new ContractConnectedObject("Contract Round Feed"),
                [UnitOperationPortCatalog.Product.Name] = new ContractConnectedObject("Contract Round Product"),
            });
    }

    public IReadOnlyList<UnitOperationHostObjectMutationCommand> CreateOptionalPackageFileMutationCommands()
    {
        return
        [
            UnitOperationHostObjectMutationCommand.SetParameterValue(
                UnitOperationParameterCatalog.PropertyPackageManifestPath.Name,
                ManifestPath),
            UnitOperationHostObjectMutationCommand.SetParameterValue(
                UnitOperationParameterCatalog.PropertyPackagePayloadPath.Name,
                PayloadPath),
        ];
    }

    public UnitOperationHostPortMaterialSnapshot ReadPortMaterial()
    {
        return UnitOperationHostPortMaterialReader.Read(UnitOperation);
    }

    public UnitOperationHostExecutionSnapshot ReadExecution()
    {
        return UnitOperationHostExecutionReader.Read(UnitOperation);
    }

    public UnitOperationHostSessionSnapshot ReadSession()
    {
        return UnitOperationHostSessionReader.Read(UnitOperation);
    }

    public UnitOperationHostValidationOutcome ValidateRound()
    {
        return UnitOperationHostValidationRunner.Validate(UnitOperation);
    }

    public UnitOperationHostCalculationOutcome CalculateRound()
    {
        return UnitOperationHostCalculationRunner.Calculate(UnitOperation);
    }

    public UnitOperationHostRoundOutcome ExecuteRound(UnitOperationHostRoundRequest? request = null)
    {
        return UnitOperationHostRoundOrchestrator.Execute(UnitOperation, request ?? UnitOperationHostRoundRequest.Default);
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

internal sealed record ContractExpectedAction(
    UnitOperationHostActionGroupKind GroupKind,
    UnitOperationHostActionTargetKind TargetKind,
    IReadOnlyList<string> TargetNames,
    string? CanonicalOperationName,
    UnitOperationHostConfigurationIssueKind IssueKind,
    string ReasonFragment,
    bool IsBlocking)
{
    public void AssertMatches(
        UnitOperationHostActionItem actual,
        string scenario,
        int expectedOrder)
    {
        ContractAssert.Equal(expectedOrder, actual.RecommendedOrder, $"{scenario} should preserve recommended order.");
        ContractAssert.Equal(GroupKind, actual.GroupKind, $"{scenario} should preserve action group.");
        ContractAssert.Equal(TargetKind, actual.Target.Kind, $"{scenario} should preserve target kind.");
        ContractAssert.SequenceEqual(TargetNames, actual.Target.Names, $"{scenario} should preserve target names.");
        ContractAssert.Equal(IsBlocking, actual.IsBlocking, $"{scenario} should preserve blocking classification.");
        ContractAssert.Equal(IssueKind, actual.IssueKind, $"{scenario} should preserve issue kind.");
        ContractAssert.Equal(CanonicalOperationName, actual.CanonicalOperationName, $"{scenario} should preserve canonical operation.");
        ContractAssert.Contains(actual.Reason, ReasonFragment, $"{scenario} should preserve action reason.");
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
