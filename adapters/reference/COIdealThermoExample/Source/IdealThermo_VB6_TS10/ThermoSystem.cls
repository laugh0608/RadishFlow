VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
  Persistable = 0  'NotPersistable
  DataBindingBehavior = 0  'vbNone
  DataSourceBehavior  = 0  'vbNone
  MTSTransactionMode  = 0  'NotAnMTSObject
END
Attribute VB_Name = "ThermoSystem"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = True
Option Explicit
Option Compare Text
Implements ICapeIdentification 'required for all CAPE-OPEN components
Implements ICapeUtilities      'required for all top-level CAPE-OPEN components
Implements ECapeRoot           'required in case of components that throw errors
Implements ECapeUnknown        'required in case of components that throw errors
Implements ECapeUser           'required in case of components that throw errors (all other error interfaces derive from this)
Implements ECapeNoImpl         'for ECapeNoImplHR errors
Implements ECapeIllegalAccess  'for ECapeIllegalAccessHR errors
Implements ICapeThermoSystem   'thermo system's main interface

'name and description
Dim name As String, description As String
'error variables (we have a simple error handling mechanism, providing only name(=desc) and scope and interface
Dim errDesc As String, errIface As String, errScope As String
'simulation context
Dim simulationContext

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
Call LoadIdealThermoModule 'make sure we are able to load the library of external functionality
'set name and description
name = "VB6 Ideal Thermo System"
description = "CO-LaN Example Ideal Thermo System VB6 implementation"
End Sub

'ICapeIdentification methods

Private Property Let ICapeIdentification_ComponentDescription(ByVal RHS As String)
 'description cannot be modified
 Call SetError("Description is read-only", "ICapeIdentification", "ComponentDescription")
 Err.Raise ECapeIllegalAccessHR
End Property

Private Property Get ICapeIdentification_ComponentDescription() As String
 'get description
 ICapeIdentification_ComponentDescription = description
End Property

Private Property Let ICapeIdentification_ComponentName(ByVal RHS As String)
 'name cannot be modified
 Call SetError("Name is read-only", "ICapeIdentification", "ComponentDescription")
 Err.Raise ECapeIllegalAccessHR
End Property

Private Property Get ICapeIdentification_ComponentName() As String
 'get name
 ICapeIdentification_ComponentName = name
End Property

'ICapeThermoSystem

Private Function ICapeThermoSystem_GetPropertyPackages() As Variant
'return the list of property packages on the system
Dim packageList As Variant
Call GetPackages(packageList)
ICapeThermoSystem_GetPropertyPackages = packageList
End Function

Private Function ICapeThermoSystem_ResolvePropertyPackage(ByVal PropertyPackageName As String) As Object
 'at this point we do not check whether the name represents a Property Package
 ' stored at the system, as the PropertyPackage implementation supports persistence,
 ' so Load may be called to initialize the system
 Dim P As ICapeIdentification
 'if we call new here, InitNew is implicitly called; this means that subsequent InitNew and Load calls will fail. We do not want that...
 Dim CLSID_ProptertyPackage As GUID
 Dim IID_ICapeIdentification As GUID
 Call CLSIDFromProgID(StrPtr("IdealThermo_VB6_TS10.PropertyPackage"), CLSID_ProptertyPackage)
 Call IIDFromString(StrPtr(StrIID_ICapeIdentification), IID_ICapeIdentification)
 If CoCreateInstance(CLSID_ProptertyPackage, 0&, CLSCTX_INPROC_SERVER, IID_ICapeIdentification, P) <> 0 Then
  Call SetError("Failed to create PropertyPackage object", "ICapeThermoSystem", "ResolvePropertyPackage")
  Err.Raise ECapeUnknownHR
 End If
 P.ComponentName = PropertyPackageName
 Set ICapeThermoSystem_ResolvePropertyPackage = P
End Function

'ICapeUtilities methods

Private Sub ICapeUtilities_Edit()
 'show TS edit dialog
 Call EditPackages
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
 'release simulation context
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



