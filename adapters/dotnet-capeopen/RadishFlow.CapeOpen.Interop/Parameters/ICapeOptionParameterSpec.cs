using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Parameters;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeOptionParameterSpec)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeOptionParameterSpec
{
    [DispId(1)]
    string DefaultValue { get; }

    [DispId(2)]
    object OptionList { get; }

    [DispId(3)]
    bool RestrictedToList
    {
        [return: MarshalAs(UnmanagedType.VariantBool)]
        get;
    }

    [DispId(4)]
    [return: MarshalAs(UnmanagedType.VariantBool)]
    bool Validate([MarshalAs(UnmanagedType.BStr)] string value, [MarshalAs(UnmanagedType.BStr)] ref string message);
}
