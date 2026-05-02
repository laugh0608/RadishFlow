using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;
using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

[ComVisible(true)]
[Guid(PlaceholderComClassIds.ParameterSpecificationPlaceholder)]
[ClassInterface(ClassInterfaceType.None)]
[ComDefaultInterface(typeof(ICapeParameterSpec))]
public sealed class UnitOperationParameterSpecificationPlaceholder : ICapeParameterSpec, ICapeOptionParameterSpec
{
    private const string InterfaceName = nameof(ICapeParameterSpec);
    private const string OptionSpecInterfaceName = nameof(ICapeOptionParameterSpec);
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
            var type = _definition.SpecificationType;
            UnitOperationComTrace.Write(
                $"{_definition.Name}.{InterfaceName}.{nameof(Type)}",
                "get-exit",
                type.ToString());
            return type;
        }
    }

    public double[] Dimensionality
    {
        get
        {
            EnsureOwnerAccess(nameof(Dimensionality));
            var dimensionality = _dimensionality.ToArray();
            UnitOperationComTrace.Write(
                $"{_definition.Name}.{InterfaceName}.{nameof(Dimensionality)}",
                "get-exit",
                $"length={dimensionality.Length}");
            return dimensionality;
        }
    }

    public string DefaultValue
    {
        get
        {
            EnsureOwnerAccess(OptionSpecInterfaceName, nameof(DefaultValue));
            var value = _definition.DefaultValue ?? string.Empty;
            UnitOperationComTrace.Write(
                $"{_definition.Name}.{OptionSpecInterfaceName}.{nameof(DefaultValue)}",
                "get-exit",
                value);
            return value;
        }
    }

    public object OptionList
    {
        get
        {
            EnsureOwnerAccess(OptionSpecInterfaceName, nameof(OptionList));
            UnitOperationComTrace.Write(
                $"{_definition.Name}.{OptionSpecInterfaceName}.{nameof(OptionList)}",
                "get-exit",
                "length=0");
            return Array.Empty<string>();
        }
    }

    public bool RestrictedToList
    {
        get
        {
            EnsureOwnerAccess(OptionSpecInterfaceName, nameof(RestrictedToList));
            UnitOperationComTrace.Write(
                $"{_definition.Name}.{OptionSpecInterfaceName}.{nameof(RestrictedToList)}",
                "get-exit",
                "False");
            return false;
        }
    }

    public bool Validate(string value, ref string message)
    {
        EnsureOwnerAccess(OptionSpecInterfaceName, nameof(Validate), value);
        UnitOperationComTrace.Write(
            $"{_definition.Name}.{OptionSpecInterfaceName}.{nameof(Validate)}",
            "enter",
            value);
        message = $"Option parameter `{_definition.Name}` accepts unrestricted string values.";
        UnitOperationComTrace.Write(
            $"{_definition.Name}.{OptionSpecInterfaceName}.{nameof(Validate)}",
            "exit",
            "True");
        return true;
    }

    private void EnsureOwnerAccess(string operation)
    {
        EnsureOwnerAccess(InterfaceName, operation, null);
    }

    private void EnsureOwnerAccess(string interfaceName, string operation, object? parameter = null)
    {
        _ensureOwnerAccess?.Invoke(interfaceName, operation, _definition.Name, parameter);
    }
}
