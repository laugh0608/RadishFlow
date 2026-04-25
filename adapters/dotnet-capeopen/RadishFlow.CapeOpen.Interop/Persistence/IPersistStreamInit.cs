using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;

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
    int Load([MarshalAs(UnmanagedType.Interface)] IStream? stream);

    [PreserveSig]
    int Save([MarshalAs(UnmanagedType.Interface)] IStream? stream, [MarshalAs(UnmanagedType.Bool)] bool clearDirty);

    [PreserveSig]
    int GetSizeMax(out long size);

    [PreserveSig]
    int InitNew();
}
