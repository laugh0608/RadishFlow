using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Unit;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeUnitPort)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ICapeUnitPort
{
    [DispId(1)]
    CapePortType portType { get; }

    [DispId(2)]
    CapePortDirection direction { get; }

    [DispId(3)]
    object? connectedObject
    {
        [return: MarshalAs(UnmanagedType.IDispatch)]
        get;
    }

    [DispId(4)]
    void Connect([MarshalAs(UnmanagedType.IDispatch)] object objectToConnect);

    [DispId(5)]
    void Disconnect();
}
