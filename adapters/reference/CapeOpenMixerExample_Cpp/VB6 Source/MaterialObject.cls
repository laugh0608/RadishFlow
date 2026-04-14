VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
  Persistable = 0  'NotPersistable
  DataBindingBehavior = 0  'vbNone
  DataSourceBehavior  = 0  'vbNone
  MTSTransactionMode  = 0  'NotAnMTSObject
END
Attribute VB_Name = "MaterialObject"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Option Explicit

'this implements a wrapper for a CAPE-OPEN material object that is either connected to a port,
' or a duplicate from an object connected to a port; it deals with version 1.0 and 1.1 of
' thermo, which is transparent to the calling object

'this is an interface class; the actual implementations are in MaterialObject10 for version 1.0 and
' MaterialObject11 for version 1.1


Public Function SetMO(MO) As Boolean
End Function

Public Function Duplicate() As MaterialObject
End Function

Public Function GetComponentIDs()
End Function

Public Function GetOverallProperty(propName As String, basis As String)
End Function

Public Function GetSinglePhasePropList()
End Function

Public Function PresentPhaseList()
End Function

Public Function GetSinglePhaseProperty(propName As String, phaseName As String, calcType As String, basis As String)
End Function

Public Sub CalcSinglePhaseProperty(propName As String, phaseName As String)
End Sub

Public Function TemperatureFromPHFlash(nCompounds As Integer, overallComposition() As Double, enthalpy As Double, pressure As Double) As Double
End Function

Public Sub SetFromFlowTPX(nCompounds As Integer, flow As Double, overallComposition() As Double, temperature As Double, pressure As Double)
End Sub

