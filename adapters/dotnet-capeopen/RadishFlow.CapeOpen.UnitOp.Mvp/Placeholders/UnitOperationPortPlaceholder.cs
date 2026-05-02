using RadishFlow.CapeOpen.Interop.Common;
using RadishFlow.CapeOpen.Interop.Errors;
using RadishFlow.CapeOpen.Interop.Unit;
using RadishFlow.CapeOpen.UnitOp.Mvp.UnitOperation;
using System.Runtime.InteropServices;

namespace RadishFlow.CapeOpen.UnitOp.Mvp.Placeholders;

[ComVisible(true)]
[Guid(PlaceholderComClassIds.PortPlaceholder)]
[ClassInterface(ClassInterfaceType.None)]
[ComDefaultInterface(typeof(ICapeUnitPort))]
public sealed class UnitOperationPortPlaceholder : ICapeIdentification, ICapeUnitPort
{
    private const string InterfaceName = nameof(ICapeUnitPort);
    private readonly Action<string, string, string?, object?>? _ensureOwnerAccess;
    private readonly Action? _onStateChanged;
    private readonly UnitOperationPortDefinition _definition;
    private UnitOperationConnectedObjectPlaceholder? _connectedObject;
    private object? _connectedObjectReference;

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
            UnitOperationComTrace.Write($"{ComponentName}.{InterfaceName}.{nameof(direction)}", "get-exit", _definition.Direction.ToString());
            return _definition.Direction;
        }
    }

    public CapePortType portType
    {
        get
        {
            EnsureOwnerAccess(nameof(portType));
            UnitOperationComTrace.Write($"{ComponentName}.{InterfaceName}.{nameof(portType)}", "get-exit", _definition.PortType.ToString());
            return _definition.PortType;
        }
    }

    public bool IsRequired => _definition.IsRequired;

    public object? connectedObject
    {
        get
        {
            EnsureOwnerAccess(nameof(connectedObject));
            UnitOperationComTrace.Write(
                $"{ComponentName}.{InterfaceName}.{nameof(connectedObject)}",
                "get-exit",
                _connectedObject is null
                    ? "null"
                    : $"{_connectedObject.ComponentName}; liveReference={_connectedObjectReference is not null}");
            return _connectedObjectReference ?? _connectedObject;
        }
    }

    public bool IsConnected => _connectedObject is not null;

    internal object? ConnectedObjectReference => _connectedObjectReference ?? _connectedObject;

    public void Connect(object objectToConnect)
    {
        UnitOperationComTrace.Write($"{nameof(UnitOperationPortPlaceholder)}.{nameof(Connect)}", "enter", ComponentName);
        EnsureOwnerAccess(nameof(Connect), objectToConnect);

        if (objectToConnect is null)
        {
            throw new CapeInvalidArgumentException(
                $"Port `{ComponentName}` cannot connect to a null object.",
                CreateContext(nameof(Connect), objectToConnect));
        }

        var connectedIdentification = ValidateConnectedObject(objectToConnect);
        var connectedSnapshot = new UnitOperationConnectedObjectPlaceholder(
            connectedIdentification.ComponentName,
            connectedIdentification.ComponentDescription);
        if (_connectedObject is not null)
        {
            if (string.Equals(_connectedObject.ComponentName, connectedSnapshot.ComponentName, StringComparison.Ordinal))
            {
                UnitOperationComTrace.Write(
                    $"{nameof(UnitOperationPortPlaceholder)}.{nameof(Connect)}",
                    "already-connected",
                    ComponentName);
                return;
            }

            throw new CapeBadInvocationOrderException(
                $"Port `{ComponentName}` is already connected. Disconnect it before replacing the connected object.",
                CreateContext(nameof(Connect), objectToConnect, requestedOperation: nameof(Disconnect)));
        }

        _connectedObject = connectedSnapshot;
        _connectedObjectReference = objectToConnect;
        _onStateChanged?.Invoke();
        UnitOperationComTrace.Write(
            $"{nameof(UnitOperationPortPlaceholder)}.{nameof(Connect)}",
            "exit",
            $"{ComponentName}->{connectedSnapshot.ComponentName}");
    }

    public void Disconnect()
    {
        UnitOperationComTrace.Write($"{nameof(UnitOperationPortPlaceholder)}.{nameof(Disconnect)}", "enter", ComponentName);
        EnsureOwnerAccess(nameof(Disconnect));

        if (_connectedObject is null)
        {
            UnitOperationComTrace.Write(
                $"{nameof(UnitOperationPortPlaceholder)}.{nameof(Disconnect)}",
                "already-disconnected",
                ComponentName);
            return;
        }

        ReleaseConnectedObjectReference(nameof(Disconnect));
        _connectedObject = null;
        _onStateChanged?.Invoke();
        UnitOperationComTrace.Write($"{nameof(UnitOperationPortPlaceholder)}.{nameof(Disconnect)}", "exit", ComponentName);
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
        UnitOperationComTrace.Write(
            $"{nameof(UnitOperationPortPlaceholder)}.{nameof(ReleaseConnectedObject)}",
            "enter",
            ComponentName);
        ReleaseConnectedObjectReference(nameof(ReleaseConnectedObject));
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

    private void ReleaseConnectedObjectReference(string operation)
    {
        if (_connectedObjectReference is null)
        {
            return;
        }

        var objectToRelease = _connectedObjectReference;
        _connectedObjectReference = null;
        ReleaseComReference(objectToRelease, operation);
    }

    private void ReleaseComReference(object objectToRelease, string operation)
    {
        try
        {
            if (!Marshal.IsComObject(objectToRelease))
            {
                UnitOperationComTrace.Write(
                    $"{nameof(UnitOperationPortPlaceholder)}.{operation}",
                    "cleared-connected-reference",
                    ComponentName);
                return;
            }

#pragma warning disable CA1416 // UnitOp.Mvp COM activation is Windows-only.
            var remaining = Marshal.ReleaseComObject(objectToRelease);
#pragma warning restore CA1416
            UnitOperationComTrace.Write(
                $"{nameof(UnitOperationPortPlaceholder)}.{operation}",
                "released-connected-reference",
                $"{ComponentName}; remaining={remaining}");
        }
        catch (Exception error)
        {
            UnitOperationComTrace.Exception($"{nameof(UnitOperationPortPlaceholder)}.{operation}.ReleaseComReference", error);
        }
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

[ComVisible(true)]
[Guid(PlaceholderComClassIds.ConnectedObjectPlaceholder)]
[ClassInterface(ClassInterfaceType.None)]
[ComDefaultInterface(typeof(ICapeIdentification))]
public sealed class UnitOperationConnectedObjectPlaceholder : ICapeIdentification
{
    public UnitOperationConnectedObjectPlaceholder(string componentName, string componentDescription)
    {
        ComponentName = componentName;
        ComponentDescription = componentDescription;
    }

    public string ComponentName { get; set; }

    public string ComponentDescription { get; set; }
}
