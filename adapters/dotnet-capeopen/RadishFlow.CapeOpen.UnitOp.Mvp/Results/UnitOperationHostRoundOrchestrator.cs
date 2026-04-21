using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostRoundOrchestrator
{
    public static UnitOperationHostRoundOutcome Execute(
        RadishFlowCapeOpenUnitOperation unitOperation)
    {
        return Execute(unitOperation, UnitOperationHostRoundRequest.Default);
    }

    public static UnitOperationHostRoundOutcome Execute(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostRoundRequest request)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);
        ArgumentNullException.ThrowIfNull(request);

        var initialViews = UnitOperationHostViewReader.Read(unitOperation);
        var actionPlan = request.ActionPlan ?? initialViews.ActionPlan;
        var finalViews = initialViews;

        UnitOperationHostActionExecutionOrchestrationResult? actionExecution = null;
        if (request.ExecuteReadyActions)
        {
            actionExecution = UnitOperationHostActionExecutionOrchestrator.ExecutePlannedActions(
                unitOperation,
                actionPlan,
                request.ActionInputSet);
            finalViews = actionExecution.Views;
        }

        UnitOperationHostRoundSupplementalMutationOutcome? supplementalMutations = null;
        if (request.SupplementalMutationCommands.Count > 0)
        {
            var mutationBatch = UnitOperationHostObjectMutationDispatcher.DispatchBatch(
                unitOperation,
                request.SupplementalMutationCommands);
            finalViews = UnitOperationHostViewReader.Read(unitOperation);
            supplementalMutations = new UnitOperationHostRoundSupplementalMutationOutcome(
                Commands: request.SupplementalMutationCommands,
                Batch: mutationBatch,
                Views: finalViews);
        }

        UnitOperationHostValidationOutcome? validation = null;
        if (ShouldRunValidation(request, actionExecution, supplementalMutations))
        {
            validation = UnitOperationHostValidationRunner.Validate(unitOperation);
            finalViews = validation.Views;
        }

        UnitOperationHostCalculationOutcome? calculation = null;
        if (ShouldRunCalculation(request, actionExecution, supplementalMutations, validation))
        {
            calculation = UnitOperationHostCalculationRunner.Calculate(unitOperation);
            finalViews = calculation.Views;
        }

        return new UnitOperationHostRoundOutcome(
            Request: request,
            InitialActionPlan: actionPlan,
            InitialViews: initialViews,
            ActionExecution: actionExecution,
            SupplementalMutations: supplementalMutations,
            Validation: validation,
            Calculation: calculation,
            FinalViews: finalViews);
    }

    private static bool ShouldRunValidation(
        UnitOperationHostRoundRequest request,
        UnitOperationHostActionExecutionOrchestrationResult? actionExecution,
        UnitOperationHostRoundSupplementalMutationOutcome? supplementalMutations)
    {
        if (!request.RunValidation)
        {
            return false;
        }

        return supplementalMutations?.FollowUp.CanValidate ??
               actionExecution?.FollowUp.CanValidate ??
               true;
    }

    private static bool ShouldRunCalculation(
        UnitOperationHostRoundRequest request,
        UnitOperationHostActionExecutionOrchestrationResult? actionExecution,
        UnitOperationHostRoundSupplementalMutationOutcome? supplementalMutations,
        UnitOperationHostValidationOutcome? validation)
    {
        if (!request.RunCalculation)
        {
            return false;
        }

        if (validation is not null)
        {
            if (request.RequireSuccessfulValidationForCalculation && !validation.IsValid)
            {
                return false;
            }

            return validation.FollowUp.CanCalculate;
        }

        if (supplementalMutations is not null)
        {
            if (request.RequireSuccessfulValidationForCalculation)
            {
                return false;
            }

            return supplementalMutations.FollowUp.CanCalculate;
        }

        if (actionExecution is not null)
        {
            if (request.RequireSuccessfulValidationForCalculation)
            {
                return false;
            }

            return actionExecution.FollowUp.CanCalculate;
        }

        return true;
    }
}

public sealed class UnitOperationHostRoundRequest
{
    public static UnitOperationHostRoundRequest Default { get; } = new();

