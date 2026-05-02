using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;
using System.Text.Json;
using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

[ComVisible(true)]
[Guid(PlaceholderComClassIds.ParameterPlaceholder)]
[ClassInterface(ClassInterfaceType.None)]
[ComDefaultInterface(typeof(ICapeParameter))]
public sealed class UnitOperationParameterPlaceholder : ICapeIdentification, ICapeParameter, ICapeParameterSpec, ICapeOptionParameterSpec
{
    private const string InterfaceName = nameof(ICapeParameter);
    private const string IdentificationInterfaceName = nameof(ICapeIdentification);
    private const string ParameterSpecInterfaceName = nameof(ICapeParameterSpec);
    private const string OptionSpecInterfaceName = nameof(ICapeOptionParameterSpec);
    private readonly Action<string, string, string?, object?>? _ensureOwnerAccess;
    private readonly Action? _onStateChanged;
    private readonly UnitOperationParameterDefinition _definition;
    private readonly UnitOperationParameterSpecificationPlaceholder _specification;
    private string? _value;

    public UnitOperationParameterPlaceholder(
        UnitOperationParameterDefinition definition,
        Action<string, string, string?, object?>? ensureOwnerAccess = null,
        Action? onStateChanged = null)
    {
        ArgumentNullException.ThrowIfNull(definition);

        _definition = definition;
        _value = Normalize(definition.DefaultValue, definition.AllowsEmptyValue);
        _ensureOwnerAccess = ensureOwnerAccess;
        _onStateChanged = onStateChanged;
        _specification = new UnitOperationParameterSpecificationPlaceholder(
            definition: definition,
            ensureOwnerAccess: ensureOwnerAccess);
        ValStatus = CapeValidationStatus.NotValidated;
    }

    public string ComponentName
    {
        get
        {
            EnsureOwnerAccess(IdentificationInterfaceName, nameof(ComponentName));
            UnitOperationComTrace.Write(
                $"{_definition.Name}.{IdentificationInterfaceName}.{nameof(ComponentName)}",
                "get-exit",
                _definition.Name);
            return _definition.Name;
        }
        set => SetImmutableComponentName(value, nameof(ComponentName));
    }

    public string ComponentDescription
    {
        get
        {
            EnsureOwnerAccess(IdentificationInterfaceName, nameof(ComponentDescription));
            UnitOperationComTrace.Write(
                $"{_definition.Name}.{IdentificationInterfaceName}.{nameof(ComponentDescription)}",
                "get-exit",
                _definition.Description);
            return _definition.Description;
        }
        set => SetImmutableComponentDescription(value, nameof(ComponentDescription));
    }

    public bool IsRequired => _definition.IsRequired;

    public UnitOperationParameterValueKind ValueKind
    {
        get
        {
            EnsureOwnerAccess(nameof(ValueKind));
            return _definition.ValueKind;
        }
    }

    public bool AllowsEmptyValue
    {
        get
        {
            EnsureOwnerAccess(nameof(AllowsEmptyValue));
            return _definition.AllowsEmptyValue;
        }
    }

    public string? RequiredCompanionParameterName
    {
        get
        {
            EnsureOwnerAccess(nameof(RequiredCompanionParameterName));
            return _definition.RequiredCompanionParameterName;
        }
    }

    public string? DefaultValue
    {
        get
        {
            EnsureOwnerAccess(nameof(DefaultValue));
            return Normalize(_definition.DefaultValue, _definition.AllowsEmptyValue);
        }
    }

    public string? Value
    {
        get
        {
            EnsureOwnerAccess(nameof(Value));
            return _value;
        }
    }

    public bool IsConfigured
    {
        get
        {
            EnsureOwnerAccess(nameof(IsConfigured));
            return AllowsEmptyValue ? _value is not null : !string.IsNullOrWhiteSpace(_value);
        }
    }

