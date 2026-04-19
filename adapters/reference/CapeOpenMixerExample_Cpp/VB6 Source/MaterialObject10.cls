VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
  Persistable = 0  'NotPersistable
  DataBindingBehavior = 0  'vbNone
  DataSourceBehavior  = 0  'vbNone
  MTSTransactionMode  = 0  'NotAnMTSObject
END
Attribute VB_Name = "MaterialObject10"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Option Explicit

Implements MaterialObject

'this is an interface to an externally implemented version 1.0 material object

Private MO10 As ICapeThermoMaterialObject 'reference to the underlying MO, if version 1.0

Private Sub Class_Initialize()
 'the SetMO method must be called for this MO to be usable
 Set MO10 = Nothing
End Sub

Public Function MaterialObject_SetMO(MO) As Boolean
 'check if this is a version 1.0 MO:
 On Error GoTo not10
 Set MO10 = MO
 'ok
 MaterialObject_SetMO = True
 Exit Function
not10:
 'not an 1.0 MO either, we do not support this
 On Error GoTo 0
 MaterialObject_SetMO = False
End Function

Public Function MaterialObject_Duplicate() As MaterialObject
 'duplicate myself
 Dim dup As New MaterialObject10
 If MO10 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 On Error GoTo MOerror
 Call dup.MaterialObject_SetMO(MO10.Duplicate)
 Set MaterialObject_Duplicate = dup
 'ok
Exit Function
MOerror:
 'get error from MO
 On Error GoTo 0
 Dim str As String
 str = "Failed to duplicate MO: " + GetErrorFromCO(MO10)
 Err.Raise 1, , str
End Function

Private Function MaterialObject_GetComponentIDs() As Variant
 Dim compIDs
 If MO10 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 'get list of compounds from MO
 On Error GoTo MOerror
 compIDs = MO10.ComponentIds
 'check the return value
 On Error GoTo dataError
 Call ValidateArray(compIDs, vbString)
 MaterialObject_GetComponentIDs = compIDs
Exit Function
MOerror:
 On Error GoTo 0
 Dim str As String
 str = "Failed to get list of compounds from MO: " + GetErrorFromCO(MO10)
 Err.Raise 1, , str
dataError:
 On Error GoTo 0
 Err.Raise 1, , "Invalid compound list returned by the MO: " + Err.description
End Function

Private Function MaterialObject_GetSinglePhasePropList() As Variant
 'get single phase property list; does not exist in version 1.0; we return a list with all supported properties instead
 Dim list
 If MO10 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 On Error GoTo MOerror
 list = MO10.GetPropList
 On Error GoTo dataError
 Call ValidateArray(list, vbString)
 MaterialObject_GetSinglePhasePropList = list
 Exit Function
MOerror:
 On Error GoTo 0
 Err.Raise 1, , "Failed to get list of properties from material object: " + GetErrorFromCO(MO10)
dataError:
 On Error GoTo 0
 Err.Raise 1, , "Invalid list of properties from material object: " + Err.description
End Function

Private Function MaterialObject_GetOverallProperty(propName As String, basis As String) As Variant
 Dim values
 If MO10 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 On Error GoTo MOerror
 'get the property (we presume that this is one of the special properties that takes NULL for calcType)
 values = MO10.GetProp(propName, "Overall", Empty, vbNullString, basis)
 'check the result
 Call ValidateArray(values, vbDouble)
 'all ok
 MaterialObject_GetOverallProperty = values
Exit Function
MOerror:
 On Error GoTo 0
 Err.Raise 1, , "Failed to get overall " + propName + " from material object: " + GetErrorFromCO(MO10)
dataError:
 On Error GoTo 0
 Err.Raise 1, , "Invalid values for overall " + propName + " from material object: " + Err.description
End Function

Private Function MaterialObject_PresentPhaseList() As Variant
 Dim list
 If MO10 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 On Error GoTo MOerror
 'get the phase list
 list = MO10.PhaseIds
 'check the result
 Call ValidateArray(list, vbString)
 'all ok
 MaterialObject_PresentPhaseList = list
Exit Function
MOerror:
 On Error GoTo 0
 Err.Raise 1, , "Failed to get present phase list from material object: " + GetErrorFromCO(MO10)
dataError:
 On Error GoTo 0
 Err.Raise 1, , "Invalid values for present phase list from material object: " + Err.description
End Function

Private Function MaterialObject_GetSinglePhaseProperty(propName As String, phaseName As String, calcType As String, basis As String) As Variant
 Dim values
 If MO10 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 On Error GoTo MOerror
 'get the property
 values = MO10.GetProp(propName, phaseName, Empty, calcType, basis)
 'check the result
 Call ValidateArray(values, vbDouble)
 'all ok
 MaterialObject_GetSinglePhaseProperty = values
