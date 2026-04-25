using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.Interop.Ole;

[ComVisible(true)]
[Guid(ComOleInterfaceIds.IOleObject)]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IOleObject
{
    [PreserveSig]
    int SetClientSite(IntPtr clientSite);

    [PreserveSig]
    int GetClientSite(out IntPtr clientSite);

    [PreserveSig]
    int SetHostNames(
        [MarshalAs(UnmanagedType.LPWStr)] string? containerApplication,
        [MarshalAs(UnmanagedType.LPWStr)] string? containerObject);

    [PreserveSig]
    int Close(uint saveOption);

    [PreserveSig]
    int SetMoniker(uint whichMoniker, IntPtr moniker);

    [PreserveSig]
    int GetMoniker(uint assign, uint whichMoniker, out IntPtr moniker);

    [PreserveSig]
    int InitFromData(IntPtr dataObject, [MarshalAs(UnmanagedType.Bool)] bool creation, uint reserved);

    [PreserveSig]
    int GetClipboardData(uint reserved, out IntPtr dataObject);

    [PreserveSig]
    int DoVerb(int verb, IntPtr message, IntPtr activeSite, int index, IntPtr parentWindow, IntPtr positionRectangle);

    [PreserveSig]
    int EnumVerbs(out IntPtr enumOleVerb);

    [PreserveSig]
    int Update();

    [PreserveSig]
    int IsUpToDate();

    [PreserveSig]
    int GetUserClassID(out Guid classId);

    [PreserveSig]
    int GetUserType(uint formOfType, out IntPtr userType);

    [PreserveSig]
    int SetExtent(uint drawAspect, ref OleSize size);

    [PreserveSig]
    int GetExtent(uint drawAspect, out OleSize size);

    [PreserveSig]
    int Advise(IntPtr adviseSink, out uint connection);

    [PreserveSig]
    int Unadvise(uint connection);

    [PreserveSig]
    int EnumAdvise(out IntPtr enumAdvise);

    [PreserveSig]
    int GetMiscStatus(uint aspect, out uint status);

    [PreserveSig]
    int SetColorScheme(IntPtr logPalette);
}
