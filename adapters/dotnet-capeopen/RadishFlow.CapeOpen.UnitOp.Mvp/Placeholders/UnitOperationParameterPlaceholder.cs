using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Parameters;
using RadishFlow.CapeOpen.Interop.Unit;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

public sealed class UnitOperationParameterPlaceholder : ICapeIdentification, ICapeParameter, ICapeParameterSpec
{
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
        Action? onStateChanged = null)
    {
        ComponentName = componentName;
        ComponentDescription = componentDescription;
        IsRequired = isRequired;
        _mode = mode;
        _defaultValue = Normalize(defaultValue);
        _value = _defaultValue;
        _onStateChanged = onStateChanged;
        ValStatus = CapeValidationStatus.NotValidated;
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }

    public bool IsRequired { get; }

    public string? Value => _value;

    public bool IsConfigured => !string.IsNullOrWhiteSpace(_value);

    public object Specification => this;

    public object? value
    {
        get => _value;
        set
        {
            if (value is not null and not string)
            {
                throw new CapeInvalidArgumentException(
                    $"Parameter `{ComponentName}` only accepts string or null values in the MVP runtime.",
                    CreateContext("value", value));
            }

            SetValueCore((string?)value);
        }
    }

    public CapeValidationStatus ValStatus { get; private set; }

    public CapeParamMode Mode
    {
        get => _mode;
        set
        {
            if (_mode == value)
            {
                return;
            }

            _mode = value;
            MarkChanged();
        }
    }

    public CapeParamType Type => CapeParamType.CAPE_OPTION;

    public double[] Dimensionality => Array.Empty<double>();

    public void SetValue(string? value)
    {
        SetValueCore(value);
    }

    public bool Validate(ref string message)
    {
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
            InterfaceName: nameof(ICapeParameter),
            Scope: "RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders",
            Operation: operation,
            ParameterName: ComponentName,
            Parameter: parameter);
    }
}
