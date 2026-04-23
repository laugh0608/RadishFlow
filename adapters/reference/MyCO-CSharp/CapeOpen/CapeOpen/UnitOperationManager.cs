using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{
    /// <summary>
    /// This class provides access to unit operations based upon .Net-based assembly location rules.
    /// </summary>
    /// <remarks>
    /// <para>
    /// The CAPE-OPEN object model is based upon Microsoft's Component Object Model (COM).
    /// Computer security practices have evolved to incorporate the principle of least priviledge,
    /// where users rights on the computer are reduced. This protects various high-risk portions
    /// of the system, such as the Windows registry, from attack. The disadvantage of least priviledges
    /// and registry restriction is that it makes development and depolyment of COM-based PMCs difficult
    /// because the local user is unable to install and register the components. 
    /// </para>
    /// <para>
    /// This class utilizes various .Net file location schemes to enable users to develop and
    /// deploy new unit operation PMCs on their machine under the restrictions imposed by 
    /// least priviledges. In particular, it can detect if the debugger is attached and can identifiy 
    /// the assembly being debugged, making it available in the debugger to check the progress of
    /// the unit. Further, it creates and uses a "CapeOpen Objects" directory located in the current user's 
    /// %ProgramFiles%\CommonFiles directory. Subdirectories under this directory are also inspected.
    /// </para>
    /// <para>
    /// Assemblies that contain PMCs developed using this .Net-based class library should be 
    /// placed in a sudirectory under the %Program Files%\CommonFiles\CapeOpen Objects directory.
    /// </para>
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("B41DECE0-6C99-4CA4-B0EB-EFADBDCE23E9")]//ICapeThermoMaterialObject_IID)
    [CapeOpen.CapeNameAttribute("UnitOperationWrapper")]
    [CapeOpen.CapeDescriptionAttribute("This class provides access to unit operations based upon .Net-based assembly location rules.")]
    [CapeOpen.CapeVersionAttribute("1.0")]
    [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(ICapeIdentificationEvents), typeof(System.ComponentModel.INotifyPropertyChanged))]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class UnitOperationWrapper : CapeUnitBase,
        ICapeUnitReport,
        ICapeUnitReportCOM,
        //ICloneable,
        System.ComponentModel.INotifyPropertyChanged,
        //IDisposable,
        System.Runtime.Serialization.ISerializable
    {
        /// <summary>
        /// The wrapped unit operation.
        /// </summary>
        [NonSerialized]
        Object p_Unit;
        System.Guid m_CLSID = System.Guid.Empty;

        /// <summary>
        /// Creates an instance of the UnitOperationWrapper unit operation wrapping the unit operation.
        /// </summary>
        /// <remarks>
        /// This constructor creates an instance of the COM-based CAPE-OPEN unit operation identified by its class 
        /// identification (CLSID) number.
        /// </remarks><param name="CLSID">The class identification number (CLSID) of a COM-based CAPE-OPEN unit operation.</param>
        public UnitOperationWrapper(System.Guid CLSID)
        {
            m_CLSID = CLSID;
            this.p_Unit = Activator.CreateInstance(Type.GetTypeFromCLSID(m_CLSID));
            ICapeUnitCOM test = (ICapeUnitCOM)this.p_Unit;
            if (test == null)
            {
                throwException(new CapeOpen.CapeInvalidArgumentException(string.Concat(Type.GetTypeFromCLSID(m_CLSID).ToString(), "is not a valid ICapeUnit unit operation"), 1));
            }
        }

        /// <summary>
        /// Creates an instance of the UnitOperationWrapper unit operation wrapping the unit operation.
        /// </summary>
        /// <remarks>
        /// This constructor creates a wrapper around a COM-based CAPE-OPEN unit operation. The unit operation is sent as
        /// an argument to the contructor.
        /// </remarks>
        /// <param name="unitOperation">A reference to a CAPE-OPEN based unit operation.</param>
        public UnitOperationWrapper(object unitOperation)
            : base()
        {
            if (!typeof(ICapeUnitCOM).IsAssignableFrom(unitOperation.GetType()))
            {
                throwException(new CapeOpen.CapeInvalidArgumentException(string.Concat(Type.GetTypeFromCLSID(m_CLSID).ToString(), "is not a valid ICapeUnit unit operation"), 1));
            }
            this.p_Unit = unitOperation;
        }

        /// <summary>
        /// Copy constructor for the UnitOperationWrapper class.
        /// </summary>
        /// <remarks>
        /// Copy constructor for wrapper around a COM-based CAPE-OPEN unit operation. The contructor creates a 
        /// new instance of the wrapped unit operation.
        /// </remarks>
        /// <param name="unitOperation">The <see craf = "UnitOperationWrapper"/> to be copied.</param>
        public UnitOperationWrapper(UnitOperationWrapper unitOperation)
            : base()
        {
            if (unitOperation.p_Unit.GetType().IsCOMObject)
            {
                this.p_Unit = Activator.CreateInstance(Type.GetTypeFromCLSID(unitOperation.m_CLSID));
                ICapeUnitCOM test = (ICapeUnitCOM)this.p_Unit;
                if (test == null)
                {
                    throwException(new CapeOpen.CapeInvalidArgumentException(string.Concat(Type.GetTypeFromCLSID(m_CLSID).ToString(), "is not a valid ICapeUnit unit operation"), 1));
                }
            }
            else
            {
                Type unitType = unitOperation.p_Unit.GetType();
                this.p_Unit = AppDomain.CurrentDomain.CreateInstanceAndUnwrap(unitType.AssemblyQualifiedName, unitType.Name);
            }
            this.Initialize();
        }

        /// <summary>
        /// Creates an instance of the UnitOperationWrapper unit operation wrapping the unit operation.
        /// </summary>
        /// <remarks>
        /// This constructor creates a new instance of a <see craf = "UnitOperationWrapper"/> from a serialized
        /// instance.
        /// </remarks>
        /// <param name="info">The serialization data for the object.</param>
        /// <param name="context">The serialization context of the serialized stream.</param>
        public UnitOperationWrapper(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context)
        {
            bool isCom = info.GetBoolean("isCOM");
            if (isCom)
            {
                m_CLSID = new System.Guid(info.GetString("CLSID"));
                this.p_Unit = Activator.CreateInstance(Type.GetTypeFromCLSID(m_CLSID));
            }
            else
            {
                Type type = (Type)info.GetValue("Unit Type", typeof(Type));
            }
            this.Initialize();
            ICapeIdentification p_Id = (ICapeIdentification)p_Unit;
            p_Id.ComponentName = info.GetString("Unit Name");
            p_Id.ComponentName = info.GetString("Unit Description");
            //this.SimulationContext = (ICapeSimulationContext)info.GetValue("Simulation Context", typeof(object));
            object[] m_ParameterValues = (object[])info.GetValue("Parameter Values", typeof(object[]));
            CapeParamMode[] m_ParameterModes = (CapeParamMode[])info.GetValue("Parameter Modes", typeof(CapeParamMode[]));
            int i = 0;
            foreach (ICapeParameter parameter in this.Parameters)
            {
                parameter.value = m_ParameterValues[i];
                parameter.Mode = m_ParameterModes[i];
                i++;
            }
            object[] portConnections = (object[])info.GetValue("PortConnections", typeof(object[]));
            i = 0;
            foreach (ICapeUnitPort port in this.Ports)            
            {
                if (portConnections[i] != null) this.Ports[i].Connect(portConnections[i]);
                i++;
            }
        }

        /// <summary>
        /// Populates a SerializationInfo with the data needed to serialize the <see craf = "UnitOperationWrapper"/>.
        /// </summary>
        /// <remarks>This method restores serializable .NET-based port connections. COM-bassed stream objects may need to 
        /// be connected manually to the ports.</remarks>
        /// <param name="info">The SerializationInfo to populate the <see craf = "UnitOperationWrapper"/> with data.</param>
        /// <param name="context">The destination <see craf = "StreamingContext"/> for this serialization.</param>
        void System.Runtime.Serialization.ISerializable.GetObjectData(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context)
        {
            bool isCOM = p_Unit.GetType().IsCOMObject;
            info.AddValue("isCOM", isCOM, typeof(bool));
            if (isCOM)
            {
                info.AddValue("CLSID", m_CLSID.ToString(), typeof(string));
            }
            else
            {
                info.AddValue("Unit Type", p_Unit.GetType(), typeof(Type));
                info.AddValue("Unit Assembly", p_Unit.GetType().Assembly, typeof(System.Reflection.Assembly));
            }

            ICapeIdentification p_Id = (ICapeIdentification)p_Unit;
            info.AddValue("Unit Name", p_Id.ComponentName, typeof(string));
            info.AddValue("Unit Description", p_Id.ComponentDescription, typeof(string));
            // ICapeUtilities p_Util = (ICapeUtilities)p_Unit;
            //if (this.SimulationContext != null)
            //{
            //    if (this.SimulationContext.GetType().IsSerializable)
            //    {
            //        info.AddValue("Simulation Context", this.SimulationContext, typeof(object));
            //    }
            //    else
            //    {
            //        info.AddValue("Simulation Context", null, typeof(object));
            //    }
            //}
            // else info.AddValue("Simulation Context", null, typeof(object));
            object[] paramValues = new object[this.Parameters.Count];
            CapeParamMode[] paramModes = new CapeParamMode[this.Parameters.Count];
            int i = 0;
            foreach (ICapeParameter parameter in this.Parameters)
            {
                paramValues[i] = parameter.value;
                paramModes[i] = parameter.Mode;
                i++;
            }
            info.AddValue("Parameter Values", paramValues, typeof(object[]));
            info.AddValue("Parameter Modes", paramModes, typeof(CapeParamMode[]));
            object[] portConnections = new object[this.Ports.Count];
            i = 0;
            foreach (ICapeUnitPort port in this.Ports)
            {                
                portConnections[i] = port.connectedObject;
                if (portConnections[i] != null)
                {
                    if (!portConnections[i].GetType().IsSerializable)
                    {
                        portConnections[i] = null;
                    }
                }
                i++;
            }
            info.AddValue("PortConnections", portConnections, typeof(object[]));
        }

        /// <summary>
        /// Creates a new object that is a copy of the current instance.</summary>
        /// <remarks>
        /// <para>
        /// The clone method calls the copy constructor.
        /// </para>       
        /// </remarks>
        /// <returns>A new object that is a copy of this instance.</returns>
        public override object Clone()
        {
            return new UnitOperationWrapper(this);
        }

        /// <summary>
        /// Gets and sets the name of the component.
        /// </summary>
        /// <remarks>
        /// <para>A particular Use Case in a system may contain several CAPE-OPEN components 
        /// of the same class. The user should be able to assign different names and 
        /// descriptions to each instance in order to refer to them unambiguously and in a 
        /// user-friendly way. Since not always the software components that are able to 
        /// set these identifications and the software components that require this information 
        /// have been developed by the same vendor, a CAPE-OPEN standard for setting and 
        /// getting this information is required.</para>
        /// <para>So, the component will not usually set its own name and description: the 
        /// user of the component will do it.</para>
        /// </remarks>
        /// <value>The unique name of the component.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.ComponentModel.DescriptionAttribute("Unit Operation Parameter Collection. Click on the (...) button to edit collection.")]
        [System.ComponentModel.CategoryAttribute("Identification")]
        override public String ComponentName
        {
            get
            {
                return ((ICapeIdentification)p_Unit).ComponentName;
            }

            set
            {
                ComponentNameChangedEventArgs args = new ComponentNameChangedEventArgs(((ICapeIdentification)p_Unit).ComponentName, value);
                ((ICapeIdentification)p_Unit).ComponentName = value;
                NotifyPropertyChanged("ComponentName");
                OnComponentNameChanged(args);
            }
        }

        /// <summary>
        ///  Gets and sets the description of the component.
        /// </summary>
        /// <remarks>
        /// <para>A particular Use Case in a system may contain several CAPE-OPEN components 
        /// of the same class. The user should be able to assign different names and 
        /// descriptions to each instance in order to refer to them unambiguously and in a 
        /// user-friendly way. Since not always the software components that are able to 
        /// set these identifications and the software components that require this information 
        /// have been developed by the same vendor, a CAPE-OPEN standard for setting and 
        /// getting this information is required.</para>
        /// <para>So, the component will not usually set its own name and description: the 
        /// user of the component will do it.</para>
        /// </remarks>
        /// <value>The description of the component.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.ComponentModel.DescriptionAttribute("Unit Operation Parameter Collection. Click on the (...) button to edit collection.")]
        [System.ComponentModel.CategoryAttribute("Identification")]
        override public String ComponentDescription
        {
            get
            {
                return ((ICapeIdentification)p_Unit).ComponentDescription;
            }
            set
            {
                ComponentDescriptionChangedEventArgs args = new ComponentDescriptionChangedEventArgs(((ICapeIdentification)p_Unit).ComponentDescription, value);
                ((ICapeIdentification)p_Unit).ComponentDescription = value;
                NotifyPropertyChanged("ComponentDescription");
                OnComponentDescriptionChanged(args);
            }
        }

        /// <summary>
        ///	Sets the component's simulation context.
        /// </summary>
        /// <remarks>
        /// <para>Allows the PME to convey the PMC a reference to the former’s 
        /// simulation  context. The simulation context will be PME objects which will 
        /// expose a given set of CO interfaces. Each of these interfaces will allow 
        /// the PMC to call back the PME in order to benefit from its exposed services 
        /// (such as creation of material templates, diagnostics or measurement unit 
        /// conversion). If the PMC does not support accessing the simulation context, 
        /// it is recommended to raise the ECapeNoImpl error.</para>
        /// <para>Initially, this method was only present in the ICapeUnit interface. 
        /// Since ICapeUtilities.SetSimulationContext is now available for any kind of 
        /// PMC, ICapeUnit. SetSimulationContext is deprecated.</para>
        /// </remarks>
        /// <value>
        /// The reference to the PME’s simulation context class. For the PMC to use 
        /// this class, this reference will have to be converted to each of the 
        /// defined CO Simulation Context interfaces.
        /// </value>
        /// <value>The simulation context provided by the PME.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("Set the simulation context")]
        public Object simulationContext
        {
            set
            {
                if (!value.GetType().IsAssignableFrom(typeof(ICapeSimulationContext))) return;
                ICapeSimulationContext context = (ICapeSimulationContext)value;
                ICapeUtilitiesCOM unit = (ICapeUtilitiesCOM)p_Unit;
                if (context != null && unit!= null)
                {
                    this.SimulationContext = context;
                    unit.simulationContext = context;
                }
            }            
        }

        /// <summary>
        ///	The component is asked to configure itself. For example a Unit Operation might create ports and parameters during this call
        /// </summary>
        /// <remarks>
        /// <para>Initially, this method was only present in the ICapeUnit interface. 
        /// Since ICapeUtilities.Initialize is now available for any kind of PMC, 
        /// ICapeUnit. Initialize is deprecated.</para>
        /// <para>The PME will order the PMC to get initialized through this method. 
        /// Any initialisation that could fail must be placed here. Initialize is 
        /// guaranteed to be the first method called by the client (except low level 
        /// methods such as class constructors or initialization persistence methods).
        /// Initialize has to be called once when the PMC is instantiated in a 
        /// particular flowsheet.</para>
        /// <para>When the initialization fails, before signalling an error, the PMC 
        /// must free all the resources that were allocated before the failure 
        /// occurred. When the PME receives this error, it may not use the PMC 
        /// anymore.</para>
        /// <para>The method terminate of the current interface must not either be 
        /// called. Hence, the PME may only release the PMC through the middleware 
        /// native mechanisms.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeLicenceError">ECapeLicenceError</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("Configuration has to take place here")]
        override public void Initialize()
        {
            ICapeUtilitiesCOM p_Util = ((ICapeUtilitiesCOM)this.p_Unit);
            p_Util.Initialize();

            ICapeIdentification p_Id = (ICapeIdentification)p_Unit;

            //wrap ports...
            ICapeCollection portColl = (ICapeCollection)((ICapeUnitCOM)(p_Unit)).ports;
            int portCount = portColl.Count();
            object[] portConnections = new object[portCount];
            for (int i = 0; i < portCount; i++)
            {
                ICapeUnitPortCOM port = (ICapeUnitPortCOM)portColl.Item(i + 1);
                this.Ports.Add(new UnitPortWrapper(port));
            }

            //wrap parameters...
            ICapeCollection paramColl = (ICapeCollection)((ICapeUtilitiesCOM)(p_Unit)).parameters;
            int paramCount = paramColl.Count();
            for (int i = 0; i < paramCount; i++)
            {
                ICapeParameter param = (ICapeParameter)paramColl.Item(i + 1);
                CapeParamType paramType = ((ICapeParameterSpecCOM)param.Specification).Type;
                if (((ICapeParameterSpecCOM)param.Specification).Type == CapeParamType.CAPE_REAL)
                {
                    this.Parameters.Add(new RealParameterWrapper(param));
                }
                if (((ICapeParameterSpecCOM)param.Specification).Type == CapeParamType.CAPE_INT)
                {
                    this.Parameters.Add(new IntegerParameterWrapper(param));
                }
                if (((ICapeParameterSpecCOM)param.Specification).Type == CapeParamType.CAPE_OPTION)
                {
                    this.Parameters.Add(new OptionParameterWrapper(param));
                }
                if (((ICapeParameterSpecCOM)param.Specification).Type == CapeParamType.CAPE_BOOLEAN)
                {
                    this.Parameters.Add(new BooleanParameterWrapper(param));
                }
                if (((ICapeParameterSpecCOM)param.Specification).Type == CapeParamType.CAPE_ARRAY)
                {
                    this.Parameters.Add(new ArrayParameterWrapper(param));
                }
            }
        }

        /// <summary>
        ///	Clean-up tasks can be performed here. References to parameters and ports are released here.
        /// </summary>
        /// <remarks>
        /// <para>Initially, this method was only present in the ICapeUnit interface. 
        /// Since ICapeUtilities.Terminate is now available for any kind of PMC, 
        /// ICapeUnit.Terminate is deprecated.</para>
        /// <para>The PME will order the PMC to get destroyed through this method. 
        /// Any uninitialization that could fail must be placed here. ‘Terminate’ is 
        /// guaranteed to be the last method called by the client (except low level 
        /// methods such as class destructors). ‘Terminate’ may be called at any time, 
        /// but may be only called once.</para>
        /// <para>When this method returns an error, the PME should report the user. 
        /// However, after that the PME is not allowed to use the PMC anymore.</para>
        /// <para>The Unit specification stated that “Terminate may check if the data 
        /// has been saved and return an error if not.” It is suggested not to follow 
        /// this recommendation, since it’s the PME responsibility to save the state 
        /// of the PMC before terminating it. In the case that a user wants to close 
        /// a simulation case without saving it, it’s better to leave the PME to 
        /// handle the situation instead of each PMC providing a different 
        /// implementation.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        [System.Runtime.InteropServices.DispIdAttribute(4), System.ComponentModel.DescriptionAttribute("Clean up has to take place here")]
        override public void Terminate()
        {
            if (p_Unit != null) ((ICapeUtilitiesCOM)p_Unit).Terminate();
            base.Terminate();
        }

        /// <summary>
        ///	Displays the PMC graphic interface, if available.
        /// </summary>
        /// <remarks>
        /// The PMC displays its user interface and allows the Flowsheet User to 
        /// interact with it. If no user interface is available it returns an error.
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(5), System.ComponentModel.DescriptionAttribute("Displays the graphic interface")]
        override public System.Windows.Forms.DialogResult Edit()
        {
            if (p_Unit is ICapeUtilitiesCOM)
                return (System.Windows.Forms.DialogResult)((ICapeUtilitiesCOM)p_Unit).Edit();
            System.Windows.Forms.MessageBox.Show("No default Editor Available.");
            return System.Windows.Forms.DialogResult.Cancel;
        }

        //// ICapeUnit Implementation

        /// <summary>
        /// Gets the flag to indicate the unit operation's validation status
        /// <see cref= "CapeValidationStatus">CapeValidationStatus</see>.
        /// </summary>
        /// <remarks>
        /// <para>Get the flag that indicates whether the Flowsheet Unit is valid (e.g. some 
        /// parameter values have changed but they have not been validated by using Validate). 
        /// It has three possible values:</para>
        /// <para>   (i)   notValidated(CAPE_NOT_VALIDATED): The PMC's <c>Validate()</c>
        /// method has not been called after the last time that its value had been 
        /// changed.</para>
        /// <para>   (ii)  invalid(CAPE_INVALID): The last time that the PMC's 
        /// <c>Validate()</c> method was called it returned false.</para>
        /// <para>   (iii) valid(CAPE_VALID): the last time that the PMC's
        /// Validate() method was called it returned true.</para>
        /// </remarks>
        /// <value>A flag that indiciates the validation status of the unit operation.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        override public CapeValidationStatus ValStatus
        {
            get
            {
                if (p_Unit != null) return ((ICapeUnitCOM)p_Unit).ValStatus;
                return CapeValidationStatus.CAPE_INVALID;
            }
        }

        /// <summary>
        ///	Executes the necessary calculations involved in the unit operation model.
        /// </summary>
        /// <remarks>
        /// <para>The Flowsheet Unit performs its calculation, that is, computes the variables 
        /// that are missing at this stage in the complete description of the input and 
        /// output streams and computes any public parameter value that needs to be 
        /// displayed. Calculate will be able to do progress monitoring and checks for 
        /// interrupts as required using the simulation context. At present, there are no
        /// standards agreed for this.</para>
        /// <para>It is recommended that Flowsheet Units perform a suitable flash 
        /// calculation on all output streams. In some cases a Simulation Executive will 
        /// be able to perform a flash calculation but the writer of a Flowsheet Unit is 
        /// in the best position to decide the correct flash to use.</para>
        /// <para>Before performing the calculation, this method should perform any final 
        /// validation tests that are required. For example, at this point the validity of 
        /// Material Objects connected to ports can be checked.</para>
        /// <para>There are no input or output arguments for this method.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeTimeOut">ECapeTimeOut</exception>
        /// <exception cref = "ECapeSolvingError">ECapeSolvingError</exception>
        /// <exception cref = "ECapeLicenceError">ECapeLicenceError</exception>
        override protected void Calculate()
        {
            ((ICapeUnitCOM)p_Unit).Calculate();
        }

        /// <summary>
        ///	Validate the unit operation to verify that the parameters and ports are 
        /// all valid. If invalid, this method returns a message indicating the 
        /// reason that the unit is invalid.
        /// </summary>
        /// <remarks>
        /// This method calls the Vaalidate method of the wrapped unit operation.
        /// </remarks>
        /// <param name="message">Returns a string with a message from the validate function.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeBadCOParameter">ECapeBadCOParameter</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        override public bool Validate(ref System.String message)
        {
            if (p_Unit != null) return ((ICapeUnitCOM)p_Unit).Validate(ref message);
            message = "No unit operation has been selected.";
            return false;
        }

        /// <summary>
        ///	Gets the list of possible reports for the unit operation.
        /// </summary>
        /// <remarks>
        ///	Gets the list of possible reports for the unit operation.
        /// </remarks>
        /// <value>
        ///	The list of possible reports for the unit operation.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        Object ICapeUnitReportCOM.reports
        {
            get
            {
                ICapeUnitReportCOM report = (ICapeUnitReportCOM)p_Unit;
                if (report != null) return report.reports;
                return new String[0];
            }
        }

        /// <summary>
        ///	Gets and sets the current active report for the unit operation.
        /// </summary>
        /// <remarks>
        ///	Gets and sets the current active report for the unit operation.
        /// </remarks>
        /// <value>
        ///	The current active report for the unit operation.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        String ICapeUnitReport.selectedReport
        {
            get
            {
                ICapeUnitReportCOM report = (ICapeUnitReportCOM)p_Unit;
                if (report != null) return report.selectedReport;
                return String.Empty;
            }

            set
            {
                ICapeUnitReportCOM report = (ICapeUnitReportCOM)p_Unit;
                if (report != null) ((ICapeUnitReportCOM)p_Unit).selectedReport = value;
            }
        }

        /// <summary>
        ///	Produces the active report for the unit operation.
        /// </summary>
        /// <remarks>
        ///	Produce the designated report. If no value has been set, it produces the default report.
        /// </remarks>
        /// <returns>String containing the text for the currently selected report.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>      
        String ICapeUnitReport.ProduceReport()
        {
            string report = String.Empty;
            ICapeUnitReportCOM unit = (ICapeUnitReportCOM)p_Unit;
            if (unit!= null) unit.ProduceReport(ref report);
            return report;
        }


        /// <summary>
        ///	Gets the list of possible reports for the unit operation.
        /// </summary>
        /// <remarks>
        ///	Gets the list of possible reports for the unit operation.
        /// </remarks>
        /// <value>
        ///	The list of possible reports for the unit operation.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.ComponentModel.CategoryAttribute("ICapeUnitReport")]
        [System.ComponentModel.DescriptionAttribute("Reports available for the unit.")]
        override public List<string> Reports
        {
            get
            {
                ICapeUnitReportCOM report = (ICapeUnitReportCOM)p_Unit;
                List<string> retVal = new List<string>();
                if (report != null)
                {
                    retVal.AddRange((string[])((ICapeUnitReportCOM)p_Unit).reports);
                }
                return retVal;
            }
        }

        /// <summary>
        ///	Gets and sets the current active report for the unit operation.
        /// </summary>
        /// <remarks>
        ///	Gets and sets the current active report for the unit operation.
        /// </remarks>
        /// <value>
        ///	The current active report for the unit operation.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.TypeConverter(typeof(SelectedReportConverter))]
        [System.ComponentModel.DescriptionAttribute("Name of the report generated by the unit.")]
        [System.ComponentModel.CategoryAttribute("ICapeUnitReport")]
        override public String selectedReport
        {
            get
            {
                ICapeUnitReportCOM report = (ICapeUnitReportCOM)p_Unit;
                if (report != null) return report.selectedReport;
                return String.Empty;
            }

            set
            {
                ICapeUnitReportCOM report = (ICapeUnitReportCOM)p_Unit;
                if (report != null) report.selectedReport = value;
            }
        }

        /// <summary>
        ///	Produces the active report for the unit operation.
        /// </summary>
        /// <remarks>
        ///	Produce the designated report. If no value has been set, it produces the default report.
        /// </remarks>
        /// <value>String containing the text for the currently selected report.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        override public String ProduceReport()
        {
            string report = String.Empty;
            ICapeUnitReportCOM unit = (ICapeUnitReportCOM)p_Unit;
            if (unit != null) unit.ProduceReport(ref report);
            return report;
        }
    }


    /// <summary>
    /// This class represents the behaviour of a Unit 
    ///	Operation connection point (Unit Operation Port). It provides different 
    ///	attributes for configuring the port as well as to connect 
    ///	it to a material, energy or information object.
    /// </summary>
    /// <remarks>
    /// <para>
    /// The unit port provides the the means by which a Flowsheet Unit is connected to its streams. 
    /// Streams are implemented by means of material objects.
    /// </para>
    /// <para>
    /// The three types of port: material, energy and 
    ///	information, have a lot of functionality in common. By combining the three into one we can simplify 
    ///	the interface to a useful degree. Each port type is to be distinguished by the value of an attribute.
    /// </para>
    /// </remarks>
    [Serializable]
    [System.ComponentModel.TypeConverter(typeof(System.ComponentModel.ExpandableObjectConverter))]
    class UnitPortWrapper : UnitPort,
        ICapeUnitPortCOM
    {
        private ICapeUnitPortCOM m_Port;

        /// <summary>
        /// Initializes a new instance of the <see cref="UnitPort"/> class.
        /// </summary>
        /// <param name="port"><see cref="ICapeUnitPort"/>ICapeUnitPortof .</param>
        public UnitPortWrapper(ICapeUnitPortCOM port) :
            base(((ICapeIdentification)port).ComponentName, ((ICapeIdentification)port).ComponentDescription, ((ICapeUnitPortCOM)port).direction, ((ICapeUnitPortCOM)port).portType)
        {
            m_Port = port;
        }

        /// <summary>Creates a new object that is a copy of the current instance.</summary>
        /// <remarks>
        /// <para>
        /// Clone can be implemented either as a deep copy or a shallow copy. In a deep copy, all objects are duplicated; 
        /// in a shallow copy, only the top-level objects are duplicated and the lower levels contain references.
        /// </para>
        /// <para>
        /// The resulting clone must be of the same type as, or compatible with, the original instance.
        /// </para>
        /// <para>
        /// See <see cref="Object.MemberwiseClone"/> for more information on cloning, deep versus shallow copies, and examples.
        /// </para>
        /// </remarks>
        /// <returns>A new object that is a copy of this instance.</returns>
        override public object Clone()
        {
            return new UnitPortWrapper(this.m_Port);
        }

        /// <summary>
        ///	Connects an object to the port. For a material port it must 
        /// be an object implementing the ICapeThermoMaterialObject interface, 
        /// for Energy and Information ports it must be an object implementing 
        /// the ICapeParameter interface. 
        /// </summary>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        override public void Connect(Object objectToConnect)
        {            
            if (typeof(ICapeThermoMaterialObject).IsAssignableFrom(objectToConnect.GetType()))
            {
                COMMaterialObjectWrapper wrapper = new COMMaterialObjectWrapper(objectToConnect);
                m_Port.Connect(wrapper.MaterialObject);
                base.Connect(wrapper);
                return;
            }
            m_Port.Connect(objectToConnect);
            base.Connect(objectToConnect);
        }


        ///// <summary>
        /////	Returns to the client the object that is connected to this port.
        ///// </summary>
        ///// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        ///// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        //Object ICapeUnitPortCOM.connectedObject
        //{
        //    get
        //    {
        //        if (base.connectedObject.GetType().IsCOMObject)
        //        {
        //            return base.connectedObject;
        //        }
        //        if (this.portType == CapePortType.CAPE_MATERIAL)
        //        {
        //            return new COMMaterialObjectWrapper(base.connectedObject);
        //        }
        //        if (this.portType == CapePortType.CAPE_ANY)
        //        {
        //            if (typeof(ICapeThermoMaterialObject).IsAssignableFrom(base.connectedObject.GetType()))
        //            {
        //                return new COMMaterialObjectWrapper(base.connectedObject);
        //            }
        //        }
        //        return base.connectedObject;
        //    }
        //}
        
        /// <summary>
        ///	Disconnects whatever object is connected to this port.
        /// </summary>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        override public void Disconnect()
        {
            m_Port.Disconnect();
            base.Disconnect();
        }
    };
}
