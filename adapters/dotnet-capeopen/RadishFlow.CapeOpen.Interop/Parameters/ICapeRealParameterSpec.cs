using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Parameters;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeRealParameterSpec)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeRealParameterSpec
{
    [DispId(1)]
    double SIDefaultValue { get; set; }

    [DispId(2)]
    double SILowerBound { get; set; }

    [DispId(3)]
    double SIUpperBound { get; set; }

    [DispId(4)]
    [return: MarshalAs(UnmanagedType.VariantBool)]
    bool SIValidate(double value, [MarshalAs(UnmanagedType.BStr)] ref string message);

    [DispId(1)]
    double DimensionedDefaultValue { get; set; }

    [DispId(2)]
    double DimensionedLowerBound { get; set; }

    [DispId(3)]
    double DimensionedUpperBound { get; set; }

    [DispId(4)]
    [return: MarshalAs(UnmanagedType.VariantBool)]
    bool DimensionedValidate(double value, [MarshalAs(UnmanagedType.BStr)] ref string message);
}
