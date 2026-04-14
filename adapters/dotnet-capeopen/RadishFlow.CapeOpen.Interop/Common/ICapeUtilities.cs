using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Common;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeUtilities)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ICapeUtilities
{
    [DispId(1)]
    object? Parameters
    {
        [return: MarshalAs(UnmanagedType.IDispatch)]
        get;
    }

    [DispId(2)]
    object? SimulationContext
    {
        [return: MarshalAs(UnmanagedType.IDispatch)]
        get;

        [param: MarshalAs(UnmanagedType.IDispatch)]
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
