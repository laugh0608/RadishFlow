VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
  Persistable = 0  'NotPersistable
  DataBindingBehavior = 0  'vbNone
  DataSourceBehavior  = 0  'vbNone
  MTSTransactionMode  = 0  'NotAnMTSObject
END
Attribute VB_Name = "MaterialObject11"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Option Explicit

Implements MaterialObject

'this is an interface to an externally implemented version 1.1 material object

Private MO11 As ICapeThermoMaterial 'reference to the underlying MO, if version 1.1

Private Sub Class_Initialize()
 'the SetMO method must be called for this MO to be usable
 Set MO11 = Nothing
End Sub

Public Function MaterialObject_SetMO(MO) As Boolean
 'check if this is a version 1.1 MO:
 On Error GoTo not11:
 Set MO11 = MO
 'ok
 MaterialObject_SetMO = True
Exit Function
not11:
 'not an 1.1 MO
 On Error GoTo 0
 MaterialObject_SetMO = False
End Function

Public Function MaterialObject_Duplicate() As MaterialObject
 'duplicate myself
 Dim str As String
 Dim dup As New MaterialObject11
 Dim newMO11 As ICapeThermoMaterial
 If MO11 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 'version 1.1 thermo
 On Error GoTo errorFromMO11
 Set newMO11 = MO11.CreateMaterial
 On Error GoTo errorFromNewMO11
 Call newMO11.CopyFromMaterial(MO11)
 Call dup.MaterialObject_SetMO(newMO11)
 Set MaterialObject_Duplicate = dup
Exit Function
errorFromMO11:
 'obtain the error description:
 On Error GoTo 0
 str = "Failed create duplicate material object: " + GetErrorFromCO(MO11)
 Err.Raise 1, , str
errorFromNewMO11:
 'obtain the error decription:
 On Error GoTo 0
 str = "Failed copy content to duplicate material object: " + GetErrorFromCO(newMO11)
 Err.Raise 1, , str
End Function

Private Function MaterialObject_GetComponentIDs() As Variant
 Dim compIDs
 Dim iComp As ICapeThermoCompounds
 Dim str As String
 Dim formulae, names, boilTmps, molWts, casnos
 If MO11 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 On Error GoTo IcompError
 Set iComp = MO11
 On Error GoTo MOerror
 Call iComp.GetCompoundList(compIDs, formulae, names, boilTmps, molWts, casnos)
 'check the return value
 On Error GoTo dataError
 Call ValidateArray(compIDs, vbString)
 MaterialObject_GetComponentIDs = compIDs
Exit Function
IcompError:
 'failed to get ICapeThermoCompounds
 On Error GoTo 0
 Err.Raise 1, , "Material object does not implement ICapeThermoCompounds"
MOerror:
 'failed to get list of compounds from MO
 On Error GoTo 0
 str = "Failed copy content to duplicate material object: " + GetErrorFromCO(iComp)
 Err.Raise 1, , str
dataError:
 On Error GoTo 0
 Err.Raise 1, , "Invalid compound list returned by the MO: " + Err.description
End Function

Private Function MaterialObject_GetSinglePhasePropList() As Variant
 'get single phase property list
 Dim list
 Dim iRoutine As ICapeThermoPropertyRoutine
 If MO11 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 On Error GoTo iError
 Set iRoutine = MO11
 On Error GoTo MOerror
 list = iRoutine.GetSinglePhasePropList
 On Error GoTo dataError
 Call ValidateArray(list, vbString)
 MaterialObject_GetSinglePhasePropList = list
 Exit Function
iError:
 On Error GoTo 0
 Err.Raise 1, , "Material object does not implement ICapePropertyRoutine"
MOerror:
 On Error GoTo 0
 Err.Raise 1, , "Failed to get list of single-phase properties from material object: " + GetErrorFromCO(iRoutine)
dataError:
 On Error GoTo 0
 Err.Raise 1, , "Invalid list of single-phase properties from material object: " + Err.description
End Function

Private Function MaterialObject_GetOverallProperty(propName As String, basis As String) As Variant
 Dim values
 If MO11 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 On Error GoTo MOerror
 'get the property
 Call MO11.GetOverallProp(propName, basis, values)
 'check the result
 Call ValidateArray(values, vbDouble)
 'all ok
 MaterialObject_GetOverallProperty = values
Exit Function
MOerror:
 On Error GoTo 0
 Err.Raise 1, , "Failed to get overall " + propName + " from material object: " + GetErrorFromCO(MO11)
dataError:
 On Error GoTo 0
 Err.Raise 1, , "Invalid values for overall " + propName + " from material object: " + Err.description
End Function

