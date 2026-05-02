using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace Mixer
{
    class MixerExample11 : CapeOpen.CUnitBase, CapeOpen.ICapeUnitReport
    {
        String calcReport;

        /// <summary>
        /// Creates an instance of the CMixerExample unit operation.
        /// </summary>
        /// <remarks>
        /// This constructor demonstates the addition of a <seealso cref = "CapeOpen.CBoolParameter"/>,
        /// <seealso cref = "CapeOpen.CIntParameter"/>, <seealso cref = "CapeOpen.COptionParameter"/>,
        /// and a <seealso cref = "CapeOpen.CRealParameter"/> parameter to the parameter collection.
        /// In addition, the mixer unit has three <seealso cref = "CapeOpen.CUnitPort"/> ports
        /// added to the port collection. See the documentation for the 
        /// <seealso cref = "Calculate"/> method for details on its implementation.
        /// </remarks>
        /// <example>
        /// An example of how to create a unit operation. Parameter and port objects are created 
        /// and added the their respective collections. Ports are implemented by the <see cref="CapeOpen.CUnitPort"/> 
        /// class and are placed in the Port Collection. Parameters are added to the Parameter Collection 
        /// class. The available parameter classes are <see cref="CapeOpen.CRealParameter"/>, <see cref="CapeOpen.CRealParameter"/>, 
        /// <see cref="CapeOpen.CRealParameter"/>, and <see cref="CapeOpen.CRealParameter"/>.
        /// <code>
        /// public MixerExample()
        /// {
        ///    // Add Ports using the CUnitPort constructor.
        ///    this.Ports.Add(new CapeOpen.CUnitPort("Inlet Port1", "Test Inlet Port1", CapeOpen.CapePortDirection.CAPE_INLET, CapeOpen.CapePortType.CAPE_MATERIAL));
        ///    this.Ports.Add(new CapeOpen.CUnitPort("Inlet Port1", "Test Inlet Port2", CapeOpen.CapePortDirection.CAPE_INLET, CapeOpen.CapePortType.CAPE_MATERIAL));
        ///    this.Ports.Add(new CapeOpen.CUnitPort("Inlet Port1", "Test Outlet Port", CapeOpen.CapePortDirection.CAPE_OUTLET, CapeOpen.CapePortType.CAPE_MATERIAL));
        ///
        ///    // Add a real valued parameter using the CRealParameter  constructor.
        ///    this.Parameters.Add(new CapeOpen.CRealParameter("PressureDrop", "Drop in pressure between the outlet from the mixer and the pressure of the lower pressure inlet.", 0.0, 0.0, 0.0, 100000000.0, CapeOpen.CapeParamMode.CAPE_INPUT, "Pa"));
        ///
        ///    // Add a real valued parameter using the CIntParameter  constructor.
        ///    this.Parameters.Add(new CapeOpen.CIntParameter("Parameter2", 12, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT));
        ///
        ///    // Add a real valued parameter using the CBoolParameter  constructor.
        ///    this.Parameters.Add(new CapeOpen.CBoolParameter("Parameter3", false, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT));
        ///
        ///    // Create an array of strings for the option parameter restricted value list.
        ///    string[] options = { "Test Value", "Another Value" };
        ///
        ///    // Add a string valued parameter using the COptionParameter constructor.
        ///    this.Parameters.Add(new CapeOpen.COptionParameter("Parameter4", "OptionParameter", "Test Value", "Test Value", options, true, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT));
        ///
        ///    // Add an available report.
        ///    this.Reports.Add("Calculation Report");
        /// }
        /// </code>
        /// </example>
        public MixerExample11()
        {
            // Add Ports using the CUnitPort constructor.
            this.Ports.Add(new CapeOpen.CUnitPort("Inlet Port1", "Test Inlet Port1", CapeOpen.CapePortDirection.CAPE_INLET, CapeOpen.CapePortType.CAPE_MATERIAL));
            this.Ports.Add(new CapeOpen.CUnitPort("Inlet Port2", "Test Inlet Port2", CapeOpen.CapePortDirection.CAPE_INLET, CapeOpen.CapePortType.CAPE_MATERIAL));
            this.Ports.Add(new CapeOpen.CUnitPort("Outlet Port", "Test Outlet Port", CapeOpen.CapePortDirection.CAPE_OUTLET, CapeOpen.CapePortType.CAPE_MATERIAL));

            // Add a real valued parameter using the CRealParameter  constructor.
            this.Parameters.Add(new CapeOpen.CRealParameter("PressureDrop", "Drop in pressure between the outlet from the mixer and the pressure of the lower pressure inlet.", 0.0, 0.0, 0.0, 100000000.0, CapeOpen.CapeParamMode.CAPE_INPUT, "Pa"));

            // Add a real valued parameter using the CIntParameter  constructor.
            this.Parameters.Add(new CapeOpen.CIntParameter("Parameter2", 12, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT));

            // Add a real valued parameter using the CBoolParameter  constructor.
            this.Parameters.Add(new CapeOpen.CBoolParameter("Parameter3", false, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT));

            // Create an array of strings for the option parameter restricted value list.
            string[] options = { "Test Value", "Another Value" };

            // Add a string valued parameter using the COptionParameter constructor.
            this.Parameters.Add(new CapeOpen.COptionParameter("Parameter4", "OptionParameter", "Test Value", "Test Value", options, true, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT));

            // Add an available report.
            this.Reports.Add("Calculation Report");
        }

        /// <summary>
        /// Initialization member required for CAPE-OPEN compliance.
        /// </summary>
        /// <remarks>
        /// The initialization tasks are accomplished in the class construcotr, so 
        /// this method is empty.
        /// </remarks>
        public override void Initialize()
        {
        }


        /// <summary>
        /// Calculation method for the CMixerExample110 unit operation.
        /// </summary>
        /// <remarks>
        /// A mixer unit operation combined the material flows from two inlet ports into the flow of a single outlet port.
        /// In order to do this calculation, the mixer unit obtains flow information from each inlet port,
        /// the adds the flows to obtain the flow of the outlet port. In the case of the mixer below, it is assumed that the
        /// components are the same in each material object and that the components are listed in the same order.
        /// After the combined flow is calculated at the values set to the outlet port, along with the 
        /// enthalpy of the stream calculated from an energy balance and the pressure determined from
        /// the inlet pressures, the outlet stream can be flahsed to determine equilibrium conditions.
        /// The last task is releasing any duplicate material objects obtained.
        /// </remarks>
        /// <example>
        /// <para>An example of how to calculate a unit operation. This method obtains material objects 
        /// from each of ports using the <seealso cref = "CapeOpen.CPortCollection"/> class. The <seealso cref = "CapeOpen.ICapeThermoMaterial"/>
        /// interface is used to obtain the names using the CompIds() method, flows of each of the 
        /// components present in the material object and overall pressure obtained using the 
        /// <seealso cref = "CapeOpen.ICapeThermoMaterial.GetOverallProp"/> method. The enthaply of 
        /// each phase in the inlet materials is calculated using <seealso cref = "CapeOpen.ICapeThermoPropertyRoutine.CalcSinglePhaseProp"/>
        /// method. The inlet enthalpy was multiplied by the phase flow (phase fraction * overall flow) 
        /// which was summed to determine the outlet stream enthalpy. 
        /// The output pressure from the lower of the two streams' pressure and pressure drop parameter. 
        /// Lastly, the results of the 
        /// calculations are applied to the output material object using the <seealso cref = "CapeOpen.ICapeThermoMaterial.SetOverallProp"/> 
        /// method. The last method of the calculate method is to call the material's 
        /// <seealso cref = "CapeOpen.ICapeThermoEquilibriumRoutine.CalcEquilibrium"/> method.</para>
        /// <para>
        /// A calculation report is generated by this method that is made avalable through the
        /// <seealso cref = "CapeOpen.ICapeUnitReport.ProduceReport"/> method.
        /// </para>
        /// <para>
        /// In this case, the inlet materials need to be released. This is accomplished using the
        /// <seealso cref = "System.Runtime.InteropServices.Marshal.ReleaseComObject"/> method.
        /// Using this method to release the outlet material object would result in an null object reference error.
        /// </para>
        /// <para>
        /// The method also documents use of the <seealso cref = "CapeOpen.CCapeObject.throwException"/> method to provide
        /// CAPE-OPEN compliant error handling.
        /// </para>
        /// <code>
        /// public override void Calculate()
        /// {
        ///    this.calcReport = String.Empty;
        ///    // Log a message using the simulation context (pop-up message commented out.
        ///    if (this.SimulationContext != null)
        ///        (this.SimulationContext as CapeOpen.ICapeDiagnostic).LogMessage("Starting Mixer Calculation");
        ///    this.calcReport = String.Concat(this.calcReport, "Starting Mixer Calculation", System.Environment.NewLine);
        ///    // Get the material Object from Port 0.
        ///    CapeOpen.ICapeThermoMaterial in1 = null;
        ///    CapeOpen.ICapeThermoMaterial in2 = null;
        ///    CapeOpen.ICapeThermoMaterial output = null;
        ///    CapeOpen.ICapeThermoMaterial temp = null;
        ///    Object comps = null;
        ///    Object forms = null;
        ///    Object names = null;
        ///    Object bTemp = null;
        ///    Object molWts = null;
        ///    Object casNos = null;
        ///    string[] compIds1 = null;
        ///    string[] compIds2 = null;
        ///    string[] compIds3 = null;
        ///    try
        ///    {
        ///        temp = (this.Ports[0].connectedObject as CapeOpen.ICapeThermoMaterial);
        ///        in1 = temp.CreateMaterial() as CapeOpen.ICapeThermoMaterial;
        ///        //in1.CopyFromMaterial(temp);
        ///        if (temp.GetType().IsCOMObject) System.Runtime.InteropServices.Marshal.ReleaseComObject(temp);
        ///        (in1 as CapeOpen.ICapeThermoCompounds).GetCompoundList(ref comps, ref forms, ref names, ref bTemp, ref molWts, ref casNos);
        ///        compIds1 = comps as string[];
        ///        temp = this.Ports[1].connectedObject as CapeOpen.ICapeThermoMaterial;
        ///        in2 = temp.CreateMaterial() as CapeOpen.ICapeThermoMaterial;
        ///        //in2.CopyFromMaterial(temp);
        ///        if (temp.GetType().IsCOMObject) System.Runtime.InteropServices.Marshal.ReleaseComObject(temp);
        ///        (in2 as CapeOpen.ICapeThermoCompounds).GetCompoundList(ref comps, ref forms, ref names, ref bTemp, ref molWts, ref casNos);
        ///        compIds2 = comps as string[];
        ///        output = this.Ports[2].connectedObject as CapeOpen.ICapeThermoMaterial;
        ///        (output as CapeOpen.ICapeThermoCompounds).GetCompoundList(ref comps, ref forms, ref names, ref bTemp, ref molWts, ref casNos);
        ///        compIds3 = comps as string[];
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        CapeOpen.CapeInvalidOperationException ex = new CapeOpen.CapeInvalidOperationException("Material object does not support CAPE-OPEN Thermodynamics 1.1.", p_Ex);
        ///        this.calcReport = String.Concat(this.calcReport, "Material object does not support CAPE-OPEN Thermodynamics 1.1.", System.Environment.NewLine);
        ///        this.throwException(ex as CapeOpen.ECapeUser);
        ///    }
        ///    int l1 = 0;
        ///    int l2 = 0;
        ///    int l3 = 0;
        ///    l1 = compIds1.Length;
        ///    l2 = compIds2.Length;
        ///    l3 = compIds3.Length;
        ///    if (l1 != l2)
        ///    {
        ///        this.calcReport = string.Concat(this.calcReport, "Compounds in imlet materials do not match.", System.Environment.NewLine);
        ///        throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials do not match.");
        ///    }
        ///    if (l1 != l3)
        ///    {
        ///        this.calcReport = string.Concat(this.calcReport, "Compounds in imlet materials does not match outlet material.", System.Environment.NewLine);
        ///        throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials does not match outlet material.");
        ///    }
        ///    for (int i = 0; i &lt; l3; i++)
        ///    {
        ///        if ((String.Compare(compIds1[i], compIds2[i])) != 0)
        ///        {
        ///            this.calcReport = string.Concat(this.calcReport, "Compounds in imlet materials do not match.", System.Environment.NewLine);
        ///            throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials do not match.");
        ///        }
        ///
        ///        if (String.Compare(compIds1[i], compIds3[i]) != 0)
        ///        {
        ///            this.calcReport = string.Concat(this.calcReport, "Compounds in imlet materials does not match outlet material.", System.Environment.NewLine);
        ///            throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials does not mAtch outlet material.");
        ///        }
        ///    }
        /// 
        ///    double[] press1 = null;
        ///    double[] press2 = null;
        ///    double[] flow1 = null;
        ///    double[] flow2 = null;
        ///    double[] flow3 = new double[l3];
        ///    string[] props = { "enthalpy" };
        ///    string[] overall = { "Overall" };
        ///    double[] enthalpy = { 0 };
        ///    Object obj = null;
        ///    Object obj1 = null;
        ///    Object obj2 = null;
        ///    Object obj3 = null;
        ///    string[] phases = null;
        ///    CapeOpen.ICapeThermoEquilibriumRoutine eqRoutine;
        ///    try
        ///    {
        ///        in1.GetOverallProp("flow", "mole", ref obj);
        ///        flow1 = obj as double[];
        ///        double totalFlow = 0;
        ///        foreach (double flow in flow1)
        ///        {
        ///            totalFlow = totalFlow + flow;
        ///        }
        ///        in1.GetOverallProp("pressure", "", ref obj);
        ///        press1 = obj as double[];
        ///        CapeOpen.ICapeThermoPropertyRoutine p_Calc = in1 as CapeOpen.ICapeThermoPropertyRoutine;
        ///        CapeOpen.ICapeThermoPhases phaseList = in1 as CapeOpen.ICapeThermoPhases;
        ///        phaseList.GetPhaseList(ref obj1, ref obj2, ref obj3);
        ///        phases = obj1 as string[];
        ///        foreach (String phase in phases)
        ///        {
        ///            p_Calc.CalcSinglePhaseProp(props, phase);
        ///            in1.GetSinglePhaseProp("enthalpy", phase, "mole", ref obj);
        ///            double[] enth = obj as double[];
        ///            in1.GetSinglePhaseProp("phaseFraction", phase, "mole", ref obj);
        ///            double[] fract = obj as double[];
        ///            enthalpy[0] = enthalpy[0] + totalFlow * fract[0] * enth[0];
        ///        }
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        CapeOpen.ECapeUser user = in1 as CapeOpen.ECapeUser;
        ///        this.calcReport = string.Concat(this.calcReport, user.description, System.Environment.NewLine);
        ///        this.throwException(user);
        ///    }
        ///    try
        ///    {
        ///        in2.GetOverallProp("flow", "mole", ref obj);
        ///        flow2 = obj as double[];
        ///        double totalFlow = 0;
        ///        foreach (double flow in flow2)
        ///        {
        ///            totalFlow = totalFlow + flow;
        ///        }
        ///        in2.GetOverallProp("pressure", "", ref obj);
        ///        press2 = obj as double[];
        ///        CapeOpen.ICapeThermoPropertyRoutine p_Calc = in2 as CapeOpen.ICapeThermoPropertyRoutine;
        ///        CapeOpen.ICapeThermoPhases phaseList = in2 as CapeOpen.ICapeThermoPhases;
        ///        phaseList.GetPhaseList(ref obj1, ref obj2, ref obj3);
        ///        foreach (String phase in phases)
        ///        {
        ///            p_Calc.CalcSinglePhaseProp(props, phase);
        ///            in2.GetSinglePhaseProp("enthalpy", phase, "mole", ref obj);
        ///            double[] enth = obj as double[];
        ///            in2.GetSinglePhaseProp("phaseFraction", phase, "mole", ref obj);
        ///            double[] fract = obj as double[];
        ///            enthalpy[0] = enthalpy[0] + totalFlow * fract[0] * enth[0];
        ///        }
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        CapeOpen.ECapeUser user = in2 as CapeOpen.ECapeUser;
        ///        this.calcReport = string.Concat(this.calcReport, user.description, System.Environment.NewLine);
        ///        this.throwException(user);
        ///    }
        ///    for (int i = 0; i &lt; l3; i++)
        ///    {
        ///        flow3[i] = flow1[i] + flow2[i];
        ///    }
        ///    try
        ///    {
        ///        output.SetOverallProp("flow", "mole", flow3);
        ///        double[] press = new double[1];
        ///        press[0] = press1[0];
        ///        if (press1[0] > press2[0]) press[0] = press2[0];
        ///        double pressdrop = (this.Parameters[0] as CapeOpen.CRealParameter).SIValue;
        ///        press[0] = press[0] - pressdrop;
        ///        output.SetOverallProp("pressure", "", press);
        ///        output.GetOverallProp("totalFlow", "mole", ref obj);
        ///        double[] totalFlow = obj as double[];
        ///        enthalpy[0] = enthalpy[0] / totalFlow[0];
        ///        output.SetOverallProp("enthalpy", "mole", enthalpy);
        ///        this.calcReport = string.Concat(this.calcReport, "The outlet pressure is: ", press[0].ToString(), "Pa", System.Environment.NewLine);
        ///        int[] status = { 0, 0 };
        ///        output.SetPresentPhases(phases, status);
        ///        this.calcReport = string.Concat(this.calcReport, "The outlet ethalpy is: ", enthalpy[0].ToString(), "J/Mole", System.Environment.NewLine);
        ///        eqRoutine = output as CapeOpen.ICapeThermoEquilibriumRoutine;
        ///        string[] spec1 = { "pressure", String.Empty, "Overall" };
        ///        string[] spec2 = { "enthalpy", String.Empty, "Overall" };
        ///        eqRoutine.CalcEquilibrium(spec1, spec2, "unspecified");
        ///        this.calcReport = string.Concat(this.calcReport, "Calculated pressure-enthalpy flash", System.Environment.NewLine);
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        CapeOpen.ECapeUser user = output as CapeOpen.ECapeUser;
        ///        this.calcReport = string.Concat(this.calcReport, user.description, System.Environment.NewLine);
        ///        this.throwException(user);
        ///    }
        ///
        ///    if (in1.GetType().IsCOMObject) System.Runtime.InteropServices.Marshal.ReleaseComObject(in1);
        ///    if (in2.GetType().IsCOMObject) System.Runtime.InteropServices.Marshal.ReleaseComObject(in2);
        ///    // Log the end of the calculation.
        ///    if (this.SimulationContext != null)
        ///        (this.SimulationContext as CapeOpen.ICapeDiagnostic).LogMessage("Ending Mixer Calculation");
        ///    this.calcReport = string.Concat(this.calcReport, "Ending Mixer Calculation");
        ///    //(this.SimulationContext as CapeOpen.ICapeDiagnostic).PopUpMessage("Ending Mixer Calculation");
        /// }
        /// </code>
        /// </example>
        /// <seealso cref="CapeOpen.ICapeThermoMaterial"/>
        /// <seealso cref="CapeOpen.ICapeThermoCompounds"/>
        /// <seealso cref="CapeOpen.ICapeThermoPhases"/>
        /// <seealso cref="CapeOpen.ICapeThermoPropertyRoutine"/>
        /// <seealso cref="CapeOpen.ICapeThermoEquilibriumRoutine"/>
        /// <seealso cref="CapeOpen.COMExceptionHandler"/>
        public override void Calculate()
        {
            this.calcReport = String.Empty;
            // Log a message using the simulation context (pop-up message commented out.
            if (this.SimulationContext != null)
                (this.SimulationContext as CapeOpen.ICapeDiagnostic).LogMessage("Starting Mixer Calculation");
            this.calcReport = String.Concat(this.calcReport, "Starting Mixer Calculation", System.Environment.NewLine);
            // Get the material Object from Port 0.
            CapeOpen.ICapeThermoMaterial in1 = null;
            CapeOpen.ICapeThermoMaterial in2 = null;
            CapeOpen.ICapeThermoMaterial output = null;
            object temp = null;
            Object comps = null;
            Object forms = null;
            Object names = null;
            Object bTemp = null;
            Object molWts = null;
            Object casNos = null;
            string[] compIds1 = null;
            string[] compIds2 = null;
            string[] compIds3 = null;
            try
            {
                temp = this.Ports[0].connectedObject;
                in1 = (temp as CapeOpen.ICapeThermoMaterial).CreateMaterial() as CapeOpen.ICapeThermoMaterial;
                in1.CopyFromMaterial(ref temp);
                if (temp.GetType().IsCOMObject) System.Runtime.InteropServices.Marshal.ReleaseComObject(temp);
                (in1 as CapeOpen.ICapeThermoCompounds).GetCompoundList(ref comps, ref forms, ref names, ref bTemp, ref molWts, ref casNos);
                compIds1 = comps as string[];
                temp = this.Ports[1].connectedObject as CapeOpen.ICapeThermoMaterial;
                in2 = (temp as CapeOpen.ICapeThermoMaterial).CreateMaterial() as CapeOpen.ICapeThermoMaterial;
                in2.CopyFromMaterial(ref temp);
                if (temp.GetType().IsCOMObject) System.Runtime.InteropServices.Marshal.ReleaseComObject(temp);
                (in2 as CapeOpen.ICapeThermoCompounds).GetCompoundList(ref comps, ref forms, ref names, ref bTemp, ref molWts, ref casNos);
                compIds2 = comps as string[];
                output = this.Ports[2].connectedObject as CapeOpen.ICapeThermoMaterial;
                (output as CapeOpen.ICapeThermoCompounds).GetCompoundList(ref comps, ref forms, ref names, ref bTemp, ref molWts, ref casNos);
                compIds3 = comps as string[];
            }
            catch (System.Exception p_Ex)
            {
                CapeOpen.CapeInvalidOperationException ex = new CapeOpen.CapeInvalidOperationException("Material object does not support CAPE-OPEN Thermodynamics 1.1.", p_Ex);
                this.calcReport = String.Concat(this.calcReport, "Material object does not support CAPE-OPEN Thermodynamics 1.1.", System.Environment.NewLine);
                this.throwException(ex as CapeOpen.ECapeUser);
            }
            int l1 = 0;
            int l2 = 0;
            int l3 = 0;
            l1 = compIds1.Length;
            l2 = compIds2.Length;
            l3 = compIds3.Length;
            if (l1 != l2)
            {
                this.calcReport = string.Concat(this.calcReport, "Compounds in imlet materials do not match.", System.Environment.NewLine);
                throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials do not match.");
            }
            if (l1 != l3)
            {
                this.calcReport = string.Concat(this.calcReport, "Compounds in imlet materials does not match outlet material.", System.Environment.NewLine);
                throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials does not match outlet material.");
            }
            for (int i = 0; i < l3; i++)
            {
                if ((String.Compare(compIds1[i], compIds2[i])) != 0)
                {
                    this.calcReport = string.Concat(this.calcReport, "Compounds in imlet materials do not match.", System.Environment.NewLine);
                    throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials do not match.");
                }

                if (String.Compare(compIds1[i], compIds3[i]) != 0)
                {
                    this.calcReport = string.Concat(this.calcReport, "Compounds in imlet materials does not match outlet material.", System.Environment.NewLine);
                    throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials does not mAtch outlet material.");
                }
            }

            double[] press1 = null;
            double[] press2 = null;
            double[] flow1 = null;
            double[] flow2 = null;
            double[] flow3 = new double[l3];
            string[] props = { "enthalpy" };
            string[] overall = { "Overall" };
            double[] enthalpy = { 0 };
            Object obj = null;
            Object obj1 = null;
            Object obj2 = null;
            Object obj3 = null;
            string[] phases = null;
            CapeOpen.ICapeThermoEquilibriumRoutine eqRoutine;
            try
            {
                in1.GetOverallProp("flow", "mole", ref obj);
                flow1 = obj as double[];
                double totalFlow = 0;
                foreach (double flow in flow1)
                {
                    totalFlow = totalFlow + flow;
                }
                in1.GetOverallProp("pressure", "", ref obj);
                press1 = obj as double[];
                CapeOpen.ICapeThermoPropertyRoutine p_Calc = in1 as CapeOpen.ICapeThermoPropertyRoutine;
                CapeOpen.ICapeThermoPhases phaseList = in1 as CapeOpen.ICapeThermoPhases;
                phaseList.GetPhaseList(ref obj1, ref obj2, ref obj3);
                phases = obj1 as string[];
                foreach (String phase in phases)
                {
                    p_Calc.CalcSinglePhaseProp(props, phase);
                    in1.GetSinglePhaseProp("enthalpy", phase, "mole", ref obj);
                    double[] enth = obj as double[];
                    in1.GetSinglePhaseProp("phaseFraction", phase, "mole", ref obj);
                    double[] fract = obj as double[];
                    enthalpy[0] = enthalpy[0] + totalFlow * fract[0] * enth[0];
                }
            }
            catch (System.Exception p_Ex)
            {
                CapeOpen.ECapeUser user = in1 as CapeOpen.ECapeUser;
                this.calcReport = string.Concat(this.calcReport, user.description, System.Environment.NewLine);
                this.throwException(user);
            }
            try
            {
                in2.GetOverallProp("flow", "mole", ref obj);
                flow2 = obj as double[];
                double totalFlow = 0;
                foreach (double flow in flow2)
                {
                    totalFlow = totalFlow + flow;
                }
                in2.GetOverallProp("pressure", "", ref obj);
                press2 = obj as double[];
                CapeOpen.ICapeThermoPropertyRoutine p_Calc = in2 as CapeOpen.ICapeThermoPropertyRoutine;
                CapeOpen.ICapeThermoPhases phaseList = in2 as CapeOpen.ICapeThermoPhases;
                phaseList.GetPhaseList(ref obj1, ref obj2, ref obj3);
                foreach (String phase in phases)
                {
                    p_Calc.CalcSinglePhaseProp(props, phase);
                    in2.GetSinglePhaseProp("enthalpy", phase, "mole", ref obj);
                    double[] enth = obj as double[];
                    in2.GetSinglePhaseProp("phaseFraction", phase, "mole", ref obj);
                    double[] fract = obj as double[];
                    enthalpy[0] = enthalpy[0] + totalFlow * fract[0] * enth[0];
                }
            }
            catch (System.Exception p_Ex)
            {
                CapeOpen.ECapeUser user = in2 as CapeOpen.ECapeUser;
                this.calcReport = string.Concat(this.calcReport, user.description, System.Environment.NewLine);
                this.throwException(user);
            }
            for (int i = 0; i < l3; i++)
            {
                flow3[i] = flow1[i] + flow2[i];
            }
            try
            {
                output.SetOverallProp("flow", "mole", flow3);
                double[] press = new double[1];
                press[0] = press1[0];
                if (press1[0] > press2[0]) press[0] = press2[0];
                double pressdrop = (this.Parameters[0] as CapeOpen.CRealParameter).SIValue;
                press[0] = press[0] - pressdrop;
                output.SetOverallProp("pressure", "", press);
                output.GetOverallProp("totalFlow", "mole", ref obj);
                double[] totalFlow = obj as double[];
                enthalpy[0] = enthalpy[0] / totalFlow[0];
                output.SetOverallProp("enthalpy", "mole", enthalpy);
                this.calcReport = string.Concat(this.calcReport, "The outlet pressure is: ", press[0].ToString(), "Pa", System.Environment.NewLine);
                int[] status = { 0, 0 };
                output.SetPresentPhases(phases, status);
                this.calcReport = string.Concat(this.calcReport, "The outlet ethalpy is: ", enthalpy[0].ToString(), "J/Mole", System.Environment.NewLine);
                eqRoutine = output as CapeOpen.ICapeThermoEquilibriumRoutine;
                string[] spec1 = { "pressure", String.Empty, "Overall" };
                string[] spec2 = { "enthalpy", String.Empty, "Overall" };
                eqRoutine.CalcEquilibrium(spec1, spec2, "unspecified");
                this.calcReport = string.Concat(this.calcReport, "Calculated pressure-enthalpy flash", System.Environment.NewLine);
            }
            catch (System.Exception p_Ex)
            {
                CapeOpen.ECapeUser user = output as CapeOpen.ECapeUser;
                this.calcReport = string.Concat(this.calcReport, user.description, System.Environment.NewLine);
                this.throwException(user);
            }

            if (in1.GetType().IsCOMObject) System.Runtime.InteropServices.Marshal.ReleaseComObject(in1);
            if (in2.GetType().IsCOMObject) System.Runtime.InteropServices.Marshal.ReleaseComObject(in2);
            // Log the end of the calculation.
            if (this.SimulationContext != null)
                (this.SimulationContext as CapeOpen.ICapeDiagnostic).LogMessage("Ending Mixer Calculation");
            this.calcReport = string.Concat(this.calcReport, "Ending Mixer Calculation");
            //(this.SimulationContext as CapeOpen.ICapeDiagnostic).PopUpMessage("Ending Mixer Calculation");
        }

        /// <summary>
        ///	Produces the active report for the Mixer operation.
        /// </summary>
        /// <remarks>
        ///	This method is declared as an explicit override of the <seealso cref="CapeOpen.ICapeUnitReport.ProduceReport"/> method. 
        /// The report generated is based upon the selected report property of this unit. If "Calculation Report" is
        /// the selected report, the report string generated during the calucation operation is returned.
        /// </remarks>
        /// <example>
        /// <code>
        /// void CapeOpen.ICapeUnitReport.ProduceReport(ref String message)
        /// {
        ///     string text = string.Empty;
        ///     if (this.selectedReport == "Calculation Report") text = this.calcReport;
        ///     else (this as CapeOpen.CUnitBase).ProduceReport(ref text);
        ///     message = text;
        /// }
        /// </code>
        /// </example>
        /// <param name = "message">String containing the text for the currently selected report.</param>
        /// <exception cref = "CapeOpen.ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "CapeOpen.ECapeNoImpl">ECapeNoImpl</exception>
        void CapeOpen.ICapeUnitReport.ProduceReport(ref String message)
        {
            string text = string.Empty;
            if (this.selectedReport == "Calculation Report") text = this.calcReport;
            else (this as CapeOpen.CUnitBase).ProduceReport(ref text);
            message = text;
        }
    }
}
