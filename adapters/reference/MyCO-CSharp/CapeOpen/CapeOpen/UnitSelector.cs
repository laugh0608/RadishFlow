using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Windows.Forms;

namespace CapeOpen
{
    /// <summary>
    /// The unit selector class provides a graphical user interface (GUI) for the <see cref="UnitOperationWrapper"/> class.
    /// </summary>
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public partial class UnitSelector : Form
    {
        Type m_Unit;
        List<String> addedTypes = new List<String>();

        /// <summary>
        /// Creates a new instance of the <see cref="UnitSelector"/> class.
        /// </summary>
        public UnitSelector()
        {
            InitializeComponent();


            this.DialogResult = System.Windows.Forms.DialogResult.Cancel;
            System.Reflection.Assembly assembly = System.Reflection.Assembly.GetExecutingAssembly();
            String directory = assembly.Location;
            Type[] types = assembly.GetTypes();
            TreeNode node = new TreeNode("Example Unit");
            this.treeView1.Nodes.Add(node);
            foreach (Type type in types)
            {
                if (type.GUID != new Guid("B41DECE0-6C99-4CA4-B0EB-EFADBDCE23E9"))
                    AddType(type, node);
            }
            if (System.Diagnostics.Debugger.IsAttached)
            {
                //System.Windows.Forms.MessageBox.Show("This option is currently not supported.");
                node = new TreeNode("Debugged Unit");
                this.treeView1.Nodes.Add(node);
                Object obj = System.Runtime.InteropServices.Marshal.GetActiveObject("VisualStudio.DTE.10.0");
                EnvDTE.DTE DTE = (EnvDTE.DTE)obj;
                try
                {
                    EnvDTE.Project project = (EnvDTE.Project)DTE.Solution.Projects.Item(1);
                    String name = project.FullName;
                    if (name != "")
                    {
                        String path = String.Concat(System.IO.Path.GetDirectoryName(name), "\\bin\\debug");
                        if (System.IO.Directory.Exists(path))
                        {
                            String[] files = System.IO.Directory.GetFiles(path);
                            foreach (String file in files)
                            {
                                String dll = file.Remove(0, file.Length - 3);
                                if (dll.ToLower() == "dll")
                                {
                                    System.Reflection.Assembly assembly2 = System.Reflection.Assembly.LoadFrom(file);
                                    Type[] types2 = assembly2.GetTypes();
                                    foreach (Type type in types2)
                                    {
                                        if (type.GUID != new Guid("883D46FE-5713-424C-BF10-7ED34263CD6D")
                                                 && type.GUID != new Guid("56E8FDFD-2000-4264-9B47-745B26BE0EC9")
                                                 && type.GUID != new Guid("B41DECE0-6C99-4CA4-B0EB-EFADBDCE23E9")
                                           )
                                            AddType(type, node);
                                    }
                                }
                            }
                        }
                        path = String.Concat(System.IO.Path.GetDirectoryName(name), "\\debug");
                        if (System.IO.Directory.Exists(path))
                        {
                            String[] files = System.IO.Directory.GetFiles(path);
                            foreach (String file in files)
                            {
                                String dll = file.Remove(0, file.Length - 3);
                                if (dll.ToLower() == "dll")
                                {
                                    System.Reflection.Assembly assembly2 = System.Reflection.Assembly.LoadFrom(file);
                                    Type[] types2 = assembly2.GetTypes();
                                    foreach (Type type in types2)
                                    {
                                        if (type.GUID != new Guid("883D46FE-5713-424C-BF10-7ED34263CD6D")
                                                 && type.GUID != new Guid("56E8FDFD-2000-4264-9B47-745B26BE0EC9")
                                                 && type.GUID != new Guid("B41DECE0-6C99-4CA4-B0EB-EFADBDCE23E9")
                                           )
                                            AddType(type, node);
                                    }
                                }
                            }
                        }
                    }
                }
                catch (System.Exception p_Ex)
                {
                    System.Windows.Forms.MessageBox.Show(p_Ex.Message);
                }
            }
            node = null;
            String CapeOpenDirectory = String.Concat(Environment.GetEnvironmentVariable("CommonProgramFiles"), "\\CAPE-OPEN Objects");
            if (System.IO.Directory.Exists(CapeOpenDirectory))
            {
                node = new TreeNode("CapeOpen Units");
                this.treeView1.Nodes.Add(node);
                AddPath(CapeOpenDirectory, node);
            }
            CapeOpenDirectory = "C:\\CAPE-OPEN Objects";
            if (System.IO.Directory.Exists(CapeOpenDirectory))
            {
                if (node == null)
                {
                    node = new TreeNode("CapeOpen Units");
                    this.treeView1.Nodes.Add(node);
                }
                AddPath(CapeOpenDirectory, node);
            }
        }

        /// <summary>
        /// The type of the unit operation to be created.
        /// </summary>
        /// <value>The type of the unit operation to be created.</value>
        public Type Unit
        {
            get
            {
                return m_Unit;
            }
        }

        private void AddType(Type type, TreeNode node)
        {
            if (!type.IsAbstract)
            {
                if (typeof(CapeOpen.ICapeUnit).IsAssignableFrom(type))
                {
                    if (type.FullName != "CapeOpen.UnitOperationManager")
                    {
                        System.Reflection.Assembly assembly = type.Assembly;
                        String versionNumber = (new System.Reflection.AssemblyName(assembly.FullName)).Version.ToString();
                        Object p_Unit = Activator.CreateInstance(type);
                        ICapeIdentification p_Id = (ICapeIdentification)p_Unit;
                        System.Object[] attributes = type.GetCustomAttributes(false);
                        String nameInfoString = type.FullName;
                        String descriptionInfoString = "";
                        String versionInfoString = "";
                        String companyURLInfoString = "";
                        String helpURLInfoString = "";
                        String aboutInfoString = "";
                        bool consumesThermo = false;
                        bool thermo10 = false;
                        bool thermo11 = false;
                        for (int i = 0; i < attributes.Length; i++)
                        {
                            if (attributes[i] is CapeConsumesThermoAttribute) consumesThermo = true;
                            if (attributes[i] is CapeSupportsThermodynamics10Attribute) thermo10 = true;
                            if (attributes[i] is CapeSupportsThermodynamics11Attribute) thermo11 = true;
                            if (attributes[i] is CapeNameAttribute) nameInfoString = ((CapeNameAttribute)attributes[i]).Name;
                            if (attributes[i] is CapeDescriptionAttribute) descriptionInfoString = ((CapeDescriptionAttribute)attributes[i]).Description;
                            if (attributes[i] is CapeVersionAttribute) versionInfoString = ((CapeVersionAttribute)attributes[i]).Version;
                            if (attributes[i] is CapeVendorURLAttribute) companyURLInfoString = ((CapeVendorURLAttribute)attributes[i]).VendorURL;
                            if (attributes[i] is CapeHelpURLAttribute) helpURLInfoString = ((CapeHelpURLAttribute)attributes[i]).HelpURL;
                            if (attributes[i] is CapeAboutAttribute) aboutInfoString = ((CapeAboutAttribute)attributes[i]).About;
                        }

                        TreeNode newNode = new TreeNode(nameInfoString);
                        node.Nodes.Add(newNode);
                        newNode.Tag = type;
                        newNode.Nodes.Add(String.Concat("Description: ", descriptionInfoString));
                        newNode.Nodes.Add(String.Concat("CapeVersion: ", versionInfoString));
                        newNode.Nodes.Add(String.Concat("ComponentVersion: ", versionNumber));
                        newNode.Nodes.Add(String.Concat("VendorURL: ", companyURLInfoString));
                        newNode.Nodes.Add(String.Concat("HelpURL: ", helpURLInfoString));
                        newNode.Nodes.Add(String.Concat("About: ", aboutInfoString));
                        if (consumesThermo)
                        {
                            TreeNode thermoNode = new TreeNode("Supported Thermo Versions");
                            newNode.Nodes.Add(thermoNode);
                            if (thermo10) thermoNode.Nodes.Add("Supports Thermo 1.0");
                            if (thermo11) thermoNode.Nodes.Add("Supports Thermo 1.1");
                        }
                    }
                }
            }
        }

        private void AddPath(String path, TreeNode node)
        {
            if (System.IO.Directory.Exists(path))
            {
                String[] directories = System.IO.Directory.GetDirectories(path);
                foreach (String directory in directories)
                {
                    String name = directory.Remove(0, path.Length + 1);
                    TreeNode newNode = new TreeNode(name);
                    node.Nodes.Add(newNode);
                    AddPath(directory, newNode);
                }
                String[] files = System.IO.Directory.GetFiles(path);
                foreach (String file in files)
                {
                    String dll = file.Remove(0, file.Length - 3);
                    if (dll.ToLower() == "dll")
                    {
                        System.Reflection.Assembly assembly = System.Reflection.Assembly.LoadFrom(file);
                        Type[] types = null;
                        types = assembly.GetTypes();
                        foreach (Type type in types)
                        {
                            if (type.GUID != new Guid("883D46FE-5713-424C-BF10-7ED34263CD6D")
                                      && type.GUID != new Guid("56E8FDFD-2000-4264-9B47-745B26BE0EC9")
                                      && type.GUID != new Guid("B41DECE0-6C99-4CA4-B0EB-EFADBDCE23E9")
                                )
                                AddType(type, node);
                        }
                    }
                }
            }
        }

        private void openToolStripMenuItem_Click(object sender, EventArgs e)
        {
            System.Windows.Forms.OpenFileDialog fileDialog = new System.Windows.Forms.OpenFileDialog();
            fileDialog.Title = "Browse to Assebly Containing Desired Unit Operation";
            fileDialog.ShowDialog();
            String fileName = fileDialog.FileName;
            String file = System.IO.Path.GetFileNameWithoutExtension(fileName);
            TreeNode node = new TreeNode(String.Concat("Units from File: ", file));
            this.treeView1.Nodes.Add(node);
            String dll = fileName.Remove(0, fileName.Length - 3);
            if (dll.ToLower() == "dll")
            {
                System.Reflection.Assembly assembly = System.Reflection.Assembly.LoadFrom(fileName);
                Type[] types = null;
                types = assembly.GetTypes();
                foreach (Type type in types)
                {
                    if (type.GUID != new Guid("883D46FE-5713-424C-BF10-7ED34263CD6D")
                                      && type.GUID != new Guid("56E8FDFD-2000-4264-9B47-745B26BE0EC9")
                                      && type.GUID != new Guid("B41DECE0-6C99-4CA4-B0EB-EFADBDCE23E9")
                                )
                        AddType(type, node);
                }
            }
        }

        private void helpToolStripMenuItem1_Click(object sender, EventArgs e)
        {

        }

        private void CancelButton_Click(object sender, EventArgs e)
        {
            throw new CapeOpen.CapeUnknownException("Unit Creation Cancelled.");
            // this.Close();
        }

        private void OKbutton_Click(object sender, EventArgs e)
        {
            TreeNode node = this.treeView1.SelectedNode;
            m_Unit = (Type)node.Tag;
            if (node.Parent == null) return;
            if (m_Unit == null) m_Unit = (Type)node.Parent.Tag;
            if (m_Unit == null) return;
            this.DialogResult = System.Windows.Forms.DialogResult.OK;
            this.Close();
        }
    }
}
