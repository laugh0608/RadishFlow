using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Common;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeUtilities)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeUtilities
{
    [DispId(1)]
    object? Parameters
    {
        [return: MarshalAs(UnmanagedType.IDispatch)]
        get;
    }

    [DispId(2)]
    IntPtr SimulationContext
    {
        [return: MarshalAs(UnmanagedType.IDispatch)]
        get;
        set;
    }

    [DispId(3)]
    void Initialize();

    [DispId(4)]
    void Terminate();

    [DispId(5)]
    [PreserveSig]
    int Edit();
}
