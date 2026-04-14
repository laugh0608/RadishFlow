using System.Text.Json;
using RadishFlow.CapeOpen.Interop.Errors;

namespace RadishFlow.CapeOpen.Adapter;

public static class RadishFlowNativeException
{
    private const string NativeScope = "RadishFlow.CapeOpen.Adapter.Native";

    public static CapeOpenException Create(
        string operation,
        RfFfiStatus status,
        string? nativeMessage,
        string? nativeErrorJson)
    {
        var diagnosticCode = TryReadJsonString(nativeErrorJson, "diagnosticCode");
        var description = BuildMessage(operation, status, nativeMessage);
        var context = BuildContext(operation, status, nativeMessage, nativeErrorJson, diagnosticCode);

        if (string.Equals(operation, "engine_create", StringComparison.Ordinal))
        {
            return new CapeFailedInitialisationException(description, context);
        }

        return status switch
        {
            RfFfiStatus.NullPointer or RfFfiStatus.InvalidUtf8 or RfFfiStatus.InvalidInput or RfFfiStatus.DuplicateId =>
                new CapeInvalidArgumentException(description, context),
            RfFfiStatus.InvalidEngineState =>
                new CapeBadInvocationOrderException(description, context),
            RfFfiStatus.MissingEntity =>
                new CapeInvalidArgumentException(description, context),
            RfFfiStatus.InvalidConnection =>
                new CapeSolvingException(description, context),
            RfFfiStatus.Thermo or RfFfiStatus.Flash =>
                new CapeSolvingException(description, context),
            RfFfiStatus.NotImplemented =>
                new CapeNoImplementationException(description, context),
            RfFfiStatus.Panic =>
                new CapeUnknownException(description, context),
            _ when string.Equals(diagnosticCode, "ffi.engine_state.snapshot_not_available", StringComparison.Ordinal) =>
                new CapeBadInvocationOrderException(description, context),
            _ => new CapeUnknownException(description, context),
        };
    }

    private static CapeOpenExceptionContext BuildContext(
        string operation,
        RfFfiStatus status,
        string? nativeMessage,
        string? nativeErrorJson,
        string? diagnosticCode)
    {
        return new CapeOpenExceptionContext(
            InterfaceName: "rf-ffi",
            Scope: NativeScope,
            Operation: operation,
            MoreInfo: diagnosticCode ?? nativeMessage,
            DiagnosticJson: nativeErrorJson,
            NativeStatus: status.ToString(),
            RequestedOperation: TryGetRequestedOperation(operation, diagnosticCode));
    }

    private static string? TryGetRequestedOperation(string operation, string? diagnosticCode)
    {
        return diagnosticCode switch
        {
            "ffi.engine_state.flowsheet_not_loaded" => "flowsheet_load_json",
            "ffi.engine_state.snapshot_not_available" => "flowsheet_solve",
            _ when string.Equals(operation, "engine_create", StringComparison.Ordinal) => null,
            _ => null,
        };
    }

    private static string? TryReadJsonString(string? json, string propertyName)
    {
        if (string.IsNullOrWhiteSpace(json) || string.Equals(json, "null", StringComparison.OrdinalIgnoreCase))
        {
            return null;
        }

        try
        {
            using var document = JsonDocument.Parse(json);
            if (document.RootElement.ValueKind != JsonValueKind.Object)
            {
                return null;
            }

            if (!document.RootElement.TryGetProperty(propertyName, out var value))
            {
                return null;
            }

            return value.ValueKind == JsonValueKind.String ? value.GetString() : null;
        }
        catch (JsonException)
        {
            return null;
        }
    }

    private static string BuildMessage(string operation, RfFfiStatus status, string? nativeMessage)
    {
        if (string.IsNullOrWhiteSpace(nativeMessage))
        {
            return $"Native call `{operation}` failed with status `{status}`.";
        }

        return $"Native call `{operation}` failed with status `{status}`: {nativeMessage}";
    }
}
