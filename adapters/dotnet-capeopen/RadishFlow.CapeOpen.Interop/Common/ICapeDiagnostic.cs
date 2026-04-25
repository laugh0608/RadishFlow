using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Common;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ICapeDiagnostic)]
[InterfaceType(ComInterfaceType.InterfaceIsDual)]
public interface ICapeDiagnostic
{
    [DispId(1)]
    void PopUpMessage(string message);

    [DispId(2)]
    void LogMessage(string message);
}
