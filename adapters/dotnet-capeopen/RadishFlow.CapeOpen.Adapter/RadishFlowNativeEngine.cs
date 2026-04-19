using System.Text;
using RadishFlow.CapeOpen.Interop.Errors;

namespace RadishFlow.CapeOpen.Adapter;

public sealed class RadishFlowNativeEngine : IDisposable
{
    private readonly RfNativeEngineHandle _handle;

    public RadishFlowNativeEngine()
    {
        RfNativeLibraryLoader.EnsureResolverInstalled();
        var status = RfNativeMethods.EngineCreate(out var handle);
        if (status != RfFfiStatus.Ok)
        {
            throw new CapeFailedInitialisationException(
                $"Failed to create native engine with status `{status}`.",
                new CapeOpenExceptionContext(
                    InterfaceName: "rf-ffi",
                    Scope: "RadishFlow.CapeOpen.Adapter.Native",
                    Operation: "engine_create",
                    MoreInfo: $"native engine create returned `{status}`",
                    NativeStatus: status.ToString()));
        }

        _handle = RfNativeEngineHandle.FromNative(handle);
    }

    public void LoadFlowsheetJson(string json)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(json);

        InvokeStatus(
            "flowsheet_load_json",
            utf8 => RfNativeMethods.FlowsheetLoadJson(
                _handle.DangerousGetHandle(),
                utf8,
                (nuint)utf8.Length),
            json);
    }

    public void LoadPropertyPackageFiles(string manifestPath, string payloadPath)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(manifestPath);
        ArgumentException.ThrowIfNullOrWhiteSpace(payloadPath);

        var manifestUtf8 = Encoding.UTF8.GetBytes(manifestPath);
        var payloadUtf8 = Encoding.UTF8.GetBytes(payloadPath);
        var status = RfNativeMethods.PropertyPackageLoadFromFiles(
            _handle.DangerousGetHandle(),
            manifestUtf8,
            (nuint)manifestUtf8.Length,
            payloadUtf8,
            (nuint)payloadUtf8.Length);
        EnsureSuccess("property_package_load_from_files", status);
    }

    public string GetPropertyPackageListJson()
    {
        return ReadOwnedUtf8String(
            "property_package_list_json",
            (out nint pointer) => RfNativeMethods.PropertyPackageListJson(
                _handle.DangerousGetHandle(),
                out pointer));
    }

    public void SolveFlowsheet(string packageId)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(packageId);

        InvokeStatus(
            "flowsheet_solve",
            utf8 => RfNativeMethods.FlowsheetSolve(
                _handle.DangerousGetHandle(),
                utf8,
                (nuint)utf8.Length),
            packageId);
    }

    public string GetFlowsheetSnapshotJson()
    {
        return ReadOwnedUtf8String(
            "flowsheet_get_snapshot_json",
            (out nint pointer) => RfNativeMethods.FlowsheetGetSnapshotJson(
                _handle.DangerousGetHandle(),
                out pointer));
    }

    public string GetStreamSnapshotJson(string streamId)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(streamId);

        var utf8 = Encoding.UTF8.GetBytes(streamId);
        return ReadOwnedUtf8String(
            "stream_get_snapshot_json",
            (out nint pointer) => RfNativeMethods.StreamGetSnapshotJson(
                _handle.DangerousGetHandle(),
                utf8,
                (nuint)utf8.Length,
                out pointer));
    }

    public string? TryGetLastErrorMessage()
    {
        return TryReadOwnedUtf8String(
            (out nint pointer) => RfNativeMethods.EngineLastErrorMessage(
                _handle.DangerousGetHandle(),
                out pointer));
    }

    public string? TryGetLastErrorJson()
    {
        return TryReadOwnedUtf8String(
            (out nint pointer) => RfNativeMethods.EngineLastErrorJson(
                _handle.DangerousGetHandle(),
                out pointer));
    }

    public void Dispose()
    {
        _handle.Dispose();
    }

    private void InvokeStatus(string operation, Func<byte[], RfFfiStatus> nativeCall, string value)
    {
        var utf8 = Encoding.UTF8.GetBytes(value);
        var status = nativeCall(utf8);
        EnsureSuccess(operation, status);
    }

    private string ReadOwnedUtf8String(
        string operation,
        NativeOwnedUtf8Call nativeCall)
    {
        var value = TryReadOwnedUtf8String(nativeCall, out var status);
        EnsureSuccess(operation, status);
        return value ?? string.Empty;
    }

    private string? TryReadOwnedUtf8String(
        NativeOwnedUtf8Call nativeCall,
        out RfFfiStatus status)
    {
        status = nativeCall(out var pointer);
        try
        {
            if (pointer == IntPtr.Zero)
            {
                return null;
            }

            return System.Runtime.InteropServices.Marshal.PtrToStringUTF8(pointer);
        }
        finally
        {
            if (pointer != IntPtr.Zero)
            {
                RfNativeMethods.RfStringFree(pointer);
            }
        }
    }

    private string? TryReadOwnedUtf8String(NativeOwnedUtf8Call nativeCall)
    {
        return TryReadOwnedUtf8String(nativeCall, out _);
    }

    private void EnsureSuccess(string operation, RfFfiStatus status)
    {
        if (status == RfFfiStatus.Ok)
        {
            return;
        }

        throw RadishFlowNativeException.Create(
            operation,
            status,
            TryGetLastErrorMessage(),
            TryGetLastErrorJson());
    }

    private delegate RfFfiStatus NativeOwnedUtf8Call(out nint pointer);
}
