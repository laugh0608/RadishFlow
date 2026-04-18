using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

public static class UnitOperationParameterCatalog
{
    public static UnitOperationParameterDefinition FlowsheetJson { get; } = new(
        Name: "Flowsheet Json",
        Description: "StoredProjectFile JSON used by the MVP unit operation skeleton.",
        IsRequired: true,
        ValueKind: UnitOperationParameterValueKind.StructuredJsonText,
        AllowsEmptyValue: false,
        RequiredCompanionParameterName: null,
        Mode: CapeParamMode.CAPE_INPUT,
        DefaultValue: null);

    public static UnitOperationParameterDefinition PropertyPackageId { get; } = new(
        Name: "Property Package Id",
        Description: "Identifier of the property package selected for the MVP unit operation skeleton.",
        IsRequired: true,
        ValueKind: UnitOperationParameterValueKind.Identifier,
        AllowsEmptyValue: false,
        RequiredCompanionParameterName: null,
        Mode: CapeParamMode.CAPE_INPUT,
        DefaultValue: null);

    public static UnitOperationParameterDefinition PropertyPackageManifestPath { get; } = new(
        Name: "Property Package Manifest Path",
        Description: "Optional manifest path for a local property package payload.",
        IsRequired: false,
        ValueKind: UnitOperationParameterValueKind.FilePath,
        AllowsEmptyValue: false,
        RequiredCompanionParameterName: "Property Package Payload Path",
        Mode: CapeParamMode.CAPE_INPUT,
        DefaultValue: null);

    public static UnitOperationParameterDefinition PropertyPackagePayloadPath { get; } = new(
        Name: "Property Package Payload Path",
        Description: "Optional payload path for a local property package payload.",
        IsRequired: false,
        ValueKind: UnitOperationParameterValueKind.FilePath,
        AllowsEmptyValue: false,
        RequiredCompanionParameterName: "Property Package Manifest Path",
        Mode: CapeParamMode.CAPE_INPUT,
        DefaultValue: null);

    private static readonly IReadOnlyList<UnitOperationParameterDefinition> OrderedDefinitionsValue =
    [
        FlowsheetJson,
        PropertyPackageId,
        PropertyPackageManifestPath,
        PropertyPackagePayloadPath,
    ];
    private static readonly IReadOnlyDictionary<string, UnitOperationParameterDefinition> DefinitionsByNameValue =
        OrderedDefinitionsValue.ToDictionary(static definition => definition.Name, StringComparer.OrdinalIgnoreCase);

    public static IReadOnlyList<UnitOperationParameterDefinition> OrderedDefinitions => OrderedDefinitionsValue;

    public static IReadOnlyList<string> OrderedNames { get; } = OrderedDefinitionsValue.Select(static definition => definition.Name).ToArray();

    public static bool TryGetByName(string name, out UnitOperationParameterDefinition definition)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(name);
        return DefinitionsByNameValue.TryGetValue(name, out definition!);
    }

    public static UnitOperationParameterDefinition GetByName(string name)
    {
        if (TryGetByName(name, out var definition))
        {
            return definition;
        }

        throw new ArgumentException($"Unknown unit operation parameter definition `{name}`.", nameof(name));
    }
}

public sealed record UnitOperationParameterDefinition(
    string Name,
    string Description,
    bool IsRequired,
    UnitOperationParameterValueKind ValueKind,
    bool AllowsEmptyValue,
    string? RequiredCompanionParameterName,
    CapeParamMode Mode,
    string? DefaultValue);
