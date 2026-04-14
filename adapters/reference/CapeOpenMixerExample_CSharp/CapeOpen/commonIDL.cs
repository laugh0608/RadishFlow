using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

// This idl file was ported from the CAPE-OPEN common.idl file and 
// the interfaces were updated to conform to .NET format.
/* IMPORTANT NOTICE
(c) The CAPE-OPEN Laboratory Network, 2002.
All rights are reserved unless specifically stated otherwise

Visit the web site at www.colan.org

This file has been edited using the editor from Microsoft Visual Studio 6.0
This file can viewed properly with any basic editors and browsers (validation done under MS Windows and Unix)
*/

// This file was developed/modified by JEAN-PIERRE BELAUD for CO-LaN organisation - August 2003

namespace CapeOpen
{

    //typedef System.Int32		CapeLong;
    //typedef System.Int16		CapeShort;

    //typedef System.Double		CapeDouble;
    //typedef System.Single		CapeFloat;

    //typedef System.Boolean		CapeBoolean;

    //typedef System.Byte		CapeChar;
    //typedef System.String		CapeString;
    //typedef System.DateTime	CapeDate;
    //typedef System.String		CapeURL;

    //typedef System.Object		CapeVariant;
    //interface class ICapeThermoMaterialObject;
    //interface class ICapeThermoSystem;
    //interface class ICapeThermoPropertyPackage;


    // Fundamental CAPE-OPEN array data types (automation compatible)
    /*
    [
    System.Runtime.InteropServices.ComVisible(false),
    System.Runtime.InteropServices.Guid("55A66445-F84A-4f43-A34C-F1B2405B3992"),
    System.ComponentModel.Description("CAPE-OPEN Integer Array Data Type")
    ]
    public ref class CapeArrayLong : public System.Collections.Generic.List<System.Int32>
    {
    public:
    CapeArrayLong() 
    {
    }
    CapeArrayLong(array<System.Int32> newList)
    {
    for (int i = 0; i < newList->Length; i++){
    this->Add(newList[i]);
    }
    }
    };

    [
    System.Runtime.InteropServices.ComVisible(false),
    System.Runtime.InteropServices.Guid("4B65B8FB-9C4F-454f-A717-16370370052D"),
    System.ComponentModel.Description("CAPE-OPEN String Array Data Type")
    ]
    public ref class CapeArrayDouble : public System.Collections.Generic.List<System.Double>
    {
    public:
    CapeArrayDouble()
    {
    }

    CapeArrayDouble(int count)
    {
    for (int i = 0; i < count; i++)
    {
    this->Add(0);
    }
    }

    CapeArrayDouble(array<System.Double> newList)
    {
    for (int i = 0; i < newList->Length; i++){
    this->Add(newList[i]);
    }
    }
    };

    [
    System.Runtime.InteropServices.ComVisible(false),
    System.Runtime.InteropServices.Guid("D06E7E49-2E42-4878-8D71-89A48DB95EEC"),
    System.ComponentModel.Description("CAPE-OPEN String Array Data Type"),
    System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.AutoDispatch)
    ]
    public ref class CapeArrayString : public System.Collections.Generic.List<System.String>
    {
    public:
    CapeArrayString()
    {
    }

    CapeArrayString(array<System.String> newList)
    {
    for (int i = 0; i < newList->Length; i++){
    this->Add(newList[i]);
    }
    }
    };

    [
    System.Runtime.InteropServices.ComVisible(false),
    System.Runtime.InteropServices.Guid("44770A1F-1D51-498e-B7C1-DF4F27A0FC13"),
    System.ComponentModel.Description("CAPE-OPEN Object Array Data Type")
    ]
    public ref class CapeArrayBoolean : System.Collections.Generic.List<System.Boolean>
    {
    public:
    CapeArrayBoolean()
    {
    }

    CapeArrayBoolean(array<bool> newList)
    {
    for (int i = 0; i < newList->Length; i++){
    this->Add(newList[i]);
    }
    }
    };

    [
    System.Runtime.InteropServices.ComVisible(false),
    System.Runtime.InteropServices.Guid("99690C3E-98FB-41eb-A451-DA843A01D9A3"),
    System.ComponentModel.Description("CAPE-OPEN Object Array Data Type")
    ]
    public ref class CapeArrayVariant : public System.Collections.Generic.List<System.Object>
    {
    public:
    CapeArrayVariant()
    {
    }

    CapeArrayVariant(array<Object> newList)
    {
    for (int i = 0; i < newList->Length; i++){
    this->Add(newList[i]);
    }
    }
    };
    */
    /// <summary>
    /// Enumeration flag to indicate parameter validation status.
    /// </summary>
    /// <remarks>
    /// <para>The enumeration has the following meanings:</para>
    /// <para>(i)   notValidated(CAPE_NOT_VALIDATED): The PMC's Validate()
    /// method has not been called after the last time that its value had been changed.</para>
    /// <para>(ii)  invalid(CAPE_INVALID): The last time that the PMC's Validate() 
    /// method was called it returned false.</para>
    /// <para>(iii) valid(CAPE_VALID): the last time that the PMC's Validate() method 
    /// was called it returned true.</para>
    /// </remarks>
    [
        Serializable,
        System.Runtime.InteropServices.ComVisibleAttribute(true),
        System.Runtime.InteropServices.GuidAttribute(COGuids.CapeValidationStatus_IID)
    ]
    public enum CapeValidationStatus
    {
        /// <summary>
        /// The PMC's Validate() method has not been called after the last time that its value had been changed.
        /// </summary>
        CAPE_NOT_VALIDATED = 0,
        /// <summary>
        /// The last time that the PMC's Validate() method was called it returned false.
        /// </summary>
        CAPE_INVALID = 1,
        /// <summary>
        /// The last time that the PMC's Validate() method was called it returned true.
        /// </summary>
        CAPE_VALID = 2
    };
    // typedef CapeValidationStatus eCapeValidationStatus;

