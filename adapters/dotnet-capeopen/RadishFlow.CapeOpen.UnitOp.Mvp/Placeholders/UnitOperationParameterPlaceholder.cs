using RadishFlow.CapeOpen.Interop.Common;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

public sealed class UnitOperationParameterPlaceholder : ICapeIdentification
{
    public UnitOperationParameterPlaceholder(string componentName, string componentDescription, bool isRequired)
    {
        ComponentName = componentName;
        ComponentDescription = componentDescription;
        IsRequired = isRequired;
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }

    public bool IsRequired { get; }

    public string? Value { get; private set; }

    public bool IsConfigured => !string.IsNullOrWhiteSpace(Value);

    public void SetValue(string? value)
    {
        Value = string.IsNullOrWhiteSpace(value) ? null : value;
    }
}
