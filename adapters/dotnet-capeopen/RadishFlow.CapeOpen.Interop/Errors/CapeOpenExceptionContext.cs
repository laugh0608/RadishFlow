namespace RadishFlow.CapeOpen.Interop.Errors;

public sealed record CapeOpenExceptionContext(
    string InterfaceName,
    string Scope,
    string Operation,
    string? MoreInfo = null,
    string? DiagnosticJson = null,
    string? NativeStatus = null,
    string? RequestedOperation = null,
    string? ParameterName = null,
    object? Parameter = null);
