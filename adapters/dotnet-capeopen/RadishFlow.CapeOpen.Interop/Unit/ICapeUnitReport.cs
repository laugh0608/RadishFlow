using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Unit;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeUnitReport)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ICapeUnitReport
{
    [DispId(1)]
    object reports
    {
        [return: MarshalAs(UnmanagedType.Struct)]
        get;
    }

    [DispId(2)]
    string selectedReport { get; set; }

    [DispId(3)]
    void ProduceReport([MarshalAs(UnmanagedType.BStr)] ref string reportContent);
}
