using Microsoft.Win32;

internal sealed record CapeOpenRegistrationDescriptor(
    string ComponentName,
    string Description,
    string ClassId,
    string ProgId,
    string VersionedProgId,
    string TypeLibraryId,
    string TypeLibraryVersion,
    string AssemblyName,
    string TypeName,
    string ResolvedComHostPath,
    string ResolvedTypeLibraryPath,
    CapeOpenRegistrationAction Action,
    CapeOpenRegistrationScope Scope,
    CapeOpenRegistrationExecutionMode ExecutionMode,
    string RequiredConfirmToken,
    IReadOnlyList<CapeOpenRegistrationCategory> Categories,
    IReadOnlyList<CapeOpenImplementedInterface> ImplementedInterfaces,
    IReadOnlyList<CapeOpenPreflightCheck> PreflightChecks,
    IReadOnlyList<CapeOpenRegistryPlanEntry> RegistryPlan,
    IReadOnlyList<CapeOpenRegistryBackupPlanEntry> BackupPlan,
    bool WritesRegistry,
    bool RequiresComRegistration,
    bool RequiresPmeAutomation,
    bool SupportsThirdPartyCapeOpenModels)
{
    public static CapeOpenRegistrationDescriptor CreateUnitOperationMvp(
        CapeOpenRegistrationAction action,
        CapeOpenRegistrationScope scope,
        CapeOpenRegistrationExecutionMode executionMode,
        string? comHostPath,
        string? typeLibraryPath)
    {
        var unitOperationType = typeof(RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.RadishFlowCapeOpenUnitOperation);
        var resolvedComHostPath = CapeOpenComHostPathResolver.Resolve(unitOperationType, comHostPath);
        var resolvedTypeLibraryPath = CapeOpenTypeLibraryPathResolver.Resolve(
            unitOperationType,
            typeLibraryPath,
            resolvedComHostPath);
        var preflightChecks = CapeOpenRegistrationPreflightChecker.Check(
            action,
            scope,
            executionMode,
            resolvedComHostPath,
            resolvedTypeLibraryPath);
        var registryPlan = CapeOpenRegistryPlanBuilder.BuildUnitOperationMvpPlan(
            action,
            scope,
            resolvedComHostPath,
            resolvedTypeLibraryPath);
        var backupPlan = CapeOpenRegistryBackupPlanBuilder.BuildUnitOperationMvpPlan(scope);
        return new CapeOpenRegistrationDescriptor(
            ComponentName: RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.DisplayName,
            Description: RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.Description,
            ClassId: RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ClassId,
            ProgId: RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ProgId,
            VersionedProgId: RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.VersionedProgId,
            TypeLibraryId: RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.TypeLibraryId,
            TypeLibraryVersion: RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.TypeLibraryVersion,
            AssemblyName: unitOperationType.Assembly.GetName().Name ?? "RadishFlow.CapeOpen.UnitOp.Mvp",
            TypeName: unitOperationType.FullName ?? unitOperationType.Name,
            ResolvedComHostPath: resolvedComHostPath,
            ResolvedTypeLibraryPath: resolvedTypeLibraryPath,
            Action: action,
            Scope: scope,
            ExecutionMode: executionMode,
            RequiredConfirmToken: CapeOpenRegistrationConfirmationToken.Create(
                action,
                scope,
                RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation.UnitOperationComIdentity.ClassId),
            Categories:
            [
                new CapeOpenRegistrationCategory(
                    Name: "CAPE-OPEN Object",
                    CategoryId: RadishFlow.CapeOpen.Interop.Guids.CapeOpenCategoryIds.CapeOpenObject),
                new CapeOpenRegistrationCategory(
                    Name: "CAPE-OPEN Unit Operation",
                    CategoryId: RadishFlow.CapeOpen.Interop.Guids.CapeOpenCategoryIds.UnitOperation),
            ],
            ImplementedInterfaces:
            [
                new CapeOpenImplementedInterface(
                    Name: "ICapeIdentification",
                    InterfaceId: RadishFlow.CapeOpen.Interop.Guids.CapeOpenInterfaceIds.ICapeIdentification),
                new CapeOpenImplementedInterface(
                    Name: "ICapeUtilities",
                    InterfaceId: RadishFlow.CapeOpen.Interop.Guids.CapeOpenInterfaceIds.ICapeUtilities),
                new CapeOpenImplementedInterface(
                    Name: "ICapeUnit",
                    InterfaceId: RadishFlow.CapeOpen.Interop.Guids.CapeOpenInterfaceIds.ICapeUnit),
                new CapeOpenImplementedInterface(
                    Name: "ICapeUnitReport",
                    InterfaceId: RadishFlow.CapeOpen.Interop.Guids.CapeOpenInterfaceIds.ICapeUnitReport),
                new CapeOpenImplementedInterface(
                    Name: "IPersistStreamInit",
                    InterfaceId: RadishFlow.CapeOpen.Interop.Persistence.ComPersistenceInterfaceIds.IPersistStreamInit),
            ],
            PreflightChecks: preflightChecks,
            RegistryPlan: registryPlan,
            BackupPlan: backupPlan,
            WritesRegistry: executionMode == CapeOpenRegistrationExecutionMode.Execute,
            RequiresComRegistration: executionMode == CapeOpenRegistrationExecutionMode.Execute,
            RequiresPmeAutomation: false,
            SupportsThirdPartyCapeOpenModels: false);
    }
}

