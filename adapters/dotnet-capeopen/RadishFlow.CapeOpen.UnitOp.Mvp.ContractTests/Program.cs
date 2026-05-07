using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Guids;
using RadishFlow.CapeOpen.Interop.Ole;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Persistence;
using RadishFlow.CapeOpen.Interop.Thermo;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.Results;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text.Json;

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
            ("pme-ole-object-probe-contract", static context => ContractTests.PmeOleObjectProbe_ExposesNoOpOleObject(context)),
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
            ("feed-material-overlay-contract", static context => ContractTests.ConnectedFeedMaterial_OverlaysBoundaryInputBeforeNativeSolve(context)),
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
internal static partial class ContractTests
{
    public static void AssemblyComIdentity_StaysAlignedWithTypeLibraryRegistration()
    {
        AssertAssemblyTypeLibraryIdentity(
            typeof(RadishFlowCapeOpenUnitOperation).Assembly,
            "UnitOp.Mvp assembly should expose a stable COM type library identity.");
        AssertAssemblyTypeLibraryIdentity(
            typeof(ICapeUtilities).Assembly,
            "Interop assembly should expose the same COM type library identity used by the frozen MVP TLB.");

        AssertComInterfaceType(
            typeof(ICapeIdentification),
            ComInterfaceType.InterfaceIsDual,
            "ICapeIdentification should stay aligned with the frozen dual IDL interface.");
        AssertComInterfaceType(
            typeof(ICapeUtilities),
            ComInterfaceType.InterfaceIsDual,
            "ICapeUtilities should stay aligned with the frozen dual IDL interface.");
        AssertComInterfaceType(
            typeof(ICapeSimulationContext),
            ComInterfaceType.InterfaceIsIUnknown,
            "ICapeSimulationContext should expose the standard marker interface shape.");
        AssertComInterfaceType(
            typeof(ICapeCOSEUtilities),
            ComInterfaceType.InterfaceIsDual,
            "ICapeCOSEUtilities should expose the standard dual COSE utilities shape.");
        AssertComInterfaceType(
            typeof(ICapeDiagnostic),
            ComInterfaceType.InterfaceIsDual,
            "ICapeDiagnostic should expose the standard dual diagnostic shape.");
        AssertComInterfaceType(
            typeof(ICapeMaterialTemplateSystem),
            ComInterfaceType.InterfaceIsDual,
            "ICapeMaterialTemplateSystem should expose the standard dual material-template shape.");
        AssertComInterfaceType(
            typeof(ICapeUnit),
            ComInterfaceType.InterfaceIsDual,
            "ICapeUnit should stay aligned with the frozen dual IDL interface.");
        AssertComInterfaceType(
            typeof(ICapeUnitReport),
            ComInterfaceType.InterfaceIsDual,
            "ICapeUnitReport should stay aligned with the frozen dual IDL interface.");
        AssertComInterfaceType(
            typeof(ICapeCollection),
            ComInterfaceType.InterfaceIsDual,
            "ICapeCollection should stay aligned with the frozen dual IDL interface.");
        AssertComInterfaceType(
            typeof(ICapeParameter),
            ComInterfaceType.InterfaceIsDual,
            "ICapeParameter should stay aligned with the frozen dual IDL interface.");
        AssertComInterfaceType(
            typeof(ICapeParameterSpec),
            ComInterfaceType.InterfaceIsDual,
            "ICapeParameterSpec should stay aligned with the frozen dual IDL interface.");
        AssertComInterfaceType(
            typeof(ICapeOptionParameterSpec),
            ComInterfaceType.InterfaceIsDual,
            "ICapeOptionParameterSpec should stay aligned with the frozen dual IDL interface.");
        AssertComInterfaceType(
            typeof(ICapeUnitPort),
            ComInterfaceType.InterfaceIsDual,
            "ICapeUnitPort should stay aligned with the frozen dual IDL interface.");
        AssertComInterfaceType(
            typeof(ECapeRoot),
            ComInterfaceType.InterfaceIsIDispatch,
            "ECapeRoot should expose the standard CAPE-OPEN dispatch error surface.");
        AssertComInterfaceType(
            typeof(ECapeUser),
            ComInterfaceType.InterfaceIsIDispatch,
            "ECapeUser should expose the standard CAPE-OPEN dispatch error surface.");
        AssertSimulationContextUsesRawPointer();

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
            typeof(UnitOperationParameterSpecificationPlaceholder),
            typeof(ICapeParameterSpec),
            "Parameter spec placeholder COM default interface should expose ICapeParameterSpec before type-specific spec interfaces.");
        AssertComDefaultInterface(
            typeof(UnitOperationPortPlaceholder),
            typeof(ICapeUnitPort),
            "Port placeholder COM default interface should expose ICapeUnitPort for connection late binding.");
        AssertComDefaultInterface(
            typeof(UnitOperationConnectedObjectPlaceholder),
            typeof(ICapeIdentification),
            "Connected object placeholder COM default interface should expose ICapeIdentification for connectedObject late binding.");
        AssertComDefaultInterface(
            typeof(UnitOperationSimulationContextPlaceholder),
            typeof(ICapeCOSEUtilities),
            "Simulation context placeholder COM default interface should expose ICapeCOSEUtilities for COFE context probing.");

