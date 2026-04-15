using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Unit;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

public sealed class UnitOperationParameterPlaceholder : ICapeIdentification, ICapeParameter, ICapeParameterSpec
{
    private const string InterfaceName = nameof(ICapeParameter);
    private readonly Action<string, string, string?, object?>? _ensureOwnerAccess;
    private readonly Action? _onStateChanged;
    private readonly string? _defaultValue;
    private CapeParamMode _mode;
    private string? _value;

    public UnitOperationParameterPlaceholder(
        string componentName,
        string componentDescription,
        bool isRequired,
        CapeParamMode mode = CapeParamMode.CAPE_INPUT,
        string? defaultValue = null,
        Action<string, string, string?, object?>? ensureOwnerAccess = null,
        Action? onStateChanged = null)
    {
        ComponentName = componentName;
        ComponentDescription = componentDescription;
        IsRequired = isRequired;
        _mode = mode;
        _defaultValue = Normalize(defaultValue);
        _value = _defaultValue;
        _ensureOwnerAccess = ensureOwnerAccess;
        _onStateChanged = onStateChanged;
        ValStatus = CapeValidationStatus.NotValidated;
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }

    public bool IsRequired { get; }

    public string? Value => _value;

    public bool IsConfigured => !string.IsNullOrWhiteSpace(_value);

    public object Specification
    {
        get
        {
            EnsureOwnerAccess(nameof(Specification));
            return this;
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

            _mode = value;
            MarkChanged();
        }
    }

    public CapeParamType Type
    {
        get
        {
            EnsureOwnerAccess(nameof(Type));
            return CapeParamType.CAPE_OPTION;
        }
    }

    public double[] Dimensionality
    {
        get
        {
            EnsureOwnerAccess(nameof(Dimensionality));
            return Array.Empty<double>();
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

        message = IsConfigured
            ? $"Parameter `{ComponentName}` is configured."
            : $"Optional parameter `{ComponentName}` is not configured.";
        ValStatus = CapeValidationStatus.Valid;
        return true;
    }

    public void Reset()
    {
        EnsureOwnerAccess(nameof(Reset));
        SetValueCore(_defaultValue);
    }

    private void SetValueCore(string? value)
    {
        var normalized = Normalize(value);
        if (string.Equals(_value, normalized, StringComparison.Ordinal))
        {
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

    private static string? Normalize(string? value)
    {
        return string.IsNullOrWhiteSpace(value) ? null : value;
    }

    private CapeOpenExceptionContext CreateContext(string operation, object? parameter)
    {
        return new CapeOpenExceptionContext(
            InterfaceName: InterfaceName,
            Scope: "RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders",
            Operation: operation,
            ParameterName: ComponentName,
            Parameter: parameter);
    }

    private void EnsureOwnerAccess(string operation, object? parameter = null)
    {
        _ensureOwnerAccess?.Invoke(InterfaceName, operation, ComponentName, parameter);
    }

    private CapeValidationStatus _valStatus;
}