internal sealed record CapeOpenRegistrationCategory(
    string Name,
    string CategoryId);

internal sealed record CapeOpenImplementedInterface(
    string Name,
    string InterfaceId);

internal sealed record CapeOpenPreflightCheck(
    string Name,
    CapeOpenPreflightCheckStatus Status,
    string Detail);

internal sealed record CapeOpenRegistryPlanEntry(
    CapeOpenRegistryPlanOperation Operation,
    string Hive,
    string KeyPath,
    string? ValueName,
    string? ValueData,
    string Reason);

internal sealed record CapeOpenRegistryBackupPlanEntry(
    string Hive,
    string KeyPath,
    bool Exists,
    string Reason);

internal sealed record CapeOpenRegistrationRunResult(
    CapeOpenRegistrationDescriptor Descriptor,
    CapeOpenRegistrationExecutionSummary? ExecutionSummary);

internal sealed record CapeOpenRegistrationExecutionSummary(
    bool Executed,
    bool Succeeded,
    string BackupDirectory,
    string BackupFilePath,
    string ExecutionLogPath,
    bool RollbackAttempted,
    bool RollbackSucceeded,
    string Message,
    IReadOnlyList<CapeOpenRegistryOperationResult> OperationResults);

internal sealed record CapeOpenRegistryOperationResult(
    CapeOpenRegistryPlanOperation Operation,
    string Hive,
    string KeyPath,
    string? ValueName,
    string? ValueData,
    CapeOpenRegistryOperationResultStatus Status,
    string Detail);

internal sealed record CapeOpenRegistryBackupBundle(
    string FormatVersion,
    DateTimeOffset CapturedAt,
    CapeOpenRegistrationAction Action,
    CapeOpenRegistrationScope Scope,
    string ClassId,
    IReadOnlyList<CapeOpenRegistryTreeSnapshot> Trees);

internal sealed record CapeOpenRegistryTreeSnapshot(
    string Hive,
    string KeyPath,
    bool Exists,
    CapeOpenRegistryKeySnapshot? Key);

internal sealed record CapeOpenRegistryKeySnapshot(
    IReadOnlyList<CapeOpenRegistryValueSnapshot> Values,
    IReadOnlyList<CapeOpenRegistryNamedSubKeySnapshot> SubKeys);

internal sealed record CapeOpenRegistryNamedSubKeySnapshot(
    string Name,
    CapeOpenRegistryKeySnapshot Snapshot);

internal sealed record CapeOpenRegistryValueSnapshot(
    string? Name,
    RegistryValueKind Kind,
    string? StringValue,
    int? DWordValue,
    long? QWordValue,
    string[]? MultiStringValue,
    string? BinaryBase64Value);

internal enum CapeOpenRegistrationAction
{
    Register,
    Unregister,
}

internal enum CapeOpenRegistrationScope
{
    CurrentUser,
    LocalMachine,
}

internal enum CapeOpenRegistrationExecutionMode
{
    DryRun,
    Execute,
}

internal enum CapeOpenRegistryPlanOperation
{
    Verify,
    SetValue,
    RegisterTypeLibrary,
    UnregisterTypeLibrary,
    DeleteTree,
}

internal enum CapeOpenPreflightCheckStatus
{
    Pass,
    Warning,
    Fail,
}

internal enum CapeOpenRegistryOperationResultStatus
{
    Applied,
    Skipped,
    Failed,
}

internal enum PortableExecutableMachineArchitecture
{
    Unknown,
    X86,
    X64,
    Arm64,
}
