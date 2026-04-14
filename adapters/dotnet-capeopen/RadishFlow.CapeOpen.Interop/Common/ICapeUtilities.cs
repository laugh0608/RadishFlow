using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Common;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeUtilities)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ICapeUtilities
{
    [DispId(1)]
    object? Parameters { get; }

    [DispId(2)]
    object? SimulationContext { set; }

    [DispId(3)]
    void Initialize();

    [DispId(4)]
    void Edit();

    [DispId(5)]
    void Terminate();
}