Private Function MaterialObject_PresentPhaseList() As Variant
 Dim list, status
 If MO11 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 On Error GoTo MOerror
 'get the phase list (we ignore the returned phase status)
 Call MO11.GetPresentPhases(list, status)
 'check the result
 Call ValidateArray(list, vbString)
 'all ok
 MaterialObject_PresentPhaseList = list
Exit Function
MOerror:
 On Error GoTo 0
 Err.Raise 1, , "Failed to get present phase list from material object: " + GetErrorFromCO(MO11)
dataError:
 On Error GoTo 0
 Err.Raise 1, , "Invalid values for present phase list from material object: " + Err.description
End Function

Private Function MaterialObject_GetSinglePhaseProperty(propName As String, phaseName As String, calcType As String, basis As String) As Variant
 Dim values
 If MO11 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 On Error GoTo MOerror
 'get the property (calcType has no meaning in version 1.1, we ignore this argument)
 Call MO11.GetSinglePhaseProp(propName, phaseName, basis, values)
 'check the result
 Call ValidateArray(values, vbDouble)
 'all ok
 MaterialObject_GetSinglePhaseProperty = values
Exit Function
MOerror:
 On Error GoTo 0
 Err.Raise 1, , "Failed to get " + propName + " for phase " + phaseName + " from material object: " + GetErrorFromCO(MO11)
dataError:
 On Error GoTo 0
 Err.Raise 1, , "Invalid values for " + propName + " of phase " + phaseName + " from material object: " + Err.description
End Function

Private Sub MaterialObject_CalcSinglePhaseProperty(propName As String, phaseName As String)
 Dim props
 Dim iRoutine As ICapeThermoPropertyRoutine
 If MO11 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 On Error GoTo iError
 Set iRoutine = MO11
 On Error GoTo MOerror
 'create an array of properties
 ReDim props(0 To 0) As String
 props(0) = propName
 Call iRoutine.CalcSinglePhaseProp(props, phaseName)
 'all ok
 Exit Sub
MOerror:
 On Error GoTo 0
 Err.Raise 1, , "Failed to calculate " + propName + " for phase " + phaseName + ": " + GetErrorFromCO(iRoutine)
iError:
 On Error GoTo 0
 Err.Raise 1, , "Material object does not implement ICapePropertyRoutine"
End Sub

Private Function MaterialObject_TemperatureFromPHFlash(nCompounds As Integer, overallComposition() As Double, enthalpy As Double, pressure As Double) As Double
 Dim v, phaseList, phaseStatus, aggStates, keyComps, flashSpec1, flashSpec2
 Dim i As Integer
 Dim iPhases As ICapeThermoPhases
 Dim iEquilibrium As ICapeThermoEquilibriumRoutine
 If MO11 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 'set composition
 On Error GoTo setCompositionError
 ReDim v(0 To nCompounds - 1) As Double
 For i = 1 To nCompounds
  v(i - 1) = overallComposition(i)
 Next i
 Call MO11.SetOverallProp("fraction", "mole", v)
 'set pressure
 On Error GoTo setPressureError
 ReDim v(0 To 0) As Double
 v(0) = pressure
 Call MO11.SetOverallProp("pressure", vbNullString, v)
 'set enthalpy
 On Error GoTo setEnthalpyError
 v(0) = enthalpy
 Call MO11.SetOverallProp("enthalpy", "mole", v)
 'all phases are allowed in this flash, get a list of all possible phases
 On Error GoTo iError
 Set iPhases = MO11
 On Error GoTo phaseListError
 Call iPhases.GetPhaseList(phaseList, aggStates, keyComps)
 'set present phases and status
 On Error GoTo setPhasesError
 ReDim phaseStatus(0 To UBound(phaseList)) As Long
 For i = 0 To UBound(phaseList)
  phaseStatus(i) = CAPE_UNKNOWNPHASESTATUS 'we do not have an initial guess for the phases, their composition and phase fractions
 Next i
 Call MO11.SetPresentPhases(phaseList, phaseStatus)
 'perform PH flash
 On Error GoTo iError2
 Set iEquilibrium = MO11
 On Error GoTo setFlashError
 ReDim flashSpec1(0 To 2) As String
 ReDim flashSpec2(0 To 2) As String
 'flash spec 1 is pressure for overall phase
 flashSpec1(0) = "Pressure"
 flashSpec1(1) = vbNullString
 flashSpec1(2) = "Overall"
 'flash spec 2 is enthalpy for overall phase
 flashSpec2(0) = "Enthalpy"
 flashSpec2(1) = vbNullString
 flashSpec2(2) = "Overall"
 'calc the flash, with unspecified solution type
 Call iEquilibrium.CalcEquilibrium(flashSpec1, flashSpec2, "Unspecified")
 'get temperature
 On Error GoTo 0
 v = MaterialObject_GetOverallProperty("temperature", vbNullString) 'this routine takes care of error handling, see above
 If (UBound(v) <> 0) Then Err.Raise 1, , "Unexpected value for temperature: scalar expected"
 MaterialObject_TemperatureFromPHFlash = v(0)
 Exit Function
