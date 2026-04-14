VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
  Persistable = 0  'NotPersistable
  DataBindingBehavior = 0  'vbNone
  DataSourceBehavior  = 0  'vbNone
  MTSTransactionMode  = 0  'NotAnMTSObject
END
Attribute VB_Name = "DoubleParameter"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = True
Option Explicit

'definition of the parameter types
Public Enum ParameterType
 ParameterSplitFactor
 ParameterHeatInput
End Enum

'implementation of a double parameter class; we have two of these in our unit operation

Implements ICapeIdentification 'required for all CAPE-OPEN components
Implements ECapeRoot           'required in case of components that throw errors
Implements ECapeUnknown        'required in case of components that throw errors
Implements ECapeUser           'required in case of components that throw errors (all other error interfaces derive from this)
Implements ICapeParameter      'the actual parameter
Implements ICapeParameterSpec  'this can be implemented by the same or a different object
Implements ICapeRealParameterSpec 'this is to be implemented by the object that implements the parameter spec

'name and description; we take them to be the same for this object; read-only
Public name As String
'error variables (we have a simple error handling mechanism, providing only name(=desc) and scope and interface
Dim errDesc As String, errIface As String, errScope As String
'type
Public typ As ParameterType 'set value at creation
'reference to the unit (the actual parameter value is not stored in this class, but in the unit itself)
Public unit As VbMixSplitUnitOp 'set value at creation

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

'ICapeParameter methods

Private Property Let ICapeParameter_Mode(ByVal RHS As CapeParamMode)
 'cannot set the mode
 Call SetError("The mode of this parameter cannot be set", "ICapeParameter", "setMode")
 Err.Raise ECapeUnknownHR
End Property

Private Property Get ICapeParameter_Mode() As CapeParamMode
 'this parameter is an input parameter
 ICapeParameter_Mode = CAPE_INPUT
End Property

Private Sub ICapeParameter_Reset()
 'reset the value to the default value
 Select Case typ
  Case ParameterSplitFactor
   unit.splitFactor = 0.5
  Case ParameterHeatInput
   unit.heatInput = 0
  Case Else
   Call SetError("Internal error: unimplemented parameter type", "ICapeParameter", "Reset")
   Err.Raise ECapeUnknownHR
 End Select
End Sub

Private Property Get ICapeParameter_Specification() As Object
 'the parameter specification can be implemented as a separate object; this is not the case
 ' for this implementation; this object implements the parameter spec itself
 Set ICapeParameter_Specification = Me
End Property

Private Function ICapeParameter_Validate(message As String) As Boolean
 'check if the current value is valid; as we insure an invalid value cannot be set,
 ' we can assume that the current value is ok
 ICapeParameter_Validate = True
End Function

Private Property Get ICapeParameter_ValStatus() As CapeValidationStatus
 'return the validation status for the current parameter; as we insure an invalid value cannot be set,
 ' we can assume that the current value is ok
 ICapeParameter_ValStatus = CAPE_VALID
End Property

Private Property Let ICapeParameter_value(ByVal RHS As Variant)
 Dim d As Double
 Dim msg As String
 'first convert the value to a number
 On Error GoTo invalidValueType
 d = RHS
 On Error GoTo 0
 'now check whether the value is in range; this way we know we always have a valid value
 On Error GoTo validateFailed
 If Not ICapeRealParameterSpec_Validate(d, msg) Then
  On Error GoTo 0
  'the value is not valid
  Call SetError("Invalid value: " + msg, "ICapeParameter", "setValue")
  Err.Raise ECapeUnknownHR
 End If
 On Error GoTo 0
 'value is ok, set to unit
 Select Case typ
  Case ParameterSplitFactor
   unit.splitFactor = d
  Case ParameterHeatInput
   unit.heatInput = d
  Case Else
   Call SetError("Internal error: unimplemented parameter type", "ICapeParameter", "setValue")
   Err.Raise ECapeUnknownHR
 End Select
 'the unit is not validated at this point
 unit.valStatus = CAPE_NOT_VALIDATED
 'all done
