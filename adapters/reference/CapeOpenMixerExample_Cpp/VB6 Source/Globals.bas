Attribute VB_Name = "Globals"
Option Explicit

Function GetErrorFromCO(o As Object) As String
 'get an error from a CAPE-OPEN object; defaults to error description in Err object if unavailable
 Dim e As String
 Dim eUser As ECapeUser
 'first store the error that sits in the error object
 e = Err.description
 'now try to get a CAPE-OPEN error
 On Error GoTo noErrorInterface
 Set eUser = o
 GetErrorFromCO = eUser.description
 Exit Function
noErrorInterface:
 'just return the error that was in the error object
 GetErrorFromCO = e
End Function

Sub ValidateArray(a, expectedType As Integer)
 'validate that the argument is a proper array and return the number of elements
 ' in case of success, the array is indexed 0 to n-1
 Dim i As Integer
 If (VarType(a) <> (vbArray Or expectedType)) Then
  Err.Raise 1, , "Unexpected data type"
 End If
 If (LBound(a, 1) <> 0) Then
  Err.Raise 1, , "Zero was expected for the lower bound of the array"
 End If
 On Error GoTo thisIsOK
 i = LBound(a, 2) 'will fail for 1-dimensional array
 On Error GoTo 0
 Err.Raise 1, , "A one-dimensional array was expected"
thisIsOK:
 Exit Sub
End Sub

