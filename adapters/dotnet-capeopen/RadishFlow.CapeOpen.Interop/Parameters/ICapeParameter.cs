using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;
using RadishFlow.CapeOpen.Interop.Unit;

namespace RadishFlow.CapeOpen.Interop.Parameters;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeParameter)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeParameter
{
    [DispId(1)]
    object Specification
    {
        [return: MarshalAs(UnmanagedType.IDispatch)]
        get;
    }

    [DispId(2)]
    object? value { get; set; }

    [DispId(3)]
    CapeValidationStatus ValStatus { get; }

    [DispId(4)]
    CapeParamMode Mode { get; set; }

    [DispId(5)]
    [return: MarshalAs(UnmanagedType.VariantBool)]
    bool Validate([MarshalAs(UnmanagedType.BStr)] ref string message);

    [DispId(6)]
    void Reset();
}
