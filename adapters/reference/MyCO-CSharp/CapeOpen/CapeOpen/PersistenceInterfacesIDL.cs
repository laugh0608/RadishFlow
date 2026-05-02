using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

//namespace CapeOpen
//{

//    [System.Runtime.InteropServices.ComImport()]
//    [System.Runtime.InteropServices.InterfaceTypeAttribute(1)]
//    [System.Runtime.InteropServices.Guid("0000010c-0000-0000-C000-000000000046")]
//    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
//    public interface IPersist
//    {
//        void GetClassID(out Guid pClassID);
//    };

//    [System.Runtime.InteropServices.ComImport()]
//    [System.Runtime.InteropServices.InterfaceTypeAttribute(1)]
//    [System.Runtime.InteropServices.Guid("00000109-0000-0000-C000-000000000046")]
//    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
//    public interface IPersistStream : IPersist
//    {
//        [System.Runtime.InteropServices.PreserveSig]
//        int IsDirty();
//        void Load(System.Runtime.InteropServices.ComTypes.IStream pStm);
//        void Save(System.Runtime.InteropServices.ComTypes.IStream pStm,
//            [System.Runtime.InteropServices.InAttribute, System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.Bool)] bool fClearDirty);
//        void GetSizeMax(out long pcbSize);

//    };

//    [System.Runtime.InteropServices.ComImport()]
//    [System.Runtime.InteropServices.InterfaceTypeAttribute(1)]
//    [System.Runtime.InteropServices.Guid("7FD52380-4E07-101B-AE2D-08002B2EC713")]
//    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
//    public interface IPersistStreamInit : IPersist
//    {
//        [System.Runtime.InteropServices.PreserveSig]
//        int IsDirty();
//        void Load(System.Runtime.InteropServices.ComTypes.IStream pStm);
//        void Save(System.Runtime.InteropServices.ComTypes.IStream pStm,
//            [System.Runtime.InteropServices.In, System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.Bool)] bool fClearDirty);
//        void GetSizeMax(out long pcbSize);
//        void InitNew();
//    };
//}

