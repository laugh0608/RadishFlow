using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostCalculationRunner
{
    public static UnitOperationHostCalculationOutcome Calculate(
        RadishFlowCapeOpenUnitOperation unitOperation)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);

        try
        {
            unitOperation.Calculate();
            return CreateOutcome(unitOperation, null);
        }
        catch (CapeOpenException error)
        {
            return CreateOutcome(unitOperation, error);
        }
    }

    private static UnitOperationHostCalculationOutcome CreateOutcome(
        RadishFlowCapeOpenUnitOperation unitOperation,
        CapeOpenException? failure)
    {
        var views = UnitOperationHostViewReader.Read(unitOperation);
        return new UnitOperationHostCalculationOutcome(
            Succeeded: failure is null,
            Failure: failure,
            Views: views);
    }
}

public sealed record UnitOperationHostCalculationOutcome(
    bool Succeeded,
    CapeOpenException? Failure,
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