        ContractAssert.True(
            typeof(ICapeUnitReport).IsAssignableFrom(typeof(RadishFlowCapeOpenUnitOperation)),
            "Unit operation should expose ICapeUnitReport as an optional PME activation/reporting surface.");
        ContractAssert.True(
            typeof(ECapeRoot).IsAssignableFrom(typeof(RadishFlowCapeOpenUnitOperation)),
            "Unit operation should expose ECapeRoot for DWSIM CAPE-OPEN exception handling compatibility.");
        ContractAssert.True(
            typeof(ECapeUser).IsAssignableFrom(typeof(RadishFlowCapeOpenUnitOperation)),
            "Unit operation should expose ECapeUser for DWSIM CAPE-OPEN exception handling compatibility.");
        ContractAssert.True(
            typeof(ICapeOptionParameterSpec).IsAssignableFrom(typeof(UnitOperationParameterSpecificationPlaceholder)),
            "Option parameter specs should expose ICapeOptionParameterSpec for PME parameter inspectors.");
        ContractAssert.True(
            typeof(ICapeParameterSpec).IsAssignableFrom(typeof(UnitOperationParameterPlaceholder)),
            "Parameter placeholders should directly expose ICapeParameterSpec for DWSIM-style parameter enumeration.");
        ContractAssert.True(
            typeof(ICapeOptionParameterSpec).IsAssignableFrom(typeof(UnitOperationParameterPlaceholder)),
            "CAPE_OPTION parameter placeholders should directly expose ICapeOptionParameterSpec for DWSIM-style parameter enumeration.");
        ContractAssert.True(
            typeof(IPersistStreamInit).IsAssignableFrom(typeof(RadishFlowCapeOpenUnitOperation)),
            "Unit operation should expose IPersistStreamInit for PME canvas object persistence probing.");
        ContractAssert.True(
            typeof(IPersistStorage).IsAssignableFrom(typeof(RadishFlowCapeOpenUnitOperation)),
            "Unit operation should expose IPersistStorage for PME canvas storage persistence probing.");
        ContractAssert.True(
            typeof(IOleObject).IsAssignableFrom(typeof(RadishFlowCapeOpenUnitOperation)),
            "Unit operation should expose IOleObject for PME canvas embedding probing.");
    }

    private static void AssertComInterfaceType(Type interfaceType, ComInterfaceType expectedInterfaceType, string context)
    {
        var interfaceTypeAttribute = interfaceType
            .GetCustomAttributes(typeof(InterfaceTypeAttribute), inherit: false)
            .OfType<InterfaceTypeAttribute>()
            .SingleOrDefault();

        ContractAssert.NotNull(interfaceTypeAttribute, $"{context} Missing InterfaceType.");
        ContractAssert.Equal(expectedInterfaceType, interfaceTypeAttribute!.Value, context);
    }

    private static void AssertSimulationContextUsesRawPointer()
    {
        var utilitiesMethods = typeof(ICapeUtilities).GetMethods();
        var setter = utilitiesMethods.SingleOrDefault(static method => method.Name == "set_SimulationContext");
        var getter = utilitiesMethods.SingleOrDefault(static method => method.Name == "get_SimulationContext");
        ContractAssert.NotNull(
            setter,
            "ICapeUtilities should expose a SimulationContext setter.");
        ContractAssert.NotNull(
            getter,
            "ICapeUtilities should keep a SimulationContext getter for late-bound native PME callers.");
        ContractAssert.SequenceEqual(
            ["get_Parameters", "set_SimulationContext", "Initialize", "Terminate", "Edit", "get_SimulationContext"],
            utilitiesMethods.Select(static method => method.Name).ToArray(),
            "ICapeUtilities vtable order should match DWSIM's setter-only CAPE-OPEN PIA before exposing the COFE-compatible getter.");
        AssertMarshalAs(
            getter!.ReturnParameter,
            UnmanagedType.IDispatch,
            "ICapeUtilities.SimulationContext getter should expose an IDispatch pointer for native PME callers.");
        var setterParameter = setter!.GetParameters().Single();
        ContractAssert.Equal(
            typeof(IntPtr),
            setterParameter.ParameterType,
            "ICapeUtilities.SimulationContext setter should receive the host context as a raw pointer.");
        AssertNoMarshalAs(
            setterParameter,
            "ICapeUtilities.SimulationContext setter should remain raw to avoid DWSIM pre-method interface marshaling.");
    }

    private static void AssertMarshalAs(ParameterInfo parameter, UnmanagedType expectedType, string context)
    {
        var marshalAs = parameter
            .GetCustomAttributes(typeof(MarshalAsAttribute), inherit: false)
            .OfType<MarshalAsAttribute>()
            .SingleOrDefault();

        ContractAssert.NotNull(marshalAs, $"{context} Missing MarshalAs.");
        ContractAssert.Equal(expectedType, marshalAs!.Value, context);
    }

    private static void AssertNoMarshalAs(ParameterInfo parameter, string context)
    {
        var marshalAs = parameter
            .GetCustomAttributes(typeof(MarshalAsAttribute), inherit: false)
            .OfType<MarshalAsAttribute>()
            .SingleOrDefault();

        ContractAssert.Null(marshalAs, context);
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

    private static void AssertRegistrationCategory(
        CapeOpenRegistrationDescriptor descriptor,
        string expectedName,
        string expectedCategoryId)
    {
        ContractAssert.True(
            descriptor.Categories.Any(category =>
                string.Equals(category.Name, expectedName, StringComparison.Ordinal) &&
                string.Equals(category.CategoryId, expectedCategoryId, StringComparison.OrdinalIgnoreCase)),
            $"Register descriptor should advertise {expectedName}.");
    }

    private static void AssertRegistryCategoryPlan(
        CapeOpenRegistrationDescriptor descriptor,
        string expectedCategoryId)
    {
        ContractAssert.True(
            descriptor.RegistryPlan.Any(entry =>
                entry.Operation == CapeOpenRegistryPlanOperation.SetValue &&
                entry.KeyPath.EndsWith($@"\Implemented Categories\{{{expectedCategoryId}}}", StringComparison.OrdinalIgnoreCase)),
            $"Register plan should write implemented category {expectedCategoryId}.");
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
        AssertRegistrationCategory(dryRunDescriptor, "CAPE-OPEN Object", CapeOpenCategoryIds.CapeOpenObject);
        AssertRegistrationCategory(dryRunDescriptor, "CAPE-OPEN Unit Operation", CapeOpenCategoryIds.UnitOperation);
        AssertRegistrationCategory(dryRunDescriptor, "CAPE-OPEN Consumes Thermodynamics", CapeOpenCategoryIds.ConsumesThermodynamics);
        AssertRegistrationCategory(dryRunDescriptor, "CAPE-OPEN Supports Thermodynamics 1.0", CapeOpenCategoryIds.SupportsThermodynamics10);
        AssertRegistrationCategory(dryRunDescriptor, "CAPE-OPEN Supports Thermodynamics 1.1", CapeOpenCategoryIds.SupportsThermodynamics11);
        AssertRegistryCategoryPlan(dryRunDescriptor, CapeOpenCategoryIds.ConsumesThermodynamics);
        AssertRegistryCategoryPlan(dryRunDescriptor, CapeOpenCategoryIds.SupportsThermodynamics10);
        AssertRegistryCategoryPlan(dryRunDescriptor, CapeOpenCategoryIds.SupportsThermodynamics11);
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
            dryRunDescriptor.ImplementedInterfaces.Any(static implementedInterface =>
                string.Equals(implementedInterface.Name, "IPersistStorage", StringComparison.Ordinal) &&
                string.Equals(implementedInterface.InterfaceId, ComPersistenceInterfaceIds.IPersistStorage, StringComparison.OrdinalIgnoreCase)),
            "Register descriptor should advertise IPersistStorage as an implemented PME canvas storage persistence interface.");
        ContractAssert.True(
            dryRunDescriptor.ImplementedInterfaces.Any(static implementedInterface =>
                string.Equals(implementedInterface.Name, "IOleObject", StringComparison.Ordinal) &&
                string.Equals(implementedInterface.InterfaceId, ComOleInterfaceIds.IOleObject, StringComparison.OrdinalIgnoreCase)),
            "Register descriptor should advertise IOleObject as an implemented PME canvas embedding interface.");
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
        utilities.set_SimulationContext(new IntPtr(1));
        var simulationContextPointer = utilities.get_SimulationContext();
        ContractAssert.True(
            simulationContextPointer != IntPtr.Zero,
            "Activation probe should return a non-null SimulationContext placeholder before a real PME context is consumed.");
        try
        {
            var simulationContextObject = Marshal.GetObjectForIUnknown(simulationContextPointer);
            ContractAssert.True(
                simulationContextObject is ICapeSimulationContext,
                "SimulationContext placeholder should support ICapeSimulationContext.");
            var coseUtilities = (ICapeCOSEUtilities)simulationContextObject;
            ContractAssert.NotNull(coseUtilities.NamedValueList, "SimulationContext placeholder should expose a NamedValueList.");
            ContractAssert.Equal(string.Empty, coseUtilities.NamedValue("FreeFORTRANchannel"), "SimulationContext placeholder should return an empty named value.");
            var diagnostic = (ICapeDiagnostic)simulationContextObject;
            diagnostic.LogMessage("activation probe");
            var materialTemplateSystem = (ICapeMaterialTemplateSystem)simulationContextObject;
            ContractAssert.NotNull(
                materialTemplateSystem.MaterialTemplates,
                "SimulationContext placeholder should expose material template names.");
        }
        finally
        {
            Marshal.Release(simulationContextPointer);
        }

        utilities.Initialize();
        ContractAssert.NotNull(utilities.Parameters, "Activation probe should read ICapeUtilities.Parameters.");
        ContractAssert.Equal(0, utilities.Edit(), "ICapeUtilities.Edit should be a successful no-op until the MVP ships a custom PME editor.");

        var capeUser = (ECapeUser)context.UnitOperation;
        ContractAssert.Equal(0, capeUser.Code, "DWSIM-compatible ECapeUser surface should be readable from the unit object.");
        ContractAssert.Equal(nameof(ICapeUtilities), capeUser.InterfaceName, "DWSIM-compatible ECapeUser surface should identify the default utility interface.");

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

        var capeUserInterfacePointer = IntPtr.Zero;
        var reportInterfacePointer = IntPtr.Zero;
        try
        {
            capeUserInterfacePointer = Marshal.GetComInterfaceForObject(context.UnitOperation, typeof(ECapeUser));
            ContractAssert.True(
                capeUserInterfacePointer != IntPtr.Zero,
                "COM QueryInterface for ECapeUser should succeed for DWSIM CAPE-OPEN exception handling.");

            reportInterfacePointer = Marshal.GetComInterfaceForObject(context.UnitOperation, typeof(ICapeUnitReport));
            ContractAssert.True(
                reportInterfacePointer != IntPtr.Zero,
                "COM QueryInterface for ICapeUnitReport should succeed for the unit operation.");
        }
        finally
        {
            if (capeUserInterfacePointer != IntPtr.Zero)
            {
                Marshal.Release(capeUserInterfacePointer);
            }

            if (reportInterfacePointer != IntPtr.Zero)
            {
                Marshal.Release(reportInterfacePointer);
            }
        }

        utilities.Terminate();
    }

    public static void PmePersistenceProbe_ExposesNoOpPersistStreamInit(ContractTestContext context)
    {
        var streamPersistence = (IPersistStreamInit)context.UnitOperation;
        var loadParameters = typeof(IPersistStreamInit).GetMethod(nameof(IPersistStreamInit.Load))!.GetParameters();
        var saveParameters = typeof(IPersistStreamInit).GetMethod(nameof(IPersistStreamInit.Save))!.GetParameters();
        ContractAssert.Equal(
            typeof(IntPtr),
            loadParameters[0].ParameterType,
            "IPersistStreamInit.Load should keep the raw stream pointer to avoid COM interface marshaling before the no-op method body.");
        ContractAssert.Equal(
            typeof(IntPtr),
            saveParameters[0].ParameterType,
            "IPersistStreamInit.Save should keep the raw stream pointer to avoid COM interface marshaling before the no-op method body.");

        ContractAssert.Equal(
            ComHResults.SOk,
            streamPersistence.GetClassID(out var classId),
            "IPersistStreamInit.GetClassID should return S_OK.");
        ContractAssert.Equal(
            Guid.Parse(UnitOperationComIdentity.ClassId),
            classId,
            "IPersistStreamInit.GetClassID should return the unit operation CLSID.");
        ContractAssert.Equal(
            ComHResults.SFalse,
            streamPersistence.IsDirty(),
            "IPersistStreamInit.IsDirty should report clean no-op persistence state.");
        ContractAssert.Equal(
            ComHResults.SOk,
            streamPersistence.InitNew(),
            "IPersistStreamInit.InitNew should accept PME canvas creation probing.");
        ContractAssert.Equal(
            ComHResults.SOk,
            streamPersistence.Load(IntPtr.Zero),
            "IPersistStreamInit.Load should no-op successfully for the MVP stateless persistence surface.");
        ContractAssert.Equal(
            ComHResults.SOk,
            streamPersistence.Save(IntPtr.Zero, clearDirty: true),
            "IPersistStreamInit.Save should no-op successfully for the MVP stateless persistence surface.");
        ContractAssert.Equal(
            ComHResults.SOk,
            streamPersistence.GetSizeMax(out var size),
            "IPersistStreamInit.GetSizeMax should return S_OK.");
        ContractAssert.Equal(0L, size, "IPersistStreamInit.GetSizeMax should report zero bytes for no-op persistence.");

        var storagePersistence = (IPersistStorage)context.UnitOperation;
        ContractAssert.Equal(
            ComHResults.SOk,
            storagePersistence.GetClassID(out var storageClassId),
            "IPersistStorage.GetClassID should return S_OK.");
        ContractAssert.Equal(
            classId,
            storageClassId,
            "IPersistStorage.GetClassID should return the same unit operation CLSID.");
        ContractAssert.Equal(
            ComHResults.SFalse,
            storagePersistence.IsDirty(),
            "IPersistStorage.IsDirty should report clean no-op persistence state.");
        ContractAssert.Equal(
            ComHResults.SOk,
            storagePersistence.InitNew(IntPtr.Zero),
            "IPersistStorage.InitNew should accept PME canvas storage creation probing.");
        ContractAssert.Equal(
            ComHResults.SOk,
            storagePersistence.Load(IntPtr.Zero),
            "IPersistStorage.Load should no-op successfully for the MVP stateless persistence surface.");
        ContractAssert.Equal(
            ComHResults.SOk,
            storagePersistence.Save(IntPtr.Zero, sameAsLoad: true),
            "IPersistStorage.Save should no-op successfully for the MVP stateless persistence surface.");
        ContractAssert.Equal(
            ComHResults.SOk,
            storagePersistence.SaveCompleted(IntPtr.Zero),
            "IPersistStorage.SaveCompleted should no-op successfully for the MVP stateless persistence surface.");
        ContractAssert.Equal(
            ComHResults.SOk,
            storagePersistence.HandsOffStorage(),
            "IPersistStorage.HandsOffStorage should no-op successfully for the MVP stateless persistence surface.");

        var streamPersistenceInterfacePointer = IntPtr.Zero;
        var storagePersistenceInterfacePointer = IntPtr.Zero;
        try
        {
            streamPersistenceInterfacePointer = Marshal.GetComInterfaceForObject(context.UnitOperation, typeof(IPersistStreamInit));
            ContractAssert.True(
                streamPersistenceInterfacePointer != IntPtr.Zero,
                "COM QueryInterface for IPersistStreamInit should succeed for the unit operation.");
            storagePersistenceInterfacePointer = Marshal.GetComInterfaceForObject(context.UnitOperation, typeof(IPersistStorage));
            ContractAssert.True(
                storagePersistenceInterfacePointer != IntPtr.Zero,
                "COM QueryInterface for IPersistStorage should succeed for the unit operation.");
        }
        finally
        {
            if (streamPersistenceInterfacePointer != IntPtr.Zero)
            {
                Marshal.Release(streamPersistenceInterfacePointer);
            }

            if (storagePersistenceInterfacePointer != IntPtr.Zero)
            {
                Marshal.Release(storagePersistenceInterfacePointer);
            }
        }
    }

    public static void PmeOleObjectProbe_ExposesNoOpOleObject(ContractTestContext context)
    {
        var oleObject = (IOleObject)context.UnitOperation;

        ContractAssert.Equal(ComHResults.SOk, oleObject.SetClientSite(IntPtr.Zero), "IOleObject.SetClientSite should accept a null client site.");
        ContractAssert.Equal(ComHResults.SOk, oleObject.GetClientSite(out var clientSite), "IOleObject.GetClientSite should return S_OK.");
        ContractAssert.Equal(IntPtr.Zero, clientSite, "IOleObject.GetClientSite should report no client site before a real container site is provided.");
        ContractAssert.Equal(ComHResults.SOk, oleObject.SetHostNames("ContractTests", "RadishFlow Unit Operation"), "IOleObject.SetHostNames should no-op successfully.");
        ContractAssert.Equal(ComHResults.SOk, oleObject.SetMoniker(0, IntPtr.Zero), "IOleObject.SetMoniker should no-op successfully.");
        ContractAssert.Equal(ComHResults.ENotImpl, oleObject.GetMoniker(0, 0, out var moniker), "IOleObject.GetMoniker should report no moniker.");
        ContractAssert.Equal(IntPtr.Zero, moniker, "IOleObject.GetMoniker should return a null moniker pointer.");
        ContractAssert.Equal(ComHResults.SOk, oleObject.InitFromData(IntPtr.Zero, creation: true, reserved: 0), "IOleObject.InitFromData should no-op successfully.");
        ContractAssert.Equal(ComHResults.ENotImpl, oleObject.GetClipboardData(0, out var dataObject), "IOleObject.GetClipboardData should report no data object.");
        ContractAssert.Equal(IntPtr.Zero, dataObject, "IOleObject.GetClipboardData should return a null data object pointer.");
        ContractAssert.Equal(ComHResults.SOk, oleObject.DoVerb(0, IntPtr.Zero, IntPtr.Zero, 0, IntPtr.Zero, IntPtr.Zero), "IOleObject.DoVerb should no-op successfully.");
        ContractAssert.Equal(OleConstants.OleObjectNoVerbs, oleObject.EnumVerbs(out var enumOleVerb), "IOleObject.EnumVerbs should report no verbs.");
        ContractAssert.Equal(IntPtr.Zero, enumOleVerb, "IOleObject.EnumVerbs should return a null enum pointer.");
        ContractAssert.Equal(ComHResults.SOk, oleObject.Update(), "IOleObject.Update should no-op successfully.");
        ContractAssert.Equal(ComHResults.SOk, oleObject.IsUpToDate(), "IOleObject.IsUpToDate should report S_OK.");
        ContractAssert.Equal(ComHResults.SOk, oleObject.GetUserClassID(out var classId), "IOleObject.GetUserClassID should return S_OK.");
        ContractAssert.Equal(Guid.Parse(UnitOperationComIdentity.ClassId), classId, "IOleObject.GetUserClassID should return the unit operation CLSID.");

        var userType = IntPtr.Zero;
        try
        {
            ContractAssert.Equal(ComHResults.SOk, oleObject.GetUserType(0, out userType), "IOleObject.GetUserType should return S_OK.");
            ContractAssert.Equal(UnitOperationComIdentity.DisplayName, Marshal.PtrToStringUni(userType), "IOleObject.GetUserType should return the display name.");
        }
        finally
        {
            if (userType != IntPtr.Zero)
            {
                Marshal.FreeCoTaskMem(userType);
            }
        }

        var size = new OleSize(100, 200);
        ContractAssert.Equal(ComHResults.SOk, oleObject.SetExtent(1, ref size), "IOleObject.SetExtent should store the requested size.");
        ContractAssert.Equal(ComHResults.SOk, oleObject.GetExtent(1, out var actualSize), "IOleObject.GetExtent should return S_OK.");
        ContractAssert.Equal(100, actualSize.Width, "IOleObject.GetExtent should preserve width.");
        ContractAssert.Equal(200, actualSize.Height, "IOleObject.GetExtent should preserve height.");
        ContractAssert.Equal(OleConstants.OleAdviseNotSupported, oleObject.Advise(IntPtr.Zero, out var connection), "IOleObject.Advise should report no advise support.");
        ContractAssert.Equal(0U, connection, "IOleObject.Advise should return a zero connection token.");
        ContractAssert.Equal(OleConstants.OleAdviseNotSupported, oleObject.Unadvise(0), "IOleObject.Unadvise should report no advise support.");
        ContractAssert.Equal(OleConstants.OleAdviseNotSupported, oleObject.EnumAdvise(out var enumAdvise), "IOleObject.EnumAdvise should report no advise support.");
        ContractAssert.Equal(IntPtr.Zero, enumAdvise, "IOleObject.EnumAdvise should return a null enum pointer.");
        ContractAssert.Equal(ComHResults.SOk, oleObject.GetMiscStatus(1, out var miscStatus), "IOleObject.GetMiscStatus should return S_OK.");
        ContractAssert.Equal((uint)OleConstants.OleMiscNone, miscStatus, "IOleObject.GetMiscStatus should return no misc flags.");
        ContractAssert.Equal(ComHResults.SOk, oleObject.SetColorScheme(IntPtr.Zero), "IOleObject.SetColorScheme should no-op successfully.");
        ContractAssert.Equal(ComHResults.SOk, oleObject.Close(0), "IOleObject.Close should no-op successfully.");

        var oleObjectPointer = IntPtr.Zero;
        try
        {
            oleObjectPointer = Marshal.GetComInterfaceForObject(context.UnitOperation, typeof(IOleObject));
            ContractAssert.True(
                oleObjectPointer != IntPtr.Zero,
                "COM QueryInterface for IOleObject should succeed for the unit operation.");
        }
        finally
        {
            if (oleObjectPointer != IntPtr.Zero)
            {
                Marshal.Release(oleObjectPointer);
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
            if (definition.SpecificationType == CapeParamType.CAPE_OPTION)
            {
                var optionSpecification = (ICapeOptionParameterSpec)parameter.Specification;
                ContractAssert.Equal(definition.DefaultValue ?? string.Empty, optionSpecification.DefaultValue, "Option parameter spec default value should be non-null for COM hosts.");
                ContractAssert.False(optionSpecification.RestrictedToList, "MVP option parameter specs should accept unrestricted strings.");
                ContractAssert.SequenceEqual(Array.Empty<string>(), (string[])optionSpecification.OptionList, "Unrestricted option parameter specs should expose an empty option list.");
                var optionValidationMessage = string.Empty;
                ContractAssert.True(optionSpecification.Validate("candidate", ref optionValidationMessage), "Unrestricted option specs should accept arbitrary string candidates.");
            }
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
        var dwsimStyleParameter = context.ParameterCollection.Item(1);
        var dwsimStyleIdentification = (ICapeIdentification)dwsimStyleParameter;
        var dwsimStyleSpecification = (ICapeParameterSpec)dwsimStyleParameter;
        var dwsimStyleOptionSpecification = (ICapeOptionParameterSpec)dwsimStyleParameter;
        var dwsimStyleCapeParameter = (ICapeParameter)dwsimStyleParameter;
        ContractAssert.Equal(
            UnitOperationParameterCatalog.FlowsheetJson.Name,
            dwsimStyleIdentification.ComponentName,
            "DWSIM-style parameter enumeration should read ICapeIdentification from the Item(i) result itself.");
        ContractAssert.Equal(
            UnitOperationParameterCatalog.FlowsheetJson.SpecificationType,
            dwsimStyleSpecification.Type,
            "DWSIM-style parameter enumeration should read ICapeParameterSpec from the Item(i) result itself.");
        ContractAssert.False(
            dwsimStyleOptionSpecification.RestrictedToList,
            "DWSIM-style option parameter inspection should read ICapeOptionParameterSpec from the Item(i) result itself.");
        ContractAssert.Equal(
            UnitOperationParameterCatalog.FlowsheetJson.Mode,
            dwsimStyleCapeParameter.Mode,
            "DWSIM-style parameter enumeration should read ICapeParameter from the Item(i) result itself.");
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
}
