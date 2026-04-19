using Microsoft.Win32.SafeHandles;

namespace RadishFlow.CapeOpen.Adapter;

internal sealed class RfNativeEngineHandle : SafeHandleZeroOrMinusOneIsInvalid
{
    public RfNativeEngineHandle()
        : base(ownsHandle: true)
    {
    }

    public static RfNativeEngineHandle FromNative(nint handle)
    {
        var safeHandle = new RfNativeEngineHandle();
        safeHandle.SetHandle(handle);
        return safeHandle;
    }

    protected override bool ReleaseHandle()
    {
        RfNativeMethods.EngineDestroy(handle);
        return true;
    }
}
