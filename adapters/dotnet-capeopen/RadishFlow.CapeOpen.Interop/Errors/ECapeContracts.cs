using System.Runtime.InteropServices;
using RadishFlow.CapeOpen.Interop.Guids;

namespace RadishFlow.CapeOpen.Interop.Errors;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ECapeRoot)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ECapeRoot
{
    [DispId(1)]
    string Name { get; }
}

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ECapeUser)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ECapeUser
{
    [DispId(1)]
    int Code { get; }

    [DispId(2)]
    string Description { get; }

    [DispId(3)]
    string Scope { get; }

    [DispId(4)]
    string InterfaceName { get; }

    [DispId(5)]
    string Operation { get; }

    [DispId(6)]
    string? MoreInfo { get; }
}

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ECapeUnknown)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ECapeUnknown;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ECapeData)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ECapeData;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ECapeLicenceError)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ECapeLicenceError;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ECapeBadCOParameter)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ECapeBadCOParameter
{
    [DispId(1)]
    string? ParameterName { get; }

    [DispId(2)]
    object? Parameter
    {
        [return: MarshalAs(UnmanagedType.IDispatch)]
        get;
    }
}

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ECapeInvalidArgument)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ECapeInvalidArgument;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ECapeOutOfResources)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ECapeOutOfResources;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ECapeNoImpl)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ECapeNoImpl;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ECapeTimeOut)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ECapeTimeOut;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ECapeFailedInitialisation)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ECapeFailedInitialisation;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ECapeSolvingError)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ECapeSolvingError;

[ComVisible(true)]
[Guid(CapeOpenInterfaceIds.ECapeBadInvOrder)]
[InterfaceType(ComInterfaceType.InterfaceIsIDispatch)]
public interface ECapeBadInvOrder
{
    [DispId(1)]
    string? RequestedOperation { get; }
}
