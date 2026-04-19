using System;
using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Unit;

[Serializable]
[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.CapePortDirection)]
public enum CapePortDirection
{
    CAPE_INLET = 0,
    CAPE_OUTLET = 1,
    CAPE_INLET_OUTLET = 2,
}
