using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Unit;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

public sealed class UnitOperationPortPlaceholder : ICapeIdentification, ICapeUnitPort
{
    private readonly Action? _onStateChanged;
    private object? _connectedObject;

    public UnitOperationPortPlaceholder(
        string componentName,
        string componentDescription,
        CapePortDirection direction,
        CapePortType portType,
        bool isRequired,
        Action? onStateChanged = null)
    {
        ComponentName = componentName;
        ComponentDescription = componentDescription;
        this.direction = direction;
        this.portType = portType;
        IsRequired = isRequired;
        _onStateChanged = onStateChanged;
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }

    public CapePortDirection direction { get; }

    public CapePortType portType { get; }

    public bool IsRequired { get; }

    public object? connectedObject => _connectedObject;

    public bool IsConnected => _connectedObject is not null;

    public void Connect(object objectToConnect)
    {
        if (objectToConnect is null)
        {
            throw new CapeInvalidArgumentException(
                $"Port `{ComponentName}` cannot connect to a null object.",
                CreateContext(nameof(Connect)));
        }

        _connectedObject = objectToConnect;
        _onStateChanged?.Invoke();
    }

    public void Disconnect()
    {
        if (_connectedObject is null)
        {
            return;
        }

        _connectedObject = null;
        _onStateChanged?.Invoke();
    }

    internal void ConnectPlaceholder()
    {
        Connect(new UnitOperationConnectedObjectPlaceholder(
            $"{ComponentName} Connection",
            $"Placeholder connection object for port `{ComponentName}`."));
    }

    private CapeOpenExceptionContext CreateContext(string operation)
    {
        return new CapeOpenExceptionContext(
            InterfaceName: nameof(ICapeUnitPort),
            Scope: "RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders",
            Operation: operation,
            ParameterName: ComponentName);
    }
}

internal sealed class UnitOperationConnectedObjectPlaceholder : ICapeIdentification
{
    public UnitOperationConnectedObjectPlaceholder(string componentName, string componentDescription)
    {
        ComponentName = componentName;
        ComponentDescription = componentDescription;
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }
}
