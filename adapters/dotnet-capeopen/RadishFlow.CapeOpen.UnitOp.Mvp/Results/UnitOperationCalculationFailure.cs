using System.Text.Json;
using RadishFlow.CapeOpen.Interop.Errors;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public sealed record UnitOperationCalculationFailure(
    string ErrorName,
    string Operation,
    string Description,
    string? RequestedOperation,
    string? NativeStatus,
    UnitOperationCalculationFailureSummary Summary)
{
    internal static UnitOperationCalculationFailure FromException(CapeOpenException error)
    {
        ArgumentNullException.ThrowIfNull(error);

        return new UnitOperationCalculationFailure(
            ErrorName: error.ErrorName,
            Operation: error.Operation,
            Description: error.Description,
            RequestedOperation: error.RequestedOperation,
            NativeStatus: error.NativeStatus,
            Summary: UnitOperationCalculationFailureSummary.FromException(error));
    }
}

public sealed record UnitOperationCalculationFailureSummary(
    string PrimaryMessage,
    string? DiagnosticCode,
    IReadOnlyList<string> RelatedUnitIds,
    IReadOnlyList<string> RelatedStreamIds,
    IReadOnlyList<UnitOperationCalculationFailurePortTarget> RelatedPortTargets)
{
    internal static UnitOperationCalculationFailureSummary FromException(CapeOpenException error)
    {
        ArgumentNullException.ThrowIfNull(error);

        if (string.IsNullOrWhiteSpace(error.DiagnosticJson))
        {
            return Empty(error.Description);
        }

        try
        {
            using var document = JsonDocument.Parse(error.DiagnosticJson);
            if (document.RootElement.ValueKind != JsonValueKind.Object)
            {
                return Empty(error.Description);
            }

            var root = document.RootElement;
            return new UnitOperationCalculationFailureSummary(
                PrimaryMessage: ReadOptionalString(root, "message") ?? error.Description,
                DiagnosticCode: ReadOptionalString(root, "diagnosticCode"),
                RelatedUnitIds: ReadStringArray(root, "relatedUnitIds"),
                RelatedStreamIds: ReadStringArray(root, "relatedStreamIds"),
                RelatedPortTargets: ReadPortTargets(root, "relatedPortTargets"));
        }
        catch (JsonException)
        {
            return Empty(error.Description);
        }
    }

    private static UnitOperationCalculationFailureSummary Empty(string description)
    {
        return new UnitOperationCalculationFailureSummary(
            PrimaryMessage: description,
            DiagnosticCode: null,
            RelatedUnitIds: Array.Empty<string>(),
            RelatedStreamIds: Array.Empty<string>(),
            RelatedPortTargets: Array.Empty<UnitOperationCalculationFailurePortTarget>());
    }

    private static string? ReadOptionalString(JsonElement element, string propertyName)
    {
        if (!element.TryGetProperty(propertyName, out var value))
        {
            return null;
        }

        return value.ValueKind == JsonValueKind.String ? value.GetString() : null;
    }

    private static IReadOnlyList<string> ReadStringArray(JsonElement element, string propertyName)
    {
        if (!element.TryGetProperty(propertyName, out var value) || value.ValueKind != JsonValueKind.Array)
        {
            return Array.Empty<string>();
        }

        var items = new List<string>(value.GetArrayLength());
        foreach (var item in value.EnumerateArray())
        {
            if (item.ValueKind != JsonValueKind.String)
            {
                continue;
            }

            var text = item.GetString();
            if (!string.IsNullOrWhiteSpace(text))
            {
                items.Add(text);
            }
        }

        return items;
    }

    private static IReadOnlyList<UnitOperationCalculationFailurePortTarget> ReadPortTargets(
        JsonElement element,
        string propertyName)
    {
        if (!element.TryGetProperty(propertyName, out var value) || value.ValueKind != JsonValueKind.Array)
        {
            return Array.Empty<UnitOperationCalculationFailurePortTarget>();
        }

        var items = new List<UnitOperationCalculationFailurePortTarget>(value.GetArrayLength());
        foreach (var item in value.EnumerateArray())
        {
            if (item.ValueKind != JsonValueKind.Object)
            {
                continue;
            }

            var unitId = ReadOptionalString(item, "unitId");
            var portName = ReadOptionalString(item, "portName");
            if (string.IsNullOrWhiteSpace(unitId) || string.IsNullOrWhiteSpace(portName))
            {
                continue;
            }

            items.Add(new UnitOperationCalculationFailurePortTarget(unitId, portName));
        }

        return items;
    }
}

public sealed record UnitOperationCalculationFailurePortTarget(
    string UnitId,
    string PortName);
