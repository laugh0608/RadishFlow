using System.Text.Json;
using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationHostPortMaterialReader
{
    public static UnitOperationHostPortMaterialSnapshot Read(
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
            return new UnitOperationHostPortMaterialSnapshot(
                State: UnitOperationHostPortMaterialState.Terminated,
                Headline: "Unit operation has been terminated.",
                PortEntries: []);
        }

        var bindings = UnitOperationConfiguredBoundaryMaterialBindings.TryParse(
            unitOperation.Parameters.GetByName(UnitOperationParameterCatalog.FlowsheetJson.Name));
        var snapshotState = DetermineState(unitOperation);
        var currentStreams = unitOperation.LastCalculationResult?.Streams
            .ToDictionary(static stream => stream.Id, StringComparer.Ordinal)
            ?? new Dictionary<string, UnitOperationCalculationStream>(StringComparer.Ordinal);
        var portEntries = UnitOperationPortCatalog.OrderedDefinitions
            .Select(definition => CreatePortEntry(unitOperation, definition, bindings, snapshotState, currentStreams))
            .ToArray();

        return new UnitOperationHostPortMaterialSnapshot(
            State: snapshotState,
            Headline: CreateHeadline(snapshotState, unitOperation, bindings),
            PortEntries: portEntries);
    }

    private static UnitOperationHostPortMaterialEntry CreatePortEntry(
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationPortDefinition definition,
        UnitOperationConfiguredBoundaryMaterialBindings bindings,
        UnitOperationHostPortMaterialState snapshotState,
        IReadOnlyDictionary<string, UnitOperationCalculationStream> currentStreams)
    {
        var port = unitOperation.Ports.GetByName(definition.Name);
        var boundStreamIds = bindings.GetBoundStreamIds(definition.BoundaryMaterialRole);
        var connectedTargetName = (port.connectedObject as ICapeIdentification)?.ComponentName;
        UnitOperationHostMaterialStreamEntry[] materialEntries = snapshotState == UnitOperationHostPortMaterialState.Available
            ? boundStreamIds
                .Where(currentStreams.ContainsKey)
                .Select(streamId => CreateMaterialEntry(currentStreams[streamId]))
                .ToArray()
            : [];
        var materialState = materialEntries.Length > 0
            ? UnitOperationHostPortMaterialState.Available
            : snapshotState == UnitOperationHostPortMaterialState.Stale && boundStreamIds.Count > 0
                ? UnitOperationHostPortMaterialState.Stale
                : UnitOperationHostPortMaterialState.None;

        return new UnitOperationHostPortMaterialEntry(
            Name: definition.Name,
            Description: definition.Description,
            IsRequired: definition.IsRequired,
            IsConnected: port.IsConnected,
            ConnectedTargetName: connectedTargetName,
            Direction: definition.Direction,
            PortType: definition.PortType,
            BoundaryMaterialRole: definition.BoundaryMaterialRole,
            MaterialState: materialState,
            BoundStreamIds: boundStreamIds,
            MaterialEntries: materialEntries);
    }

    private static UnitOperationHostMaterialStreamEntry CreateMaterialEntry(
        UnitOperationCalculationStream stream)
    {
        return new UnitOperationHostMaterialStreamEntry(
            StreamId: stream.Id,
            StreamName: stream.Name,
            TemperatureK: stream.TemperatureK,
            PressurePa: stream.PressurePa,
            TotalMolarFlowMolS: stream.TotalMolarFlowMolS,
            Phases: stream.Phases
                .Select(static phase => new UnitOperationHostMaterialPhaseEntry(
                    Label: phase.Label,
                    PhaseFraction: phase.PhaseFraction))
                .ToArray());
    }

    private static UnitOperationHostPortMaterialState DetermineState(
        RadishFlowCapeOpenUnitOperation unitOperation)
    {
        if (unitOperation.LastCalculationResult is not null)
        {
            return UnitOperationHostPortMaterialState.Available;
        }

        if (unitOperation.HostMaterialResultsStale)
        {
            return UnitOperationHostPortMaterialState.Stale;
        }

        return UnitOperationHostPortMaterialState.None;
    }

    private static string CreateHeadline(
        UnitOperationHostPortMaterialState state,
        RadishFlowCapeOpenUnitOperation unitOperation,
        UnitOperationConfiguredBoundaryMaterialBindings bindings)
    {
        return state switch
        {
            UnitOperationHostPortMaterialState.Available => unitOperation.LastCalculationResult!.Summary.PrimaryMessage,
            UnitOperationHostPortMaterialState.Stale => "Material results are stale and require Calculate() to refresh.",
            UnitOperationHostPortMaterialState.Terminated => "Unit operation has been terminated.",
            UnitOperationHostPortMaterialState.None when unitOperation.LastCalculationFailure is not null
                => "No current material result is available because the last calculation failed.",
            UnitOperationHostPortMaterialState.None when bindings.HasConfiguredBoundaryStreams
                => "No current material result is available for the configured host ports.",
            _ => "No configured material boundary is available.",
        };
    }
}

