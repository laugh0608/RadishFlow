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
    private readonly string _initialComponentName;
    private readonly string _initialComponentDescription;
    private object? _connectedObject;
    private string _componentName;
    private string _componentDescription;

    public UnitOperationPortPlaceholder(
        string componentName,
        string componentDescription,
        CapePortDirection direction,
        CapePortType portType,
        bool isRequired,
        Action<string, string, string?, object?>? ensureOwnerAccess = null,
        Action? onStateChanged = null)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(componentName);
        ArgumentNullException.ThrowIfNull(componentDescription);

        _componentName = componentName;
        _componentDescription = componentDescription;
        _initialComponentName = componentName;
        _initialComponentDescription = componentDescription;
        _direction = direction;
        _portType = portType;
        IsRequired = isRequired;
        _ensureOwnerAccess = ensureOwnerAccess;
        _onStateChanged = onStateChanged;
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
        if (_connectedObject is not null)
        {
            if (ReferenceEquals(_connectedObject, connectedIdentification))
            {
                return;
            }

            throw new CapeBadInvocationOrderException(
                $"Port `{ComponentName}` is already connected. Disconnect it before replacing the connected object.",
                CreateContext(nameof(Connect), objectToConnect, requestedOperation: nameof(Disconnect)));
        }

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
        if (_connectedObject is not null)
        {
            return;
        }

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

    private string SetImmutableComponentName(string value, string operation)
    {
        EnsureOwnerAccess(operation, value);
        ArgumentException.ThrowIfNullOrWhiteSpace(value);

        if (string.Equals(_initialComponentName, value, StringComparison.Ordinal))
        {
            return _initialComponentName;
        }

        throw new CapeInvalidArgumentException(
            $"Port `{_initialComponentName}` does not allow ComponentName mutation in the MVP runtime.",
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
            $"Port `{_initialComponentName}` does not allow ComponentDescription mutation in the MVP runtime.",
            CreateContext(operation, value));
    }

    private CapeOpenExceptionContext CreateContext(
        string operation,
        object? parameter = null,
        string? requestedOperation = null)
    {
        return new CapeOpenExceptionContext(
            InterfaceName: InterfaceName,
            Scope: "RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders",
            Operation: operation,
            ParameterName: _componentName,
            Parameter: parameter,
            RequestedOperation: requestedOperation);
    }

    private void EnsureOwnerAccess(string operation, object? parameter = null)
    {
        _ensureOwnerAccess?.Invoke(InterfaceName, operation, _componentName, parameter);
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
