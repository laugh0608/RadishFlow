using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Unit;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeUnit)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ICapeUnit
{
    [DispId(1)]
    object? Ports
    {
        [return: MarshalAs(UnmanagedType.IDispatch)]
        get;
    }

    [DispId(2)]
    CapeValidationStatus ValStatus { get; }

    [DispId(3)]
    void Calculate();

    [DispId(4)]
    [return: MarshalAs(UnmanagedType.VariantBool)]
    bool Validate([MarshalAs(UnmanagedType.BStr)] ref string message);
}
