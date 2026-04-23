Imports System
Imports System.Collections.Generic
Imports System.Text
Public Class MixerExample
    Inherits CapeOpen.CUnitBase



    ''' <summary>
    ''' Creates an instance of the CMixerExample unit operation.
    ''' </summary>
    ''' <remarks>
    ''' Me constructor demonstates the addition of a <seealso c= "CapeOpen.CBoolParameter"/>,
    ''' <seealso c= "CapeOpen.CIntParameter"/>, <seealso c= "CapeOpen.COptionParameter"/>,
    ''' and a <seealso c= "CapeOpen.CRealParameter"/> parameter to the parameter collection.
    ''' In addition, the mixer unit has three <seealso c= "CapeOpen.CUnitPort"/> ports
    ''' added to the port collection. See the documentation for the 
    ''' <seealso c= "Calculate"/> method for details on its implementation.
    ''' </remarks>
    ''' <example>
    ''' An example of how to create a unit operation. Parameter and port objects are created 
    ''' and added the their respective collections. Ports are implemented by the <see cref="CapeOpen.CUnitPort"/> 
    ''' class and are placed in the Port Collection. Parameters are added to the Parameter Collection 
    ''' class. The available parameter classes are <see cref="CapeOpen.CRealParameter"/>, <see cref="CapeOpen.CRealParameter"/>, 
    ''' <see cref="CapeOpen.CRealParameter"/>, and <see cref="CapeOpen.CRealParameter"/>.
    ''' <code>
    '''     Public Sub New()
    ''' 
    ''' ' Add Ports using the CUnitPort constructor.
    '''         Me.Ports.Add(New CapeOpen.CUnitPort("Inlet Port1", "Test Inlet Port1", CapeOpen.CapePortDirection.CAPE_INLET, CapeOpen.CapePortType.CAPE_MATERIAL))
    '''         Me.Ports.Add(New CapeOpen.CUnitPort("Inlet Port2", "Test Inlet Port2", CapeOpen.CapePortDirection.CAPE_INLET, CapeOpen.CapePortType.CAPE_MATERIAL))
    '''         Me.Ports.Add(New CapeOpen.CUnitPort("Outlet Port", "Test Outlet Port", CapeOpen.CapePortDirection.CAPE_OUTLET, CapeOpen.CapePortType.CAPE_MATERIAL))
    ''' 
    ''' ' Add a real valued parameter using the CRealParameter  constructor.
    '''         Me.Parameters.Add(New CapeOpen.CRealParameter("PressureDrop", "Drop in pressure between the outlet from the mixer and the pressure of the lower pressure inlet.", 0.0, 0.0, 0.0, 100000000.0, CapeOpen.CapeParamMode.CAPE_INPUT, "Pa"))
    ''' 
    '''  ' Add a real valued parameter using the CIntParameter  constructor.
    '''         Me.Parameters.Add(New CapeOpen.CIntParameter("Parameter2", 12, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT))
    ''' 
    '''  ' Add a real valued parameter using the CBoolParameter  constructor.
    '''         Me.Parameters.Add(New CapeOpen.CBoolParameter("Parameter3", False, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT))
    ''' 
    '''  ' Create an array of strings for the option parameter restricted value list.
    '''     Dim options As String() = {"Test Value", "Another Value"}
    ''' 
    '''  ' Add a string valued parameter using the COptionParameter constructor.
    '''     Me.Parameters.Add(New CapeOpen.COptionParameter("Parameter4", "OptionParameter", "Test Value", "Test Value", options, True, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT))
    ''' 
    '''  ' Add an available report.
    '''         Me.Reports.Add("Calculation Report")
    ''' </code>
    ''' </example>
    Public Sub New()

        ' Add Ports using the CUnitPort constructor.
        Me.Ports.Add(New CapeOpen.CUnitPort("Inlet Port1", "Test Inlet Port1", CapeOpen.CapePortDirection.CAPE_INLET, CapeOpen.CapePortType.CAPE_MATERIAL))
        Me.Ports.Add(New CapeOpen.CUnitPort("Inlet Port2", "Test Inlet Port2", CapeOpen.CapePortDirection.CAPE_INLET, CapeOpen.CapePortType.CAPE_MATERIAL))
        Me.Ports.Add(New CapeOpen.CUnitPort("Outlet Port", "Test Outlet Port", CapeOpen.CapePortDirection.CAPE_OUTLET, CapeOpen.CapePortType.CAPE_MATERIAL))

        ' Add a real valued parameter using the CRealParameter  constructor.
        Me.Parameters.Add(New CapeOpen.CRealParameter("PressureDrop", "Drop in pressure between the outlet from the mixer and the pressure of the lower pressure inlet.", 0.0, 0.0, 0.0, 100000000.0, CapeOpen.CapeParamMode.CAPE_INPUT, "Pa"))

        ' Add a real valued parameter using the CIntParameter  constructor.
        Me.Parameters.Add(New CapeOpen.CIntParameter("Parameter2", 12, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT))

        ' Add a real valued parameter using the CBoolParameter  constructor.
        Me.Parameters.Add(New CapeOpen.CBoolParameter("Parameter3", False, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT))

        ' Create an array of strings for the option parameter restricted value list.
        Dim options As String() = {"Test Value", "Another Value"}

        ' Add a string valued parameter using the COptionParameter constructor.
        Me.Parameters.Add(New CapeOpen.COptionParameter("Parameter4", "OptionParameter", "Test Value", "Test Value", options, True, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT))

        ' Add an available report.
        Me.Reports.Add("Calculation Report")
    End Sub

    ''' <summary>
    ''' Initialization member required for CAPE-OPEN compliance.
    ''' </summary>
    ''' <remarks>
    ''' The initialization tasks are accomplished in the class construcotr, so 
    ''' Me method is empty.
    ''' </remarks>
    Public Overrides Sub Initialize()

    End Sub


    ''' <summary>
    ''' Calculation method for the CMixerExample110 unit operation.
    ''' </summary>
    ''' <remarks>
    ''' A mixer unit operation combined the material flows from two inlet ports into the flow of a single outlet port.
    ''' In order to do Me calculation, the mixer unit obtains flow information from each inlet port,
    ''' the adds the flows to obtain the flow of the outlet port. In the case of the mixer below, it is assumed that the
    ''' components are the same in each material object and that the components are listed in the same order.
    ''' After the combined flow is calculated at the values set to the outlet port, along with the 
    ''' enthalpy of the stream calculated from an energy balance and the pressure determined from
    ''' the inlet pressures, the outlet stream can be flahsed to determine equilibrium conditions.
    ''' The last task is releasing any duplicate material objects obtained.
    ''' </remarks>
    ''' <example>
    ''' <para>An example of how to calculate a unit operation. Me method obtains material objects 
    ''' from each of ports using the <seealso c= "CapeOpen.CPortCollection"/> class. The <seealso c= "CapeOpen.ICapeThermoMaterial"/>
    ''' interface is used to obtain the names using the CompIds() method, flows of each of the 
    ''' components present in the material object and overall pressure obtained using the 
    ''' <seealso c= "CapeOpen.ICapeThermoMaterial.GetOverallProp"/> method. The enthaply of 
    ''' each phase in the inlet materials is calculated using <seealso c= "CapeOpen.ICapeThermoPropertyRoutine.CalcSinglePhaseProp"/>
    ''' method. The inlet enthalpy was multiplied by the phase flow (phase fraction * overall flow) 
    ''' which was summed to determine the outlet stream enthalpy. 
    ''' The output pressure from the lower of the two streams' pressure and pressure drop parameter. 
    ''' Lastly, the results of the 
    ''' calculations are applied to the output material object using the <seealso c= "CapeOpen.ICapeThermoMaterial.SetOverallProp"/> 
    ''' method. The last method of the calculate method is to call the material's 
    ''' <seealso c= "CapeOpen.ICapeThermoEquilibriumRoutine.CalcEquilibrium"/> method.</para>
    ''' <para>
    ''' A calculation report is generated by Me method that is made avalable through the
    ''' <seealso c= "CapeOpen.ICapeUnitReport.ProduceReport"/> method.
    ''' </para>
    ''' <para>
    ''' In Me case, the inlet materials need to be released. Me is accomplished using the
    ''' <seealso c= "System.Runtime.InteropServices.Marshal.ReleaseComObject"/> method.
    ''' Using Me method to release the outlet material object would result in an null object reference error.
    ''' </para>
    ''' <para>
    ''' The method also documents use of the <seealso c= "CapeOpen.CCapeObject.throwException"/> method to provide
    ''' CAPE-OPEN compliant error handling.
    ''' </para>
    ''' <code>
    '''     Public Overrides Sub Calculate()
    '''     Try
    '''     Dim phases() As String = {"Overall"}
    '''     Dim props() As String = {"enthalpy"}
    '''     Dim in1 As CapeOpen.ICapeThermoMaterialObject = Me.Ports(0).connectedObject
    '''     Dim in1Comps() As String = in1.ComponentIds
    '''     Dim in1Flow() As Double = in1.GetProp("flow", "Overall", Nothing, Nothing, "mole")
    '''         in1.CalcProp(props, phases, "Mixture")
    '''     Dim in1Enthalpy() As Double = in1.GetProp("enthalpy", "Overall", Nothing, "Mixture", "mole")
    '''     Dim in2 As CapeOpen.ICapeThermoMaterialObject = Me.Ports(1).connectedObject
    '''     Dim in2Comps() As String = in2.ComponentIds
    '''     Dim in2Flow() As Double = in2.GetProp("flow", "Overall", Nothing, Nothing, "mole")
    '''         in2.CalcProp(props, phases, "Mixture")
    '''     Dim in2Enthalpy() As Double = in2.GetProp("enthalpy", "Overall", Nothing, "Mixture", "mole")
    '''     Dim outPort As CapeOpen.ICapeThermoMaterialObject = Me.Ports(2).connectedObject
    '''     Dim values(0) As Double
    '''         values(0) = in1Enthalpy(0) + in2Enthalpy(0)
    '''         outPort.SetProp("enthalpy", "Overall", Nothing, "Mixture", "mole", values)
    '''         values(0) = Me.Parameters(0).value
    '''         outPort.SetProp("Pressure", "Overall", Nothing, Nothing, Nothing, values)
    '''         ReDim values(in1Comps.Length - 1)
    '''     Dim i As Integer
    '''         For i = 0 To in1Comps.Length - 1
    '''             values(i) = in1Flow(i) + in2Flow(i)
    '''         Next i
    '''         outPort.SetProp("flow", "Overall", in1Comps, Nothing, "mole", values)
    '''         outPort.CalcEquilibrium("PH", Nothing)
    ''' 
    '''        Catch p_Ex As System.Exception
    '''           System.Windows.Forms.MessageBox.Show(p_Ex.ToString())
    '''       End Try
    '''
    ''' End Sub
    ''' </code>
    ''' </example>
    ''' <seealso cref="CapeOpen.ICapeThermoMaterial"/>
    ''' <seealso cref="CapeOpen.ICapeThermoCompounds"/>
    ''' <seealso cref="CapeOpen.ICapeThermoPhases"/>
    ''' <seealso cref="CapeOpen.ICapeThermoPropertyRoutine"/>
    ''' <seealso cref="CapeOpen.ICapeThermoEquilibriumRoutine"/>
    ''' <seealso cref="CapeOpen.COMExceptionHandler"/>
    Public Overrides Sub Calculate()
        Try

            Dim phases() As String = {"Overall"}
            Dim props() As String = {"enthalpy"}
            Dim temp As CapeOpen.ICapeThermoMaterialObject = Me.Ports(0).connectedObject
            Dim in1 As CapeOpen.ICapeThermoMaterialObject = temp.Duplicate()
            If temp.GetType().IsCOMObject Then
                System.Runtime.InteropServices.Marshal.ReleaseComObject(temp)
            End If
            Dim in1Comps() As String = in1.ComponentIds
            Dim in1Flow() As Double = in1.GetProp("flow", "Overall", Nothing, Nothing, "mole")
            Dim pressure() As Double = in1.GetProp("Pressure", "Overall", Nothing, "Mixture", Nothing)
            in1.CalcProp(props, phases, "Mixture")
            Dim in1Enthalpy() As Double = in1.GetProp("enthalpy", "Overall", Nothing, "Mixture", "mole")
            temp = Me.Ports(1).connectedObject
            Dim in2 As CapeOpen.ICapeThermoMaterialObject = temp.Duplicate()
            If temp.GetType().IsCOMObject Then
                System.Runtime.InteropServices.Marshal.ReleaseComObject(temp)
            End If
            Dim in2Comps() As String = in2.ComponentIds
            Dim in2Flow() As Double = in2.GetProp("flow", "Overall", Nothing, Nothing, "mole")
            Dim pressure2() As Double = in2.GetProp("Pressure", "Overall", Nothing, "Mixture", Nothing)
            in2.CalcProp(props, phases, "Mixture")
            Dim in2Enthalpy() As Double = in2.GetProp("enthalpy", "Overall", Nothing, "Mixture", "mole")
            Dim outPort As CapeOpen.ICapeThermoMaterialObject = Me.Ports(2).connectedObject
            Dim values(0) As Double
            values(0) = (in1Enthalpy(0) * in1Flow(0) + in2Enthalpy(0) * in2Flow(0)) / (in1Flow(0) + in2Flow(0))
            outPort.SetProp("enthalpy", "Overall", Nothing, "Mixture", "mole", values)
            If pressure(0) > pressure2(0) Then
                pressure(0) = pressure2(0)
            End If
            values(0) = pressure(0) - Me.Parameters(0).value
            outPort.SetProp("Pressure", "Overall", Nothing, Nothing, Nothing, values)
            ReDim values(in1Comps.Length - 1)
            Dim i As Integer
            For i = 0 To in1Comps.Length - 1
                values(i) = in1Flow(i) + in2Flow(i)
            Next i
            outPort.SetProp("flow", "Overall", in1Comps, Nothing, "mole", values)
            outPort.CalcEquilibrium("PH", Nothing)
            If in1.GetType().IsCOMObject Then
                System.Runtime.InteropServices.Marshal.ReleaseComObject(in1)
            End If
            If in2.GetType().IsCOMObject Then
                System.Runtime.InteropServices.Marshal.ReleaseComObject(in2)
            End If
        Catch p_Ex As System.Exception
            System.Windows.Forms.MessageBox.Show(p_Ex.ToString())
        End Try

    End Sub

End Class
