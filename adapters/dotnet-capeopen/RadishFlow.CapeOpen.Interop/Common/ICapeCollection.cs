using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Common;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeCollection)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeCollection
{
    [DispId(1)]
    [return: MarshalAs(UnmanagedType.IDispatch)]
    object Item(object index);

    [DispId(2)]
    int Count();
}
