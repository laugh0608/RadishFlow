VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
  Persistable = 1  'Persistable
  DataBindingBehavior = 0  'vbNone
  DataSourceBehavior  = 0  'vbNone
  MTSTransactionMode  = 0  'NotAnMTSObject
END
Attribute VB_Name = "VbMixSplitUnitOp"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = True
Option Explicit
Option Compare Text

'this class is marked Persistable; this means that all properties will be persisted
' VB automatically implements in this case IPersistStreamInit interfaces, and provides
' entry points Class_InitProperties, Class_ReadProperties, Class_WriteProperties

'A reference must be set to the CAPE-OPEN type libaries before adding the interface definitions
' this project contains a reference to the version 1.1 CAPE-OPEN type libraries

'after initial compilation of the project, we want to keep the GUID of this class the same
' for future builds. We thus back-up a copy of the DLL and make sure in the project settings
' that we are binary compatible with this DLL

'when the class is compiled, we can find the GUID in the registry using regedit. This we
' can use to create a .reg file that will add the required additional CAPE-OPEN information
' to the registry; normally this reg file will open by default with regedit, on 64-bit
' windows systems it may be required to specifically use regedt32.exe for updating the
' registry using the reg file; the reg file adds the CapeDescription key and the CATIDs
' for a CAPE-OPEN component and a CAPE-OPEN Unit Operation

'adding the following statements will show the interface definitions in the class list;
' the methods are added by VB once selected in the method list
Implements ICapeIdentification 'required for all CAPE-OPEN components
Implements ICapeUtilities      'required for all top-level CAPE-OPEN components
Implements ECapeRoot           'required in case of components that throw errors
Implements ECapeUnknown        'required in case of components that throw errors
Implements ECapeUser           'required in case of components that throw errors (all other error interfaces derive from this)
Implements ICapeUnit           'the main unit operation interface
Implements ICapeUnitReport     'this is an optional interface; this unit exposes a sample report only

'the properties we persist
' (correspond to the values of the parameters we expose)
Public splitFactor As Double 'dimensionless
Public heatInput As Double   'Watt
'name and description
Dim name As String, description As String
'error variables (we have a simple error handling mechanism, providing only name(=desc) and scope and interface
Dim errDesc As String, errIface As String, errScope As String
'local variables
Dim currentReportIndex As Integer
Dim simulationContext As Object
Dim parameterCollection As Collection
Dim portCollection As Collection
Public valStatus As CapeValidationStatus
Public NaN As Double
Dim nCompounds As Integer 'stored at validation time

'function to store error

Private Sub SetError(desc As String, iface As String, scope As String)
 'call this function before returning a CAPE-OPEN error
 ' these strings are then available via the CAPE-OPEN error interfaces
 errDesc = desc
 errIface = iface
 errScope = scope
End Sub

'the constructor

Private Sub Class_Initialize()
 Dim par As DoubleParameter
 Dim port As MaterialPort
 'defaults for the variables
 splitFactor = 0.5
 heatInput = 0#
 currentReportIndex = -1
 '... and locals
 Set simulationContext = Nothing
 valStatus = CAPE_NOT_VALIDATED
 'default name and description (can be changed via ICapeIdentification)
 name = "VBMixSplitExample"
 description = "Visual Basic 6.0 Mixer and Splitter Example according to CAPE-OPEN Unit Operation specification"
 'create the parameter collection
 Set parameterCollection = New Collection
 parameterCollection.name = "VB Mixer Splitter Parameter Collection"
 'create the parameters
 Set par = New DoubleParameter
 par.typ = ParameterSplitFactor
 par.name = "Split factor"
 Set par.unit = Me
 Call parameterCollection.AddItem(par)
 Set par = New DoubleParameter
 par.typ = ParameterHeatInput
 par.name = "Heat input"
 Set par.unit = Me
 Call parameterCollection.AddItem(par)
 'create the port collection
 Set portCollection = New Collection
 portCollection.name = "VB Mixer Splitter Port Collection"
 'create the ports; we create 2 feeds and 2 products
 Set port = New MaterialPort
 port.direction = CAPE_INLET
 port.name = "Feed 1"
 Call portCollection.AddItem(port)
 Set port = New MaterialPort
 port.direction = CAPE_INLET
 port.name = "Feed 2"
 Call portCollection.AddItem(port)
 Set port = New MaterialPort
 port.direction = CAPE_OUTLET
 port.name = "Product 1"
 Call portCollection.AddItem(port)
 Set port = New MaterialPort
 port.direction = CAPE_OUTLET
 port.name = "Product 2"
 Call portCollection.AddItem(port)
 'VB6 has no native support for NaN values (not-a-number); this is however the value used for numbers that
 ' are missing in CAPE-OPEN, so we will generate one with a detour here
 NaN = GetQNaN()
