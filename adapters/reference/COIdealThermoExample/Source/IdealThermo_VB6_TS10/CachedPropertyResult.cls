VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
  Persistable = 0  'NotPersistable
  DataBindingBehavior = 0  'vbNone
  DataSourceBehavior  = 0  'vbNone
  MTSTransactionMode  = 0  'NotAnMTSObject
END
Attribute VB_Name = "CachedPropRes"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
'this class cached property calculation results
' during CalcProp, so that we can set the property
' on the material object only at the point that
' all property calculations have succeeded

Public propName As String
Public phase As String
Public basis As String
Public values As Variant

Public Sub SetScalar(d As Double)
 ReDim values(0 To 0) As Double
 values(0) = d
End Sub
