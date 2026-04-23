/*Copyright 2016 Caster

* Licensed under the Apache License, Version 2.0 (the "License");
* you may not use this file except in compliance with the License.
* You may obtain a copy of the License at
* 
*     http://www.apache.org/licenses/LICENSE-2.0
* 
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific language governing permissions and
* limitations under the License.*/

using System;
using System.Linq;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;
using System.Runtime.Serialization.Formatters.Binary;
using System.Threading;
using System.Windows;
using Microsoft.Win32;
using System.Collections.Generic;
using System.Threading.Tasks;
using CasterUnitCore.Reports;
using CasterCore;
using CAPEOPEN;
using System.Runtime.CompilerServices;

namespace CasterUnitCore
{
    /// <summary>
    /// Base class of unit operation, must be inherit
    /// Parameter initialize and port initialize and calculation should be in this class
    /// Genericity is not allowed, because of IPersist
    /// </summary>
    [Serializable]
    [ComVisible(true)]
    [ComDefaultInterface(typeof(ICapeUnit))]
    [Guid("E78BFCC3-4865-4AF5-8BEE-276C49DE506F")]
    public abstract class CasterUnitOperationBase :
        CapeOpenBaseObject,
        ICapeUnit, ICapeUtilities,
        ICapeUnitReport, CasterCore.IPersistStream  //The origin CAPE-OPEN IPersistStream has a problem, so overload it
    {
        #region fields

        /// <summary>
        /// is unit valid
        /// </summary>
        protected CapeValidationStatus Status = CapeValidationStatus.CAPE_NOT_VALIDATED;
        /// <summary>
        /// reference to simulationContext
        /// </summary>
        protected ICapeSimulationContext _simulationContext;
        /// <summary>
        /// whether the unit is loaded from file
        /// </summary>
        protected bool Isloaded;
        /// <summary>
        /// new thread to show GUI, important for aspen plus
        /// </summary>
        [NonSerialized]
        protected Thread GuiThread;
        /// <summary>
        /// calculate instance, contains derived instance of SpecCalculator
        /// </summary>
        public Calculator SpecCalculator { get; set; }
        /// <summary>
        /// Collection of ports connected to this unit
        /// </summary>
        public readonly CapeCollection Ports;
        /// <summary>
        /// Collection of Parameters, should be CAPE_INPUT or CAPE_INPUT_OUTPUT
        /// </summary>
        public readonly CapeCollection Parameters;
        /// <summary>
        /// Collection of results, should be CAPE_OUTPUT Parameters, used to display results after calculation
        /// </summary>
        public readonly CapeCollection Results;
        /// <summary>
        /// Unique id of an instance, to identify the unit in CasterUnitLocator, default using Guid
        /// </summary>
        public readonly string UnitId;
        /// <summary>
        /// Contains reports available to this unit operation
        /// </summary>
        public readonly List<ReportBase> Reports;

        [NonSerialized]
        private ReportBase _selectedReport;

        #endregion

        #region Events

        /// <summary>
        /// Called during Initialize
        /// </summary>
        public event EventHandler OnInitialize;
        /// <summary>
        /// Called during Terminate
        /// </summary>
        public event EventHandler OnTerminate;
        /// <summary>
        /// Called when COSE ask for Parameters interface
        /// </summary>
        public event EventHandler<CapeCollection> OnGetParameters;
        /// <summary>
        /// Called when COSE ask for ports interface
        /// </summary>
        public event EventHandler<CapeCollection> OnGetPorts;

        #endregion

        #region Constructor
        /// <summary>
        /// This constructor assign SpecCalculator, create Ports, Parameters and Results and call their initialize method
        /// </summary>
        /// <paramCollection name="specCalculator">the specification calculate class</paramCollection>
        /// <paramCollection name="className">name of this unit operation</paramCollection>
        /// <paramCollection name="description">description of this unit operation</paramCollection>
        public CasterUnitOperationBase(Calculator specCalculator, string className, string description)
            : base(className, description, true, true)
        {
            CasterLogger.Debug($"UnitOperation {className} Initializing.");
            Debug.WriteLine($"UnitOperation {className} Initializing.");

            this.SpecCalculator = specCalculator;
            SpecCalculator.UnitOp = this;
            Isloaded = false;
            UnitId = Guid.NewGuid().ToString("B");

            Ports = new CapeCollection("Ports",
                "In and Out ports of this unit.",
                (item) => item is ICapeUnitPort);

            Parameters = new CapeCollection("Parameters",
                "User input Parameters of this unit.",
                (item) =>
                    item is ICapeParameter &&
                    (item as ICapeParameter).Mode != CapeParamMode.CAPE_OUTPUT);

            Results = new CapeCollection("Results",
                "Calculate Results, output Parameters.",
                (item) =>
                    item is ICapeParameter &&
                    (item as ICapeParameter).Mode != CapeParamMode.CAPE_INPUT);

            Reports = new List<ReportBase>();

            CasterLogger.Debug("UnitOperation Initialize Completed.");
            Debug.WriteLine("UnitOperation Initialize Completed.");
        }

        #endregion

        #region Get Parameters, Ports and Reports

        /// <summary>
        /// Add parameter to Parameters.
        /// if you use some variable point to the variable in Parameters, they might change before BeforeCalculate
        /// </summary>
        public abstract void InitParameters();

        /// <summary>
        /// Add port to Ports
        /// </summary>
        public abstract void InitPorts();

        /// <summary>
        /// Add result parameter to Results
        /// </summary>
        public abstract void InitResults();

        /// <summary>
        /// Add available reports
        /// </summary>
        public virtual void InitReports()
        {
            CasterLogger.Debug("Init reports");
            Reports.Add(new StatusReport());
            Reports.Add(new LastRunReport());
            CasterLogger.Debug("Finish init reports");
        }

        /// <summary> 
        /// Default action is to check whether all Parameters is valid, override to customize
        /// </summary>
        protected virtual bool ParamtersValidate(out string message)
        {
            CasterLogger.Debug("ParamtersValidate");
            bool valid = true;
            message = "";
            foreach (var parameter in Parameters)
            {
                string temp = "";
                if (!((CapeParameterBase)parameter.Value).Validate(ref temp))
                {
                    message += "\n" + temp;
                    valid = false;
                }
            }
            if (valid)
            {
                CasterLogger.Debug("Parameter valid.");
                return true;
            }
            else
            {
                CasterLogger.Debug("Parameter invalid. Msg: " + message);
                return false;
            }
        }

        /// <summary>
        /// Default action is to check if an inlet and a outlet material is connected, override to customize
        /// </summary>
        protected virtual bool PortsValidate(out string message)
        {
            CasterLogger.Debug("PortsValidate");
            bool hasInlet = false;
            bool hasOutlet = false;
            message = "";
            foreach (var port in Ports)
            {
                CapeMaterialPort p = port.Value as CapeMaterialPort;
                if (p == null) continue;
                if (p.direction == CapePortDirection.CAPE_INLET && p.IsConnected())
                    hasInlet = true;
                else if (p.direction == CapePortDirection.CAPE_OUTLET && p.IsConnected())
                    hasOutlet = true;
            }
            if (!hasInlet)
            {
                CasterLogger.Debug("Inlet material is not connected.");
                message += "Inlet material is not connected.";
            }
            if (!hasOutlet)
            {
                CasterLogger.Debug("Inlet material is not connected.");
                message += "Outlet material is not connected.";
            }
            CasterLogger.Debug("PortsValidate result: " + (hasInlet && hasOutlet));
            return hasInlet && hasOutlet;
        }

        /// <summary>
        /// Called after the compound list is changed
        /// </summary>
        protected void UpdateCompoundList()
        {
            CasterLogger.Debug("UpdateCompoundList");
            foreach (var item in Ports)
            {
                CapeMaterialPort p = item.Value as CapeMaterialPort;
                if (p != null && p.Material != null)
                    p.Material.UpdateCompoundList();
            }
        }

        #endregion

        #region ICapeUnit

        /// <summary>
        /// raw Calculate interface called by simulator, NO need to modify this
        /// </summary>
        public virtual void Calculate()
        {
            CasterLogger.Debug("Calculate");

            UpdateCompoundList();
            CasterLogger.Debug("BeforeCalculate");
            SpecCalculator.BeforeCalculate();
            CapeDiagnostic.LogMessage("{0} : Calculate Start.", ComponentName);
            CasterLogger.Debug("Calculate");
            SpecCalculator.Calculate();
            CapeDiagnostic.LogMessage("{0} : Calculate Complete.", ComponentName);
            CasterLogger.Debug("OutputResult");
            SpecCalculator.OutputResult();

            CasterLogger.Debug("Calculate Completed");
        }

        /// <summary>
        /// This method will call ParameterValidate and PortValidate, in most case, no need to override this, just override ParameterValidate and PortValidate
        /// </summary>
        /// <paramCollection name="message">return the combined error message</paramCollection>
        public virtual bool Validate(ref string message)
        {
            CasterLogger.Debug("Validate");

            string parameterMessage = null;
            string portMessage = null;

            Status = CapeValidationStatus.CAPE_VALID;

            if (!ParamtersValidate(out parameterMessage))
                Status = CapeValidationStatus.CAPE_INVALID;

            if (!PortsValidate(out portMessage))
                Status = CapeValidationStatus.CAPE_INVALID;

            message = "" + parameterMessage + '\n' + portMessage;
            if (Status == CapeValidationStatus.CAPE_VALID)
            {
                CasterLogger.Debug("Validate success.");
                return true;
            }
            else
            {
                CasterLogger.DebugFormatted("Validate fails. {0}", message);
                Debug.WriteLine("Validate fails. {0}", message);
                return false;
            }
        }
        /// <summary>
        /// raw ports interface called by simulator
        /// </summary>
        object ICapeUnit.ports
        {
            get
            {
                OnGetPorts?.Invoke(this, Ports);
                return Ports;
            }
        }
        /// <summary>
        /// raw ValStatus interface called by simulator
        /// </summary>
        public CapeValidationStatus ValStatus
        {
            get { return Status; }
        }

        #endregion

        #region ICapeUtilities

        /// <summary>
        /// raw Initialize interface called by simulator, do nothing but write a debug message
        /// </summary>
        public virtual void Initialize()
        {
            CasterLogger.Debug("Initialize");
            Debug.WriteLine("Initialize");

            //Is bad to use virtual method here, but considering that I have create three instances above, so it works for now.
            try
            {
                CasterLogger.Debug("Start Initialize Parameters.");
                Debug.WriteLine("Start Initialize Parameters.");
                InitParameters();

                CasterLogger.Debug("Start Initialize Ports.");
                Debug.WriteLine("Start Initialize Ports.");
                InitPorts();

                CasterLogger.Debug("Start Initialize Results.");
                Debug.WriteLine("Start Initialize Results.");
                InitResults();

                CasterLogger.Debug("Start Initialize Reports.");
                Debug.WriteLine("Start Initialize Reports.");
                InitReports();

                CasterLogger.Debug("Start SetUnitOp");
                foreach (var report in Reports)
                {
                    report.SetUnitOp(this);
                }
            }
            catch (Exception e)
            {
                CasterLogger.Debug("Initialize Failed: " + e.Message);
                Debug.WriteLine("Initialize Failed: " + e.Message);
                throw new ECapeUnknownException(this,
                    "UnitOperation Initialize Failed." + e.Message,
                    e);
            }

            CasterLogger.Debug("OnInitialize");
            OnInitialize?.Invoke(this, EventArgs.Empty);

            Debug.WriteLine("Initialize Completed.");
        }

        /// <summary>
        /// raw Terminate interface called by simulator, release COM object write a debug message
        /// </summary>
        public virtual void Terminate()
        {
            CasterLogger.Debug("Terminate");
            Debug.WriteLine("Terminate");
            OnTerminate?.Invoke(this, EventArgs.Empty);

            CasterLogger.Debug("Start Release COM object");
            if (simulationContext != null && simulationContext.GetType().IsCOMObject)
                Marshal.FinalReleaseComObject(simulationContext);

            foreach (var port in Ports)
            {
                ((CapeUnitPortBase)port.Value).Disconnect();
                ((CapeUnitPortBase)port.Value).Dispose();
            }

            CasterLogger.Debug("Terminate Done.");
            Debug.WriteLine("Terminate Done.");
        }

        /// <summary>
        /// raw Edit interface called by simulator, create a new thread to display GUI
        /// </summary>
        public int Edit()
        {
            CasterLogger.Debug("Edit.");
            Debug.WriteLine("Edit");
            if (GuiThread != null && GuiThread.IsAlive)
                return 0;     //if not abort, return 0
            GuiThread = new Thread(OpenEditWindow);
            GuiThread.SetApartmentState(ApartmentState.STA);
            GuiThread.Start();
            GuiThread.Join();
            return 1;
        }

        /// <summary>
        /// This method will open the edit window in STA mode
        /// if you want to open your custom window, override this method
        /// After your window closed, MUST abort GuiThread!!!
        /// </summary>
        [STAThread]
        protected virtual void OpenEditWindow()
        {
            CasterLogger.Debug("OpenEditWindow");
            ParameterWindow paramWindow = new ParameterWindow(this, Parameters, Results);
            if (paramWindow.ShowDialog() == true)
            {
                //foreach (var param in paramWindow.ViewModel.Parameters)
                //{
                //    ((ICapeParameter)Parameters[pair.Key]).value = ((ICapeParameter)pair.Value).value;
                //}
            }
            GuiThread.DisableComObjectEagerCleanup();
            GuiThread.Abort();
            CasterLogger.Debug("OpenEditWindow");
        }

        /// <summary>
        /// raw Parameters interface called by simulator
        /// </summary>
        object ICapeUtilities.parameters
        {
            get
            {
                OnGetParameters?.Invoke(this, Parameters);
                return Parameters;
            }
        }

        /// <summary>
        /// set and get simulationContext interface passed by simulator
        /// </summary>
        public object simulationContext
        {
            get { return _simulationContext; }
            set
            {
                _simulationContext = value as ICapeSimulationContext;
                CapeDiagnostic.SetSimulationContext(value as ICapeDiagnostic);
                CapeMaterialTemplateSystem.SetSimulationContext(value as ICapeMaterialTemplateSystem);
                CapeCOSEUtilities.SetSimulationContext(value as ICapeCOSEUtilities);
            }
        }

        #endregion

        #region ICapeUnitReport

        public virtual void ProduceReport(ref string message)
        {
            CasterLogger.Debug("ProduceReport");
            message = SelectedReportObj?.ProduceReport();
            CasterLogger.Debug("ProduceReport Complete");
        }

        dynamic ICapeUnitReport.reports =>
           (from report in Reports select report.Name).ToArray();

        public virtual ReportBase SelectedReportObj
        {
            get => _selectedReport != null ?
                _selectedReport :
                Reports.FirstOrDefault();
            set
            {
                if (!Reports.Contains(value))
                    throw new ECapeUnknownException(this,
                        $"{value.Name} is not an avaliable option for {ComponentName}",
                        null, typeof(ICapeUnitReport).ToString());
                _selectedReport = value;
            }
        }

        public virtual string selectedReport
        {
            get => _selectedReport.Name;
            set
            {
                var match = Reports.Find(x => x.Name == value);
                if (match == null)
                    throw new ECapeUnknownException(this,
                        $"{value} is not an avaliable option for {ComponentName}",
                        null, typeof(ICapeUnitReport).ToString());
                SelectedReportObj = match;
            }
        }

        #endregion

        #region IPersist
        /// <summary>
        /// return CLSID
        /// </summary>
        /// <paramCollection name="pClassID"></paramCollection>
        public void GetClassID(out Guid pClassID)
        {
            pClassID = this.GetType().GUID;
        }

        #endregion

        #region IPersistStream
        /// <summary>
        /// whether the unit has been modified, raw interface
        /// </summary>
        public virtual int IsDirty()
        {
            return Convert.ToInt16(Dirty);
        }
        /// <summary>
        /// Get Total size of saving
        /// </summary>
        public virtual void GetSizeMax(out long pcbSize)
        {
            object[] objArray = new object[5];
            objArray[0] = this.ComponentName;
            objArray[1] = this.ComponentDescription;
            var paramArray = new CapeCollectionPair[this.Parameters.Count];
            this.Parameters.CopyTo(paramArray, 0);
            objArray[2] = paramArray;
            var resultArray = new CapeCollectionPair[this.Results.Count];
            this.Results.CopyTo(resultArray, 0);
            objArray[3] = resultArray;
            var portArray = new CapeCollectionPair[this.Ports.Count];
            this.Ports.CopyTo(portArray, 0);
            objArray[4] = portArray;

            MemoryStream memoryStream = new MemoryStream();
            new BinaryFormatter().Serialize(memoryStream, objArray);
            pcbSize = memoryStream.Length;
        }

        /// <summary>
        /// Save ComponentName,ComponentDescription,Parameters,Results,Ports
        /// </summary>
        public virtual void Save(IStream pStm, [MarshalAs(UnmanagedType.U1)]bool fClearDirty)
        {
            CasterLogger.Debug("Save");
            byte[] pv1 = new byte[2];
            object[] objArray = new object[5];
            objArray[0] = this.ComponentName;
            objArray[1] = this.ComponentDescription;
            var paramArray = new CapeCollectionPair[this.Parameters.Count];
            this.Parameters.CopyTo(paramArray, 0);
            objArray[2] = paramArray;
            var resultArray = new CapeCollectionPair[this.Results.Count];
            this.Results.CopyTo(resultArray, 0);
            objArray[3] = resultArray;
            var portArray = new CapeCollectionPair[this.Ports.Count];
            this.Ports.CopyTo(portArray, 0);
            objArray[4] = portArray;

            MemoryStream memoryStream = new MemoryStream();
            new BinaryFormatter().Serialize(memoryStream, objArray);
            byte[] numArray = memoryStream.ToArray();
            memoryStream.Close();
            pv1[0] = (byte)(numArray.Length % 256);
            pv1[1] = (byte)(numArray.Length / 256);
            try
            {
                pStm.Write(pv1, 2, IntPtr.Zero);
                IStream stream = pStm;
                byte[] pv2 = numArray;
                int length = pv2.Length;
                IntPtr pcbWritten = IntPtr.Zero;
                stream.Write(pv2, length, pcbWritten);
                //Marshal.ReleaseComObject(pStm);
                if (!fClearDirty)
                    return;
                this.Dirty = false;
            }
            catch (Exception ex)
            {
                CasterLogger.Debug("Save Failed: " + ex.Message);
                MessageBox.Show(ex.ToString());
                //Marshal.ReleaseComObject(pStm);
            }
            CasterLogger.Debug("Save Success");
            CasterLogger.Debug("Save");
        }

        /// <summary>
        /// Load ComponentName,ComponentDescription,Parameters,Results,Ports
        /// </summary>
        public virtual void Load(IStream pStm)
        {
            CasterLogger.Debug("Load");
            this.Isloaded = true;
            byte[] pv = new byte[2];
            IntPtr pcbRead = IntPtr.Zero;
            pStm.Read(pv, 2, IntPtr.Zero);
            int cb = pv[1] * 256 + pv[0];
            byte[] numArray = new byte[cb];
            pStm.Read(numArray, cb, pcbRead);
            //Marshal.ReleaseComObject(pStm);
            MemoryStream memoryStream = new MemoryStream(numArray);
            BinaryFormatter binaryFormatter = new BinaryFormatter();
            binaryFormatter.Binder = new UBinder();
            try
            {
                CasterLocator.UnRegister(this);
                object[] objArray = binaryFormatter.Deserialize(memoryStream) as object[];
                this.ComponentName = objArray[0].ToString();
                this.ComponentDescription = objArray[1].ToString();

                //Parameters.Clear();
                //If the parameters have changed, the new or diffent parameter will not be replaced
                foreach (var pair in objArray[2] as CapeCollectionPair[])
                {
                    //Parameters.Add(pair);
                    try
                    {
                        ((CapeParameterBase)Parameters[pair.Key]).value = ((CapeParameterBase)pair.Value).value;
                    }
                    catch (Exception e)
                    {
                        CasterLogger.Error(e.Message);
                    }
                }
                //Results.Clear();
                foreach (var pair in objArray[3] as CapeCollectionPair[])
                {
                    //Results.Add(pair);
                    try
                    {
                        ((CapeParameterBase)Results[pair.Key]).value = ((CapeParameterBase)pair.Value).value;
                    }
                    catch (Exception e)
                    {
                        CasterLogger.Error(e.Message);
                    }
                }
                //Ports.Clear();
                foreach (var pair in objArray[4] as CapeCollectionPair[])
                {
                    //Ports.Add(pair);
                    try
                    {
                        ((CapeParameterBase)Ports[pair.Key]).value = ((CapeParameterBase)pair.Value).value;
                    }
                    catch (Exception e)
                    {
                        CasterLogger.Error(e.Message);
                    }
                }
            }
            catch (Exception ex)
            {
                CasterLogger.Debug("Load failed: " + ex.Message);
                MessageBox.Show(ex.ToString());
            }
            CasterLogger.Debug("Load success");
            memoryStream.Close();
        }

        #endregion

        #region Register

        /// <summary>
        /// register function, no need to modify, the information will get through the Attribute of UnitOp class
        /// For registry, run regasm xxx.dll, then run regasm xxx.dll /tlb xxx.tlb /codebase
        /// </summary>
        /// <paramCollection name="t"></paramCollection>
        public static void RegisterFunction(Type t)
        {
            string progId = "";
            string className = "";
            string description = "";
            string progIdVersionIndenpent = "";
            string capeVersion = "";
            string about = "";
            string vendorURL = "";
            string helpURL = "";
            string componentVersion = "";
            Guid typeLibID = Guid.Empty;
            List<Guid> categoryGUIDs = new List<Guid>();
            //Get attribute like CapeName
            Assembly assembly = t.Assembly;
            foreach (var attribute in t.GetCustomAttributes(true))
            {
                progIdVersionIndenpent = attribute is CapeNameAttribute ? ((CapeNameAttribute)attribute).Name : progIdVersionIndenpent;
                description = attribute is CapeDescriptionAttribute ? ((CapeDescriptionAttribute)attribute).Description : description;
                capeVersion = attribute is CapeVersionAttribute ? ((CapeVersionAttribute)attribute).Version : capeVersion;
                about = attribute is CapeAboutAttribute ? ((CapeAboutAttribute)attribute).About : about;
                vendorURL = attribute is CapeVendorURLAttribute ? ((CapeVendorURLAttribute)attribute).VendorURL : vendorURL;
                helpURL = attribute is CapeHelpURLAttribute ? ((CapeHelpURLAttribute)attribute).HelpURL : helpURL;
                typeLibID = attribute is TypeLibIDAttribute ? ((TypeLibIDAttribute)attribute).GUID : typeLibID;
                if (attribute is CapeCategoryAttribute)
                    categoryGUIDs.Add(((CapeCategoryAttribute)attribute).GUID);
            }
            className = t.ToString();
            if (progIdVersionIndenpent == "") progIdVersionIndenpent = t.Namespace;
            componentVersion = t.Assembly.GetName().Version.ToString();
            progId = progIdVersionIndenpent + capeVersion;
            var classGuid = t.GUID.ToString("B");

            //progID
            RegistryKey keyProgID = Registry.ClassesRoot.CreateSubKey(progId);
            Console.WriteLine($"Open key {keyProgID.Name}");
            try
            {
                keyProgID.SetValue(null, className, RegistryValueKind.String);
                RegistryKey subKey = keyProgID.CreateSubKey("CLSID");
                subKey.SetValue(null, classGuid);
                Console.WriteLine($"Set CLSID to {classGuid}");
                subKey.Close();
            }
            catch (System.Exception e)
            {
                MessageBox.Show(e.Message);
            }
            keyProgID.Close();

            //versionIndenpentProgID
            RegistryKey keyProgIDVersionIndenpent = Registry.ClassesRoot.CreateSubKey(progIdVersionIndenpent);
            Console.WriteLine($"Open key {keyProgIDVersionIndenpent}");
            try
            {
                keyProgIDVersionIndenpent.SetValue(null, className, RegistryValueKind.String);
                RegistryKey subKey = keyProgIDVersionIndenpent.CreateSubKey("CLSID");
                subKey.SetValue(null, classGuid);
                Console.WriteLine($"Set CLSID to {classGuid}");
                subKey.Close();
                subKey = keyProgIDVersionIndenpent.CreateSubKey("CurVer");
                subKey.SetValue(null, progId);
                Console.WriteLine($"Set CurVer to {progId}");
                subKey.Close();
            }
            catch (System.Exception e)
            {
                MessageBox.Show(e.Message);
            }
            keyProgIDVersionIndenpent.Close();

            //CLSID
            RegistryKey keyCLSID = Registry.ClassesRoot.OpenSubKey("CLSID", true);
            try
            {
                keyCLSID = keyCLSID.CreateSubKey(classGuid);
                Console.WriteLine($"Open key {keyCLSID}");
                keyCLSID.SetValue(null, className);
                Console.WriteLine($"Set value to {className}");

                RegistryKey subKey;

                subKey = keyCLSID.CreateSubKey("ProgID");
                subKey.SetValue(null, progId);
                Console.WriteLine($"Set ProgID to {progId}");
                subKey.Close();

                subKey = keyCLSID.CreateSubKey("VersionIndependentProgID");
                subKey.SetValue(null, progIdVersionIndenpent);
                Console.WriteLine($"Set VersionIndependentProgID to {progIdVersionIndenpent}");
                subKey.Close();

                subKey = keyCLSID.CreateSubKey("AppID");
                subKey.SetValue(null, progId);
                Console.WriteLine($"Set AppID to {progId}");
                subKey.Close();

                if (typeLibID != Guid.Empty)
                {
                    subKey = keyCLSID.CreateSubKey("TypeLib");
                    subKey.SetValue(null, typeLibID.ToString("B"));
                    Console.WriteLine($"Set TypeLib to {progId}");
                    subKey.Close();
                }

                subKey = keyCLSID.CreateSubKey("Implemented Categories");
                Console.WriteLine($"Open key {subKey}");
                subKey.CreateSubKey("{678C09A5-7D66-11D2-A67D-00105A42887F}");
                subKey.CreateSubKey("{678C09A1-7D66-11D2-A67D-00105A42887F}");
                subKey.CreateSubKey("{62C8FE65-4EBB-45e7-B440-6E39B2CDBF29}");
                Console.WriteLine($"Add default category");
                foreach (var guid in categoryGUIDs)
                    if (guid != Guid.Empty)
                    {
                        subKey.CreateSubKey(guid.ToString("B"));
                        Console.WriteLine($"Add {guid}");
                    }
                subKey.Close();

                subKey = keyCLSID.CreateSubKey("CapeDescription");
                Console.WriteLine($"Open key {subKey}");
                subKey.SetValue("Name", progIdVersionIndenpent);
                Console.WriteLine($"Set Name to {progIdVersionIndenpent}");
                subKey.SetValue("Description", description);
                Console.WriteLine($"Set Description to {description}");
                subKey.SetValue("CapeVersion", capeVersion);
                Console.WriteLine($"Set CapeVersion to {capeVersion}");
                subKey.SetValue("ComponentVersion", componentVersion);
                Console.WriteLine($"Set ComponentVersion to {componentVersion}");
                subKey.SetValue("VendorURL", vendorURL);
                Console.WriteLine($"Set VendorURL to {vendorURL}");
                subKey.SetValue("HelpURL", helpURL);
                Console.WriteLine($"Set HelpURL to {helpURL}");
                subKey.SetValue("About", about);
                Console.WriteLine($"Set About to {about}");
                subKey.Close();
            }
            catch (System.Exception e)
            {
                MessageBox.Show(e.Message);
            }
            Console.WriteLine($"Complete write to {keyCLSID}");
            keyCLSID.Close();
        }

        /// <summary>
        /// Unregister function
        /// </summary>
        public static void UnRegisterFunction(Type t)
        {
            string progId = "";
            string progIdVersionIndenpent = "";
            string capeVersion = "";
            Assembly assembly = t.Assembly;
            foreach (var attribute in t.GetCustomAttributes(false))
            {
                progIdVersionIndenpent = attribute is CapeNameAttribute ? ((CapeNameAttribute)attribute).Name : progIdVersionIndenpent;
                capeVersion = attribute is CapeVersionAttribute ? ((CapeVersionAttribute)attribute).Version : capeVersion;
            }
            progId = progIdVersionIndenpent + capeVersion;
            Registry.ClassesRoot.DeleteSubKeyTree(progId, false);
            Console.WriteLine($"Delete {progId}");
            Registry.ClassesRoot.DeleteSubKeyTree(progIdVersionIndenpent, false);
            Console.WriteLine($"Delete {progIdVersionIndenpent}");
            Registry.ClassesRoot.OpenSubKey("CLSID", true).DeleteSubKeyTree(t.GUID.ToString("B"), false);
            Console.WriteLine($"Delete CLSID {t.GUID.ToString("B")}");
        }
        #endregion
    }
}