    public object Specification
    {
        get
        {
            EnsureOwnerAccess(nameof(Specification));
            UnitOperationComTrace.Write(
                $"{ComponentName}.{InterfaceName}.{nameof(Specification)}",
                "get-exit",
                _definition.SpecificationType.ToString());
            return _specification;
        }
    }

    public object? value
    {
        get
        {
            EnsureOwnerAccess(nameof(value));
            UnitOperationComTrace.Write(
                $"{ComponentName}.{InterfaceName}.{nameof(value)}",
                "get-exit",
                _value is null ? "null" : "provided");
            return _value;
        }
        set
        {
            EnsureOwnerAccess(nameof(value), parameter: value);
            UnitOperationComTrace.Write(
                $"{ComponentName}.{InterfaceName}.{nameof(value)}",
                "set-enter",
                value is null ? "null" : value.ToString());

            if (value is not null and not string)
            {
                throw new CapeInvalidArgumentException(
                    $"Parameter `{ComponentName}` only accepts string or null values in the MVP runtime.",
                    CreateContext(nameof(value), value));
            }

            SetValueCore((string?)value);
            UnitOperationComTrace.Write(
                $"{ComponentName}.{InterfaceName}.{nameof(value)}",
                "set-exit");
        }
    }

    public CapeValidationStatus ValStatus
    {
        get
        {
            EnsureOwnerAccess(nameof(ValStatus));
            UnitOperationComTrace.Write(
                $"{ComponentName}.{InterfaceName}.{nameof(ValStatus)}",
                "get-exit",
                _valStatus.ToString());
            return _valStatus;
        }
        private set => _valStatus = value;
    }

    public CapeParamMode Mode
    {
        get
        {
            EnsureOwnerAccess(nameof(Mode));
            UnitOperationComTrace.Write(
                $"{ComponentName}.{InterfaceName}.{nameof(Mode)}",
                "get-exit",
                _definition.Mode.ToString());
            return _definition.Mode;
        }
        set
        {
            EnsureOwnerAccess(nameof(Mode), parameter: value);
            UnitOperationComTrace.Write(
                $"{ComponentName}.{InterfaceName}.{nameof(Mode)}",
                "set-enter",
                value.ToString());

            if (_definition.Mode == value)
            {
                UnitOperationComTrace.Write(
                    $"{ComponentName}.{InterfaceName}.{nameof(Mode)}",
                    "set-exit",
                    "unchanged");
                return;
            }

            throw new CapeInvalidArgumentException(
                $"Parameter `{ComponentName}` does not allow Mode mutation in the MVP runtime.",
                CreateContext(nameof(Mode), value));
        }
    }

    public CapeParamType Type
    {
        get
        {
            EnsureOwnerAccess(ParameterSpecInterfaceName, nameof(Type));
            var type = _definition.SpecificationType;
            UnitOperationComTrace.Write(
                $"{ComponentName}.{ParameterSpecInterfaceName}.{nameof(Type)}",
                "get-exit",
                type.ToString());
            return type;
        }
    }

    public double[] Dimensionality
    {
        get
        {
            EnsureOwnerAccess(ParameterSpecInterfaceName, nameof(Dimensionality));
            var dimensionality = _definition.SpecificationDimensionality.ToArray();
            UnitOperationComTrace.Write(
                $"{ComponentName}.{ParameterSpecInterfaceName}.{nameof(Dimensionality)}",
                "get-exit",
                $"length={dimensionality.Length}");
            return dimensionality;
        }
    }

    string ICapeOptionParameterSpec.DefaultValue
    {
        get
        {
            EnsureOwnerAccess(OptionSpecInterfaceName, nameof(ICapeOptionParameterSpec.DefaultValue));
            var value = _definition.DefaultValue ?? string.Empty;
            UnitOperationComTrace.Write(
                $"{ComponentName}.{OptionSpecInterfaceName}.{nameof(ICapeOptionParameterSpec.DefaultValue)}",
                "get-exit",
                value);
            return value;
        }
    }

