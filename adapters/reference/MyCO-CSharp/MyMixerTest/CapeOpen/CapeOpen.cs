using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

// <summary>
// This namespace provides a .Net translation of the CAPE-OPEN interfaces and implements 
// core components of the CAPE-OPEN object model. More information on CAPE-OPEN is available 
// from the CO-Lan website, http:\\www.co-lan.org.
// </summary>
namespace CapeOpen
{

    /// <summary>
    /// Provides a text name for the registration of a CAPE-OPEN object.</summary>
    /// <remarks>
    /// <para>
    /// This attribute is used during the registration of a CAPE-OPEN object with the
    /// COM registry to set the value of the CapeName[Name] registration key. </para>
    /// </remarks>
    /// <c>
    /// [Serializable]
    /// [System.Runtime.InteropServices.ComVisible(true)]
    /// [System.Runtime.InteropServices.Guid("C79CAFD3-493B-46dc-8585-1118A0B5B4AB")]//ICapeThermoMaterialObject_IID)
    /// [System.ComponentModel.Description("")]
    /// [CapeOpen.CapeNameAttribute("MixerExamle")]
    /// [CapeOpen.CapeDescriptionAttribute("An example mixer unit operation.")]
    /// [CapeOpen.CapeVersionAttribute("1.0")]
    /// [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    /// public class CMixerExample : public CUnitBase
    /// </c>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.AttributeUsageAttribute(System.AttributeTargets.Class)]
    public class CapeNameAttribute : System.Attribute
    {
        private String m_Name;
        /// <summary>Initializes a new instance of the CapeDescriptionAttribute class.</summary>
        /// <remarks>The value of the description is used by the registration function to set the value of the CapeDescription[Description] registration key.</remarks>
        /// <param name = "name">The CAPE-OPEN component description.</param>
        public CapeNameAttribute(String name)
        {
            m_Name = name;
        }

        /// <summary>Gets the the name information.</summary>
        /// <remarks>The value of the name.</remarks>
        /// <value>The CAPE-OPEN component name.</value>
        public String Name
        {
            get
            {
                return m_Name;
            }
        }
    };
    /// <summary>Provides a text description for the registration of a CAPE-OPEN object.</summary>
    /// <remarks><para>
    /// This attribute is used during the registration of a CAPE-OPEN object with the
    /// COM registry to set the value of the CapeDescription[Description] registration key. </para>
    /// </remarks>
    /// <c>
    /// [Serializable]
    /// [System.Runtime.InteropServices.ComVisible(true)]
    /// [System.Runtime.InteropServices.Guid("C79CAFD3-493B-46dc-8585-1118A0B5B4AB")]//ICapeThermoMaterialObject_IID)
    /// [System.ComponentModel.Description("")]
    /// [CapeOpen.CapeDescriptionAttribute("An example mixer unit operation.")]
    /// [CapeOpen.CapeVersionAttribute("1.0")]
    /// [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    /// public class CMixerExample : public CUnitBase
    /// </c>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.AttributeUsageAttribute(System.AttributeTargets.Class)]
    public class CapeDescriptionAttribute : System.Attribute
    {
        private String m_Description;

        /// <summary>Initializes a new instance of the CapeDescriptionAttribute class.</summary>
        /// <remarks>The value of the description is used by the registration function to set the value of the CapeDescription[Description] registration key.</remarks>
        /// <param name = "description">The CAPE-OPEN component description.</param>
        public CapeDescriptionAttribute(String description)
        {
            m_Description = description;
        }

        /// <summary>Gets the the description information.</summary>
        /// <remarks>The value of the description.</remarks>
        /// <value>The CAPE-OPEN component description.</value>
        public String Description
        {
            get
            {
                return m_Description;
            }
        }
    };
    /// <summary>Provides the CAPE-OPEN version number for the registration of a CAPE-OPEN object.</summary>
    /// <remarks><para>
    /// This attribute is used during the registration of a CAPE-OPEN object with the
    /// COM registry to set the value of the CapeDescription[CapeVersion] registration key. </para>
    /// </remarks>
    /// <c>
    /// [Serializable]
    /// [System.Runtime.InteropServices.ComVisible(true)]
    /// [System.Runtime.InteropServices.Guid("C79CAFD3-493B-46dc-8585-1118A0B5B4AB")]//ICapeThermoMaterialObject_IID)
    /// [System.ComponentModel.Description("")]
    /// [CapeOpen.CapeDescriptionAttribute("An example mixer unit operation.")]
    /// [CapeOpen.CapeVersionAttribute("1.0")]
    /// [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    /// public class CMixerExample : public CUnitBase
    /// </c>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.AttributeUsageAttribute(System.AttributeTargets.Class)]
    public class CapeVersionAttribute : System.Attribute
    {
        private String m_Version;
        /// <summary>Initializes a new instance of the CapeVersionAttribute class.</summary>
        /// <remarks>The value of the description is used by the registration function to set the value of the CapeDescription[CapeVersion] registration key.</remarks>
        /// <param name = "version">The version of the CAPE-OPEN interfaces that this object supports.</param>
        public CapeVersionAttribute(String version)
        {
            m_Version = version;
        }