End Sub

Private Function GetQNaN() As Double
 'generate a quiet NaN, Visual Basic has no native support for this
 On Error Resume Next
 GetQNaN = 0 / 0
End Function

'Making an object persistable exposes the following methods

Private Sub Class_InitProperties()
 'this call will result from the simulation environment calling InitNew
 ' it is safest to presume the simulation environment misbehaves, and we use the constructor of the object instead to initialize the properties
End Sub

Private Sub Class_ReadProperties(PropBag As PropertyBag)
 splitFactor = PropBag.ReadProperty("SplitFactor", 0.5)
 heatInput = PropBag.ReadProperty("HeatInput", 0#)
 name = PropBag.ReadProperty("ComponentName", name)
 description = PropBag.ReadProperty("ComponentDescription", name)
End Sub

Private Sub Class_WriteProperties(PropBag As PropertyBag)
 Call PropBag.WriteProperty("SplitFactor", splitFactor)
 Call PropBag.WriteProperty("HeatInput", heatInput)
 Call PropBag.WriteProperty("ComponentName", name)
 Call PropBag.WriteProperty("ComponentDescription", description)
End Sub

'ICapeIdentification methods

Private Property Let ICapeIdentification_ComponentDescription(ByVal RHS As String)
 'set description
 description = RHS
End Property

Private Property Get ICapeIdentification_ComponentDescription() As String
 'get description
 ICapeIdentification_ComponentDescription = description
End Property

Private Property Let ICapeIdentification_ComponentName(ByVal RHS As String)
 'set name
 name = RHS
End Property

Private Property Get ICapeIdentification_ComponentName() As String
 'get name
 ICapeIdentification_ComponentName = name
End Property

'ICapeUtilities methods

Private Sub ICapeUtilities_Edit()
 'show a simple edit dialog
 Dim editDlg As New EditForm
 Set editDlg.unit = Me
 editDlg.Show vbModal
 'after edit, we consider ourselves not validated
 valStatus = CAPE_NOT_VALIDATED
End Sub

Private Sub ICapeUtilities_Initialize()
 'does nothing; should be called by the simulation environment after InitNew or Load
End Sub

Private Property Get ICapeUtilities_parameters() As Object
 'return the parameter collection
 Set ICapeUtilities_parameters = parameterCollection
End Property

Private Property Let ICapeUtilities_simulationContext(ByVal RHS As Object)
 'set the simulation context
 ' it is not used by this unit operation, but it may be used to log messages in simulation environments that support this
 Set simulationContext = RHS
End Property

Private Sub ICapeUtilities_Terminate()
 'disconnect the ports
 On Error GoTo 0
 Dim i As Integer, count As Integer
 Dim port As ICapeUnitPort
 count = portCollection.ICapeCollection_Count
 For i = 1 To count
  Set port = portCollection.ICapeCollection_Item(i)
  Call port.Disconnect
 Next i
 'release the simulation context
 Set simulationContext = Nothing
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

'ICapeUnit methods

Private Property Get ICapeUnit_ports() As Object
 'return the port collection
 Set ICapeUnit_ports = portCollection
End Property

Private Property Get ICapeUnit_ValStatus() As CapeValidationStatus
 'return the current validation status
 ICapeUnit_ValStatus = valStatus
End Property

Private Function ICapeUnit_Validate(message As String) As Boolean
 'validate the unit
 Dim valid As Boolean
 Dim port As MaterialPort
 Dim portName1 As String, portName2 As String
 Dim haveConnectedFeed As Boolean
 Dim haveConnectedProduct As Boolean
 Dim compList1, compList2
 Dim same As Boolean
 Dim haveEnthalpy As Boolean
 Dim propList
 Dim i As Integer, j As Integer, k As Integer
 On Error GoTo validationError
 'presume we are valid, unless we fail a test
 valid = True
 'perform validity tests
 ' check if at least one feed port is connected
 haveConnectedFeed = False
 haveConnectedProduct = False
 For i = 1 To 4 'loop over all ports
  Set port = portCollection.ICapeCollection_Item(i)
  If (port.Connected) Then
   If (i <= 2) Then
    'this is a feed port
    haveConnectedFeed = True
   Else
    'this is a product port
    haveConnectedProduct = True
   End If
  End If
 Next i
 If Not haveConnectedFeed Then
  message = "At least one feed port must be connected"
  valid = False
 ElseIf Not haveConnectedProduct Then
  message = "At least one product port must be connected"
  valid = False
 End If
 If valid Then
  'additional validation, check that for all connected ports, the component slate is the same
  ' find the first connected port
  For i = 1 To 4
   Set port = portCollection.ICapeCollection_Item(i)
   If port.Connected Then
    'get the component ID list from this port
    compList1 = port.GetMaterial.GetComponentIDs
    portName1 = port.name
    'check the remaining ports
    For j = i + 1 To 4
     Set port = portCollection.ICapeCollection_Item(j)
     If port.Connected Then
      compList2 = port.GetMaterial.GetComponentIDs
      'store the number of compounds, we use it during calculation
      nCompounds = UBound(compList1) + 1
      'check that these are the same
      same = True
      If (UBound(compList1) <> UBound(compList2)) Then
       same = False
      Else
       For k = 0 To UBound(compList1)
        If (compList1(k) <> compList2(k)) Then 'string comparison is case insentive (see Option Compare above this file)
         same = False
         Exit For
        End If
       Next k
      End If
      If Not same Then
       'not ok
       portName2 = port.name
       message = "The compound lists exposed by the material objects connected to ports " + portName1 + " and " + portName2 + " are not the same"
       valid = False
       Exit For 'break out of loop j
      End If
     End If
    Next j
    Exit For 'break out of loop i
   End If
  Next i
 End If
 If valid Then
  'check that enthalpy is a supported property (use first connected port)
  For i = 1 To 4
   Set port = portCollection.ICapeCollection_Item(i)
   If port.Connected Then
    propList = port.GetMaterial.GetSinglePhasePropList
    'check if we have enthalpy
    haveEnthalpy = False
    If Not (IsEmpty(propList)) Then
     For j = 0 To UBound(propList)
      If (propList(j) = "Enthalpy") Then 'string comparison is case insensitive
       haveEnthalpy = True
       Exit For
      End If
     Next j
    End If
    If Not haveEnthalpy Then
     message = "Enthalpy does not appear to be supported by simulation environment. This unit operation requires enthalpy."
     valid = False
    End If
    Exit For
   End If
  Next i
 End If
 'set the validation status
 If (valid) Then valStatus = CAPE_VALID Else valStatus = CAPE_INVALID
 'done
 ICapeUnit_Validate = valid
Exit Function
validationError:
 'we have run into an error; the current error description is in the Err object
 ' notice that the MaterialObject objects obtain suitable CAPE-OPEN error
 ' information from the corresponding CAPE-OPEN Material Objects and place this
 ' in the Err object
 On Error GoTo 0
 Call SetError(Err.description, "ICapeUnit", "Validate")
 Err.Raise ECapeUnknownHR
End Function

Private Sub ICapeUnit_Calculate()
 'calculate the unit
 Dim i As Integer, j As Integer
 Dim port As MaterialPort
 Dim mat As MaterialObject, dupMat As MaterialObject
 Dim compFlows() As Double, composition() As Double
 Dim pressure As Double, temperature As Double
 Dim totalEnthalpy As Double 'J/s
 Dim flow As Double, phaseFraction As Double
 Dim phaseList
 Dim phase As String
 Dim v
 Dim d As Double
 'make sure the unit is valid
 If (valStatus = CAPE_INVALID) Then
  Call SetError("Unit is not valid", "ICapeUnit", "Calculate")
  Err.Raise ECapeUnknownHR
 End If
 If (valStatus = CAPE_NOT_VALIDATED) Then
  Call SetError("Unit has not been validated", "ICapeUnit", "Calculate")
  Err.Raise ECapeUnknownHR
 End If
 'all errors below are forwarded as a CAPE-OPEN error
 On Error GoTo calculateError
 'initialize variables
 ReDim composition(nCompounds)
 ReDim compFlows(nCompounds)
 For i = 1 To nCompounds
  compFlows(i) = 0
 Next i
 pressure = 0
 totalEnthalpy = 0
 Set dupMat = Nothing
 'get the total compound flows and enthalpy flow from the feed, and keep track of min pressure
 ' the compound flows we can get directly; the enthalpy we must calculate from a duplicate material
 For i = 1 To 2
  Set port = portCollection.ICapeCollection_Item(i)
  If (port.Connected) Then
   Set mat = port.GetMaterial
   'get pressure
   v = mat.GetOverallProperty("pressure", vbNullString)
   If (UBound(v) <> 0) Then
    Err.Raise 1, , "Unexpected values for pressure; expected a scalar"
   End If
   If (pressure = 0) Then
    'use this pressure
    pressure = v(0) 'Pa
   Else
    'use the minimum pressure
    If (v(0) < pressure) Then pressure = v(0) 'Pa
   End If
   'get total flow
   v = mat.GetOverallProperty("totalFlow", "mole")
   If (UBound(v) <> 0) Then
    Err.Raise 1, , "Unexpected values for totalflow; expected a scalar"
   End If
   If (v(0) > 0) Then 'otherwise this flow does not contribute to the overall flow
    flow = v(0) 'mol/s
    'get overall composition
    v = mat.GetOverallProperty("fraction", "mole")
    If (UBound(v) <> (nCompounds - 1)) Then 'indexed from 0 to nCompounds-1
     Err.Raise 1, , "Unexpected values for fraction; unexpected number of values"
    End If
    'add to compound flows
    For j = 1 To nCompounds
     compFlows(j) = compFlows(j) + flow * v(j - 1)
    Next j
    'calculate overall enthalpy on a duplicate material object
    ' (we are not allowed to calculate properties on material objects connected to feeds)
    Set dupMat = mat.Duplicate
    'we need to loop over each phase
    phaseList = dupMat.PresentPhaseList
    If (IsEmpty(phaseList)) Then Err.Raise 1, , "No phases are present on duplicate material of feed material"
    For j = 0 To UBound(phaseList)
     phase = phaseList(j)
     'get the phase fraction for this phase
     v = dupMat.GetSinglePhaseProperty("phaseFraction", phase, vbNullString, "mole") 'mol/mol
     If (UBound(v) <> 0) Then
      Err.Raise 1, , "Unexpected values for phase fraction of phase " + phase + "; expected a scalar"
     End If
     If (v(0) > 0) Then 'or this phase will not contribute to total enthalpy
      phaseFraction = v(0)
      'calculate the enthalpy for this phase
      Call dupMat.CalcSinglePhaseProperty("enthalpy", phase)
      'get the enthalpy for this phase
      v = dupMat.GetSinglePhaseProperty("enthalpy", phase, "mixture", "mole") 'J/mol
      'total enthalpy is increased by f * sum ( phaseFraction * enthalpy of phase )
      totalEnthalpy = totalEnthalpy + flow * phaseFraction * v(0) '[J/s]=[J/s]+[mol/s]*[mol/mol]*[J/mol]
     End If
    Next j
   End If
  End If
 Next i
 'we have the feed values; calculate the temperature from the total enthalpy
 totalEnthalpy = totalEnthalpy + heatInput 'add the heat input to total enthalpy
 'calculate overall composition
 flow = 0
 For i = 1 To nCompounds
  flow = flow + compFlows(i)
 Next i
 If (flow = 0) Then
  'the total flow is zero; we cannot do a PH flash in this case. Instead, we will use the average temperature of the feed streams
  ' we can also not determine product composition; we will use the average feed composition
  ' although we can fail this run; we can attempt to return a result that matches mass and energy balance
  If (heatInput <> 0) Then
   'we cannot possibly match the energy balance
   Err.Raise 1, , "Unable to close the energy balance with zero total flow and non-zero heat input"
  End If
  For i = 1 To nCompounds
   composition(i) = 0
  Next i
  temperature = 0
  d = 0
  For i = 1 To 2
   Set port = portCollection.ICapeCollection_Item(i)
   If (port.Connected) Then
    Set mat = port.GetMaterial
    'get temperature
    v = mat.GetOverallProperty("temperature", vbNullString)
    If (UBound(v) <> 0) Then
     Err.Raise 1, , "Unexpected values for temperature; expected a scalar"
    End If
    temperature = temperature + v(0)
    'get composition
    v = mat.GetOverallProperty("fraction", "mole")
    If (UBound(v) <> (nCompounds - 1)) Then
     Err.Raise 1, , "Unexpected values for fraction; unexpected number of values"
    End If
    For j = 1 To nCompounds
     composition(j) = composition(j) + v(j - 1)
    Next j
    'divide by one more stream later on
    d = d + 1#
   End If
  Next i
  temperature = temperature / d
  For i = 1 To nCompounds
   composition(i) = composition(i) / d
  Next i
 Else
  'calculate the composition from compounds flows
  For i = 1 To nCompounds
   composition(i) = compFlows(i) / flow
  Next i
  'use a duplicate material object, we will use the last one that have have used from the feed analysis above
  If (dupMat Is Nothing) Then Err.Raise 1, , "Internal error: duplicate material not set"
  'calculate temperature from a PH flash at duplicate material;
  ' enthalpy specification is in [J/mol] = [J/s]/[mol/s]
  temperature = dupMat.TemperatureFromPHFlash(nCompounds, composition, totalEnthalpy / flow, pressure)
 End If
 'now we have the total temperature, total composition and total flow, set the outputs using a TP flash
 j = 0 'check how many connected products we have
 For i = 3 To 4
  Set port = portCollection.ICapeCollection_Item(i)
  If (port.Connected) Then
   j = j + 1
  End If
 Next i
 'loop over the products to set result
 For i = 3 To 4
  Set port = portCollection.ICapeCollection_Item(i)
  If (port.Connected) Then
   If (j = 2) Then
    'we have both products connected, use splitFactor and 1-splitFactor
    If (i = 3) Then
     d = splitFactor
    Else
     d = 1# - splitFactor
    End If
   Else
    'we have only one product connected, which will take all product flow
    d = 1#
   End If
   Call port.GetMaterial.SetFromFlowTPX(nCompounds, d * flow, composition, temperature, pressure)
  End If
 Next i
 'done
Exit Sub
calculateError:
 'forward the internal error as a CAPE-OPEN error
 On Error GoTo 0
 Call SetError("Calculate failed: " + Err.description, "ICapeUnit", "Calculate")
 Err.Raise ECapeUnknownHR
End Sub

'ICapeUnit methods
' this is an optional interface; a sample report here shows how to do it

Private Property Get ICapeUnitReport_reports() As Variant
'return the list of reports. We have only a sample report:
Dim v
ReDim v(0 To 0) As String
v(0) = "Sample report"
ICapeUnitReport_reports = v
End Property

Private Property Let ICapeUnitReport_selectedReport(ByVal RHS As String)
If (StrComp(RHS, "Sample report") = 0) Then
 'select the sample report
 currentReportIndex = 0
Else
 'invalid report
 Call SetError("Invalid report name: no such report", "ICapeUnitReport", "putSelectedReport")
 Err.Raise ECapeUnknownHR
End If
End Property

Private Property Get ICapeUnitReport_selectedReport() As String
If (currentReportIndex) Then
 Call SetError("A report has not been selected", "ICapeUnitReport", "ProduceReport")
 Err.Raise ECapeUnknownHR
End If
'the sample report is selected
ICapeUnitReport_selectedReport = "Sample report"
End Property

Private Sub ICapeUnitReport_ProduceReport(reportContent As String)
If (currentReportIndex) Then
 Call SetError("A report has not been selected", "ICapeUnitReport", "ProduceReport")
 Err.Raise ECapeUnknownHR
End If
'we do not need to check which report is selected, we only have one
reportContent = "Example Mixer Splitter Report Content"
End Sub



