using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostViewReader
{
    public static UnitOperationHostViewSnapshot Read(
        RadishFlowCapeOpenUnitOperation unitOperation)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);

        var configuration = UnitOperationHostConfigurationReader.Read(unitOperation);
        var actionPlan = UnitOperationHostActionPlanReader.Read(configuration);
        var portMaterial = UnitOperationHostPortMaterialReader.Read(unitOperation);
        var execution = UnitOperationHostExecutionReader.Read(unitOperation);
        var report = UnitOperationHostReportReader.Read(unitOperation);
        var session = UnitOperationHostSessionReader.CreateSnapshot(
            configuration,
            actionPlan,
            portMaterial,
            execution,
            report);

        return new UnitOperationHostViewSnapshot(
            Configuration: configuration,
            ActionPlan: actionPlan,
            PortMaterial: portMaterial,
            Execution: execution,
            Report: report,
            Session: session);
    }
}

public sealed record UnitOperationHostViewSnapshot(
    UnitOperationHostConfigurationSnapshot Configuration,
    UnitOperationHostActionPlan ActionPlan,
    UnitOperationHostPortMaterialSnapshot PortMaterial,
    UnitOperationHostExecutionSnapshot Execution,
    UnitOperationHostReportSnapshot Report,
    UnitOperationHostSessionSnapshot Session)
{
    public string Headline => Session.Headline;
}
