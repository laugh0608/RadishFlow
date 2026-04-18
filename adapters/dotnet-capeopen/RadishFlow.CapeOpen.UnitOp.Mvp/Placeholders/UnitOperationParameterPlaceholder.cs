using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Unit;
using System.Text.Json;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

public sealed class UnitOperationParameterPlaceholder : ICapeIdentification, ICapeParameter
{
    private const string InterfaceName = nameof(ICapeParameter);
    private readonly Action<string, string, string?, object?>? _ensureOwnerAccess;
    private readonly Action? _onStateChanged;
    private readonly string? _defaultValue;
    private readonly UnitOperationParameterValueKind _valueKind;
    private readonly string? _requiredCompanionParameterName;
    private readonly string _initialComponentName;
    private readonly string _initialComponentDescription;
    private readonly UnitOperationParameterSpecificationPlaceholder _specification;
    private CapeParamMode _mode;
    private string? _value;
    private string _componentName;
    private string _componentDescription;

    public UnitOperationParameterPlaceholder(
        string componentName,
        string componentDescription,
        bool isRequired,
        UnitOperationParameterValueKind valueKind,
        bool allowsEmptyValue = false,
        string? requiredCompanionParameterName = null,
        CapeParamMode mode = CapeParamMode.CAPE_INPUT,
        string? defaultValue = null,
        Action<string, string, string?, object?>? ensureOwnerAccess = null,
        Action? onStateChanged = null)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(componentName);
        ArgumentNullException.ThrowIfNull(componentDescription);

        _componentName = componentName;
        _componentDescription = componentDescription;
        _initialComponentName = componentName;
        _initialComponentDescription = componentDescription;
        IsRequired = isRequired;
        AllowsEmptyValue = allowsEmptyValue;
        _valueKind = valueKind;
        _requiredCompanionParameterName = Normalize(requiredCompanionParameterName, allowsEmptyValue: false);
        _mode = mode;
        _defaultValue = Normalize(defaultValue, allowsEmptyValue);
        _value = _defaultValue;
        _ensureOwnerAccess = ensureOwnerAccess;
        _onStateChanged = onStateChanged;
        _specification = new UnitOperationParameterSpecificationPlaceholder(
            parameterName: componentName,
            type: CapeParamType.CAPE_OPTION,
            dimensionality: [],
            ensureOwnerAccess: ensureOwnerAccess);
        ValStatus = CapeValidationStatus.NotValidated;
    }

    public string ComponentName
    {
        get
        {
            EnsureOwnerAccess(nameof(ComponentName));
            return _componentName;
        }
        set => _componentName = SetImmutableComponentName(value, nameof(ComponentName));
    }

    public string ComponentDescription
    {
        get
        {
            EnsureOwnerAccess(nameof(ComponentDescription));
            return _componentDescription;
        }
        set => _componentDescription = SetImmutableComponentDescription(value, nameof(ComponentDescription));
    }

    public bool IsRequired { get; }

    public UnitOperationParameterValueKind ValueKind
    {
        get
        {
            EnsureOwnerAccess(nameof(ValueKind));
            return _valueKind;
        }
    }

    public bool AllowsEmptyValue
    {
        get
        {
            EnsureOwnerAccess(nameof(AllowsEmptyValue));
            return _allowsEmptyValue;
        }
        private init => _allowsEmptyValue = value;
    }

    public string? RequiredCompanionParameterName
    {
        get
        {
            EnsureOwnerAccess(nameof(RequiredCompanionParameterName));
            return _requiredCompanionParameterName;
        }
    }

    public string? DefaultValue
    {
        get
        {
            EnsureOwnerAccess(nameof(DefaultValue));
            return _defaultValue;
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
            return _specification;
        }
    }

    public object? value
    {
        get
        {
            EnsureOwnerAccess(nameof(value));
            return _value;
        }
        set
        {
            EnsureOwnerAccess(nameof(value), value);

            if (value is not null and not string)
            {
                throw new CapeInvalidArgumentException(
                    $"Parameter `{ComponentName}` only accepts string or null values in the MVP runtime.",
                    CreateContext(nameof(value), value));
            }

            SetValueCore((string?)value);
        }
    }

    public CapeValidationStatus ValStatus
    {
        get
        {
            EnsureOwnerAccess(nameof(ValStatus));
            return _valStatus;
        }
        private set => _valStatus = value;
    }

    public CapeParamMode Mode
    {
        get
        {
            EnsureOwnerAccess(nameof(Mode));
            return _mode;
        }
        set
        {
            EnsureOwnerAccess(nameof(Mode), value);

            if (_mode == value)
            {
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
            EnsureOwnerAccess(nameof(Type));
            return _specification.Type;
        }
    }

    public double[] Dimensionality
    {
        get
        {
            EnsureOwnerAccess(nameof(Dimensionality));
            return _specification.Dimensionality;
        }
    }

    public void SetValue(string? value)
    {
        EnsureOwnerAccess(nameof(SetValue), value);
        SetValueCore(value);
    }

    public bool Validate(ref string message)
    {
        EnsureOwnerAccess(nameof(Validate));

        if (IsRequired && !IsConfigured)
        {
            message = $"Required parameter `{ComponentName}` is not configured.";
            ValStatus = CapeValidationStatus.Invalid;
            return false;
        }

        if (IsConfigured && !TryValidateConfiguredValue(out var validationError))
        {
            message = validationError;
            ValStatus = CapeValidationStatus.Invalid;
            return false;
        }

        message = IsConfigured
            ? $"Parameter `{ComponentName}` is configured as {DescribeValueKind(ValueKind)}."
            : $"Optional parameter `{ComponentName}` is not configured.";
        ValStatus = CapeValidationStatus.Valid;
        return true;
    }

    public void Reset()
    {
        EnsureOwnerAccess(nameof(Reset));
        SetValueCore(_defaultValue, forceValidationReset: true);
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

    private string SetImmutableComponentName(string value, string operation)
    {
        EnsureOwnerAccess(operation, value);
        ArgumentException.ThrowIfNullOrWhiteSpace(value);

        if (string.Equals(_initialComponentName, value, StringComparison.Ordinal))
        {
            return _initialComponentName;
        }

        throw new CapeInvalidArgumentException(
            $"Parameter `{_initialComponentName}` does not allow ComponentName mutation in the MVP runtime.",
            CreateContext(operation, value));
    }

    private string SetImmutableComponentDescription(string value, string operation)
    {
        EnsureOwnerAccess(operation, value);
        ArgumentNullException.ThrowIfNull(value);

        if (string.Equals(_initialComponentDescription, value, StringComparison.Ordinal))
        {
            return _initialComponentDescription;
        }

        throw new CapeInvalidArgumentException(
            $"Parameter `{_initialComponentName}` does not allow ComponentDescription mutation in the MVP runtime.",
            CreateContext(operation, value));
    }

    private CapeOpenExceptionContext CreateContext(string operation, object? parameter)
    {
        return new CapeOpenExceptionContext(
            InterfaceName: InterfaceName,
            Scope: "RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders",
            Operation: operation,
            ParameterName: _componentName,
            Parameter: parameter);
    }

    private void EnsureOwnerAccess(string operation, object? parameter = null)
    {
        _ensureOwnerAccess?.Invoke(InterfaceName, operation, _componentName, parameter);
    }

    private CapeValidationStatus _valStatus;
    private bool _allowsEmptyValue;
}
