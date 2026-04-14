VERSION 5.00
Begin VB.Form EditForm 
   Caption         =   "Edit VB Mixer Splitter Example:"
   ClientHeight    =   1890
   ClientLeft      =   60
   ClientTop       =   345
   ClientWidth     =   4680
   Icon            =   "EditForm.frx":0000
   LinkTopic       =   "Form1"
   ScaleHeight     =   1890
   ScaleWidth      =   4680
   StartUpPosition =   2  'CenterScreen
   Begin VB.CommandButton okButton 
      Cancel          =   -1  'True
      Caption         =   "&OK"
      Default         =   -1  'True
      Height          =   375
      Left            =   3240
      TabIndex        =   6
      Top             =   1440
      Width           =   1335
   End
   Begin VB.TextBox editHeatInput 
      Height          =   285
      Left            =   1440
      TabIndex        =   3
      Top             =   480
      Width           =   2175
   End
   Begin VB.TextBox editSplitFactor 
      Height          =   285
      Left            =   1440
      TabIndex        =   1
      Top             =   120
      Width           =   2175
   End
   Begin VB.Label Label4 
      Caption         =   "CO-LaN Mixer Splitter example, provided by AmsterCHEM"
      Height          =   255
      Left            =   120
      TabIndex        =   5
      Top             =   960
      Width           =   4455
   End
   Begin VB.Label Label3 
      Caption         =   "Watt"
      Height          =   255
      Left            =   3720
      TabIndex        =   4
      Top             =   480
      Width           =   735
   End
   Begin VB.Label Label2 
      Caption         =   "Heat input:"
      Height          =   255
      Left            =   120
      TabIndex        =   2
      Top             =   495
      Width           =   1215
   End
   Begin VB.Label Label1 
      Caption         =   "Split factor:"
      Height          =   255
      Left            =   120
      TabIndex        =   0
      Top             =   135
      Width           =   1215
   End
End
Attribute VB_Name = "EditForm"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Option Explicit

Private Declare Function SetWindowPos Lib "user32" (ByVal hWnd As Long, ByVal hWndInsertAfter As Long, ByVal x As Long, ByVal y As Long, ByVal cx As Long, ByVal cy As Long, ByVal wFlags As Long) As Long

Public unit As VbMixSplitUnitOp 'set at construction

Private Sub Form_Activate()
'set this window top-most; comes in handy when running in AspenPlus
Call SetWindowPos(hWnd, -1, 0, 0, 0, 0, &H3) 'topmost, NOMOVE, NOSIZE
End Sub

Private Sub Form_Load()
'set initial values
editSplitFactor.Text = str(unit.splitFactor)
editHeatInput.Text = str(unit.heatInput)
End Sub

Private Sub UseNewValues()
'get new values
On Error GoTo 0
unit.splitFactor = Val(editSplitFactor.Text)
If (unit.splitFactor < 0) Then unit.splitFactor = 0 Else If (unit.splitFactor > 1) Then unit.splitFactor = 1
unit.heatInput = Val(editHeatInput.Text)
End Sub

Private Sub Form_Terminate()
Call UseNewValues
End Sub

Private Sub okButton_Click()
Call UseNewValues
Me.Hide
End Sub
