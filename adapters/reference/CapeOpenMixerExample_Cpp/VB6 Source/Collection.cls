VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
  Persistable = 0  'NotPersistable
  DataBindingBehavior = 0  'vbNone
  DataSourceBehavior  = 0  'vbNone
  MTSTransactionMode  = 0  'NotAnMTSObject
END
Attribute VB_Name = "Collection"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = True
Option Explicit

'this is a generic CAPE-OPEN collection; this unit operation implements a port and a parameter collection, so the Collection class is used twice

Implements ICapeIdentification 'required for all CAPE-OPEN components
Implements ECapeRoot           'required in case of components that throw errors
Implements ECapeUnknown        'required in case of components that throw errors
Implements ECapeUser           'required in case of components that throw errors (all other error interfaces derive from this)
Implements ICapeCollection     'the main interface of this class

'the objects in the collection we store as ICapeIdentification objects
Dim count As Integer
Dim objects() As ICapeIdentification

'name and description; we take them to be the same for this object; read-only
Public name As String
'error variables (we have a simple error handling mechanism, providing only name(=desc) and scope and interface
Dim errDesc As String, errIface As String, errScope As String

'initialize

Private Sub Class_Initialize()
 count = 0
 ReDim objects(0)
End Sub

'add an object

Public Sub AddItem(item As ICapeIdentification)
 count = count + 1
 ReDim Preserve objects(count)
 Set objects(count) = item
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

'ICapeCollection methods

Public Function ICapeCollection_Count() As Long
 'return the count
 ICapeCollection_Count = count
End Function

Public Function ICapeCollection_Item(ByVal id As Variant) As Object
 'the ID here can be a name, or a 1-based index
 Dim strID As String
 Dim index As Integer
 If (IsNumeric(id)) Then
  index = id
  If (index < 0) Or (index > count) Then
   Call SetError("Index out of range", "ICapeCollection", "Item")
   Err.Raise ECapeUnknownHR
  End If
  Set ICapeCollection_Item = objects(index)
  Exit Function
 End If
 'presume it is a name
 On Error GoTo notAKnownDataType
 strID = id
 On Error GoTo 0
 'check if we have an object by this name
 ' (for larger collections, this is not the most efficient implementation possible)
 For index = 1 To count
  If (objects(index).ComponentName = strID) Then
   Set ICapeCollection_Item = objects(index)
   Exit Function
  End If
 Next index
 'we did not find the object
 Call SetError("No object in the collection with name: " + strID, "ICapeCollection", "Item")
 Err.Raise ECapeUnknownHR
notAKnownDataType:
 'not sure how to identify this object
 On Error GoTo 0
 Call SetError("An item in a collection must be identified by index or name", "ICapeCollection", "Item")
 Err.Raise ECapeUnknownHR
End Function