    object ICapeOptionParameterSpec.OptionList
    {
        get
        {
            EnsureOwnerAccess(OptionSpecInterfaceName, nameof(ICapeOptionParameterSpec.OptionList));
            UnitOperationComTrace.Write(
                $"{ComponentName}.{OptionSpecInterfaceName}.{nameof(ICapeOptionParameterSpec.OptionList)}",
                "get-exit",
                "length=0");
            return Array.Empty<string>();
        }
    }

    bool ICapeOptionParameterSpec.RestrictedToList
    {
        get
        {
            EnsureOwnerAccess(OptionSpecInterfaceName, nameof(ICapeOptionParameterSpec.RestrictedToList));
            UnitOperationComTrace.Write(
                $"{ComponentName}.{OptionSpecInterfaceName}.{nameof(ICapeOptionParameterSpec.RestrictedToList)}",
                "get-exit",
                "False");
            return false;
        }
    }

    bool ICapeOptionParameterSpec.Validate(string value, ref string message)
    {
        EnsureOwnerAccess(OptionSpecInterfaceName, nameof(ICapeOptionParameterSpec.Validate), parameter: value);
        UnitOperationComTrace.Write(
            $"{ComponentName}.{OptionSpecInterfaceName}.{nameof(ICapeOptionParameterSpec.Validate)}",
            "enter",
            value);
        message = $"Option parameter `{ComponentName}` accepts unrestricted string values.";
        UnitOperationComTrace.Write(
            $"{ComponentName}.{OptionSpecInterfaceName}.{nameof(ICapeOptionParameterSpec.Validate)}",
            "exit",
            "True");
        return true;
    }

    public void SetValue(string? value)
    {
        EnsureOwnerAccess(nameof(SetValue), parameter: value);
        SetValueCore(value);
    }

    public bool Validate(ref string message)
    {
        EnsureOwnerAccess(nameof(Validate));
        UnitOperationComTrace.Write($"{ComponentName}.{InterfaceName}.{nameof(Validate)}", "enter");

        if (IsRequired && !IsConfigured)
        {
            message = $"Required parameter `{ComponentName}` is not configured.";
            ValStatus = CapeValidationStatus.Invalid;
            UnitOperationComTrace.Write(
                $"{ComponentName}.{InterfaceName}.{nameof(Validate)}",
                "exit",
                $"False; {message}");
            return false;
        }

        if (IsConfigured && !TryValidateConfiguredValue(out var validationError))
        {
            message = validationError;
            ValStatus = CapeValidationStatus.Invalid;
            UnitOperationComTrace.Write(
                $"{ComponentName}.{InterfaceName}.{nameof(Validate)}",
                "exit",
                $"False; {message}");
            return false;
        }

        message = IsConfigured
            ? $"Parameter `{ComponentName}` is configured as {DescribeValueKind(ValueKind)}."
            : $"Optional parameter `{ComponentName}` is not configured.";
        ValStatus = CapeValidationStatus.Valid;
        UnitOperationComTrace.Write(
            $"{ComponentName}.{InterfaceName}.{nameof(Validate)}",
            "exit",
            $"True; {message}");
        return true;
    }

    public void Reset()
    {
        EnsureOwnerAccess(nameof(Reset));
        SetValueCore(DefaultValue, forceValidationReset: true);
    }

    private void SetValueCore(string? value, bool forceValidationReset = false)
    {
        var normalized = Normalize(value, AllowsEmptyValue);
        if (string.Equals(_value, normalized, StringComparison.Ordinal))
        {
            if (forceValidationReset && ValStatus != CapeValidationStatus.NotValidated)
            {
                MarkChanged();
            }

            return;
        }

        _value = normalized;
        MarkChanged();
    }

    private void MarkChanged()
    {
        ValStatus = CapeValidationStatus.NotValidated;
        _onStateChanged?.Invoke();
    }

