using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

public sealed class UnitOperationPortPlaceholder : ICapeIdentification, ICapeUnitPort
{
    private const string InterfaceName = nameof(ICapeUnitPort);
    private readonly Action<string, string, string?, object?>? _ensureOwnerAccess;
    private readonly Action? _onStateChanged;
    private readonly UnitOperationPortDefinition _definition;
    private object? _connectedObject;

    public UnitOperationPortPlaceholder(
        UnitOperationPortDefinition definition,
        Action<string, string, string?, object?>? ensureOwnerAccess = null,
        Action? onStateChanged = null)
    {
        ArgumentNullException.ThrowIfNull(definition);

        _definition = definition;
        _ensureOwnerAccess = ensureOwnerAccess;
        _onStateChanged = onStateChanged;
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

    public CapePortDirection direction
    {
        get
        {
            EnsureOwnerAccess(nameof(direction));
            return _definition.Direction;
        }
    }

    public CapePortType portType
    {
        get
        {
            EnsureOwnerAccess(nameof(portType));
            return _definition.PortType;
        }
    }

    public bool IsRequired => _definition.IsRequired;

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

    private void SetImmutableComponentName(string value, string operation)
    {
        EnsureOwnerAccess(operation, value);
        ArgumentException.ThrowIfNullOrWhiteSpace(value);

        if (string.Equals(_definition.Name, value, StringComparison.Ordinal))
        {
            return;
        }

        throw new CapeInvalidArgumentException(
            $"Port `{_definition.Name}` does not allow ComponentName mutation in the MVP runtime.",
            CreateContext(operation, value));
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
            $"Port `{_definition.Name}` does not allow ComponentDescription mutation in the MVP runtime.",
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
            ParameterName: _definition.Name,
            Parameter: parameter,
            RequestedOperation: requestedOperation);
    }

    private void EnsureOwnerAccess(string operation, object? parameter = null)
    {
        _ensureOwnerAccess?.Invoke(InterfaceName, operation, _definition.Name, parameter);
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
