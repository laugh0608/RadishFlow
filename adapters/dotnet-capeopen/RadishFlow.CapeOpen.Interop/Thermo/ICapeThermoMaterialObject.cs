using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Thermo;

[ComImport]
[ComVisible(false)]
[Guid(CapeOpenInterfaceIds.ICapeThermoMaterialObject)]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface ICapeThermoMaterialObject
{
    object? ComponentIds { get; }

    object? PhaseIds { get; }

    object? GetUniversalConstant(object? props);

    object? GetComponentConstant(object? props, object? compIds);

    void CalcProp(object? props, object? phases, string calcType);

    object? GetProp(string property, string phase, object? compIds, string calcType, string basis);

    void SetProp(string property, string phase, object? compIds, string calcType, string basis, object? values);

    void CalcEquilibrium(string flashType, object? props);
}
