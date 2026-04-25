using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.Interop.Persistence;

[ComVisible(true)]
[Guid(ComPersistenceInterfaceIds.IPersistStreamInit)]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IPersistStreamInit
{
    [PreserveSig]
    int GetClassID(out Guid classId);

    [PreserveSig]
    int IsDirty();

    [PreserveSig]
    int Load(IntPtr stream);

    [PreserveSig]
    int Save(IntPtr stream, [MarshalAs(UnmanagedType.Bool)] bool clearDirty);

    [PreserveSig]
    int GetSizeMax(out long size);

    [PreserveSig]
    int InitNew();
}
