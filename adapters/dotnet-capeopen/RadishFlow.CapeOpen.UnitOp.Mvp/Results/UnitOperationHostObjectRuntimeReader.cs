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
                ParameterCollection: new UnitOperationHostParameterCollectionRuntime(
                    Name: UnitOperationParameterCatalog.CollectionDefinition.Name,
                    Description: UnitOperationParameterCatalog.CollectionDefinition.Description,
                    Entries: []),
                PortCollection: new UnitOperationHostPortCollectionRuntime(
                    Name: UnitOperationPortCatalog.CollectionDefinition.Name,
                    Description: UnitOperationPortCatalog.CollectionDefinition.Description,
                    Entries: []));
        }

        var parameterEntries = UnitOperationParameterCatalog.OrderedDefinitions
            .Select(definition => CreateParameterEntry(unitOperation, definition))
            .ToArray();
        var portEntries = UnitOperationPortCatalog.OrderedDefinitions
            .Select(definition => CreatePortEntry(unitOperation, definition))
            .ToArray();

        return new UnitOperationHostObjectRuntimeSnapshot(
            LifecycleState: MapLifecycleState(lifecycleState),
            ParameterCollection: new UnitOperationHostParameterCollectionRuntime(
                Name: UnitOperationParameterCatalog.CollectionDefinition.Name,
                Description: UnitOperationParameterCatalog.CollectionDefinition.Description,
                Entries: parameterEntries),
            PortCollection: new UnitOperationHostPortCollectionRuntime(
                Name: UnitOperationPortCatalog.CollectionDefinition.Name,
                Description: UnitOperationPortCatalog.CollectionDefinition.Description,
                Entries: portEntries));
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
    UnitOperationHostParameterCollectionRuntime ParameterCollection,
    UnitOperationHostPortCollectionRuntime PortCollection)
{
    public IReadOnlyList<UnitOperationHostParameterRuntimeEntry> ParameterEntries => ParameterCollection.Entries;

    public IReadOnlyList<UnitOperationHostPortRuntimeEntry> PortEntries => PortCollection.Entries;

    public UnitOperationHostParameterRuntimeEntry GetParameter(string name)
    {
        return ParameterCollection.GetEntry(name);
    }

    public UnitOperationHostPortRuntimeEntry GetPort(string name)
    {
        return PortCollection.GetEntry(name);
    }
}

public sealed record UnitOperationHostParameterCollectionRuntime(
    string Name,
    string Description,
    IReadOnlyList<UnitOperationHostParameterRuntimeEntry> Entries)
{
    public int Count => Entries.Count;

    public UnitOperationHostParameterRuntimeEntry GetEntry(string name)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(name);

        foreach (var entry in Entries)
        {
            if (string.Equals(entry.Name, name, StringComparison.OrdinalIgnoreCase))
            {
                return entry;
            }
        }

        throw new ArgumentException($"Unknown unit operation host runtime parameter `{name}`.", nameof(name));
    }
}

public sealed record UnitOperationHostPortCollectionRuntime(
    string Name,
    string Description,
    IReadOnlyList<UnitOperationHostPortRuntimeEntry> Entries)
{
    public int Count => Entries.Count;

    public UnitOperationHostPortRuntimeEntry GetEntry(string name)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(name);

        foreach (var entry in Entries)
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