        /// <summary>Gets the the CAPE-OPEN version number.</summary>
        /// <remarks>The value of the CAPE-OPEN version number.</remarks>
        /// <value>The CAPE-OPEN component CAPE-OPEN version number.</value>
        public String Version
        {
            get
            {
                return m_Version;
            }
        }
    };
    /// <summary>Provides a vendor URL for the registration of a CAPE-OPEN object.</summary>
    /// <remarks><para>
    /// This attribute is used during the registration of a CAPE-OPEN object with the
    /// COM registry to set the value of the CapeDescription[VendorURL] registration key. </para>
    /// </remarks>
    /// <c>
    /// [Serializable]
    /// [System.Runtime.InteropServices.ComVisible(true)]
    /// [System.Runtime.InteropServices.Guid("C79CAFD3-493B-46dc-8585-1118A0B5B4AB")]//ICapeThermoMaterialObject_IID)
    /// [System.ComponentModel.Description("")]
    /// [CapeOpen.CapeDescriptionAttribute("An example mixer unit operation.")]
    /// [CapeOpen.CapeVersionAttribute("1.0")]
    /// [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    /// public class CMixerExample : public CUnitBase
    /// </c>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.AttributeUsageAttribute(System.AttributeTargets.Class)]
    public class CapeVendorURLAttribute : System.Attribute
    {
        private String m_VendorURL;
        /// <summary>Initializes a new instance of the CapeVendorURLAttribute class.</summary>
        /// <remarks>The value of the description is used by the registration function to set the value of the CapeDescription[VendorURL] registration key.</remarks>
        /// <param name = "VendorURL">The CAPE-OPEN component Vendor URL.</param>
        public CapeVendorURLAttribute(String VendorURL)
        {
            m_VendorURL = VendorURL;
        }

        /// <summary>Gets the the VendorURL information.</summary>
        /// <remarks>The value of the VendorURL.</remarks>
        /// <value>The CAPE-OPEN component VendorURL.</value>
        public String VendorURL
        {
            get
            {
                return m_VendorURL;
            }
        }
    };
    /// <summary>Provides a help URL for the registration of a CAPE-OPEN object.</summary>
    /// <remarks><para>
    /// This attribute is used during the registration of a CAPE-OPEN object with the
    /// COM registry to set the value of the CapeDescription[HelpURL] registration key. </para>
    /// </remarks>
    /// <c>
    /// [Serializable]
    /// [System.Runtime.InteropServices.ComVisible(true)]
    /// [System.Runtime.InteropServices.Guid("C79CAFD3-493B-46dc-8585-1118A0B5B4AB")]//ICapeThermoMaterialObject_IID)
    /// [System.ComponentModel.Description("")]
    /// [CapeOpen.CapeDescriptionAttribute("An example mixer unit operation.")]
    /// [CapeOpen.CapeVersionAttribute("1.0")]
    /// [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    /// public class CMixerExample : public CUnitBase
    /// </c>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.AttributeUsageAttribute(System.AttributeTargets.Class)]
    public class CapeHelpURLAttribute : System.Attribute
    {
        private String m_HelpURL;

        /// <summary>Initializes a new instance of the CapeHelpURLAttribute class.</summary>
        /// <remarks>The value of the Help URL is used by the registration function to set the value of the CapeDescription[HelpURL] registration key.</remarks>
        /// <param name = "HelpURL">The CAPE-OPEN component Help URL.</param>
        public CapeHelpURLAttribute(String HelpURL)
        {
            m_HelpURL = HelpURL;
        }

        /// <summary>Gets the the HelpURL information.</summary>
        /// <remarks>The value of the HelpURL.</remarks>
        /// <value>The CAPE-OPEN component HelpURL.</value>
        public String HelpURL
        {
            get
            {
                return m_HelpURL;
            }
        }
    };

