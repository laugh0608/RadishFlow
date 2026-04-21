using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostValidationRunner
{
    public static UnitOperationHostValidationOutcome Validate(
        RadishFlowCapeOpenUnitOperation unitOperation)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);

        var message = string.Empty;
        var isValid = unitOperation.Validate(ref message);
        var views = UnitOperationHostViewReader.Read(unitOperation);

        return new UnitOperationHostValidationOutcome(
            IsValid: isValid,
            Message: message,
            ValidationStatus: unitOperation.ValStatus,
            Views: views);
    }
}

public sealed record UnitOperationHostValidationOutcome(
    bool IsValid,
    string Message,
    CapeValidationStatus ValidationStatus,
    UnitOperationHostViewSnapshot Views)
{
    public UnitOperationHostFollowUp FollowUp { get; } =
        UnitOperationHostFollowUpPlanner.CreateFromCurrentState(Views.Session);

    public UnitOperationHostConfigurationSnapshot Configuration => Views.Configuration;

    public UnitOperationHostActionPlan ActionPlan => Views.ActionPlan;

    public UnitOperationHostPortMaterialSnapshot PortMaterial => Views.PortMaterial;

    public UnitOperationHostExecutionSnapshot Execution => Views.Execution;

    public UnitOperationHostReportSnapshot Report => Views.Report;

    public UnitOperationHostSessionSnapshot Session => Views.Session;
}
