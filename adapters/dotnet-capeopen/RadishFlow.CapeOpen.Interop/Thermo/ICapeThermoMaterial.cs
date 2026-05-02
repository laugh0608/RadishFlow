using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Thermo;

[ComImport]
[ComVisible(false)]
[Guid(CapeOpenInterfaceIds.ICapeThermoMaterial)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeThermoMaterial
{
    [DispId(1)]
    void ClearAllProps();

    [DispId(2)]
    void CopyFromMaterial([MarshalAs(UnmanagedType.IDispatch)] ref object source);

    [DispId(3)]
    [return: MarshalAs(UnmanagedType.IDispatch)]
    object CreateMaterial();

    [DispId(4)]
    void GetOverallProp(string property, string? basis, ref object? results);

    [DispId(5)]
    void GetOverallTPFraction(ref double temperature, ref double pressure, ref object? composition);

    [DispId(6)]
    void GetPresentPhases(ref object? phaseLabels, ref object? phaseStatus);

    [DispId(7)]
    void GetSinglePhaseProp(string property, string phaseLabel, string? basis, ref object? results);

    [DispId(8)]
    void GetTPFraction(string phaseLabel, ref double temperature, ref double pressure, ref object? composition);

    [DispId(9)]
    void GetTwoPhaseProp(string property, object? phaseLabels, string? basis, ref object? results);

    [DispId(10)]
    void SetOverallProp(string property, string? basis, object? values);

    [DispId(11)]
    void SetPresentPhases(object? phaseLabels, object? phaseStatus);

    [DispId(12)]
    void SetSinglePhaseProp(string property, string phaseLabel, string? basis, object? values);

    [DispId(13)]
    void SetTwoPhaseProp(string property, object? phaseLabels, string? basis, object? values);
}
