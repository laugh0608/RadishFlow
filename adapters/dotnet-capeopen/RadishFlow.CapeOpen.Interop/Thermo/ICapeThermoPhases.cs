using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Thermo;

// CAPE-OPEN Thermo 1.1, section 6.4. Consumed from host materials; not exposed by the PMC.
[ComImport]
[ComVisible(false)]
[Guid(CapeOpenInterfaceIds.ICapeThermoPhases)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeThermoPhases
{
    [DispId(1)]
    int GetNumPhases();

    [DispId(2)]
    object? GetPhaseInfo(string phaseLabel, string phaseAttribute);

    [DispId(3)]
    void GetPhaseList([In, Out] ref object? phaseLabels, [In, Out] ref object? stateOfAggregation,
        [In, Out] ref object? keyCompoundId);
}
