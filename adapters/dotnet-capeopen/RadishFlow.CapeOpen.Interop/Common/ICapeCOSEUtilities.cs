using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Common;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeCOSEUtilities)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeCOSEUtilities
{
    [DispId(1)]
    object NamedValueList
    {
        [return: MarshalAs(UnmanagedType.Struct)]
        get;
    }

    [DispId(2)]
    object NamedValue(string value);
}
