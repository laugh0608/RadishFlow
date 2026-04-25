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
public sealed class UnitOperationParameterPlaceholder : ICapeIdentification, ICapeParameter
{
    private const string InterfaceName = nameof(ICapeParameter);
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
            EnsureOwnerAccess(nameof(ComponentName));
            return _definition.Name;
        }
        set => SetImmutableComponentName(value, nameof(ComponentName));
    }

    public string ComponentDescription
    {
        get
        {
            EnsureOwnerAccess(nameof(ComponentDescription));
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
            return _definition.Mode;
        }
        set
        {
            EnsureOwnerAccess(nameof(Mode), value);

            if (_definition.Mode == value)
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
        EnsureOwnerAccess(operation, value);
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
        EnsureOwnerAccess(operation, value);
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
        _ensureOwnerAccess?.Invoke(InterfaceName, operation, _definition.Name, parameter);
    }

    private CapeValidationStatus _valStatus;
}
