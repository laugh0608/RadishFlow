using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostObjectDefinitionReader
{
    private static readonly UnitOperationHostParameterCapabilities ParameterCapabilitiesValue = new(
        CanWriteValue: true,
        CanResetValue: true,
        CanMutateMode: false,
        CanMutateIdentity: false);

    private static readonly UnitOperationHostPortCapabilities PortCapabilitiesValue = new(
        CanConnect: true,
        CanDisconnect: true,
        CanReplaceConnectionWithoutDisconnect: false,
        CanMutateIdentity: false);

    public static UnitOperationHostParameterCapabilities ParameterCapabilities => ParameterCapabilitiesValue;

    public static UnitOperationHostPortCapabilities PortCapabilities => PortCapabilitiesValue;

    public static UnitOperationHostObjectDefinitionSnapshot Read()
    {
        var parameterEntries = UnitOperationParameterCatalog.OrderedDefinitions
            .Select(static definition => CreateParameterEntry(definition))
            .ToArray();
        var portEntries = UnitOperationPortCatalog.OrderedDefinitions
            .Select(static definition => CreatePortEntry(definition))
            .ToArray();

        return new UnitOperationHostObjectDefinitionSnapshot(
            ParameterCollection: new UnitOperationHostParameterCollectionDefinition(
                Name: UnitOperationParameterCatalog.CollectionDefinition.Name,
                Description: UnitOperationParameterCatalog.CollectionDefinition.Description,
                Entries: parameterEntries),
            PortCollection: new UnitOperationHostPortCollectionDefinition(
                Name: UnitOperationPortCatalog.CollectionDefinition.Name,
                Description: UnitOperationPortCatalog.CollectionDefinition.Description,
                Entries: portEntries));
    }

    private static UnitOperationHostParameterDefinitionEntry CreateParameterEntry(UnitOperationParameterDefinition definition)
    {
        return new UnitOperationHostParameterDefinitionEntry(
            Name: definition.Name,
            Description: definition.Description,
            IsRequired: definition.IsRequired,
            ValueKind: definition.ValueKind,
            AllowsEmptyValue: definition.AllowsEmptyValue,
            RequiredCompanionParameterName: definition.RequiredCompanionParameterName,
            ConfigurationOperationName: definition.ConfigurationOperationName,
            Mode: definition.Mode,
            DefaultValue: definition.DefaultValue,
            SpecificationType: definition.SpecificationType,
            SpecificationDimensionality: [.. definition.SpecificationDimensionality],
            Capabilities: ParameterCapabilitiesValue);
    }

    private static UnitOperationHostPortDefinitionEntry CreatePortEntry(UnitOperationPortDefinition definition)
    {
        return new UnitOperationHostPortDefinitionEntry(
            Name: definition.Name,
            Description: definition.Description,
            IsRequired: definition.IsRequired,
            Direction: definition.Direction,
            PortType: definition.PortType,
            ConnectionOperationName: definition.ConnectionOperationName,
            BoundaryMaterialRole: definition.BoundaryMaterialRole,
            Capabilities: PortCapabilitiesValue);
    }
}

public sealed record UnitOperationHostObjectDefinitionSnapshot(
    UnitOperationHostParameterCollectionDefinition ParameterCollection,
    UnitOperationHostPortCollectionDefinition PortCollection)
{
    public IReadOnlyList<UnitOperationHostParameterDefinitionEntry> ParameterEntries => ParameterCollection.Entries;

    public IReadOnlyList<UnitOperationHostPortDefinitionEntry> PortEntries => PortCollection.Entries;

    public UnitOperationHostParameterDefinitionEntry GetParameter(string name)
    {
        return ParameterCollection.GetEntry(name);
    }

    public UnitOperationHostPortDefinitionEntry GetPort(string name)
    {
        return PortCollection.GetEntry(name);
    }
}

public sealed record UnitOperationHostParameterCollectionDefinition(
    string Name,
    string Description,
    IReadOnlyList<UnitOperationHostParameterDefinitionEntry> Entries)
{
    public int Count => Entries.Count;

    public UnitOperationHostParameterDefinitionEntry GetEntry(string name)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(name);

        foreach (var entry in Entries)
        {
            if (string.Equals(entry.Name, name, StringComparison.OrdinalIgnoreCase))
            {
                return entry;
            }
        }

        throw new ArgumentException($"Unknown unit operation host parameter definition `{name}`.", nameof(name));
    }
}

public sealed record UnitOperationHostPortCollectionDefinition(
    string Name,
    string Description,
    IReadOnlyList<UnitOperationHostPortDefinitionEntry> Entries)
{
    public int Count => Entries.Count;

    public UnitOperationHostPortDefinitionEntry GetEntry(string name)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(name);

        foreach (var entry in Entries)
        {
            if (string.Equals(entry.Name, name, StringComparison.OrdinalIgnoreCase))
            {
                return entry;
            }
        }

        throw new ArgumentException($"Unknown unit operation host port definition `{name}`.", nameof(name));
    }
}

public sealed record UnitOperationHostParameterDefinitionEntry(
    string Name,
    string Description,
    bool IsRequired,
    UnitOperationParameterValueKind ValueKind,
    bool AllowsEmptyValue,
    string? RequiredCompanionParameterName,
    string ConfigurationOperationName,
    CapeParamMode Mode,
    string? DefaultValue,
    CapeParamType SpecificationType,
    IReadOnlyList<double> SpecificationDimensionality,
    UnitOperationHostParameterCapabilities Capabilities);

public sealed record UnitOperationHostPortDefinitionEntry(
    string Name,
    string Description,
    bool IsRequired,
    CapePortDirection Direction,
    CapePortType PortType,
    string ConnectionOperationName,
    UnitOperationPortBoundaryMaterialRole BoundaryMaterialRole,
    UnitOperationHostPortCapabilities Capabilities);

public sealed record UnitOperationHostParameterCapabilities(
    bool CanWriteValue,
    bool CanResetValue,
    bool CanMutateMode,
    bool CanMutateIdentity);

public sealed record UnitOperationHostPortCapabilities(
    bool CanConnect,
    bool CanDisconnect,
    bool CanReplaceConnectionWithoutDisconnect,
    bool CanMutateIdentity);
