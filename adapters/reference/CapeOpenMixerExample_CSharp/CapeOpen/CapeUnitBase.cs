using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{
    class SelectedReportConverter : System.ComponentModel.StringConverter
    {

        override public bool GetStandardValuesSupported(System.ComponentModel.ITypeDescriptorContext context)
        {
            return true;
        }
        override public System.ComponentModel.TypeConverter.StandardValuesCollection GetStandardValues(System.ComponentModel.ITypeDescriptorContext context)
        {
            CapeUnitBase unit = (CapeUnitBase)context.Instance;
            return new System.ComponentModel.TypeConverter.StandardValuesCollection(unit.Reports);
        }

        override public bool GetStandardValuesExclusive(System.ComponentModel.ITypeDescriptorContext context)
        {
            return true;
        }
    };

    /// <summary>
    /// Abstract base class to be used to develop unit operation models. 
    /// </summary>
    /// <remarks>
    /// This abstract class contains all required functionality for a unit operation
    /// PMC except the <c>Calculate()</c> method, which is a pure virtual function that 
    /// must be overridden. To use, add  parameters and ports to the appropriate collection 
    /// and implement the <c>Calculate()</c> method.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(IUnitOperationValidatedEventArgs))]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public abstract class CapeUnitBase : CapeObjectBase,
        ICapeUnit,
        ICapeUnitCOM,
        ICapeUnitReport,
        ICapeUnitReportCOM
    //  IPersist,
    //  IPersistStream,
    //  IPersistStreamInit
    {
        private PortCollection m_Ports;
        private CapeValidationStatus m_ValStatus;
        // private bool m_dirty;
        private String m_selecetdReport;
        private System.Collections.Generic.List<String> m_Reports;
        // Track whether Dispose has been called.
        private bool _disposed;
        
        /// <summary>
        /// Gets the collection of unit operation ports. 
        /// </summary>
        /// <remarks>
        /// Return type is System.Object and this method is simply here for classic 
        /// COM-based CAPE-OPEN interop.
        /// </remarks>
        /// <value>The port collection of the unit operation.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        Object ICapeUnitCOM.ports
        {
            get
            {
                return m_Ports;
            }
        }

        /// <summary>
        ///	Gets the list of possible reports for the unit operation.
        /// </summary>
        /// <value>
        /// Return type is System.Object and this method is simply here for 
        /// classic COM-based CAPE-OPEN interop.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        Object ICapeUnitReportCOM.reports
        {
            get
            {
                return m_Reports.ToArray();
            }
        }


        /// <summary>
        ///	Produces the active report for the unit operation.
        /// </summary>
        /// <remarks>
        ///	Produce the designated report. If no value has been set, it produces the default report.
        /// </remarks>
        /// <param name = "message">String containing the text for the currently selected report.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        void ICapeUnitReportCOM.ProduceReport(ref string message)
        {
            message = this.ProduceReport();
        }
    
    /// <summary>
        ///	Constructor for the unit operation.
        /// </summary>
        /// <remarks>
        /// This method is creates the port and parameter collections for the unit 
        /// operation. As a result, ports and parameters can be added in the constructor
        /// for the derived unt or during the <c>Initialize()</c> call. 
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeLicenceError">ECapeLicenceError</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        public CapeUnitBase()
            : base()
        {
            m_Ports = new PortCollection();
            m_Ports.AddingNew += new System.ComponentModel.AddingNewEventHandler(m_Ports_AddingNew);
            m_Ports.ListChanged += new System.ComponentModel.ListChangedEventHandler(m_Ports_ListChanged);
            m_ValStatus = CapeValidationStatus.CAPE_NOT_VALIDATED;
            m_Reports = new System.Collections.Generic.List<String>();
            m_Reports.Add("Default Report");
            m_selecetdReport = "Default Report";
        }

        /// <summary>
        /// Finalizer for the <see cref = "CapeUnitBase"/> class.
        /// </summary>
        /// <remarks>
        /// This will finalize the current instance of the class.
        /// </remarks>
        ~CapeUnitBase()
        {
            this.Dispose(true);
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
        /// <param name = "objectToBeCopied">The unit operation being cloned.</param>
        public CapeUnitBase(CapeUnitBase objectToBeCopied)
            : base((CapeObjectBase)objectToBeCopied)
        {
            m_Ports = (PortCollection)objectToBeCopied.Ports.Clone();
            m_Reports.AddRange(objectToBeCopied.Reports);
            m_ValStatus = CapeValidationStatus.CAPE_NOT_VALIDATED;
            this.m_selecetdReport = objectToBeCopied.selectedReport;
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
                    foreach (UnitPort port in m_Ports)
                        port.Disconnect();
                    m_Ports.Clear();
                    _disposed = true;
                }
                base.Dispose(disposing);
            }
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
            Type unitType = this.GetType();
            CapeUnitBase retVal = (CapeUnitBase)Activator.CreateInstance(unitType);
            retVal.Parameters.Clear();
            foreach (CapeParameter param in this.Parameters)
            {
                retVal.Parameters.Add((CapeParameter)param.Clone());
            }
            retVal.Ports.Clear();
            foreach (UnitPort port in this.Ports)
            {
                retVal.Ports.Add((UnitPort)port.Clone());
            }
            retVal.Reports.Clear();
            retVal.Reports.AddRange(this.m_Reports);
            retVal.selectedReport = m_selecetdReport;
            retVal.SimulationContext = this.SimulationContext;
            return retVal;
        }

        /// <summary>
        /// Represents the method that will handle the changing the name of a component.
        /// </summary>
        /// <remarks>
        /// When you create a ComponentNameChangedHandler delegate, you identify the method that will handle the event. To associate the event with your event handler, add an 
        /// instance of the delegate to the event. The event handler is called whenever the event occurs, unless you remove the delegate. For more information about delegates, 
        /// see Events and Delegates.
        /// </remarks>
        /// <param name = "sender">The PMC that is the source .</param>
        /// <param name = "args">A <see cref = "System.ComponentModel.ListChangedEventArgs">System.ComponentModel.ListChangedEventArgs</see> that provides information about the name change.</param>
        void m_Ports_ListChanged(object sender, System.ComponentModel.ListChangedEventArgs args)
        {
            OnPortCollectionListChanged(args);
        }

        /// <summary>
        /// Represents the method that will handle the changing the name of a component.
        /// </summary>
        /// <remarks>
        /// When you create a ComponentNameChangedHandler delegate, you identify the method that will handle the event. To associate the event with your event handler, add an 
        /// instance of the delegate to the event. The event handler is called whenever the event occurs, unless you remove the delegate. For more information about delegates, 
        /// see Events and Delegates.
        /// </remarks>
        /// <param name = "sender">The PMC that is the source .</param>
        /// <param name = "args">A <see cref = "System.ComponentModel.AddingNewEventArgs">System.ComponentModel.AddingNewEventArgs</see> that provides information about the name change.</param>
        void m_Ports_AddingNew(object sender, System.ComponentModel.AddingNewEventArgs args)
        {
            OnPortCollectionAddingNew(args);
        }

        /// <summary>
        /// Occurs when the list or an item in the list changes.
        /// </summary>
        /// <remarks>ListChanged notifications for item value changes are only raised if the 
        /// list item type implements the INotifyPropertyChanged interface.</remarks> 
        public event System.ComponentModel.ListChangedEventHandler PortCollectionListChanged;

        /// <summary>
        /// Occurs when the list or an item in the list changes.
        /// </summary>
        /// <remarks>ListChanged notifications for item value changes are only raised if the 
        /// list item type implements the INotifyPropertyChanged interface.</remarks> 
        /// <param name = "args">A <see cref = "System.ComponentModel.ListChangedEventArgs">System.ComponentModel.ListChangedEventArgs</see> that contains information about the event.</param>
        protected void OnPortCollectionListChanged(System.ComponentModel.ListChangedEventArgs args)
        {
            if (PortCollectionListChanged != null)
            {
                PortCollectionListChanged(this, args);
            }
        }

        /// <summary>
        /// Occurs when the user Adds a new element to the port collection.
        /// </summary>
        /// <remarks>The event to be handles when the name of the PMC is changed.</remarks> 
        public event System.ComponentModel.AddingNewEventHandler PortCollectionAddingNew;

        /// <summary>
        /// Occurs before an item is added to the list.
        /// </summary>
        /// <remarks>
        /// The AddingNew event occurs before a new object is added to the collection 
        /// represented by the Items property. This event is raised after the AddNew method is 
        /// called, but before the new item is created and added to the internal list. By 
        /// handling this event, the programmer can provide custom item creation and insertion 
        /// behavior without being forced to derive from the BindingList&gt;T&lt; class. 
        /// </remarks>
        /// <param name = "args">A <see cref = "System.ComponentModel.AddingNewEventArgs">System.ComponentModel.AddingNewEventArgs</see> that contains information about the event.</param>
        protected void OnPortCollectionAddingNew(System.ComponentModel.AddingNewEventArgs args)
        {
            if (PortCollectionAddingNew != null)
            {
                PortCollectionAddingNew(this, args);
            }
        }


        /// <summary>
        /// Occurs when the user validates the unit operation.
        /// </summary>
        public event UnitOperationValidatedHandler UnitOperationValidated;
        /// <summary>
        /// Occurs when a unit operation is validated.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnUnitOperationValidated</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnUnitOperationValidated</c> in a derived class, be sure to call the base class's <c>OnUnitOperationValidated</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "args">A <see cref = "UnitOperationValidatedEventArgs">UnitOperationValidatedEventArgs</see> that contains information about the event.</param>
        protected void OnUnitOperationValidated(UnitOperationValidatedEventArgs args)
        {
            if (UnitOperationValidated != null)
            {
                UnitOperationValidated(this, args);
            }
        }

        /// <summary>
        /// Occurs when the user begins the calculation of the unit operation.
        /// </summary>
        public event UnitOperationBeginCalculationHandler UnitOperationBeginCalculation;
        /// <summary>
        /// Occurs at the start of a unit operation calculation process.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnUnitOperationBeginCalculation</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnUnitOperationBeginCalculation</c> in a derived class, be sure to call the base class's <c>OnUnitOperationBeginCalculation</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "message">A string that contains information about the the calculation.</param>
        protected void OnUnitOperationBeginCalculation(String message)
        {
            UnitOperationBeginCalculationEventArgs args = new UnitOperationBeginCalculationEventArgs(this.ComponentName, message);
            if (UnitOperationBeginCalculation != null)
            {
                UnitOperationBeginCalculation(this, args);
            }
        }

        /// <summary>
        /// Occurs at the completion of a calculation of a unit operation.
        /// </summary>
        public event UnitOperationEndCalculationHandler UnitOperationEndCalculation;
        /// <summary>
        /// Occurs at the completion of a unit operation calculation process.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnUnitOperationEndCalculation</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnUnitOperationEndCalculation</c> in a derived class, be sure to call the base class's <c>OnUnitOperationBeginCalculation</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "message">A string that contains information about the the calculation.</param>
        protected void OnUnitOperationEndCalculation(string message)
        {
            UnitOperationEndCalculationEventArgs args = new UnitOperationEndCalculationEventArgs(this.ComponentName, message);
            if (UnitOperationEndCalculation != null)
            {
                UnitOperationEndCalculation(this, args);
            }
        }

        // Removed because no longer supporting IDisposable.

        //// Dispose(bool disposing) executes in two distinct scenarios.
        //// If disposing equals true, the method has been called directly
        //// or indirectly by a user's code. Managed and unmanaged resources
        //// can be disposed.
        //// If disposing equals false, the method has been called by the
        //// runtime from inside the finalizer and you should not reference
        //// other objects. Only unmanaged resources can be disposed.
        ///// <summary>
        ///// Releases the unmanaged resources used by the CapeIdentification object and optionally releases 
        ///// the managed resources.
        ///// </summary>
        ///// <remarks><para>This method is called by the public <see href="http://msdn.microsoft.com/en-us/library/system.componentmodel.component.dispose.aspx">Dispose</see>see> 
        ///// method and the <see href="http://msdn.microsoft.com/en-us/library/system.object.finalize.aspx">Finalize</see> method. 
        ///// <bold>Dispose()</bold> invokes the protected <bold>Dispose(Boolean)</bold> method with the disposing
        ///// parameter set to <bold>true</bold>. <see href="http://msdn.microsoft.com/en-us/library/system.object.finalize.aspx">Finalize</see> 
        ///// invokes <bold>Dispose</bold> with disposing set to <bold>false</bold>.</para>
        ///// <para>When the <italic>disposing</italic> parameter is <bold>true</bold>, this method releases all 
        ///// resources held by any managed objects that this Component references. This method invokes the 
        ///// <bold>Dispose()</bold> method of each referenced object.</para>
        ///// <para><bold>Notes to Inheritors</bold></para>
        ///// <para><bold>Dispose</bold> can be called multiple times by other objects. When overriding 
        ///// <bold>Dispose(Boolean)</bold>, be careful not to reference objects that have been previously 
        ///// disposed of in an earlier call to <bold>Dispose</bold>. For more information about how to 
        ///// implement <bold>Dispose(Boolean)</bold>, see <see href="http://msdn.microsoft.com/en-us/library/fs2xkftw.aspx">Implementing a Dispose Method</see>.</para>
        ///// <para>For more information about <bold>Dispose</bold> and <see href="http://msdn.microsoft.com/en-us/library/system.object.finalize.aspx">Finalize</see>, 
        ///// see <see href="http://msdn.microsoft.com/en-us/library/498928w2.aspx">Cleaning Up Unmanaged Resources</see> 
        ///// and <see href="http://msdn.microsoft.com/en-us/library/ddae83kx.aspx">Overriding the Finalize Method</see>.</para>
        ///// </remarks> 
        ///// <param name = "disposing">true to release both managed and unmanaged resources; false to release only unmanaged resources.</param>
        //protected override void Dispose(bool disposing)
        //{
        //    // Check to see if Dispose has already been called.
        //    if (!this.disposed)
        //    {
        //        // If disposing equals true, dispose all managed
        //        // and unmanaged resources.
        //        if (disposing)
        //        {
        //            // disconnect all ports and clear the list...
        //            foreach (UnitPort port in m_Ports){
        //                port.Disconnect();
        //            }
        //            m_Ports.Clear();

        //            // Dispose managed resources.
        //            component.Dispose();
        //        }
        //        base.Dispose(disposing);
        //    }
        //}

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
        public new static void RegisterFunction(Type t)
        {
            System.Reflection.Assembly assembly = t.Assembly;
            String versionNumber = (new System.Reflection.AssemblyName(assembly.FullName)).Version.ToString();

            String keyname = String.Concat("CLSID\\{", t.GUID.ToString(), "}\\Implemented Categories");
            Microsoft.Win32.RegistryKey catidKey = Microsoft.Win32.Registry.ClassesRoot.OpenSubKey(keyname, true);
            catidKey.CreateSubKey(COGuids.CapeOpenComponent_CATID);
            catidKey.CreateSubKey(COGuids.CapeUnitOperation_CATID);

            keyname = String.Concat("CLSID\\{", t.GUID.ToString(), "}\\InprocServer32");
            Microsoft.Win32.RegistryKey key = Microsoft.Win32.Registry.ClassesRoot.OpenSubKey(keyname, true);
            String[] keys = key.GetSubKeyNames();
            for (int i = 0; i < keys.Length; i++)
            {
                if (keys[i] == versionNumber)
                {
                    key.DeleteSubKey(keys[i]);
                }
            }
            key.SetValue("CodeBase", assembly.CodeBase);
            key.Close();

            key = Microsoft.Win32.Registry.ClassesRoot.OpenSubKey(String.Concat("CLSID\\{", t.GUID.ToString(), "}"), true);
            keyname = String.Concat("CLSID\\{", t.GUID.ToString(), "}\\CapeDescription");

            Object[] attributes = t.GetCustomAttributes(false);
            String nameInfoString = t.FullName;
            String descriptionInfoString = "";
            String versionInfoString = "";
            String companyURLInfoString = "";
            String helpURLInfoString = "";
            String aboutInfoString = "";
            for (int i = 0; i < attributes.Length; i++)
            {
                if (attributes[i] is CapeFlowsheetMonitoringAttribute) catidKey.CreateSubKey(COGuids.CATID_MONITORING_OBJECT);
                if (attributes[i] is CapeConsumesThermoAttribute) catidKey.CreateSubKey(COGuids.Consumes_Thermo_CATID);
                if (attributes[i] is CapeSupportsThermodynamics10Attribute) catidKey.CreateSubKey(COGuids.SupportsThermodynamics10_CATID);
                if (attributes[i] is CapeSupportsThermodynamics11Attribute) catidKey.CreateSubKey(COGuids.SupportsThermodynamics11_CATID);
                if (attributes[i] is CapeNameAttribute) nameInfoString = ((CapeNameAttribute)attributes[i]).Name;
                if (attributes[i] is CapeDescriptionAttribute) descriptionInfoString = ((CapeDescriptionAttribute)attributes[i]).Description;
                if (attributes[i] is CapeVersionAttribute) versionInfoString = ((CapeVersionAttribute)attributes[i]).Version;
                if (attributes[i] is CapeVendorURLAttribute) versionInfoString = ((CapeVendorURLAttribute)attributes[i]).VendorURL;
                if (attributes[i] is CapeHelpURLAttribute) helpURLInfoString = ((CapeHelpURLAttribute)attributes[i]).HelpURL;
                if (attributes[i] is CapeAboutAttribute) aboutInfoString = ((CapeAboutAttribute)attributes[i]).About;
            }
            catidKey.Close();
            key = Microsoft.Win32.Registry.ClassesRoot.CreateSubKey(keyname);
            key.SetValue("Name", nameInfoString);
            key.SetValue("Description", descriptionInfoString);
            key.SetValue("CapeVersion", versionInfoString);
            key.SetValue("ComponentVersion", versionNumber);
            key.SetValue("VendorURL", companyURLInfoString);
            key.SetValue("HelpURL", helpURLInfoString);
            key.SetValue("About", aboutInfoString);
            key.Close();
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
        public new static void UnregisterFunction(Type t)
        {
            String keyname = String.Concat("CLSID\\{", t.GUID.ToString(), "}");
            Microsoft.Win32.RegistryKey key = Microsoft.Win32.Registry.ClassesRoot.OpenSubKey(keyname, true);
            String[] keyNames = key.GetSubKeyNames();
            for (int i = 0; i < keyNames.Length; i++)
            {
                key.DeleteSubKeyTree(keyNames[i]);
            }
            String[] valueNames = key.GetValueNames();
            for (int i = 0; i < valueNames.Length; i++)
            {
                key.DeleteValue(valueNames[i]);
            }
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
        public override System.Windows.Forms.DialogResult Edit()
        {
            BaseUnitEditor editor = new BaseUnitEditor(this);
            editor.ShowDialog();
            return editor.DialogResult;
        }

        /// <summary>
        /// Gets the collection of unit operation ports.
        /// </summary>
        /// <remarks>
        /// <para>Return an interface to a collection containing the list of unit ports 
        /// (e.g. <see cref = "PortCollection"/>).</para>
        /// <para>Return the collection of unit ports (i.e. ICapeCollection). 
        /// These are delivered as a collection of elements exposing the interfaces 
        /// <see cref = "ICapeUnitPort"/>.</para>
        /// </remarks>
        /// <value>The port collection of the unit operation.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        //        [System.ComponentModel.EditorAttribute(typeof(capePortCollectionEditor), typeof(System.Drawing.Design.UITypeEditor))]
        [System.ComponentModel.CategoryAttribute("ICapeUnit")]
        [System.ComponentModel.DescriptionAttribute("Unit Operation Port Collection. Click on the (...) button to edit collection.")]
        [System.ComponentModel.TypeConverter(typeof(PortCollectionTypeConverter))]
        public PortCollection Ports
        {
            get
            {
                return m_Ports;
            }
        }

        /// <summary>
        /// Gets the flag to indicate the unit operation's validation status
        /// </summary>
        /// <remarks>
        /// <para> Get the flag that indicates whether the Flowsheet Unit is valid (e.g. some 
        /// parameter values have changed but they have not been validated by using 
        /// Validate). It has three possible values:</para>
        /// <list type="bullet"> 
        /// <item>notValidated(CAPE_NOT_VALIDATED)</item>
        /// <description>The unit’s validate() method has not 
        /// been called since the last operation that could have changed the validation 
        /// status of the unit, for example an update to a parameter value of a connection 
        /// to a port.</description>
        /// <item>invalid(CAPE_INVALID)</item>
        /// <description>The last time the unit’s validate() method was 
        /// called it returned false.</description>
        /// <item>valid(CAPE_VALID)</item>
        /// <description>The last time the unit’s validate() method was 
        /// called it returned true.</description>
        /// </list>
        /// </remarks>
        /// <value>A flag that indiciates the validation status of the unit operation.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        /// <see cref= "CapeValidationStatus">CapeValidationStatus</see>.
        [System.ComponentModel.CategoryAttribute("ICapeUnit")]
        [System.ComponentModel.DescriptionAttribute("Validation status of the unit. Either CAPE_VALID, CAPE_NOT_VALIDATED or CAPE_INVALID")]
        public virtual CapeValidationStatus ValStatus
        {
            get
            {
                return m_ValStatus;
            }
        }


        /// <summary>
        /// Gets the message returned during the last validation of the unit operation.
        /// </summary>
        /// <remarks>
        /// Gets the message that was retured fromt he last attempt to validate the Flowsheet Unit (e.g. some 
        /// parameter values have changed but they have not been validated by using 
        /// Validate). 
        /// </remarks>
        /// <value>The message returned during the last validation of the unit operation.</value>
        /// <see cref= "CapeValidationStatus">CapeValidationStatus</see>.
        [System.ComponentModel.CategoryAttribute("ICapeUnit")]
        [System.ComponentModel.DescriptionAttribute("Validation message of the unit.")]
        public virtual string ValidationMessage
        {
            get
            {
                return this.m_ValidationMessage;
            }
        }

        /// <summary>
        ///	Executes the necessary calculations involved in the unit operation model.
        /// </summary>
        /// <remarks>
        /// <para>This method is called by the PME to execute the calculation of the unit operation. The calculation process
        /// first fires the <see cref = "UnitOperationBeginCalculation" /> event. After the event is fired, the 
        /// <see cref = "OnCalculate"/> method is called. Derived classes must implement the 
        /// <see cref = "OnCalculate"/> method. After the unit has completed its calculation, this method fires the 
        /// <see cref = "UnitOperationEndCalculation"/> event.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeTimeOut">ECapeTimeOut</exception>
        /// <exception cref = "ECapeSolvingError">ECapeSolvingError</exception>
        /// <exception cref = "ECapeLicenceError">ECapeLicenceError</exception>
        void ICapeUnitCOM.Calculate()
        {
            this.OnUnitOperationBeginCalculation("Starting Calculation");
            this.OnCalculate();
            this.OnUnitOperationEndCalculation("Calculation completed normally.");
        }


        /// <summary>
        ///	Executes the necessary calculations involved in the unit operation model.
        /// </summary>
        /// <remarks>
        /// <para>This method is called by the PME to execute the calculation of the unit operation. The calculation process
        /// first fires the <see cref = "UnitOperationBeginCalculation" /> event. After the event is fired, the 
        /// <see cref = "OnCalculate"/> method is called. Derived classes must implement the 
        /// <see cref = "OnCalculate"/> method. After the unit has completed its calculation, this method fires the 
        /// <see cref = "UnitOperationEndCalculation"/> event.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeTimeOut">ECapeTimeOut</exception>
        /// <exception cref = "ECapeSolvingError">ECapeSolvingError</exception>
        /// <exception cref = "ECapeLicenceError">ECapeLicenceError</exception>
        void ICapeUnit.Calculate()
        {
            this.OnUnitOperationBeginCalculation("Starting Calculation");
            this.OnCalculate();
            this.OnUnitOperationEndCalculation("Calculation completed normally.");
        }

        /// <summary>
        ///	Executes the necessary calculations involved in the unit operation model.
        /// </summary>
        /// <remarks>
        /// <para>This method is called by the PME to execute the calculation of the unit operation. The calculation process
        /// first fires the <see cref = "UnitOperationBeginCalculation" /> event. After the event is fired, the 
        /// <see cref = "OnCalculate"/> method is called. Derived classes must implement the 
        /// <see cref = "OnCalculate"/> method. After the unit has completed its calculation, this method fires the 
        /// <see cref = "UnitOperationEndCalculation"/> event.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeTimeOut">ECapeTimeOut</exception>
        /// <exception cref = "ECapeSolvingError">ECapeSolvingError</exception>
        /// <exception cref = "ECapeLicenceError">ECapeLicenceError</exception>
        void OnCalculate()
        {
            this.OnUnitOperationBeginCalculation("Starting Calculation");
            this.Calculate();
            this.OnUnitOperationEndCalculation("Calculation completed normally.");
        }

        /// <summary>
        ///	This method is called by the Calculate method to perform necessary calculations involved in the 
        ///	unit operation model.
        /// </summary>
        /// <remarks>
        /// <para>The Flowsheet Unit performs its calculation, that is, computes the variables 
        /// that are missing at this stage in the complete description of the input and 
        /// output streams and computes any public parameter value that needs to be 
        /// displayed. OnCalculate will be able to do progress monitoring and checks for 
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
        abstract protected void Calculate();

        /// <summary>
        /// Validates the unit operation. 
        /// </summary>
        /// <remarks>
        /// <para>Sets the flag that indicates whether the Flowsheet Unit is valid by 
        /// validating the ports and parameters of the Flowsheet Unit. For example, this 
        /// method could check that all mandatory ports have connections and that the 
        /// values of all parameters are within bounds.</para>
        /// <para>Note that the Simulation Executive can call the Validate routine at any 
        /// time, in particular it may be called before the executive is ready to call 
        /// the Calculate method. This means that Material Objects connected to unit ports 
        /// may not be correctly configured when Validate is called. The recommended approach 
        /// is for this method to validate parameters and ports but not Material Object 
        /// configuration. A second level of validation to check Material Objects can be
        /// implemented as part of Calculate, when it is reasonable to expect that the 
        /// Material Objects connected to ports will be correctly configured. </para>
        /// <para>The base-class implementation of this method traverses the port and 
        /// parameter collections and calls the  <see cref = "Validate"/> method of each 
        /// member. The unit is valid if all port and parameters are valid, which is 
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
        public override bool Validate(ref String message)
        {
            // m_dirty = true;
            UnitOperationValidatedEventArgs args;
            if (!base.Validate(ref message))
            {
                args = new UnitOperationValidatedEventArgs(this.ComponentName, message, m_ValStatus, CapeValidationStatus.CAPE_INVALID);
                m_ValStatus = CapeValidationStatus.CAPE_INVALID;
                OnUnitOperationValidated(args);
                this.m_ValidationMessage = message;
                return false;
            }
            for (int i = 0; i < this.Ports.Count; i++)
            {
                if (m_Ports[i].connectedObject == null)
                {
                    message = String.Concat("Port ", ((CapeIdentification)m_Ports[i]).ComponentName, " does not have a connected object.");
                    this.m_ValidationMessage = message;
                    args = new UnitOperationValidatedEventArgs(this.ComponentName, message, m_ValStatus, CapeValidationStatus.CAPE_INVALID);
                    m_ValStatus = CapeValidationStatus.CAPE_INVALID;
                    OnUnitOperationValidated(args);
                    return false;
                }
            }
            message = "Unit is valid.";
            this.m_ValidationMessage = message;
            args = new UnitOperationValidatedEventArgs(this.ComponentName, message, m_ValStatus, CapeValidationStatus.CAPE_INVALID);
            m_ValStatus = CapeValidationStatus.CAPE_VALID;
            OnUnitOperationValidated(args);
            return true;
        }
        /*        // IPersist
                /// <summary>This method retrieves the class identifier (CLSID) of an object. The 
                /// CLSID is a unique value that identifies the code that can manipulate the 
                /// persistent data.</summary>
                /// <remarks>
                /// The GetClassID method retrieves the class identifier (CLSID) for an object, 
                /// used in later operations to load object-specific code into the caller's context.
                /// </remarks>
                /// <param name ="pClassID"><para>Pointer to the location of the CLSID on return.</para>
                /// <para>The CLSID is a globally unique identifier (GUID) that uniquely represents 
                /// an object class that defines the code that can manipulate the object's data. </para>
                /// </param>
                void IPersist.GetClassID(out Guid pClassID)
                {
                    pClassID = this.GetType().GUID;
                }

                // IPersistStream

                /// <summary>
                /// This method checks the object for changes since it was last saved.
                /// </summary>
                /// <remarks>
                /// This method checks whether an object has changed since it was last saved so you can 
                /// avoid losing information in objects that have not yet been saved.
                /// </remarks>
                /// <returns>
                /// <para>If the object has changed since it was last saved, the return value 
                /// is the HRESULT S_OK, which has a numerical value of 0. </para>
                /// <para>If the object has not changed since the last save, the return value 
                /// is the HRESULT S_FALSE, which has a numerical value of 1. </para>
                /// </returns>
                int IPersistStream.IsDirty()
                {
                    if (m_dirty) return 0;
                    return 1;
                }

                /// <summary>This method saves an object to the specified stream.</summary>
                /// <remarks>
                /// IPersistStream.Save saves an object into the specified stream and indicates 
                /// whether the object should reset its dirty flag.
                /// </remarks>
                /// <param name ="fClearDirty">Value that indicates whether to clear the dirty flag after the save 
                /// is complete. If TRUE, the flag should be cleared. If FALSE, the flag should be left unchanged. </param>
                /// <param name ="pStm">IStream pointer to the stream into which the object should be saved. </param>
                void IPersistStream.Save(System.Runtime.InteropServices.ComTypes.IStream pStm, bool fClearDirty)
                {
                    Byte[] arrLen = new Byte[2];
                    // Convert the string into a byte array    
                    System.IO.MemoryStream memoryStream = new System.IO.MemoryStream();
                    Object[] saveObjects = new Object[4];
                    saveObjects[0] = this.ComponentName;
                    saveObjects[1] = this.ComponentDescription;
                    saveObjects[2] = this.Parameters;
                    saveObjects[3] = m_Ports;
                    System.Runtime.Serialization.Formatters.Binary.BinaryFormatter binaryFormatter = new System.Runtime.Serialization.Formatters.Binary.BinaryFormatter();
                    binaryFormatter.Serialize(memoryStream, saveObjects);
                    Byte[] bytes = memoryStream.ToArray();
                    memoryStream.Close();
                    // construct length (separate into two separate bytes)    
                    arrLen[0] = (Byte)(bytes.Length % 256);
                    arrLen[1] = (Byte)(bytes.Length / 256);
                    try
                    {
                        // Save the array in the stream    
                        pStm.Write(arrLen, 2, IntPtr.Zero);
                        pStm.Write(bytes, bytes.Length, IntPtr.Zero);
                        System.Runtime.InteropServices.Marshal.ReleaseComObject(pStm);
                        if (fClearDirty) m_dirty = false;
                    }
                    catch (System.Exception p_Ex)
                    {
                        System.Windows.Forms.MessageBox.Show(p_Ex.ToString());
                        System.Runtime.InteropServices.Marshal.ReleaseComObject(pStm);
                    }
                }

                /// <summary>This method initializes an object from the stream where it was previously saved.</summary>
                /// <remarks>
                /// This method loads an object from its associated stream.
                /// </remarks>
                /// <param name ="pStm">IStream pointer to the stream from which the object should be loaded. </param>
                void IPersistStream.Load(System.Runtime.InteropServices.ComTypes.IStream pStm)
                {
                    m_Loaded = true;
                    m_Ports.Clear();
                    this.Parameters.Clear();
                    int cb;
                    Byte[] arrLen = new Byte[2];
                    // Read the length of the string  
                    IntPtr pcb = IntPtr.Zero;
                    pStm.Read(arrLen, 2, IntPtr.Zero);
                    // Calculate the length    
                    cb = 256 * arrLen[1] + arrLen[0];
                    // Read the stream to get the string    
                    Byte[] bytes = new Byte[cb];
                    pStm.Read(bytes, cb, pcb);
                    System.Runtime.InteropServices.Marshal.ReleaseComObject(pStm);
                    // Deserialize byte array    
                    System.IO.MemoryStream memoryStream = new System.IO.MemoryStream(bytes);
                    System.Runtime.Serialization.Formatters.Binary.BinaryFormatter binaryFormatter = new System.Runtime.Serialization.Formatters.Binary.BinaryFormatter();
                    try
                    {
                        AppDomain domain = AppDomain.CurrentDomain;
                        domain.AssemblyResolve += new ResolveEventHandler(CapeOpen.CapeUnitBase.MyResolveEventHandler);
                        Object[] loadObject = (Object[])(binaryFormatter.Deserialize(memoryStream));
                        this.ComponentName = loadObject[0].ToString();
                        this.ComponentDescription = loadObject[1].ToString();
                        ParameterCollection parameters = (ParameterCollection)loadObject[2];
                        foreach (ICapeParameter param in parameters)
                        {
                            this.Parameters.Add(param);
                        }
                        m_Ports = (PortCollection)loadObject[3];
                        domain.AssemblyResolve -= new ResolveEventHandler(CapeOpen.CapeUnitBase.MyResolveEventHandler);
                    }
                    catch (System.Exception p_Ex)
                    {
                        System.Windows.Forms.MessageBox.Show(p_Ex.ToString());
                    }
                    memoryStream.Close();
                }

                /// <summary>This method returns the size, in bytes, of the stream needed to save the object.</summary>
                /// <remarks>
                /// <para>This method returns the size needed to save an object. </para>
                /// <para>You can call this method to determine the size and set the necessary 
                /// buffers before calling the Save method.</para>
                /// </remarks>
                /// <param name ="pcbSize">Pointer to a 64-bit unsigned integer value indicating the size, in bytes, of the stream needed to save this object. </param>
                void IPersistStream.GetSizeMax(out long pcbSize)
                {
                    pcbSize = 0;
                }
                // IPersistStreamInit

                /// <summary>
                /// This method checks the object for changes since it was last saved.
                /// </summary>
                /// <remarks>
                /// This method checks whether an object has changed since it was last saved so you can 
                /// avoid losing information in objects that have not yet been saved.
                /// </remarks>
                /// <returns>
                /// <para>If the object has changed since it was last saved, the return value 
                /// is the HRESULT S_OK, which has a numerical value of 0. </para>
                /// <para>If the object has not changed since the last save, the return value 
                /// is the HRESULT S_FALSE, which has a numerical value of 1. </para>
                /// </returns>
                int IPersistStreamInit.IsDirty()
                {
                    if (m_dirty) return 0;
                    return 1;
                }

                /// <summary>This method saves an object to the specified stream.</summary>
                /// <remarks>
                /// IPersistStream.Save saves an object into the specified stream and indicates 
                /// whether the object should reset its dirty flag.
                /// </remarks>
                /// <param name ="fClearDirty">Value that indicates whether to clear the dirty flag after the save 
                /// is complete. If TRUE, the flag should be cleared. If FALSE, the flag should be left unchanged. </param>
                /// <param name ="pStm">IStream pointer to the stream into which the object should be saved. </param>
                void IPersistStreamInit.Save(System.Runtime.InteropServices.ComTypes.IStream pStm, bool fClearDirty)
                {
                    Byte[] arrLen = new Byte[2];
                    // Convert the string into a byte array    
                    System.IO.MemoryStream memoryStream = new System.IO.MemoryStream();
                    Object[] saveObjects = new Object[4];
                    saveObjects[0] = this.ComponentName;
                    saveObjects[1] = this.ComponentDescription;
                    saveObjects[2] = this.Parameters;
                    saveObjects[3] = m_Ports;
                    System.Runtime.Serialization.Formatters.Binary.BinaryFormatter binaryFormatter = new System.Runtime.Serialization.Formatters.Binary.BinaryFormatter();
                    binaryFormatter.Serialize(memoryStream, saveObjects);
                    Byte[] bytes = memoryStream.ToArray();
                    memoryStream.Close();
                    // construct length (separate into two separate bytes)    
                    arrLen[0] = (Byte)(bytes.Length % 256);
                    arrLen[1] = (Byte)(bytes.Length / 256);
                    try
                    {
                        // Save the array in the stream    
                        pStm.Write(arrLen, 2, IntPtr.Zero);
                        pStm.Write(bytes, bytes.Length, IntPtr.Zero);
                        System.Runtime.InteropServices.Marshal.ReleaseComObject(pStm);
                        if (fClearDirty) m_dirty = false;
                    }
                    catch (System.Exception p_Ex)
                    {
                        System.Windows.Forms.MessageBox.Show(p_Ex.ToString());
                        System.Runtime.InteropServices.Marshal.ReleaseComObject(pStm);
                    }
                }

                /// <summary>This method initializes an object from the stream where it was previously saved.</summary>
                /// <remarks>
                /// This method loads an object from its associated stream.
                /// </remarks>
                /// <param name ="pStm">IStream pointer to the stream from which the object should be loaded. </param>
                void IPersistStreamInit.Load(System.Runtime.InteropServices.ComTypes.IStream pStm)
                {
                    m_Loaded = true;
                    m_Ports.Clear();
                    this.Parameters.Clear();
                    int cb;
                    Byte[] arrLen = new Byte[2];
                    // Read the length of the string  
                    IntPtr pcb = IntPtr.Zero;
                    pStm.Read(arrLen, 2, IntPtr.Zero);
                    // Calculate the length    
                    cb = 256 * arrLen[1] + arrLen[0];
                    // Read the stream to get the string    
                    Byte[] bytes = new Byte[cb];
                    pStm.Read(bytes, cb, pcb);
                    System.Runtime.InteropServices.Marshal.ReleaseComObject(pStm);
                    // Deserialize byte array    
                    System.IO.MemoryStream memoryStream = new System.IO.MemoryStream(bytes);
                    System.Runtime.Serialization.Formatters.Binary.BinaryFormatter binaryFormatter = new System.Runtime.Serialization.Formatters.Binary.BinaryFormatter();
                    try
                    {
                        AppDomain domain = AppDomain.CurrentDomain;
                        domain.AssemblyResolve += new ResolveEventHandler(CapeOpen.CapeUnitBase.MyResolveEventHandler);
                        Object[] loadObject = (Object[])(binaryFormatter.Deserialize(memoryStream));
                        this.ComponentName = loadObject[0].ToString();
                        this.ComponentDescription = loadObject[1].ToString();
                        ParameterCollection parameters = (ParameterCollection)loadObject[2];
                        foreach (ICapeParameter param in parameters)
                        {
                            this.Parameters.Add(param);
                        }
                        m_Ports = (PortCollection)loadObject[3];
                        domain.AssemblyResolve -= new ResolveEventHandler(CapeOpen.CapeUnitBase.MyResolveEventHandler);
                    }
                    catch (System.Exception p_Ex)
                    {
                        System.Windows.Forms.MessageBox.Show(p_Ex.ToString());
                    }
                    memoryStream.Close();
                }

                /// <summary>This method returns the size, in bytes, of the stream needed to save the object.</summary>
                /// <remarks>
                /// <para>This method returns the size needed to save an object. </para>
                /// <para>You can call this method to determine the size and set the necessary 
                /// buffers before calling the Save method.</para>
                /// </remarks>
                /// <param name ="pcbSize">Pointer to a 64-bit unsigned integer value indicating the size, in bytes, of the stream needed to save this object. </param>
                void IPersistStreamInit.GetSizeMax(out long pcbSize)
                {
                    pcbSize = 0;
                }

                /// <summary>Initializes an object to a default state. This method is to be called 
                /// instead of IPersistStreamInit.Load.</summary>
                /// <remarks>
                /// If the object has already been initialized with IPersistStreamInit.Load, then 
                /// this method must return E_UNEXPECTED.
                /// </remarks>
                void IPersistStreamInit.InitNew()
                {
                    if (m_Loaded)
                    {
                        CapeOpen.CapeUnexpectedException p_Ex = new CapeUnexpectedException("The object has already been initialized with IPersistStreamInit.Load.");
                        throw p_Ex;
                    }
                }
        */

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
        virtual public System.Collections.Generic.List<String> Reports
        {
            get
            {
                return m_Reports;
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
        virtual public String selectedReport
        {
            get
            {
                return m_selecetdReport;
            }
            set
            {
                m_selecetdReport = value;
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
        virtual public String ProduceReport()
        {
            String retVal = String.Empty;
            String validMessage = String.Empty;
            bool valid = this.Validate(ref validMessage);
            CapeOpen.CapeValidationStatus status = this.ValStatus;
            retVal = String.Concat(retVal, "Unit Operation: ", this.ComponentName, Environment.NewLine);
            retVal = String.Concat(retVal, "   Description: ", this.ComponentDescription, Environment.NewLine, Environment.NewLine);
            retVal = String.Concat(retVal, "Validation Status: ", this.ValStatus.ToString(), Environment.NewLine);
            retVal = String.Concat(retVal, "                   ", validMessage, Environment.NewLine, Environment.NewLine);
            retVal = String.Concat(retVal, "Parameters: ", Environment.NewLine);

            for (int i = 0; i < this.Parameters.Count; i++)
            {
                ICapeParameter p_Param = (ICapeParameter)this.Parameters[i];
                CapeIdentification p_ParamId = (CapeIdentification)p_Param;
                retVal = String.Concat(retVal, "     Parameter ", i.ToString(), ": ", p_ParamId.ComponentName, Environment.NewLine);
                retVal = String.Concat(retVal, "     Desription: ", p_ParamId.ComponentDescription, Environment.NewLine);
                CapeOpen.ICapeParameterSpec p_Spec = (ICapeParameterSpec)p_Param.Specification;
                retVal = String.Concat(retVal, "     Type: ", p_Spec.Type.ToString(), Environment.NewLine);
                valid = p_Param.Validate(validMessage);
                retVal = String.Concat(retVal, "     Validation Status: ", p_Param.ValStatus.ToString(), Environment.NewLine);
                retVal = String.Concat(retVal, "          ", validMessage, Environment.NewLine);
                if (p_Spec.Type == CapeParamType.CAPE_ARRAY)
                {
                    retVal = String.Concat(retVal, "     Values:", Environment.NewLine);
                    Object[] values;
                    if (p_Param.value is Object[])
                    {
                        values = (Object[])p_Param.value;
                        for (int j = 0; j < values.Length; j++)
                        {
                            ICapeParameter p_ParamArrayElement;
                            if (values[j] is ICapeParameter)
                            {
                                p_ParamArrayElement = (ICapeParameter)values[j];
                                retVal = String.Concat(retVal, String.Concat("          ", p_ParamArrayElement.value.ToString(), Environment.NewLine));
                            }
                            else
                            {
                                retVal = String.Concat(retVal, String.Concat("          ", values[j].ToString(), Environment.NewLine));
                            }
                        }
                    }
                }
                if (p_Spec.Type == CapeParamType.CAPE_REAL)
                {
                    ICapeRealParameterSpecCOM p_Real = (ICapeRealParameterSpecCOM)p_Spec;
                    RealParameter p_RealParam;
                    if (p_Param is RealParameter)
                    {
                        p_RealParam = (RealParameter)p_Param;
                        retVal = String.Concat(retVal, "     Value: ", p_RealParam.DimensionedValue.ToString(), Environment.NewLine);
                        retVal = String.Concat(retVal, "     Dimensionality: ", p_RealParam.Unit, Environment.NewLine);
                    }
                    else
                    {
                        retVal = String.Concat(retVal, "     Value: ", p_Param.value.ToString(), Environment.NewLine);
                        retVal = String.Concat(retVal, "     Dimensionality: ", p_Spec.Dimensionality.ToString(), Environment.NewLine);
                    }
                    retVal = String.Concat(retVal, "     Default Value: ", p_Real.DefaultValue, Environment.NewLine);
                    retVal = String.Concat(retVal, "     Lower Bound: ", p_Real.LowerBound, Environment.NewLine);
                    retVal = String.Concat(retVal, "     Upper Bound: ", p_Real.UpperBound, Environment.NewLine);
                }
                if (p_Spec.Type == CapeParamType.CAPE_INT)
                {
                    retVal = String.Concat(retVal, "     Value: ", p_Param.value.ToString(), Environment.NewLine);
                    CapeOpen.ICapeIntegerParameterSpec p_IntParam = (ICapeIntegerParameterSpec)p_Spec;
                    retVal = String.Concat(retVal, "     Default Value: ", p_IntParam.DefaultValue, Environment.NewLine);
                    retVal = String.Concat(retVal, "     Lower Bound: ", p_IntParam.LowerBound, Environment.NewLine);
                    retVal = String.Concat(retVal, "     Upper Bound: ", p_IntParam.UpperBound, Environment.NewLine);
                }
                if (p_Spec.Type == CapeParamType.CAPE_BOOLEAN)
                {
                    retVal = String.Concat(retVal, "     Value: ", p_Param.value.ToString(), Environment.NewLine);
                    CapeOpen.ICapeBooleanParameterSpec p_Bool = (ICapeBooleanParameterSpec)p_Spec;
                    retVal = String.Concat(retVal, "     Default Value: ", p_Bool.DefaultValue, Environment.NewLine);
                }
                if (p_Spec.Type == CapeParamType.CAPE_OPTION)
                {
                    retVal = String.Concat(retVal, "     Value: ", p_Param.value.ToString(), Environment.NewLine);
                    CapeOpen.ICapeOptionParameterSpec p_Opt = (ICapeOptionParameterSpec)p_Spec;
                    retVal = String.Concat(retVal, "     Default Value: ", p_Opt.DefaultValue, Environment.NewLine);
                    if (p_Opt.RestrictedToList)
                    {
                        retVal = String.Concat(retVal, "     Restricted to List: TRUE", Environment.NewLine);
                    }
                    else
                    {
                        retVal = String.Concat(retVal, "     Restricted to List: FALSE", Environment.NewLine);
                    }
                    retVal = String.Concat(retVal, "     Option List Values: ", Environment.NewLine);
                    String[] options = (String[])p_Opt.OptionList;
                    for (int j = 0; j < options.Length; j++)
                    {
                        retVal = String.Concat(retVal, "          Option[", j, "]: ", options[j], Environment.NewLine);
                    }
                }

                retVal = String.Concat(retVal, Environment.NewLine);
            }
            retVal = String.Concat(retVal, Environment.NewLine, "Ports: ", Environment.NewLine);
            for (int i = 0; i < this.Ports.Count; i++)
            {
                ICapeUnitPort p_Port = (ICapeUnitPort)m_Ports[i];
                CapeIdentification p_PortId = (CapeIdentification)p_Port;
                retVal = String.Concat(retVal, "     Port ", i.ToString(), ": ", p_PortId.ComponentName, Environment.NewLine);
                retVal = String.Concat(retVal, "     Desription:  ", p_PortId.ComponentDescription, Environment.NewLine);
                ICapeIdentification p_PortConnectedObjectId = (ICapeIdentification)p_Port.connectedObject;
                retVal = String.Concat(retVal, "     Port Type: ", p_Port.portType.ToString(), Environment.NewLine);
                retVal = String.Concat(retVal, "     Port Direction: ", p_Port.direction.ToString(), Environment.NewLine);
                if (p_PortConnectedObjectId != null)
                {
                    retVal = String.Concat(retVal, "     Connected Object:  ", p_PortConnectedObjectId.ComponentName, Environment.NewLine);
                }
                else
                {
                    retVal = String.Concat(retVal, "     No Connected Object", Environment.NewLine);
                }
                retVal = String.Concat(retVal, Environment.NewLine);
            }
            return retVal;
        }
    };
}
