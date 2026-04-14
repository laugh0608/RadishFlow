namespace RadishFlow.CapeOpen.Adapter;

public sealed class RadishFlowNativeException : Exception
{
    public RadishFlowNativeException(
        string operation,
        RfFfiStatus status,
        string? nativeMessage,
        string? nativeErrorJson)
        : base(BuildMessage(operation, status, nativeMessage))
    {
        Operation = operation;
        Status = status;
        NativeMessage = nativeMessage;
        NativeErrorJson = nativeErrorJson;
    }

    public string Operation { get; }

    public RfFfiStatus Status { get; }

    public string? NativeMessage { get; }

    public string? NativeErrorJson { get; }

    private static string BuildMessage(string operation, RfFfiStatus status, string? nativeMessage)
    {
        if (string.IsNullOrWhiteSpace(nativeMessage))
        {
            return $"Native call `{operation}` failed with status `{status}`.";
        }

        return $"Native call `{operation}` failed with status `{status}`: {nativeMessage}";
    }
}
