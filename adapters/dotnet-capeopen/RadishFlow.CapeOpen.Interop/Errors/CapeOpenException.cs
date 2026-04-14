using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.Interop.Errors;

public abstract class CapeOpenException : COMException, ECapeUser
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
    }

    public string ErrorName { get; }

    public string Description { get; }

    public new int ErrorCode => HResult;

    public string InterfaceName { get; }

    public string Scope { get; }

    public string Operation { get; }

    public string? MoreInfo { get; }

    public string? DiagnosticJson { get; }

    public string? NativeStatus { get; }
}
