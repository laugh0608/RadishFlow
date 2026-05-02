using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostObjectMutationDispatcher
{
    public static UnitOperationHostObjectMutationOutcome SetParameterValue(
        RadishFlowCapeOpenUnitOperation unitOperation,
        string parameterName,
        string? value)
    {
        return Dispatch(
            unitOperation,
            UnitOperationHostObjectMutationCommand.SetParameterValue(parameterName, value));
    }

    public static UnitOperationHostObjectMutationOutcome ResetParameter(
        RadishFlowCapeOpenUnitOperation unitOperation,
        string parameterName)
    {
        return Dispatch(
            unitOperation,
            UnitOperationHostObjectMutationCommand.ResetParameter(parameterName));
    }

    public static UnitOperationHostObjectMutationOutcome ConnectPort(
        RadishFlowCapeOpenUnitOperation unitOperation,
        string portName,
        object objectToConnect)
    {
        return Dispatch(
            unitOperation,
            UnitOperationHostObjectMutationCommand.ConnectPort(portName, objectToConnect));
    }

    public static UnitOperationHostObjectMutationOutcome DisconnectPort(
        RadishFlowCapeOpenUnitOperation unitOperation,
        string portName)
    {
        return Dispatch(
            unitOperation,
            UnitOperationHostObjectMutationCommand.DisconnectPort(portName));
    }

    public static UnitOperationHostObjectMutationOutcome Dispatch(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostObjectMutationCommand command)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);
        ArgumentNullException.ThrowIfNull(command);

        return command.Kind switch
        {
            UnitOperationHostObjectMutationKind.SetParameterValue => ApplySetParameterValue(unitOperation, command),
            UnitOperationHostObjectMutationKind.ResetParameter => ApplyResetParameter(unitOperation, command),
            UnitOperationHostObjectMutationKind.ConnectPort => ApplyConnectPort(unitOperation, command),
            UnitOperationHostObjectMutationKind.DisconnectPort => ApplyDisconnectPort(unitOperation, command),
            _ => throw new ArgumentOutOfRangeException(nameof(command), command.Kind, "Unknown object mutation command kind."),
        };
    }

    public static UnitOperationHostObjectMutationBatchResult DispatchBatch(
        RadishFlowCapeOpenUnitOperation unitOperation,
        IReadOnlyList<UnitOperationHostObjectMutationCommand> commands)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);
        ArgumentNullException.ThrowIfNull(commands);

        var outcomes = new List<UnitOperationHostObjectMutationOutcome>(commands.Count);
        var invalidatedValidation = false;
        var invalidatedReport = false;

        foreach (var command in commands)
        {
            var outcome = Dispatch(unitOperation, command);
            outcomes.Add(outcome);
            invalidatedValidation |= outcome.InvalidatesValidation;
            invalidatedReport |= outcome.InvalidatesCalculationReport;
        }

        return new UnitOperationHostObjectMutationBatchResult(
            AppliedCount: outcomes.Count,
            Outcomes: outcomes,
            InvalidatedValidation: invalidatedValidation,
            InvalidatedCalculationReport: invalidatedReport);
    }

    private static UnitOperationHostObjectMutationOutcome ApplySetParameterValue(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostObjectMutationCommand command)
    {
        var parameter = unitOperation.Parameters.GetByName(command.TargetName);
        parameter.SetValue((string?)command.Payload);
        return CreateOutcome(command.Kind, UnitOperationHostActionTargetKind.Parameter, parameter.ComponentName);
    }

    private static UnitOperationHostObjectMutationOutcome ApplyResetParameter(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostObjectMutationCommand command)
    {
        var parameter = unitOperation.Parameters.GetByName(command.TargetName);
        parameter.Reset();
        return CreateOutcome(command.Kind, UnitOperationHostActionTargetKind.Parameter, parameter.ComponentName);
    }

    private static UnitOperationHostObjectMutationOutcome ApplyConnectPort(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostObjectMutationCommand command)
    {
        var port = unitOperation.Ports.GetByName(command.TargetName);
        port.Connect(command.Payload!);
        return CreateOutcome(command.Kind, UnitOperationHostActionTargetKind.Port, port.ComponentName);
    }

    private static UnitOperationHostObjectMutationOutcome ApplyDisconnectPort(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationHostObjectMutationCommand command)
    {
        var port = unitOperation.Ports.GetByName(command.TargetName);
        port.Disconnect();
        return CreateOutcome(command.Kind, UnitOperationHostActionTargetKind.Port, port.ComponentName);
    }

    private static UnitOperationHostObjectMutationOutcome CreateOutcome(
        UnitOperationHostObjectMutationKind operation,
        UnitOperationHostActionTargetKind targetKind,
        string targetName)
    {
        return new UnitOperationHostObjectMutationOutcome(
            Succeeded: true,
            Operation: operation,
            Target: new UnitOperationHostActionTarget(targetKind, [targetName]),
            InvalidatesValidation: true,
            InvalidatesCalculationReport: true);
    }
}

public sealed record UnitOperationHostObjectMutationCommand(
    UnitOperationHostObjectMutationKind Kind,
    string TargetName,
    object? Payload)
{
    public static UnitOperationHostObjectMutationCommand SetParameterValue(string parameterName, string? value)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(parameterName);
        return new UnitOperationHostObjectMutationCommand(
            UnitOperationHostObjectMutationKind.SetParameterValue,
            parameterName,
            value);
    }

    public static UnitOperationHostObjectMutationCommand ResetParameter(string parameterName)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(parameterName);
        return new UnitOperationHostObjectMutationCommand(
            UnitOperationHostObjectMutationKind.ResetParameter,
            parameterName,
            null);
    }

    public static UnitOperationHostObjectMutationCommand ConnectPort(string portName, object objectToConnect)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(portName);
        ArgumentNullException.ThrowIfNull(objectToConnect);
        return new UnitOperationHostObjectMutationCommand(
            UnitOperationHostObjectMutationKind.ConnectPort,
            portName,
            objectToConnect);
    }

    public static UnitOperationHostObjectMutationCommand DisconnectPort(string portName)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(portName);
        return new UnitOperationHostObjectMutationCommand(
            UnitOperationHostObjectMutationKind.DisconnectPort,
            portName,
            null);
    }
}

public sealed record UnitOperationHostObjectMutationOutcome(
    bool Succeeded,
    UnitOperationHostObjectMutationKind Operation,
    UnitOperationHostActionTarget Target,
    bool InvalidatesValidation,
    bool InvalidatesCalculationReport);

public sealed record UnitOperationHostObjectMutationBatchResult(
    int AppliedCount,
    IReadOnlyList<UnitOperationHostObjectMutationOutcome> Outcomes,
    bool InvalidatedValidation,
    bool InvalidatedCalculationReport);

public enum UnitOperationHostObjectMutationKind
{
    SetParameterValue,
    ResetParameter,
    ConnectPort,
    DisconnectPort,
}