Exit Function
MOerror:
 On Error GoTo 0
 Err.Raise 1, , "Failed to get " + propName + " for phase " + phaseName + " from material object: " + GetErrorFromCO(MO10)
dataError:
 On Error GoTo 0
 Err.Raise 1, , "Invalid values for " + propName + " of phase " + phaseName + " from material object: " + Err.description
End Function

Private Sub MaterialObject_CalcSinglePhaseProperty(propName As String, phaseName As String)
 Dim props, phases
 If MO10 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 On Error GoTo MOerror
 'create an array of properties
 ReDim props(0 To 0) As String
 props(0) = propName
 'create an array of phases
 ReDim phases(0 To 0) As String
 phases(0) = phaseName
 Call MO10.CalcProp(props, phases, "mixture") 'we only calculate mixture properties in this unit operation
 'all ok
 Exit Sub
MOerror:
 On Error GoTo 0
 Err.Raise 1, , "Failed to calculate " + propName + " for phase " + phaseName + ": " + GetErrorFromCO(MO10)
End Sub

Private Function MaterialObject_TemperatureFromPHFlash(nCompounds As Integer, overallComposition() As Double, enthalpy As Double, pressure As Double) As Double
 Dim v
 Dim i As Integer
 If MO10 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 'set composition
 On Error GoTo setCompositionError
 ReDim v(0 To nCompounds - 1) As Double
 For i = 1 To nCompounds
  v(i - 1) = overallComposition(i)
 Next i
 Call MO10.SetProp("fraction", "overall", Empty, vbNullString, "mole", v)
 'set pressure
 On Error GoTo setPressureError
 ReDim v(0 To 0) As Double
 v(0) = pressure
 Call MO10.SetProp("pressure", "overall", Empty, vbNullString, vbNullString, v)
 'set enthalpy
 On Error GoTo setEnthalpyError
 v(0) = enthalpy
 Call MO10.SetProp("enthalpy", "overall", Empty, "mixture", "mole", v)
 'perform PH flash
 On Error GoTo setFlashError
 Call MO10.CalcEquilibrium("PH", Empty)
 'get temperature
 On Error GoTo 0
 v = MaterialObject_GetOverallProperty("temperature", vbNullString) 'this routine takes care of error handling, see above
 If (UBound(v) <> 0) Then Err.Raise 1, , "Unexpected value for temperature: scalar expected"
 MaterialObject_TemperatureFromPHFlash = v(0)
 Exit Function
setCompositionError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set overall composition on material: " + GetErrorFromCO(MO10)
setPressureError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set overall pressure on material: " + GetErrorFromCO(MO10)
setEnthalpyError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set overall enthalpy on material: " + GetErrorFromCO(MO10)
setFlashError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to calculate PH flash on material: " + GetErrorFromCO(MO10)
End Function

Private Sub MaterialObject_SetFromFlowTPX(nCompounds As Integer, flow As Double, overallComposition() As Double, temperature As Double, pressure As Double)
 'set flow, composition, T and P and perform an equilibrium calculation
 Dim v
 Dim i As Integer
 If MO10 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 'set composition
 On Error GoTo setCompositionError
 ReDim v(0 To nCompounds - 1) As Double
 For i = 1 To nCompounds
  v(i - 1) = overallComposition(i)
 Next i
 Call MO10.SetProp("fraction", "overall", Empty, vbNullString, "mole", v)
 'set temperature
 On Error GoTo setTemperatureError
 ReDim v(0 To 0) As Double
 v(0) = temperature
 Call MO10.SetProp("temperature", "overall", Empty, vbNullString, vbNullString, v)
 'set pressure
 On Error GoTo setPressureError
 v(0) = pressure
 Call MO10.SetProp("pressure", "overall", Empty, vbNullString, vbNullString, v)
 'set totalflow
 On Error GoTo setFlowError
 v(0) = flow
 Call MO10.SetProp("totalFlow", "overall", Empty, vbNullString, "mole", v)
 'perform TP flash
 On Error GoTo setFlashError
 Call MO10.CalcEquilibrium("TP", Empty)
 Exit Sub
setCompositionError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set overall composition on material: " + GetErrorFromCO(MO10)
setTemperatureError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set overall temperature on material: " + GetErrorFromCO(MO10)
setPressureError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set overall pressure on material: " + GetErrorFromCO(MO10)
setFlowError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set totalFlow on material: " + GetErrorFromCO(MO10)
setFlashError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to calculate TP flash on material: " + GetErrorFromCO(MO10)
End Sub