    /// <summary>
    /// Event thrown to indicate that the name of a component has changed.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("F79EA405-4002-4fb2-AED0-C1E48793637D")]
    [System.ComponentModel.DescriptionAttribute("CapeIdentificationEvents Interface")]
    [System.Runtime.InteropServices.InterfaceTypeAttribute(System.Runtime.InteropServices.ComInterfaceType.InterfaceIsIDispatch)]
    interface IComponentNameChangedEventArgs
    {
        /// <summary>
        /// The name of the PMC prior to the name change.</summary>
        /// <remarks>The former name of the unit can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The name of the unit prior to the name change.</value>
        String OldName
        {
            get;
        }
        /// <summary>
        /// The name of the PMC after the name change.</summary>
        /// <remarks>The new name of the unit can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The name of the unit after the name change.</value>
        String NewName
        {
            get;
        }
    };

    /// <summary>
    /// Event thrown to indicate that the description of a component has changed.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("34C43BD3-86B2-46d4-8639-E0FA5721EC5C")]
    [System.ComponentModel.DescriptionAttribute("CapeIdentificationEvents Interface")]
    [System.Runtime.InteropServices.InterfaceTypeAttribute(System.Runtime.InteropServices.ComInterfaceType.InterfaceIsIDispatch)]
    interface IComponentDescriptionChangedEventArgs
    {
        /// <summary>
        /// The description of the PMC prior to the name change.</summary>
        /// <remarks>The former description of the unit can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The description of the unit prior to the description change.</value>
        String OldDescription
        {
            get;
        }
        /// <summary>
        /// The name of the PMC after the name change.</summary>
        /// <remarks>The description name of the unit can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The description of the unit after the description change.</value>
        String NewDescription
        {
            get;
        }
    };

    /// <summary>
    /// Provides data for the CapeIdentification.ComponentNameChanged event.
    /// </summary>
    /// <remarks>
    /// A CapeIdentification.NameChangedEventArgs event specifies the old and new name of the PMC.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("D78014E7-FB1D-43ab-B807-B219FAB97E8B")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class ComponentNameChangedEventArgs : System.EventArgs//,
        //IComponentNameChangedEventArgs
    {
        private String m_oldName;
        private String m_newName;

        /// <summary>Creates an instance of the NameChangedEventArgs class with the old and new name.</summary>
        /// <remarks>You can use this constructor when raising the NameChangedEvent at run time to specify a 
        /// specific the name of the PMC having its name changed.
        /// </remarks>
        /// <param name = "oldName">The name of the PMC prior to the name change.</param>
        /// <param name = "newName">The name of the PMC after the name change.</param>
        public ComponentNameChangedEventArgs(String oldName, String newName)
            : base()
        {
            m_oldName = oldName;
            m_newName = newName;
        }

        /// <summary>
        /// The name of the PMC prior to the name change.</summary>
        /// <remarks>The former name of the unit can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The name of the unit prior to the name change.</value>
        public String OldName
        {
            get
            {
                return m_oldName;
            }
        }
        /// <summary>
        /// The name of the PMC after the name change.</summary>
        /// <remarks>The new name of the unit can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The name of the unit after the name change.</value>
        public String NewName
        {
            get
            {
                return m_newName;
            }
        }
    };