    /// <summary>Provides text about information for the registration of a CAPE-OPEN object.</summary>
    /// <remarks><para>
    /// This attribute is used during the registration of a CAPE-OPEN object with the
    /// COM registry to set the value of the CapeDescription[About] registration key. </para>
    /// </remarks>
    /// <c>
    /// [Serializable]
    /// [System.Runtime.InteropServices.ComVisible(true)]
    /// [System.Runtime.InteropServices.Guid("C79CAFD3-493B-46dc-8585-1118A0B5B4AB")]//ICapeThermoMaterialObject_IID)
    /// [System.ComponentModel.Description("")]
    /// [CapeOpen.CapeDescriptionAttribute("An example mixer unit operation.")]
    /// [CapeOpen.CapeVersionAttribute("1.0")]
    /// [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    /// public class CMixerExample : public CUnitBase
    /// </c>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.AttributeUsageAttribute(System.AttributeTargets.Class)]
    public class CapeAboutAttribute : System.Attribute
    {
        private String m_About;
        /// <summary>Initializes a new instance of the CapeAboutAttribute class.</summary>
        /// <remarks>The value of the about text is used by the registration function to set the value of the CapeDescription[About] registration key.</remarks>
        /// <param name = "About">The CAPE-OPEN component about information.</param>
        public CapeAboutAttribute(String About)
        {
            m_About = About;
        }

        /// <summary>Gets the the about information.</summary>
        /// <remarks>The value of the about information.</remarks>
        /// <value>The CAPE-OPEN component about information.</value>
        public String About
        {
            get
            {
                return m_About;
            }
        }
    };


    /// <summary>Provides information regarding whether the object is a CAPE-OPEN Unit Operation used  
    /// during registration of a CAPE-OPEN object.</summary>
    /// <remarks><para>
    /// This attribute is used during the registration of a CAPE-OPEN object with the
    /// COM registry to add the CapeUnitOperation_CATID to the object's registration key. </para>
    /// </remarks>
    /// <c>
    /// [Serializable]
    /// [System.Runtime.InteropServices.ComVisible(true)]
    /// [System.Runtime.InteropServices.Guid("C79CAFD3-493B-46dc-8585-1118A0B5B4AB")]//ICapeThermoMaterialObject_IID)
    /// [System.ComponentModel.Description("")]
    /// [CapeOpen.CapeDescriptionAttribute("An example mixer unit operation.")]
    /// [CapeOpen.CapeVersionAttribute("1.0")]
    /// [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    /// [CapeOpen.CapeUnitOperation_CATID(true)]
    /// public class CMixerExample : public CUnitBase
    /// </c>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.AttributeUsageAttribute(System.AttributeTargets.Class)]
    public class CapeUnitOperationAttribute : System.Attribute
    {
        private bool m_UnitOp;
        /// <summary>Initializes a new instance of the CapeUnitOperationAttribute class.</summary>
        /// <remarks>This attribute is used to indicate whether the object is a CAPE-OPEN Unit Operation.
        /// It is also used by the COM registration function to place the appropriate CATID value in the system registry
        /// for this object.</remarks>
        /// <param name = "isUnit">The CAPE-OPEN component is a CAPE-OPEN Unit Operation object.</param>
        public CapeUnitOperationAttribute(bool isUnit)
        {
            m_UnitOp = isUnit;
        }

        /// <summary>Gets the the about information.</summary>
        /// <remarks>This property indicates whether the object uses the CAPE-OPEN Unti Operation interfaces.</remarks>
        /// <value>A boolean value indicating whether the CAPE-OPEN component is a Unit Operation.</value>
        public bool IsUnit
        {
            get
            {
                return m_UnitOp;
            }
        }
    };

    /// <summary>Provides information regarding whether the object supports Flowsheet monitoring used  
    /// during registration of a CAPE-OPEN object.</summary>
    /// <remarks><para>
    /// This attribute is used during the registration of a CAPE-OPEN object with the
    /// COM registry to add the CATID_MONITORING_OBJECT to the object's registration key. </para>
    /// </remarks>
    /// <c>
    /// [Serializable]
    /// [System.Runtime.InteropServices.ComVisible(true)]
    /// [System.Runtime.InteropServices.Guid("C79CAFD3-493B-46dc-8585-1118A0B5B4AB")]//ICapeThermoMaterialObject_IID)
    /// [System.ComponentModel.Description("")]
    /// [CapeOpen.CapeDescriptionAttribute("An example mixer unit operation.")]
    /// [CapeOpen.CapeVersionAttribute("1.0")]
    /// [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    /// [CapeOpen.CapeFlowsheetMonitoringAttribute(true)]
    /// public class WARAddIn : CapeObjectBase
    /// </c>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.AttributeUsageAttribute(System.AttributeTargets.Class)]
    public class CapeFlowsheetMonitoringAttribute : System.Attribute
    {
        private bool m_Monitors;
        /// <summary>Initializes a new instance of the CapeFlowsheetMonitoringAttribute class.</summary>
        /// <remarks>This attribute is used to indicate whether the object uses the flowsheet monitoring capbility of a PME.
        /// It is also used by the COM registration function to place the appropriate CATID value in the system registry
        /// for this object.</remarks>
        /// <param name = "monitors">The CAPE-OPEN component is a flowsheet monitoring object.</param>
        public CapeFlowsheetMonitoringAttribute(bool monitors)
        {
            m_Monitors = monitors;
        }

