using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.Interop.Persistence;

[ComVisible(true)]
[Guid(ComPersistenceInterfaceIds.IPersistStorage)]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IPersistStorage
{
    [PreserveSig]
    int GetClassID(out Guid classId);

    [PreserveSig]
    int IsDirty();

    [PreserveSig]
    int InitNew(IntPtr storage);

    [PreserveSig]
    int Load(IntPtr storage);

    [PreserveSig]
    int Save(IntPtr storage, [MarshalAs(UnmanagedType.Bool)] bool sameAsLoad);

    [PreserveSig]
    int SaveCompleted(IntPtr storage);

    [PreserveSig]
    int HandsOffStorage();
}
