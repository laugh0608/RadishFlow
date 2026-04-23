using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;
using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

[ComVisible(true)]
[Guid(PlaceholderComClassIds.ParameterSpecificationPlaceholder)]
[ClassInterface(ClassInterfaceType.None)]
public sealed class UnitOperationParameterSpecificationPlaceholder : ICapeParameterSpec
{
    private const string InterfaceName = nameof(ICapeParameterSpec);
    private readonly Action<string, string, string?, object?>? _ensureOwnerAccess;
    private readonly UnitOperationParameterDefinition _definition;
    private readonly double[] _dimensionality;

    public UnitOperationParameterSpecificationPlaceholder(
        UnitOperationParameterDefinition definition,
        Action<string, string, string?, object?>? ensureOwnerAccess = null)
    {
        ArgumentNullException.ThrowIfNull(definition);

        _definition = definition;
        _dimensionality = [.. definition.SpecificationDimensionality];
        _ensureOwnerAccess = ensureOwnerAccess;
    }

    public CapeParamType Type
    {
        get
        {
            EnsureOwnerAccess(nameof(Type));
            return _definition.SpecificationType;
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
        _ensureOwnerAccess?.Invoke(InterfaceName, operation, _definition.Name, null);
    }
}
