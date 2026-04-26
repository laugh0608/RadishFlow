using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Thermo;

[ComImport]
[ComVisible(false)]
[Guid(CapeOpenInterfaceIds.ICapeThermoCompounds)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeThermoCompounds
{
    [DispId(1)]
    object? GetCompoundConstant(object? props, object? compIds);

    [DispId(2)]
    void GetCompoundList(
        ref object? compIds,
        ref object? formulae,
        ref object? names,
        ref object? boilTemps,
        ref object? molwts,
        ref object? casnos);
}
