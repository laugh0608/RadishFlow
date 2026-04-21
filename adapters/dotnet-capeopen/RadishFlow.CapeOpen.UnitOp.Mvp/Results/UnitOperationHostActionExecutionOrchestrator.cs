using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostActionExecutionOrchestrator
{
    public static UnitOperationHostActionExecutionOrchestrationResult ExecutePlannedActions(
        RadishFlowCapeOpenUnitOperation unitOperation)
    {
        return ExecutePlannedActions(unitOperation, UnitOperationHostActionExecutionInputSet.Empty);
    }

    public static UnitOperationHostActionExecutionOrchestrationResult ExecutePlannedActions(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostActionExecutionInputSet inputSet)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);
        ArgumentNullException.ThrowIfNull(inputSet);

        var actionPlan = UnitOperationHostActionPlanReader.Read(unitOperation);
        return ExecutePlannedActions(unitOperation, actionPlan, inputSet);
    }

    public static UnitOperationHostActionExecutionOrchestrationResult ExecutePlannedActions(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostActionPlan actionPlan,
        UnitOperationHostActionExecutionInputSet inputSet)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);
        ArgumentNullException.ThrowIfNull(actionPlan);
        ArgumentNullException.ThrowIfNull(inputSet);

        var requestPlan = UnitOperationHostActionExecutionRequestPlanner.Plan(actionPlan, inputSet);
        var execution = UnitOperationHostActionExecutionDispatcher.ApplyActionBatch(unitOperation, requestPlan.Requests);
        return CreateResult(unitOperation, actionPlan, requestPlan, execution);
    }

    private static UnitOperationHostActionExecutionOrchestrationResult CreateResult(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostActionPlan initialActionPlan,
        UnitOperationHostActionExecutionRequestPlan requestPlan,
        UnitOperationHostActionExecutionBatchResult execution)
    {
        var views = UnitOperationHostViewReader.Read(unitOperation);

        return new UnitOperationHostActionExecutionOrchestrationResult(
            InitialActionPlan: initialActionPlan,
            RequestPlan: requestPlan,
            Execution: execution,
            Views: views);
    }
}

public sealed record UnitOperationHostActionExecutionOrchestrationResult(
    UnitOperationHostActionPlan InitialActionPlan,
    UnitOperationHostActionExecutionRequestPlan RequestPlan,
    UnitOperationHostActionExecutionBatchResult Execution,
    UnitOperationHostViewSnapshot Views)
{
    public UnitOperationHostFollowUp FollowUp { get; } =
        UnitOperationHostFollowUpPlanner.CreateFromActionExecution(RequestPlan, Execution, Views);

    public UnitOperationHostConfigurationSnapshot Configuration => Views.Configuration;

    public UnitOperationHostActionPlan ActionPlan => Views.ActionPlan;

    public UnitOperationHostPortMaterialSnapshot PortMaterial => Views.PortMaterial;

    public UnitOperationHostExecutionSnapshot ExecutionSnapshot => Views.Execution;

    public UnitOperationHostReportSnapshot Report => Views.Report;

    public UnitOperationHostSessionSnapshot Session => Views.Session;

    public int PlannedActionCount => RequestPlan.EntryCount;

    public int ReadyRequestCount => RequestPlan.RequestCount;

    public int MissingInputCount => RequestPlan.MissingInputCount;

    public bool HasMissingInputs => RequestPlan.HasMissingInputs;

    public bool HasLifecycleOperations => RequestPlan.HasLifecycleOperations || Execution.HasLifecycleOperations;

    public bool HasUnsupportedActions => RequestPlan.HasUnsupportedActions || Execution.HasUnsupportedActions;

    public bool AppliedMutations => Execution.AppliedMutationCount > 0;

    public bool RequiresValidationRefresh => Execution.InvalidatedValidation;

    public bool RequiresCalculationRefresh =>
        Execution.InvalidatedCalculationReport || Session.Summary.RequiresCalculateRefresh;
}
