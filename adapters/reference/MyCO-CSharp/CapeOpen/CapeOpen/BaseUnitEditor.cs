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
    /// Base class for a unit operation editor.
    /// </summary>
    /// <remarks> This editor can be used by all unit operations. It creates a tabbed form 
    /// that exposes the unit's properties (parameters and ports) on the first tab. This 
    /// editor can be inherited and tabs added to the tab control to customize the form
    /// for a unit operation.
    /// </remarks>
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public partial class BaseUnitEditor : Form
    {
        private CapeUnitBase m_Unit;

        /// <remarks>
        /// Constructor for a standard unit operation editor.
        /// </remarks>
        /// <param name = "unit">The unit operation to be edited.</param>
        public BaseUnitEditor(CapeUnitBase unit)
        {
            InitializeComponent();
            m_Unit = unit;
            this.propertyGrid1.SelectedObject = unit;
        }

        private void CloseButton_Click(object sender, EventArgs e)
        {
            this.DialogResult = System.Windows.Forms.DialogResult.OK;
            this.Close();
        }
    }
}