        /// <summary>Gets the the about information.</summary>
        /// <remarks>This property indicates whether the object uses the flowsheet monitoring interfaces of the PME.</remarks>
        /// <value>A boolean value indicating whether the CAPE-OPEN component supports flowsheet monitoring.</value>
        public bool Monitors
        {
            get
            {
                return m_Monitors;
            }
        }
    };

    /// <summary>Provides information regarding whether the object consumes Thermodynamics 
    /// during registration of a CAPE-OPEN object.</summary>
    /// <remarks><para>
    /// This attribute is used during the registration of a CAPE-OPEN object with the
    /// COM registry to add the Consumes_Thermo_CATID to the object's registration key. </para>
    /// </remarks>
    /// <c>
    /// [Serializable]
    /// [System.Runtime.InteropServices.ComVisible(true)]
    /// [System.Runtime.InteropServices.Guid("C79CAFD3-493B-46dc-8585-1118A0B5B4AB")]//ICapeThermoMaterialObject_IID)
    /// [System.ComponentModel.Description("")]
    /// [CapeOpen.CapeDescriptionAttribute("An example mixer unit operation.")]
    /// [CapeOpen.CapeVersionAttribute("1.0")]
    /// [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    /// [CapeOpen.CapeConsumesThermoAttribute(true)]
    /// public class CMixerExample : public CUnitBase
    /// </c>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.AttributeUsageAttribute(System.AttributeTargets.Class)]
    public class CapeConsumesThermoAttribute : System.Attribute
    {
        private bool m_ConsumesThermo;

        /// <summary>Initializes a new instance of the CapeConsumesThermoAttribute class.</summary>
        /// <remarks>This attribute is used to indicate whether the object consumes thermodynamic
        /// models. It is also used by the COM registration function to place the appropriate CATID
        /// value in the system registry.</remarks>
        /// <param name = "consumes">A boolean value indicating whether the CAPE-OPEN component consumes thermodynamics.</param>
        public CapeConsumesThermoAttribute(bool consumes)
        {
            m_ConsumesThermo = consumes;
        }

        /// <summary>Gets information about whether the object consumes Thermodynamics models.</summary>
        /// <remarks>This property indicates whether the object consumes thermodynamics.</remarks>
        /// <value>The CAPE-OPEN component consumes thermodynamics.</value>
        public bool ConsumesThermo
        {
            get
            {
                return m_ConsumesThermo;
            }
        }
    };

    /// <summary>Provides information regarding whether the object supports Thermodynamics 
    /// version 1.0 during registration of a CAPE-OPEN object.</summary>
    /// <remarks><para>
    /// This attribute is used during the registration of a CAPE-OPEN object with the
    /// COM registry to add the SupportsThermodynamics10_CATID to the object's registration key. </para>
    /// </remarks>
    /// <c>
    /// [Serializable]
    /// [System.Runtime.InteropServices.ComVisible(true)]
    /// [System.Runtime.InteropServices.Guid("C79CAFD3-493B-46dc-8585-1118A0B5B4AB")]//ICapeThermoMaterialObject_IID)
    /// [System.ComponentModel.Description("")]
    /// [CapeOpen.CapeDescriptionAttribute("An example mixer unit operation.")]
    /// [CapeOpen.CapeVersionAttribute("1.0")]
    /// [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    /// [CapeOpen.CapeConsumesThermoAttribute(true)]
    /// [CapeOpen.CapeSupportsThermodynamics10Attribute(true)]
    /// public class CMixerExample : public CUnitBase
    /// </c>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.AttributeUsageAttribute(System.AttributeTargets.Class)]
    public class CapeSupportsThermodynamics10Attribute : System.Attribute
    {
        private bool m_Supported;

