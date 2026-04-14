using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.Interop.Errors;

public abstract class CapeOpenException : COMException, ECapeRoot, ECapeUser
{
    protected CapeOpenException(
        string errorName,
        string description,
        int hresult,
        CapeOpenExceptionContext context,
        Exception? innerException = null)
        : base(description, innerException)
    {
        ErrorName = errorName;
        Description = description;
        HResult = hresult;
        InterfaceName = context.InterfaceName;
        Scope = context.Scope;
        Operation = context.Operation;
        MoreInfo = context.MoreInfo;
        DiagnosticJson = context.DiagnosticJson;
        NativeStatus = context.NativeStatus;
        RequestedOperation = context.RequestedOperation;
        ParameterName = context.ParameterName;
        Parameter = context.Parameter;
    }

    public string ErrorName { get; }

    public string Name => ErrorName;

    public string Description { get; }

    public int Code => HResult;

    public new int ErrorCode => Code;

    public string InterfaceName { get; }

    public string Scope { get; }

    public string Operation { get; }

    public string? MoreInfo { get; }

    public string? DiagnosticJson { get; }

    public string? NativeStatus { get; }

    public string? RequestedOperation { get; }

    public string? ParameterName { get; }

    public object? Parameter { get; }
}
