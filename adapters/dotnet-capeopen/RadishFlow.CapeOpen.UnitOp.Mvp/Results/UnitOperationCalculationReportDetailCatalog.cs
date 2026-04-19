namespace RadishFlow.CapeOpen.UnitOp.Mvp.Results;

public static class UnitOperationCalculationReportDetailCatalog
{
    public const string Status = "status";
    public const string HighestSeverity = "highestSeverity";
    public const string DiagnosticCount = "diagnosticCount";
    public const string RelatedUnitIds = "relatedUnitIds";
    public const string RelatedStreamIds = "relatedStreamIds";
    public const string Error = "error";
    public const string Operation = "operation";
    public const string RequestedOperation = "requestedOperation";
    public const string NativeStatus = "nativeStatus";
    public const string DiagnosticCode = "diagnosticCode";
    public const string RelatedPortTarget = "relatedPortTarget";

    private static readonly IReadOnlyList<string> SuccessStableKeyOrderValue =
    [
        Status,
        HighestSeverity,
        DiagnosticCount,
        RelatedUnitIds,
        RelatedStreamIds,
    ];

    private static readonly IReadOnlyList<string> FailureStableKeyOrderValue =
    [
        Error,
        Operation,
        RequestedOperation,
        NativeStatus,
        DiagnosticCode,
        RelatedUnitIds,
        RelatedStreamIds,
        RelatedPortTarget,
    ];

    public static IReadOnlyList<string> SuccessStableKeyOrder => SuccessStableKeyOrderValue;

    public static IReadOnlyList<string> FailureStableKeyOrder => FailureStableKeyOrderValue;

    public static IReadOnlyList<string> GetStableKeyOrder(UnitOperationCalculationReportState state)
    {
        return state switch
        {
            UnitOperationCalculationReportState.None => Array.Empty<string>(),
            UnitOperationCalculationReportState.Success => SuccessStableKeyOrderValue,
            UnitOperationCalculationReportState.Failure => FailureStableKeyOrderValue,
            _ => throw new ArgumentOutOfRangeException(nameof(state)),
        };
    }
}
