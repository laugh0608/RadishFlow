using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{
    /// <summary>
    /// Interface that provides access to unit operation packages available on the computer.
    /// </summary>
    /// <remarks>
    /// <para>This interface is used to access the various substiuent unit operation models 
    /// available within the current system.</para>
    /// <para>In the class library, the <see cref ="UnitOperationSystem">UnitOperationSystem</see> class provides a list of all
    /// unit operation classes registered with COM and all .Net-based property unit operations that are contained in the Global Assembly Cache.</para>
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.GuidAttribute("08D7828D-40FD-4CA1-A71D-2F25617DA133")]
    [System.ComponentModel.DescriptionAttribute("IUnitOperationSystem Interface")]
    public interface IUnitOperationSystem
    {
        /// <summary>
        /// Get the list of available unit operation PMCs
        /// </summary>
        /// <remarks>
        /// Returns StringArray of unit operation PMC names available in the GAC and COM.
        /// </remarks>
        /// <returns>
        /// The returned set of available unit operation PMCs.
        /// A System.Object containing a String array marshalled from a COM Object.
        /// </returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("method GetUnitOperations")]
        Object GetUnitOperations();

        /// <summary>
        /// Get the list of all available unit operation PMCs
        /// </summary>
        /// <remarks>
        /// Returns StringArray of unit operation PMC names registered with COM using the CAPE-OPEN unit operation Category (CATID).
        /// </remarks>
        /// <returns>
        /// The returned set of available unit operation PMCs.
        /// A System.Object containing a String array marshalled from a COM Object.
        /// </returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("method GetUnitOperations")]
        Object GetCOMUnitOperations();

        /// <summary>
        /// Get the list of all .Net CAPE-OPEN unit operation PMCs
        /// </summary>
        /// <remarks>
        /// Returns StringArray of unit operation PMC names available in the GAC and in the program files "Common Files\Cape-Open" directory.
        /// </remarks>
        /// <returns>
        /// The returned set of available unit operation PMCs.
        /// A System.Object containing a String array marshalled from a COM Object.
        /// </returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("method GetUnitOperations")]
        Object GetDotNetUnitOperations();

        /// <summary>
        /// Resolve a particular unit operation
        /// </summary>
        /// <remarks>
        /// Resolves referenced unit operation PMC to a unit operation interface.
        /// </remarks>
        /// <returns>
        /// The Unit Operation via the ICapeUnit Interface.
        /// </returns>
        /// <param name = "unitOperation">
        /// The unit operation to be resolved.
        /// </param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("method ResolveUnitOperation")]
        [return: System.Runtime.InteropServices.MarshalAsAttribute(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
        Object ResolveUnitOperation(System.String unitOperation);
    };

    struct UnitOperationInformation
    {
        public String Name;
        public String Description;
        public String CapeVersion;
        public String ComponentVersion;
        public String VendorURL;
        public String HelpURL;
        public String About;
        public System.Type Type;
        public string ProgID;
        public System.Guid CLSID;
        public string Assembly;
    }


    /// <summary>
    /// A class that implements the <see cref ="ICapeThermoSystem">ICapeThermoSystem</see> interface and provides access 
    /// to COM and .Net-based property packages available on the current computer.
    /// </summary>
    /// <remarks>
    /// <para>This class provides a list of all
    /// classes Property Packages registered with COM and all .Net-based property packages that are contained in the Global Assembly Cache.</para>
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("3A223DEE-8414-4802-8391-D1B11B276A0F")]
    [System.ComponentModel.DescriptionAttribute("CapeThermoSystem Wrapper")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class UnitOperationSystem : IUnitOperationSystem
    {

        System.Collections.Generic.List<UnitOperationInformation> m_UnitOperations = new System.Collections.Generic.List<UnitOperationInformation>();
        System.Collections.Generic.List<UnitOperationInformation> m_ComBasedUnitOperations = new System.Collections.Generic.List<UnitOperationInformation>();
        System.Collections.Generic.List<UnitOperationInformation> m_DotNetUnitOperations = new System.Collections.Generic.List<UnitOperationInformation>();

        /// <summary>
        /// A class that implements the <see cref ="IUnitOperationSystem">IUnitOperationSystem</see> interface and provides access 
        /// to COM and .Net-based unit operation models available on the current computer.
        /// </summary>
        /// <remarks>
        /// <para>This class provides a list of all
        /// cunit operation classes registered with COM and all .Net-based property packages 
        /// that are contained in the Global Assembly Cache.</para>
        /// </remarks>
        public UnitOperationSystem()
        {
            AppDomain domain = AppDomain.CurrentDomain;
            domain.AssemblyResolve += new ResolveEventHandler(MyResolveEventHandler);
            this.InsertAvailableCOM();
            this.InsertAvailableNET();
        }


        /// <summary>
        /// Get the list of all available unit operation PMCs
        /// </summary>
        /// <remarks>
        /// Returns StringArray of unit operation PMC names registered with COM using the CAPE-OPEN unit operation Category (CATID).
        /// </remarks>
        /// <returns>
        /// The returned set of available unit operation PMCs.
        /// A System.Object containing a String array marshalled from a COM Object.
        /// </returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        Object IUnitOperationSystem.GetCOMUnitOperations()
        {
            string[] retVal = new string[m_ComBasedUnitOperations.Count];
            for (int i = 0; i < m_ComBasedUnitOperations.Count; i++)
            {
                retVal[i] = m_ComBasedUnitOperations[i].ProgID;
            }
            return retVal;
        }

        /// <summary>
        /// Get the list of all .Net CAPE-OPEN unit operation PMCs
        /// </summary>
        /// <remarks>
        /// Returns StringArray of unit operation PMC names available in the GAC and in the program files "Common Files\Cape-Open" directory.
        /// </remarks>
        /// <returns>
        /// The returned set of available unit operation PMCs.
        /// A System.Object containing a String array marshalled from a COM Object.
        /// </returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        Object IUnitOperationSystem.GetDotNetUnitOperations()
        {
            string[] retVal = new string[m_DotNetUnitOperations.Count];
            for (int i = 0; i < m_DotNetUnitOperations.Count; i++)
            {
                retVal[i] = m_DotNetUnitOperations[i].ProgID;
            }
            return retVal;
        }
        
        /// <summary>
        /// Get the list of available unit operation PMCs
        /// </summary>
        /// <remarks>
        /// Returns StringArray of unit operation PMC names available in the GAC and COM.
        /// </remarks>
        /// <returns>
        /// The returned set of available unit operation PMCs.
        /// A System.Object containing a String array marshalled from a COM Object.
        /// </returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        Object IUnitOperationSystem.GetUnitOperations()
        {
            return this.GetUnitOperations();
        }

        /// <summary>
        /// Resolve a particular unit operation
        /// </summary>
        /// <remarks>
        /// Resolves referenced unit operation PMC to a unit operation interface.
        /// </remarks>
        /// <returns>
        /// The Unit Operation via the ICapeUnit Interface.
        /// </returns>
        /// <param name = "unitOperation">
        /// The unit operation to be resolved.
        /// </param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        Object IUnitOperationSystem.ResolveUnitOperation(System.String unitOperation)
        {
            return this.ResolveUnitOperation(unitOperation);
        }

        /// <summary>
        /// Get the list of available unit operation PMCs
        /// </summary>
        /// <remarks>
        /// Returns StringArray of unit operation PMC names available in the GAC and COM.
        /// </remarks>
        /// <returns>
        /// The returned set of available unit operation PMCs.
        /// A System.Object containing a String array marshalled from a COM Object.
        /// </returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        public string[] GetUnitOperations()
        {
            string[] retVal = new string[m_UnitOperations.Count];
            for (int i = 0; i < m_UnitOperations.Count; i++)
            {
                retVal[i] = m_UnitOperations[i].ProgID;
            }
            return retVal;
        }

        /// <summary>
        /// Get the list of all available unit operation PMCs
        /// </summary>
        /// <remarks>
        /// Returns StringArray of unit operation PMC names registered with COM using the CAPE-OPEN unit operation Category (CATID).
        /// </remarks>
        /// <returns>
        /// The returned set of available unit operation PMCs.
        /// A System.Object containing a String array marshalled from a COM Object.
        /// </returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        public string[] GetCOMUnitOperations()
        {
            string[] retVal = new string[m_ComBasedUnitOperations.Count];
            for (int i = 0; i < m_ComBasedUnitOperations.Count; i++)
            {
                retVal[i] = m_ComBasedUnitOperations[i].ProgID;
            }
            return retVal;
        }

        /// <summary>
        /// Get the list of all .Net CAPE-OPEN unit operation PMCs
        /// </summary>
        /// <remarks>
        /// Returns StringArray of unit operation PMC names available in the GAC and in the program files "Common Files\Cape-Open" directory.
        /// </remarks>
        /// <returns>
        /// The returned set of available unit operation PMCs.
        /// A System.Object containing a String array marshalled from a COM Object.
        /// </returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        public string[] GetDotNetUnitOperations()
        {
            string[] retVal = new string[m_DotNetUnitOperations.Count];
            for (int i = 0; i < m_DotNetUnitOperations.Count; i++)
            {
                retVal[i] = m_DotNetUnitOperations[i].ProgID;
            }
            return retVal;
        }

        /// <summary>
        /// Resolve a particular unit operation
        /// </summary>
        /// <remarks>
        /// Resolves referenced unit operation PMC to a unit operation interface.
        /// </remarks>
        /// <returns>
        /// The Unit Operation via the ICapeUnit Interface.
        /// </returns>
        /// <param name = "unitOperation">
        /// The unit operation to be resolved.
        /// </param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        public object ResolveUnitOperation(System.String unitOperation)
        {
            for (int i = 0; i < m_UnitOperations.Count; i++)
            {
                System.Type type = m_UnitOperations[i].Type;
                if (m_UnitOperations[i].ProgID == unitOperation)
                {
                    System.Type unitType = m_UnitOperations[i].Type;
                    if (typeof(CapeUnitBase).IsAssignableFrom(unitType))
                    {
                        return AppDomain.CurrentDomain.CreateInstanceAndUnwrap(m_UnitOperations[i].Type.Assembly.FullName, m_UnitOperations[i].Type.FullName);
                    }
                    if (m_UnitOperations[i].Type.IsCOMObject)
                    {
                        return new UnitOperationWrapper(m_UnitOperations[i].CLSID);
                    }
                    return new UnitOperationWrapper(AppDomain.CurrentDomain.CreateInstanceAndUnwrap(m_UnitOperations[i].Type.Assembly.FullName, m_UnitOperations[i].Type.FullName));
                }
            }
            return null;
        }

        /// <summary>
        /// Used to resolve assemblies available to the <see cref = "UnitOperationSystem"/>
        /// </summary>
        /// <remarks>
        /// <para>This method resolves the assembly locations for types being created. The priority for locating 
        /// assemblies is the current applcation domain, then the Cape-Open directory located in the Common Program
        /// Files environment directory (typically C:\Program Files\Common Files\Cape-Open).</para>
        /// <para></para>
        /// </remarks>
        /// <param name="sender">The source of the event.</param>
        /// <param name="args">Information about the item to be resolved</param>
        /// <returns>The assembly that resolves the type, assembly, or resource; or null if the assembly cannot be resolved.</returns>
        static public System.Reflection.Assembly MyResolveEventHandler(Object sender, System.ResolveEventArgs args)
        {
            System.Reflection.Assembly[] assemblies = AppDomain.CurrentDomain.GetAssemblies();
            foreach (System.Reflection.Assembly assembly in assemblies)
            {
                if (args.Name == assembly.FullName)
                {
                    return assembly;
                }
            }
            string path = System.Environment.GetFolderPath(Environment.SpecialFolder.CommonProgramFiles) + "\\Cape-Open";
            return SearchPathForAssemblies(path, args.Name);
        }

        /// <summary>
        /// Searches a path for the desired assembly.
        /// </summary>
        /// <remarks>When this method is called, the path is scanned for the desired assembly.</remarks>
        /// <param name="path">Path to be search. The search includes all subdirectories.</param>
        /// <param name="assemblyName">The name of the deisred assembly.</param>
        /// <returns>The desired Assembly.</returns>
        static public System.Reflection.Assembly SearchPathForAssemblies(string path, string assemblyName)
        {
            string[] directories = System.IO.Directory.GetDirectories(path);
            foreach (string directory in directories)
            {
                return SearchPathForAssemblies(directory, assemblyName);
            }
            string[] files = System.IO.Directory.GetFiles(path, "dll");
            foreach (string file in files)
            {
                System.Reflection.Assembly assembly = System.Reflection.Assembly.LoadFrom(file);
                if (assembly.FullName == assemblyName)
                {
                    return assembly;
                }
            }
            return null;
        }

        void InsertAvailableCOM()
        {
            //initialize namespace Controls Category namespace Controlsds
            // CAPE-OPEN Category - namespace ControlsComponent_CATID
            String rgcid5;
            //rgcid1 = "";//{678c09a1-7d66-11d2-a67d-00105a42887f}";
            // External CAPE-OPEN Thermo Routines - CapeExternalThermoRoutine_CATID
            //rgcid2 = "";//{678c09a2-7d66-11d2-a67d-00105a42887f}";
            // CAPE-OPEN Thermo System - CapeThermoSystem_CATID
            //rgcid3 = "";//{678c09a3-7d66-11d2-a67d-00105a42887f}";
            // CAPE-OPEN Thermo Property Package - CapeThermoPropertyPackage_CATID
            //rgcid4 = "";//{678c09a4-7d66-11d2-a67d-00105a42887f}";
            // CAPE-OPEN Unit Operation - CapeUnitOperation_CATID
            rgcid5 = "{678c09a5-7d66-11d2-a67d-00105a42887f}";
            // CAPE-OPEN Thermo Equilibrium Server - CapeThermoEquilibriumServer_CATID
            //rgcid6 = "";//{678c09a6-7d66-11d2-a67d-00105a42887f}";

            Microsoft.Win32.RegistryKey key = Microsoft.Win32.Registry.ClassesRoot.OpenSubKey("CLSID");
            // get the classes that implement the various Cape Open categories
            // CapeUnit category
            String[] sknames = key.GetSubKeyNames();
            key.Close();
            for (int n = 0; n < sknames.Length; n++)
            {
                key = Microsoft.Win32.Registry.ClassesRoot.OpenSubKey(String.Concat("CLSID\\", sknames[n]));
                String[] subnames = key.GetSubKeyNames();
                bool found = false;
                String name = null;//, image = null;
                Type type = null;
                System.Guid CLSID = System.Guid.Empty;
                //System.Drawing.Bitmap bitty = null;
                for (int i = 0; i < subnames.Length; i++)
                {
                    if (subnames[i] == "Implemented Categories")
                    {
                        Microsoft.Win32.RegistryKey subkey = Microsoft.Win32.Registry.ClassesRoot.OpenSubKey("CLSID\\" + sknames[n] + "\\Implemented Categories");
                        String[] vals = subkey.GetSubKeyNames();
                        for (int z = 0; z < vals.Length; z++)
                        {
                            if (String.Compare(vals[z], rgcid5, true) == 0)
                                found = true;
                        }
                        subkey.Close();
                    }
                    else if (String.Compare(subnames[i], "progid", true) == 0 && found == true)
                    {
                        Microsoft.Win32.RegistryKey subkey = Microsoft.Win32.Registry.ClassesRoot.OpenSubKey(String.Concat("CLSID\\", sknames[n], "\\ProgID"));
                        String[] vals = subkey.GetValueNames();
                        if (vals.Length >= 1)
                        {
                            name = (String)(subkey.GetValue(vals[0], typeof(String)));
                            CLSID = new System.Guid(sknames[n]);
                            type = Type.GetTypeFromCLSID(CLSID);
                        }
                        subkey.Close();
                        Microsoft.Win32.RegistryKey descriptionKey = Microsoft.Win32.Registry.ClassesRoot.OpenSubKey(String.Concat("CLSID\\", sknames[n], "\\CapeDescription"));
                        if (descriptionKey != null && !typeof(CapeUnitBase).IsAssignableFrom(type) && CLSID != new System.Guid("48c6795c-a67d-4807-8881-c4c9e02418d0"))
                        {
                            UnitOperationInformation unit = new UnitOperationInformation();
                            String[] descriptions = descriptionKey.GetValueNames();
                            unit.Name = descriptionKey.GetValue("Name", String.Empty).ToString();
                            unit.Description = descriptionKey.GetValue("Description", String.Empty).ToString();
                            unit.CapeVersion = descriptionKey.GetValue("CapeVersion", String.Empty).ToString();
                            unit.ComponentVersion = descriptionKey.GetValue("ComponentVersion", String.Empty).ToString();
                            unit.VendorURL = descriptionKey.GetValue("VendorURL", String.Empty).ToString();
                            unit.HelpURL = descriptionKey.GetValue("HelpURL", String.Empty).ToString();
                            unit.About = descriptionKey.GetValue("About", String.Empty).ToString();
                            unit.Type = type;
                            unit.ProgID = name;
                            unit.CLSID = CLSID;
                            unit.Assembly = "";
                            m_UnitOperations.Add(unit);
                            m_ComBasedUnitOperations.Add(unit);
                            descriptionKey.Close();
                        }
                        else
                        {
                            //infos.Add(null);
                        }

                    }
                    //else if (found == true) //&& ((subnames[i].CompareTo("ToolboxBitmap32") == 0) ||	subnames[i].CompareTo("DefaultIcon")))
                    //{
                    //    Microsoft.Win32.RegistryKey subkey = Microsoft.Win32.Registry.ClassesRoot.OpenSubKey(String.Concat("CLSID\\", sknames[n], "\\ToolboxBitmap32"));
                    //    string[] vals = subkey.GetValueNames();
                    //    if (vals.Length >= 1)
                    //    {
                    //        image = (String)(subkey.GetValue(vals[0], typeof(String)));
                    //        int pos = image.IndexOf(",");
                    //        if (pos == -1)
                    //        {
                    //            System.Drawing.Icon ic = null;// System::Drawing::Icon::FromHandle(ExtractIcon(System::Diagnostics::Process::GetCurrentProcess()->Handle,image,0));
                    //            bitty = ic.ToBitmap();
                    //        }
                    //        else
                    //        {
                    //            /*						first = image->Substring(0,pos);
                    //            second = image->Substring(pos+1);
                    //            IntPtr ptr = System::Diagnostics::Process::GetCurrentProcess()->Handle;
                    //            UInt32 hic = ExtractIcon(ptr,first,Convert::ToUInt32(second));
                    //            if(hic ==0)
                    //            bitty = gcnew Bitmap(imageList2->Images[0]);
                    //            else
                    //            {
                    //            System::Drawing::Icon^ ic = System::Drawing::Icon::FromHandle(IntPtr(hic));
                    //            bitty = ic->ToBitmap();
                    //            }
                    //            */
                    //        }
                    //    }
                    //    subkey.Close();
                    //}
                }
                if (found == true && name != null)
                {
                    //if(!listBox1.Items.Contains(new ListViewItem(name)))
                    //{
                    //    listBox1.Items.Add(name);
                    //    if(bitty == nullptr)
                    //        bitty = gcnew Bitmap(imageList2.Images[0]);
                    //    imageList1.Images.Add(bitty);
                    //    ToolStripButton button = new ToolStripButton(name, bitty);
                    //    this.toolStrip1.Items.Add(button);
                    //    button.Visible = true;
                    //    button.Tag = type;
                    //    button.DisplayStyle = ToolStripItemDisplayStyle.Image;
                    //}
                }
                key.Close();
            }
        }

        void InsertAvailableNET()
        {
            System.Reflection.Assembly[] assemblies = AppDomain.CurrentDomain.GetAssemblies();
            foreach (System.Reflection.Assembly assembly in assemblies)
            {
                this.InsertUnitsFromAssembly(assembly);
            }
            string path = System.Environment.GetFolderPath(Environment.SpecialFolder.CommonProgramFiles);
            path = path + "\\Cape-Open";
            this.InsertsUnitsFromPath(path);
        }

        void InsertUnitsFromAssembly(System.Reflection.Assembly assembly)
        {
            Type[] types = assembly.GetExportedTypes();
            foreach (Type type in types)
            {
                if (typeof(CapeOpen.ICapeUnit).IsAssignableFrom(type) && !type.IsAbstract
                    && type != typeof(UnitOperationWrapper))
                {
                    UnitOperationInformation unit = new UnitOperationInformation();
                    object[] attributes = type.GetCustomAttributes(true);
                    for (int i = 0; i < attributes.Length; i++)
                    {
                        if (attributes[i] is CapeNameAttribute) unit.Name = ((CapeNameAttribute)attributes[i]).Name;
                        if (attributes[i] is CapeDescriptionAttribute) unit.Description = ((CapeDescriptionAttribute)attributes[i]).Description;
                        if (attributes[i] is CapeVersionAttribute) unit.CapeVersion = ((CapeVersionAttribute)attributes[i]).Version;
                        if (attributes[i] is CapeVendorURLAttribute) unit.VendorURL = ((CapeVendorURLAttribute)attributes[i]).VendorURL;
                        if (attributes[i] is CapeHelpURLAttribute) unit.HelpURL = ((CapeHelpURLAttribute)attributes[i]).HelpURL;
                        if (attributes[i] is CapeAboutAttribute) unit.About = ((CapeAboutAttribute)attributes[i]).About;
                        unit.ProgID = type.FullName;
                        unit.Assembly = assembly.FullName;
                    }
                    unit.Type = type;
                    m_UnitOperations.Add(unit);
                    m_DotNetUnitOperations.Add(unit);
                }
            }
        }

        void InsertUnitsFromSpecialFolder(System.Environment.SpecialFolder folder)
        {
            string path = System.Environment.GetFolderPath(folder);
            this.InsertsUnitsFromPath(path);
        }

        void InsertsUnitsFromPath(string path)
        {
            string[] directories = System.IO.Directory.GetDirectories(path);
            foreach (string directory in directories)
            {
                this.InsertsUnitsFromPath(directory);
            }
            string[] files = System.IO.Directory.GetFiles(path, "dll");
            foreach (string file in files)
            {
                System.Reflection.Assembly assembly = System.Reflection.Assembly.LoadFrom(file);
                this.InsertUnitsFromAssembly(assembly);
            }
        }
    }
}

