using RadishFlow.CapeOpen.Interop.Common;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

public sealed class UnitOperationPortPlaceholder : ICapeIdentification
{
    public UnitOperationPortPlaceholder(
        string componentName,
        string componentDescription,
        string direction,
        string kind,
        bool isRequired)
    {
        ComponentName = componentName;
        ComponentDescription = componentDescription;
        Direction = direction;
        Kind = kind;
        IsRequired = isRequired;
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }

    public string Direction { get; }

    public string Kind { get; }

    public bool IsRequired { get; }

    public bool IsConnected { get; private set; }

    public void SetConnected(bool isConnected)
    {
        IsConnected = isConnected;
    }
}