    private bool TryValidateConfiguredValue(out string message)
    {
        if (_value is null)
        {
            message = $"Parameter `{ComponentName}` is not configured.";
            return false;
        }

        switch (ValueKind)
        {
            case UnitOperationParameterValueKind.StructuredJsonText:
                try
                {
                    using var _ = JsonDocument.Parse(_value);
                }
                catch (JsonException)
                {
                    message = $"Parameter `{ComponentName}` must contain valid JSON text.";
                    return false;
                }

                break;
            case UnitOperationParameterValueKind.Identifier:
                if (!IsTrimmed(_value))
                {
                    message = $"Identifier parameter `{ComponentName}` must not contain leading or trailing whitespace.";
                    return false;
                }

                if (ContainsControlCharacters(_value))
                {
                    message = $"Identifier parameter `{ComponentName}` must not contain control characters.";
                    return false;
                }

                break;
            case UnitOperationParameterValueKind.FilePath:
                if (!IsTrimmed(_value))
                {
                    message = $"File path parameter `{ComponentName}` must not contain leading or trailing whitespace.";
                    return false;
                }

                if (ContainsControlCharacters(_value))
                {
                    message = $"File path parameter `{ComponentName}` must not contain control characters.";
                    return false;
                }

                break;
            default:
                throw new ArgumentOutOfRangeException(nameof(ValueKind), ValueKind, "Unsupported parameter value kind.");
        }

        message = $"Parameter `{ComponentName}` is configured.";
        return true;
    }

    private static string DescribeValueKind(UnitOperationParameterValueKind valueKind)
    {
        return valueKind switch
        {
            UnitOperationParameterValueKind.StructuredJsonText => "structured JSON text",
            UnitOperationParameterValueKind.Identifier => "identifier text",
            UnitOperationParameterValueKind.FilePath => "file path text",
            _ => throw new ArgumentOutOfRangeException(nameof(valueKind), valueKind, "Unsupported parameter value kind."),
        };
    }

    private static string? Normalize(string? value, bool allowsEmptyValue)
    {
        if (value is null)
        {
            return null;
        }

        return !allowsEmptyValue && string.IsNullOrWhiteSpace(value) ? null : value;
    }

    private static bool IsTrimmed(string value)
    {
        return string.Equals(value, value.Trim(), StringComparison.Ordinal);
    }

    private static bool ContainsControlCharacters(string value)
    {
        return value.Any(char.IsControl);
    }

    private void SetImmutableComponentDescription(string value, string operation)
    {
        EnsureOwnerAccess(operation, parameter: value);
        ArgumentNullException.ThrowIfNull(value);

        if (string.Equals(_definition.Description, value, StringComparison.Ordinal))
        {
            return;
        }

        throw new CapeInvalidArgumentException(
            $"Parameter `{_definition.Name}` does not allow ComponentDescription mutation in the MVP runtime.",
            CreateContext(operation, value));
    }

    private void SetImmutableComponentName(string value, string operation)
    {
        EnsureOwnerAccess(operation, parameter: value);
        ArgumentException.ThrowIfNullOrWhiteSpace(value);

        if (string.Equals(_definition.Name, value, StringComparison.Ordinal))
        {
            return;
        }

        throw new CapeInvalidArgumentException(
            $"Parameter `{_definition.Name}` does not allow ComponentName mutation in the MVP runtime.",
            CreateContext(operation, value));
    }

    private CapeOpenExceptionContext CreateContext(string operation, object? parameter)
    {
        return new CapeOpenExceptionContext(
            InterfaceName: InterfaceName,
            Scope: "RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders",
            Operation: operation,
            ParameterName: _definition.Name,
            Parameter: parameter);
    }

    private void EnsureOwnerAccess(string operation, object? parameter = null)
    {
        EnsureOwnerAccess(InterfaceName, operation, parameter);
    }

    private void EnsureOwnerAccess(string interfaceName, string operation, object? parameter = null)
    {
        _ensureOwnerAccess?.Invoke(interfaceName, operation, _definition.Name, parameter);
    }

    private CapeValidationStatus _valStatus;
}