    public UnitOperationHostRoundRequest(
        UnitOperationHostActionExecutionInputSet? actionInputSet = null,
        bool executeReadyActions = true,
        bool runValidation = false,
        bool runCalculation = false,
        bool requireSuccessfulValidationForCalculation = true,
        IReadOnlyList<UnitOperationHostObjectMutationCommand>? supplementalMutationCommands = null,
        UnitOperationHostActionPlan? actionPlan = null)
    {
        ActionInputSet = actionInputSet ?? UnitOperationHostActionExecutionInputSet.Empty;
        ExecuteReadyActions = executeReadyActions;
        RunValidation = runValidation;
        RunCalculation = runCalculation;
        RequireSuccessfulValidationForCalculation = requireSuccessfulValidationForCalculation;
        SupplementalMutationCommands = CopySupplementalMutationCommands(supplementalMutationCommands);
        ActionPlan = actionPlan;
    }

    public UnitOperationHostActionExecutionInputSet ActionInputSet { get; }

    public bool ExecuteReadyActions { get; }

    public bool RunValidation { get; }

    public bool RunCalculation { get; }

    public bool RequireSuccessfulValidationForCalculation { get; }

    public IReadOnlyList<UnitOperationHostObjectMutationCommand> SupplementalMutationCommands { get; }

    public UnitOperationHostActionPlan? ActionPlan { get; }

    private static IReadOnlyList<UnitOperationHostObjectMutationCommand> CopySupplementalMutationCommands(
        IReadOnlyList<UnitOperationHostObjectMutationCommand>? commands)
    {
        if (commands is null || commands.Count == 0)
        {
            return [];
        }

        return commands.Select(static command =>
        {
            ArgumentNullException.ThrowIfNull(command);
            return command;
        }).ToArray();
    }
}

