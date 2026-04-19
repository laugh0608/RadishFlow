Public Class MixerExample11
    Inherits CapeOpen.CUnitBase
    Implements CapeOpen.ICapeUnitReport


    Dim calcReport As String

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
    '''         Me.calcReport = String.Empty
    '''     ' Log a message using the simulation context (pop-up message commented out.
    '''         If Not (Me.SimulationContext Is Nothing) Then
    '''             CType(Me.SimulationContext, CapeOpen.ICapeDiagnostic).LogMessage("Starting Mixer Calculation")
    '''         End If
    '''         Me.calcReport = String.Concat(Me.calcReport, "Starting Mixer Calculation", System.Environment.NewLine)
    '''     ' Get the material Object from Port 0.
    '''     Dim in1 As CapeOpen.ICapeThermoMaterial
    '''     Dim in2 As CapeOpen.ICapeThermoMaterial
    '''     Dim output As CapeOpen.ICapeThermoMaterial
    '''     Dim temp As Object
    '''     Dim comps As Object
    '''     Dim forms As Object
    '''     Dim names As Object
    '''     Dim bTemp As Object
    '''     Dim molWts As Object
    '''     Dim casNos As Object
    '''     Dim compIds1 As String()
    '''     Dim compIds2 As String()
    '''     Dim compIds3 As String()
    '''         Try
    ''' 
    '''       temp = Me.Ports(0).connectedObject
    '''             in1 = CType(temp, CapeOpen.ICapeThermoMaterial).CreateMaterial()
    '''             in1.CopyFromMaterial(temp)
    '''             If (temp.GetType().IsCOMObject) Then
    '''                 System.Runtime.InteropServices.Marshal.ReleaseComObject(temp)
    '''             End If
    '''             CType(in1, CapeOpen.ICapeThermoCompounds).GetCompoundList(comps, forms, names, bTemp, molWts, casNos)
    '''             compIds1 = comps
    '''             temp = Me.Ports(1).connectedObject
    '''             in2 = CType(temp, CapeOpen.ICapeThermoMaterial).CreateMaterial()
    '''             in2.CopyFromMaterial(temp)
    '''             If (temp.GetType().IsCOMObject) Then
    '''                 System.Runtime.InteropServices.Marshal.ReleaseComObject(temp)
    '''             End If
    '''             CType(in2, CapeOpen.ICapeThermoCompounds).GetCompoundList(comps, forms, names, bTemp, molWts, casNos)
    '''             compIds2 = comps
    '''             output = Me.Ports(2).connectedObject
    '''             CType(output, CapeOpen.ICapeThermoCompounds).GetCompoundList(comps, forms, names, bTemp, molWts, casNos)
    '''             compIds3 = comps
    '''         Catch p_Ex As System.Exception
    '''     Dim ex As CapeOpen.CapeInvalidOperationException = New CapeOpen.CapeInvalidOperationException("Material object does not support CAPE-OPEN Thermodynamics 1.1.", p_Ex)
    '''             Me.calcReport = String.Concat(Me.calcReport, "Material object does not support CAPE-OPEN Thermodynamics 1.1.", System.Environment.NewLine)
    '''             Me.throwException(ex)
    '''         End Try
    ''' 
    '''    Dim l1 As Integer = compIds1.Length
    '''     Dim l2 As Integer = compIds2.Length
    '''     Dim l3 As Integer = compIds3.Length
    '''         If (l1 &lt;&gt; l2) Then
    '''             Me.calcReport = String.Concat(Me.calcReport, "Compounds in imlet materials do not match.", System.Environment.NewLine)
    '''             Throw New CapeOpen.CapeInvalidOperationException("Compounds in imlet materials do not match.")
    '''         End If
    '''         If (l1 &lt;&gt; l3) Then
    '''             Me.calcReport = String.Concat(Me.calcReport, "Compounds in imlet materials does not match outlet material.", System.Environment.NewLine)
    '''             Throw New CapeOpen.CapeInvalidOperationException("Compounds in imlet materials does not match outlet material.")
    '''         End If
    '''     Dim press1 As Double()
    '''     Dim press2 As Double()
    '''     Dim flowOut As Double()
    '''     Dim flow2 As Double()
    '''     Dim enthalpy As Double()
    '''     Dim phases As String()
    '''     Dim props As String() = {"enthalpy"}
    '''     Dim overall As String() = {"Overall"}
    '''     Dim obj As Object
    '''     Dim obj1 As Object
    '''     Dim obj2 As Object
    '''     Dim obj3 As Object
    '''     Dim totalFlow As Double = 0
    '''         For i As Integer = 0 To l3
    '''             If compIds1(i) &lt;&gt; compIds2(i) Then
    '''                 Me.calcReport = String.Concat(Me.calcReport, "Compounds in imlet materials do not match.", System.Environment.NewLine)
    '''                 Throw New CapeOpen.CapeInvalidOperationException("Compounds in imlet materials do not match.")
    '''             ElseIf compIds1(i) &lt;&gt; compIds3(i) Then
    '''                 Me.calcReport = String.Concat(Me.calcReport, "Compounds in imlet materials does not match outlet material.", System.Environment.NewLine)
    '''                 Throw New CapeOpen.CapeInvalidOperationException("Compounds in imlet materials does not mAtch outlet material.")
    '''             End If
    '''         Next
    ''' 
    '''    Try
    '''             in1.GetOverallProp("flow", "mole", flowOut)
    '''             For Each flow As Double In flowOut
    '''                 totalFlow = totalFlow + flow
    '''             Next
    '''             in1.GetOverallProp("pressure", "", press1)
    '''             CType(in1, CapeOpen.ICapeThermoPhases).GetPhaseList(phases, obj1, obj2)
    '''             For Each phase As String In phases
    '''                 CType(in1, CapeOpen.ICapeThermoPropertyRoutine).CalcSinglePhaseProp(props, phase)
    '''     Dim enth As Double()
    '''     Dim fract As Double()
    '''                 in1.GetSinglePhaseProp("enthalpy", phase, "mole", enth)
    '''                 in1.GetSinglePhaseProp("phaseFraction", phase, "mole", fract)
    '''                 enthalpy(0) = enthalpy(0) + totalFlow * fract(0) * enth(0)
    '''             Next
    ''' 
    '''        Catch p_Ex As System.Exception
    '''     Dim user As CapeOpen.ECapeUser = in1
    ''' 
    '''            Me.calcReport = String.Concat(Me.calcReport, user.description, System.Environment.NewLine)
    '''             Me.throwException(user)
    '''         End Try
    '''         Try
    '''             in2.GetOverallProp("flow", "mole", flow2)
    '''             totalFlow = 0
    '''             For Each flow As Double In flow2
    '''                 totalFlow = totalFlow + flow
    '''             Next
    '''             in2.GetOverallProp("pressure", "", press2)
    '''             CType(in2, CapeOpen.ICapeThermoPhases).GetPhaseList(phases, obj1, obj2)
    '''             For Each phase As String In phases
    '''                 CType(in2, CapeOpen.ICapeThermoPropertyRoutine).CalcSinglePhaseProp(props, phase)
    '''     Dim enth As Double()
    '''     Dim fract As Double()
    '''                 in1.GetSinglePhaseProp("enthalpy", phase, "mole", enth)
    '''                 in1.GetSinglePhaseProp("phaseFraction", phase, "mole", fract)
    '''                 enthalpy(0) = enthalpy(0) + totalFlow * fract(0) * enth(0)
    '''             Next
    ''' 
    '''        Catch p_Ex As System.Exception
    '''     Dim user As CapeOpen.ECapeUser = in1
    '''             Me.calcReport = String.Concat(Me.calcReport, user.description, System.Environment.NewLine)
    '''             Me.throwException(user)
    '''         End Try
    '''         For i As Integer = 0 To l3
    '''             flowOut(i) = flowOut(i) + flow2(i)
    '''         Next
    '''         totalFlow = 0
    '''         For Each flow As Double In flowOut
    '''             totalFlow = totalFlow + flow
    '''         Next
    '''         Try
    '''             output.SetOverallProp("flow", "mole", flowOut)
    '''     Dim press As Double() = press1
    '''             If press1(0) > press2(0) Then
    '''                 press(0) = press2(0)
    '''             End If
    '''     Dim pressdrop As Double = CType(Me.Parameters("PressureDrop"), CapeOpen.CRealParameter).SIValue
    '''             press(0) = press(0) - pressdrop
    '''             output.SetOverallProp("pressure", "", press)
    '''             output.GetOverallProp("flow", "mole", flowOut)
    '''             enthalpy(0) = enthalpy(0) / totalFlow
    '''             output.SetOverallProp("enthalpy", "mole", enthalpy)
    '''             Me.calcReport = String.Concat(Me.calcReport, "The outlet pressure is: ", press(0).ToString(), "Pa", System.Environment.NewLine)
    '''     Dim status As Integer() = {0, 0}
    ''' 
    '''            output.SetPresentPhases(phases, status)
    '''             Me.calcReport = String.Concat(Me.calcReport, "The outlet ethalpy is: ", enthalpy(0).ToString(), "J/Mole", System.Environment.NewLine)
    '''     Dim spec1 As String() = {"pressure", String.Empty, "Overall"}
    '''     Dim spec2 As String() = {"enthalpy", String.Empty, "Overall"}
    '''             CType(output, CapeOpen.ICapeThermoEquilibriumRoutine).CalcEquilibrium(spec1, spec2, "unspecified")
    '''             Me.calcReport = String.Concat(Me.calcReport, "Calculated pressure-enthalpy flash", System.Environment.NewLine)
    '''         Catch p_Ex As System.Exception
    '''     Dim user As CapeOpen.ECapeUser = output
    '''             Me.calcReport = String.Concat(Me.calcReport, user.description, System.Environment.NewLine)
    '''             Me.throwException(user)
    '''         End Try
    '''         If in1.GetType().IsCOMObject Then
    '''             System.Runtime.InteropServices.Marshal.ReleaseComObject(in1)
    '''         End If
    '''         If in2.GetType().IsCOMObject Then
    '''            System.Runtime.InteropServices.Marshal.ReleaseComObject(in2)
    '''         End If
    ''' 
    '''    ' Log the end of the calculation.
    '''         If Not (Me.SimulationContext Is Nothing) Then
    '''             CType(Me.SimulationContext, CapeOpen.ICapeDiagnostic).LogMessage("Ending Mixer Calculation")
    '''         End If
    '''     'If Not (Me.SimulationContext Is Nothing) Then
    '''     '    CType(Me.SimulationContext, CapeOpen.ICapeDiagnostic).LogMessage("Ending Mixer Calculation")
    '''     'End If
    '''         Me.calcReport = String.Concat(Me.calcReport, "Ending Mixer Calculation")
    ''' 
    '''     End Sub
    ''' </code>
    ''' </example>
    ''' <seealso cref="CapeOpen.ICapeThermoMaterial"/>
    ''' <seealso cref="CapeOpen.ICapeThermoCompounds"/>
    ''' <seealso cref="CapeOpen.ICapeThermoPhases"/>
    ''' <seealso cref="CapeOpen.ICapeThermoPropertyRoutine"/>
    ''' <seealso cref="CapeOpen.ICapeThermoEquilibriumRoutine"/>
    ''' <seealso cref="CapeOpen.COMExceptionHandler"/>
    Public Overrides Sub Calculate()
        Me.calcReport = String.Empty
        ' Log a message using the simulation context (pop-up message commented out.
        If Not (Me.SimulationContext Is Nothing) Then
            CType(Me.SimulationContext, CapeOpen.ICapeDiagnostic).LogMessage("Starting Mixer Calculation")
        End If
        Me.calcReport = String.Concat(Me.calcReport, "Starting Mixer Calculation", System.Environment.NewLine)
        ' Get the material Object from Port 0.
        Dim in1 As CapeOpen.ICapeThermoMaterial
        Dim in2 As CapeOpen.ICapeThermoMaterial
        Dim output As CapeOpen.ICapeThermoMaterial
        Dim temp As Object
        Dim comps As Object
        Dim forms As Object
        Dim names As Object
        Dim bTemp As Object
        Dim molWts As Object
        Dim casNos As Object
        Dim compIds1 As String()
        Dim compIds2 As String()
        Dim compIds3 As String()
        Try
            temp = Me.Ports(0).connectedObject
            in1 = CType(temp, CapeOpen.ICapeThermoMaterial).CreateMaterial()
            in1.CopyFromMaterial(temp)
            If (temp.GetType().IsCOMObject) Then
                System.Runtime.InteropServices.Marshal.ReleaseComObject(temp)
            End If
            CType(in1, CapeOpen.ICapeThermoCompounds).GetCompoundList(comps, forms, names, bTemp, molWts, casNos)
            compIds1 = comps
            temp = Me.Ports(1).connectedObject
            in2 = CType(temp, CapeOpen.ICapeThermoMaterial).CreateMaterial()
            in2.CopyFromMaterial(temp)
            If (temp.GetType().IsCOMObject) Then
                System.Runtime.InteropServices.Marshal.ReleaseComObject(temp)
            End If
            CType(in2, CapeOpen.ICapeThermoCompounds).GetCompoundList(comps, forms, names, bTemp, molWts, casNos)
            compIds2 = comps
            output = Me.Ports(2).connectedObject
            CType(output, CapeOpen.ICapeThermoCompounds).GetCompoundList(comps, forms, names, bTemp, molWts, casNos)
            compIds3 = comps
        Catch p_Ex As System.Exception
            Dim ex As CapeOpen.CapeInvalidOperationException = New CapeOpen.CapeInvalidOperationException("Material object does not support CAPE-OPEN Thermodynamics 1.1.", p_Ex)
            Me.calcReport = String.Concat(Me.calcReport, "Material object does not support CAPE-OPEN Thermodynamics 1.1.", System.Environment.NewLine)
            Me.throwException(ex)
        End Try

        Dim l1 As Integer = compIds1.Length
        Dim l2 As Integer = compIds2.Length
        Dim l3 As Integer = compIds3.Length
        If (l1 <> l2) Then
            Me.calcReport = String.Concat(Me.calcReport, "Compounds in imlet materials do not match.", System.Environment.NewLine)
            Throw New CapeOpen.CapeInvalidOperationException("Compounds in imlet materials do not match.")
        End If
        If (l1 <> l3) Then
            Me.calcReport = String.Concat(Me.calcReport, "Compounds in imlet materials does not match outlet material.", System.Environment.NewLine)
            Throw New CapeOpen.CapeInvalidOperationException("Compounds in imlet materials does not match outlet material.")
        End If
        Dim press1 As Double()
        Dim press2 As Double()
        Dim flowOut As Double()
        Dim flow2 As Double()
        Dim enthalpy As Double() = {0}
        Dim phases As String()
        Dim props As String() = {"enthalpy"}
        Dim overall As String() = {"Overall"}
        Dim obj1 As Object
        Dim obj2 As Object
        Dim totalFlow As Double = 0
        For i As Integer = 0 To l3 - 1
            If compIds1(i) <> compIds2(i) Then
                Me.calcReport = String.Concat(Me.calcReport, "Compounds in imlet materials do not match.", System.Environment.NewLine)
                Throw New CapeOpen.CapeInvalidOperationException("Compounds in imlet materials do not match.")
            ElseIf compIds1(i) <> compIds3(i) Then
                Me.calcReport = String.Concat(Me.calcReport, "Compounds in imlet materials does not match outlet material.", System.Environment.NewLine)
                Throw New CapeOpen.CapeInvalidOperationException("Compounds in imlet materials does not mAtch outlet material.")
            End If
        Next

        Try
            in1.GetOverallProp("flow", "mole", flowOut)
            For Each flow As Double In flowOut
                totalFlow = totalFlow + flow
            Next
            in1.GetOverallProp("pressure", "", press1)
            CType(in1, CapeOpen.ICapeThermoPhases).GetPhaseList(phases, obj1, obj2)
            For Each phase As String In phases
                CType(in1, CapeOpen.ICapeThermoPropertyRoutine).CalcSinglePhaseProp(props, phase)
                Dim enth As Double()
                Dim fract As Double()
                in1.GetSinglePhaseProp("enthalpy", phase, "mole", enth)
                in1.GetSinglePhaseProp("phaseFraction", phase, "mole", fract)
                enthalpy(0) = enthalpy(0) + totalFlow * fract(0) * enth(0)
            Next

        Catch p_Ex As System.Exception
            Dim user As CapeOpen.ECapeUser = in1
            Me.calcReport = String.Concat(Me.calcReport, user.description, System.Environment.NewLine)
            Me.throwException(user)
        End Try
        Try
            in2.GetOverallProp("flow", "mole", flow2)
            totalFlow = 0
            For Each flow As Double In flow2
                totalFlow = totalFlow + flow
            Next
            in2.GetOverallProp("pressure", "", press2)
            CType(in2, CapeOpen.ICapeThermoPhases).GetPhaseList(phases, obj1, obj2)
            For Each phase As String In phases
                CType(in2, CapeOpen.ICapeThermoPropertyRoutine).CalcSinglePhaseProp(props, phase)
                Dim enth As Double()
                Dim fract As Double()
                in2.GetSinglePhaseProp("enthalpy", phase, "mole", enth)
                in2.GetSinglePhaseProp("phaseFraction", phase, "mole", fract)
                enthalpy(0) = enthalpy(0) + totalFlow * fract(0) * enth(0)
            Next

        Catch p_Ex As System.Exception
            Dim user As CapeOpen.ECapeUser = in1
            Me.calcReport = String.Concat(Me.calcReport, user.description, System.Environment.NewLine)
            Me.throwException(user)
        End Try
        For i As Integer = 0 To l3 - 1
            flowOut(i) = flowOut(i) + flow2(i)
        Next
        totalFlow = 0
        For Each flow As Double In flowOut
            totalFlow = totalFlow + flow
        Next
        Try
            output.SetOverallProp("flow", "mole", flowOut)
            Dim press As Double() = press1
            If press1(0) > press2(0) Then
                press(0) = press2(0)
            End If
            Dim pressdrop As Double = CType(Me.Parameters(0), CapeOpen.CRealParameter).SIValue
            press(0) = press(0) - pressdrop
            output.SetOverallProp("pressure", "", press)
            output.GetOverallProp("flow", "mole", flowOut)
            enthalpy(0) = enthalpy(0) / totalFlow
            output.SetOverallProp("enthalpy", "mole", enthalpy)
            Me.calcReport = String.Concat(Me.calcReport, "The outlet pressure is: ", press(0).ToString(), "Pa", System.Environment.NewLine)
            Dim status As Integer() = {0, 0}
            output.SetPresentPhases(phases, status)
            Me.calcReport = String.Concat(Me.calcReport, "The outlet ethalpy is: ", enthalpy(0).ToString(), "J/Mole", System.Environment.NewLine)
            Dim spec1 As String() = {"pressure", String.Empty, "Overall"}
            Dim spec2 As String() = {"enthalpy", String.Empty, "Overall"}
            CType(output, CapeOpen.ICapeThermoEquilibriumRoutine).CalcEquilibrium(spec1, spec2, "unspecified")
            Me.calcReport = String.Concat(Me.calcReport, "Calculated pressure-enthalpy flash", System.Environment.NewLine)
        Catch p_Ex As System.Exception
            Dim user As CapeOpen.ECapeUser = output
            Me.calcReport = String.Concat(Me.calcReport, user.description, System.Environment.NewLine)
            Me.throwException(user)
        End Try
        If in1.GetType().IsCOMObject Then
            System.Runtime.InteropServices.Marshal.ReleaseComObject(in1)
        End If
        If in2.GetType().IsCOMObject Then
            System.Runtime.InteropServices.Marshal.ReleaseComObject(in2)
        End If

        ' Log the end of the calculation.
        If Not (Me.SimulationContext Is Nothing) Then
            CType(Me.SimulationContext, CapeOpen.ICapeDiagnostic).LogMessage("Ending Mixer Calculation")
        End If
        'If Not (Me.SimulationContext Is Nothing) Then
        '    CType(Me.SimulationContext, CapeOpen.ICapeDiagnostic).LogMessage("Ending Mixer Calculation")
        'End If
        Me.calcReport = String.Concat(Me.calcReport, "Ending Mixer Calculation")

    End Sub

    ''' <summary>
    '''	Produces the active report for the Mixer operation.
    ''' </summary>
    ''' <remarks>
    '''	Me method is declared as an explicit override of the <seealso cref="CapeOpen.ICapeUnitReport.ProduceReport"/> method. 
    ''' The report generated is based upon the selected report property of Me unit. If "Calculation Report" is
    ''' the selected report, the report string generated during the calucation operation is returned.
    ''' </remarks>
    ''' <example>
    ''' <code>
    ''' void CapeOpen.ICapeUnitReport.ProduceReport(String message)
    ''' {
    '''     string text = string.Empty
    '''     if (Me.selectedReport == "Calculation Report") text = Me.calcReport
    '''     else (Me as CapeOpen.CUnitBase).ProduceReport(text)
    '''     message = text
    ''' }
    ''' </code>
    ''' </example>
    ''' <param name = "message">String containing the text for the currently selected report.</param>
    ''' <exception cref= "CapeOpen.ECapeUnknown">The error to be raised when other error(s),  specified for Me operation, are not suitable.</exception>
    ''' <exception cref= "CapeOpen.ECapeNoImpl">ECapeNoImpl</exception>
    Public Sub ProduceReport(ByRef message As String) Implements CapeOpen.ICapeUnitReport.ProduceReport
        Dim text As String = String.Empty
        If (Me.selectedReport = "Calculation Report") Then
            text = Me.calcReport
        Else
            CType(Me, CapeOpen.CUnitBase).ProduceReport(text)
        End If
        message = text
    End Sub

End Class
