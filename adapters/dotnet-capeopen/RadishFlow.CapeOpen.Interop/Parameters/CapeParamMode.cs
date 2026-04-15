using System;
using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Parameters;

[Serializable]
[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.CapeParamMode)]
public enum CapeParamMode
{
    CAPE_INPUT = 0,
    CAPE_OUTPUT = 1,
    CAPE_INPUT_OUTPUT = 2,
}
