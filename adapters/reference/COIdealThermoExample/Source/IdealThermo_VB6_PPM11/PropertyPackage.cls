VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
  Persistable = 1  'Persistable
  DataBindingBehavior = 0  'vbNone
  DataSourceBehavior  = 0  'vbNone
  MTSTransactionMode  = 0  'NotAnMTSObject
END
Attribute VB_Name = "PropertyPackage"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = True
Option Explicit
Option Compare Text
'this is the property package class; it cannot be created directly
' by the PME; instead, the PME should use the PackageManager class to
' create an instance of a PropertyPackage
'this class is marked persistable, so that it implements methods
' ReadProperties and WriteProperties (behind the screens this will
' implement IPersistStreamInit

Implements ICapeIdentification             'required for all CAPE-OPEN components
Implements ICapeUtilities                  'required for all top-level CAPE-OPEN components
Implements ECapeRoot                       'required in case of components that throw errors
Implements ECapeUnknown                    'required in case of components that throw errors
Implements ECapeUser                       'required in case of components that throw errors (all other error interfaces derive from this)
Implements ECapeNoImpl                     'for ECapeNoImplHR errors
Implements ECapeIllegalAccess              'for ECapeIllegalAccessHR errors
Implements ECapeBadInvOrder                'for ECapeBadInvOrderHR errors
Implements ECapeComputation                'for ECapeComputationHR errors
Implements ECapePersistenceSystemError     'for ECapePersistenceSystemErrorHR errors
Implements ECapeThrmPropertyNotAvailable   'for partial results
Implements ICapeThermoMaterialContext      'for the context material
Implements ICapeThermoCompounds            'exposes the compounds and their properties
Implements ICapeThermoPhases               'exposes the phases and their properties
Implements ICapeThermoPropertyRoutine      'for property calculations
Implements ICapeThermoEquilibriumRoutine   'for flash calculations
Implements ICapeThermoUniversalConstant    'exposes universal constants

'name and description
Dim name As String, description As String
'error variables (we have a simple error handling mechanism, providing only name(=desc) and scope and interface
Dim errDesc As String, errIface As String, errScope As String, requestedOperation As String
'simulation context
Dim simulationContext
'temp file name for persistence
Dim TempFileName As String
'handle to the property package
Dim ppHandle As Long
'check whether Terminate has been called
Dim terminateCalled As Boolean
'check for initialization
Dim initialized As Boolean
'NaN
Dim NaN As Double
'context material object
Dim contextMO As ICapeThermoMaterial
'compounds on context material
Dim contextCompIndices() As Long

'function to store error

Private Sub SetError(desc As String, iface As String, scope As String)
 'call this function before returning a CAPE-OPEN error
 ' these strings are then available via the CAPE-OPEN error interfaces
 errDesc = desc
 errIface = iface
 errScope = scope
End Sub

'contructor

Private Sub Class_Initialize()
'name will be overridden after creation
description = "CO-LaN Example Ideal Thermo Property Package VB6 implementation"
TempFileName = "" 'set to content of property package in case we get loaded from persistence
Call InitializeProperties 'make sure the lists of property names are initialized
ppHandle = PPCreatePropertyPackage() 'create the property package
terminateCalled = False 'set to true at terminate
initialized = False 'the underlying property package is not yet initialized
Set contextMO = Nothing 'set by ICapeThermoMaterialContext
'VB6 has no native support for NaN values (not-a-number); this is however the value used for numbers that
' are missing in CAPE-OPEN, so we will generate one with a detour here
NaN = GetQNaN()
End Sub

Private Function GetQNaN() As Double
'generate a quiet NaN, Visual Basic has no native support for this
On Error Resume Next
GetQNaN = 0 / 0
End Function

' loading and saving

Private Sub Class_InitProperties()
'this is called on InitNew
End Sub

Private Sub Class_ReadProperties(PropBag As PropertyBag)
 'create a temp file name (will be used during Init)
 TempFileName = GetTempFileName
 'read stored data from property bag
 Dim storedData As String
 storedData = PropBag.ReadProperty("StoredData", "")
 If storedData = "" Then
  Call SetError("Failed to retrieve stored data", "IPersistStreamInit", "Load")
  Err.Raise ECapePersistenceSystemErrorHR
 End If
 'save to temp file
 On Error GoTo persistenceError
 Open TempFileName For Output As #1
 Print #1, storedData; 'new lines are included
 Close 1
 Exit Sub
 'generic error handler
persistenceError:
 On Error Resume Next
 Close #1
 Kill TempFileName
 On Error GoTo 0
 TempFileName = ""
 Call SetError(Err.description, "IPersistStreamInit", "Load")
 Err.Raise ECapePersistenceSystemErrorHR
End Sub

Private Sub Class_WriteProperties(PropBag As PropertyBag)
 Call Init("ICapePersistStreamInit", "Save")
 Dim storedData As String
 Dim c As String * 1
 Dim tempFile As String 'local temp file name
 tempFile = GetTempFileName
 If Not PPSave(ppHandle, tempFile) Then
  Call SetError("Property package failed to save: " + PPGetLastError(ppHandle), "IPersistStreamInit", "Save")
  Err.Raise ECapePersistenceSystemErrorHR
 End If
 On Error GoTo persistenceError
 'read all text into a string
 Open tempFile For Binary Access Read As #1
 Do Until EOF(1)
  Get #1, , c
  If (Asc(c) <> 0) Then storedData = storedData + c 'prevent the terminating 0 character from being in the string
 Loop
 Close #1
 Call PropBag.WriteProperty("StoredData", storedData)
 Kill tempFile
 Exit Sub
 'generic error handler
persistenceError:
 On Error Resume Next
 Close #1
 Kill tempFile
 On Error GoTo 0
 Call SetError(Err.description, "IPersistStreamInit", "Save")
 Err.Raise ECapePersistenceSystemErrorHR
End Sub

'destructor

Private Sub Class_Terminate()
 Call ICapeUtilities_Terminate 'in case the PME failed to do so
End Sub

'routine to check initialization
' we do not depend on the PME called Initialize at all, or in the right order
' instead, we call this routine in case the property package must be initialized
' and we keep track of it so that we only initialize once
'make sure that the caller is not catching errors while calling this routine

Private Sub Init(interface As String, scope As String)
'initialize (either from name or from stored content); return non-zero HRESULT in case of failure
If (terminateCalled) Then
 'terminate has been called
 requestedOperation = "N/A"
 Call SetError("Terminate has been called", interface, scope)
 Err.Raise ECapeBadInvOrderHR
End If
If Not (initialized) Then
 Dim initRes As Boolean
 If TempFileName = "" Then
  'initialize from name
  initRes = PPLoadFromPPFile(ppHandle, name)
 Else
  'initialze from temp file
  initRes = PPLoad(ppHandle, TempFileName)
 End If
 If Not initRes Then
  'loading failed
  Call SetError(PPGetLastError(ppHandle), interface, scope)
  Err.Raise ECapeUnknownHR
 End If
 If TempFileName = "" Then
  'we have not been loaded. It is not clear from the spec whether on a derived object, InitNew should be called.
  ' VB6 is pretty picky about saving objects that have not been initialized, so we make sure to call InitNew at this
  ' point (which will fail in case already called by the PME)
  'note that this requires a reference to Persist.tlb in the Typelib folder
  On Error Resume Next
  Dim ipi As IPersistStreamInit
  Set ipi = Me
  Call ipi.InitNew
  On Error GoTo 0
 Else
  'clean up temp file
  On Error Resume Next
  Call Kill(TempFileName)
  TempFileName = ""
  On Error GoTo 0
 End If
 initialized = True
End If
End Sub

'ICapeIdentification methods

Private Property Let ICapeIdentification_ComponentDescription(ByVal RHS As String)
 description = RHS
End Property

Private Property Get ICapeIdentification_ComponentDescription() As String
 'get description
 ICapeIdentification_ComponentDescription = description
End Property

Private Property Let ICapeIdentification_ComponentName(ByVal RHS As String)
 name = RHS
End Property

Private Property Get ICapeIdentification_ComponentName() As String
 'get name
 ICapeIdentification_ComponentName = name
End Property

'helper routines

'get a temporary file name
Private Function GetTempFileName() As String
Dim tempFile As String
Dim tempPath As String
tempPath = Space(512) 'MAX_PATH
Call GetTempPathA(512, tempPath)
tempFile = Space(512)
Call GetTempFileNameA(tempPath, "PP", 0, tempFile)
GetTempFileName = Left$(tempFile, InStr(tempFile, Chr$(0)) - 1)
End Function

'get an error from a CAPE-OPEN object
' call this routine only from within an error handler; it depends on err.description
Private Function GetCOError(o As Object) As String
 GetCOError = Err.description 'this is what we will return in case the CAPE-OPEN object does not return something useful
 On Error GoTo noUserInterface
 Dim eUser As ECapeUser
 Set eUser = o 'get the ECapeUser interface from the CAPE-OPEN object
 GetCOError = eUser.description
 Exit Function
noUserInterface:
 On Error GoTo noRootErrorInterface
 Dim eRoot As ECapeRoot
 Set eRoot = o 'get the ECapeRoot interface from the CAPE-OPEN object
 GetCOError = eRoot.name
 Exit Function
noRootErrorInterface:
End Function

