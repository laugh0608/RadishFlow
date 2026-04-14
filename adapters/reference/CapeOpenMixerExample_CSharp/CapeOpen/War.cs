using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{
    /// <summary>
    /// Summary for WAR
    /// </summary>
    [Serializable()]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("0BE9CCFD-29B4-4a42-B34E-76F5FE9B6BB4")]
    [CapeOpen.CapeName("WAR Add-In")]
    [CapeOpen.CapeAbout("Waste Reduction Algorithm Add-in")]
    [CapeOpen.CapeDescription("Waste Reduction Algorithm Add-in")]
    [CapeOpen.CapeVersion("1.0")]
    [CapeOpen.CapeVendorURL("http://www.epa.gov/nrmrl/std/sab/war/sim_war.htm")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    [CapeOpen.CapeFlowsheetMonitoringAttribute(true)]
    public class WARAddIn : CapeObjectBase
    {

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
            return new WARAddIn();
        }

        private System.Data.DataTable warDataTable = new System.Data.DataTable();
        private void AddData(String XmlData)
        {
            System.Xml.XmlDocument document = new System.Xml.XmlDocument();
            document.LoadXml(XmlData);
            System.Xml.XmlNodeList list = document.SelectNodes("dataroot/data");
            System.Collections.IEnumerator ienum = list.GetEnumerator();
            while (ienum.MoveNext())
            {
                System.Xml.XmlNode current = (System.Xml.XmlNode)ienum.Current;
                System.Data.DataRow dataRow = warDataTable.NewRow();
                warDataTable.Rows.Add(dataRow);
                System.Collections.IEnumerator childEnum = current.ChildNodes.GetEnumerator();
                while (childEnum.MoveNext())
                {
                    System.Xml.XmlNode currentChild = (System.Xml.XmlNode)childEnum.Current;
                    if (currentChild.Name == "Mol_ID")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["Mol Id"] = value;
                    }
                    if (currentChild.Name == "DIPPR ID")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["DIPPR ID"] = value;
                    }
                    if (currentChild.Name == "ASPENID")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["ASPENID"] = value;
                    }
                    if (currentChild.Name == "ChemicalName")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["ChemicalName"] = value;
                    }
                    if (currentChild.Name == "CAS")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["CAS"] = value;
                    }
                    if (currentChild.Name == "Formula")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["Formula"] = value;
                    }
                    if (currentChild.Name == "CLASS")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["class"] = value;
                    }
                    if (currentChild.Name == "MW")
                    {
                        dataRow["molecularWeight"] = Convert.ToDouble(currentChild.InnerText);
                    }
                    if (currentChild.Name == "Rat_LD50_Value")
                    {
                        dataRow["Rat LD50"] = Convert.ToDouble(currentChild.InnerText);
                    }
                    if (currentChild.Name == "Rat_LD50_Notes")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["Rat LD50 Notes"] = value;
                    }
                    if (currentChild.Name == "Rat_LD50_Source")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["Rat LD50 Source"] = value;
                    }
                    if (currentChild.Name == "OSHA_TWA_Value")
                    {
                        dataRow["OSHA PEL"] = Convert.ToDouble(currentChild.InnerText);
                    }
                    if (currentChild.Name == "OSHA_TWA_Source")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["OSHA Source"] = value;
                    }
                    if (currentChild.Name == "OSHA_TWA_Notes")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["OSHA Notes"] = value;
                    }
                    if (currentChild.Name == "FHM_LC50_Value")
                    {
                        dataRow["Fathead LC50"] = Convert.ToDouble(currentChild.InnerText);
                    }
                    if (currentChild.Name == "FHM_LC50_Notes")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["Fathead LC50 Notes"] = value;
                    }
                    if (currentChild.Name == "FHM_LC50_Source")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["Fathead LC50 Source"] = value;
                    }
                    if (currentChild.Name == "PCO_Value")
                    {
                        dataRow["Photochemical Oxidation Potential"] = Convert.ToDouble(currentChild.InnerText);
                    }
                    if (currentChild.Name == "PCO_Source")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["Photochemical Oxidation Potential Source"] = value;
                    }
                    if (currentChild.Name == "GWP_Value")
                    {
                        dataRow["Global Warming Potential"] = Convert.ToDouble(currentChild.InnerText);
                    }
                    if (currentChild.Name == "GWP_Source")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["Global Warming Potential Source"] = value;
                    }
                    if (currentChild.Name == "OD_Value")
                    {
                        dataRow["Ozone Depletion Potential"] = Convert.ToDouble(currentChild.InnerText);
                    }
                    if (currentChild.Name == "OD_Source")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["Ozone Depletion Potential Source"] = value;
                    }
                    if (currentChild.Name == "AP_Value")
                    {
                        dataRow["Acidification Potential"] = Convert.ToDouble(currentChild.InnerText);
                    }
                    if (currentChild.Name == "AP_Source")
                    {
                        String value = currentChild.InnerText;
                        while (value.StartsWith(" ")) value = value.Substring(1);
                        while (value.EndsWith(" ")) value = value.Remove(value.Length - 1);
                        dataRow["Acidification Potential Source"] = value;
                    }
                }
            }
        }

        /// <summary>
        ///	Displays the PMC graphic interface, if available.
        /// </summary>
        /// <remarks>
        /// The PMC displays its user interface and allows the Flowsheet User to 
        /// interact with it. If no user interface is available it returns an error.
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        public WARAddIn()
        {
            warDataTable = new System.Data.DataTable();
            warDataTable.Columns.Add("Mol Id", typeof(System.String));
            warDataTable.Columns.Add("DIPPR ID", typeof(System.String));
            warDataTable.Columns.Add("ASPENID", typeof(System.String));
            warDataTable.Columns.Add("ChemicalName", typeof(System.String));
            warDataTable.Columns.Add("CAS", typeof(System.String));
            warDataTable.Columns.Add("Formula", typeof(System.String));
            warDataTable.Columns.Add("class", typeof(System.String));
            warDataTable.Columns.Add("molecularWeight", typeof(double));
            warDataTable.Columns.Add("Rat LD50", typeof(double));
            warDataTable.Columns.Add("Rat LD50 Notes", typeof(System.String));
            warDataTable.Columns.Add("Rat LD50 Source", typeof(System.String));
            warDataTable.Columns.Add("OSHA PEL", typeof(double));
            warDataTable.Columns.Add("OSHA Notes", typeof(System.String));
            warDataTable.Columns.Add("OSHA Source", typeof(System.String));
            warDataTable.Columns.Add("Fathead LC50", typeof(double));
            warDataTable.Columns.Add("Fathead LC50 Notes", typeof(System.String));
            warDataTable.Columns.Add("Fathead LC50 Source", typeof(System.String));
            warDataTable.Columns.Add("Global Warming Potential", typeof(double));
            warDataTable.Columns.Add("Global Warming Potential Source", typeof(System.String));
            warDataTable.Columns.Add("Ozone Depletion Potential", typeof(double));
            warDataTable.Columns.Add("Ozone Depletion Potential Source", typeof(System.String));
            warDataTable.Columns.Add("Photochemical Oxidation Potential", typeof(double));
            warDataTable.Columns.Add("Photochemical Oxidation Potential Source", typeof(System.String));
            warDataTable.Columns.Add("Acidification Potential", typeof(double));
            warDataTable.Columns.Add("Acidification Potential Source", typeof(System.String));
            System.AppDomain domain = System.AppDomain.CurrentDomain;
            //System.Reflection.Assembly myAssembly = System.Reflection.Assembly.GetExecutingAssembly();
            //String[] resources = myAssembly.GetManifestResourceNames();
            //System.IO.Stream resStream = myAssembly.GetManifestResourceStream("CapeOpen.Resources.WARdata.xml.resources");
            //System.Resources.ResourceReader resReader = new System.Resources.ResourceReader(resStream);
            //System.Collections.IDictionaryEnumerator en = resReader.GetEnumerator();
            //String temp = String.Empty;
            //while (en.MoveNext())
            //{
            //    if (en.Key.ToString() == "WARdata") temp = en.Value.ToString();
            //}
            this.AddData(Properties.Resources.WARdata);
        }

        /// <summary>
        ///	Displays the PMC graphic interface, if available.
        /// </summary>
        /// <remarks>
        /// The PMC displays its user interface and allows the Flowsheet User to 
        /// interact with it. If no user interface is available it returns an error.
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        public override System.Windows.Forms.DialogResult Edit()
        {
            try
            {
                CapeOpen.WARalgorithm war = new WARalgorithm(this.warDataTable, this.FlowsheetMonitoring);
                System.Windows.Forms.DialogResult result = war.ShowDialog();
                war.Dispose();
                return result;
             }
            catch (System.Exception p_Ex)
            {
                this.throwException(new CapeOpen.CapeUnknownException(p_Ex.Message, p_Ex));
                return System.Windows.Forms.DialogResult.Cancel;
            }
        }
    }
}
