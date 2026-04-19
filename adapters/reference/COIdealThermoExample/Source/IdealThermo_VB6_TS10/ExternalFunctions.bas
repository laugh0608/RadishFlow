Attribute VB_Name = "ExternalFunctions"
Option Explicit

'all functions declared here are from IdealThermoModule.dll; for
' usage, this DLL and IdealThermoModule.dll should be located in
' the same folder. For debugging, it is handy to have the folder
' that contains IdealThermoModule.dll in the path

'see VB6Exports.cpp inn the IdealThermoModule project for documentation of
' these functions

'thermo system functions
Public Declare Sub GetPackages Lib "IdealThermoModule.dll" (ByRef packages As Variant)
Public Declare Sub EditPackages Lib "IdealThermoModule.dll" ()

'property package functions
' instead of creating an external COM object, we reference property
' packages by handle. This way, all we need is external routines

Public Declare Function PPCreatePropertyPackage Lib "IdealThermoModule.dll" () As Long
Public Declare Sub PPDeletePropertyPackage Lib "IdealThermoModule.dll" (ByVal handle As Long)
Public Declare Function PPGetLastError Lib "IdealThermoModule.dll" (ByVal handle As Long) As Variant
Public Declare Function PPLoad Lib "IdealThermoModule.dll" (ByVal handle As Long, ByVal path As String) As Boolean
Public Declare Function PPSave Lib "IdealThermoModule.dll" (ByVal handle As Long, ByVal path As String) As Boolean
Public Declare Function PPLoadFromPPFile Lib "IdealThermoModule.dll" (ByVal handle As Long, ByVal ppName As String) As Boolean
Public Declare Sub PPEdit Lib "IdealThermoModule.dll" (ByVal handle As Long)
Public Declare Function PPGetCompoundCount Lib "IdealThermoModule.dll" (ByVal handle As Long, ByRef count As Long) As Boolean
Public Declare Function PPGetCompoundStringConstant Lib "IdealThermoModule.dll" (ByVal handle As Long, ByVal compIndex As Long, ByVal constID As Long) As Variant
Public Declare Function PPGetCompoundRealConstant Lib "IdealThermoModule.dll" (ByVal handle As Long, ByVal compIndex As Long, ByVal constID As Long, ByRef value As Double) As Boolean
Public Declare Function PPGetTemperatureDependentProperty Lib "IdealThermoModule.dll" (ByVal handle As Long, ByVal compIndex As Long, ByVal propID As Long, ByVal T As Double, ByRef value As Double) As Boolean
Public Declare Function PPGetPropertyResult Lib "IdealThermoModule.dll" (ByVal handle As Long, ByVal resultIndex As Long) As Variant
Public Declare Function PPCalcSinglePhaseProps Lib "IdealThermoModule.dll" (ByVal handle As Long, ByVal nComp As Long, ByRef compIndices As Long, ByVal phaseID As Long, ByVal T As Double, ByVal P As Double, ByRef X As Double, ByVal nProp As Long, ByRef propIDs As Long) As Boolean
Public Declare Function PPCalcTwoPhaseProps Lib "IdealThermoModule.dll" (ByVal handle As Long, ByVal nComp As Long, ByRef compIndices As Long, ByVal phaseID1 As Long, ByVal phaseID2 As Long, ByVal T1 As Double, ByVal T2 As Double, ByVal P1 As Double, ByVal P2 As Double, ByRef X1 As Double, ByRef X2 As Double, ByVal nProp As Long, ByRef propIDs As Long) As Boolean
Public Declare Function PPFlashPhaseResult Lib "IdealThermoModule.dll" (ByVal handle As Long, ByVal index As Long, ByRef phase As Long, ByRef phaseFrac As Variant, ByRef phaseComposition As Variant) As Boolean
Public Declare Function PPFlash Lib "IdealThermoModule.dll" (ByVal handle As Long, ByVal nComp As Long, ByRef compIndices As Long, ByRef X As Double, ByVal flashType As Long, ByVal phaseType As Long, ByVal spec1 As Double, ByVal spec2 As Double, ByRef phaseCount As Long, ByRef T As Double, ByRef P As Double) As Boolean

'some standard windows functionality we need:
Public Declare Function GetTempPathA Lib "kernel32" (ByVal nBufferLength As Long, ByVal lpBuffer As String) As Long
Public Declare Function GetTempFileNameA Lib "kernel32" (ByVal lpszPath As String, ByVal lpPrefixString As String, ByVal wUnique As Long, ByVal lpTempFileName As String) As Long

'for cocreateinstance of a property package (see ResolvePropertyPackage)
Public Type GUID
   Data1 As Long
   Data2 As Integer
   Data3 As Integer
   Data4(1 To 8) As Byte
End Type
Public Declare Function CoCreateInstance Lib "ole32.dll" (rclsid As Any, ByVal pUnkOuter As Long, ByVal dwClsContext As Long, riid As Any, pvarResult As Object) As Long
Public Declare Function CLSIDFromProgID Lib "ole32.dll" (ByVal lpszProgID As Long, ByRef pGuid As Any) As Long
Public Declare Function IIDFromString Lib "ole32.dll" (ByVal lpszProgID As Long, ByRef pGuid As Any) As Long
Public Const CLSCTX_INPROC_SERVER As Long = 1&
Public Const StrIID_ICapeIdentification As String = "{678C0990-7D66-11D2-A67D-00105A42887F}"

'function to make sure that the library gets loaded from the current folder, if available
Dim dllHandle As Long
Dim dllRefCount As Long
Private Declare Function LoadLibrary Lib "kernel32" Alias "LoadLibraryA" (ByVal lpLibFileName As String) As Long
Private Declare Function FreeLibrary Lib "kernel32" (ByVal hLibModule As Long) As Long

Public Sub LoadIdealThermoModule()
If dllHandle <> 0 Then
 dllRefCount = dllRefCount + 1
 Exit Sub
End If
'try to load IdealThermoModule.dll from the path
dllHandle = LoadLibrary("IdealThermoModule.dll")
If (dllHandle = 0) Then
 'try to load IdealThermoModule.dll from the folder that contains this DLL
 dllHandle = LoadLibrary(App.path + "\IdealThermoModule.dll")
 If (dllHandle = 0) Then MsgBox "IdealThermoModule.dll cannot be located", vbCritical, "VB6 Thermo System:"
End If
If (dllHandle <> 0) Then dllRefCount = 1
End Sub

Public Sub UnloadIdealThermoModule()
If (dllHandle <> 0) Then
 dllRefCount = dllRefCount - 1
 If dllRefCount = 0 Then
  Call FreeLibrary(dllHandle)
  dllHandle = 0
 End If
End If
End Sub

