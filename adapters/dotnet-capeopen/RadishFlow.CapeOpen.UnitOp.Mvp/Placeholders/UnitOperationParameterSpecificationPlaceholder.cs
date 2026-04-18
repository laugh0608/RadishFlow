using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Parameters;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

internal sealed class UnitOperationParameterSpecificationPlaceholder : ICapeParameterSpec
{
    private const string InterfaceName = nameof(ICapeParameterSpec);
    private readonly Action<string, string, string?, object?>? _ensureOwnerAccess;
    private readonly string _parameterName;
    private readonly CapeParamType _type;
    private readonly double[] _dimensionality;

    public UnitOperationParameterSpecificationPlaceholder(
        string parameterName,
        CapeParamType type,
        double[] dimensionality,
        Action<string, string, string?, object?>? ensureOwnerAccess = null)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(parameterName);
        ArgumentNullException.ThrowIfNull(dimensionality);

        _parameterName = parameterName;
        _type = type;
        _dimensionality = [.. dimensionality];
        _ensureOwnerAccess = ensureOwnerAccess;
    }

    public CapeParamType Type
    {
        get
        {
            EnsureOwnerAccess(nameof(Type));
            return _type;
        }
    }

    public double[] Dimensionality
    {
        get
        {
            EnsureOwnerAccess(nameof(Dimensionality));
            return [.. _dimensionality];
        }
    }

    private void EnsureOwnerAccess(string operation)
    {
        _ensureOwnerAccess?.Invoke(InterfaceName, operation, _parameterName, null);
    }
}