public sealed record UnitOperationHostRoundOutcome(
    UnitOperationHostRoundRequest Request,
    UnitOperationHostActionPlan InitialActionPlan,
    UnitOperationHostViewSnapshot InitialViews,
    UnitOperationHostActionExecutionOrchestrationResult? ActionExecution,
    UnitOperationHostRoundSupplementalMutationOutcome? SupplementalMutations,
    UnitOperationHostValidationOutcome? Validation,
    UnitOperationHostCalculationOutcome? Calculation,
    UnitOperationHostViewSnapshot FinalViews)
{
    public UnitOperationHostFollowUp FollowUp { get; } = CreateFollowUp(
        ActionExecution,
        SupplementalMutations,
        Validation,
        Calculation,
        FinalViews);

    public UnitOperationHostRoundStopKind StopKind { get; } =
        CreateStopKind(ActionExecution, SupplementalMutations, Validation, Calculation, FinalViews);

    public UnitOperationHostConfigurationSnapshot Configuration => FinalViews.Configuration;

    public UnitOperationHostActionPlan ActionPlan => FinalViews.ActionPlan;

    public UnitOperationHostPortMaterialSnapshot PortMaterial => FinalViews.PortMaterial;

    public UnitOperationHostExecutionSnapshot Execution => FinalViews.Execution;

    public UnitOperationHostReportSnapshot Report => FinalViews.Report;

    public UnitOperationHostSessionSnapshot Session => FinalViews.Session;

    public bool ExecutedActions => ActionExecution is not null;

    public bool ExecutedSupplementalMutations => SupplementalMutations is not null;

    public bool ExecutedValidation => Validation is not null;

    public bool ExecutedCalculation => Calculation is not null;

    public bool Completed => StopKind == UnitOperationHostRoundStopKind.Completed;

    private static UnitOperationHostRoundStopKind CreateStopKind(
        UnitOperationHostActionExecutionOrchestrationResult? actionExecution,
        UnitOperationHostRoundSupplementalMutationOutcome? supplementalMutations,
        UnitOperationHostValidationOutcome? validation,
        UnitOperationHostCalculationOutcome? calculation,
        UnitOperationHostViewSnapshot finalViews)
    {
        ArgumentNullException.ThrowIfNull(finalViews);

        if (finalViews.Configuration.State == UnitOperationHostConfigurationState.Terminated ||
            finalViews.Session.State == UnitOperationHostSessionState.Terminated)
        {
            return UnitOperationHostRoundStopKind.Terminated;
        }

        if (calculation is not null)
        {
            return calculation.Succeeded
                ? UnitOperationHostRoundStopKind.Completed
                : ClassifyFromFollowUp(calculation.FollowUp, UnitOperationHostRoundStopKind.CalculationFailed);
        }

        if (validation is not null)
        {
            return validation.IsValid
                ? UnitOperationHostRoundStopKind.Completed
                : ClassifyFromFollowUp(validation.FollowUp, UnitOperationHostRoundStopKind.ValidationFailed);
        }

        if (supplementalMutations is not null)
        {
            return ClassifyFromFollowUp(supplementalMutations.FollowUp, UnitOperationHostRoundStopKind.Completed);
        }

        if (actionExecution is not null)
        {
            if (actionExecution.HasLifecycleOperations)
            {
                return UnitOperationHostRoundStopKind.LifecycleOperationRequired;
            }

            if (actionExecution.HasMissingInputs)
            {
                return UnitOperationHostRoundStopKind.MissingInputs;
            }

            return ClassifyFromFollowUp(actionExecution.FollowUp, UnitOperationHostRoundStopKind.Completed);
        }

        return ClassifyFromFollowUp(
            UnitOperationHostFollowUpPlanner.CreateFromCurrentState(finalViews.Session),
            UnitOperationHostRoundStopKind.Completed);
    }

    private static UnitOperationHostFollowUp CreateFollowUp(
        UnitOperationHostActionExecutionOrchestrationResult? actionExecution,
        UnitOperationHostRoundSupplementalMutationOutcome? supplementalMutations,
        UnitOperationHostValidationOutcome? validation,
        UnitOperationHostCalculationOutcome? calculation,
        UnitOperationHostViewSnapshot finalViews)
    {
        ArgumentNullException.ThrowIfNull(finalViews);

        return calculation?.FollowUp ??
               validation?.FollowUp ??
               supplementalMutations?.FollowUp ??
               actionExecution?.FollowUp ??
               UnitOperationHostFollowUpPlanner.CreateFromCurrentState(finalViews.Session);
    }

    private static UnitOperationHostRoundStopKind ClassifyFromFollowUp(
        UnitOperationHostFollowUp followUp,
        UnitOperationHostRoundStopKind fallback)
    {
        ArgumentNullException.ThrowIfNull(followUp);

        return followUp.Kind switch
        {
            UnitOperationHostFollowUpKind.LifecycleOperation => UnitOperationHostRoundStopKind.LifecycleOperationRequired,
            UnitOperationHostFollowUpKind.ProvideInputs => UnitOperationHostRoundStopKind.MissingInputs,
            UnitOperationHostFollowUpKind.Terminated => UnitOperationHostRoundStopKind.Terminated,
            _ => fallback,
        };
    }
}

public sealed record UnitOperationHostRoundSupplementalMutationOutcome(
    IReadOnlyList<UnitOperationHostObjectMutationCommand> Commands,
    UnitOperationHostObjectMutationBatchResult Batch,
    UnitOperationHostViewSnapshot Views)
{
    public UnitOperationHostFollowUp FollowUp { get; } =
        UnitOperationHostFollowUpPlanner.CreateFromMutationBatch(Batch, Views);

    public UnitOperationHostConfigurationSnapshot Configuration => Views.Configuration;

    public UnitOperationHostActionPlan ActionPlan => Views.ActionPlan;

    public UnitOperationHostPortMaterialSnapshot PortMaterial => Views.PortMaterial;

    public UnitOperationHostExecutionSnapshot Execution => Views.Execution;

    public UnitOperationHostReportSnapshot Report => Views.Report;

    public UnitOperationHostSessionSnapshot Session => Views.Session;
}

public enum UnitOperationHostRoundStopKind
{
    Completed,
    LifecycleOperationRequired,
    MissingInputs,
    ValidationFailed,
    CalculationFailed,
    Terminated,
}
