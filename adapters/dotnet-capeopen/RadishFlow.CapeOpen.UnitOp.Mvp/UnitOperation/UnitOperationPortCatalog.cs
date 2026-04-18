using RadishFlow.CapeOpen.Interop.Unit;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

public static class UnitOperationPortCatalog
{
    public static UnitOperationPortDefinition Feed { get; } = new(
        Name: "Feed",
        Description: "Required inlet material placeholder port.",
        Direction: CapePortDirection.CAPE_INLET,
        PortType: CapePortType.CAPE_MATERIAL,
        IsRequired: true);

    public static UnitOperationPortDefinition Product { get; } = new(
        Name: "Product",
        Description: "Required outlet material placeholder port.",
        Direction: CapePortDirection.CAPE_OUTLET,
        PortType: CapePortType.CAPE_MATERIAL,
        IsRequired: true);

    private static readonly IReadOnlyList<UnitOperationPortDefinition> OrderedDefinitionsValue =
    [
        Feed,
        Product,
    ];

    public static IReadOnlyList<UnitOperationPortDefinition> OrderedDefinitions => OrderedDefinitionsValue;

    public static IReadOnlyList<string> OrderedNames { get; } = OrderedDefinitionsValue.Select(static definition => definition.Name).ToArray();
}

public sealed record UnitOperationPortDefinition(
    string Name,
    string Description,
    CapePortDirection Direction,
    CapePortType PortType,
    bool IsRequired);