        /// <summary>Initializes a new instance of the CapeSupportsThermodynamics10Attribute class.</summary>
        /// <remarks>This attribute is used to indicate whether the object supports thermodynamic
        /// version 1.0. It is also used by the COM registration function to place the appropriate CATID
        /// value in the system registry.</remarks>
        /// <param name = "supported">The CAPE-OPEN component consumes thermodynamics.</param>
        public CapeSupportsThermodynamics10Attribute(bool supported)
        {
            m_Supported = supported;
        }

        /// <summary>Gets the the about information.</summary>
        /// <remarks>This property indicates whether the object consumes thermodynamics.</remarks>
        /// <value>The CAPE-OPEN component supports thermodynamics version 1.0.</value>
        public bool Supported
        {
            get
            {
                return m_Supported;
            }
        }
    };

    /// <summary>Provides information regarding whether the object supports Thermodynamics 
    /// version 1.0 during registration of a CAPE-OPEN object.</summary>
    /// <remarks><para>
    /// This attribute is used during the registration of a CAPE-OPEN object with the
    /// COM registry to add the SupportsThermodynamics10_CATID to the object's registration key. </para>
    /// </remarks>
    /// <c>
    /// [Serializable]
    /// [System.Runtime.InteropServices.ComVisible(true)]
    /// [System.Runtime.InteropServices.Guid("C79CAFD3-493B-46dc-8585-1118A0B5B4AB")]//ICapeThermoMaterialObject_IID)
    /// [System.ComponentModel.Description("")]
    /// [CapeOpen.CapeDescriptionAttribute("An example mixer unit operation.")]
    /// [CapeOpen.CapeVersionAttribute("1.0")]
    /// [CapeOpen.CapeVendorURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeHelpURLAttribute("http:\\www.epa.gov")]
    /// [CapeOpen.CapeAboutAttribute("US Environmental Protection Agency\nCincinnati, Ohio")]
    /// [CapeOpen.CapeConsumesThermoAttribute(true)]
    /// [CapeOpen.CapeSupportsThermodynamics11Attribute(true)]
    /// public class CMixerExample110 : public CUnitBase
    /// </c>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.AttributeUsageAttribute(System.AttributeTargets.Class)]
    public class CapeSupportsThermodynamics11Attribute : System.Attribute
    {
        private bool m_Supported;
        /// <summary>Initializes a new instance of the CapeSupportsThermodynamics10Attribute class.</summary>
        /// <remarks>This attribute is used to indicate whether the object supports thermodynamic
        /// version 1.0. It is also used by the COM registration function to place the appropriate CATID
        /// value in the system registry.</remarks>
        /// <param name = "supported">The CAPE-OPEN component consumes thermodynamics.</param>
        public CapeSupportsThermodynamics11Attribute(bool supported)
        {
            m_Supported = supported;
        }

        /// <summary>Gets the the about information.</summary>
        /// <remarks>This property indicates whether the object consumes thermodynamics.</remarks>
        /// <value>The CAPE-OPEN component supports thermodynamics version 1.0.</value>
        public bool Supported
        {
            get
            {
                return m_Supported;
            }
        }
    };

    /*	[
    odl,
    uuid(B777A1BD-0C88-11D3-822E-00C04F4F66C9),
    version(20.0),
    helpstring("IATCapeXRealParameterSpec Interface"),
    dual,
    oleautomation
    ]
    interface IATCapeXRealParameterSpec : IDispatch {
    [id(0x60040003), propget, helpstring(" Provide the Aspen Plus display units for for this parameter.")]
    HRESULT DisplayUnits([out, retval] BSTR* bsUOM);
    };

    typedef [version(1.0)]
    enum {
    ErrorSeverityTerminal = 0,
    ErrorSeveritySevere = 1,
    ErrorSeverityError = 2,
    ErrorSeverityWarning = 3
    } __MIDL___MIDL_itf_AspenCapeX_0244_0001;

    [
    odl,
    uuid(B777A1B9-0C88-11D3-822E-00C04F4F66C9),
    version(1.0),
    hidden,
    dual,
    nonextensible,
    oleautomation
    ]
    interface IATCapeXDiagnostic : IDispatch {
    [id(0x60040000), helpstring("Print a message to the history device.")]
    HRESULT SendMsgToHistory([in] BSTR message);
    [id(0x60040001), helpstring("Print a message to the terminal device.")]
    HRESULT SendMsgToTerminal([in] BSTR message);
    [id(0x60040002), helpstring("Signal a simulation error.")]
    HRESULT RaiseError(
    [in] ErrorSeverity severity, 
    [in] BSTR context, 
    [in] BSTR message);
    };
    */
};
