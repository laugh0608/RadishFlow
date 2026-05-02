using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Thermo;

[ComImport]
[ComVisible(false)]
[Guid(CapeOpenInterfaceIds.ICapeThermoEquilibriumRoutine)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeThermoEquilibriumRoutine
{
    [DispId(1)]
    void CalcEquilibrium(object? specification1, object? specification2, string solutionType);

    [DispId(2)]
    [return: MarshalAs(UnmanagedType.VariantBool)]
    bool CheckEquilibriumSpec(object? specification1, object? specification2, string solutionType);
}