Exit Property
invalidValueType:
 On Error GoTo 0
 Call SetError("Invalid data type for value; expected a real number", "ICapeParameter", "setValue")
 Err.Raise ECapeUnknownHR
validateFailed:
 On Error GoTo 0
 Call SetError("Failed to validate number: " + errDesc, "ICapeParameter", "setValue")
 Err.Raise ECapeUnknownHR
End Property

Private Property Get ICapeParameter_value() As Variant
 'return the value of the parameter
 Dim v
 Select Case typ
  Case ParameterSplitFactor
   v = unit.splitFactor
  Case ParameterHeatInput
   v = unit.heatInput
  Case Else
   Call SetError("Internal error: unimplemented parameter type", "ICapeParameter", "getValue")
   Err.Raise ECapeUnknownHR
 End Select
 ICapeParameter_value = v
End Property

'ICapeParameterSpec methods

Private Property Get ICapeParameterSpec_Dimensionality() As Variant
 'dimensionality is returned as a real array containing the coefficients for
 ' m, kg, S, A, K, mole, cd, rad, optionally followed by a delta indicator
 'any trailing elements that are zero can be left out
 Dim v() As Double
 ReDim v(0) 'make sure other apps see this as an array of double
 'only fill in for those parameters that have a dimension
 Select Case typ
  Case ParameterHeatInput
   'W = J / s = kg m ^2 / s ^3
   ReDim v(2)
   v(0) = 2 'm
   v(1) = 1 'kg
   v(2) = -3 's
 End Select
 ICapeParameterSpec_Dimensionality = v
End Property

Private Property Get ICapeParameterSpec_Type() As CapeParamType
 'this is a real parameter
 ICapeParameterSpec_Type = CAPE_REAL
End Property

'ICapeRealParameterSpec methods

Private Property Get ICapeRealParameterSpec_DefaultValue() As Double
 'return default value
 Dim def As Double
 Select Case typ
  Case ParameterSplitFactor
   def = 0.5
  Case ParameterHeatInput
   def = 0
  Case Else
   Call SetError("Internal error: unimplemented parameter type", "ICapeRealParameterSpec", "DefaultValue")
   Err.Raise ECapeUnknownHR
 End Select
 ICapeRealParameterSpec_DefaultValue = def
End Property

Private Property Get ICapeRealParameterSpec_LowerBound() As Double
 Dim lw As Double
 Select Case typ
  Case ParameterSplitFactor
   lw = 0#
  Case ParameterHeatInput
   lw = unit.NaN
  Case Else
   Call SetError("Internal error: unimplemented parameter type", "ICapeRealParameterSpec", "LowerBound")
   Err.Raise ECapeUnknownHR
 End Select
 ICapeRealParameterSpec_LowerBound = lw
End Property

Private Property Get ICapeRealParameterSpec_UpperBound() As Double
 Dim up As Double
 Select Case typ
  Case ParameterSplitFactor
   up = 1#
  Case ParameterHeatInput
   up = unit.NaN
  Case Else
   Call SetError("Internal error: unimplemented parameter type", "ICapeRealParameterSpec", "UpperBound")
   Err.Raise ECapeUnknownHR
 End Select
 ICapeRealParameterSpec_UpperBound = up
End Property

Private Function ICapeRealParameterSpec_Validate(ByVal value As Double, message As String) As Boolean
 'check if a value is in range
 Select Case typ
  Case ParameterSplitFactor
   If (value < 0#) Then
    message = "Value below lower limit of 0"
    ICapeRealParameterSpec_Validate = False
    Exit Function
   End If
   If (value > 1#) Then
    message = "Value above upper limit of 1"
    ICapeRealParameterSpec_Validate = False
    Exit Function
   End If
  Case ParameterHeatInput
   'nothing to check
  Case Else
   Call SetError("Internal error: unimplemented parameter type", "ICapeRealParameterSpec", "UpperBound")
   Err.Raise ECapeUnknownHR
 End Select
 ICapeRealParameterSpec_Validate = True 'value is ok
End Function
