using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

public static class UnitOperationParameterCatalog
{
    public static UnitOperationCollectionDefinition CollectionDefinition { get; } = new(
        Name: "Parameters",
        Description: "Public CAPE-OPEN parameter collection for the MVP unit operation.");

    public static UnitOperationParameterDefinition FlowsheetJson { get; } = new(
        Name: "Flowsheet Json",
        Description: "StoredProjectFile JSON used by the MVP unit operation skeleton.",
        IsRequired: true,
        ValueKind: UnitOperationParameterValueKind.StructuredJsonText,
        AllowsEmptyValue: false,
        RequiredCompanionParameterName: null,
        ConfigurationOperationName: nameof(RadishFlowCapeOpenUnitOperation.LoadFlowsheetJson),
        Mode: CapeParamMode.CAPE_INPUT,
        DefaultValue: null,
        SpecificationType: CapeParamType.CAPE_OPTION,
        SpecificationDimensionality: Array.Empty<double>());

    public static UnitOperationParameterDefinition PropertyPackageId { get; } = new(
        Name: "Property Package Id",
        Description: "Identifier of the property package selected for the MVP unit operation skeleton.",
        IsRequired: true,
        ValueKind: UnitOperationParameterValueKind.Identifier,
        AllowsEmptyValue: false,
        RequiredCompanionParameterName: null,
        ConfigurationOperationName: nameof(RadishFlowCapeOpenUnitOperation.SelectPropertyPackage),
        Mode: CapeParamMode.CAPE_INPUT,
        DefaultValue: null,
        SpecificationType: CapeParamType.CAPE_OPTION,
        SpecificationDimensionality: Array.Empty<double>());

    public static UnitOperationParameterDefinition PropertyPackageManifestPath { get; } = new(
        Name: "Property Package Manifest Path",
        Description: "Optional manifest path for a local property package payload.",
        IsRequired: false,
        ValueKind: UnitOperationParameterValueKind.FilePath,
        AllowsEmptyValue: false,
        RequiredCompanionParameterName: "Property Package Payload Path",
        ConfigurationOperationName: nameof(RadishFlowCapeOpenUnitOperation.LoadPropertyPackageFiles),
        Mode: CapeParamMode.CAPE_INPUT,
        DefaultValue: null,
        SpecificationType: CapeParamType.CAPE_OPTION,
        SpecificationDimensionality: Array.Empty<double>());

    public static UnitOperationParameterDefinition PropertyPackagePayloadPath { get; } = new(
        Name: "Property Package Payload Path",
        Description: "Optional payload path for a local property package payload.",
        IsRequired: false,
        ValueKind: UnitOperationParameterValueKind.FilePath,
        AllowsEmptyValue: false,
        RequiredCompanionParameterName: "Property Package Manifest Path",
        ConfigurationOperationName: nameof(RadishFlowCapeOpenUnitOperation.LoadPropertyPackageFiles),
        Mode: CapeParamMode.CAPE_INPUT,
        DefaultValue: null,
        SpecificationType: CapeParamType.CAPE_OPTION,
        SpecificationDimensionality: Array.Empty<double>());

    private static readonly IReadOnlyList<UnitOperationParameterDefinition> OrderedDefinitionsValue =
        ValidateDefinitions(
        [
        FlowsheetJson,
        PropertyPackageId,
        PropertyPackageManifestPath,
        PropertyPackagePayloadPath,
        ]);
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

    private static IReadOnlyList<UnitOperationParameterDefinition> ValidateDefinitions(
        IReadOnlyList<UnitOperationParameterDefinition> definitions)
    {
        var definitionsByName = definitions.ToDictionary(static definition => definition.Name, StringComparer.OrdinalIgnoreCase);
        foreach (var definition in definitions)
        {
            if (string.IsNullOrWhiteSpace(definition.ConfigurationOperationName))
            {
                throw new InvalidOperationException(
                    $"Unit operation parameter definition `{definition.Name}` must declare a non-empty configuration operation.");
            }

            if (definition.RequiredCompanionParameterName is not { Length: > 0 } companionName)
            {
                continue;
            }

            if (!definitionsByName.TryGetValue(companionName, out var companionDefinition))
            {
                throw new InvalidOperationException(
                    $"Unit operation parameter definition `{definition.Name}` references unknown companion `{companionName}`.");
            }

            if (!string.Equals(companionDefinition.RequiredCompanionParameterName, definition.Name, StringComparison.OrdinalIgnoreCase))
            {
                throw new InvalidOperationException(
                    $"Unit operation parameter companion contract must be symmetric between `{definition.Name}` and `{companionDefinition.Name}`.");
            }

            if (!string.Equals(
                definition.ConfigurationOperationName,
                companionDefinition.ConfigurationOperationName,
                StringComparison.Ordinal))
            {
                throw new InvalidOperationException(
                    $"Companion parameters `{definition.Name}` and `{companionDefinition.Name}` must share the same configuration operation.");
            }
        }

        return definitions;
    }
}

public sealed record UnitOperationParameterDefinition(
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
    IReadOnlyList<double> SpecificationDimensionality);