'get compound list from variant
' make sure that the caller is not catching errors while calling this routine
' make sure the package is initialized when calling this routine
' this is not the fastest way to go about it; production implementations should aim for a faster method (e.g. use a hash table)
' we do not perform checks that the underlying property package will perform, such as compounds being in the list multiple times
' in some contexts, an empty list means all compounds no the property package
Private Sub GetCompoundIndices(compIDs As Variant, iface As String, scope As String, emptyListOK As Boolean, compIndices() As Long)
 Dim count As Long
 Dim i As Long, j As Long
 Dim ppCompCount As Long
 'get package compound count
 If Not PPGetCompoundCount(ppHandle, ppCompCount) Then
  Call SetError("Failed to get compound count from property package: " + PPGetLastError(ppHandle), iface, scope)
  Err.Raise ECapeUnknownHR
 End If
 'check for empty list
 If IsEmpty(compIDs) Then
  If (emptyListOK) Then
   'this means all compounds
   ReDim compIndices(0 To ppCompCount - 1)
   For i = 0 To ppCompCount - 1
    compIndices(i) = i
   Next i
   Exit Sub
  End If
  'not ok (and must be from MO)
  Call SetError("Empty list of compounds from material object", iface, scope)
  Err.Raise ECapeUnknownHR
 End If
 'check number of compounds in the list
 On Error GoTo invalidCompList 'calling Ubound on an invalid or empty array throws an error
 count = UBound(compIDs, 1) + 1
 On Error GoTo 0
 'loop over compounds
 ReDim compIndices(0 To count - 1)
 On Error GoTo invalidComponent 'in case we cannot assign to string
 For i = 0 To count - 1
  Dim compName As String
  compName = compIDs(i)
  'check if we can locate this component
  Dim found As Boolean
  found = False
  For j = 0 To ppCompCount - 1
   Dim ppCompName
   ppCompName = PPGetCompoundStringConstant(ppHandle, j, CompoundName)
   If IsEmpty(ppCompName) Then
    'no luck
    On Error GoTo 0
    Call SetError("Failed to get compound name from property package: " + PPGetLastError(ppHandle), iface, scope)
    Err.Raise ECapeUnknownHR
   End If
   If (ppCompName = compName) Then
    compIndices(i) = j
    found = True
    Exit For
   End If
  Next j
  If Not found Then
   On Error GoTo 0
   Call SetError("Invalid compound ID """ + compName + """ from material object (no such compound)", iface, scope)
   Err.Raise ECapeUnknownHR
  End If
 Next i
 On Error GoTo 0
 'all done
 Exit Sub
invalidCompList:
  Call SetError("Invalid or empty list of compounds", iface, scope)
  Err.Raise ECapeUnknownHR
invalidComponent:
  Call SetError("Invalid list of compounds; expected a string array", iface, scope)
  Err.Raise ECapeUnknownHR
End Sub

'get an overall property from the context material
' make sure that the caller does not catch errors and that the context material has been set
Private Function GetOverallProp(iface As String, scope As String, propName As String, basis As String, expectedCount As Long) As Double()
 Dim res() As Double
 'get the property
 Dim val As Variant
 On Error GoTo getPropError
 Call contextMO.GetOverallProp(propName, basis, val)
 'check the length
 Dim count As Long
 On Error GoTo invalidValueError
 count = UBound(val, 1) + 1
 If count <> expectedCount Then
  On Error GoTo 0
  Call SetError("Invalid values for overall " + propName + " from material object: unexpected number of values", iface, scope)
  Err.Raise ECapeUnknownHR
 End If
 Dim i As Long
 ReDim res(0 To count - 1)
 For i = 0 To count - 1
  res(i) = val(i)
 Next i
 'all ok
 GetOverallProp = res
 Exit Function
getPropError:
 Call SetError("Failed to get overall " + propName + " from context material: " + GetCOError(contextMO), iface, scope)
 Err.Raise ECapeUnknownHR
invalidValueError:
 Call SetError("Invalid values for overall " + propName + " from context material: " + Err.description, iface, scope)
 Err.Raise ECapeUnknownHR
End Function

'get an phase property from the context material
' make sure that the caller does not catch errors and that the context material has been set
Private Function GetPhaseProp(iface As String, scope As String, propName As String, phaseID As String, basis As String, expectedCount As Long) As Double()
 Dim res() As Double
 'get the property
 Dim val As Variant
 On Error GoTo getPropError
 Call contextMO.GetSinglePhaseProp(propName, phaseID, basis, val)
 'check the length
 Dim count As Long
 On Error GoTo invalidValueError
 count = UBound(val, 1) + 1
 If count <> expectedCount Then
  On Error GoTo 0
  Call SetError("Invalid values for " + phaseID + " " + propName + " from material object: unexpected number of values", iface, scope)
  Err.Raise ECapeUnknownHR
 End If
 Dim i As Long
 ReDim res(0 To count - 1)
 For i = 0 To count - 1
  res(i) = val(i)
 Next i
 'all ok
 GetPhaseProp = res
 Exit Function
getPropError:
 Call SetError("Failed to get " + phaseID + " " + propName + " from context material: " + GetCOError(contextMO), iface, scope)
 Err.Raise ECapeUnknownHR
invalidValueError:
 Call SetError("Invalid values for " + phaseID + " " + propName + " from context material: " + Err.description, iface, scope)
 Err.Raise ECapeUnknownHR
End Function

'ICapeThermoCompounds interface

Private Function ICapeThermoCompounds_GetCompoundConstant(ByVal props As Variant, ByVal compIDs As Variant) As Variant
 Call Init("ICapeThermoCompounds ", "GetCompoundConstant")
 'gets string or real components for all compounds on material object. Results is returned as an array
 ' of Variants (containing string or double values). For version 1.1, this function may return partial
 ' results in case a property is not available for some compounds. This is not implemented here as the
 ' underlying property package implementation will return values for all compounds
 Dim nProp As Long, iProp As Long, nComp As Long, iComp As Long, compIndex As Long
 Dim realVal As Double
 Dim propName As String
 Dim compIndices() As Long
 'get compounds
 Call GetCompoundIndices(compIDs, "ICapeThermoCompounds ", "GetCompoundConstant", True, compIndices)
 nComp = UBound(compIndices, 1) + 1 'gives an error on empty compound list
 'check the list of properties
 On Error GoTo EmptyPropList
 nProp = UBound(props, 1) + 1
 On Error GoTo 0
 'alloc results list
 Dim res
 ReDim res(0 To nComp * nProp - 1) As Variant
 Dim index As Long 'index into the result array
 index = 0
 'loop over the properties
 ' production implementations should interpret all property names once and use a hash table
 For iProp = 0 To nProp - 1
  On Error GoTo invalidPropInList
  propName = props(iProp)
  On Error GoTo 0
  For iComp = 0 To nComp - 1
   compIndex = compIndices(iComp)
   If (propName = "molecularWeight") Then
    If Not PPGetCompoundRealConstant(ppHandle, compIndex, MolecularWeight, realVal) Then
failedGetValue:
     Dim e As String
     e = PPGetLastError(ppHandle)
     Call SetError("Failed to get " + propName + " for " + PPGetCompoundStringConstant(ppHandle, compIndex, CompoundName) + ": " + e, "ICapeThermoCompounds ", "GetCompoundConstant")
     Err.Raise ECapeComputationHR
    End If
    res(index) = realVal
   ElseIf (propName = "criticalTemperature") Then
    If Not PPGetCompoundRealConstant(ppHandle, compIndex, CriticalTemperature, realVal) Then GoTo failedGetValue
    res(index) = realVal
   ElseIf (propName = "criticalPressure") Then
    If Not PPGetCompoundRealConstant(ppHandle, compIndex, CriticalPressure, realVal) Then GoTo failedGetValue
    res(index) = realVal
   ElseIf (propName = "criticalVolume") Then
    If Not PPGetCompoundRealConstant(ppHandle, compIndex, CriticalVolume, realVal) Then GoTo failedGetValue
    res(index) = realVal
   ElseIf (propName = "criticalDensity") Then
    If Not PPGetCompoundRealConstant(ppHandle, compIndex, CriticalVolume, realVal) Then GoTo failedGetValue
    realVal = 1# / realVal
    res(index) = realVal
   ElseIf (propName = "normalBoilingPoint") Then
    If Not PPGetCompoundRealConstant(ppHandle, compIndex, NormalBoilingPoint, realVal) Then GoTo failedGetValue
    res(index) = realVal
   ElseIf (propName = "liquidDensityAt25C") Then
    If Not PPGetTemperatureDependentProperty(ppHandle, compIndex, LiquidDensity, 298.15, realVal) Then GoTo failedGetValue
    res(index) = realVal
   ElseIf (propName = "liquidVolumeAt25C") Then
    If Not PPGetTemperatureDependentProperty(ppHandle, compIndex, LiquidDensity, 298.15, realVal) Then GoTo failedGetValue
    realVal = 1# / realVal
    res(index) = realVal
   ElseIf (propName = "charge") Then
    realVal = 0 'all compounds have zero charge in this package
    res(index) = realVal
   ElseIf (propName = "casRegistryNumber") Then
    res(index) = PPGetCompoundStringConstant(ppHandle, compIndex, CASNumber)
    If (IsEmpty(res(index))) Then GoTo failedGetValue
   ElseIf (propName = "chemicalFormula") Then
    res(index) = PPGetCompoundStringConstant(ppHandle, compIndex, ChemicalFormula)
    If (IsEmpty(res(index))) Then GoTo failedGetValue
   Else
    'not supported
    Call SetError("Invalid/unsupported compound constant """ + propName + """", "ICapeThermoCompounds", "GetCompoundConstant")
    Err.Raise ECapeInvalidArgumentHR
   End If
   index = index + 1
  Next iComp
 Next iProp
 ICapeThermoCompounds_GetCompoundConstant = res
 Exit Function
 'error handlers:
EmptyPropList:
 Call SetError("Invalid or empty list of properties", "ICapeThermoCompounds ", "GetCompoundConstant")
 Err.Raise ECapeInvalidArgumentHR
invalidPropInList:
 Call SetError("Invalid entry in list of properties", "ICapeThermoCompounds ", "GetCompoundConstant")
 Err.Raise ECapeInvalidArgumentHR
End Function

Private Sub ICapeThermoCompounds_GetCompoundList(compIDs As Variant, formulae As Variant, names As Variant, boilTemps As Variant, molwts As Variant, casnos As Variant)
 Call Init("ICapeThermoCompounds ", "GetCompoundList")
  'get the list of components supported by this property package. This package returns
 ' identical values for names and compIds
 Dim count As Long, i As Long
 Dim d As Double
 Dim v As Variant
 If Not PPGetCompoundCount(ppHandle, count) Then
  Call SetError("Failed to get compound count: " + PPGetLastError(ppHandle), "ICapeThermoCompounds", "GetCompoundList")
  Err.Raise ECapeComputationHR
 End If
 ReDim compIDs(0 To count - 1) As String
 ReDim formulae(0 To count - 1) As String
 ReDim names(0 To count - 1) As String
 ReDim boilTemps(0 To count - 1) As Double
 ReDim molwts(0 To count - 1) As Double
 ReDim casnos(0 To count - 1) As String
 For i = 0 To count - 1
  v = PPGetCompoundStringConstant(ppHandle, i, CompoundName)
  If IsEmpty(v) Then
   Call SetError("Failed to get name of compound: " + PPGetLastError(ppHandle), "ICapeThermoCompounds", "GetCompoundList")
   Err.Raise ECapeComputationHR
  End If
  compIDs(i) = v
  names(i) = v
  'remainder is not critical, no error in case of fail
  v = PPGetCompoundStringConstant(ppHandle, i, ChemicalFormula)
  If (IsEmpty(v)) Then formulae(i) = vbNullString Else formulae(i) = v
  v = PPGetCompoundStringConstant(ppHandle, i, CASNumber)
  If (IsEmpty(v)) Then casnos(i) = vbNullString Else casnos(i) = v
  'Missing real values should be set to NaN
  ' (however, the underlying property package will not fail)
  If Not PPGetCompoundRealConstant(ppHandle, i, NormalBoilingPoint, d) Then d = NaN
  boilTemps(i) = d
  If Not PPGetCompoundRealConstant(ppHandle, i, MolecularWeight, d) Then d = NaN
  molwts(i) = d
 Next i
End Sub

Private Function ICapeThermoCompounds_GetConstPropList() As Variant
 'get a list of all compound constants. Production implementations should cache and copy this list
 Dim res
 ReDim res(0 To 10) As String 'variant array of strings
 res(0) = "molecularWeight"
 res(1) = "criticalTemperature"
 res(2) = "criticalPressure"
 res(3) = "criticalVolume"
 res(4) = "criticalDensity"
 res(5) = "normalBoilingPoint"
 res(6) = "liquidDensityAt25C"
 res(7) = "liquidVolumeAt25C"
 res(8) = "charge"
 res(9) = "casRegistryNumber"
 res(10) = "chemicalFormula"
 ICapeThermoCompounds_GetConstPropList = res
End Function

Private Function ICapeThermoCompounds_GetNumCompounds() As Long
 Call Init("ICapeThermoCompounds ", "GetNumCompounds")
 'return the number of compounds
 Dim count As Long
 If Not PPGetCompoundCount(ppHandle, count) Then
  Call SetError("Failed to get compound count: " + PPGetLastError(ppHandle), "ICapeThermoCompounds", "GetNumCompounds")
  Err.Raise ECapeComputationHR
 End If
 ICapeThermoCompounds_GetNumCompounds = count
End Function

Private Sub ICapeThermoCompounds_GetPDependentProperty(ByVal props As Variant, ByVal Pressure As Double, ByVal compIDs As Variant, propVals As Variant)
 'this package does not expose any P dependent properties
 Call SetError("Property package does not expose pressure dependent properties", "ICapeThermoCompounds", "GetPDependentProperty")
 Err.Raise ECapeNoImplHR
End Sub

Private Function ICapeThermoCompounds_GetPDependentPropList() As Variant
 'we do not support P dependent properties, hence, empty list
 ICapeThermoCompounds_GetPDependentPropList = Empty
End Function

Private Sub ICapeThermoCompounds_GetTDependentProperty(ByVal props As Variant, ByVal Temperature As Double, ByVal compIDs As Variant, propVals As Variant)
 Call Init("ICapeThermoCompounds ", "GetTDependentProperty")
 'get temperature dependent properties; return an array of real values, one value for each compound, one value for each property
 ' if a value is not available for one or more compounds, an ECapeThrmPropertyNotAvailable can be returned, in combination with partial
 ' results. The caller must then check each property value. If on the other hand a request is made for a non-supported property, we
 ' return an error
 Dim partialResults As Boolean
 partialResults = False 'set to true in case of missing values
 Dim nProp As Long, iProp As Long, nComp As Long, iComp As Long, compIndex As Long, propID As Long
 Dim realVal As Double
 Dim propName As String
 Dim compIndices() As Long
 'get compounds
 Call GetCompoundIndices(compIDs, "ICapeThermoCompounds ", "GetTDependentProperty", True, compIndices)
 nComp = UBound(compIndices, 1) + 1 'gives an error on empty compound list
 'check the list of properties
 On Error GoTo EmptyPropList
 nProp = UBound(props, 1) + 1
 On Error GoTo 0
 'alloc results list
 Dim res() As Double
 ReDim res(0 To nComp * nProp - 1)
 Dim index As Long 'index into the result array
 index = 0
 'loop over the properties
 ' production implementations should interpret all property names once and use a hash table
 For iProp = 0 To nProp - 1
  On Error GoTo invalidPropInList
  propName = props(iProp)
  On Error GoTo 0
  'check which property this is (production implementations should use a hash table)
  Dim found As Boolean
  found = False
  For propID = 0 To TDepPropertyCount - 1
   If (TDepPropNames(propID) = propName) Then
    found = True
    Exit For
   End If
  Next propID
  If Not found Then
   'invalid/unsupported T-dependent property
   Call SetError("Invalid/unsupported temperature dependent property """ + propName + """", "ICapeThermoCompounds ", "GetTDependentProperty")
   Err.Raise ECapeInvalidArgumentHR
  End If
  'calculate the propery for all compounds
  For iComp = 0 To nComp - 1
   compIndex = compIndices(iComp)
   If Not PPGetTemperatureDependentProperty(ppHandle, compIndex, propID, Temperature, realVal) Then
    realVal = NaN
    partialResults = True
   End If
   res(index) = realVal
   index = index + 1
  Next iComp
 Next iProp
 propVals = res
 If (partialResults) Then
  'raise a partial results error
  Call SetError("Values for one or more compounds are not available", "ICapeThermoCompounds ", "GetTDependentProperty")
  Err.Raise ECapeThrmPropertyNotAvailableHR
 End If
 Exit Sub
 'error handlers:
EmptyPropList:
 Call SetError("Invalid or empty list of properties", "ICapeThermoCompounds ", "GetTDependentProperty")
 Err.Raise ECapeInvalidArgumentHR
invalidPropInList:
 Call SetError("Invalid entry in list of properties", "ICapeThermoCompounds ", "GetTDependentProperty")
 Err.Raise ECapeInvalidArgumentHR
End Sub

Private Function ICapeThermoCompounds_GetTDependentPropList() As Variant
 'get a list of supported temperature dependent properties; production implementations should cache and copy this list
 Dim res(0 To TDepPropertyCount - 1) As String
 Dim i As Long
 For i = 0 To TDepPropertyCount - 1
  res(i) = TDepPropNames(i)
 Next i
 ICapeThermoCompounds_GetTDependentPropList = res
End Function

'ICapeThermoEquilibriumRoutine interface

Private Sub GetEquilibriumSpec(ByVal specification1 As Variant, ByVal specification2 As Variant, ByVal solutionType As String, ByRef fType As Long, ByRef phaseType As Long, scope As String)
 'helper function, called by CalcEquilibrium and CheckEquilibriumSpec
 ' make sure that the calling function does not catch errors or handles the errors thrown by this routine properly
 Dim haveT As Boolean, haveP As Boolean, haveVFm As Boolean, haveVF As Boolean, haveH As Boolean, haveS As Boolean
 Dim flashSpec As Long, spec As Variant, i As Long
 Dim nElement As Long, prop As String, phase As String, basis As String
 haveT = False
 haveP = False
 haveVFm = False
 haveVF = False
 haveH = False
 haveS = False
 For flashSpec = 1 To 2
  'check the content of this specification:
  If (flashSpec = 1) Then spec = specification1 Else spec = specification2
  'must be an string array with 3 or 4 elements
  On Error GoTo invalidFlashSpec
  nElement = UBound(spec) + 1
  On Error GoTo 0
  If nElement <> 3 And nElement <> 4 Then
   Call SetError("Invalid flash specification " + Trim(Str(flashSpec)) + ": expected 3 or 4 elements", "ICapeThermoEquilibriumRoutine", scope)
   Err.Raise ECapeInvalidArgumentHR
  End If
  If (nElement = 4) Then
   'compound spec must be empty
   Dim s As String
   On Error GoTo noString
   s = spec(3)
   On Error GoTo 0
   If s <> "" Then
    Call SetError("Invalid/unsupported flash specification " + Trim(Str(flashSpec)) + ": compound based specification are not supported", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
  End If
  On Error GoTo noString
  prop = spec(0)
  phase = spec(2)
  basis = spec(1)
  On Error GoTo 0
  'check property
  If (prop = "temperature") Then
   'temperature spec
   If (basis <> "") Then
    Call SetError("Invalid/unsupported flash specification " + Trim(Str(flashSpec)) + ": no basis expected for temperature", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   If (phase <> "overall") Then
    Call SetError("Invalid/unsupported flash specification " + Trim(Str(flashSpec)) + ": expected overall phase for temperature", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   If (haveT) Then
    Call SetError("Multiple temperature specifications", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   haveT = True
  ElseIf (prop = "pressure") Then
   'pressure spec
   If (basis <> "") Then
    Call SetError("Invalid/unsupported flash specification " + Trim(Str(flashSpec)) + ": no basis expected for pressure", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   If (phase <> "overall") Then
    Call SetError("Invalid/unsupported flash specification " + Trim(Str(flashSpec)) + ": expected overall phase for pressure", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   If (haveP) Then
    Call SetError("Multiple pressure specifications", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   haveP = True
  ElseIf (prop = "phaseFraction") Then
   'phaseFraction spec
   Dim isMass As Boolean
   If (basis = "mass") Then
    isMass = True
   ElseIf (basis = "mole") Then
    isMass = False
   Else
    Call SetError("Invalid/unsupported flash specification " + Trim(Str(flashSpec)) + ": expected mole or mass basis for phase fraction", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   If (phase <> "gas") Then
    Call SetError("Invalid/unsupported flash specification " + Trim(Str(flashSpec)) + ": phase fraction is only supported for gas phase", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   If (haveVF Or haveVFm) Then
    Call SetError("Multiple gas fraction specifications", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   If (isMass) Then haveVFm = True Else haveVF = True
  ElseIf (prop = "enthalpy") Then
   'enthalpy spec
   ' basis should be NULL, but we will accept mole and mass as well (and silently ignore)
   If (basis <> "") And (basis <> "mole") And (basis <> "mass") Then
    Call SetError("Invalid/unsupported flash specification " + Trim(Str(flashSpec)) + ": no basis expected for enthalpy", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   If (phase <> "overall") Then
    Call SetError("Invalid/unsupported flash specification " + Trim(Str(flashSpec)) + ": expected overall phase for enthalpy", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   If (haveH) Then
    Call SetError("Multiple enthalpy specifications", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   haveH = True
  ElseIf (prop = "entropy") Then
   'entropy spec
   ' basis should be NULL, but we will accept mole and mass as well (and silently ignore)
   If (basis <> "") And (basis <> "mole") And (basis <> "mass") Then
    Call SetError("Invalid/unsupported flash specification " + Trim(Str(flashSpec)) + ": no basis expected for entropy", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   If (phase <> "overall") Then
    Call SetError("Invalid/unsupported flash specification " + Trim(Str(flashSpec)) + ": expected overall phase for entropy", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   If (haveS) Then
    Call SetError("Multiple entropy specifications", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
   haveS = True
  Else
   'invalid/unsupported spec
    Call SetError("Invalid/unsupported flash specification " + Trim(Str(flashSpec)) + ": """ + prop + """ is not supported for flash specification", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
  End If
 Next flashSpec
 'now we have two out of {T, P, VF, VFm, H, S}; let's see if the combination is supported
 If (haveT) Then
  If (haveP) Then
   fType = TP
  ElseIf (haveVF) Then
   fType = TVF
  ElseIf (haveVFm) Then
   fType = TVFm
  Else
unsupportedCombination:
   Call SetError("Unsupported combination of flash specifications", "ICapeThermoEquilibriumRoutine", scope)
   Err.Raise ECapeInvalidArgumentHR
  End If
 ElseIf (haveP) Then
  If (haveVF) Then
   fType = PVF
  ElseIf (haveVFm) Then
   fType = PVFm
  ElseIf (haveH) Then
   fType = PH
  ElseIf (haveS) Then
   fType = PS
  Else
   GoTo unsupportedCombination
  End If
 Else
  GoTo unsupportedCombination
 End If
 'we have a supported combination, check solution type
 If (solutionType <> "") Then
  If (solutionType <> "unspecified") Then
   'could be normal or retrograde
   If (solutionType = "normal") Then
    'ok in case of VF flashes
    If ((Not haveVF) And (Not haveVFm)) Then
     Call SetError("The normal flash solution type is only allowed in case of a phase fraction specification", "ICapeThermoEquilibriumRoutine", scope)
     Err.Raise ECapeInvalidArgumentHR
    End If
   ElseIf (solutionType = "retrograde") Then
    'valid in case of VF flashes
    If ((Not haveVF) And (Not haveVFm)) Then
     Call SetError("The retrograde flash solution type is only allowed in case of a phase fraction specification", "ICapeThermoEquilibriumRoutine", scope)
     Err.Raise ECapeInvalidArgumentHR
    End If
    '... but we do not support it
    Call SetError("Retrograde flashes are not supported", "ICapeThermoEquilibriumRoutine", scope)
    Err.Raise ECapeInvalidArgumentHR
   End If
  End If
 End If
 'check that we have a context MO
 If contextMO Is Nothing Then
  requestedOperation = "SetMaterial"
  Call SetError("Material context not set", "ICapeThermoEquilibriumRoutine", scope)
  Err.Raise ECapeBadInvOrder
 End If
 'check which phases are present on the context MO
 Dim phaseLabels As Variant, phaseStatus As Variant
 Dim nPhases As Long
 On Error GoTo failedGetPhases
 Call contextMO.GetPresentPhases(phaseLabels, phaseStatus)
 'check phase labels
 On Error GoTo InvalidPhaseList
 nPhases = UBound(phaseLabels) + 1
 On Error GoTo 0
 'we use binary Or on VaporOnly and LiquidOnly to build the phaseType value
 phaseType = 0
 For i = 0 To nPhases - 1
  Dim phaseName As String
  On Error GoTo invalidPresentPhase
  phaseName = phaseLabels(i)
  On Error GoTo 0
  If phaseName = "Gas" Then
   phaseType = phaseType Or VaporOnly
  ElseIf phaseName = "Liquid" Then
   phaseType = phaseType Or LiquidOnly
  Else
   'unsupported phase
   Call SetError("Invalid/undefied phase """ + phaseName + """ in list of present phases from context material", "ICapeThermoEquilibriumRoutine", scope)
   Err.Raise ECapeUnknownHR
  End If
 Next i
 If (phaseType = 0) Then
  'no valid phases present
  Call SetError("No phases are present on the context material", "ICapeThermoEquilibriumRoutine", scope)
  Err.Raise ECapeUnknownHR
 End If
 Exit Sub
 'error handlers:
invalidFlashSpec:
 Call SetError("Invalid or empty flash specification " + Trim(Str(flashSpec)), "ICapeThermoEquilibriumRoutine", scope)
 Err.Raise ECapeInvalidArgumentHR
noString:
 Call SetError("Invalid or empty flash specification " + Trim(Str(flashSpec)) + ": expected string array", "ICapeThermoEquilibriumRoutine", scope)
 Err.Raise ECapeInvalidArgumentHR
failedGetPhases:
 Call SetError("Failed to get list of present phases from context material: " + GetCOError(contextMO), "ICapeThermoEquilibriumRoutine", scope)
 Err.Raise ECapeUnknownHR
InvalidPhaseList:
 Call SetError("Invalid or empty list of present phases from context material", "ICapeThermoEquilibriumRoutine", scope)
 Err.Raise ECapeUnknownHR
invalidPresentPhase:
 Call SetError("Invalid list of present phases from material object: string array expected", "ICapeThermoEquilibriumRoutine", scope)
 Err.Raise ECapeUnknownHR
End Sub

Private Function ICapeThermoEquilibriumRoutine_CheckEquilibriumSpec(ByVal specification1 As Variant, ByVal specification2 As Variant, ByVal solutionType As String) As Boolean
 'check the equilibrium spec
 On Error GoTo notSupported
 Dim fType As Long, phaseType As Long
 Call GetEquilibriumSpec(specification1, specification2, solutionType, fType, phaseType, "CheckEquilibriumSpec")
 ICapeThermoEquilibriumRoutine_CheckEquilibriumSpec = True
 Exit Function
notSupported:
 'not supported, don't raise an error though
 ICapeThermoEquilibriumRoutine_CheckEquilibriumSpec = False
 Exit Function
End Function

Private Sub ICapeThermoEquilibriumRoutine_CalcEquilibrium(ByVal specification1 As Variant, ByVal specification2 As Variant, ByVal solutionType As String)
 Call Init("ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
 Dim fType As Long, phaseType As Long
 Dim mustSetT As Boolean, mustSetP As Boolean
 Dim specVal1 As Double, specVal2 As Double
 Dim X() As Double, values() As Double
 Dim nComp As Long, i As Long
 'check the equilibrium spec (also checks for context material, no explicit check required)
 Call GetEquilibriumSpec(specification1, specification2, solutionType, fType, phaseType, "CalcEquilibrium")
 'number of compounds
 nComp = UBound(contextCompIndices) + 1
 'get the flash spec values, depending on the flash type
 mustSetT = True 'set to false if T is part of flash specifications
 mustSetP = True 'set to false if P is part of flash specifications
 If fType = TP Then
  'TP flash, special case as we use GetOverallTPFraction
  mustSetT = False
  mustSetP = False
  Dim composition
  On Error GoTo TPXError
  Call contextMO.GetOverallTPFraction(specVal1, specVal2, composition)
  On Error GoTo 0
  'check the length
  Dim count As Long
  On Error GoTo invalidCompositionError
  count = UBound(composition, 1) + 1
  On Error GoTo 0
  If count <> nComp Then
   Call SetError("Invalid overall composition values from material object: unexpected number of values", "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
   Err.Raise ECapeUnknownHR
  End If
  ReDim X(0 To nComp - 1)
  On Error GoTo invalidCompositionError
  For i = 0 To count - 1
   X(i) = composition(i)
  Next i
  On Error GoTo 0
 Else
  'get overall composition
  X = GetOverallProp("ICapeThermoEquilibriumRoutine", "CalcEquilibrium", "fraction", "mole", nComp)
  'get first spec value, depending on flash type
  Select Case fType
   Case TVFm, TVF
    'get temperature
    values = GetOverallProp("ICapeThermoEquilibriumRoutine", "CalcEquilibrium", "temperature", vbNullString, 1)
    mustSetT = False
   Case PVF, PVFm, PH, PS
    'get temperature
    values = GetOverallProp("ICapeThermoEquilibriumRoutine", "CalcEquilibrium", "pressure", vbNullString, 1)
    mustSetP = False
   Case Else
    Call SetError("Internal error: unhandled flash type", "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
    Err.Raise ECapeUnknownHR
  End Select
  specVal1 = values(0)
  'get second spec value, depending on flash type
  Select Case fType
   Case TVF, PVF
    'get molar vapor phase fraction
    values = GetPhaseProp("ICapeThermoEquilibriumRoutine", "CalcEquilibrium", "phaseFraction", "gas", "mole", 1)
   Case TVFm, PVFm
    'get mass vapor phase fraction
    values = GetPhaseProp("ICapeThermoEquilibriumRoutine", "CalcEquilibrium", "phaseFraction", "gas", "mass", 1)
   Case PH
    'get overall enthalpy
    values = GetOverallProp("ICapeThermoEquilibriumRoutine", "CalcEquilibrium", "enthalpy", "mole", 1)
   Case PS:
    'get overall entropy
    values = GetOverallProp("ICapeThermoEquilibriumRoutine", "CalcEquilibrium", "entropy", "mole", 1)
   Case Else
    Call SetError("Internal error: unhandled flash type", "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
    Err.Raise ECapeUnknownHR
  End Select
  specVal2 = values(0)
 End If
 'calc the equilibrium
 Dim phaseCount As Long
 Dim T As Double, P As Double
 If Not PPFlash(ppHandle, nComp, contextCompIndices(0), X(0), fType, phaseType, specVal1, specVal2, phaseCount, T, P) Then
  Call SetError("Flash failed: " + PPGetLastError(ppHandle), "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
  Err.Raise ECapeComputationHR
 End If
 'set present phases
 Dim phaseIndex As Long
 Dim phases
 Dim phaseStatus
 ReDim phases(0 To phaseCount - 1) As String 'Variant array of strings
 ReDim phaseStatus(0 To phaseCount - 1) As Long 'Variant array of integers
 For phaseIndex = 0 To phaseCount - 1
  If Not PPFlashPhase(ppHandle, phaseIndex, phaseType) Then
   Call SetError("Failed to obtain phase type: " + PPGetLastError(ppHandle), "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
   Err.Raise ECapeComputationHR
  End If
  If (phaseType = Vapor) Then phases(phaseIndex) = "Gas" Else phases(phaseIndex) = "Liquid"
  phaseStatus(phaseIndex) = CAPE_ATEQUILIBRIUM
 Next phaseIndex
 On Error GoTo FailedSetPhases
 Call contextMO.SetPresentPhases(phases, phaseStatus)
 On Error GoTo 0
 'set T, P, composition and phase fraction on material object for all resulting phases
 Dim scalar
 ReDim scalar(0 To 0) As Double 'variant array of doubles, length 1
 For phaseIndex = 0 To phaseCount - 1
  Dim phaseFraction As Variant
  Dim phaseComposition As Variant
  If Not PPFlashPhaseResult(ppHandle, phaseIndex, phaseType, phaseFraction, phaseComposition) Then
   Call SetError("Failed to obtain phase result: " + PPGetLastError(ppHandle), "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
   Err.Raise ECapeComputationHR
  End If
  'phase name
  Dim phaseName As String
  If (phaseType = Vapor) Then phaseName = "Gas" Else phaseName = "Liquid"
  'set phase fraction
  On Error GoTo setPhaseFracError
  Call contextMO.SetSinglePhaseProp("phaseFraction", phaseName, "mole", phaseFraction)
  'set composition
  On Error GoTo setPhaseCompError
  Call contextMO.SetSinglePhaseProp("fraction", phaseName, "mole", phaseComposition)
  'set temperature
  scalar(0) = T
  On Error GoTo setPhaseTError
  Call contextMO.SetSinglePhaseProp("temperature", phaseName, vbNullString, scalar)
  'set pressure
  scalar(0) = P
  On Error GoTo setPhasePError
  Call contextMO.SetSinglePhaseProp("pressure", phaseName, vbNullString, scalar)
  On Error GoTo 0
 Next phaseIndex
 'set temperature and pressure, if not part of the flash specifications
 If (mustSetT) Then
  scalar(0) = T
  On Error GoTo setOverallTError
  Call contextMO.SetOverallProp("temperature", vbNullString, scalar)
  On Error GoTo 0
 End If
 If (mustSetP) Then
  scalar(0) = P
  On Error GoTo setOverallPError
  Call contextMO.SetOverallProp("pressure", vbNullString, scalar)
  On Error GoTo 0
 End If
 Exit Sub
 'error handlers:
TPXError:
 Call SetError("GetOverallTPFraction failed on context material: " + GetCOError(contextMO), "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
invalidCompositionError:
 Call SetError("Invalid or empty composition array from GetOverallTPFraction", "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
FailedSetPhases:
 Call SetError("Failed to set present phases on context material: " + GetCOError(contextMO), "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
setPhaseFracError:
 Call SetError("Failed to set " + phaseName + " fraction on context material: " + GetCOError(contextMO), "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
setPhaseCompError:
 Call SetError("Failed to set " + phaseName + " composition on context material: " + GetCOError(contextMO), "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
setPhaseTError:
 Call SetError("Failed to set " + phaseName + " temperature on context material: " + GetCOError(contextMO), "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
setPhasePError:
 Call SetError("Failed to set " + phaseName + " pressure on context material: " + GetCOError(contextMO), "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
setOverallTError:
 Call SetError("Failed to set overall temperature on context material: " + GetCOError(contextMO), "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
setOverallPError:
 Call SetError("Failed to set overall pressure on context material: " + GetCOError(contextMO), "ICapeThermoEquilibriumRoutine", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
End Sub

'ICapeThermoMaterialContext interface

Private Sub ICapeThermoMaterialContext_SetMaterial(ByVal material As Object)
 'set the context material
 On Error GoTo noInterface
 Set contextMO = material
 'get the compound from the material context
 ' We know that the compound list is constant in between two calls to SetMaterial. Hence, we can obtain the
 ' list of compounds now. Production implementations may want to postpone getting the compound list until it
 ' is actually required (as to avoid obtaining a compound list when it is not required) and merely invalidate
 ' the stored list of compounds at this point
 On Error GoTo noCompIface
 Dim iComps As ICapeThermoCompounds
 Set iComps = contextMO
 'check the compound IDs
 On Error GoTo compListError
 Dim compIDs As Variant, formulae As Variant, names As Variant, boilTemps As Variant, molwts As Variant, casnos As Variant
 Call iComps.GetCompoundList(compIDs, formulae, names, boilTemps, molwts, casnos)
 'get the compound indices from compIDs
 On Error GoTo invalidCompsError
 Call GetCompoundIndices(compIDs, "ICapeThermoMaterialContext", "SetMaterial", False, contextCompIndices)
 'all ok
 Exit Sub
 'error handlers
noInterface:
 'ICapeThermoMaterial interface not exposed
 Set contextMO = Nothing 'make sure that next call does not succeed with unexpected material
 Call SetError("Material object does not expose ICapeThermoMaterial", "ICapeThermoMaterialContext", "SetMaterial")
 Err.Raise ECapeInvalidArgumentHR
noCompIface:
 'ICapeThermoCompounds interface not exposed
 Set contextMO = Nothing 'make sure that next call does not succeed with unexpected material
 Call SetError("Material object does not expose ICapeThermoCompounds", "ICapeThermoMaterialContext", "SetMaterial")
 Err.Raise ECapeInvalidArgumentHR
compListError:
 'problem obtaining the compound list
 Set contextMO = Nothing 'make sure that next call does not succeed with unexpected material
 Call SetError("Failed to get list of present compounds from material: " + GetCOError(contextMO), "ICapeThermoMaterialContext", "SetMaterial")
 Err.Raise ECapeInvalidArgumentHR
invalidCompsError:
 'invalid list of compounds (SetError has already been called, just re-throw the error)
 Set contextMO = Nothing 'make sure that next call does not succeed with unexpected material
 Err.Raise ECapeInvalidArgumentHR
End Sub

Private Sub ICapeThermoMaterialContext_UnsetMaterial()
'release the context material
Set contextMO = Nothing
End Sub

'ICapeThermoPhases interface

Private Function ICapeThermoPhases_GetNumPhases() As Long
'we expose "Gas" and "Liquid"
ICapeThermoPhases_GetNumPhases = 2
End Function

Private Sub ICapeThermoPhases_GetPhaseList(phaseLabels As Variant, stateOfAggregation As Variant, keyCompoundId As Variant)
'whereas in version 1.0 the labels of phases indicate the state of aggregation, in version 1.1 these are
' separate concepts. We are therefore free to choose the labels (display names) of the phases. We choose "Gas"
' for the vapor phase, and "Liquid" for the liquid phase
ReDim phaseLabels(0 To 1) As String
ReDim stateOfAggregation(0 To 1) As String
ReDim keyCompoundId(0 To 1) As String
'vapor phase
phaseLabels(0) = "Gas"
stateOfAggregation(0) = "Vapor"
keyCompoundId(0) = vbNullString
'liquid phase
phaseLabels(1) = "Liquid"
stateOfAggregation(1) = "Liquid"
keyCompoundId(1) = vbNullString
End Sub

Private Function ICapeThermoPhases_GetPhaseInfo(ByVal phaseLabel As String, ByVal phaseAttribute As String) As Variant
'get attributes of a phase; the following are predefined attributes:
' StateOfAggregation, KeyCompoundId, ExcludedCompoundId, DensityDescription, UserDescription, TypeOfSolid
' for other attributes, we will simply return an empty string.
'we only support StateOfAggregation, UserDescription
Dim value As String
value = vbNullString 'set to something else in case defined
If (phaseLabel = "Gas") Then
 If (phaseAttribute = "StateOfAggregation") Then
  value = "Vapor"
 ElseIf (phaseAttribute = "UserDescription") Then
  value = "Ideal gas phase"
 End If
ElseIf (phaseLabel = "Liquid") Then
 If (phaseAttribute = "StateOfAggregation") Then
  value = "Liquid"
 ElseIf (phaseAttribute = "UserDescription") Then
  value = "Ideal liquid phase"
 End If
Else
 Call SetError("Unsupported/undefined phase """ + phaseLabel + """", "ICapeThermoPhases", "GetPhaseInfo")
 Err.Raise ECapeInvalidArgumentHR
End If
ICapeThermoPhases_GetPhaseInfo = value
End Function

'ICapeThermoPropertyRoutine interface

Private Sub ICapeThermoPropertyRoutine_CalcAndGetLnPhi(ByVal phaseLabel As String, ByVal Temperature As Double, ByVal Pressure As Double, ByVal moleNumbers As Variant, ByVal fFlags As Long, lnPhi As Variant, lnPhiDT As Variant, lnPhiDP As Variant, lnPhiDn As Variant)
 Call Init("ICapeThermoPropertyRoutine ", "CalcAndGetLnPhi")
 Dim i As Long, count As Long, index As Long
 Dim propIDs() As Long
 If (fFlags = 0) Then Exit Sub 'nothing to do
 'check how many props we calculate
 count = 0
 If ((fFlags And CAPE_LOG_FUGACITY_COEFFICIENTS) <> 0) Then count = count + 1
 If ((fFlags And CAPE_T_DERIVATIVE) <> 0) Then count = count + 1
 If ((fFlags And CAPE_P_DERIVATIVE) <> 0) Then count = count + 1
 If ((fFlags And CAPE_MOLE_NUMBERS_DERIVATIVES) <> 0) Then count = count + 1
 'make a list of properties to calculate
 ReDim propIDs(0 To count - 1)
 index = 0
 If ((fFlags And CAPE_LOG_FUGACITY_COEFFICIENTS) <> 0) Then
  propIDs(index) = LogFugacityCoefficient
  index = index + 1
 End If
 If ((fFlags And CAPE_T_DERIVATIVE) <> 0) Then
  propIDs(index) = LogFugacityCoefficientDT
  index = index + 1
 End If
 If ((fFlags And CAPE_P_DERIVATIVE) <> 0) Then
  propIDs(index) = LogFugacityCoefficientDP
  index = index + 1
 End If
 If ((fFlags And CAPE_MOLE_NUMBERS_DERIVATIVES) <> 0) Then
  propIDs(index) = LogFugacityCoefficientDn
  index = index + 1
 End If
 'check the context MO (as we use the context MO's compound indices)
 If contextMO Is Nothing Then
  requestedOperation = "SetMaterial"
  Call SetError("Material context not set", "ICapeThermoPropertyRoutine", "CalcAndGetLnPhi")
  Err.Raise ECapeBadInvOrder
 End If
 'check the composition
 ' Note, despite its name, the moleNumbers must be interpreted as composition; for
 ' lnPhi, lnPhiDT and lnPhiDP this does not matter, but for lnPhiDn this means that
 ' the results are to be returned for a total of 1 mole of substance.
 Dim X() As Double
 Dim nComp As Long
 nComp = UBound(contextCompIndices, 1) + 1
 'check the length
 Dim xcount As Long
 On Error GoTo invalidCompositionError
 xcount = UBound(moleNumbers, 1) + 1
 On Error GoTo 0
 If xcount <> nComp Then
  Call SetError("Invalid composition: unexpected number of values", "ICapeThermoPropertyRoutine", "CalcAndGetLnPhi")
  Err.Raise ECapeUnknownHR
 End If
 ReDim X(0 To nComp - 1)
 On Error GoTo invalidCompositionError
 For i = 0 To xcount - 1
  X(i) = moleNumbers(i)
 Next i
 On Error GoTo 0
 'check the phase
 Dim phase As Long
 If phaseLabel = "Gas" Then
  phase = Vapor
 ElseIf phaseLabel = "Liquid" Then
  phase = Liquid
 Else
  Call SetError("Invalid/undefined phase", "ICapeThermoPropertyRoutine", "CalcAndGetLnPhi")
  Err.Raise ECapeInvalidArgumentHR
 End If
 'perform the calculations
 If Not PPCalcSinglePhaseProps(ppHandle, nComp, contextCompIndices(0), phase, Temperature, Pressure, X(0), count, propIDs(0)) Then
  Call SetError("Failed to calculate properties: " + PPGetLastError(ppHandle), "ICapeThermoPropertyRoutine", "CalcAndGetLnPhi")
  Err.Raise ECapeComputationHR
 End If
 'obtain property results
 For i = 0 To count - 1
  Select Case propIDs(i)
   Case LogFugacityCoefficient
    lnPhi = PPGetPropertyResult(ppHandle, i)
    If IsEmpty(lnPhi) Then
     Call SetError("Failed to get results for lnPhi: " + PPGetLastError(ppHandle), "ICapeThermoPropertyRoutine", "CalcAndGetLnPhi")
     Err.Raise ECapeComputationHR
    End If
   Case LogFugacityCoefficientDT
    lnPhiDT = PPGetPropertyResult(ppHandle, i)
    If IsEmpty(lnPhiDT) Then
     Call SetError("Failed to get results for lnPhiDT: " + PPGetLastError(ppHandle), "ICapeThermoPropertyRoutine", "CalcAndGetLnPhi")
     Err.Raise ECapeComputationHR
    End If
   Case LogFugacityCoefficientDP
    lnPhiDP = PPGetPropertyResult(ppHandle, i)
    If IsEmpty(lnPhiDP) Then
     Call SetError("Failed to get results for lnPhiDP: " + PPGetLastError(ppHandle), "ICapeThermoPropertyRoutine", "CalcAndGetLnPhi")
     Err.Raise ECapeComputationHR
    End If
   Case LogFugacityCoefficientDn
    lnPhiDn = PPGetPropertyResult(ppHandle, i)
    If IsEmpty(lnPhiDn) Then
     Call SetError("Failed to get results for lnPhiDn: " + PPGetLastError(ppHandle), "ICapeThermoPropertyRoutine", "CalcAndGetLnPhi")
     Err.Raise ECapeComputationHR
    End If
   Case Else
    Call SetError("Internal error: unhandled property", "ICapeThermoPropertyRoutine", "CalcAndGetLnPhi")
    Err.Raise ECapeUnknownHR
  End Select
 Next i
 Exit Sub
 'error handlers:
invalidCompositionError:
 Call SetError("Invalid or empty composition array", "ICapeThermoPropertyRoutine", "CalcAndGetLnPhi")
 Err.Raise ECapeInvalidArgumentHR
End Sub

Private Sub ICapeThermoPropertyRoutine_CalcSinglePhaseProp(ByVal props As Variant, ByVal phaseLabel As String)
 Call Init("ICapeThermoPropertyRoutine ", "CalcSinglePhaseProp")
 Dim nComp As Long, i As Long, j As Long
 'check context material
 If contextMO Is Nothing Then
  requestedOperation = "SetMaterial"
  Call SetError("Material context not set", "ICapeThermoPropertyRoutine", "CalcSinglePhaseProp")
  Err.Raise ECapeBadInvOrder
 End If
 nComp = UBound(contextCompIndices, 1) + 1
 'check phase
 Dim phase As Long
 If (phaseLabel = "Gas") Then
  phase = Vapor
 ElseIf (phaseLabel = "Liquid") Then
  phase = Liquid
 Else
  Call SetError("Unknown/undefined phase """ + phaseLabel + """", "ICapeThermoPropertyRoutine", "CalcSinglePhaseProp")
  Err.Raise ECapeInvalidArgumentHR
 End If
 'get T, P fraction for phase
 Dim T As Double, P As Double, composition As Variant
 On Error GoTo FailedGetTPFraction
 Call contextMO.GetTPFraction(phaseLabel, T, P, composition)
 On Error GoTo 0
 'check composition
 Dim X() As Double
 Dim count As Long
 On Error GoTo invalidCompositionError
 count = UBound(composition, 1) + 1
 On Error GoTo 0
 If count <> nComp Then
  Call SetError("Invalid composition values from material object: unexpected number of values", "ICapeThermoPropertyRoutine", "CalcSinglePhaseProp")
  Err.Raise ECapeUnknownHR
 End If
 ReDim X(0 To nComp - 1)
 On Error GoTo invalidCompositionError
 For i = 0 To count - 1
  X(i) = composition(i)
 Next i
 On Error GoTo 0
 'check the list of properties
 Dim nProp As Long, propIDs() As Long
 Dim propName As String
 On Error GoTo invalidPropList
 nProp = UBound(props, 1) + 1
 On Error GoTo 0
 ReDim propIDs(0 To nProp - 1)
 'resolve the property names (production implementations should use a hash table)
 For i = 0 To nProp - 1
  On Error GoTo invalidProp
  propName = props(i) 'note that actions like this cause an implicit string copy in VB6; hard to avoid as we cannot reference strings
  On Error GoTo 0
  Dim found As Boolean
  found = False
  For j = 0 To SinglePhasePropertyCount - 1
   If propName = SinglePhasePropNames(j) Then
    propIDs(i) = j
    found = True
   End If
  Next j
  If Not found Then
   Call SetError("Invalid/unsupported single phase property """ + propName + """", "ICapeThermoPropertyRoutine", "CalcSinglePhaseProp")
   Err.Raise ECapeInvalidArgumentHR
  End If
 Next i
 'calc the properties
 If Not PPCalcSinglePhaseProps(ppHandle, nComp, contextCompIndices(0), phase, T, P, X(0), nProp, propIDs(0)) Then
  Call SetError("Failed to calculate properties: " + PPGetLastError(ppHandle), "ICapeThermoPropertyRoutine", "CalcSinglePhaseProp")
  Err.Raise ECapeComputationHR
 End If
 'set the properties on the context MO
 Dim values
 Dim basis As String
 For i = 0 To nProp - 1
  propName = props(i) 'note that actions like this cause an implicit string copy in VB6; hard to avoid as we cannot reference strings
  values = PPGetPropertyResult(ppHandle, i)
  If IsEmpty(values) Then
   Call SetError("Failed to get results for " + propName + ": " + PPGetLastError(ppHandle), "ICapeThermoPropertyRoutine", "CalcSinglePhaseProp")
   Err.Raise ECapeComputationHR
  End If
  If SinglePhasePropMoleBasis(propIDs(i)) Then
   basis = "mole"
  Else
   basis = vbNullString
  End If
  On Error GoTo failedSetProp
  Call contextMO.SetSinglePhaseProp(propName, phaseLabel, basis, values)
  On Error GoTo 0
 Next i
 Exit Sub
 'error handlers:
FailedGetTPFraction:
 Call SetError("GetTPFraction failed on context material: " + GetCOError(contextMO), "ICapeThermoPropertyRoutine", "CalcSinglePhaseProp")
 Err.Raise ECapeUnknownHR
invalidCompositionError:
 Call SetError("Invalid or empty composition array from context material", "ICapeThermoPropertyRoutine", "CalcSinglePhaseProp")
 Err.Raise ECapeUnknownHR
invalidPropList:
 Call SetError("Invalid or empty property list", "ICapeThermoPropertyRoutine", "CalcSinglePhaseProp")
 Err.Raise ECapeInvalidArgumentHR
invalidProp:
 Call SetError("Property list contains one or more invalid values (string array expected)", "ICapeThermoPropertyRoutine", "CalcSinglePhaseProp")
 Err.Raise ECapeInvalidArgumentHR
failedSetProp:
 Call SetError("Failed to set values for " + propName + ": " + GetCOError(contextMO), "ICapeThermoPropertyRoutine", "CalcSinglePhaseProp")
 Err.Raise ECapeUnknownHR
End Sub

Private Sub ICapeThermoPropertyRoutine_CalcTwoPhaseProp(ByVal props As Variant, ByVal phaseLabels As Variant)
 Call Init("ICapeThermoPropertyRoutine ", "CalcTwoPhaseProp")
 Dim nComp As Long, i As Long, j As Long
 'check context material
 If contextMO Is Nothing Then
  requestedOperation = "SetMaterial"
  Call SetError("Material context not set", "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
  Err.Raise ECapeBadInvOrder
 End If
 nComp = UBound(contextCompIndices, 1) + 1
 'check phases
 On Error GoTo invalidPhaseArray
 i = UBound(phaseLabels, 1)
 On Error GoTo 0
 If (i <> 1) Then
  Call SetError("Invalid phase array: expected 2 strings", "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
  Err.Raise ECapeInvalidArgumentHR
 End If
 Dim phase1 As Long, phase2 As Long
 Dim phaseLabel1 As String, phaseLabel2 As String
 On Error GoTo invalidPhaseArray
 phaseLabel1 = phaseLabels(0)
 On Error GoTo 0
 If (phaseLabel1 = "Gas") Then
  phase1 = Vapor
 ElseIf (phaseLabel1 = "Liquid") Then
  phase1 = Liquid
 Else
  Call SetError("Unknown/undefined phase """ + phaseLabel1 + """", "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
  Err.Raise ECapeInvalidArgumentHR
 End If
 On Error GoTo invalidPhaseArray
 phaseLabel2 = phaseLabels(1)
 On Error GoTo 0
 If (phaseLabel2 = "Gas") Then
  phase2 = Vapor
 ElseIf (phaseLabel2 = "Liquid") Then
  phase2 = Liquid
 Else
  Call SetError("Unknown/undefined phase """ + phaseLabel2 + """", "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
  Err.Raise ECapeInvalidArgumentHR
 End If
 If (phase1 = phase2) Then
  Call SetError("Phases in phase list cannot be the same", "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
  Err.Raise ECapeInvalidArgumentHR
 End If
 'get T, P fraction for phase 1
 Dim T1 As Double, P1 As Double, composition As Variant
 On Error GoTo FailedGetTPFraction
 Call contextMO.GetTPFraction(phaseLabel1, T1, P1, composition)
 On Error GoTo 0
 'check composition
 Dim X1() As Double
 Dim count As Long
 On Error GoTo invalidCompositionError
 count = UBound(composition, 1) + 1
 On Error GoTo 0
 If count <> nComp Then
  Call SetError("Invalid composition values from material object: unexpected number of values", "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
  Err.Raise ECapeUnknownHR
 End If
 ReDim X1(0 To nComp - 1)
 On Error GoTo invalidCompositionError
 For i = 0 To count - 1
  X1(i) = composition(i)
 Next i
 On Error GoTo 0
 'get T, P fraction for phase 1
 Dim T2 As Double, P2 As Double
 composition = Empty 'in case PME does not clear the array
 On Error GoTo FailedGetTPFraction
 Call contextMO.GetTPFraction(phaseLabel2, T2, P2, composition)
 On Error GoTo 0
 'check composition
 Dim X2() As Double
 On Error GoTo invalidCompositionError
 count = UBound(composition, 1) + 1
 On Error GoTo 0
 If count <> nComp Then
  Call SetError("Invalid composition values from material object: unexpected number of values", "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
  Err.Raise ECapeUnknownHR
 End If
 ReDim X2(0 To nComp - 1)
 On Error GoTo invalidCompositionError
 For i = 0 To count - 1
  X2(i) = composition(i)
 Next i
 On Error GoTo 0
 'check the list of properties
 Dim nProp As Long, propIDs() As Long
 Dim propName As String
 On Error GoTo invalidPropList
 nProp = UBound(props, 1) + 1
 On Error GoTo 0
 ReDim propIDs(0 To nProp - 1)
 'resolve the property names (production implementations should use a hash table)
 For i = 0 To nProp - 1
  On Error GoTo invalidProp
  propName = props(i) 'note that actions like this cause an implicit string copy in VB6; hard to avoid as we cannot reference strings
  On Error GoTo 0
  Dim found As Boolean
  found = False
  For j = 0 To TwoPhasePropertyCount - 1
   If propName = TwoPhasePropNames(j) Then
    propIDs(i) = j
    found = True
   End If
  Next j
  If Not found Then
   Call SetError("Invalid/unsupported two-phase property """ + propName + """", "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
   Err.Raise ECapeInvalidArgumentHR
  End If
 Next i
 'calc the properties
 If Not PPCalcTwoPhaseProps(ppHandle, nComp, contextCompIndices(0), phase1, phase2, T1, T2, P1, P2, X1(0), X2(0), nProp, propIDs(0)) Then
  Call SetError("Failed to calculate properties: " + PPGetLastError(ppHandle), "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
  Err.Raise ECapeComputationHR
 End If
 'set the properties on the context MO
 Dim vals
 Dim basis As String
 For i = 0 To nProp - 1
  propName = props(i) 'note that actions like this cause an implicit string copy in VB6; hard to avoid as we cannot reference strings
  vals = PPGetPropertyResult(ppHandle, i)
  If IsEmpty(vals) Then
   Call SetError("Failed to get results for " + propName + ": " + PPGetLastError(ppHandle), "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
   Err.Raise ECapeComputationHR
  End If
  On Error GoTo failedSetProp
  Call contextMO.SetTwoPhaseProp(propName, phaseLabels, vbNullString, vals)
  On Error GoTo 0
 Next i
 Exit Sub
 'error handlers:
invalidPhaseArray:
 Call SetError("Invalid phase array: expected 2 strings", "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
 Err.Raise ECapeInvalidArgumentHR
FailedGetTPFraction:
 Call SetError("GetTPFraction failed on context material: " + GetCOError(contextMO), "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
 Err.Raise ECapeUnknownHR
invalidCompositionError:
 Call SetError("Invalid or empty composition array from context material", "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
 Err.Raise ECapeUnknownHR
invalidPropList:
 Call SetError("Invalid or empty property list", "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
 Err.Raise ECapeInvalidArgumentHR
invalidProp:
 Call SetError("Property list contains one or more invalid values (string array expected)", "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
 Err.Raise ECapeInvalidArgumentHR
failedSetProp:
 Call SetError("Failed to set values for " + propName + ": " + GetCOError(contextMO), "ICapeThermoPropertyRoutine", "CalcTwoPhaseProp")
 Err.Raise ECapeUnknownHR
End Sub

Private Function ICapeThermoPropertyRoutine_CheckSinglePhasePropSpec(ByVal property As String, ByVal phaseLabel As String) As Boolean
 'we check if a property can be calculated. Such as check should not involve range checking on the calculation
 ' conditions, so we do not need the context material. We only check whether ther property is supported for the specified phase
 Dim res As Boolean, i As Long
 res = False 'set to true if supported
 'check the phase
 Dim isVapor As Boolean
 isVapor = (phaseLabel = "Gas")
 If (isVapor) Or (phaseLabel = "Liquid") Then
  'check property (production implementations should use a hash table)
  For i = 0 To SinglePhasePropertyCount - 1
   If (property = SinglePhasePropNames(i)) Then
    'property is supported....
    res = True
    If (isVapor) Then
     If (Left(property, 1) = "a") Then
      'this is activity or one of its derivatives, not supported for vapor phase
      res = False
     End If
    End If
    Exit For
   End If
  Next i
 End If
 ICapeThermoPropertyRoutine_CheckSinglePhasePropSpec = res
End Function

Private Function ICapeThermoPropertyRoutine_CheckTwoPhasePropSpec(ByVal property As String, ByVal phaseLabels As Variant) As Boolean
 'we check if a property can be calculated. Such as check should not involve range checking on the calculation
 ' conditions, so we do not need the context material. We only check whether ther property is supported for the specified phase
 Dim res As Boolean, i As Long
 res = False 'set to true if supported
 'we need vapor and liquid (if the phase array is incorrect, we just return not supported)
 On Error GoTo notSupported
 If UBound(phaseLabels, 1) = 1 Then
  Dim haveVap As Boolean, haveLiq As Boolean
  Dim phase As String
  haveVap = False
  haveLiq = False
  phase = phaseLabels(0)
  If (phase = "Gas") Then haveVap = True Else If (phase = "Liquid") Then haveLiq = True
  phase = phaseLabels(1)
  If (phase = "Gas") Then haveVap = True Else If (phase = "Liquid") Then haveLiq = True
  If (haveVap And haveLiq) Then
   'check property (production implementations should use a hash table)
   For i = 0 To TwoPhasePropertyCount - 1
    If (property = TwoPhasePropNames(i)) Then
     'property is supported
     res = True
     Exit For
    End If
   Next i
  End If
 End If
 ICapeThermoPropertyRoutine_CheckTwoPhasePropSpec = res
 Exit Function:
 'Error handlers:
notSupported:
 ICapeThermoPropertyRoutine_CheckTwoPhasePropSpec = False
End Function

Private Function ICapeThermoPropertyRoutine_GetSinglePhasePropList() As Variant
 'return a list of supported single phase properties; production implementations should cache and copy this list
 Dim res
 Dim i As Long
 ReDim res(0 To SinglePhasePropertyCount - 1) As String 'variant array of strings
 For i = 0 To SinglePhasePropertyCount - 1
  res(i) = SinglePhasePropNames(i)
 Next i
 ICapeThermoPropertyRoutine_GetSinglePhasePropList = res
End Function

Private Function ICapeThermoPropertyRoutine_GetTwoPhasePropList() As Variant
 'return a list of supported two-phase properties; production implementations should cache and copy this list
 Dim res
 Dim i As Long
 ReDim res(0 To TwoPhasePropertyCount - 1) As String 'variant array of strings
 For i = 0 To TwoPhasePropertyCount - 1
  res(i) = TwoPhasePropNames(i)
 Next i
 ICapeThermoPropertyRoutine_GetTwoPhasePropList = res
End Function

'ICapeThermoUniversalConstant interface

Private Function ICapeThermoUniversalConstant_GetUniversalConstant(ByVal constantId As String) As Variant
 'return values of universal constants as array of VARIANTs
 ' (the material object is not used)
 Dim res
 Dim val As Double
 'check property
 ' (in practise this routine is hardly ever called; a hash table may not be warrented here)
 If (constantId = "molarGasConstant") Then
  val = 8.314472 'we put this first in the list, as this is the value most likely to be queried
 ElseIf (constantId = "avogadroConstant") Then
  val = 6.02214199E+23
 ElseIf (constantId = "boltzmannConstant") Then
  val = 1.3806503E-23
 ElseIf (constantId = "speedOfLightInVacuum") Then
  val = 299792458#
 ElseIf (constantId = "standardAccelerationOfGravity") Then
  val = 9.80665
 Else
  Call SetError("Invalid or unsupported universal constant """ + constantId + """", "ICapeThermoUniversalConstant", "GetUniversalConstant")
  Err.Raise ECapeInvalidArgumentHR
 End If
 res = val 'assign double to Variant
 ICapeThermoUniversalConstant_GetUniversalConstant = res
End Function

Private Function ICapeThermoUniversalConstant_GetUniversalConstantList() As Variant
 'return a list of supported universal constants
 Dim res
 ReDim res(0 To 4) As String 'Variant array of strings
 res(0) = "molarGasConstant"
 res(1) = "avogadroConstant"
 res(2) = "boltzmannConstant"
 res(3) = "speedOfLightInVacuum"
 res(4) = "standardAccelerationOfGravity"
 ICapeThermoUniversalConstant_GetUniversalConstantList = res
End Function

'ICapeUtilities methods

Private Sub ICapeUtilities_Edit()
 Call Init("ICapeUtilities", "Edit")
 Call PPEdit(ppHandle) 'note that the list of compounds may have changed after this call
End Sub

Private Sub ICapeUtilities_Initialize()
 'does nothing; should be called by the simulation environment after InitNew or Load
End Sub

Private Property Get ICapeUtilities_parameters() As Object
 'we do not have parameters
 Call SetError("This object does not expose parameters", "ICapeUtilities", "get_parameters")
 Err.Raise ECapeNoImplHR
End Property

Private Property Let ICapeUtilities_simulationContext(ByVal RHS As Object)
 'set the simulation context
 ' it is not used by this unit operation, but it may be used to log messages in simulation environments that support this
 Set simulationContext = RHS
End Property

Private Sub ICapeUtilities_Terminate()
 On Error Resume Next
 terminateCalled = True 'no operations are valid after this
 Call ICapeThermoMaterialContext_UnsetMaterial 'release the context material, if any
 Set simulationContext = Nothing 'release simulation context, if any
 If (ppHandle <> 0) Then
  'release underlying property package
  Call PPDeletePropertyPackage(ppHandle)
  ppHandle = 0
 End If
 If (TempFileName <> "") Then
  'delete the temporary file
  Call Kill(TempFileName)
  TempFileName = ""
 End If
End Sub

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

Private Property Get ECapeBadInvOrder_requestedOperation() As String
 'return requested operation
 ECapeBadInvOrder_requestedOperation = requestedOperation
End Property



