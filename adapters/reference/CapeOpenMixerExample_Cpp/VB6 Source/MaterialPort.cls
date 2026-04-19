VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
  Persistable = 0  'NotPersistable
  DataBindingBehavior = 0  'vbNone
  DataSourceBehavior  = 0  'vbNone
  MTSTransactionMode  = 0  'NotAnMTSObject
END
Attribute VB_Name = "MaterialPort"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = True
Option Explicit

'this is a an implementation of a CAPE-OPEN material port, suitable for both feeds and products

Implements ICapeIdentification 'required for all CAPE-OPEN components
Implements ECapeRoot           'required in case of components that throw errors
Implements ECapeUnknown        'required in case of components that throw errors
Implements ECapeUser           'required in case of components that throw errors (all other error interfaces derive from this)
Implements ICapeUnitPort       'main interface for a port

'name and description; we take them to be the same for this object; read-only
Public name As String
'error variables (we have a simple error handling mechanism, providing only name(=desc) and scope and interface
Dim errDesc As String, errIface As String, errScope As String
'port variables:
Public direction As CapePortDirection 'set at creation
'the connected object
Private connectedObject As Object

'some methods for internal use

Public Property Get Connected() As Boolean
 Connected = Not (connectedObject Is Nothing)
End Property

'in order to deal with version 1.0 and version 1.1 thermo transparently, we operate on an internal
' material object description

Friend Function GetMaterial() As MaterialObject
 'caller should verify that the port is connected
 If Not Connected() Then
  Call SetError("Internal error, request for material from unconnected port", "N/A", "GetMaterial")
  Err.Raise ECapeUnknownHR
 End If
 Dim MO11 As New MaterialObject11
 If Not MO11.MaterialObject_SetMO(connectedObject) Then
  'not version 1.1, try version 1.0
  Dim MO10 As New MaterialObject10
  If Not MO10.MaterialObject_SetMO(connectedObject) Then
   'not version 1.0 either
   Call SetError("Internal error, port is connected to an unsupported material object", "N/A", "GetMaterial")
   Err.Raise ECapeUnknownHR
  Else
   Set GetMaterial = MO10
  End If
 Else
  Set GetMaterial = MO11
 End If
End Function

'initialize

Private Sub Class_Initialize()
 Set connectedObject = Nothing 'unconnected
End Sub

'function to store error

Private Sub SetError(desc As String, iface As String, scope As String)
 'call this function before returning a CAPE-OPEN error
 ' these strings are then available via the CAPE-OPEN error interfaces
 errDesc = desc
 errIface = iface
 errScope = scope
End Sub

'ICapeIdentification methods

Private Property Let ICapeIdentification_ComponentDescription(ByVal RHS As String)
 'set description is not allowed
 Call SetError("The description of this object cannot be changed", "ICapeIdentification", "setComponentDescription")
 Err.Raise ECapeUnknownHR
End Property

Private Property Get ICapeIdentification_ComponentDescription() As String
 'get description, same as name
 ICapeIdentification_ComponentDescription = name
End Property

Private Property Let ICapeIdentification_ComponentName(ByVal RHS As String)
 'set name is not allowed
 Call SetError("The name of this object cannot be changed", "ICapeIdentification", "setComponentDescription")
 Err.Raise ECapeUnknownHR
End Property

Private Property Get ICapeIdentification_ComponentName() As String
 'get name
 ICapeIdentification_ComponentName = name
End Property

'error interfaces

Private Property Get ECapeRoot_name() As String
 'we consider name and description of an error the same in this example
 ECapeRoot_name = errDesc
End Property

Private Property Get ECapeUser_code() As Long
 'we do not support an error code
 ECapeUser_code = 0
End Property

Private Property Get ECapeUser_description() As String
 'return error description
 ECapeUser_description = errDesc
End Property

Private Property Get ECapeUser_interfaceName() As String
 'return error interface
 ECapeUser_interfaceName = errIface
End Property

Private Property Get ECapeUser_moreInfo() As String
 'return more info (not supported)
 ECapeUser_moreInfo = ""
End Property

Private Property Get ECapeUser_operation() As String
 'return operation (not supported)
 ECapeUser_operation = "unknown"
End Property

Private Property Get ECapeUser_scope() As String
 'return error scope
 ECapeUser_scope = errScope
End Property

'ICapeUnitPort methods

Private Sub ICapeUnitPort_Connect(ByVal objectToConnect As Object)
 'check if this is an MO we can deal with
 Dim MO11 As New MaterialObject11
 Dim MO10 As New MaterialObject10
 If Not MO11.MaterialObject_SetMO(objectToConnect) Then
  If Not MO10.MaterialObject_SetMO(objectToConnect) Then
   'we cannot use this object, whatever it implements, we do not support
   Call SetError("The object does not expose Material Object interfaces supported by this unit operation", "ICapeUnitPort", "Connect")
   Err.Raise ECapeUnknownHR
  End If
 End If
 'object is ok
 Set connectedObject = objectToConnect
End Sub

Private Sub ICapeUnitPort_Disconnect()
 Set connectedObject = Nothing 'we are now disconnected
End Sub

Private Property Get ICapeUnitPort_connectedObject() As Object
 'return the connected object
 Set ICapeUnitPort_connectedObject = connectedObject
End Property

Private Property Get ICapeUnitPort_direction() As CapePortDirection
 'return my direction
 ICapeUnitPort_direction = direction
End Property

Private Property Get ICapeUnitPort_portType() As CapePortType
 'return my type; I am a material port
 ICapeUnitPort_portType = CAPE_MATERIAL
End Property
