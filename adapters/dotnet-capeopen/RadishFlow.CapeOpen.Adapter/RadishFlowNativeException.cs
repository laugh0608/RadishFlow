using System.Text.Json;
using RadishFlow.CapeOpen.Interop.Errors;

namespace RadishFlow.CapeOpen.Adapter;

public sealed class RadishFlowNativeException : CapeOpenException
{
    public RadishFlowNativeException(
        string operation,
        RfFfiStatus status,
        string? nativeMessage,
        string? nativeErrorJson)
        : base(
            MapSemantic(operation, status, nativeErrorJson).errorName,
            BuildMessage(operation, status, nativeMessage),
            MapSemantic(operation, status, nativeErrorJson).hresult,
            BuildContext(operation, status, nativeMessage, nativeErrorJson))
    {
        Status = status;
        NativeMessage = nativeMessage;
        NativeErrorJson = nativeErrorJson;
    }

    public RfFfiStatus Status { get; }

    public string? NativeMessage { get; }

    public string? NativeErrorJson { get; }

    private static CapeOpenExceptionContext BuildContext(
        string operation,
        RfFfiStatus status,
        string? nativeMessage,
        string? nativeErrorJson)
    {
        var scope = TryReadJsonString(nativeErrorJson, "code") ?? "rf_ffi";
        var moreInfo = TryReadJsonString(nativeErrorJson, "diagnosticCode") ?? nativeMessage;
        return new CapeOpenExceptionContext(
            InterfaceName: "rf-ffi",
            Scope: scope,
            Operation: operation,
            MoreInfo: moreInfo,
            DiagnosticJson: nativeErrorJson,
            NativeStatus: status.ToString());
    }

    private static (string errorName, int hresult) MapSemantic(
        string operation,
        RfFfiStatus status,
        string? nativeErrorJson)
    {
        var code = TryReadJsonString(nativeErrorJson, "code");
        var diagnosticCode = TryReadJsonString(nativeErrorJson, "diagnosticCode");

        if (string.Equals(operation, "engine_create", StringComparison.Ordinal))
        {
            return ("ECapeFailedInitialisation", CapeOpenErrorHResults.ECapeFailedInitialisation);
        }

        return status switch
        {
            RfFfiStatus.NullPointer or RfFfiStatus.InvalidUtf8 or RfFfiStatus.InvalidInput or RfFfiStatus.DuplicateId =>
                ("ECapeInvalidArgument", CapeOpenErrorHResults.ECapeInvalidArgument),
            RfFfiStatus.InvalidEngineState =>
                ("ECapeBadInvOrder", CapeOpenErrorHResults.ECapeBadInvOrder),
            RfFfiStatus.MissingEntity when string.Equals(code, "missing_entity", StringComparison.Ordinal) =>
                ("ECapeInvalidArgument", CapeOpenErrorHResults.ECapeInvalidArgument),
            RfFfiStatus.InvalidConnection =>
                ("ECapeSolvingError", CapeOpenErrorHResults.ECapeSolvingError),
            RfFfiStatus.Thermo or RfFfiStatus.Flash =>
                ("ECapeSolvingError", CapeOpenErrorHResults.ECapeSolvingError),
            RfFfiStatus.NotImplemented =>
                ("ECapeNoImpl", CapeOpenErrorHResults.ECapeNoImpl),
            RfFfiStatus.Panic =>
                ("ECapeUnknown", CapeOpenErrorHResults.ECapeUnknown),
            _ when string.Equals(diagnosticCode, "ffi.engine_state.snapshot_not_available", StringComparison.Ordinal) =>
                ("ECapeBadInvOrder", CapeOpenErrorHResults.ECapeBadInvOrder),
            _ => ("ECapeUnknown", CapeOpenErrorHResults.ECapeUnknown),
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