setCompositionError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set overall composition on material: " + GetErrorFromCO(MO11)
setPressureError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set overall pressure on material: " + GetErrorFromCO(MO11)
setEnthalpyError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set overall enthalpy on material: " + GetErrorFromCO(MO11)
setPhasesError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set present phases on material: " + GetErrorFromCO(MO11)
setFlashError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to calculate PH flash on material: " + GetErrorFromCO(MO11)
iError:
 On Error GoTo 0
 Err.Raise 1, , "Material object does not expose ICapeThermoPhases interface"
phaseListError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to get list of supported phases from material object: " + GetErrorFromCO(iPhases)
iError2:
 On Error GoTo 0
 Err.Raise 1, , "Material object does not expose ICapeThermoEquilibriumRoutine interface"
End Function

Private Sub MaterialObject_SetFromFlowTPX(nCompounds As Integer, flow As Double, overallComposition() As Double, temperature As Double, pressure As Double)
 Dim v, phaseList, phaseStatus, aggStates, keyComps, flashSpec1, flashSpec2
 Dim i As Integer
 Dim iPhases As ICapeThermoPhases
 Dim iEquilibrium As ICapeThermoEquilibriumRoutine
 If MO11 Is Nothing Then
  'caller must call SetMO first
  Err.Raise 1, , "Internal material object not initialized"
 End If
 'set composition
 On Error GoTo setCompositionError
 ReDim v(0 To nCompounds - 1) As Double
 For i = 1 To nCompounds
  v(i - 1) = overallComposition(i)
 Next i
 Call MO11.SetOverallProp("fraction", "mole", v)
 'set temperature
 On Error GoTo setTemperatureError
 ReDim v(0 To 0) As Double
 v(0) = temperature
 Call MO11.SetOverallProp("temperature", vbNullString, v)
 'set pressure
 On Error GoTo setPressureError
 v(0) = pressure
 Call MO11.SetOverallProp("pressure", vbNullString, v)
 'set pressure
 On Error GoTo setFlowError
 v(0) = flow
 Call MO11.SetOverallProp("totalFlow", "mole", v)
 'all phases are allowed in this flash, get a list of all possible phases
 On Error GoTo iError
 Set iPhases = MO11
 On Error GoTo phaseListError
 Call iPhases.GetPhaseList(phaseList, aggStates, keyComps)
 'set present phases and status
 On Error GoTo setPhasesError
 ReDim phaseStatus(0 To UBound(phaseList)) As Long
 For i = 0 To UBound(phaseList)
  phaseStatus(i) = CAPE_UNKNOWNPHASESTATUS 'we do not have an initial guess for the phases, their composition and phase fractions
 Next i
 Call MO11.SetPresentPhases(phaseList, phaseStatus)
 'perform TP flash
 On Error GoTo iError2
 Set iEquilibrium = MO11
 On Error GoTo setFlashError
 ReDim flashSpec1(0 To 2) As String
 ReDim flashSpec2(0 To 2) As String
 'flash spec 1 is pressure for overall phase
 flashSpec1(0) = "Pressure"
 flashSpec1(1) = vbNullString
 flashSpec1(2) = "Overall"
 'flash spec 2 is temperature for overall phase
 flashSpec2(0) = "Temperature"
 flashSpec2(1) = vbNullString
 flashSpec2(2) = "Overall"
 'calc the flash, with unspecified solution type
 Call iEquilibrium.CalcEquilibrium(flashSpec1, flashSpec2, "Unspecified")
 Exit Sub
setCompositionError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set overall composition on material: " + GetErrorFromCO(MO11)
setTemperatureError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set overall temperature on material: " + GetErrorFromCO(MO11)
setPressureError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set overall pressure on material: " + GetErrorFromCO(MO11)
setFlowError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set total flow on material: " + GetErrorFromCO(MO11)
setPhasesError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to set present phases on material: " + GetErrorFromCO(MO11)
setFlashError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to calculate TP flash on material: " + GetErrorFromCO(MO11)
iError:
 On Error GoTo 0
 Err.Raise 1, , "Material object does not expose ICapeThermoPhases interface"
phaseListError:
 On Error GoTo 0
 Err.Raise 1, , "Failed to get list of supported phases from material object: " + GetErrorFromCO(iPhases)
iError2:
 On Error GoTo 0
 Err.Raise 1, , "Material object does not expose ICapeThermoEquilibriumRoutine interface"
End Sub



