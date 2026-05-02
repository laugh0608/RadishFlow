using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Parameters;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeParameterSpec)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeParameterSpec
{
    [DispId(1)]
    CapeParamType Type { get; }

    [DispId(2)]
    double[] Dimensionality { get; }
}