public sealed record UnitOperationHostPortMaterialSnapshot(
    UnitOperationHostPortMaterialState State,
    string Headline,
    IReadOnlyList<UnitOperationHostPortMaterialEntry> PortEntries)
{
    public int PortCount => PortEntries.Count;

    public UnitOperationHostPortMaterialEntry GetPort(string name)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(name);

        foreach (var entry in PortEntries)
        {
            if (string.Equals(entry.Name, name, StringComparison.OrdinalIgnoreCase))
            {
                return entry;
            }
        }

        throw new ArgumentException($"Unknown unit operation host port/material entry `{name}`.", nameof(name));
    }
}

public sealed record UnitOperationHostPortMaterialEntry(
    string Name,
    string Description,
    bool IsRequired,
    bool IsConnected,
    string? ConnectedTargetName,
    CapePortDirection Direction,
    CapePortType PortType,
    UnitOperationPortBoundaryMaterialRole BoundaryMaterialRole,
    UnitOperationHostPortMaterialState MaterialState,
    IReadOnlyList<string> BoundStreamIds,
    IReadOnlyList<UnitOperationHostMaterialStreamEntry> MaterialEntries)
{
    public bool HasCurrentMaterialEntries => MaterialEntries.Count > 0;
}

public sealed record UnitOperationHostMaterialStreamEntry(
    string StreamId,
    string StreamName,
    double TemperatureK,
    double PressurePa,
    double TotalMolarFlowMolS,
    IReadOnlyList<UnitOperationHostMaterialPhaseEntry> Phases);

public sealed record UnitOperationHostMaterialPhaseEntry(
    string Label,
    double PhaseFraction);

public enum UnitOperationHostPortMaterialState
{
    None,
    Stale,
    Available,
    Terminated,
}

