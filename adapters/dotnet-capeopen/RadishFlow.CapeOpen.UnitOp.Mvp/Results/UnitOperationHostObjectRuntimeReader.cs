using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostObjectRuntimeReader
{
    public static UnitOperationHostObjectRuntimeSnapshot Read(
        RadishFlowCapeOpenUnitOperation unitOperation)
    {
        ArgumentNullException.ThrowIfNull(unitOperation);

        var lifecycleState = unitOperation.HostLifecycleState;
        if (lifecycleState == UnitOperationLifecycleState.Disposed)
        {
            throw new ObjectDisposedException(unitOperation.GetType().FullName);
        }

        if (lifecycleState == UnitOperationLifecycleState.Terminated)
        {
            return new UnitOperationHostObjectRuntimeSnapshot(
                LifecycleState: UnitOperationHostObjectRuntimeState.Terminated,
                ParameterEntries: [],
                PortEntries: []);
        }

        var parameterEntries = UnitOperationParameterCatalog.OrderedDefinitions
            .Select(definition => CreateParameterEntry(unitOperation, definition))
            .ToArray();
        var portEntries = UnitOperationPortCatalog.OrderedDefinitions
            .Select(definition => CreatePortEntry(unitOperation, definition))
            .ToArray();

        return new UnitOperationHostObjectRuntimeSnapshot(
            LifecycleState: MapLifecycleState(lifecycleState),
            ParameterEntries: parameterEntries,
            PortEntries: portEntries);
    }

    private static UnitOperationHostObjectRuntimeState MapLifecycleState(UnitOperationLifecycleState lifecycleState)
    {
        return lifecycleState switch
        {
            UnitOperationLifecycleState.Constructed => UnitOperationHostObjectRuntimeState.Constructed,
            UnitOperationLifecycleState.Initialized => UnitOperationHostObjectRuntimeState.Initialized,
            UnitOperationLifecycleState.Terminated => UnitOperationHostObjectRuntimeState.Terminated,
            UnitOperationLifecycleState.Disposed => throw new ObjectDisposedException(typeof(RadishFlowCapeOpenUnitOperation).FullName),
            _ => throw new ArgumentOutOfRangeException(nameof(lifecycleState), lifecycleState, "Unknown unit operation lifecycle state."),
        };
    }

    private static UnitOperationHostParameterRuntimeEntry CreateParameterEntry(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationParameterDefinition definition)
    {
        var parameter = unitOperation.Parameters.GetByName(definition.Name);
        return new UnitOperationHostParameterRuntimeEntry(
            Name: definition.Name,
            Description: definition.Description,
            IsRequired: definition.IsRequired,
            IsConfigured: parameter.IsConfigured,
            ValueKind: definition.ValueKind,
            RequiredCompanionParameterName: definition.RequiredCompanionParameterName,
            ConfigurationOperationName: definition.ConfigurationOperationName,
            Mode: definition.Mode,
            DefaultValue: definition.DefaultValue,
            SpecificationType: definition.SpecificationType,
            SpecificationDimensionality: [.. definition.SpecificationDimensionality]);
    }

    private static UnitOperationHostPortRuntimeEntry CreatePortEntry(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationPortDefinition definition)
    {
        var port = unitOperation.Ports.GetByName(definition.Name);
        return new UnitOperationHostPortRuntimeEntry(
            Name: definition.Name,
            Description: definition.Description,
            IsRequired: definition.IsRequired,
            IsConnected: port.IsConnected,
            Direction: definition.Direction,
            PortType: definition.PortType,
            ConnectionOperationName: definition.ConnectionOperationName,
            BoundaryMaterialRole: definition.BoundaryMaterialRole);
    }
}

public sealed record UnitOperationHostObjectRuntimeSnapshot(
    UnitOperationHostObjectRuntimeState LifecycleState,
    IReadOnlyList<UnitOperationHostParameterRuntimeEntry> ParameterEntries,
    IReadOnlyList<UnitOperationHostPortRuntimeEntry> PortEntries)
{
    public UnitOperationHostParameterRuntimeEntry GetParameter(string name)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(name);

        foreach (var entry in ParameterEntries)
        {
            if (string.Equals(entry.Name, name, StringComparison.OrdinalIgnoreCase))
            {
                return entry;
            }
        }

        throw new ArgumentException($"Unknown unit operation host runtime parameter `{name}`.", nameof(name));
    }

    public UnitOperationHostPortRuntimeEntry GetPort(string name)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(name);

        foreach (var entry in PortEntries)
        {
            if (string.Equals(entry.Name, name, StringComparison.OrdinalIgnoreCase))
            {
                return entry;
            }
        }

        throw new ArgumentException($"Unknown unit operation host runtime port `{name}`.", nameof(name));
    }
}

public sealed record UnitOperationHostParameterRuntimeEntry(
    string Name,
    string Description,
    bool IsRequired,
    bool IsConfigured,
    UnitOperationParameterValueKind ValueKind,
    string? RequiredCompanionParameterName,
    string ConfigurationOperationName,
    CapeParamMode Mode,
    string? DefaultValue,
    CapeParamType SpecificationType,
    IReadOnlyList<double> SpecificationDimensionality);

public sealed record UnitOperationHostPortRuntimeEntry(
    string Name,
    string Description,
    bool IsRequired,
    bool IsConnected,
    CapePortDirection Direction,
    CapePortType PortType,
    string ConnectionOperationName,
    UnitOperationPortBoundaryMaterialRole BoundaryMaterialRole);

public enum UnitOperationHostObjectRuntimeState
{
    Constructed,
    Initialized,
    Terminated,
}
