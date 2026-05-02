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
    [System.Runtime.InteropServices.Guid("883D46FE-5713-424C-BF10-7ED34263CD6D")]//ICapeThermoMaterialObject_IID)
    [System.ComponentModel.Description("")]
    [CapeOpen.CapeNameAttribute("MixerExample")]
    [CapeOpen.CapeDescriptionAttribute("An example mixer unit operation written in C#.")]
    [CapeOpen.CapeVersionAttribute("1.0")]
    [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    [CapeOpen.CapeConsumesThermoAttribute(true)]
    [CapeOpen.CapeUnitOperation(true)]
    [CapeOpen.CapeSupportsThermodynamics10Attribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class MixerExample : CapeUnitBase
    {
        /// <summary>
        /// Creates an instance of the CMixerExample unit operation.
        /// </summary>
        /// <remarks>
        /// This constructor demonstates the addition of a <see cref = "BooleanParameter"/>,
        /// <see cref = "IntegerParameter"/>, <see cref = "OptionParameter"/>,
        /// and a <see cref = "RealParameter"/> parameter to the parameter collection.
        /// In addition, the mixer unit has three <see cref = "UnitPort"/> ports
        /// added to the port collection. See the documentation for the 
        /// <see cref = "MixerExample.Calculate"/> method for details on its implementation.
        /// </remarks>
        /// <example>
        /// An example of how to create a unit operation. Parameter and port objects are created 
        /// and added the their respective collections. Ports are implemented by the <see cref="UnitPort"/> 
        /// class and are placed in the Port Collection. Parameters are added to the Parameter Collection 
        /// class. The available parameter classes are <see cref="RealParameter"/>, <see cref="IntegerParameter"/>, 
        /// <see cref="BooleanParameter"/>, and <see cref="OptionParameter"/>.
        /// <code>
		/// public MixerExample()
		/// {
		///     // Add Ports using the UnitPort constructor.
		///     this.Ports.Add(new UnitPort("Inlet Port1", "Test Inlet Port1", CapePortDirection.CAPE_INLET, CapePortType.CAPE_MATERIAL));
		///     this.Ports.Add(new UnitPort("Inlet Port2", "Test Inlet Port2", CapePortDirection.CAPE_INLET, CapePortType.CAPE_MATERIAL));
		///     this.Ports.Add(new UnitPort("Outlet Port", "Test Outlet Port", CapePortDirection.CAPE_OUTLET, CapePortType.CAPE_MATERIAL));
		/// 
		///     // Add a real valued parameter using the RealParameter  constructor.
		///     RealParameter real = new RealParameter("PressureDrop", "Drop in pressure between the outlet from the mixer and the pressure of the lower pressure inlet.", 0.0, 0.0, 0.0, 100000000.0, CapeParamMode.CAPE_INPUT, "Pa");
		///     this.Parameters.Add(real);
		/// 
		///     // Add a real valued parameter using the IntegerParameter  constructor.
		///     this.Parameters.Add(new IntegerParameter("Integer Parameter", "This is an example of an integer parameter.", 12, 12, 0, 100, CapeParamMode.CAPE_INPUT_OUTPUT));
		/// 
		///     // Add a real valued parameter using the BooleanParameter  constructor.
		///     this.Parameters.Add(new BooleanParameter("Boolean Parameter", "This is an example of a boolean parameter.", false, false, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT));
		/// 
		///     // Create an array of strings for the option parameter restricted value list.
		///     String[] options = { "Test Value", "Another Value" };
		/// 
		///     // Add a string valued parameter using the OptionParameter constructor.
		///     this.Parameters.Add(new OptionParameter("OptionParameter", "This is an example of an option parameter.", "Test Value", "Test Value", options, true, CapeParamMode.CAPE_INPUT_OUTPUT));
		/// 
		///     // Add an available report.
		///     this.Reports.Add("Report 2");
		/// }
		/// </code>
        /// </example>
        public MixerExample()
        {
            // Add Ports using the UnitPort constructor.
            this.Ports.Add(new UnitPort("Inlet Port1", "Test Inlet Port1", CapePortDirection.CAPE_INLET, CapePortType.CAPE_MATERIAL));
            this.Ports.Add(new UnitPort("Inlet Port2", "Test Inlet Port2", CapePortDirection.CAPE_INLET, CapePortType.CAPE_MATERIAL));
            this.Ports.Add(new UnitPort("Outlet Port", "Test Outlet Port", CapePortDirection.CAPE_OUTLET, CapePortType.CAPE_MATERIAL));

            // Add a real valued parameter using the RealParameter  constructor.
            RealParameter real = new RealParameter("PressureDrop", "Drop in pressure between the outlet from the mixer and the pressure of the lower pressure inlet.", 0.0, 0.0, 0.0, 100000000.0, CapeParamMode.CAPE_INPUT, "Pa");
            this.Parameters.Add(real);

            // Add a real valued parameter using the IntegerParameter  constructor.
            this.Parameters.Add(new IntegerParameter("Integer Parameter", "This is an example of an integer parameter.", 12, 12, 0, 100, CapeParamMode.CAPE_INPUT_OUTPUT));

            // Add a real valued parameter using the BooleanParameter  constructor.
            this.Parameters.Add(new BooleanParameter("Boolean Parameter", "This is an example of a boolean parameter.", false, false, CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT));

            // Create an array of strings for the option parameter restricted value list.
            String[] options = { "Test Value", "Another Value" };

            // Add a string valued parameter using the OptionParameter constructor.
            this.Parameters.Add(new OptionParameter("OptionParameter", "This is an example of an option parameter.", "Test Value", "Test Value", options, true, CapeParamMode.CAPE_INPUT_OUTPUT));

            // Add an available report.
            this.Reports.Add("Report 2");
        }

        /// <summary>
        /// Calculation method for the MixerExample unit operation.
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
        /// from each of ports using the <see cref = "PortCollection"/> class. The <see cref = "ICapeThermoMaterialObject"/>
        /// interface is used to obtain the names using the CompIds() method, flows of each of the 
        /// components present in the material object, overall pressure and overall enthalpy are
        /// obtained using the <see cref = "ICapeThermoMaterialObject.GetProp"/> method. The overall enthalpy of the stram is 
        /// calculated using <see cref = "ICapeThermoMaterialObject.CalcProp"/> method. The unit then combines the flows, 
        /// calculates the output stream enthalpy, and determines output pressure from the lower of 
        /// the two streams' pressure and pressure drop parameter. Lastly, the results of the 
        /// calculations are applied to the output material object using the <see cref = "ICapeThermoMaterialObject.SetProp"/> 
        /// method. The last method of the calculate method is to call the material's 
        /// <see cref = "ICapeThermoMaterialObject.CalcEquilibrium"/> method.</para>
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
        /// protected override void Calculate()
        /// {
        ///    // Log a message using the simulation context (pop-up message commented out.
        ///    if (this.SimulationContext != null)
        ///        ((ICapeDiagnostic)this.SimulationContext).LogMessage("Starting Mixer Calculation");
        ///    //(CapeOpen.ICapeDiagnostic)(this.SimulationContext).PopUpMessage("Starting Mixer Calculation");
        ///    
        ///    // Get the material Object from Port 0.
        ///    ICapeThermoMaterialObject in1 = null;
        ///    ICapeThermoMaterialObject tempMO = null;
        ///    try
        ///    {
        ///        tempMO = (ICapeThermoMaterialObject)this.Ports[0].connectedObject;
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        this.OnUnitOperationEndCalculation("Error - Material object does not support CAPE-OPEN Thermodynamics 1.0.");
        ///        CapeOpen.CapeInvalidOperationException ex = new CapeOpen.CapeInvalidOperationException("Material object does not support CAPE-OPEN Thermodynamics 1.0.", p_Ex);
        ///        this.throwException(ex);
        ///    }
        ///    
        ///    // Duplicate the port, its an input port, always use a duplicate.
        ///    try
        ///    {
        ///        in1 = (ICapeThermoMaterialObject)tempMO.Duplicate();
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        this.OnUnitOperationEndCalculation("Error - Object connected to Inlet Port 1 does not support CAPE-OPEN Thermodynamics 1.0.");
        ///        CapeOpen.CapeInvalidOperationException ex = new CapeOpen.CapeInvalidOperationException("Object connected to Inlet Port 1 does not support CAPE-OPEN Thermodynamics 1.0.", p_Ex);
        ///        this.throwException(ex);
        ///    }
        ///    // Arrays for the GetProps and SetProps call for enthaply.
        ///    String[] phases = { "Overall" };
        ///    String[] props = { "enthalpy" };
        ///    
        ///    // Declare variables for calculations.
        ///    String[] in1Comps = null;
        ///    double[] in1Flow = null;
        ///    double[] in1Enthalpy = null;
        ///    double[] pressure = null;
        ///    double totalFlow1 = 0;
        ///    
        ///    // Exception catching code...
        ///    try
        ///    {
        ///        // Get Strings, must cast to string array data type.
        ///        in1Comps = (String[])in1.ComponentIds;
        ///        
        ///        // Get flow. Arguments are the property; phase, in this case, Overall; compound identifications
        ///        // in this case, the null returns the property for all components; calculation type, in this case,  
        ///        // null, no calculation type; and lastly, the basis, moles. Result is cast to a double array, and will contain one value.
        ///        in1Flow = (double[])in1.GetProp("flow", "Overall", null, null, "mole");
        ///        
        ///        // Get pressure. Arguments are the property; phase, in this case, Overall; compound identifications
        ///        // in this case, the null returns the property for all components; calculation type, in this case, the 
        ///        // mixture; and lastly, the basis, moles. Result is cast to a double array, and will contain one value.
        ///        pressure = (double[])in1.GetProp("Pressure", "Overall", null, "Mixture", null);
        ///        
        ///        // The following code adds the individual flows to get the total flow for the stream.
        ///        for (int i = 0; i &lt; in1Flow.Length; i++)
        ///        {
        ///            totalFlow1 = totalFlow1 + in1Flow[i];
        ///        }
        ///        
        ///        // Calculates the mixture enthalpy of the stream.
        ///        in1.CalcProp(props, phases, "Mixture");
        ///        
        ///        // Get the enthalpy of the stream. Arguments are the property, enthalpy; the phase, overall;
        ///        // a null pointer, required as the overall enthalpy is desired; the calculation type is
        ///        // mixture; and the basis is moles.
        ///        in1Enthalpy = (double[])in1.GetProp("enthalpy", "Overall", null, "Mixture", "mole");
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        // Exception handling, wraps a COM exception, shows the message, and re-throws the excecption.
        ///        if (p_Ex is System.Runtime.InteropServices.COMException)
        ///        {
        ///            System.Runtime.InteropServices.COMException comException = (System.Runtime.InteropServices.COMException)p_Ex;
        ///            p_Ex = CapeOpen.COMExceptionHandler.ExceptionForHRESULT(in1, p_Ex);
        ///        }
        ///        this.throwException(p_Ex);
        ///    }
        ///    IDisposable disp = in1 as IDisposable;
        ///    if (disp != null)
        ///    {
        ///        disp.Dispose();
        ///    }
        ///    
        ///
        ///    // Get the material Object from Port 0.
        ///    ICapeThermoMaterialObject in2 = null;
        ///    tempMO = null;
        ///    try
        ///    {
        ///        tempMO = (ICapeThermoMaterialObject)this.Ports[1].connectedObject;
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        this.OnUnitOperationEndCalculation("Error - Material object does not support CAPE-OPEN Thermodynamics 1.0.");
        ///        CapeOpen.CapeInvalidOperationException ex = new CapeOpen.CapeInvalidOperationException("Material object does not support CAPE-OPEN Thermodynamics 1.0.", p_Ex);
        ///        this.throwException(ex);
        ///    }
        ///    
        ///    // Duplicate the port, its an input port, always use a duplicate.
        ///    try
        ///    {
        ///        in2 = (ICapeThermoMaterialObject)tempMO.Duplicate();
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        this.OnUnitOperationEndCalculation("Error - Object connected to Inlet Port 1 does not support CAPE-OPEN Thermodynamics 1.0.");
        ///        CapeOpen.CapeInvalidOperationException ex = new CapeOpen.CapeInvalidOperationException("Object connected to Inlet Port 1 does not support CAPE-OPEN Thermodynamics 1.0.", p_Ex);
        ///        this.throwException(ex);
        ///    }
        ///    
        ///    // Declare variables.
        ///    String[] in2Comps = null;
        ///    double[] in2Flow = null;
        ///    double[] in2Enthalpy = null;
        ///    double totalFlow2 = 0;
        ///    
        ///    // Try block.
        ///    try
        ///    {
        ///        // Get the component identifications.
        ///        in2Comps = in2.ComponentIds;
        ///        
        ///        // Get flow. Arguments are the property; phase, in this case, Overall; compound identifications
        ///        // in this case, the null returns the property for all components; calculation type, in this case,  
        ///        // null, no calculation type; and lastly, the basis, moles. Result is cast to a double array, and will contain one value.
        ///        in2Flow = in2.GetProp("flow", "Overall", null, null, "mole");
        ///        
        ///        // Get pressure. Arguments are the property; phase, in this case, Overall; compound identifications
        ///        // in this case, the null returns the property for all components; calculation type, in this case, the 
        ///        // mixture; and lastly, the basis, moles. Result is cast to a double array, and will contain one value.
        ///        double[] press = in2.GetProp("Pressure", "Overall", null, "Mixture", null);
        ///        if (press[0] &lt; pressure[0]) pressure[0] = press[0];
        ///        
        ///        // The following code adds the individual flows to get the total flow for the stream.
        ///        for (int i = 0; i &lt; in2Flow.Length; i++)
        ///        {
        ///            totalFlow2 = totalFlow2 + in2Flow[i];
        ///        }
        ///        
        ///        // Calculates the mixture enthalpy of the stream.
        ///        in2.CalcProp(props, phases, "Mixture");
        ///        
        ///        // Get the enthalpy of the stream. Arguments are the property, enthalpy; the phase, overall;
        ///        // a null pointer, required as the overall enthalpy is desired; the calculation type is
        ///        // mixture; and the basis is moles.
        ///        in2Enthalpy = in2.GetProp("enthalpy", "Overall", null, "Mixture", "mole");
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        System.Runtime.InteropServices.COMException comException = (System.Runtime.InteropServices.COMException)p_Ex;
        ///        if (comException != null)
        ///        {
        ///            p_Ex = CapeOpen.COMExceptionHandler.ExceptionForHRESULT(in2, p_Ex);
        ///        }
        ///        this.throwException(p_Ex);
        ///    }
        ///    // Release the material object if it is a COM object.
        ///    disp = in2 as IDisposable;
        ///    if (disp != null)
        ///    {
        ///        disp.Dispose();
        ///    }
        ///    
        /// 
        ///    // Get the outlet material object.
        ///    ICapeThermoMaterialObject outPort = (ICapeThermoMaterialObject)this.Ports[2].connectedObject;
        ///    
        ///    // An empty, one-member array to set values in the outlet material stream.
        ///    double[] values = new double[1];
        ///    
        ///    // Use energy balanace to calculate the outlet enthalpy.
        ///    values[0] = (in1Enthalpy[0] * totalFlow1 + in2Enthalpy[0] * totalFlow2) / (totalFlow1 + totalFlow2);
        ///    try
        ///    {
        ///        // Set the outlet enthalpy, for the overall phase, with a mixture calculation type
        ///        // to the value calculated above.
        ///        outPort.SetProp("enthalpy", "Overall", null, "Mixture", "mole", values);
        ///        
        ///        // Set the outlet pressure to the lower of the to inlet pressures less the value of the 
        ///        // pressure drop parameter.
        ///        pressure[0] = pressure[0] - (((RealParameter)(this.Parameters[0])).SIValue);
        ///        
        ///        // Set the outlet pressure.
        ///        outPort.SetProp("Pressure", "Overall", null, null, null, pressure);
        ///        
        ///        // Resize the value array for the number of components.
        ///        values = new double[in1Comps.Length];
        ///        
        ///        // Calculate the individual flow for each component.
        ///        for (int i = 0; i &lt; in1Comps.Length; i++)
        ///        {
        ///            values[i] = in1Flow[i] + in2Flow[i];
        ///        }
        ///        // Set the outlet flow by component. Note, this is for overall phase and mole flows.
        ///        // The component Identifications are used as a check.
        ///        outPort.SetProp("flow", "Overall", in1Comps, null, "mole", values);
        ///        
        ///        // Calculate equilibrium using a "pressure-enthalpy" flash type.
        ///        outPort.CalcEquilibrium("PH", null);
        ///        
        ///        // Release the material object if it is a COM object.
        ///        if (outPort.GetType().IsCOMObject)
        ///        {
        ///            System.Runtime.InteropServices.Marshal.FinalReleaseComObject(outPort);
        ///        }
        ///    }
        ///    catch (System.Exception p_Ex)
        ///    {
        ///        System.Runtime.InteropServices.COMException comException = (System.Runtime.InteropServices.COMException)p_Ex;
        ///        if (comException != null)
        ///        {
        ///            p_Ex = CapeOpen.COMExceptionHandler.ExceptionForHRESULT(outPort, p_Ex);
        ///        }
        ///        this.throwException(p_Ex);
        ///    }
        ///    
        ///    // Log the end of the calculation.
        ///    if (this.SimulationContext != null)
        ///        ((CapeOpen.ICapeDiagnostic)this.SimulationContext).LogMessage("Ending Mixer Calculation");
        ///    //(CapeOpen.ICapeDiagnostic)(this.SimulationContext).PopUpMessage("Ending Mixer Calculation");
        /// }
        /// 
        /// </code>
        /// </example>
        /// <see cref="ICapeThermoMaterialObject"/>
        /// <see cref="COMExceptionHandler"/>
        protected override void Calculate()
        {
            // Log a message using the simulation context (pop-up message commented out.
            if (this.SimulationContext != null)
                ((ICapeDiagnostic)this.SimulationContext).LogMessage("Starting Mixer Calculation");
            //(CapeOpen.ICapeDiagnostic>(this.SimulationContext).PopUpMessage("Starting Mixer Calculation");

            // Get the material Object from Port 0.
            ICapeThermoMaterialObject in1 = null;
            ICapeThermoMaterialObject tempMO = null;
            try
            {
                tempMO = (ICapeThermoMaterialObject)this.Ports[0].connectedObject;
            }
            catch (System.Exception p_Ex)
            {
                this.OnUnitOperationEndCalculation("Error - Material object does not support CAPE-OPEN Thermodynamics 1.0.");
                CapeOpen.CapeInvalidOperationException ex = new CapeOpen.CapeInvalidOperationException("Material object does not support CAPE-OPEN Thermodynamics 1.0.", p_Ex);
                this.throwException(ex);
            }

            // Duplicate the port, its an input port, always use a duplicate.
            try
            {
                in1 = (ICapeThermoMaterialObject)tempMO.Duplicate();
            }
            catch (System.Exception p_Ex)
            {
                this.OnUnitOperationEndCalculation("Error - Object connected to Inlet Port 1 does not support CAPE-OPEN Thermodynamics 1.0.");
                CapeOpen.CapeInvalidOperationException ex = new CapeOpen.CapeInvalidOperationException("Object connected to Inlet Port 1 does not support CAPE-OPEN Thermodynamics 1.0.", p_Ex);
                this.throwException(ex);
            }
            // Arrays for the GetProps and SetProps call for enthaply.
            String[] phases = { "Overall" };
            String[] props = { "enthalpy" };

            // Declare variables for calculations.
            String[] in1Comps = null;
            double[] in1Flow = null;
            double[] in1Enthalpy = null;
            double[] pressure = null;
            double totalFlow1 = 0;

            // Exception catching code...
            try
            {
                // Get Strings, must cast to string array data type.
                in1Comps = (String[])in1.ComponentIds;

                // Get flow. Arguments are the property; phase, in this case, Overall; compound identifications
                // in this case, the null returns the property for all components; calculation type, in this case,  
                // null, no calculation type; and lastly, the basis, moles. Result is cast to a double array, and will contain one value.
                in1Flow = (double[])in1.GetProp("flow", "Overall", null, null, "mole");

                // Get pressure. Arguments are the property; phase, in this case, Overall; compound identifications
                // in this case, the null returns the property for all components; calculation type, in this case, the 
                // mixture; and lastly, the basis, moles. Result is cast to a double array, and will contain one value.
                pressure = (double[])in1.GetProp("Pressure", "Overall", null, "Mixture", null);

                // The following code adds the individual flows to get the total flow for the stream.
                for (int i = 0; i < in1Flow.Length; i++)
                {
                    totalFlow1 = totalFlow1 + in1Flow[i];
                }

                // Calculates the mixture enthalpy of the stream.
                in1.CalcProp(props, phases, "Mixture");

                // Get the enthalpy of the stream. Arguments are the property, enthalpy; the phase, overall;
                // a null pointer, required as the overall enthalpy is desired; the calculation type is
                // mixture; and the basis is moles.
                in1Enthalpy = (double[])in1.GetProp("enthalpy", "Overall", null, "Mixture", "mole");
            }
            catch (System.Exception p_Ex)
            {
                // Exception handling, wraps a COM exception, shows the message, and re-throws the excecption.

                if (p_Ex is System.Runtime.InteropServices.COMException)
                {
                    System.Runtime.InteropServices.COMException comException = (System.Runtime.InteropServices.COMException)p_Ex;
                    p_Ex = CapeOpen.COMExceptionHandler.ExceptionForHRESULT(in1, p_Ex);
                }
                this.throwException(p_Ex);
            }
            IDisposable disp = in1 as IDisposable;
            if (disp != null)
            {
                disp.Dispose();
            }


            // Get the material Object from Port 0.
            ICapeThermoMaterialObject in2 = null;
            tempMO = null;
            try
            {
                tempMO = (ICapeThermoMaterialObject)this.Ports[1].connectedObject;
            }
            catch (System.Exception p_Ex)
            {
                this.OnUnitOperationEndCalculation("Error - Material object does not support CAPE-OPEN Thermodynamics 1.0.");
                CapeOpen.CapeInvalidOperationException ex = new CapeOpen.CapeInvalidOperationException("Material object does not support CAPE-OPEN Thermodynamics 1.0.", p_Ex);
                this.throwException(ex);
            }

            // Duplicate the port, its an input port, always use a duplicate.
            try
            {
                in2 = (ICapeThermoMaterialObject)tempMO.Duplicate();
            }
            catch (System.Exception p_Ex)
            {
                this.OnUnitOperationEndCalculation("Error - Object connected to Inlet Port 1 does not support CAPE-OPEN Thermodynamics 1.0.");
                CapeOpen.CapeInvalidOperationException ex = new CapeOpen.CapeInvalidOperationException("Object connected to Inlet Port 1 does not support CAPE-OPEN Thermodynamics 1.0.", p_Ex);
                this.throwException(ex);
            }
            
            // Declare variables.
            String[] in2Comps = null;
            double[] in2Flow = null;
            double[] in2Enthalpy = null;
            double totalFlow2 = 0;

            // Try block.
            try
            {
                // Get the component identifications.
                in2Comps = in2.ComponentIds;

                // Get flow. Arguments are the property; phase, in this case, Overall; compound identifications
                // in this case, the null returns the property for all components; calculation type, in this case,  
                // null, no calculation type; and lastly, the basis, moles. Result is cast to a double array, and will contain one value.
                in2Flow = in2.GetProp("flow", "Overall", null, null, "mole");

                // Get pressure. Arguments are the property; phase, in this case, Overall; compound identifications
                // in this case, the null returns the property for all components; calculation type, in this case, the 
                // mixture; and lastly, the basis, moles. Result is cast to a double array, and will contain one value.
                double[] press = in2.GetProp("Pressure", "Overall", null, "Mixture", null);
                if (press[0] < pressure[0]) pressure[0] = press[0];

                // The following code adds the individual flows to get the total flow for the stream.
                for (int i = 0; i < in2Flow.Length; i++)
                {
                    totalFlow2 = totalFlow2 + in2Flow[i];
                }

                // Calculates the mixture enthalpy of the stream.
                in2.CalcProp(props, phases, "Mixture");

                // Get the enthalpy of the stream. Arguments are the property, enthalpy; the phase, overall;
                // a null pointer, required as the overall enthalpy is desired; the calculation type is
                // mixture; and the basis is moles.
                in2Enthalpy = in2.GetProp("enthalpy", "Overall", null, "Mixture", "mole");
            }
            catch (System.Exception p_Ex)
            {
                System.Runtime.InteropServices.COMException comException = (System.Runtime.InteropServices.COMException)p_Ex;
                if (comException != null)
                {
                    p_Ex = CapeOpen.COMExceptionHandler.ExceptionForHRESULT(in2, p_Ex);
                }
                this.throwException(p_Ex);
            }
            // Release the material object if it is a COM object.
            disp = in2 as IDisposable;
            if (disp != null)
            {
                disp.Dispose();
            }


            // Get the outlet material object.
            ICapeThermoMaterialObject outPort = (ICapeThermoMaterialObject)this.Ports[2].connectedObject;
            
            // An empty, one-member array to set values in the outlet material stream.
            double[] values = new double[1];

            // Use energy balanace to calculate the outlet enthalpy.
            values[0] = (in1Enthalpy[0] * totalFlow1 + in2Enthalpy[0] * totalFlow2) / (totalFlow1 + totalFlow2);
            try
            {
                // Set the outlet enthalpy, for the overall phase, with a mixture calculation type
                // to the value calculated above.
                outPort.SetProp("enthalpy", "Overall", null, "Mixture", "mole", values);

                // Set the outlet pressure to the lower of the to inlet pressures less the value of the 
                // pressure drop parameter.
                pressure[0] = pressure[0] - (((RealParameter)(this.Parameters[0])).SIValue); 

                // Set the outlet pressure.
                outPort.SetProp("Pressure", "Overall", null, null, null, pressure);

                // Resize the value array for the number of components.
                values = new double[in1Comps.Length];

                // Calculate the individual flow for each component.
                for (int i = 0; i < in1Comps.Length; i++)
                {
                    values[i] = in1Flow[i] + in2Flow[i];
                }
                // Set the outlet flow by component. Note, this is for overall phase and mole flows.
                // The component Identifications are used as a check.
                outPort.SetProp("flow", "Overall", in1Comps, null, "mole", values);

                // Calculate equilibrium using a "pressure-enthalpy" flash type.
                outPort.CalcEquilibrium("PH", null);

                // Release the material object if it is a COM object.
                if (outPort.GetType().IsCOMObject)
                {
                    System.Runtime.InteropServices.Marshal.FinalReleaseComObject(outPort);
                }
            }
            catch (System.Exception p_Ex)
            {
                System.Runtime.InteropServices.COMException comException = (System.Runtime.InteropServices.COMException)p_Ex;
                if (comException != null)
                {
                    p_Ex = CapeOpen.COMExceptionHandler.ExceptionForHRESULT(outPort, p_Ex);
                }
                this.throwException(p_Ex);
            }
            
            // Log the end of the calculation.
            if (this.SimulationContext != null)
                ((CapeOpen.ICapeDiagnostic)this.SimulationContext).LogMessage("Ending Mixer Calculation");
            //(CapeOpen.ICapeDiagnostic>(this.SimulationContext).PopUpMessage("Ending Mixer Calculation");
        }

        /// <summary>
        ///	Produces the active report for the Mixer Example unit operation.
        /// </summary>
        /// <remarks>
        /// The ProduceReport method creates the active report for the unit operation. The method looks to the 
        /// <see cref="CapeUnitBase.selectedReport"/> and generates the required report. If a local report has
        /// ben added likne in <see cref="MixerExample.MixerExample"/>, this method must generate that report.
        /// </remarks>
        /// <example>
        /// An example of how to produce a report for a unit operation. In this case, the report can be either 
        /// "Report 2" defined in the <see cref="MixerExample.MixerExample"/> or the "Default Report" from 
        /// <see cref="CapeUnitBase"/>. If "Default Report" is selected, then the <see cref="CapeUnitBase.ProduceReport"/>
        /// method is called, and the message parameter forwarded. Otherwise, the report is generated in this method.
        /// "Default Report" gen.
        /// <code>
        /// public override void ProduceReport(ref String message)
        /// {
        ///     if (this.selectedReport == "Default Report") base.ProduceReport(ref message);
        ///     if (this.selectedReport == "Report 2") message = "This is the alternative Report.";
        /// }
        /// </code>
        /// </example>
        /// <returns>The report text.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        public override string ProduceReport()
        {
            if (this.selectedReport == "Default Report") return base.ProduceReport();
            if (this.selectedReport == "Report 2") return "This is the alternative Report.";
            return string.Empty;
        }
    };
}