    /// <summary>
    /// Provides data for the CapeIdentification.ComponentDescriptionChanged event.
    /// </summary>
    /// <remarks>
    /// A CapeIdentification.DescriptionChangedEventArgs event specifies the old and new description of the PMC.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("0C51C4F1-20E8-413d-93E1-4704B888354A")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class ComponentDescriptionChangedEventArgs : System.EventArgs,
        IComponentDescriptionChangedEventArgs
    {
        private String m_oldDescription;
        private String m_newDescription;

        /// <summary>Creates an instance of the DescriptionChangedEventArgs class with the old and new name.</summary>
        /// <remarks>You can use this constructor when raising the DescriptionChangedEvent at run time to specify a 
        /// specific the description of the PMC having its name changed.
        /// </remarks>
        /// <param name = "oldDescription">The description of the PMC prior to the description change.</param>
        /// <param name = "newDescription">The description of the PMC after the description change.</param>
        public ComponentDescriptionChangedEventArgs(String oldDescription, String newDescription)
            : base()
        {
            m_oldDescription = oldDescription;
            m_newDescription = newDescription;
        }

        /// <summary>
        /// The description of the PMC prior to the name change.</summary>
        /// <remarks>The former description of the unit can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The description of the unit prior to the description change.</value>
        public String OldDescription
        {
            get
            {
                return m_oldDescription;
            }
        }
        /// <summary>
        /// The name of the PMC after the name change.</summary>
        /// <remarks>The description name of the unit can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The description of the unit after the description change.</value>
        public String NewDescription
        {
            get
            {
                return m_newDescription;
            }
        }
    };
    /// <summary>
    /// Provides methods to identify and describe a CAPE-OPEN component.
    /// </summary>
    /// <remarks>
    /// <para>As illustration, we remind requirements coming from the existing interface 
    /// specification and being connected with the Identification concept:</para>
    /// <para>The Unit Operations Interfaces have the following requirements:</para>
    /// <para>* If a flowsheet contains two instances of a Unit Operation of a particular 
    /// class, the COSE needs to provide the user a textual identifier to distinguish each 
    /// of the instances. For instance, when the COSE requires to report about an error 
    /// occurred in one of the Unit Operations.</para>
    /// <para>* When the COSE shows the user its GUI to connect the COSE’s streams to the 
    /// Unit Operation ports, the COSE needs to request the Unit for its list of available 
    /// ports. For the user to identify the ports, the user needs some distinctive textual 
    /// information for each of them.</para>
    /// <para>* When the COSE exposes to the user its interfaces to browse or set the 
    /// value of an internal parameter of a Unit Operation, the COSE needs to request the 
    /// Unit for its list of available parameters. No matter if this COSE’s interface is 
    /// a GUI or a programming interface, each parameter must be identified by a textual 
    /// string.</para>
    /// <para>The ICapeThermoMaterialObject (used by both Unit and Thermo interfaces):</para>
    /// <para>* If a Unit Operation has encountered an error accessing a stream 
    /// (<see cref ="ICapeThermoMaterialObject">ICapeThermoMaterialObject</see>), the 
    /// Unit might decide to report it to the user. It would be desirable the stream to 
    /// have a textual identifier for the user to be able to quickly know which stream 
    /// failed.</para>
    /// <para>The Thermodynamic Interfaces have the following requirements:</para>
    /// <para>* The <see cref ="ICapeThermoSystem">ICapeThermoSystem</see>
    /// and the <see cref ="ICapeThermoPropertyPackage">ICapeThermoPropertyPackage</see> 
    /// interfaces don’t require an identification interface, since both of them have been 
    /// designed as singletons (a single instance of each component class is required). 
    /// That means that there is no need to identify this instance: its class description 
    /// would be enough. However, the user might decide anyway to assign a name or a 
    /// description to the CAPE-OPEN property systems or property packages used in her/his 
    /// flowsheet. Furthermore, if these interfaces evolve, the singleton approach could 
    /// be removed. In this case, identifying each instance will be a must.</para>
    /// <para>The Solvers Interfaces have the following requirements:</para>
    /// <para>* Many objects should provide the functionality coming from the 
    /// Identification Common Interface.</para>
    /// <para>The SMST Interfaces have the following requirements:</para>
    /// <para>* The CO SMST component package depends on the Identification Interface 
    /// package. The interface ICapeSMSTFactory must provide the Identification 
    /// capabilities.</para>
    /// <para>Reference document: Identification Common Interface</para>
    /// </remarks>
    [System.Runtime.InteropServices.InterfaceType(System.Runtime.InteropServices.ComInterfaceType.InterfaceIsIDispatch)]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("5F5087A7-B27B-4b4f-902D-5F66E34A0CBE")]
    [System.ComponentModel.DescriptionAttribute("CapeIdentificationEvents Interface")]
    interface ICapeIdentificationEvents
    {
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
        /// <param name = "sender">The PMC that raised the event.</param>
        /// <param name = "args">A <see cref = "ComponentNameChangedEventArgs ">ParameterDefaultValueChanged</see> that contains information about the event.</param>
        void ComponentNameChanged(
            [System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]Object sender,
            [System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]Object args
            );

        /// <summary>
        /// Gets and sets the description of the component.
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
        /// <param name = "sender">The PMC that raised the event.</param>
        /// <param name = "args">A <see cref = "ComponentDescriptionChangedEventArgs ">ParameterDefaultValueChanged</see> that contains information about the event.</param>
        void ComponentDescriptionChanged(
            [System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]Object sender,
            [System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]Object args
            );
    };
    /// <summary>
    /// Provides methods to identify and describe a CAPE-OPEN component.
    /// </summary>
    /// <remarks>
    /// <para>As illustration, we remind requirements coming from the existing interface 
    /// specification and being connected with the Identification concept:</para>
    /// <para>The Unit Operations Interfaces have the following requirements:</para>
    /// <para>* If a flowsheet contains two instances of a Unit Operation of a particular 
    /// class, the COSE needs to provide the user a textual identifier to distinguish each 
    /// of the instances. For instance, when the COSE requires to report about an error 
    /// occurred in one of the Unit Operations.</para>
    /// <para>* When the COSE shows the user its GUI to connect the COSE’s streams to the 
    /// Unit Operation ports, the COSE needs to request the Unit for its list of available 
    /// ports. For the user to identify the ports, the user needs some distinctive textual 
    /// information for each of them.</para>
    /// <para>* When the COSE exposes to the user its interfaces to browse or set the 
    /// value of an internal parameter of a Unit Operation, the COSE needs to request the 
    /// Unit for its list of available parameters. No matter if this COSE’s interface is 
    /// a GUI or a programming interface, each parameter must be identified by a textual 
    /// string.</para>
    /// <para>The ICapeThermoMaterialObject (used by both Unit and Thermo interfaces):</para>
    /// <para>* If a Unit Operation has encountered an error accessing a stream 
    /// (<see cref ="ICapeThermoMaterialObject">ICapeThermoMaterialObject</see>), the 
    /// Unit might decide to report it to the user. It would be desirable the stream to 
    /// have a textual identifier for the user to be able to quickly know which stream 
    /// failed.</para>
    /// <para>The Thermodynamic Interfaces have the following requirements:</para>
    /// <para>* The <see cref ="ICapeThermoSystem">ICapeThermoSystem</see>
    /// and the <see cref ="ICapeThermoPropertyPackage">ICapeThermoPropertyPackage</see> 
    /// interfaces don’t require an identification interface, since both of them have been 
    /// designed as singletons (a single instance of each component class is required). 
    /// That means that there is no need to identify this instance: its class description 
    /// would be enough. However, the user might decide anyway to assign a name or a 
    /// description to the CAPE-OPEN property systems or property packages used in her/his 
    /// flowsheet. Furthermore, if these interfaces evolve, the singleton approach could 
    /// be removed. In this case, identifying each instance will be a must.</para>
    /// <para>The Solvers Interfaces have the following requirements:</para>
    /// <para>* Many objects should provide the functionality coming from the 
    /// Identification Common Interface.</para>
    /// <para>The SMST Interfaces have the following requirements:</para>
    /// <para>* The CO SMST component package depends on the Identification Interface 
    /// package. The interface ICapeSMSTFactory must provide the Identification 
    /// capabilities.</para>
    /// <para>Reference document: Identification Common Interface</para>
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute(COGuids.CapeIdentification_IID)]
    [System.ComponentModel.DescriptionAttribute("CapeIdentification Interface")]
    public interface ICapeIdentification
    {
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
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("property ComponentName")]
        String ComponentName
        {
            get;
            set;
        }

        /// <summary>
        /// Gets and sets the description of the component.
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
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("property ComponentName")]
        String ComponentDescription
        {
            get;
            set;
        }
    };

    /// <summary>
    /// Represents the method that will handle the changing the name of a component.
    /// </summary>
    /// <remarks>
    /// When you create a ComponentNameChangedHandler delegate, you identify the method that will handle the event. To associate the event with your event handler, add an 
    /// instance of the delegate to the event. The event handler is called whenever the event occurs, unless you remove the delegate. For more information about delegates, 
    /// see Events and Delegates.
    /// </remarks>
    /// <param name = "sender">The PMC that is the source .</param>
    /// <param name = "args">A <see cref = "ComponentNameChangedEventArgs">NameChangedEventArgs</see> that provides information about the name change.</param>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    public delegate void ComponentNameChangedHandler(Object sender, ComponentNameChangedEventArgs args);

    /// <summary>
    /// Represents the method that will handle the changing of the description of a component.
    /// </summary>
    /// <remarks>
    /// When you create a ComponentNameChangedHandler delegate, you identify the method that will handle the event. To 
    /// associate the event with your event handler, add an instance of the delegate to the event. The event handler is 
    /// called whenever the event occurs, unless you remove the delegate. For more information about delegates, see Events 
    /// and Delegates.
    /// </remarks>
    /// <param name = "sender">The PMC that is the source of the event.</param>
    /// <param name = "args">A <see cref = "ComponentDescriptionChangedEventArgs">DescriptionChangedEventArgs</see> that 
    /// provides information about the description change.</param>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    public delegate void ComponentDescriptionChangedHandler(Object sender, ComponentDescriptionChangedEventArgs args);

    /// <summary>
    /// Provides methods to identify and describe a CAPE-OPEN component.
    /// </summary>
    /// <remarks>
    /// <para>Allows the user to assign different names and descriptions to each 
    /// instance of a PMC in order to refer to them unambiguously and in a 
    /// user-friendly way. Since not always the software components that are able to 
    /// set these identifications and the software components that require this 
    /// information have been developed by the same vendor, a CAPE-OPEN standard for 
    /// setting and getting this information is required.</para>
    /// <para>Reference document: Identification Common Interface</para>
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(ICapeIdentificationEvents), typeof(System.ComponentModel.INotifyPropertyChanged))]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("BF54DF05-924C-49a5-8EBB-733E37C38085")]
    [System.ComponentModel.DescriptionAttribute("CapeIdentification Interface")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    abstract public class CapeIdentification : //System.ComponentModel.Component,
        ICapeIdentification,
        IDisposable,
        ICloneable,
        System.ComponentModel.INotifyPropertyChanged
        
    {
        /// <summary>
        /// The name of the component.
        /// </summary>
        private String m_ComponentName;
        /// <summary>
        /// The description of the component.
        /// </summary>
        private String m_ComponentDescription;        
        // Track whether Dispose has been called.
        private bool _disposed;

        /// <summary>
        /// Creates an instance of the CapeIdentification class with default values for the name and 
        /// description of the PMC.
        /// </summary>
        /// <remarks>
        /// This constructor uses the <see cref="System.Type"/> of the PMC object being constructed as default 
        /// values for the ComponentName and ComponentDescription properties. If the PMC object has a 
        /// <see cref="CapeNameAttribute"/>, then the <see cref="CapeNameAttribute.Name"/> property is used for the name.
        /// Likewise, if the object has a <see cref="CapeDescriptionAttribute"/>, then the 
        /// <see cref="CapeDescriptionAttribute.Description"/> property is used for the description.
        /// </remarks>
        public CapeIdentification()
        {
            _disposed = false;
            m_ComponentName = this.GetType().FullName;
            m_ComponentDescription = this.GetType().FullName;
            Object[] attributes = this.GetType().GetCustomAttributes(false);
            for (int i = 0; i < attributes.Length; i++)
            {
                if (attributes[i] is CapeNameAttribute) m_ComponentName = ((CapeNameAttribute)attributes[i]).Name;
                if (attributes[i] is CapeDescriptionAttribute) m_ComponentDescription = ((CapeDescriptionAttribute)attributes[i]).Description;
            }
        }

        /// <summary>
        /// Creates an instance of the CapeIdentification class with the name and a default description of the PMC.
        /// </summary>
        /// <remarks>
        /// This constructor uses the provided name for the ComponentName of the PMC object being constructed. A 
        /// default value for the ComponentDescription properties is then assigned. If the PMC object has a 
        /// <see cref="CapeDescriptionAttribute"/>, then the <see cref="CapeDescriptionAttribute.Description"/> 
        /// property is used for the description.
        /// </remarks>
        /// <param name = "name">The name of the PMC.</param>
        public CapeIdentification(String name)
        {
            _disposed = false;
            m_ComponentName = name;
            m_ComponentDescription = this.GetType().FullName;
            Object[] attributes = this.GetType().GetCustomAttributes(false);
            for (int i = 0; i < attributes.Length; i++)
            {
                if (attributes[i] is CapeDescriptionAttribute) m_ComponentDescription = ((CapeDescriptionAttribute)attributes[i]).Description;
            }
        }

        /// <summary
        /// >Creates an instance of the CapeIdentification class with the name and description of the PMC.
        /// </summary>
        /// <remarks>
        /// You can use this constructor to specify a specific name and description of the PMC.
        /// </remarks>
        /// <param name = "name">The name of the PMC.</param>
        /// <param name = "description">The description of the PMC.</param>
        public CapeIdentification(String name, String description)
        {
            _disposed = false;
            m_ComponentName = name;
            m_ComponentDescription = description;
        }


        /// <summary>
        /// Copy constructor of the CapeIdentification class.
        /// </summary>
        /// <remarks>
        /// Creates an instance of the CapeIdentification class with ComponentName equal to the original PMC's 
        /// ComponentName + (Copy). The copy has the same CapeDescription as the original.
        /// </remarks>
        /// <param name = "objectToBeCopied">The object being copied.</param>
        protected CapeIdentification(CapeIdentification objectToBeCopied)
        {
            _disposed = false;
            m_ComponentName = objectToBeCopied.ComponentName + "(Copy)";
            m_ComponentDescription = objectToBeCopied.ComponentDescription;
        }
        
        /// <summary>
        /// Creates a new object that is a copy of the current instance.
        /// </summary>
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
        public abstract object Clone();

        // Implement IDisposable.
        // Do not make this method virtual.
        // A derived class should not be able to override this method.
        /// <summary>
        /// Releases all resources used by the CapeIdentification object.
        /// </summary>
        /// <remarks>Call Dispose when you are finished using the CapeIdentification object. The Dispose method 
        /// leaves the CapeIdentification object in an unusable state. After calling Dispose, you must release 
        /// all references to the Component so the garbage collector can reclaim the memory that the CapeIdentification 
        /// object was occupying. For more information, see <see href="http://msdn.microsoft.com/en-us/library/498928w2.aspx">
        /// Cleaning Up Unmanaged Resources and Implementing a Dispose Method.</see></remarks> 
        public void Dispose()
        {
            this.Dispose(true);
            // This object will be cleaned up by the Dispose method.
            // Therefore, you should call GC.SupressFinalize to
            // take this object off the finalization queue
            // and prevent finalization code for this object
            // from executing a second time.
            GC.SuppressFinalize(this);
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
        protected virtual void Dispose(bool disposing)
        {
            // Check to see if Dispose has already been called.
            if (!_disposed)
            {
                 //If disposing equals true, dispose all managed
                // and unmanaged resources.
                if (disposing)
                {
                    // Dispose managed resources.
                    //component.Dispose();
                }
                // Note disposing has been done.
                _disposed = true;
            }
        }

        /// <summary>
        /// Notifies the collection that the value of a property of the parameter has been changed.
        /// </summary>
        /// <remarks>The PropertyChanged event can indicate all properties on the object have changed by using either 
        /// null or String.Empty as the property name in the PropertyChangedEventArgs.
        /// </remarks>
        public event System.ComponentModel.PropertyChangedEventHandler PropertyChanged;

        // This method is called by the Set accessor of each property. 
        // The CallerMemberName attribute that is applied to the optional propertyName 
        // parameter causes the property name of the caller to be substituted as an argument. 
        /// <summary>
        /// Notifies the collection that the value of a proparty of the parameter has been changed.
        /// </summary>
        /// <remarks>The PropertyChanged event can indicate all properties on the object have changed by using either 
        /// null or String.Empty as the property name in the PropertyChangedEventArgs.
        /// </remarks>
        /// <param name="propertyName">The name of the property that was chnaged.</param>
        protected void NotifyPropertyChanged(/* .Net 4.5 [System.Runtime.CompilerServices.CallerMemberName]*/ String propertyName)
        {
            if (PropertyChanged != null)
            {
                PropertyChanged(this, new System.ComponentModel.PropertyChangedEventArgs(propertyName));
            }
        }

        /// <summary>
        /// Occurs when the user changes of the name of a component.
        /// </summary>
        /// <remarks>The event to be handles when the name of the PMC is changed.</remarks> 
        public event ComponentNameChangedHandler ComponentNameChanged;

        /// <summary>
        /// Occurs when the user changes of the description of a component.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnComponentNameChanged</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnComponentNameChanged</c> in a derived class, be sure to call the base class's <c>OnComponentNameChanged</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "args">A <see cref = "ComponentNameChangedEventArgs">NameChangedEventArgs</see> that contains information about the event.</param>
        protected void OnComponentNameChanged(ComponentNameChangedEventArgs args)
        {
            if (ComponentNameChanged != null)
            {
                ComponentNameChanged(this, args);
            }
        }

        /// <summary>
        /// Occurs when the user changes of the description of a component.
        /// </summary>
        /// <remarks>The event to be handles when the description of the PMC is changed.</remarks> 
        public event ComponentDescriptionChangedHandler ComponentDescriptionChanged;

        /// <summary>
        /// Occurs when the user changes of the description of a component.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnComponentDescriptionChanged</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnComponentDescriptionChanged</c> in a derived class, be sure to call the base class's <c>OnComponentDescriptionChanged</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "args">A <see cref = "ComponentDescriptionChangedEventArgs">DescriptionChangedEventArgs</see> that contains information about the event.</param>
        protected void OnComponentDescriptionChanged(ComponentDescriptionChangedEventArgs args)
        {
            if (ComponentDescriptionChanged != null)
            {
                ComponentDescriptionChanged(this, args);
            }
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
        public virtual String ComponentName
        {
            get
            {
                return m_ComponentName;
            }

            set
            {
                ComponentNameChangedEventArgs args = new ComponentNameChangedEventArgs(m_ComponentName, value);
                m_ComponentName = value;
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
        public virtual String ComponentDescription
        {
            get
            {
                return m_ComponentDescription;
            }
            set
            {
                ComponentDescriptionChangedEventArgs args = new ComponentDescriptionChangedEventArgs(m_ComponentDescription, value);
                m_ComponentDescription = value;
                NotifyPropertyChanged("ComponentDescription");
                OnComponentDescriptionChanged(args);
            }
        }
    };


    /// <summary>
    /// This interface provides the behaviour for a read-only collection. It can be 
    /// used for storing ports or parameters.
    /// </summary>
    /// <remarks>
    /// <para>The aim of the Collection interface is to give a CAPE-OPEN component 
    /// the possibility to expose a list of objects to any client of the component. 
    /// The client will not be able to modify the collection, i.e. removing, 
    /// replacing or adding elements. However, since the client will have access to 
    /// any CAPE-OPEN interface exposed by the items of the collection, it will be 
    /// able to modify the state of any element.</para>
    /// <para>CAPE-OPEN Collections don’t allow exposing basic types such as 
    /// numerical values or strings. Indeed, using CapeArrays is more convenient 
    /// here.</para>
    /// <para>Not all the items of a collection must belong to the same class. It is 
    /// enough if they implement the same interface or set of interfaces. A CAPE-OPEN 
    /// specification a component that exposes a collection interface must state 
    /// clearly which interfaces must be implemented by all the items of the 
    /// collection.</para>
    /// <para>Reference document: Collection Common Interface</para>
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.GuidAttribute(COGuids.ICapeCollection_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeCollection Interface")]
    interface ICapeCollection
    {
        /// <summary>
        ///	Gets the specific item stored within the collection, identified by its 
        /// ICapeIdentification.ComponentName or 1-based index passed as an argument 
        /// to the method.
        /// </summary>
        /// <remarks>
        /// Return an element from the collection. The requested element can be 
        /// identified by its actual name (e.g. type CapeString) or by its position 
        /// in the collection (e.g. type CapeLong). The name of an element is the 
        /// value returned by the ComponentName() method of its ICapeIdentification 
        /// interface. The advantage of retrieving an item by name rather than by 
        /// position is that it is much more efficient. This is because it is faster 
        /// to check all names from the server part than checking then from the 
        /// client, where a lot of COM/CORBA calls would be required.
        /// </remarks>
        /// <param name = "index">
        /// <para>Identifier for the requested item:</para>
        /// <para>name of item (the variant contains a string)</para>
        /// <para>position in collection (it contains a long)</para>
        /// </param>
        /// <returns>
        /// System.Object containing the requested collection item.
        /// </returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeOutOfBounds">ECapeOutOfBounds</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("gets an item specified by index or name")]
        [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]// This attribute specifices the return value is an IDispatch pointer.
        Object Item(Object index);

        /// <summary>
        ///	Gets the number of items currently stored in the collection.
        /// </summary>
        /// <remarks>Return the number of items in the collection.</remarks>
        /// <returns>Return the number of items in the collection.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("Number of items in the collection")]
        int Count();
    };
    /*   
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
       [System.Runtime.InteropServices.ComVisibleAttribute(true)]
       public delegate void CollectionAddingNewHandler(Object sender, System.ComponentModel.AddingNewEventArgs args);


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
       [System.Runtime.InteropServices.ComVisibleAttribute(true)]
       public delegate void CollectionListChangedHandler(Object sender, System.ComponentModel.ListChangedEventArgs args);

       /// <summary>
       /// </summary>
       /// <remarks>
       /// </remarks>
       [System.Runtime.InteropServices.InterfaceType(System.Runtime.InteropServices.ComInterfaceType.InterfaceIsIDispatch)]
       [System.Runtime.InteropServices.ComVisibleAttribute(true)]
       [System.Runtime.InteropServices.GuidAttribute("DE9CDE6E-A2D4-4BFF-AA3A-8699FCF3E0EB")]
       [System.ComponentModel.DescriptionAttribute("CapeCollectionEvents Interface")]
       interface ICapeCollectionEvents
       {
           /// <summary>
           /// Occurs when the user changes of the value of a paramter.
           /// </summary>
           /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
           /// <para>The <c>OnComponentNameChanged</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
           /// technique for handling the event in a derived class.</para>
           /// <para>Notes to Inheritors: </para>
           /// <para>When overriding <c>OnParameterValueChanged</c> in a derived class, be sure to call the base class's <c>OnParameterValueChanged</c> method so that registered 
           /// delegates receive the event.</para>
           /// </remarks>
           /// <param name = "sender">The <see cref = "RealParameter">RealParameter</see> that raised the event.</param>
           /// <param name = "args">A <see cref = "CollectionAddingNew">CollectionAddingNew</see> that contains information about the event.</param>
           void CollectionAddingNew(object sender, object args);

           /// <summary>
           /// Occurs when the user changes of the mode of a parameter.
           /// </summary>
           /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
           /// <para>The <c>OnParameterModeChanged</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
           /// technique for handling the event in a derived class.</para>
           /// <para>Notes to Inheritors: </para>
           /// <para>When overriding <c>OnParameterModeChanged</c> in a derived class, be sure to call the base class's <c>OnParameterModeChanged</c> method so that registered 
           /// delegates receive the event.</para>
           /// </remarks>
           /// <param name = "sender">The <see cref = "RealParameter">RealParameter</see> that raised the event.</param>
           /// <param name = "args">A <see cref = "CollectionListChanged">CollectionListChanged</see> that contains information about the event.</param>
           void CollectionListChanged(object sender, object args);
       }
   */
    /// <summary>
    ///	Interface that exposes a PMC's parameters, controls the PMC's lifecycle, 
    /// provides access to the PME through the simulation context, and provides a 
    /// means for the PME to edit the PMC.
    /// </summary>
    /// <remarks>
    /// <para>When a PME requires some kind of functionality, with the help of the 
    /// CAPE-OPEN categories, the user is able to select and create a CO class which 
    /// will expose the required CO interfaces. There is the need for the PME to 
    /// exchange some information with this instance of the PMC. This information 
    /// consists in a set of simple unrelated functionalities that will be useful for 
    /// any kind of CAPE-OPEN component, since they will allow maximum integration 
    /// between clients and servers. All these functionalities can be grouped in a 
    /// single interface. Some of the functionalities to fulfil consist in exchanging 
    /// interface references between the PMC and the PME. Instead of adding these 
    /// properties to each business interfaces, it is much more convenient to
    /// add them to a single common interface which refers to the whole PMC.</para>
    /// <para>Furthermore, there is a need for getting parameters, editing and 
    /// lifecycling.</para>
    /// <para>The interface should fulfil the following requirements:</para>
    /// <para>Parameters:</para>
    /// <para>So far, only Unit Operations can expose their public parameters, through 
    /// property ICapeUnit.parameters, which returns a collection of parameters. This 
    /// property allows COSEs to support design specs between two CAPE-OPEN Unit 
    /// Operations. That means that the CAPE-OPEN interfaces are powerful enough to 
    /// allow that the design spec of a given Unit Operation (exposed through public 
    /// parameters) depends on transformations of public parameters exposed by other 
    /// CAPE-OPEN Unit Operations. If also other components, such as Material Object, 
    /// would be able to expose public parameters, the functionality the aforementioned 
    /// described functionality could be extended. Other functionalities would be:
    /// </para>
    /// <para>(i) allowing optimizers to use a public variable exposed by any 
    /// CAPE-OPEN component.</para>
    /// <para>(ii) Allow performing regression on the interaction parameters of a 
    /// CAPE-OPEN Property Package.</para>
    /// <para>Centralising the property that accesses these collections in a single 
    /// entry point, helps to clarify the life cycle usage standards for these 
    /// collections. That means that it will be easier for the PMC clients to know 
    /// how often they have to check whether the contents of these collections have 
    /// changed (although the collection object will be valid until the PMC is 
    /// destroyed). Setting general rules for the usage of these collections makes 
    /// the business interface specifications more regular and simpler. Obviously, 
    /// since too general rules may reduce flexibility, PMC specifications might point
    /// out exceptions to the general rule. Let’s see how would this affect the 
    /// particular PMC specifications:</para>
    /// <para>Simulation context:</para>
    /// <para>So far, most of CAPE-OPEN interfaces have been designed to allow a 
    /// client to access the functionality of a CAPE-OPEN component. Since clients 
    /// will often be Simulation Environment, CAPE-OPEN components would benefit from 
    /// using functionality provided by their client, a COSE for instance. These 
    /// services provided by any PMEs are defined within the Simulation Context COSE 
    /// Interface specification document.</para>
    /// <para>The following interfaces are designed:</para>
    /// <para>(i) Thermo Material Template Systems: Theses interface allows a PMC to 
    /// choose between all the Thermo Material factories supported by the PME. These 
    /// factories will allow the PMC to create a thermo material object associated to 
    /// the elected Property Package (which can be CAPE-OPEN or not).</para>
    /// <para>(ii) Diagnostics: This interface will allow to integrate seamlessly the 
    /// diagnostics messages generated by any PMC with the mechanisms supported by 
    /// the PME to display this information to the user.</para>
    /// <para>(iii) COSEUtilities: In the same idea of this specification document, 
    /// PME has also its own utilities interface in order to gather many basic 
    /// operations. For instance that allows the PME to supply a list of standardised 
    /// values.</para>
    /// <para>Edit:</para>
    /// <para>The Edit method defined by the UNIT specification proved to be very useful in 
    /// order to provide Graphical User Interface (GUI) capabilities highly customized 
    /// to each type of UNIT implementation. There’s no reason why other PMCs could 
    /// not benefit of this capability. Obviously, when a PMC provides Edit 
    /// functionality, being able to persist its state is a desired requirements, to 
    /// prevent the user from having to repeatedly reconfigure the PMC.</para>
    /// <para>LifeCycle:</para>
    /// <para>There is probably no strict necessity to expose directly initialization 
    /// nor destruction functions, since these should be invoked automatically by the 
    /// used middleware (COM/CORBA). That is, the initialization could be performed 
    /// in the constructor of the class and the destroy in its destructor. However, 
    /// in some cases the client could need to invoke them explicitly. For example, 
    /// all actions that could fail should be invoked by these methods. If these 
    /// actions were places in the constructor or destructor, potential failures 
    /// would cause memory leak, and they would be difficult to track, since it would 
    /// not be clear if the component has been created/destroyed or not. Examples of 
    /// cases where they are useful:</para>
    /// <para>(i) Initialize: The client might need to initialize a given set of PMC 
    /// in a specific order, in case that there are dependencies between them. Some 
    /// PMC may be wrappers to other components, or may need an external file to get 
    /// initialized. This initialization process may often fail, or the user may even 
    /// decide to cancel it. Moving these actions from the class constructor to the 
    /// initialize method allows communicating the client that the construction of
    /// the component must be aborted in some cases.</para>
    /// <para>(ii) Destructors: The PMC primary object should destroy here all its secondary 
    /// objects. Relying on the native destructor could cause deadlocks when loop 
    /// references exist between PMC objects. See in the example diagram below that 
    /// after the client releases its reference to the Unit Operation, both the Unit 
    /// and the Parameter are being used by another objects. So, without an explicit 
    /// terminate method, none of them would be ever terminated.</para>
    /// <para>Reference document: Utilities Common Interface</para>
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeUtilities_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeUtilities Interface")]
    interface ICapeUtilitiesCOM
    {
        /// <summary>
        ///	Gets the component's collection of parameters. 
        /// </summary>
        /// <remarks>
        /// <para>Return the collection of Public Unit Parameters (i.e. 
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
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("Gets parameter collection")]
        //[return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
        Object parameters
        {
            [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
            get;
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
        /// PMC, ICapeUnit.SetSimulationContext is deprecated.</para>
        /// </remarks>
        /// <value>
        /// The reference to the PME’s simulation context class. For the PMC to use 
        /// this class, this reference will have to be converted to each of the 
        /// defined CO Simulation Context interfaces.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("Set the simulation context")]
        Object simulationContext
        {
            [param: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
            set;
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
        void Initialize();

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
        void Terminate();

        /// <summary>
        ///	Displays the PMC graphic interface, if available.
        /// </summary>
        /// <remarks>
        /// The PMC displays its user interface and allows the Flowsheet User to 
        /// interact with it. If no user interface is available it returns an error.
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(5), System.ComponentModel.DescriptionAttribute("Displays the graphic interface")]
        [System.Runtime.InteropServices.PreserveSig]
        int Edit();
    };


    /// <summary>
    ///	Interface that exposes a PMC's parameters, controls the PMC's lifecycle, 
    /// provides access to the PME through the simulation context, and provides a 
    /// means for the PME to edit the PMC.
    /// </summary>
    /// <remarks>
    /// <para>When a PME requires some kind of functionality, with the help of the 
    /// CAPE-OPEN categories, the user is able to select and create a CO class which 
    /// will expose the required CO interfaces. There is the need for the PME to 
    /// exchange some information with this instance of the PMC. This information 
    /// consists in a set of simple unrelated functionalities that will be useful for 
    /// any kind of CAPE-OPEN component, since they will allow maximum integration 
    /// between clients and servers. All these functionalities can be grouped in a 
    /// single interface. Some of the functionalities to fulfil consist in exchanging 
    /// interface references between the PMC and the PME. Instead of adding these 
    /// properties to each business interfaces, it is much more convenient to
    /// add them to a single common interface which refers to the whole PMC.</para>
    /// <para>Furthermore, there is a need for getting parameters, editing and 
    /// lifecycling.</para>
    /// <para>The interface should fulfil the following requirements:</para>
    /// <para>Parameters:</para>
    /// <para>So far, only Unit Operations can expose their public parameters, through 
    /// property ICapeUnit.parameters, which returns a collection of parameters. This 
    /// property allows COSEs to support design specs between two CAPE-OPEN Unit 
    /// Operations. That means that the CAPE-OPEN interfaces are powerful enough to 
    /// allow that the design spec of a given Unit Operation (exposed through public 
    /// parameters) depends on transformations of public parameters exposed by other 
    /// CAPE-OPEN Unit Operations. If also other components, such as Material Object, 
    /// would be able to expose public parameters, the functionality the aforementioned 
    /// described functionality could be extended. Other functionalities would be:
    /// </para>
    /// <para>(i) allowing optimizers to use a public variable exposed by any 
    /// CAPE-OPEN component.</para>
    /// <para>(ii) Allow performing regression on the interaction parameters of a 
    /// CAPE-OPEN Property Package.</para>
    /// <para>Centralising the property that accesses these collections in a single 
    /// entry point, helps to clarify the life cycle usage standards for these 
    /// collections. That means that it will be easier for the PMC clients to know 
    /// how often they have to check whether the contents of these collections have 
    /// changed (although the collection object will be valid until the PMC is 
    /// destroyed). Setting general rules for the usage of these collections makes 
    /// the business interface specifications more regular and simpler. Obviously, 
    /// since too general rules may reduce flexibility, PMC specifications might point
    /// out exceptions to the general rule. Let’s see how would this affect the 
    /// particular PMC specifications:</para>
    /// <para>Simulation context:</para>
    /// <para>So far, most of CAPE-OPEN interfaces have been designed to allow a 
    /// client to access the functionality of a CAPE-OPEN component. Since clients 
    /// will often be Simulation Environment, CAPE-OPEN components would benefit from 
    /// using functionality provided by their client, a COSE for instance. These 
    /// services provided by any PMEs are defined within the Simulation Context COSE 
    /// Interface specification document.</para>
    /// <para>The following interfaces are designed:</para>
    /// <para>(i) Thermo Material Template Systems: Theses interface allows a PMC to 
    /// choose between all the Thermo Material factories supported by the PME. These 
    /// factories will allow the PMC to create a thermo material object associated to 
    /// the elected Property Package (which can be CAPE-OPEN or not).</para>
    /// <para>(ii) Diagnostics: This interface will allow to integrate seamlessly the 
    /// diagnostics messages generated by any PMC with the mechanisms supported by 
    /// the PME to display this information to the user.</para>
    /// <para>(iii) COSEUtilities: In the same idea of this specification document, 
    /// PME has also its own utilities interface in order to gather many basic 
    /// operations. For instance that allows the PME to supply a list of standardised 
    /// values.</para>
    /// <para>Edit:</para>
    /// <para>The Edit method defined by the UNIT specification proved to be very useful in 
    /// order to provide Graphical User Interface (GUI) capabilities highly customized 
    /// to each type of UNIT implementation. There’s no reason why other PMCs could 
    /// not benefit of this capability. Obviously, when a PMC provides Edit 
    /// functionality, being able to persist its state is a desired requirements, to 
    /// prevent the user from having to repeatedly reconfigure the PMC.</para>
    /// <para>LifeCycle:</para>
    /// <para>There is probably no strict necessity to expose directly initialization 
    /// nor destruction functions, since these should be invoked automatically by the 
    /// used middleware (COM/CORBA). That is, the initialization could be performed 
    /// in the constructor of the class and the destroy in its destructor. However, 
    /// in some cases the client could need to invoke them explicitly. For example, 
    /// all actions that could fail should be invoked by these methods. If these 
    /// actions were places in the constructor or destructor, potential failures 
    /// would cause memory leak, and they would be difficult to track, since it would 
    /// not be clear if the component has been created/destroyed or not. Examples of 
    /// cases where they are useful:</para>
    /// <para>(i) Initialize: The client might need to initialize a given set of PMC 
    /// in a specific order, in case that there are dependencies between them. Some 
    /// PMC may be wrappers to other components, or may need an external file to get 
    /// initialized. This initialization process may often fail, or the user may even 
    /// decide to cancel it. Moving these actions from the class constructor to the 
    /// initialize method allows communicating the client that the construction of
    /// the component must be aborted in some cases.</para>
    /// <para>(ii) Destructors: The PMC primary object should destroy here all its secondary 
    /// objects. Relying on the native destructor could cause deadlocks when loop 
    /// references exist between PMC objects. See in the example diagram below that 
    /// after the client releases its reference to the Unit Operation, both the Unit 
    /// and the Parameter are being used by another objects. So, without an explicit 
    /// terminate method, none of them would be ever terminated.</para>
    /// <para>Reference document: Utilities Common Interface</para>
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.ComponentModel.DescriptionAttribute("ICapeUtilities Interface")]
    public interface ICapeUtilities
    {
        /// <summary>
        ///	Gets the component's collection of parameters. 
        /// </summary>
        /// <remarks>
        /// <para>Return the collection of Public Unit Parameters (i.e. 
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
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("Gets parameter collection")]
        ParameterCollection Parameters
        {
            get;
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
        /// PMC, ICapeUnit.SetSimulationContext is deprecated.</para>
        /// </remarks>
        /// <value>
        /// The reference to the PME’s simulation context class. For the PMC to use 
        /// this class, this reference will have to be converted to each of the 
        /// defined CO Simulation Context interfaces.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("Set the simulation context")]
        ICapeSimulationContext SimulationContext
        {
            get;
            set;
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
        void Initialize();

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
        void Terminate();

        /// <summary>
        ///	Displays the PMC graphic interface, if available.
        /// </summary>
        /// <remarks>
        /// The PMC displays its user interface and allows the Flowsheet User to 
        /// interact with it. If no user interface is available it returns an error.
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(5), System.ComponentModel.DescriptionAttribute("Displays the graphic interface")]
        System.Windows.Forms.DialogResult Edit();
    };

    /// <summary>
    /// Represents the method that will handle the changing of the simualtion context of a PMC.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void SimulationContextChangedHandler(Object sender, System.EventArgs args);
}

