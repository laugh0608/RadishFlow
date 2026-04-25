using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Common;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeIdentification)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeIdentification
{
    [DispId(1)]
    string ComponentName { get; set; }

    [DispId(2)]
    string ComponentDescription { get; set; }
}
