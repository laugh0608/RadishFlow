namespace RadishFlow.CapeOpen.Interop.Errors;

public sealed class CapeUnknownException : CapeOpenException, ECapeUnknown
{
    public CapeUnknownException(string description, CapeOpenExceptionContext context)
        : base("ECapeUnknown", description, CapeOpenErrorHResults.ECapeUnknown, context)
    {
    }
}

public sealed class CapeDataException : CapeOpenException, ECapeData
{
    public CapeDataException(string description, CapeOpenExceptionContext context)
        : base("ECapeData", description, CapeOpenErrorHResults.ECapeData, context)
    {
    }
}

public sealed class CapeLicenceErrorException : CapeOpenException, ECapeLicenceError
{
    public CapeLicenceErrorException(string description, CapeOpenExceptionContext context)
        : base("ECapeLicenceError", description, CapeOpenErrorHResults.ECapeLicenceError, context)
    {
    }
}

public sealed class CapeBadInvocationOrderException : CapeOpenException, ECapeBadInvOrder
{
    public CapeBadInvocationOrderException(string description, CapeOpenExceptionContext context)
        : base("ECapeBadInvOrder", description, CapeOpenErrorHResults.ECapeBadInvOrder, context)
    {
    }
}

public sealed class CapeInvalidArgumentException : CapeOpenException, ECapeInvalidArgument
{
    public CapeInvalidArgumentException(string description, CapeOpenExceptionContext context)
        : base("ECapeInvalidArgument", description, CapeOpenErrorHResults.ECapeInvalidArgument, context)
    {
    }
}

public sealed class CapeOutOfResourcesException : CapeOpenException, ECapeOutOfResources
{
    public CapeOutOfResourcesException(string description, CapeOpenExceptionContext context)
        : base("ECapeOutOfResources", description, CapeOpenErrorHResults.ECapeOutOfResources, context)
    {
    }
}

public sealed class CapeNoImplementationException : CapeOpenException, ECapeNoImpl
{
    public CapeNoImplementationException(string description, CapeOpenExceptionContext context)
        : base("ECapeNoImpl", description, CapeOpenErrorHResults.ECapeNoImpl, context)
    {
    }
}

public sealed class CapeTimeOutException : CapeOpenException, ECapeTimeOut
{
    public CapeTimeOutException(string description, CapeOpenExceptionContext context)
        : base("ECapeTimeOut", description, CapeOpenErrorHResults.ECapeTimeOut, context)
    {
    }
}

public sealed class CapeFailedInitialisationException : CapeOpenException, ECapeFailedInitialisation
{
    public CapeFailedInitialisationException(string description, CapeOpenExceptionContext context)
        : base("ECapeFailedInitialisation", description, CapeOpenErrorHResults.ECapeFailedInitialisation, context)
    {
    }
}

public sealed class CapeSolvingException : CapeOpenException, ECapeSolvingError
{
    public CapeSolvingException(string description, CapeOpenExceptionContext context)
        : base("ECapeSolvingError", description, CapeOpenErrorHResults.ECapeSolvingError, context)
    {
    }
}

public sealed class CapeBadCoParameterException : CapeOpenException, ECapeBadCOParameter
{
    public CapeBadCoParameterException(string description, CapeOpenExceptionContext context)
        : base("ECapeBadCOParameter", description, CapeOpenErrorHResults.ECapeBadCOParameter, context)
    {
    }
}
