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
' by the PME; instead, the PME should use the ThermoSystem class to
' create an instance of a PropertyPackage
'this class is marked persistable, so that it implements methods
' ReadProperties and WriteProperties (behind the screens this will
' implement IPersistStreamInit

Implements ICapeIdentification 'required for all CAPE-OPEN components
Implements ICapeUtilities      'required for all top-level CAPE-OPEN components
Implements ECapeRoot           'required in case of components that throw errors
Implements ECapeUnknown        'required in case of components that throw errors
Implements ECapeUser           'required in case of components that throw errors (all other error interfaces derive from this)
Implements ECapeNoImpl         'for ECapeNoImplHR errors
Implements ECapeIllegalAccess  'for ECapeIllegalAccessHR errors
Implements ECapeBadInvOrder    'for ECapeBadInvOrderHR errors
Implements ECapeComputation    'for ECapeComputationHR errors
Implements ICapeThermoPropertyPackage 'property package interface
Implements ECapePersistenceSystemError 'for ECapePersistenceSystemErrorHR errors

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

'get compound list from material object
' make sure that the caller is not catching errors while calling this routine
' make sure the package is initialized when calling this routine
' this is not the fastest way to go about it; production implementations should aim for a faster method (e.g. use a hash table)
' we do not perform checks that the underlying property package will perform, such as compounds being in the list multiple times
Private Sub GetCompoundsFromMaterial(materialObject As ICapeThermoMaterialObject, scope As String, ByRef compIndices() As Long)
 Dim compIds
 Dim count As Long
 Dim i As Long, j As Long
 On Error GoTo GetCompIDsError
 compIds = materialObject.ComponentIds
 On Error GoTo noComps 'calling Ubound on an empty array throws an error
 count = UBound(compIds, 1) + 1
 On Error GoTo 0
 Dim ppCompCount As Long
 If Not PPGetCompoundCount(ppHandle, ppCompCount) Then
  Call SetError("Failed to get compound count from property package: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", scope)
  Err.Raise ECapeUnknownHR
 End If
 ReDim compIndices(0 To count - 1)
 On Error GoTo invalidComponent 'in case we cannot assign to string
 For i = 0 To count - 1
  Dim compName As String
  compName = compIds(i)
  'check if we can locate this component
  Dim found As Boolean
  found = False
  For j = 0 To ppCompCount - 1
   Dim ppCompName
   ppCompName = PPGetCompoundStringConstant(ppHandle, j, CompoundName)
   If IsEmpty(ppCompName) Then
    'no luck
    On Error GoTo 0
    Call SetError("Failed to get compound name from property package: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", scope)
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
   Call SetError("Invalid compound ID """ + compName + """ from material object (no such compound)", "ICapeThermoPropertyPackage", scope)
   Err.Raise ECapeUnknownHR
  End If
 Next i
 On Error GoTo 0
 'all done
 Exit Sub
GetCompIDsError:
 're-throw error as CAPE-OPEN error
 Call SetError("get_ComponentIds failed: " + GetCOError(materialObject), "ICapeThermoPropertyPackage", scope)
 Err.Raise ECapeUnknownHR
noComps:
  Call SetError("Invalid or empty list of compounds from material object", "ICapeThermoPropertyPackage", scope)
  Err.Raise ECapeUnknownHR
invalidComponent:
  Call SetError("Invalid list of compounds from material object; expected a string array", "ICapeThermoPropertyPackage", scope)
  Err.Raise ECapeUnknownHR
End Sub

'get a property from a material object
' make sure that the caller does not catch errors
Private Function GetMaterialProp(materialObject As ICapeThermoMaterialObject, scope As String, propName As String, phaseID As String, calcType As String, basis As String, expectedCount As Long) As Double()
 Dim res() As Double
 'get the property
 Dim val As Variant
 On Error GoTo getPropError
 val = materialObject.GetProp(propName, phaseID, Empty, calcType, basis)
 'check the length
 Dim count As Long
 On Error GoTo invalidValueError
 count = UBound(val, 1) + 1
 If count <> expectedCount Then
  On Error GoTo 0
  Call SetError("Invalid values for " + phaseID + " " + propName + " from material object: unexpected number of values", "ICapeThermoPropertyPackage", scope)
  Err.Raise ECapeUnknownHR
 End If
 Dim i As Long
 ReDim res(0 To count - 1)
 For i = 0 To count - 1
  res(i) = val(i)
 Next i
 'all ok
 GetMaterialProp = res
 Exit Function
getPropError:
 Call SetError("Failed to get " + propName + " for " + phaseID + " from material object: " + GetCOError(materialObject), "ICapeThermoPropertyPackage", scope)
 Err.Raise ECapeUnknownHR
invalidValueError:
 Call SetError("Invalid values for " + phaseID + " " + propName + " from material object: " + Err.description, "ICapeThermoPropertyPackage", scope)
 Err.Raise ECapeUnknownHR
End Function

'ICapeThermoPropertyPackage methods

Private Sub ICapeThermoPropertyPackage_CalcEquilibrium(ByVal materialObject As Object, ByVal flashType As String, ByVal props As Variant)
 'This method performs an equilibrium calculation on all allowed phases and sets the results
 ' on the material object. The flash type indicates what kind of flash to calculate (e.g. "TP") and
 ' the material object contains the calculation conditions (overall composition, and for a "TP" flash
 ' temperature and pressure). For all equilibrium phases, composition and phase fraction are stored
 ' at the material object. In case pressure or temperature where not part of the equilibrium request
 ' (e.g. T in a "PH" flash), they are set on the material object as well.
 'It is not advised to pass any properties to be calculated in the props array. If however properties
 ' are to be calculated, they will be calculated for calcType "Mixture" for all resulting phases
 ' (which does not include "Overall" or "VaporLiquid")
 Call Init("ICapeThermoPropertyPackage", "CalcEquilibrium")
 'get the material object interface and list of compounds on the MO
 Dim compIndices() As Long
 Dim mat As ICapeThermoMaterialObject
 On Error GoTo invalidInterface
 Set mat = materialObject
 On Error GoTo 0
 Call GetCompoundsFromMaterial(mat, "CalcEquilibrium", compIndices)
 Dim nComp As Long
 nComp = UBound(compIndices, 1) + 1
 'check the flash type and get flash spec values from MO
 Dim spec1val As Double, spec2val As Double
 Dim flash As Long
 Dim mustSetT As Boolean, mustSetP As Boolean
 Dim vals() As Double
 mustSetT = True
 mustSetP = True
 Select Case flashType
  Case "TP", "PT"
   flash = TP
   mustSetT = False
   mustSetP = False
   vals = GetMaterialProp(mat, "CalcEquilibrium", "Temperature", "Overall", vbNullString, vbNullString, 1)
   spec1val = vals(0)
   vals = GetMaterialProp(mat, "CalcEquilibrium", "Pressure", "Overall", vbNullString, vbNullString, 1)
   spec2val = vals(0)
  Case "TVF"
   mustSetT = False
   flash = "Overall"
   vals = GetMaterialProp(mat, "CalcEquilibrium", "Temperature", "Overall", vbNullString, vbNullString, 1)
   spec1val = vals(0)
   vals = GetMaterialProp(mat, "CalcEquilibrium", "PhaseFraction", "Vapor", vbNullString, "mole", 1)
   spec2val = vals(0)
  Case "PVF"
   mustSetP = False
   flash = PVF
   vals = GetMaterialProp(mat, "CalcEquilibrium", "Pressure", "Overall", vbNullString, vbNullString, 1)
   spec1val = vals(0)
   vals = GetMaterialProp(mat, "CalcEquilibrium", "PhaseFraction", "Vapor", vbNullString, "mole", 1)
   spec2val = vals(0)
  Case "PH"
   mustSetP = False
   flash = PH
   vals = GetMaterialProp(mat, "CalcEquilibrium", "Pressure", "Overall", vbNullString, vbNullString, 1)
   spec1val = vals(0)
   vals = GetMaterialProp(mat, "CalcEquilibrium", "Enthalpy", "Overall", "Mixture", "mole", 1)
   spec2val = vals(0)
  Case "PS"
   mustSetP = False
   flash = PS
   vals = GetMaterialProp(mat, "CalcEquilibrium", "Pressure", "Overall", vbNullString, vbNullString, 1)
   spec1val = vals(0)
   vals = GetMaterialProp(mat, "CalcEquilibrium", "Entropy", "Overall", "Mixture", "mole", 1)
   spec2val = vals(0)
  Case Else
   Call SetError("Invalid or unsupported flash type: " + flashType, "ICapeThermoPropertyPackage", "CalcEquilibrium")
   Err.Raise ECapeInvalidArgumentHR
 End Select
 'get overall composition from material object
 Dim X() As Double
 X = GetMaterialProp(mat, "CalcEquilibrium", "fraction", "overall", vbNullString, "mole", nComp)
 'calc the equilibrium
 Dim phaseCount As Long
 Dim T As Double, P As Double
 If Not PPFlash(ppHandle, nComp, compIndices(0), X(0), flash, VaporLiquid, spec1val, spec2val, phaseCount, T, P) Then
  Call SetError("Flash failed: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", "CalcEquilibrium")
  Err.Raise ECapeComputationHR
 End If
 'set composition and phase fraction on material object for all resulting phases
 Dim phaseIndex As Long
 For phaseIndex = 0 To phaseCount - 1
  Dim phaseFraction As Variant
  Dim phaseComposition As Variant
  Dim phaseType As Long
  If Not PPFlashPhaseResult(ppHandle, phaseIndex, phaseType, phaseFraction, phaseComposition) Then
   Call SetError("Failed to obtain phase result: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", "CalcEquilibrium")
   Err.Raise ECapeComputationHR
  End If
  'phase name
  Dim phaseName As String
  If (phaseType = Vapor) Then phaseName = "Vapor" Else phaseName = "Liquid"
  'set phase fraction
  On Error GoTo setPhaseFracError
  Call mat.SetProp("phaseFraction", phaseName, Empty, vbNullString, "mole", phaseFraction)
  'set composition
  On Error GoTo setPhaseCompError
  Call mat.SetProp("fraction", phaseName, Empty, vbNullString, "mole", phaseComposition)
  On Error GoTo 0
  'check array of properties
  If Not IsEmpty(props) Then
   Dim i As Long, j As Long
   'make a list of single phase properties (production implementations should use a hash table and construct this list outside of the loop)
   Dim nProp As Long
   Dim propName As String
   On Error GoTo invalidPropList
   nProp = UBound(props, 1) + 1
   Dim propIDs() As Long
   ReDim propIDs(0 To nProp - 1)
   For i = 0 To nProp - 1
    Dim found As Boolean
    found = False
    On Error GoTo invalidPropName
    propName = props(i)
    On Error GoTo 0
    For j = 0 To SinglePhasePropertyCount
     If (SinglePhasePropNames(j) = propName) Then
      found = True
      propIDs(i) = j
      Exit For
     End If
    Next j
    If Not found Then
     Call SetError("Invalid/unsupported single phase mixture property """ + propName + """ in property list", "ICapeThermoPropertyPackage", "CalcEquilibrium")
     Err.Raise ECapeInvalidArgumentHR
    End If
   Next i
   'convert phase composition
   ReDim X(0 To nComp - 1) 'we need X as a double array rather than as Variant
   For i = 0 To nComp - 1
    X(i) = phaseComposition(i)
   Next i
   'calc properties
   If Not PPCalcSinglePhaseProps(ppHandle, nComp, compIndices(0), phaseType, T, P, X(0), nProp, propIDs(0)) Then
    Call SetError("Failed to calculate properties for equilibrium phase: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", "CalcEquilibrium")
    Err.Raise ECapeComputationHR
   End If
   'set property results for phase
   For i = 0 To nProp - 1
    On Error GoTo invalidPropName
    propName = props(i) 'should not fail (we tried above, production implementations should avoid this extra string allocation)
    Dim v
    If Not PPGetPropertyResult(ppHandle, i) Then
     Call SetError("Failed to get property results for equilibrium phase: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", "CalcEquilibrium")
     Err.Raise ECapeComputationHR
    End If
    Dim basis As String
    If (SinglePhasePropMoleBasis(propIDs(i))) Then
     basis = "mole"
    Else
     basis = vbNullString
    End If
    On Error GoTo SetPropError
    Call mat.SetProp(propName, phaseName, Empty, "mixture", basis, v)
    On Error GoTo 0
   Next i
  End If
 Next phaseIndex
 'set temperature and pressure, if not part of the flash specifications
 If (mustSetT) Then
  Dim Tvalue As Variant
  ReDim Tvalue(0 To 0) As Double
  Tvalue(0) = T
  On Error GoTo setTemperatureError
  Call mat.SetProp("temperature", "overall", Empty, vbNullString, vbNullString, Tvalue)
  On Error GoTo 0
 End If
 If (mustSetP) Then
  Dim Pvalue As Variant
  ReDim Pvalue(0 To 0) As Double
  Pvalue(0) = P
  On Error GoTo setPressureError
  Call mat.SetProp("pressure", "overall", Empty, vbNullString, vbNullString, Pvalue)
  On Error GoTo 0
 End If
 Exit Sub
 'error handlers:
invalidInterface:
 Call SetError("Failed to get ICapeThermoMaterialObject interface from material object", "ICapeThermoPropertyPackage", "CalcEquilibrium")
 Err.Raise ECapeInvalidArgumentHR
setPhaseFracError:
 Call SetError("Failed to set " + phaseName + " fraction on material object: " + GetCOError(mat), "ICapeThermoPropertyPackage", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
setPhaseCompError:
 Call SetError("Failed to set " + phaseName + " composition on material object: " + GetCOError(mat), "ICapeThermoPropertyPackage", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
setTemperatureError:
 Call SetError("Failed to set temperature on material object: " + GetCOError(mat), "ICapeThermoPropertyPackage", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
setPressureError:
 Call SetError("Failed to set pressure on material object: " + GetCOError(mat), "ICapeThermoPropertyPackage", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
SetPropError:
 Call SetError("Failed to set """ + propName + """ on material object: " + GetCOError(mat), "ICapeThermoPropertyPackage", "CalcEquilibrium")
 Err.Raise ECapeUnknownHR
invalidPropList:
 Call SetError("Invalid list of properties", "ICapeThermoPropertyPackage", "CalcEquilibrium")
 Err.Raise ECapeInvalidArgumentHR
invalidPropName:
 Call SetError("Invalid property name in list of properties", "ICapeThermoPropertyPackage", "CalcEquilibrium")
 Err.Raise ECapeInvalidArgumentHR
End Sub

Private Sub ICapeThermoPropertyPackage_CalcProp(ByVal materialObject As Object, ByVal props As Variant, ByVal phases As Variant, ByVal calcType As String)
 Call Init("ICapeThermoPropertyPackage", "CalcProp")
 'The version 1.0 CAPE-OPEN CalcProp is rather generic. It allows for calculation of temperature
 ' dependent properties, single-phase mixture properties, two-phase properties, overall properties
 ' all at once. Hence, we have quite some checks to perform here. This is better arranged in version 1.1
 'We do allow for overall property calculations, but only for volume, enthalpy and entropy, and
 ' none of their derivatives. For all mixture properties, we only allow for Mixture calculation type,
 ' whereas for all temperature dependent properties, we only allow for Pure calculation type. For
 ' temperature dependent properties, the phase does not matter, except for that we use the same phase
 ' for storing the results at the material object. For two-phase properties, the only allowed phase
 ' is VaporLiquid.
 'We must also not set any properties on the Material Object until all property calculations have
 ' succeeded. This is a demand that is not actually enforced by many implementations, but we will
 ' do so in this implementation. To do so, we buffer all results and only set them at the point all
 ' calculations are completed (buffering is at the expense of allocating structures to do so)
 'This routine is intended for demonstrative purposes. Pruction implementations should aim for a
 ' more efficient routine (e.g. pre-check the list of properties before looping over the phases,
 ' use hash tables for property lookups, avoid caching of results and corresponding memory allocations,
 ' ...)
 Dim cachedProperties() As CachedPropRes
 Dim cachedPropertyCount As Long
 Dim propRes As CachedPropRes
 Dim error As String
 Dim isPure As Boolean
 Dim v As Variant
 Dim propVals() As Double
 Dim iProp As Long, iPhase As Long, propID As Long, i As Long, j As Long, nProp As Long, nPhase As Long, phaseID As Long
 Dim T As Double, P As Double
 Dim X() As Double, X2() As Double
 Dim propIDs() As Long
 Dim compIndices() As Long
 Dim nComp As Long
 Dim propName As String, phaseName As String
 Dim mat As ICapeThermoMaterialObject
 Dim found As Boolean
 cachedPropertyCount = 0
 'get MO
 On Error GoTo MaterialInterfaceError
 Set mat = materialObject
 On Error GoTo 0
 'get compounds
 Call GetCompoundsFromMaterial(mat, "CalcProp", compIndices)
 nComp = UBound(compIndices, 1) + 1 'gives an error on empty compound list
 'check calc type
 If (calcType = "Mixture") Then
  isPure = False
 ElseIf (calcType = "Pure") Then
  isPure = True
 Else
  Call SetError("Invalid calcType """ + calcType + """", "ICapeThermoPropertyPackage", "CalcProp")
  Err.Raise ECapeInvalidArgumentHR
 End If
 'check prop list
 On Error GoTo EmptyPropList
 nProp = UBound(props, 1) + 1
 'check phase list
 On Error GoTo EmptyPhaseList
 nPhase = UBound(phases, 1) + 1
 On Error GoTo 0
 If (isPure) Then
  'Pure calculations, we only allow temperature dependent properties and do not care about the phase
  'get temperature
  On Error GoTo FailedGetTemperature
  propVals = GetMaterialProp(mat, "CalcProp", "Temperature", "overall", vbNullString, vbNullString, 1)
  T = propVals(0)
  On Error GoTo 0
  'loop over properties
  For iProp = 0 To nProp - 1
   On Error GoTo invalidPropInList
   propName = props(iProp)
   On Error GoTo 0
   found = False
   For propID = 0 To TDepPropertyCount - 1 'production implementations should use a hash table or equivalent
    If (TDepPropNames(propID) = propName) Then
     found = True
     'calc temperature dependent property for all compounds
     ReDim v(0 To nComp - 1) As Double 'store the result here
     For i = 0 To nComp - 1
      Dim d As Double
      If Not PPGetTemperatureDependentProperty(ppHandle, compIndices(i), propID, T, d) Then
       Dim e As String
       e = PPGetLastError(ppHandle)
       Call SetError("Failed to calculate " + propName + " for " + PPGetCompoundStringConstant(ppHandle, compIndices(i), CompoundName) + ": " + e, "ICapePropertyPackage", "CalcProp")
       Err.Raise ECapeComputationHR
      End If
      v(i) = d
     Next i
     'store this result for all requested phases
     For iPhase = 0 To nPhase - 1
      Set propRes = New CachedPropRes
      propRes.propName = propName
      On Error GoTo invalidPhaseInList
      propRes.phase = phases(iPhase)
      On Error GoTo 0
      If (TDepPropMoleBasis(propID)) Then
       propRes.basis = "mole"
      Else
       propRes.basis = vbNullString
      End If
      propRes.values = v
      'add to array of prop calculation results to set
      ReDim Preserve cachedProperties(0 To cachedPropertyCount)
      Set cachedProperties(cachedPropertyCount) = propRes
      cachedPropertyCount = cachedPropertyCount + 1
     Next iPhase
     Exit For
    End If
   Next propID
   If Not found Then
    Call SetError("Invalid or unsupported property """ + propName + """ (calcType Pure)", "ICapePropertyPackage", "CalcProp")
    Err.Raise ECapeInvalidArgumentHR
   End If
  Next iProp
 Else
  'mixture calculations, loop over the phases
  For iPhase = 0 To nPhase - 1
   On Error GoTo invalidPhaseInList
   phaseName = phases(iPhase)
   On Error GoTo 0
   'check what kind of calculation
   If (phaseName = "Vapor") Or (phaseName = "Liquid") Then
    'single phase property calculations
    If (phaseName = "Vapor") Then phaseID = Vapor Else phaseID = Liquid
    'get temperature
    On Error GoTo FailedGetTemperature
    propVals = GetMaterialProp(mat, "CalcProp", "Temperature", "overall", vbNullString, vbNullString, 1)
    T = propVals(0)
    'get pressure
    On Error GoTo FailedGetPressure
    propVals = GetMaterialProp(mat, "CalcProp", "Pressure", "overall", vbNullString, vbNullString, 1)
    P = propVals(0)
    'get composition
    On Error GoTo FailedGetComposition
    X = GetMaterialProp(mat, "CalcProp", "fraction", phaseName, vbNullString, "mole", nComp)
    On Error GoTo 0
    'make a list of properties to calculate:
    ReDim propIDs(0 To nProp - 1)
    For i = 0 To nProp - 1
     On Error GoTo invalidPropInList
     propName = props(iProp)
     On Error GoTo 0
     found = False
     For j = 0 To SinglePhasePropertyCount - 1 'production implementations should use a hash table or equivalent (and lookup the properties outside of the phase loop)
      If (SinglePhasePropNames(j) = propName) Then
       found = True
       propIDs(i) = j
       Exit For
      End If
     Next j
     If Not found Then
      Call SetError("Invalid/unsupported single phase property """ + propName + """", "ICapeThermoPropertyPackage", "CalcProp")
      Err.Raise ECapeInvalidArgumentHR
     End If
    Next i
    'calculate all properties simultaneously
    If Not PPCalcSinglePhaseProps(ppHandle, nComp, compIndices(0), phaseID, T, P, X(0), nProp, propIDs(0)) Then
     Call SetError("Property calculations failed: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", "CalcProp")
     Err.Raise ECapeComputationHR
    End If
    'get the results, and store for setting on the material
    For i = 0 To nProp - 1
     On Error GoTo invalidPropInList 'should not fail, we tried this above (production implementations should avoid this extra string allocation)
     propName = props(iProp)
     On Error GoTo 0
     v = PPGetPropertyResult(ppHandle, i)
     If IsEmpty(v) Then
      Call SetError("Failed to obtain property result: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", "CalcProp")
      Err.Raise ECapeComputationHR
     End If
     'cache for setting later on
     Set propRes = New CachedPropRes
     propRes.propName = propName
     propRes.phase = phaseName
     If (SinglePhasePropMoleBasis(propIDs(i))) Then
      propRes.basis = "mole"
     Else
      propRes.basis = vbNullString
     End If
     propRes.values = v
     'add to array of prop calculation results to set
     ReDim Preserve cachedProperties(0 To cachedPropertyCount)
     Set cachedProperties(cachedPropertyCount) = propRes
     cachedPropertyCount = cachedPropertyCount + 1
    Next i
   ElseIf (phaseName = "Overall") Then
    'overall property calculations
    ' we only support enthalpy, entropy and volume
    'make a list of properties to calculate:
    ReDim propIDs(0 To nProp - 1)
    For i = 0 To nProp - 1
     On Error GoTo invalidPropInList
     propName = props(iProp)
     On Error GoTo 0
     If (propName = "Enthalpy") Then 'production implementations should use a hash table or equivalent (and lookup the properties outside of the phase loop)
      propIDs(i) = Enthalpy
     ElseIf (propName = "Entropy") Then
      propIDs(i) = Entropy
     ElseIf (propName = "Volume") Then
      propIDs(i) = Volume
     Else
      'not supported
      Call SetError("Invalid/unsupported overall property """ + propName + """", "ICapeThermoPropertyPackage", "CalcProp")
      Err.Raise ECapeInvalidArgumentHR
     End If
    Next i
    'get temperature
    On Error GoTo FailedGetTemperature
    propVals = GetMaterialProp(mat, "CalcProp", "Temperature", "overall", vbNullString, vbNullString, 1)
    T = propVals(0)
    'get pressure
    On Error GoTo FailedGetPressure
    propVals = GetMaterialProp(mat, "CalcProp", "Pressure", "overall", vbNullString, vbNullString, 1)
    P = propVals(0)
    'get a list of phases on the MO
    On Error GoTo FailedGetPresentPhases
    v = mat.PhaseIds
    On Error GoTo 0
    Dim nPresentPhases As Long
    On Error GoTo EmptyPresentPhaseList
    nPresentPhases = UBound(v, 1) + 1
    On Error GoTo 0
    'we store the contributions for each phase here:
    Dim propValResults() As Double
    ReDim propValResults(0 To nProp - 1)
    For i = 0 To nProp - 1
     propValResults(i) = 0
    Next i
    'loop over the present phases
    For i = 0 To nPresentPhases - 1
     'get phase ID
     On Error GoTo InvalidPresentPhase
     phaseName = v(i)
     On Error GoTo 0
     Dim presentPhase As Long
     If phaseName = "Vapor" Then
      presentPhase = Vapor
     ElseIf phaseName = "Liquid" Then
      presentPhase = Liquid
     Else
      Call SetError("Material object reports invalid or undefined present phase """ + phaseName + """", "ICapeThermoPropertyPackage", "CalcProp")
      Err.Raise ECapeComputationHR
     End If
     'get phase fraction
     Dim phaseFaction As Double
     On Error GoTo FailedGetPhaseFraction
     propVals = GetMaterialProp(mat, "CalcProp", "phaseFraction", phaseName, vbNullString, "mole", 1)
     phaseFaction = propVals(0)
     If (phaseFaction > 0) Then 'else no contribution to overall property
      'get composition
      On Error GoTo FailedGetComposition
      X = GetMaterialProp(mat, "CalcProp", "fraction", phaseName, vbNullString, "mole", nComp)
      On Error GoTo 0
      'calc properties for this phase, simultaneously
      If Not PPCalcSinglePhaseProps(ppHandle, nComp, compIndices(0), phaseID, T, P, X(0), nProp, propIDs(0)) Then
       Call SetError("Property calculations failed: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", "CalcProp")
       Err.Raise ECapeComputationHR
      End If
      'get the results, and store for setting on the material
      For j = 0 To nProp - 1
       Dim propResult '(as Variant...)
       propResult = PPGetPropertyResult(ppHandle, j)
       If IsEmpty(propResult) Then
        Call SetError("Failed to obtain property result: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", "CalcProp")
        Err.Raise ECapeComputationHR
       End If
       propValResults(j) = propValResults(j) + phaseFaction * propResult(0)
      Next j
     End If
    Next i
    For iProp = 0 To nProp - 1
     'store property value
     On Error GoTo invalidPropInList
     propName = props(iProp) 'should not fail, we checked above (production implementations should avoid this extra string allocation)
     On Error GoTo 0
     Set propRes = New CachedPropRes
     propRes.propName = propName
     propRes.phase = "overall"
     propRes.basis = "mole"
     Call propRes.SetScalar(propValResults(iProp))
     'add to array of prop calculation results to set
     ReDim Preserve cachedProperties(0 To cachedPropertyCount)
     Set cachedProperties(cachedPropertyCount) = propRes
     cachedPropertyCount = cachedPropertyCount + 1
    Next iProp
   ElseIf (phaseName = "VaporLiquid") Then
    'two phase property calculations
    'get temperature
    On Error GoTo FailedGetTemperature
    propVals = GetMaterialProp(mat, "CalcProp", "Temperature", "overall", vbNullString, vbNullString, 1)
    T = propVals(0)
    'get pressure
    On Error GoTo FailedGetPressure
    propVals = GetMaterialProp(mat, "CalcProp", "Pressure", "overall", vbNullString, vbNullString, 1)
    P = propVals(0)
    'make a list of properties to calculate
    ReDim propIDs(0 To nProp - 1)
    For i = 0 To nProp - 1
     On Error GoTo invalidPropInList
     propName = props(iProp)
     On Error GoTo 0
     found = False
     For j = 0 To TwoPhasePropertyCount - 1 'production implementations should use a hash table or equivalent (and lookup the properties outside of the phase loop)
      If (TwoPhasePropNames(j) = propName) Then
       found = True
       propIDs(i) = j
       Exit For
      End If
     Next j
     If Not found Then
      Call SetError("Invalid/unsupported two-phase property """ + propName + """", "ICapeThermoPropertyPackage", "CalcProp")
      Err.Raise ECapeInvalidArgumentHR
     End If
    Next i
    'get vapor composition
    On Error GoTo FailedGetComposition
    phaseName = "Vapor" 'used in error handler
    X = GetMaterialProp(mat, "CalcProp", "fraction", "Vapor", vbNullString, "mole", nComp)
    phaseName = "Liquid" 'used in error handler
    X2 = GetMaterialProp(mat, "CalcProp", "fraction", "Vapor", vbNullString, "mole", nComp)
    On Error GoTo 0
    'calculate all properties simultaneously
    If Not PPCalcTwoPhaseProps(ppHandle, nComp, compIndices(0), Vapor, Liquid, T, T, P, P, X(0), X2(0), nProp, propIDs(0)) Then
     Call SetError("Property calculations failed: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", "CalcProp")
     Err.Raise ECapeComputationHR
    End If
    'get the results, and store for setting on the material
    For i = 0 To nProp - 1
     On Error GoTo invalidPropInList 'should not fail, we tried this above (production implementations should avoid this extra string allocation)
     propName = props(iProp)
     On Error GoTo 0
     v = PPGetPropertyResult(ppHandle, i)
     If IsEmpty(v) Then
      Call SetError("Failed to obtain property result: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", "CalcProp")
      Err.Raise ECapeComputationHR
     End If
     'cache for setting later on
     Set propRes = New CachedPropRes
     propRes.propName = propName
     propRes.phase = "VaporLiquid"
     propRes.basis = vbNullString
     propRes.values = v
     'add to array of prop calculation results to set
     ReDim Preserve cachedProperties(0 To cachedPropertyCount)
     Set cachedProperties(cachedPropertyCount) = propRes
     cachedPropertyCount = cachedPropertyCount + 1
    Next i
   Else
    'unsupported phase
    Call SetError("Invalid or unsupported phase """ + phaseName + """", "ICapePropertyPackage", "CalcProp")
    Err.Raise ECapeUnknownHR
   End If
  Next iPhase
 End If
 'all calculations succeeded, set the results at the material object
 On Error GoTo SetPropError
 For i = 0 To cachedPropertyCount - 1
  Set propRes = cachedProperties(i)
  Call mat.SetProp(propRes.propName, propRes.phase, Empty, calcType, propRes.basis, propRes.values)
 Next i
 Exit Sub
 'error handlers:
MaterialInterfaceError:
 Call SetError("Failed to get ICapeThermoMaterialObject interface from material", "ICapePropertyPackage", "CalcProp")
 Err.Raise ECapeInvalidArgumentHR
EmptyPropList:
 Call SetError("Invalid or empty list of properties", "ICapePropertyPackage", "CalcProp")
 Err.Raise ECapeInvalidArgumentHR
EmptyPhaseList:
 Call SetError("Invalid or empty list of phases", "ICapePropertyPackage", "CalcProp")
 Err.Raise ECapeInvalidArgumentHR
SetPropError:
 Call SetError("Failed to set " + propRes.phase + " " + propRes.propName + ": " + GetCOError(mat), "ICapePropertyPackage", "CalcProp")
 Err.Raise ECapeUnknownHR
invalidPropInList:
 Call SetError("Invalid property in property list", "ICapePropertyPackage", "CalcProp")
 Err.Raise ECapeInvalidArgumentHR
invalidPhaseInList:
 Call SetError("Invalid phase in phase list", "ICapePropertyPackage", "CalcProp")
 Err.Raise ECapeInvalidArgumentHR
FailedGetTemperature:
 Call SetError("Failed to get temperature from material object: " + GetCOError(mat), "ICapePropertyPackage", "CalcProp")
 Err.Raise ECapeUnknownHR
FailedGetPressure:
 Call SetError("Failed to get pressure from material object: " + GetCOError(mat), "ICapePropertyPackage", "CalcProp")
 Err.Raise ECapeUnknownHR
FailedGetComposition:
 Call SetError("Failed to get " + phaseName + " composition from material object: " + GetCOError(mat), "ICapePropertyPackage", "CalcProp")
 Err.Raise ECapeUnknownHR
FailedGetPhaseFraction:
 Call SetError("Failed to get " + phaseName + " fraction from material object: " + GetCOError(mat), "ICapePropertyPackage", "CalcProp")
 Err.Raise ECapeUnknownHR
FailedGetPresentPhases:
 Call SetError("Failed to get list of present phases from material object: " + GetCOError(mat), "ICapePropertyPackage", "CalcProp")
 Err.Raise ECapeUnknownHR
EmptyPresentPhaseList:
 Call SetError("List of present phases on material object is empty", "ICapePropertyPackage", "CalcProp")
 Err.Raise ECapeUnknownHR
InvalidPresentPhase:
 Call SetError("Invalid entry in list of present phases from material object", "ICapePropertyPackage", "CalcProp")
 Err.Raise ECapeUnknownHR
End Sub

Private Function ICapeThermoPropertyPackage_GetComponentConstant(ByVal materialObject As Object, ByVal props As Variant) As Variant
 'gets string or real components for all compounds on material object. Results is returned as an array
 ' of Variants (containing string or double values). For version 1.0, this function fails in case any
 ' of the compound constants cannot be obtained
 Call Init("ICapeThermoPropertyPackage", "GetComponentConstant")
 Dim nProp As Long, iProp As Long, nComp As Long, iComp As Long, compIndex As Long
 Dim realVal As Double
 Dim propName As String
 Dim compIndices() As Long
 Dim mat As ICapeThermoMaterialObject
 'get MO
 On Error GoTo MaterialInterfaceError
 Set mat = materialObject
 On Error GoTo 0
 'get compounds
 Call GetCompoundsFromMaterial(mat, "CalcProp", compIndices)
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
     Call SetError("Failed to get " + propName + " for " + PPGetCompoundStringConstant(ppHandle, compIndex, CompoundName) + ": " + e, "ICapeThermoPropertyPackage", "GetComponentConstant")
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
    Call SetError("Invalid/unsupported compound constant """ + propName + """", "ICapeThermoPropertyPackage", "GetComponentConstant")
    Err.Raise ECapeInvalidArgumentHR
   End If
   index = index + 1
  Next iComp
 Next iProp
 ICapeThermoPropertyPackage_GetComponentConstant = res
 Exit Function
 'error handlers:
MaterialInterfaceError:
 Call SetError("Failed to get ICapeThermoMaterialObject interface from material", "ICapePropertyPackage", "GetComponentConstant")
 Err.Raise ECapeInvalidArgumentHR
EmptyPropList:
 Call SetError("Invalid or empty list of properties", "ICapePropertyPackage", "GetComponentConstant")
 Err.Raise ECapeInvalidArgumentHR
invalidPropInList:
 Call SetError("Invalid entry in list of properties", "ICapePropertyPackage", "GetComponentConstant")
 Err.Raise ECapeInvalidArgumentHR
End Function

Private Sub ICapeThermoPropertyPackage_GetComponentList(compIds As Variant, formulae As Variant, names As Variant, boilTemps As Variant, molwt As Variant, casno As Variant)
 'get the list of components supported by this property package. This package returns
 ' identical values for names and compIds
 Call Init("ICapeThermoPropertyPackage", "GetComponentList")
 Dim count As Long, i As Long
 Dim d As Double
 Dim v As Variant
 If Not PPGetCompoundCount(ppHandle, count) Then
  Call SetError("Failed to get compound count: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", "GetComponentList")
  Err.Raise ECapeComputationHR
 End If
 ReDim compIds(0 To count - 1) As String
 ReDim formulae(0 To count - 1) As String
 ReDim names(0 To count - 1) As String
 ReDim boilTemps(0 To count - 1) As Double
 ReDim molwt(0 To count - 1) As Double
 ReDim casno(0 To count - 1) As String
 For i = 0 To count - 1
  v = PPGetCompoundStringConstant(ppHandle, i, CompoundName)
  If IsEmpty(v) Then
   Call SetError("Failed to get name of compound: " + PPGetLastError(ppHandle), "ICapeThermoPropertyPackage", "GetComponentList")
   Err.Raise ECapeComputationHR
  End If
  compIds(i) = v
  names(i) = v
  'remainder is not critical, no error in case of fail
  v = PPGetCompoundStringConstant(ppHandle, i, ChemicalFormula)
  If (IsEmpty(v)) Then formulae(i) = vbNullString Else formulae(i) = v
  v = PPGetCompoundStringConstant(ppHandle, i, CASNumber)
  If (IsEmpty(v)) Then casno(i) = vbNullString Else casno(i) = v
  'Missing real values should be set to NaN
  ' (however, the underlying property package will not fail)
  If Not PPGetCompoundRealConstant(ppHandle, i, NormalBoilingPoint, d) Then d = NaN
  boilTemps(i) = d
  If Not PPGetCompoundRealConstant(ppHandle, i, MolecularWeight, d) Then d = NaN
  molwt(i) = d
 Next i
End Sub

Private Function ICapeThermoPropertyPackage_GetPhaseList() As Variant
 'returns the list of defined phases; vapor phases must start with "Vapor", liquid phases
 ' must start with "Liquid", solid phases must start with "Solid". We only support
 ' Vapor and Liquid, but as we allow for VaporLiquid and Overall property calculations,
 ' we add those to the list as well
 'production implementations should cache and copy this list
 Dim res As Variant
 ReDim res(0 To 3) As String
 res(0) = "Overall"
 res(1) = "Vapor"
 res(2) = "Liquid"
 res(3) = "VaporLiquid"
 ICapeThermoPropertyPackage_GetPhaseList = res
End Function

Private Function ICapeThermoPropertyPackage_GetPropList() As Variant
 'return a list of all properties that can be calculated
 'production implementations should cache and copy this list
 Dim res
 Dim i As Long, index As Long
 ReDim res(0 To TDepPropertyCount + SinglePhasePropertyCount + TwoPhasePropertyCount - 1) As String
 index = 0 'index into res
 For i = 0 To TDepPropertyCount - 1
  res(index) = TDepPropNames(i)
  index = index + 1
 Next i
 For i = 0 To SinglePhasePropertyCount - 1
  res(index) = SinglePhasePropNames(i)
  index = index + 1
 Next i
 For i = 0 To TwoPhasePropertyCount - 1
  res(index) = TwoPhasePropNames(i)
  index = index + 1
 Next i
 ICapeThermoPropertyPackage_GetPropList = res
End Function

Private Function ICapeThermoPropertyPackage_GetUniversalConstant(ByVal materialObject As Object, ByVal props As Variant) As Variant
 'return values of universal constants as array of VARIANTs
 ' (the material object is not used)
 Dim nProp As Long, i As Long
 Dim propName As String
 Dim val As Double
 On Error GoTo EmptyPropList
 nProp = UBound(props, 1) + 1
 On Error GoTo 0
 'alloc result
 Dim res As Variant
 ReDim res(0 To nProp - 1) As Variant
 'loop over properties
 ' (in practise this routine is hardly ever called; a hash table may not be warrented here)
 For i = 0 To nProp - 1
  On Error GoTo invalidProp
  propName = props(i)
  On Error GoTo 0
  If (propName = "molarGasConstant") Then
   val = 8.314472 'we put this first in the list, as this is the value most likely to be queried
  ElseIf (propName = "avogadroConstant") Then
   val = 6.02214199E+23
  ElseIf (propName = "boltzmannConstant") Then
   val = 1.3806503E-23
  ElseIf (propName = "speedOfLightInVacuum") Then
   val = 299792458#
  ElseIf (propName = "standardAccelerationOfGravity") Then
   val = 9.80665
  Else
   Call SetError("Invalid or unsupported universal constant """ + propName + """", "ICapeThermoPropertyPackage", "GetUniversalConstant")
   Err.Raise ECapeInvalidArgumentHR
  End If
  res(i) = val
 Next i
 ICapeThermoPropertyPackage_GetUniversalConstant = res
 Exit Function
 'error handlers:
EmptyPropList:
 Call SetError("Invalid or empty list of properties", "ICapeThermoPropertyPackage", "GetUniversalConstant")
 Err.Raise ECapeInvalidArgumentHR
invalidProp:
 Call SetError("Invalid entry in list of properties", "ICapeThermoPropertyPackage", "GetUniversalConstant")
 Err.Raise ECapeInvalidArgumentHR
End Function

Private Function ICapeThermoPropertyPackage_PropCheck(ByVal materialObject As Object, ByVal props As Variant) As Variant
 'this routine checks whether a property 'can be calculated'. The routine should not take into account
 ' the calculation conditions at the material (so the material is unused), and as we do not have a phase
 ' specification or calctype specification, we only check whether the property appears in the property list
 Dim nProp As Long, iProp As Long, i As Long
 Dim found As Boolean
 Dim propName As String
 'check prop list
 On Error GoTo EmptyPropList
 nProp = UBound(props, 1) + 1
 On Error GoTo 0
 'alloc result
 Dim res
 ReDim res(0 To nProp) As Boolean
 'loop over props
 For iProp = 0 To nProp - 1
  found = False
  On Error GoTo invalidProp
  propName = props(i)
  On Error GoTo 0
  'check single phase prop list (production implementations should use a hash table)
  For i = 0 To SinglePhasePropertyCount - 1
   If propName = SinglePhasePropNames(i) Then
    found = True
    Exit For
   End If
  Next i
  If Not found Then
   'check two-phase prop list (production implementations should use a hash table)
   For i = 0 To TwoPhasePropertyCount - 1
    If propName = TwoPhasePropNames(i) Then
     found = True
     Exit For
    End If
   Next i
   If Not found Then
    'check T dependent property (production implementations should use a hash table)
    For i = 0 To TDepPropertyCount - 1
     If propName = TDepPropNames(i) Then
      found = True
      Exit For
     End If
    Next i
   End If
  End If
notAValidProp:
  On Error GoTo 0
  res(i) = found
 Next iProp
 ICapeThermoPropertyPackage_PropCheck = res
 Exit Function
 'error handlers:
EmptyPropList:
 Call SetError("Invalid or empty list of properties", "ICapeThermoPropertyPackage", "PropCheck")
 Err.Raise ECapeInvalidArgumentHR
invalidProp:
 'we do not throw an error, instead we return unsupported
 Resume notAValidProp
End Function

Private Function ICapeThermoPropertyPackage_ValidityCheck(ByVal materialObject As Object, ByVal props As Variant) As Variant
 'this function is not implemented; see spec for details
 Call SetError("ValidityCheck is not implemented", "ICapeThermoPropertyPackage", "ValidityCheck")
 Err.Raise ECapeNoImplHR
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
 Set simulationContext = Nothing 'release simulation context
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


