using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Unit;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

public sealed class UnitOperationPortPlaceholder : ICapeIdentification, ICapeUnitPort
{
    private const string InterfaceName = nameof(ICapeUnitPort);
    private readonly Action<string, string, string?, object?>? _ensureOwnerAccess;
    private readonly Action? _onStateChanged;
    private readonly CapePortDirection _direction;
    private readonly CapePortType _portType;
    private object? _connectedObject;

    public UnitOperationPortPlaceholder(
        string componentName,
        string componentDescription,
        CapePortDirection direction,
        CapePortType portType,
        bool isRequired,
        Action<string, string, string?, object?>? ensureOwnerAccess = null,
        Action? onStateChanged = null)
    {
        ComponentName = componentName;
        ComponentDescription = componentDescription;
        _direction = direction;
        _portType = portType;
        IsRequired = isRequired;
        _ensureOwnerAccess = ensureOwnerAccess;
        _onStateChanged = onStateChanged;
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }

    public CapePortDirection direction
    {
        get
        {
            EnsureOwnerAccess(nameof(direction));
            return _direction;
        }
    }

    public CapePortType portType
    {
        get
        {
            EnsureOwnerAccess(nameof(portType));
            return _portType;
        }
    }

    public bool IsRequired { get; }

    public object? connectedObject
    {
        get
        {
            EnsureOwnerAccess(nameof(connectedObject));
            return _connectedObject;
        }
    }

    public bool IsConnected => _connectedObject is not null;

    public void Connect(object objectToConnect)
    {
        EnsureOwnerAccess(nameof(Connect), objectToConnect);

        if (objectToConnect is null)
        {
            throw new CapeInvalidArgumentException(
                $"Port `{ComponentName}` cannot connect to a null object.",
                CreateContext(nameof(Connect), objectToConnect));
        }

        var connectedIdentification = ValidateConnectedObject(objectToConnect);
        _connectedObject = connectedIdentification;
        _onStateChanged?.Invoke();
    }

    public void Disconnect()
    {
        EnsureOwnerAccess(nameof(Disconnect));

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

    internal void ReleaseConnectedObject()
    {
        _connectedObject = null;
    }

    private ICapeIdentification ValidateConnectedObject(object objectToConnect)
    {
        if (objectToConnect is not ICapeIdentification identifiedObject)
        {
            throw new CapeInvalidArgumentException(
                $"Port `{ComponentName}` only accepts connected objects that implement ICapeIdentification in the MVP runtime.",
                CreateContext(nameof(Connect), objectToConnect));
        }

        if (string.IsNullOrWhiteSpace(identifiedObject.ComponentName))
        {
            throw new CapeInvalidArgumentException(
                $"Port `{ComponentName}` requires connected objects to expose a non-empty ComponentName.",
                CreateContext(nameof(Connect), objectToConnect));
        }

        return identifiedObject;
    }

    private CapeOpenExceptionContext CreateContext(string operation, object? parameter = null)
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
