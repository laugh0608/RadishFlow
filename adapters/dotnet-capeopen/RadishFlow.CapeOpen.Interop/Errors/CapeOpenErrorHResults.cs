namespace RadishFlow.CapeOpen.Interop.Errors;

public static class CapeOpenErrorHResults
{
    // Source: adapters/reference/CapeOpenMixerExample_CSharp/CapeOpen/errorIDL.cs
    public const int ECapeUnknown = unchecked((int)0x80040501);
    public const int ECapeData = unchecked((int)0x80040502);
    public const int ECapeLicenceError = unchecked((int)0x80040503);
    public const int ECapeBadCOParameter = unchecked((int)0x80040504);
    public const int ECapeInvalidArgument = unchecked((int)0x80040506);
    public const int ECapeNoImpl = unchecked((int)0x80040509);
    public const int ECapeComputation = unchecked((int)0x8004050B);
    public const int ECapeOutOfResources = unchecked((int)0x8004050C);
    public const int ECapeTimeOut = unchecked((int)0x8004050E);
    public const int ECapeFailedInitialisation = unchecked((int)0x8004050F);
    public const int ECapeSolvingError = unchecked((int)0x80040510);
    public const int ECapeBadInvOrder = unchecked((int)0x80040511);
}
