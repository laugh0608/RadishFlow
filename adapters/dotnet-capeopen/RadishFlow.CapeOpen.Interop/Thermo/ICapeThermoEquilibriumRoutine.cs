using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Thermo;

[ComImport]
[ComVisible(false)]
[Guid(CapeOpenInterfaceIds.ICapeThermoEquilibriumRoutine)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ICapeThermoEquilibriumRoutine
{
    [DispId(1)]
    void CalcEquilibrium(string[] specification1, string[] specification2, string solutionType);

    [DispId(2)]
    bool CheckEquilibriumSpec(string[] specification1, string[] specification2, string solutionType);
}
