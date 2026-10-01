using System.Text;
using RadishFlow.CapeOpen.Interop.Errors;

namespace RadishFlow.CapeOpen.Adapter;

public sealed class RadishFlowNativeEngine : IDisposable
{
    private readonly RfNativeEngineHandle _handle;
    private readonly object _gate = new();

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
            (handle, utf8) => RfNativeMethods.FlowsheetLoadJson(
                handle,
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
        WithHandle(handle =>
        {
            var status = RfNativeMethods.PropertyPackageLoadFromFiles(
                handle,
                manifestUtf8,
                (nuint)manifestUtf8.Length,
                payloadUtf8,
                (nuint)payloadUtf8.Length);
            EnsureSuccess(handle, "property_package_load_from_files", status);
            return status;
        });
    }

    public string GetPropertyPackageListJson()
    {
        return ReadOwnedUtf8String(
            "property_package_list_json",
            (RfNativeEngineHandle handle, out nint pointer) => RfNativeMethods.PropertyPackageListJson(
                handle,
                out pointer));
    }

    public void SolveFlowsheet(string packageId)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(packageId);

        InvokeStatus(
            "flowsheet_solve",
            (handle, utf8) => RfNativeMethods.FlowsheetSolve(
                handle,
                utf8,
                (nuint)utf8.Length),
            packageId);
    }

    public string GetFlowsheetSnapshotJson()
    {
        return ReadOwnedUtf8String(
            "flowsheet_get_snapshot_json",
            (RfNativeEngineHandle handle, out nint pointer) => RfNativeMethods.FlowsheetGetSnapshotJson(
                handle,
                out pointer));
    }

    public string GetStreamSnapshotJson(string streamId)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(streamId);

        var utf8 = Encoding.UTF8.GetBytes(streamId);
        return ReadOwnedUtf8String(
            "stream_get_snapshot_json",
            (RfNativeEngineHandle handle, out nint pointer) => RfNativeMethods.StreamGetSnapshotJson(
                handle,
                utf8,
                (nuint)utf8.Length,
                out pointer));
    }

    public string? TryGetLastErrorMessage()
    {
        return WithHandle(handle => TryReadOwnedUtf8String(
            handle, RfNativeMethods.EngineLastErrorMessage, out _));
    }

    public string? TryGetLastErrorJson()
    {
        return WithHandle(handle => TryReadOwnedUtf8String(
            handle, RfNativeMethods.EngineLastErrorJson, out _));
    }

    public void Dispose()
    {
        lock (_gate)
        {
            _handle.Dispose();
        }
    }

    // The complete operation, including native error retrieval, shares the same
    // gate as Dispose. SafeHandle marshalling protects each P/Invoke call.
    internal T WithHandle<T>(Func<RfNativeEngineHandle, T> operation)
    {
        lock (_gate)
        {
            ObjectDisposedException.ThrowIf(_handle.IsClosed, this);
            return operation(_handle);
        }
    }

    private void InvokeStatus(
        string operation,
        Func<RfNativeEngineHandle, byte[], RfFfiStatus> nativeCall,
        string value)
    {
        var utf8 = Encoding.UTF8.GetBytes(value);
        WithHandle(handle =>
        {
            var status = nativeCall(handle, utf8);
            EnsureSuccess(handle, operation, status);
            return status;
        });
    }

    private string ReadOwnedUtf8String(string operation, NativeOwnedUtf8Call nativeCall)
    {
        return WithHandle(handle =>
        {
            var value = TryReadOwnedUtf8String(handle, nativeCall, out var status);
            EnsureSuccess(handle, operation, status);
            return value ?? string.Empty;
        });
    }

    private static string? TryReadOwnedUtf8String(
        RfNativeEngineHandle handle,
        NativeOwnedUtf8Call nativeCall,
        out RfFfiStatus status)
    {
        status = nativeCall(handle, out var pointer);
        try
        {
            return pointer == IntPtr.Zero
                ? null
                : System.Runtime.InteropServices.Marshal.PtrToStringUTF8(pointer);
        }
        finally
        {
            if (pointer != IntPtr.Zero)
            {
                RfNativeMethods.RfStringFree(pointer);
            }
        }
    }

    private static void EnsureSuccess(RfNativeEngineHandle handle, string operation, RfFfiStatus status)
    {
        if (status == RfFfiStatus.Ok)
        {
            return;
        }

        throw RadishFlowNativeException.Create(
            operation,
            status,
            TryReadOwnedUtf8String(handle, RfNativeMethods.EngineLastErrorMessage, out _),
            TryReadOwnedUtf8String(handle, RfNativeMethods.EngineLastErrorJson, out _));
    }

    private delegate RfFfiStatus NativeOwnedUtf8Call(RfNativeEngineHandle handle, out nint pointer);
}
