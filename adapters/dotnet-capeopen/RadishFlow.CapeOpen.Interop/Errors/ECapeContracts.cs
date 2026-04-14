namespace RadishFlow.CapeOpen.Interop.Errors;

public interface ECapeRoot
{
    string InterfaceName { get; }

    string Scope { get; }

    string Operation { get; }

    string? MoreInfo { get; }
}

public interface ECapeUser : ECapeRoot
{
    string Description { get; }

    int ErrorCode { get; }
}

public interface ECapeUnknown : ECapeUser;

public interface ECapeData : ECapeUser;

public interface ECapeLicenceError : ECapeUser;

public interface ECapeBadInvOrder : ECapeUser;

public interface ECapeInvalidArgument : ECapeUser;

public interface ECapeOutOfResources : ECapeUser;

public interface ECapeNoImpl : ECapeUser;

public interface ECapeTimeOut : ECapeUser;

public interface ECapeFailedInitialisation : ECapeUser;

public interface ECapeSolvingError : ECapeUser;

public interface ECapeBadCOParameter : ECapeUser;
