using System;
using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Unit;

[Serializable]
[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.CapePortType)]
public enum CapePortType
{
    CAPE_MATERIAL = 0,
    CAPE_ENERGY = 1,
    CAPE_INFORMATION = 2,
    CAPE_ANY = 3,
}
