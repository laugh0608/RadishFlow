using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Thermo;

[ComImport]
[ComVisible(false)]
[Guid(CapeOpenInterfaceIds.ICapeThermoMaterial)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ICapeThermoMaterial
{
    [DispId(1)]
    void ClearAllProps();

    [DispId(10)]
    void SetOverallProp(string property, string basis, double[] values);

    [DispId(11)]
    void SetPresentPhases(string[] phaseLabels, CapePhaseStatus[] phaseStatus);

    [DispId(12)]
    void SetSinglePhaseProp(string property, string phaseLabel, string basis, double[] values);
}
