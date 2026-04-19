using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{
    /// <summary>
    /// This is a mixer eample class that models an adiabtic mixer.
    /// </summary>
    /// <remarks>
    /// <para>The mixer  conducts a material and energy balance to determine output flow
    /// from the input flows.</para>
    /// <para> There are four parameters. Only one is used in the calculation, the
    /// real-valued pressure drop. The pressure of the material attached to the outlet
    /// port is set to the pressure of the lower inlet material objects less the 
    /// value of the pressure drop parameter. The remaining parameters are provided
    /// as a demonstration of integer, boolean and option parameters.</para>
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("56E8FDFD-2000-4264-9B47-745B26BE0EC9")]
    [System.ComponentModel.Description("")]
    [CapeOpen.CapeNameAttribute("MixerExample110")]
    [CapeOpen.CapeDescriptionAttribute("An example mixer unit operation using CAPE-OPEN Thermo Version 1.1.")]
    [CapeOpen.CapeVersionAttribute("1.0")]
    [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    [CapeOpen.CapeConsumesThermoAttribute(true)]
    [CapeOpen.CapeSupportsThermodynamics11Attribute(true)]
    [CapeOpen.CapeUnitOperation(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class MixerExample110 : CapeUnitBase
    {
        private String calcReport;

        /// <summary>
        /// Creates an instance of the CMixerExample110 unit operation.
        /// </summary>
        /// <remarks>
        /// This constructor demonstates the addition of a <see cref = "BooleanParameter"/>,
        /// <see cref = "IntegerParameter"/>, <see cref = "OptionParameter"/>,
        /// and a <see cref = "RealParameter"/> parameter to the parameter collection.
        /// In addition, the mixer unit has three <see cref = "UnitPort"/> ports
        /// added to the port collection. See the documentation for the 
        /// <see cref = "MixerExample110.Calculate"/> method for details on its implementation.
        /// </remarks>
        /// <example>
        /// An example of how to create a unit operation. Parameter and port objects are created 
        /// and added the their respective collections. Ports are implemented by the <see cref="UnitPort"/> 
        /// class and are placed in the Port Collection. Parameters are added to the Parameter Collection 
        /// class. The available parameter classes are <see cref="RealParameter"/>, <see cref="IntegerParameter"/>, 
        /// <see cref="BooleanParameter"/>, and <see cref="OptionParameter"/>.
        /// <code>
        /// public MixerExample110()
        /// {
        ///     // Add Ports using the CUnitPort constructor.
        ///     this.Ports.Add(new UnitPort("Inlet Port1", "Test Inlet Port1", CapePortDirection.CAPE_INLET, CapePortType.CAPE_MATERIAL));
        ///     this.Ports.Add(new UnitPort("Inlet Port2", "Test Inlet Port2", CapePortDirection.CAPE_INLET, CapePortType.CAPE_MATERIAL));
        ///     this.Ports.Add(new UnitPort("Outlet Port", "Test Outlet Port", CapePortDirection.CAPE_OUTLET, CapePortType.CAPE_MATERIAL));
        /// 
        ///     // Add a real valued parameter using the CRealParameter  constructor.
        ///     this.Parameters.Add(new RealParameter("PressureDrop", "Drop in pressure between the outlet from the mixer and the pressure of the lower pressure inlet.", 0.0, 0.0, 0.0, 100000000.0, CapeParamMode.CAPE_INPUT, "Pa"));
        /// 
        ///     // Add a real valued parameter using the CIntParameter  constructor.
        ///     this.Parameters.Add(new IntegerParameter("Integer Parameter", 12, CapeParamMode.CAPE_INPUT_OUTPUT));
        /// 
        ///     // Add a real valued parameter using the CBoolParameter  constructor.
        ///     this.Parameters.Add(new BooleanParameter("Boolean Parameter", false, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT));
        /// 
        ///     // Create an array of strings for the option parameter restricted value list.
        ///     String[] options = { "Test Value", "Another Value" };
        /// 
        ///     // Add a string valued parameter using the COptionParameter constructor.
        ///     this.Parameters.Add(new OptionParameter("Option Paramter", "Example Option Parameter", "Test Value", "Test Value", options, true, CapeParamMode.CAPE_INPUT_OUTPUT));
        /// 
        ///     // Add an available report.
        ///     this.Reports.Add("Calculation Report");
        ///     this.calcReport = "The unit has not been calcauted.";
        /// }
        /// </code>
        /// </example>
        public MixerExample110(): base()
        {
            // Add Ports using the CUnitPort constructor.
            this.Ports.Add(new UnitPort("Inlet Port1", "Test Inlet Port1", CapePortDirection.CAPE_INLET, CapePortType.CAPE_MATERIAL));
            this.Ports.Add(new UnitPort("Inlet Port2", "Test Inlet Port2", CapePortDirection.CAPE_INLET, CapePortType.CAPE_MATERIAL));
            this.Ports.Add(new UnitPort("Outlet Port", "Test Outlet Port", CapePortDirection.CAPE_OUTLET, CapePortType.CAPE_MATERIAL));

            // Add a real valued parameter using the CRealParameter  constructor.
            this.Parameters.Add(new RealParameter("PressureDrop", "Drop in pressure between the outlet from the mixer and the pressure of the lower pressure inlet.", 0.0, 0.0, 0.0, 100000000.0, CapeParamMode.CAPE_INPUT, "Pa"));

            // Add a real valued parameter using the CIntParameter  constructor.
            this.Parameters.Add(new IntegerParameter("Integer Parameter", 12, CapeParamMode.CAPE_INPUT_OUTPUT));

            // Add a real valued parameter using the CBoolParameter  constructor.
            this.Parameters.Add(new BooleanParameter("Boolean Parameter", false, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT));

            // Create an array of strings for the option parameter restricted value list.
            String[] options = { "Test Value", "Another Value" };

            // Add a string valued parameter using the COptionParameter constructor.
            this.Parameters.Add(new OptionParameter("Option Paramter", "Example Option Parameter", "Test Value", "Test Value", options, true, CapeParamMode.CAPE_INPUT_OUTPUT));

            // Add an available report.
            this.Reports.Add("Calculation Report");
            this.calcReport = "The unit has not been calcauted.";
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
        /// from each of ports using the <see cref = "PortCollection"/> class. The <see cref = "ICapeThermoMaterial"/>
        /// interface is used to obtain the names using the CompIds() method, flows of each of the 
        /// components present in the material object and overall pressure obtained using the 
        /// <see cref = "ICapeThermoMaterial.GetOverallProp"/> method. The enthaply of 
        /// each phase in the inlet materials is calculated using <see cref = "ICapeThermoPropertyRoutine.CalcSinglePhaseProp"/>
        /// method. The inlet enthalpy was multiplied by the phase flow (phase fraction * overall flow) 
        /// which was summed to determine the outlet stream enthalpy. 
        /// The output pressure from the lower of the two streams' pressure and pressure drop parameter. 
        /// Lastly, the results of the 
        /// calculations are applied to the output material object using the <see cref = "ICapeThermoMaterial.SetOverallProp"/> 
        /// method. The last method of the calculate method is to call the material's 
        /// <see cref = "ICapeThermoEquilibriumRoutine.CalcEquilibrium"/> method.</para>
        /// <para>
        /// A calculation report is generated by this method that is made avalable through the
        /// <see cref = "MixerExample110.ProduceReport"/> method.
        /// </para>
        /// <para>
        /// In this case, the inlet materials need to be released. This is accomplished using the
        /// <see cref = "System.Runtime.InteropServices.Marshal.ReleaseComObject"/> method.
        /// Using this method to release the outlet material object would result in an null object reference error.
        /// </para>
        /// <para>
        /// The method also documents use of the <see cref = "CapeObjectBase.throwException"/> method to provide
        /// CAPE-OPEN compliant error handling.
        /// </para>
        /// <code>
        ///protected override void Calculate()
        ///{
        ///    this.calcReport = String.Empty;
        ///    // Log a message using the simulation context (pop-up message commented out.
        ///    if (this.SimulationContext is CapeOpen.ICapeDiagnostic)
        ///        ((CapeOpen.ICapeDiagnostic)this.SimulationContext).LogMessage("Starting Mixer Calculation");
        ///    this.calcReport = String.Concat(this.calcReport, "Starting Mixer Calculation", System.Environment.NewLine);
        ///    // Get the material Object from Port 0.
        ///    ICapeThermoMaterial in1 = null;
        ///    ICapeThermoMaterial in2 = null;
        ///    ICapeThermoMaterial outlet = null;
        ///    ICapeThermoMaterial temp = null;
        ///    String[] compIds1 = null;
        ///    String[] compIds2 = null;
        ///    String[] compIds3 = null;
        ///    String[] forms = null;
        ///    String[] names = null;
        ///    double[] bTemps = null;
        ///    double[] molWts = null;
        ///    String[] casNos = null;
        ///    try
        ///    {
        ///        temp = (ICapeThermoMaterial)this.Ports[0].connectedObject;
        ///        in1 = (ICapeThermoMaterial)temp.CreateMaterial();
        ///        in1.CopyFromMaterial(temp);
        ///        if (temp.GetType().IsCOMObject) System.Runtime.InteropServices.Marshal.ReleaseComObject(temp);
        ///        ((CapeOpen.ICapeThermoCompounds)in1).GetCompoundList(ref compIds1, ref forms, ref names, ref bTemps, ref molWts, ref casNos);
        ///        temp = (ICapeThermoMaterial)this.Ports[1].connectedObject;
        ///        in2 = (ICapeThermoMaterial)temp.CreateMaterial();
        ///        in2.CopyFromMaterial(temp);
        ///        if (temp.GetType().IsCOMObject) System.Runtime.InteropServices.Marshal.FinalReleaseComObject(temp);
        ///        ((CapeOpen.ICapeThermoCompounds)in2).GetCompoundList(ref compIds2, ref forms, ref names, ref bTemps, ref molWts, ref casNos);
        ///        outlet = (ICapeThermoMaterial)this.Ports[2].connectedObject;
        ///        ((CapeOpen.ICapeThermoCompounds)outlet).GetCompoundList(ref compIds3, ref forms, ref names, ref bTemps, ref molWts, ref casNos);
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        CapeOpen.CapeInvalidOperationException ex = new CapeOpen.CapeInvalidOperationException("Material object does not support CAPE-OPEN Thermodynamics 1.1.", p_Ex);
        ///        this.calcReport = String.Concat(this.calcReport, "Material object does not support CAPE-OPEN Thermodynamics 1.1.", System.Environment.NewLine);
        ///        this.throwException(ex);
        ///    }
        ///    int l1 = compIds1.Length;
        ///    int l2 = compIds2.Length;
        ///    int l3 = compIds3.Length;
        ///    if (l1 != l2)
        ///    {
        ///        this.calcReport = String.Concat(this.calcReport, "Compounds in imlet materials do not match.", System.Environment.NewLine);
        ///        throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials do not match.");
        ///    }
        ///    if (l1 != l3)
        ///    {
        ///        this.calcReport = String.Concat(this.calcReport, "Compounds in imlet materials does not match outlet material.", System.Environment.NewLine);
        ///        throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials does not match outlet material.");
        ///    }
        ///    for (int i = 0; i &lt; l3; i++)
        ///    {
        ///        if ((String.Compare(compIds1[i], compIds2[i])) != 0)
        ///        {
        ///            this.calcReport = String.Concat(this.calcReport, "Compounds in imlet materials do not match.", System.Environment.NewLine);
        ///            throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials do not match.");
        ///        }
        ///        
        ///        if (String.Compare(compIds1[i], compIds3[i]) != 0)
        ///        {
        ///            this.calcReport = String.Concat(this.calcReport, "Compounds in imlet materials does not match outlet material.", System.Environment.NewLine);
        ///            throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials does not mAtch outlet material.");
        ///        }
        ///    }
        ///    
        ///    double[] flow1 = null;
        ///    double[] flow2 = null;
        ///    double[] press1 = null;
        ///    double[] press2 = null;
        ///    double[] flow3 = new double[l3];
        ///    String[] props = { "enthalpy" };
        ///    String[] overall = { "Overall" };
        ///    double[] enthalpy = { 0 };
        ///    String[] phases = null;
        ///    CapeOpen.ICapeThermoEquilibriumRoutine eqRoutine;
        ///    string[] strArr1 = null;
        ///    string[] strArr2 = null;
        ///    try
        ///    {
        ///    
        ///        in1.GetOverallProp("flow", "mole", ref flow1);
        ///        double totalFlow = 0;
        ///        foreach (double flow in flow1)
        ///        {
        ///            totalFlow = totalFlow + flow;
        ///        }
        ///        in1.GetOverallProp("pressure", "", ref press1);
        ///        CapeOpen.ICapeThermoPropertyRoutine p_Calc = (CapeOpen.ICapeThermoPropertyRoutine)in1;
        ///        CapeOpen.ICapeThermoPhases phaseList = (CapeOpen.ICapeThermoPhases)in1;
        ///        phaseList.GetPhaseList(ref phases, ref strArr1, ref strArr2);
        ///        foreach (String phase in phases)
        ///        {
        ///            p_Calc.CalcSinglePhaseProp(props, phase);
        ///            double[] enth = null;
        ///            in1.GetSinglePhaseProp("enthalpy", phase, "mole", ref enth);
        ///            double[] fract = null;
        ///            in1.GetSinglePhaseProp("phaseFraction", phase, "mole", ref fract);
        ///            enthalpy[0] = enthalpy[0] + totalFlow * fract[0] * enth[0];
        ///        }
        ///        IDisposable disp = in1 as IDisposable;
        ///        if (disp != null)
        ///        {
        ///            disp.Dispose();
        ///        }
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        CapeOpen.ECapeUser user = (CapeOpen.ECapeUser)in1;
        ///        this.calcReport = String.Concat(this.calcReport, user.description, System.Environment.NewLine);
        ///        this.throwException(p_Ex);
        ///    }
        ///    try
        ///    {
        ///        in2.GetOverallProp("flow", "mole", ref flow2);
        ///        double totalFlow = 0;
        ///        foreach (double flow in flow2)
        ///        {
        ///            totalFlow = totalFlow + flow;
        ///        }
        ///        in2.GetOverallProp("pressure", "", ref press2);
        ///        CapeOpen.ICapeThermoPropertyRoutine p_Calc = (CapeOpen.ICapeThermoPropertyRoutine)in2;
        ///        CapeOpen.ICapeThermoPhases phaseList = (CapeOpen.ICapeThermoPhases)in2;
        ///        phaseList.GetPhaseList(ref phases, ref strArr1, ref strArr2);
        ///        foreach (String phase in phases)
        ///        {
        ///            p_Calc.CalcSinglePhaseProp(props, phase);
        ///            double[] enth = null;
        ///            in2.GetSinglePhaseProp("enthalpy", phase, "mole", ref enth);
        ///            double[] fract = null;
        ///            in2.GetSinglePhaseProp("phaseFraction", phase, "mole", ref fract);
        ///            enthalpy[0] = enthalpy[0] + totalFlow * fract[0] * enth[0];
        ///        }
        ///        IDisposable disp = in2 as IDisposable;
        ///        if (disp != null)
        ///        {
        ///            disp.Dispose();
        ///        }
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        CapeOpen.ECapeUser user = (CapeOpen.ECapeUser)in2;
        ///        this.calcReport = String.Concat(this.calcReport, user.description, System.Environment.NewLine);
        ///        this.throwException(p_Ex);
        ///    }
        ///    for (int i = 0; i &lt; l3; i++)
        ///    {
        ///        flow3[i] = flow1[i] + flow2[i];
        ///    }
        ///    try
        ///    {
        ///        outlet.SetOverallProp("flow", "mole", flow3);
        ///        double[] press = new double[1];
        ///        press[0] = press1[0];
        ///        if (press1[0] &gt; press2[0]) press[0] = press2[0];
        ///        double pressdrop = ((CapeOpen.RealParameter)this.Parameters[0]).SIValue;
        ///        press[0] = press[0] - pressdrop;
        ///        outlet.SetOverallProp("pressure", "", press);
        ///        double[] totalFlow = null;
        ///        outlet.GetOverallProp("totalFlow", "mole", ref totalFlow);
        ///        enthalpy[0] = enthalpy[0] / totalFlow[0];
        ///        outlet.SetOverallProp("enthalpy", "mole", enthalpy);
        ///        this.calcReport = String.Concat(this.calcReport, "The outlet pressure is: ", press[0].ToString(), "Pa", System.Environment.NewLine);
        ///        CapePhaseStatus[] status = { CapePhaseStatus.CAPE_ATEQUILIBRIUM, CapePhaseStatus.CAPE_ATEQUILIBRIUM };
        ///        outlet.SetPresentPhases(phases, status);
        ///        this.calcReport = String.Concat(this.calcReport, "The outlet ethalpy is: ", enthalpy[0].ToString(), "J/Mole", System.Environment.NewLine);
        ///        eqRoutine = (CapeOpen.ICapeThermoEquilibriumRoutine)outlet;
        ///        String[] spec1 = { "pressure", String.Empty, "Overall" };
        ///        String[] spec2 = { "enthalpy", String.Empty, "Overall" };
        ///        eqRoutine.CalcEquilibrium(spec1, spec2, "unspecified");
        ///        this.calcReport = String.Concat(this.calcReport, "Calculated pressure-enthalpy flash", System.Environment.NewLine);
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        CapeOpen.ECapeUser user = (CapeOpen.ECapeUser)outlet;
        ///        this.calcReport = String.Concat(this.calcReport, user.description, System.Environment.NewLine);
        ///        this.throwException(p_Ex);
        ///    }
        ///    // Log the end of the calculation.
        ///    this.calcReport = String.Concat(this.calcReport, "Ending Mixer Calculation");
        ///    if (this.SimulationContext is CapeOpen.ICapeDiagnostic)
        ///        ((CapeOpen.ICapeDiagnostic)this.SimulationContext).LogMessage("Ending Mixer Calculation");
        ///}
        /// </code>
        /// </example>
        /// <see cref="ICapeThermoMaterial"/>
        /// <see cref="ICapeThermoCompounds"/>
        /// <see cref="ICapeThermoPhases"/>
        /// <see cref="ICapeThermoPropertyRoutine"/>
        /// <see cref="ICapeThermoEquilibriumRoutine"/>
        /// <see cref="COMExceptionHandler"/>
        protected override void Calculate()
        {
            this.calcReport = String.Empty;
            // Log a message using the simulation context (pop-up message commented out.
            if (this.SimulationContext is CapeOpen.ICapeDiagnostic)
                ((CapeOpen.ICapeDiagnostic)this.SimulationContext).LogMessage("Starting Mixer Calculation");
            this.calcReport = String.Concat(this.calcReport, "Starting Mixer Calculation", System.Environment.NewLine);
            // Get the material Object from Port 0.
            ICapeThermoMaterial in1 = null;
            ICapeThermoMaterial in2 = null;
            ICapeThermoMaterial outlet = null;
            ICapeThermoMaterial temp = null;            
            String[] compIds1 = null;
            String[] compIds2 = null;
            String[] compIds3 = null;
            String[] forms = null;
            String[] names = null;
            double[] bTemps = null;
            double[] molWts = null;
            String[] casNos = null;
            try
            {
                temp = (ICapeThermoMaterial)this.Ports[0].connectedObject;
                in1 = (ICapeThermoMaterial)temp.CreateMaterial();
                in1.CopyFromMaterial(temp);
                if (temp.GetType().IsCOMObject) System.Runtime.InteropServices.Marshal.ReleaseComObject(temp);
                ((CapeOpen.ICapeThermoCompounds)in1).GetCompoundList(ref compIds1, ref forms, ref names, ref bTemps, ref molWts, ref casNos);
                temp = (ICapeThermoMaterial)this.Ports[1].connectedObject;
                in2 = (ICapeThermoMaterial)temp.CreateMaterial();
                in2.CopyFromMaterial(temp);
                if (temp.GetType().IsCOMObject) System.Runtime.InteropServices.Marshal.FinalReleaseComObject(temp);
                ((CapeOpen.ICapeThermoCompounds)in2).GetCompoundList(ref compIds2, ref forms, ref names, ref bTemps, ref molWts, ref casNos);
                outlet = (ICapeThermoMaterial)this.Ports[2].connectedObject;
                ((CapeOpen.ICapeThermoCompounds)outlet).GetCompoundList(ref compIds3, ref forms, ref names, ref bTemps, ref molWts, ref casNos);
            }
            catch (System.Exception p_Ex)
            {
                CapeOpen.CapeInvalidOperationException ex = new CapeOpen.CapeInvalidOperationException("Material object does not support CAPE-OPEN Thermodynamics 1.1.", p_Ex);
                this.calcReport = String.Concat(this.calcReport, "Material object does not support CAPE-OPEN Thermodynamics 1.1.", System.Environment.NewLine);
                this.throwException(ex);
            }
            int l1 = compIds1.Length;
            int l2 = compIds2.Length;
            int l3 = compIds3.Length;
            if (l1 != l2)
            {
                this.calcReport = String.Concat(this.calcReport, "Compounds in imlet materials do not match.", System.Environment.NewLine);
                throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials do not match.");
            }
            if (l1 != l3)
            {
                this.calcReport = String.Concat(this.calcReport, "Compounds in imlet materials does not match outlet material.", System.Environment.NewLine);
                throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials does not match outlet material.");
            }
            for (int i = 0; i < l3; i++)
            {
                if ((String.Compare(compIds1[i], compIds2[i])) != 0)
                {
                    this.calcReport = String.Concat(this.calcReport, "Compounds in imlet materials do not match.", System.Environment.NewLine);
                    throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials do not match.");
                }

                if (String.Compare(compIds1[i], compIds3[i]) != 0)
                {
                    this.calcReport = String.Concat(this.calcReport, "Compounds in imlet materials does not match outlet material.", System.Environment.NewLine);
                    throw new CapeOpen.CapeInvalidOperationException("Compounds in imlet materials does not mAtch outlet material.");
                }
            }

            double[] flow1 = null;
            double[] flow2 = null;
            double[] press1 = null;
            double[] press2 = null;
            double[] flow3 = new double[l3];
            String[] props = { "enthalpy" };
            String[] overall = { "Overall" };
            double[] enthalpy = { 0 };            
            String[] phases = null;
            CapeOpen.ICapeThermoEquilibriumRoutine eqRoutine;
            string[] strArr1 = null;
            string[] strArr2 = null;
            try
            {
                
                in1.GetOverallProp("flow", "mole", ref flow1);
                double totalFlow = 0;
                foreach (double flow in flow1)
                {
                    totalFlow = totalFlow + flow;
                }
                in1.GetOverallProp("pressure", "", ref press1);
                CapeOpen.ICapeThermoPropertyRoutine p_Calc = (CapeOpen.ICapeThermoPropertyRoutine)in1;
                CapeOpen.ICapeThermoPhases phaseList = (CapeOpen.ICapeThermoPhases)in1;
                phaseList.GetPhaseList(ref phases, ref strArr1, ref strArr2);
                foreach (String phase in phases)
                {
                    p_Calc.CalcSinglePhaseProp(props, phase);
                    double[] enth = null;
                    in1.GetSinglePhaseProp("enthalpy", phase, "mole", ref enth);
                    double[] fract = null;
                    in1.GetSinglePhaseProp("phaseFraction", phase, "mole", ref fract);
                    enthalpy[0] = enthalpy[0] + totalFlow * fract[0] * enth[0];
                }
                IDisposable disp = in1 as IDisposable;
                if (disp != null)
                {
                    disp.Dispose();
                }
            }
            catch (System.Exception p_Ex)
            {
                CapeOpen.ECapeUser user = (CapeOpen.ECapeUser)in1;
                this.calcReport = String.Concat(this.calcReport, user.description, System.Environment.NewLine);
                this.throwException(p_Ex);
            }
            try
            {
                in2.GetOverallProp("flow", "mole", ref flow2);
                double totalFlow = 0;
                foreach (double flow in flow2)
                {
                    totalFlow = totalFlow + flow;
                }
                in2.GetOverallProp("pressure", "", ref press2);
                CapeOpen.ICapeThermoPropertyRoutine p_Calc = (CapeOpen.ICapeThermoPropertyRoutine)in2;
                CapeOpen.ICapeThermoPhases phaseList = (CapeOpen.ICapeThermoPhases)in2;
                phaseList.GetPhaseList(ref phases, ref strArr1, ref strArr2);
                foreach (String phase in phases)
                {
                    p_Calc.CalcSinglePhaseProp(props, phase);
                    double[] enth = null;
                    in2.GetSinglePhaseProp("enthalpy", phase, "mole", ref enth);
                    double[] fract= null;
                    in2.GetSinglePhaseProp("phaseFraction", phase, "mole", ref fract);
                    enthalpy[0] = enthalpy[0] + totalFlow * fract[0] * enth[0];
                }
                IDisposable disp = in2 as IDisposable;
                if (disp != null)
                {
                    disp.Dispose();
                }
            }
            catch (System.Exception p_Ex)
            {
                CapeOpen.ECapeUser user = (CapeOpen.ECapeUser)in2;
                this.calcReport = String.Concat(this.calcReport, user.description, System.Environment.NewLine);
                this.throwException(p_Ex);
            }
            for (int i = 0; i < l3; i++)
            {
                flow3[i] = flow1[i] + flow2[i];
            }
            try
            {
                outlet.SetOverallProp("flow", "mole", flow3);
                double[] press = new double[1];
                press[0] = press1[0];
                if (press1[0] > press2[0]) press[0] = press2[0];
                double pressdrop = ((CapeOpen.RealParameter)this.Parameters[0]).SIValue;
                press[0] = press[0] - pressdrop;
                outlet.SetOverallProp("pressure", "", press);
                double[] totalFlow = null;
                outlet.GetOverallProp("totalFlow", "mole", ref totalFlow);
                enthalpy[0] = enthalpy[0] / totalFlow[0];
                outlet.SetOverallProp("enthalpy", "mole", enthalpy);
                this.calcReport = String.Concat(this.calcReport, "The outlet pressure is: ", press[0].ToString(), "Pa", System.Environment.NewLine);
                CapePhaseStatus[] status = { CapePhaseStatus.CAPE_ATEQUILIBRIUM, CapePhaseStatus.CAPE_ATEQUILIBRIUM};
                outlet.SetPresentPhases(phases, status);
                this.calcReport = String.Concat(this.calcReport, "The outlet ethalpy is: ", enthalpy[0].ToString(), "J/Mole", System.Environment.NewLine);
                eqRoutine = (CapeOpen.ICapeThermoEquilibriumRoutine)outlet;
                String[] spec1 = { "pressure", String.Empty, "Overall" };
                String[] spec2 = { "enthalpy", String.Empty, "Overall" };
                eqRoutine.CalcEquilibrium(spec1, spec2, "unspecified");
                this.calcReport = String.Concat(this.calcReport, "Calculated pressure-enthalpy flash", System.Environment.NewLine);
            }
            catch (System.Exception p_Ex)
            {
                CapeOpen.ECapeUser user = (CapeOpen.ECapeUser)outlet;
                this.calcReport = String.Concat(this.calcReport, user.description, System.Environment.NewLine);
                this.throwException(p_Ex);
            }
            // Log the end of the calculation.
            this.calcReport = String.Concat(this.calcReport, "Ending Mixer Calculation");
            if (this.SimulationContext is CapeOpen.ICapeDiagnostic)
                ((CapeOpen.ICapeDiagnostic)this.SimulationContext).LogMessage("Ending Mixer Calculation");
        }

        /// <summary>
        ///	Produces the active report for the Mixer operation.
        /// </summary>
        /// <remarks>
        ///	This method is declared as an explicit override of the <see cref="CapeOpen.ICapeUnitReport.ProduceReport"/> method. 
        /// The report generated is based upon the selected report property of this unit. If "Calculation Report" is
        /// the selected report, the report string generated during the calucation operation is returned.
        /// </remarks>
        /// <returns>The report text.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        public override string ProduceReport()
        {
            if (base.selectedReport == "Default Report")
            {
                return base.ProduceReport();
            }
            if (base.selectedReport == "Calculation Report") return this.calcReport;
            return string.Empty;
        }
    };
}
