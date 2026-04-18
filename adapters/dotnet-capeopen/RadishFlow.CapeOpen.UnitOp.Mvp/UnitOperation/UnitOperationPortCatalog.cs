using RadishFlow.CapeOpen.Interop.Unit;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

public static class UnitOperationPortCatalog
{
    public static UnitOperationPortDefinition Feed { get; } = new(
        Name: "Feed",
        Description: "Required inlet material placeholder port.",
        Direction: CapePortDirection.CAPE_INLET,
        PortType: CapePortType.CAPE_MATERIAL,
        IsRequired: true,
        ConnectionOperationName: nameof(RadishFlowCapeOpenUnitOperation.SetPortConnected),
        BoundaryMaterialRole: UnitOperationPortBoundaryMaterialRole.BoundaryInputs);

    public static UnitOperationPortDefinition Product { get; } = new(
        Name: "Product",
        Description: "Required outlet material placeholder port.",
        Direction: CapePortDirection.CAPE_OUTLET,
        PortType: CapePortType.CAPE_MATERIAL,
        IsRequired: true,
        ConnectionOperationName: nameof(RadishFlowCapeOpenUnitOperation.SetPortConnected),
        BoundaryMaterialRole: UnitOperationPortBoundaryMaterialRole.BoundaryOutputs);

    private static readonly IReadOnlyList<UnitOperationPortDefinition> OrderedDefinitionsValue =
        ValidateDefinitions(
        [
        Feed,
        Product,
        ]);
    private static readonly IReadOnlyDictionary<string, UnitOperationPortDefinition> DefinitionsByNameValue =
        OrderedDefinitionsValue.ToDictionary(static definition => definition.Name, StringComparer.OrdinalIgnoreCase);

    public static IReadOnlyList<UnitOperationPortDefinition> OrderedDefinitions => OrderedDefinitionsValue;

    public static IReadOnlyList<string> OrderedNames { get; } = OrderedDefinitionsValue.Select(static definition => definition.Name).ToArray();

    public static bool TryGetByName(string name, out UnitOperationPortDefinition definition)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(name);
        return DefinitionsByNameValue.TryGetValue(name, out definition!);
    }

    public static UnitOperationPortDefinition GetByName(string name)
    {
        if (TryGetByName(name, out var definition))
        {
            return definition;
        }

        throw new ArgumentException($"Unknown unit operation port definition `{name}`.", nameof(name));
    }

    private static IReadOnlyList<UnitOperationPortDefinition> ValidateDefinitions(
        IReadOnlyList<UnitOperationPortDefinition> definitions)
    {
        foreach (var definition in definitions)
        {
            if (string.IsNullOrWhiteSpace(definition.ConnectionOperationName))
            {
                throw new InvalidOperationException(
                    $"Unit operation port definition `{definition.Name}` must declare a non-empty connection operation.");
            }
        }

        return definitions;
    }
}

public sealed record UnitOperationPortDefinition(
    string Name,
    string Description,
    CapePortDirection Direction,
    CapePortType PortType,
    bool IsRequired,
    string ConnectionOperationName,
    UnitOperationPortBoundaryMaterialRole BoundaryMaterialRole);

public enum UnitOperationPortBoundaryMaterialRole
{
    BoundaryInputs,
    BoundaryOutputs,
}
