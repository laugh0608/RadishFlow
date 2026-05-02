using System.Runtime.InteropServices;
using System.Runtime.CompilerServices;
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
    [SpecialName]
    void set_SimulationContext(IntPtr value);

    [DispId(3)]
    void Initialize();

    [DispId(4)]
    void Terminate();

    [DispId(5)]
    [PreserveSig]
    int Edit();

    [DispId(2)]
    [SpecialName]
    [return: MarshalAs(UnmanagedType.IDispatch)]
    IntPtr get_SimulationContext();
}
