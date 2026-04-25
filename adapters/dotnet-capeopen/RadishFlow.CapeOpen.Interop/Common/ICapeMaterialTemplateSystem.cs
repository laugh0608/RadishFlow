using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Common;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeMaterialTemplateSystem)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeMaterialTemplateSystem
{
    [DispId(1)]
    object MaterialTemplates
    {
        [return: MarshalAs(UnmanagedType.Struct)]
        get;
    }

    [DispId(2)]
    [return: MarshalAs(UnmanagedType.IDispatch)]
    object? CreateMaterialTemplate(string materialTemplateName);
}