internal sealed record UnitOperationConfiguredBoundaryMaterialBindings(
    IReadOnlyList<string> BoundaryInputStreamIds,
    IReadOnlyList<string> BoundaryOutputStreamIds)
{
    public bool HasConfiguredBoundaryStreams => BoundaryInputStreamIds.Count > 0 || BoundaryOutputStreamIds.Count > 0;

    public IReadOnlyList<string> GetBoundStreamIds(UnitOperationPortBoundaryMaterialRole boundaryRole)
    {
        return boundaryRole switch
        {
            UnitOperationPortBoundaryMaterialRole.BoundaryInputs => BoundaryInputStreamIds,
            UnitOperationPortBoundaryMaterialRole.BoundaryOutputs => BoundaryOutputStreamIds,
            _ => throw new ArgumentOutOfRangeException(nameof(boundaryRole), boundaryRole, "Unknown unit operation port boundary role."),
        };
    }

    public static UnitOperationConfiguredBoundaryMaterialBindings TryParse(
        Placeholders.UnitOperationParameterPlaceholder flowsheetParameter)
    {
        ArgumentNullException.ThrowIfNull(flowsheetParameter);

        if (!flowsheetParameter.IsConfigured || string.IsNullOrWhiteSpace(flowsheetParameter.Value))
        {
            return new UnitOperationConfiguredBoundaryMaterialBindings([], []);
        }

        try
        {
            return Parse(flowsheetParameter.Value!);
        }
        catch (JsonException)
        {
            return new UnitOperationConfiguredBoundaryMaterialBindings([], []);
        }
        catch (InvalidDataException)
        {
            return new UnitOperationConfiguredBoundaryMaterialBindings([], []);
        }
    }

    private static UnitOperationConfiguredBoundaryMaterialBindings Parse(string flowsheetJson)
    {
        using var document = JsonDocument.Parse(flowsheetJson);
        var root = RequireObject(document.RootElement, "$");
        var documentElement = RequireObject(GetRequiredProperty(root, "document", "$"), "$.document");
        var flowsheet = RequireObject(GetRequiredProperty(documentElement, "flowsheet", "$.document"), "$.document.flowsheet");
        var units = GetRequiredProperty(flowsheet, "units", "$.document.flowsheet");
        if (units.ValueKind != JsonValueKind.Object)
        {
            throw new InvalidDataException("Expected `$.document.flowsheet.units` to be a JSON object.");
        }

        var boundaryInputs = new List<string>();
        var boundaryOutputs = new List<string>();
        var downstreamMaterialInputs = new HashSet<string>(StringComparer.Ordinal);

        var unitMaterialPorts = new List<UnitMaterialPortSet>();
        foreach (var unitProperty in units.EnumerateObject())
        {
            var unit = RequireObject(unitProperty.Value, $"$.document.flowsheet.units.{unitProperty.Name}");
            var ports = GetRequiredProperty(unit, "ports", $"$.document.flowsheet.units.{unitProperty.Name}");
            if (ports.ValueKind != JsonValueKind.Array)
            {
                throw new InvalidDataException($"Expected `$.document.flowsheet.units.{unitProperty.Name}.ports` to be a JSON array.");
            }

            var inletStreamIds = new List<string>();
            var outletStreamIds = new List<string>();
            foreach (var portValue in ports.EnumerateArray())
            {
                var port = RequireObject(portValue, $"$.document.flowsheet.units.{unitProperty.Name}.ports[]");
                if (!string.Equals(ReadOptionalString(port, "kind"), "material", StringComparison.OrdinalIgnoreCase))
                {
                    continue;
                }

                var streamId = ReadOptionalString(port, "stream_id");
                if (string.IsNullOrWhiteSpace(streamId))
                {
                    continue;
                }

                var direction = ReadOptionalString(port, "direction");
                if (string.Equals(direction, "inlet", StringComparison.OrdinalIgnoreCase))
                {
                    inletStreamIds.Add(streamId);
                    downstreamMaterialInputs.Add(streamId);
                    continue;
                }

                if (string.Equals(direction, "outlet", StringComparison.OrdinalIgnoreCase))
                {
                    outletStreamIds.Add(streamId);
                }
            }

            unitMaterialPorts.Add(new UnitMaterialPortSet(inletStreamIds, outletStreamIds));
        }

        foreach (var unitPorts in unitMaterialPorts)
        {
            if (unitPorts.InletStreamIds.Count == 0)
            {
                AppendDistinct(boundaryInputs, unitPorts.OutletStreamIds);
            }

            foreach (var outletStreamId in unitPorts.OutletStreamIds)
            {
                if (!downstreamMaterialInputs.Contains(outletStreamId))
                {
                    AppendDistinct(boundaryOutputs, [outletStreamId]);
                }
            }
        }

        return new UnitOperationConfiguredBoundaryMaterialBindings(boundaryInputs, boundaryOutputs);
    }

    private static void AppendDistinct(List<string> target, IReadOnlyList<string> values)
    {
        foreach (var value in values)
        {
            if (!target.Contains(value, StringComparer.Ordinal))
            {
                target.Add(value);
            }
        }
    }

    private static JsonElement RequireObject(JsonElement element, string path)
    {
        if (element.ValueKind != JsonValueKind.Object)
        {
            throw new InvalidDataException(
                $"Expected `{path}` to be a JSON object but found `{element.ValueKind}`.");
        }

        return element;
    }

    private static JsonElement GetRequiredProperty(JsonElement element, string propertyName, string path)
    {
        if (!element.TryGetProperty(propertyName, out var property))
        {
            throw new InvalidDataException(
                $"Required property `{path}.{propertyName}` is missing from the configured flowsheet JSON.");
        }

        return property;
    }

    private static string? ReadOptionalString(JsonElement element, string propertyName)
    {
        if (!element.TryGetProperty(propertyName, out var value) || value.ValueKind != JsonValueKind.String)
        {
            return null;
        }

        return value.GetString();
    }

    private sealed record UnitMaterialPortSet(
        IReadOnlyList<string> InletStreamIds,
        IReadOnlyList<string> OutletStreamIds);
}
