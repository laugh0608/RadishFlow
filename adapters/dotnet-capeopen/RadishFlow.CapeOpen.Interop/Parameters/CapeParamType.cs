using System;
using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Parameters;

[Serializable]
[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.CapeParamType)]
public enum CapeParamType
{
    CAPE_REAL = 0,
    CAPE_INT = 1,
    CAPE_OPTION = 2,
    CAPE_BOOLEAN = 3,
    CAPE_ARRAY = 4,
}
