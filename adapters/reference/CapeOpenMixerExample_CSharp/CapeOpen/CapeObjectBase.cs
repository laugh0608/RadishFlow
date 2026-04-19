using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{
    /// <summary>
    /// Abstract base class that implements ICapeIdentification and ICapeUtilities. 
    /// </summary>
    /// <remarks>
    /// This abstract class contains all required functionality for ICapeIdentification and ICapeUtilities
    /// It can be inherited and used as any generalized PMC. The derived class will register itself as a 
    /// CAPE-OPEN Component (Category GUID of 678c09a1-7d66-11d2-a67d-00105a42887f) and a Flowsheet
    /// monitoring Object (Category GUID of 7BA1AF89-B2E4-493d-BD80-2970BF4CBE99).
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public abstract class CapeObjectBase : CapeIdentification,
        ICapeUtilities,
        ICapeUtilitiesCOM,
        CapeOpen.ECapeUser,
        CapeOpen.ECapeRoot
    {
        /// <summary>
        /// The message returned during the last validation of the unit operation.
        /// </summary>
        protected string m_ValidationMessage;
        private ParameterCollection m_Parameters;
        [NonSerialized]
        private System.Exception p_Exception;

        // Track whether Dispose has been called.
        private bool _disposed;
        /// <summary>
        ///	The simulation context that can be used by the PMC.
        /// </summary>
        /// <remarks>
        /// The simulation cotext provides access to the PME, enabling the PMC to access the PME's COSE interfaces <see cref ="ICapeDiagnostic"/>, 
        /// <see cref ="ICapeMaterialTemplateSystem"/> and <see cref ="ICapeCOSEUtilities"/>.
        /// </remarks>
        [NonSerialized]
        private ICapeSimulationContext m_SimulationContext;

        static System.Reflection.Assembly MyResolveEventHandler(Object sender, System.ResolveEventArgs args)
        {
            return typeof(CapeObjectBase).Assembly;
        }


        /// <summary>
        ///	Displays the PMC graphic interface, if available.
        /// </summary>
        /// <remarks>
        /// <para>By default, this method throws a <see cref="CapeNoImplException">CapeNoImplException</see>
        /// that according to the CAPE-OPEN specification, is interpreted by the process
        /// modeling environment as indicating that the PMC does not have a editor 
        /// GUI, and the PME must perform editing steps.</para>
        /// <para>In order for a PMC to provide its own editor, the Edit method will
        /// need to be overridden to create a graphical editor. When the user requests the flowheet
        /// to show the editor, this method will be called to edit the unit. Overriden classes should
        /// not return a failure (throw and exception) as this will be interpreted by the flowsheeting 
        /// tool as the unit not providing its own editor.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        int ICapeUtilitiesCOM.Edit()
        {
            try
            {
                System.Windows.Forms.DialogResult result = this.Edit();
                if (result == System.Windows.Forms.DialogResult.OK)
                    return 0;
                return 1;
            }
            catch(System.Exception p_Ex)
            {
                throw new CapeNoImplException("No editor available");
            }
        }
        
        /// <summary>
        ///	Gets the component's collection of parameters.
        /// </summary>
        /// <value>
        /// Return type is System.Object and this method is simply here for classic 
        /// COM-based CAPE-OPEN interop.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        Object ICapeUtilitiesCOM.parameters
        {
            get
            {
                return m_Parameters;
            }
        }

        /// <summary>
        ///	Sets the component's simulation context.
        /// </summary>
        /// <remarks>
        /// This method provides access to the COSE's interfaces <see cref ="ICapeDiagnostic"/>, 
        /// <see cref ="ICapeMaterialTemplateSystem"/> and <see cref ="ICapeCOSEUtilities"/>.
        /// </remarks>
        /// <value>The simulation context assigned by the Flowsheeting Environment.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        Object ICapeUtilitiesCOM.simulationContext
        {
            set
            {
                if (value is ICapeSimulationContext)
                    m_SimulationContext = (ICapeSimulationContext)value;
            }
        }


        /// <summary>
        ///	Clean-up tasks can be performed here. 
        /// </summary>
        /// <remarks>
        /// <para>The CAPE-OPEN object should releases all of its allocated resources during this call. This is 
        /// called before the object destructor by the PME. Terminate may check if the data has been 
        /// saved and return an error if not.</para>
        /// <para>There are no input or output arguments for this method.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        void ICapeUtilitiesCOM.Terminate()
        {
            this.Terminate();
        }

        /// <summary>
        ///	Initialization can be performed here. 
        /// </summary>
        /// <remarks>
        /// <para>The CAPE_OPEN object can allocated resources during this method. This is 
        /// called after the object constructor by the PME. .</para>
        /// <para>There are no input or output arguments for this method.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s), specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        void ICapeUtilitiesCOM.Initialize()
        {
            this.Initialize();
        }

        /// <summary>
        ///	Constructor for the unit operation.
        /// </summary>
        /// <remarks>
        /// This method is creates the parameter collections for the object. As a result, 
        /// parameters can be added in the constructor
        /// for the derived object or during the <c>Initialize()</c> call. 
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeLicenceError">ECapeLicenceError</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        public CapeObjectBase()
            : base()
        {
            m_Parameters = new ParameterCollection();
            this.m_SimulationContext = null;
            this.m_ValidationMessage = "This object has not been validated.";
            _disposed = false;
        }

        /// <summary>
        /// Finalizer for the <see cref = "CapeObjectBase"/> class.
        /// </summary>
        /// <remarks>
        /// This will finalize the current instance of the class.
        /// </remarks>
        ~CapeObjectBase()
        {
            this.Dispose();
        }

        /// <summary>
        ///	Constructor for the unit operation.
        /// </summary>
        /// <remarks>
        /// This method is creates the parameter collections for the object. As a result, 
        /// parameters can be added in the constructor
        /// for the derived object or during the <c>Initialize()</c> call. 
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeLicenceError">ECapeLicenceError</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        /// <param name = "name">The name of the PMC.</param>
        public CapeObjectBase(String name)
            : base(name)
        {
            m_Parameters = new ParameterCollection();
            this.m_SimulationContext = null;
            this.m_ValidationMessage = "This object has not been validated.";
            _disposed = false;
        }

        /// <summary>
        ///	Constructor for the unit operation.
        /// </summary>
        /// <remarks>
        /// This method is creates the parameter collections for the object. As a result, 
        /// parameters can be added in the constructor
        /// for the derived object or during the <c>Initialize()</c> call. 
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeLicenceError">ECapeLicenceError</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        /// <param name = "name">The name of the PMC.</param>
        /// <param name = "description">The description of the PMC.</param>
        public CapeObjectBase(String name, String description)
            : base(name, description)
        {
            m_Parameters = new ParameterCollection();
            this.m_SimulationContext = null;
            this.m_ValidationMessage = "This object has not been validated.";
            _disposed = false;
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
        /// <param name = "objectToBeCopied">The object being copied.</param>
        public CapeObjectBase(CapeObjectBase objectToBeCopied)
            : base((CapeIdentification)objectToBeCopied)
        {
            m_SimulationContext = objectToBeCopied.m_SimulationContext;
            m_Parameters.Clear();
            foreach (CapeParameter parameter in objectToBeCopied.Parameters)
            {
                m_Parameters.Add((CapeParameter)parameter.Clone());
            }
            this.m_ValidationMessage = "This object has not been validated.";
            _disposed = false;
        }


        /// <summary>
        /// Creates a new object that is a copy of the current instance.</summary>
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
            CapeObjectBase retVal = (CapeObjectBase)AppDomain.CurrentDomain.CreateInstanceAndUnwrap(this.GetType().AssemblyQualifiedName, this.GetType().FullName);
            retVal.Parameters.Clear();
            foreach (CapeParameter param in this.Parameters)
            {
                retVal.Parameters.Add((CapeParameter)param.Clone());
            }
            retVal.SimulationContext = null;
            if (retVal.GetType().IsAssignableFrom(typeof(ICapeSimulationContext)))
                retVal.SimulationContext = (ICapeSimulationContext)this.m_SimulationContext;
            return retVal;
        }
                        
        // Dispose(bool disposing) executes in two distinct scenarios.
        // If disposing equals true, the method has been called directly
        // or indirectly by a user's code. Managed and unmanaged resources
        // can be disposed.
        // If disposing equals false, the method has been called by the
        // runtime from inside the finalizer and you should not reference
        // other objects. Only unmanaged resources can be disposed.
        /// <summary>
        /// Releases the unmanaged resources used by the CapeIdentification object and optionally releases 
        /// the managed resources.
        /// </summary>
        /// <remarks><para>This method is called by the public <see href="http://msdn.microsoft.com/en-us/library/system.componentmodel.component.dispose.aspx">Dispose</see>see> 
        /// method and the <see href="http://msdn.microsoft.com/en-us/library/system.object.finalize.aspx">Finalize</see> method. 
        /// <bold>Dispose()</bold> invokes the protected <bold>Dispose(Boolean)</bold> method with the disposing
        /// parameter set to <bold>true</bold>. <see href="http://msdn.microsoft.com/en-us/library/system.object.finalize.aspx">Finalize</see> 
        /// invokes <bold>Dispose</bold> with disposing set to <bold>false</bold>.</para>
        /// <para>When the <italic>disposing</italic> parameter is <bold>true</bold>, this method releases all 
        /// resources held by any managed objects that this Component references. This method invokes the 
        /// <bold>Dispose()</bold> method of each referenced object.</para>
        /// <para><bold>Notes to Inheritors</bold></para>
        /// <para><bold>Dispose</bold> can be called multiple times by other objects. When overriding 
        /// <bold>Dispose(Boolean)</bold>, be careful not to reference objects that have been previously 
        /// disposed of in an earlier call to <bold>Dispose</bold>. For more information about how to 
        /// implement <bold>Dispose(Boolean)</bold>, see <see href="http://msdn.microsoft.com/en-us/library/fs2xkftw.aspx">Implementing a Dispose Method</see>.</para>
        /// <para>For more information about <bold>Dispose</bold> and <see href="http://msdn.microsoft.com/en-us/library/system.object.finalize.aspx">Finalize</see>, 
        /// see <see href="http://msdn.microsoft.com/en-us/library/498928w2.aspx">Cleaning Up Unmanaged Resources</see> 
        /// and <see href="http://msdn.microsoft.com/en-us/library/ddae83kx.aspx">Overriding the Finalize Method</see>.</para>
        /// </remarks> 
        /// <param name = "disposing">true to release both managed and unmanaged resources; false to release only unmanaged resources.</param>
        protected override void Dispose(bool disposing)
        {
            // Check to see if Dispose has already been called.
            if (!_disposed)
            {
                // If disposing equals true, dispose all managed
                // and unmanaged resources.
                if (disposing)
                {
                    if (m_SimulationContext != null)
                    {
                        if (m_SimulationContext.GetType().IsCOMObject)
                            System.Runtime.InteropServices.Marshal.FinalReleaseComObject(m_SimulationContext);
                    }
                    m_SimulationContext = null;
                    m_Parameters.Clear();
                    _disposed = true;
                }
                base.Dispose(disposing);
            }
        }

        /// <summary>
        ///	The function that controls COM registration.  
        /// </summary>
        /// <remarks>
        ///	This function adds the registration keys specified in the CAPE-OPEN Method and
        /// Tools specifications. In particular, it indicates that this unit operation implements
        /// the CAPE-OPEN Unit Operation Category Identification. It also adds the CapeDescription
        /// registry keys using the <see cref ="CapeNameAttribute"/>, <see cref ="CapeDescriptionAttribute"/>, <see cref ="CapeVersionAttribute"/>
        /// <see cref ="CapeVendorURLAttribute"/>, <see cref ="CapeHelpURLAttribute"/>, 
        /// and <see cref ="CapeAboutAttribute"/> attributes.
        /// </remarks>
        /// <param name = "t">The type of the class being registered.</param> 
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.ComRegisterFunction()]
        public static void RegisterFunction(Type t)
        {
            RegistrationHelper(Microsoft.Win32.RegistryKey.OpenBaseKey(Microsoft.Win32.RegistryHive.ClassesRoot, Microsoft.Win32.RegistryView.Registry32), t);
            RegistrationHelper(Microsoft.Win32.RegistryKey.OpenBaseKey(Microsoft.Win32.RegistryHive.ClassesRoot, Microsoft.Win32.RegistryView.Registry64), t);
        }

        private static void RegistrationHelper(Microsoft.Win32.RegistryKey baseKey, Type t)
        {
            System.Reflection.Assembly assembly = t.Assembly;
            String versionNumber = (new System.Reflection.AssemblyName(assembly.FullName)).Version.ToString();

            String keyname = String.Concat("CLSID\\{", t.GUID.ToString(), "}");
            Microsoft.Win32.RegistryKey classKey = baseKey.CreateSubKey(keyname);
            Microsoft.Win32.RegistryKey catidKey = classKey.CreateSubKey("Implemented Categories");
            catidKey.CreateSubKey(COGuids.CapeOpenComponent_CATID);

            Object[] attributes = t.GetCustomAttributes(false);
            String nameInfoString = t.FullName;
            String descriptionInfoString = "";
            String versionInfoString = "";
            String companyURLInfoString = "";
            String helpURLInfoString = "";
            String aboutInfoString = "";
            for (int i = 0; i < attributes.Length; i++)
            {
                if (attributes[i] is CapeUnitOperationAttribute) catidKey.CreateSubKey(COGuids.CapeUnitOperation_CATID);
                if (attributes[i] is CapeFlowsheetMonitoringAttribute) catidKey.CreateSubKey(COGuids.CATID_MONITORING_OBJECT);
                if (attributes[i] is CapeConsumesThermoAttribute) catidKey.CreateSubKey(COGuids.Consumes_Thermo_CATID);
                if (attributes[i] is CapeSupportsThermodynamics10Attribute) catidKey.CreateSubKey(COGuids.SupportsThermodynamics10_CATID);
                if (attributes[i] is CapeSupportsThermodynamics11Attribute) catidKey.CreateSubKey(COGuids.SupportsThermodynamics11_CATID);
                if (attributes[i] is CapeNameAttribute) nameInfoString = ((CapeNameAttribute)attributes[i]).Name;
                if (attributes[i] is CapeDescriptionAttribute) descriptionInfoString = ((CapeDescriptionAttribute)attributes[i]).Description;
                if (attributes[i] is CapeVersionAttribute) versionInfoString = ((CapeVersionAttribute)attributes[i]).Version;
                if (attributes[i] is CapeVendorURLAttribute) companyURLInfoString = ((CapeVendorURLAttribute)attributes[i]).VendorURL;
                if (attributes[i] is CapeHelpURLAttribute) helpURLInfoString = ((CapeHelpURLAttribute)attributes[i]).HelpURL;
                if (attributes[i] is CapeAboutAttribute) aboutInfoString = ((CapeAboutAttribute)attributes[i]).About;
            }

            Microsoft.Win32.RegistryKey descriptKey = classKey.CreateSubKey("CapeDescription");
            descriptKey.SetValue("Name", nameInfoString);
            descriptKey.SetValue("Description", descriptionInfoString);
            descriptKey.SetValue("CapeVersion", versionInfoString);
            descriptKey.SetValue("ComponentVersion", versionNumber);
            descriptKey.SetValue("VendorURL", companyURLInfoString);
            descriptKey.SetValue("HelpURL", helpURLInfoString);
            descriptKey.SetValue("About", aboutInfoString);
            catidKey.Close();
            descriptKey.Close();
            classKey.Close();
        }
        /// <summary>
        ///	This function controls the removal of the class from the COM registry when the class is unistalled.  
        /// </summary>
        /// <remarks>
        ///	The method will remove all subkeys added to the class' regristration, including the CAPE-OPEN
        /// specific keys added in the <see cref ="RegisterFunction"/> method.
        /// </remarks>
        /// <param name = "t">The type of the class being unregistered.</param> 
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.ComUnregisterFunction()]
        public static void UnregisterFunction(Type t)
        {
            UnregisterHelper(Microsoft.Win32.RegistryKey.OpenBaseKey(Microsoft.Win32.RegistryHive.ClassesRoot, Microsoft.Win32.RegistryView.Registry32), t);
            UnregisterHelper(Microsoft.Win32.RegistryKey.OpenBaseKey(Microsoft.Win32.RegistryHive.ClassesRoot, Microsoft.Win32.RegistryView.Registry64), t);
        }

        private static void UnregisterHelper(Microsoft.Win32.RegistryKey baseKey, Type t)
        {
            String keyname = String.Concat("CLSID\\{", t.GUID.ToString(), "}");
            baseKey.DeleteSubKeyTree(keyname, false);
        }

        /// <summary>
        ///	Initialization can be performed here. 
        /// </summary>
        /// <remarks>
        /// <para>The CAPE_OPEN object can allocated resources during this method. This is 
        /// called after the object constructor by the PME. .</para>
        /// <para>There are no input or output arguments for this method.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s), specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        virtual public void Initialize()
        {
        }


        /// <summary>
        ///	Clean-up tasks can be performed here. 
        /// </summary>
        /// <remarks>
        /// <para>The CAPE-OPEN object should releases all of its allocated resources during this call. This is 
        /// called before the object destructor by the PME. Terminate may check if the data has been 
        /// saved and return an error if not.</para>
        /// <para>There are no input or output arguments for this method.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        virtual public void Terminate()
        {
            this.Dispose();
        }

        /// <summary>
        ///	Gets the component's collection of parameters. 
        /// </summary>
        /// <remarks>
        /// <para>Return the collection of Public Parameters (i.e. 
        /// <see cref = "ICapeCollection"/>.</para>
        /// <para>These are delivered as a collection of elements exposing the interface 
        /// <see cref = "ICapeParameter"/>. From there, the client could extract the 
        /// <see cref = "ICapeParameterSpec"/> interface or any of the typed
        /// interfaces such as <see cref = "ICapeRealParameterSpec"/>, once the client 
        /// establishes that the Parameter is of type double.</para>
        /// </remarks>
        /// <value>The parameter collection of the unit operation.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        //[System.ComponentModel.EditorAttribute(typeof(ParameterCollectionEditor), typeof(System.Drawing.Design.UITypeEditor))]
        [System.ComponentModel.CategoryAttribute("Parameter Collection")]
        [System.ComponentModel.TypeConverter(typeof(ParameterCollectionTypeConverter))]
        public ParameterCollection Parameters
        {
            get
            {
                return m_Parameters;
            }
        }

        /// <summary>
        /// Validates the PMC. 
        /// </summary>
        /// <remarks>
        /// <para>Validates the parameter collection. This base-class implementation of this method 
        /// traverses the parameter collections and calls the  <see cref = "Validate"/> method of each 
        /// member parameter. The PMC is valid if all parameters are valid, which is 
        /// signified by the Validate method returning <c>true</c>.</para>
        /// </remarks>
        /// <returns>
        /// <para>true, if the unit is valid.</para>
        /// <para>false, if the unit is not valid.</para>
        /// </returns>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeBadCOParameter">ECapeBadCOParameter</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        public virtual bool Validate(ref String message)
        {
            message = "Object is valid.";
            this.m_ValidationMessage = message;
            for (int i = 0; i < this.Parameters.Count; i++)
            {
                String testString = String.Empty;
                if (!m_Parameters[i].Validate(ref testString))
                {
                    message = testString;
                    this.m_ValidationMessage = message;
                    return false;
                }
            }
            return true;
        }

        /// <summary>
        ///	Displays the PMC graphic interface, if available.
        /// </summary>
        /// <remarks>
        /// <para>By default, this method throws a <see cref="CapeNoImplException">CapeNoImplException</see>
        /// that according to the CAPE-OPEN specification, is interpreted by the process
        /// modeling environment as indicating that the PMC does not have a editor 
        /// GUI, and the PME must perform editing steps.</para>
        /// <para>In order for a PMC to provide its own editor, the Edit method will
        /// need to be overridden to create a graphical editor. When the user requests the flowheet
        /// to show the editor, this method will be called to edit the unit. Overriden classes should
        /// not return a failure (throw and exception) as this will be interpreted by the flowsheeting 
        /// tool as the unit not providing its own editor.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        public virtual System.Windows.Forms.DialogResult Edit()
        {
            throw new CapeNoImplException("No Object Editor");
        }


        /// <summary>
        ///	Gets and sets the component's simulation context.
        /// </summary>
        /// <remarks>
        /// This method provides access to the COSE's interfaces <see cref ="ICapeDiagnostic"/>, 
        /// <see cref ="ICapeMaterialTemplateSystem"/> and <see cref ="ICapeCOSEUtilities"/>.
        /// </remarks>
        /// <value>The simulation context assigned by the Flowsheeting Environment.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        public ICapeSimulationContext SimulationContext
        {
            get
            {
                return m_SimulationContext;
            }
            set
            {
                m_SimulationContext = value;
            }
        }
        /// <summary>
        ///	Gets the component's flowsheet monitoring object.
        /// </summary>
        /// <remarks>
        /// This method provides access to the COSE's interfaces <see cref ="ICapeDiagnostic"/>, 
        /// <see cref ="ICapeMaterialTemplateSystem"/> and <see cref ="ICapeCOSEUtilities"/>.
        /// </remarks>
        /// <value>The simulation context assigned by the Flowsheeting Environment.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        public ICapeFlowsheetMonitoring FlowsheetMonitoring
        {
            get
            {
                if (m_SimulationContext is ICapeFlowsheetMonitoring)
                {
                    return (ICapeFlowsheetMonitoring)m_SimulationContext;
                }
                return null;
            }
        }

        /// <summary>
        /// Throws an exception and exposes the exception object.
        /// </summary>
        /// <remarks>
        /// This method allows the derived class to conform to the CAPE-OPEN error handling standards and still use .Net 
        /// exception handling. In order to use this class, create an exception object that derives from <see cref ="ECapeUser"/>.
        /// Use the exception object as the argument to this function. As a result, the information in the expcetion will be exposed using the CAPE-OPEN 
        /// exception handing and will be thrown to .Net clients.
        /// </remarks>
        /// <param name="exception">The exception that will the throw.</param>
        public void throwException(System.Exception exception)
        {
            p_Exception = exception;
            throw this.p_Exception;
        }

        // ECapeRoot method
        // returns the message string in the System.ApplicationException.
        /// <summary>
        /// The name of the exception being thrown.
        /// </summary>
        /// <remarks>
        /// The name of the exception being thrown.
        /// </remarks>
        /// <value>
        /// The name of the exception being thrown.
        /// </value>
        [System.ComponentModel.BrowsableAttribute(false)]
        String ECapeRoot.Name
        {
            get
            {
                if (p_Exception is ECapeRoot) return ((ECapeRoot)p_Exception).Name;
                return "";
            }
        }

        /// <summary>
        /// Code to designate the subcategory of the error. 
        /// </summary>
        /// <remarks>
        /// The assignment of values is left to each implementation. So that is a 
        /// proprietary code specific to the CO component provider. By default, set to 
        /// the CAPE-OPEN error HRESULT <see cref = "CapeErrorInterfaceHR"/>.
        /// </remarks>
        /// <value>
        /// The HRESULT value for the exception.
        /// </value>
        [System.ComponentModel.BrowsableAttribute(false)]
        int ECapeUser.code
        {
            get
            {
                return ((ECapeUser)p_Exception).code;
            }
        }

        /// <summary>
        /// The description of the error.
        /// </summary>
        /// <remarks>
        /// The error description can include a more verbose description of the condition that
        /// caused the error.
        /// </remarks>
        /// <value>
        /// A string description of the exception.
        /// </value>
        [System.ComponentModel.BrowsableAttribute(false)]
        String ECapeUser.description
        {
            get
            {
                return ((ECapeUser)p_Exception).description;
            }
        }

        /// <summary>
        /// The scope of the error.
        /// </summary>
        /// <remarks>
        /// This property provides a list of packages where the error occurred. 
        /// For example <see cref = "ICapeIdentification"/>.
        /// </remarks>
        /// <value>The source of the error.</value>
        [System.ComponentModel.BrowsableAttribute(false)]
        String ECapeUser.scope
        {
            get
            {
                return ((ECapeUser)p_Exception).scope;
            }
        }

        /// <summary>
        /// The name of the interface where the error is thrown. This is a mandatory field."
        /// </summary>
        /// <remarks>
        /// The interface that the error was thrown.
        /// </remarks>
        /// <value>The name of the interface.</value>
        [System.ComponentModel.BrowsableAttribute(false)]
        String ECapeUser.interfaceName
        {
            get
            {
                return ((ECapeUser)p_Exception).interfaceName;
            }
        }

        /// <summary>
        /// The name of the operation where the error is thrown. This is a mandatory field.
        /// </summary>
        /// <remarks>
        /// This field provides the name of the operation being perfomed when the exception was raised.
        /// </remarks>
        /// <value>The operation name.</value>
        [System.ComponentModel.BrowsableAttribute(false)]
        String ECapeUser.operation
        {
            get
            {
                return ((ECapeUser)p_Exception).operation;
            }
        }

        /// <summary>
        /// An URL to a page, document, web site,  where more information on the error can be found. The content of this information is obviously implementation dependent.
        /// </summary>
        /// <remarks>
        /// This field provides an internet URL where more information about the error can be found.
        /// </remarks>
        /// <value>The URL.</value>
        [System.ComponentModel.BrowsableAttribute(false)]
        String ECapeUser.moreInfo
        {
            get
            {
                return ((ECapeUser)p_Exception).moreInfo;
            }
        }

        /// <summary>
        /// Writes a message to the terminal.
        /// </summary>
        /// <remarks>
        /// <para>Write a string to the terminal.</para>
        /// <para>This method is called when a message needs to be brought to the user’s attention.
        /// The implementation should ensure that the string is written out to a dialogue box or 
        /// to a message list that the user can easily see.</para>
        /// <para>A priori this message has to be displayed as soon as possible to the user.</para>
        /// </remarks>
        /// <param name = "message">The text to be displayed.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public void PopUpMessage(System.String message)
        {
            if (m_SimulationContext != null)
            {
                if (m_SimulationContext is ICapeDiagnostic)
                {
                    ((ICapeDiagnostic)this.m_SimulationContext).PopUpMessage(message);
                }
            }
        }
        /// <summary>
        /// Writes a string to the PME's log file.
        /// </summary>
        /// <remarks>
        /// <para>Write a string to a log.</para>
        /// <para>This method is called when a message needs to be recorded for logging purposes. 
        /// The implementation is expected to write the string to a log file or other journaling 
        /// device.</para>
        /// </remarks>
        /// <param name = "message">The text to be logged.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public void LogMessage(System.String message)
        {
            if (m_SimulationContext != null)
            {
                if (m_SimulationContext is ICapeDiagnostic)
                {
                    ((ICapeDiagnostic)this.m_SimulationContext).LogMessage(message);
                }
            }
        }
    };
}
