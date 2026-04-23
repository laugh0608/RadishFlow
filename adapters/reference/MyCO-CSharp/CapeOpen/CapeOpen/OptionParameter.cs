using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{

    class OptionParameterValueConverter : System.ComponentModel.StringConverter
    {

        public override bool GetStandardValuesSupported(System.ComponentModel.ITypeDescriptorContext context)
        {
            return true;
        }

        public override System.ComponentModel.TypeConverter.StandardValuesCollection GetStandardValues(System.ComponentModel.ITypeDescriptorContext context)
        {
            OptionParameter param = (OptionParameter)(context.Instance);
            return new System.ComponentModel.TypeConverter.StandardValuesCollection(param.OptionList);
        }

        public override bool GetStandardValuesExclusive(System.ComponentModel.ITypeDescriptorContext context)
        {
            OptionParameter param = (OptionParameter)context.Instance;
            return param.RestrictedToList;
        }
    };

    /// <summary>
    /// </summary>
    /// <remarks>
    /// </remarks>
    [System.Runtime.InteropServices.InterfaceType(System.Runtime.InteropServices.ComInterfaceType.InterfaceIsIDispatch)]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("991F95FB-2210-4E44-99B3-4AB793FF46C2")]
    [System.ComponentModel.DescriptionAttribute("CapeRealParameterEvents Interface")]
    interface IOptionParameterSpecEvents
    {
        /// <summary>
        /// Occurs when the user changes of the default value of a parameter.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnParameterDefaultValueChanged</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnParameterDefaultValueChanged</c> in a derived class, be sure to call the base class's <c>OnParameterDefaultValueChanged</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "sender">The <see cref = "RealParameter">RealParameter</see> that raised the event.</param>
        /// <param name = "args">A <see cref = "ParameterDefaultValueChanged">ParameterDefaultValueChanged</see> that contains information about the event.</param>
        void ParameterDefaultValueChanged(object sender, object args);

        /// <summary>
        /// Occurs when the user changes of the lower bound of a parameter.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnComponentNameChanged</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnComponentNameChanged</c> in a derived class, be sure to call the base class's <c>OnComponentNameChanged</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "sender">The <see cref = "RealParameter">RealParameter</see> that raised the event.</param>
        /// <param name = "args">A <see cref = "ParameterValueChangedEventArgs">ParameterValueChangedEventArgs</see> that contains information about the event.</param>
        void ParameterOptionListChanged(object sender, object args);

        /// <summary>
        /// Occurs when the user changes of the upper bound of a parameter.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnParameterUpperBoundChanged</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnParameterUpperBoundChanged</c> in a derived class, be sure to call the base class's <c>OnParameterUpperBoundChanged</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "sender">The <see cref = "RealParameter">RealParameter</see> that raised the event.</param>
        /// <param name = "args">A <see cref = "ParameterLowerBoundChangedEventArgs">ParameterUpperBoundChangedEventArgs</see> that contains information about the event.</param>
        void ParameterRestrictedToListChanged(object sender, object args);

        /// <summary>
        /// Occurs when a parameter is validated.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnParameterValidated</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnParameterValidated</c> in a derived class, be sure to call the base class's <c>OnParameterValidated</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "sender">The <see cref = "RealParameter">RealParameter</see> that raised the event.</param>
        /// <param name = "args">A <see cref = "ParameterValidatedEventArgs">ParameterValidatedEventArgs</see> that contains information about the event.</param>
        void ParameterValidated(object sender, object args);
    }
    
    /// <summary>
    /// String Parameter class that implements the ICapeParameter and ICapeOptionParameterSpec CAPE-OPEN interfaces.
    /// </summary>
    /// <remarks>
    /// This class implements ICapeParameter, ICapeParameterSpec, ICapeOptionParameterSpec, and ICapeIdentification. 
    /// It returns either a string or a System.Object, which is converted to a Variant containing a BSTR by COM Interop.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(IOptionParameterSpecEvents))]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("8EB0F647-618C-4fcc-A16F-39A9D57EA72E")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class OptionParameter : CapeParameter,
        ICapeParameter,
        ICapeParameterSpec,
        ICapeParameterSpecCOM,
        ICapeOptionParameterSpec,
        ICapeOptionParameterSpecCOM,
        ICloneable,
        System.ComponentModel.INotifyPropertyChanged
    {
        private String m_value;
        private String m_DefaultValue;
        private String[] m_OptionList;
        private bool m_Restricted;
        
        /// <summary>
        /// Gets and sets the value for this Parameter.
        /// </summary>
        /// <remarks>
        /// This value uses the System.Object data type for compatibility with 
        /// COM-based CAPE-OPEN.
        /// </remarks>
        /// <value>System.Object</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        override public Object value
        {
            get
            {
                return m_value;
            }
            set
            {
                String message = String.Empty; ;
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_value, value);
                if (this.Validate(value.ToString(), ref message))
                {
                    m_value = value.ToString();
                    OnParameterValueChanged(args);
                    NotifyPropertyChanged("Value");
                    return;
                }
                throw new CapeInvalidArgumentException(message, 0);
            }
        }

        /// <summary>
        /// Gets the list of valid values for the parameter if 'RestrictedtoList' public is true.
        /// </summary>
        /// <remarks>
        /// Used in validating the parameter if the <see cref = "RestrictedToList">RestrictedToList</see>
        /// is set to <c>true</c>.
        /// </remarks>
        /// <value>
        /// String array as a System.Object, COM Variant containing a SafeArray of BSTR.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        [System.ComponentModel.DescriptionAttribute("Gets and Sets the list of valid values for the parameter if 'RestrictedtoList' public is true.")]
        Object ICapeOptionParameterSpecCOM.OptionList
        {
            get
            {
                return m_OptionList;
            }
        }

        /// <summary>
        /// Constructor for the String-valued parameter
        /// </summary>
        /// <remarks>
        /// This constructor sets the ICapeIdentification.ComponentName of the 
        /// parameter. The parameter's value and default value are set to the value. 
        /// </remarks>
        /// <param name = "name">Sets as the ComponentName of the parameter's ICapeIdentification interface.</param>
        /// <param name = "value">Sets the inital value of the parameter.</param>
        public OptionParameter(String name, String value)
            : base(name, String.Empty, CapeParamMode.CAPE_INPUT_OUTPUT)
        {
            m_value = value;
            this.Mode = CapeOpen.CapeParamMode.CAPE_INPUT_OUTPUT;
            m_DefaultValue = value;
            m_ValStatus = CapeOpen.CapeValidationStatus.CAPE_VALID;
        }
        /// <summary>
        /// Constructor for the boolean-valued parameter
        /// </summary>
        /// <remarks>
        /// This constructor sets the ICapeIdentification.ComponentName and 
        /// ICapeIdentification.ComponentDescription of the 
        /// parameter. The parameter's value and default value are set to the value. 
        /// Additionally, the parameters CapeParameterMode is set.
        /// </remarks>
        /// <param name = "name">Sets as the ComponentName of the parameter's ICapeIdentification interface.</param>
        /// <param name = "description">Sets as the ComponentDescription of the parameter's ICapeIdentification interface.</param>
        /// <param name = "value">Sets the inital value of the parameter.</param>
        /// <param name = "defaultValue">Sets the default value of the parameter.</param>
        /// <param name = "options">String array used as the list acceptable options.</param>
        /// <param name = "restricted">Sets whether the parameter value is restricted to values in the option list.</param>
        /// <param name = "mode">Sets the CapeParamMode mode of the parameter.</param>
        public OptionParameter(String name, String description, String value, String defaultValue, String[] options, bool restricted, CapeParamMode mode)
            : base(name, description, mode)
        {
            m_value = value;
            this.Mode = mode;
            m_DefaultValue = defaultValue;
            m_OptionList = options;
            m_Restricted = restricted;
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
            return new OptionParameter(this.ComponentName, this.ComponentDescription, this.Value, this.DefaultValue, this.OptionList, this.RestrictedToList, this.Mode);
        }

        /// <summary>
        /// Occurs when the user changes of the lower bound of the parameter changes.
        /// </summary>
        public event ParameterOptionListChangedHandler ParameterOptionListChanged;
        /// <summary>
        /// Occurs when the user changes of the option list of a parameter.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnParameterOptionListChanged</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnParameterOptionListChanged</c> in a derived class, be sure to call the base class's <c>OnParameterOptionListChanged</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "args">A <see cref = "ParameterValueChangedEventArgs">ParameterOptionListChangedEventArgs</see> that contains information about the event.</param>
        protected void OnParameterOptionListChanged(ParameterOptionListChangedEventArgs args)
        {
            if (ParameterOptionListChanged != null)
            {
                ParameterOptionListChanged(this, args);
            }
        }

        /// <summary>
        /// Occurs when the user changes of the upper bound of the parameter changes.
        /// </summary>
        public event ParameterRestrictedToListChangedHandler ParameterRestrictedToListChanged;
        /// <summary>
        /// Occurs when the user changes of the upper bound of a parameter.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnParameterUpperBoundChanged</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnParameterUpperBoundChanged</c> in a derived class, be sure to call the base class's <c>OnParameterUpperBoundChanged</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "args">A <see cref = "ParameterLowerBoundChangedEventArgs">ParameterUpperBoundChangedEventArgs</see> that contains information about the event.</param>
        protected void OnParameterRestrictedToListChanged(ParameterRestrictedToListChangedEventArgs args)
        {
            if (ParameterRestrictedToListChanged != null)
            {
                ParameterRestrictedToListChanged(this, args);
            }
        }

        /// <summary>
        /// Gets and sets the value of the parameter.
        /// </summary>
        /// <remarks>
        /// The value is returned as a String, which marshals as a BSTR to COM.
        /// </remarks>
        /// <returns>
        /// System.String
        /// </returns>
        /// <value>
        /// The value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.TypeConverter(typeof(OptionParameterValueConverter))]
        [System.ComponentModel.CategoryAttribute("ICapeParameter")]
        [System.ComponentModel.DescriptionAttribute("Gets and sets the value of the parameter.")]
        public String Value
        {
            [System.ComponentModel.DescriptionAttribute("Gets the value of the parameter.")]
            get
            {
                return m_value;
            }
            [System.ComponentModel.DescriptionAttribute("Sets the value of the parameter.")]
            set
            {
                String message = string.Empty;
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_value, value);
                if (this.Validate(value, ref message))
                {
                    m_value = value;
                    OnParameterValueChanged(args);
                    NotifyPropertyChanged("Value");
                    return;
                }
                throw new CapeInvalidArgumentException(message, 0);
            }
        }

        /// <summary>
        /// Validates the current value of the parameter against the parameter's specification.
        /// </summary>
        /// <remarks>
        /// If the value of the <see cref = "RestrictedToList">RestrictedToList</see>
        /// public is set to <c>true</c>, the parameter is valid only if the current value
        /// is included in the <see cref = "OptionList">OptionList</see>. If the 
        /// value of <see cref = "RestrictedToList">RestrictedToList</see> public is <c>false</c>
        /// any valid String is a valid value for the parameter.
        /// </remarks>
        /// <returns>True if the string argument is valid, false if it is not.</returns>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public override bool Validate(ref String message)
        {
            ParameterValidatedEventArgs args;
            if (m_Restricted)
            {
                bool inList = false;
                for (int i = 0; i < m_OptionList.Length; i++)
                {
                    if (m_OptionList[i] == m_value)
                    {
                        inList = true;
                    }
                }
                if (!inList)
                {
                    message = "Value is not in the optin list.";
                    args = new ParameterValidatedEventArgs(this.ComponentName, message, m_ValStatus, CapeOpen.CapeValidationStatus.CAPE_INVALID);
                    m_ValStatus = CapeOpen.CapeValidationStatus.CAPE_INVALID;
                    NotifyPropertyChanged("ValStatus");
                    OnParameterValidated(args);
                    return false;
                }
            }
            message = "Value is valid.";
            args = new ParameterValidatedEventArgs(this.ComponentName, message, m_ValStatus, CapeOpen.CapeValidationStatus.CAPE_VALID);
            m_ValStatus = CapeOpen.CapeValidationStatus.CAPE_VALID;
            NotifyPropertyChanged("ValStatus");
            OnParameterValidated(args);
            return true;
        }

        /// <summary>
        /// Sets the value of the parameter to its default value.
        /// </summary>
        /// <remarks>
        ///  This method sets the parameter's value to the default value.
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        public override void Reset()
        {
            ParameterResetEventArgs args = new ParameterResetEventArgs(this.ComponentName);
            m_value = m_DefaultValue;
            OnParameterReset(args);
            NotifyPropertyChanged("Value");
        }

        // ICapeParameterSpec
        /// <summary>
        /// Gets the type of the parameter. 
        /// </summary>
        /// <remarks>
        /// Gets the <see cref = "CapeParamType"/> of the parameter for which this is a specification: real 
        /// (CAPE_REAL), integer(CAPE_INT), option(CAPE_OPTION), boolean(CAPE_BOOLEAN) 
        /// or array(CAPE_ARRAY).
        /// </remarks>
        /// <value>The parameter type. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeParameterSpec")]
        public override CapeParamType Type
        {
            get
            {
                return CapeParamType.CAPE_OPTION;
            }
        }

        //ICapeOptionParameterSpec

        /// <summary>
        /// Gets and Sets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// Gets and sets the default value of the parameter.
        /// </remarks>
        /// <value>The default value for the parameter. </value>
        [System.ComponentModel.CategoryAttribute("ICapeOptionParameterSpec")]
        [System.ComponentModel.DescriptionAttribute("Gets and Sets the default value of the parameter.")]
        public String DefaultValue
        {
            get
            {
                return m_DefaultValue;
            }
            set
            {
                String message = string.Empty;
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_value, value);
                if (this.Validate(value, ref message))
                {
                    m_DefaultValue = value;
                    OnParameterValueChanged(args);
                    NotifyPropertyChanged("DefaultValue");
                    return;
                }
                throw new CapeInvalidArgumentException(message, 0);
            }
        }

        /// <summary>
        /// Gets and Sets the list of valid values for the parameter if 'RestrictedtoList' public is true.
        /// </summary>
        /// <remarks>
        /// Used in validating the parameter if the <see cref = "RestrictedToList">RestrictedToList</see>
        /// is set to <c>true</c>.
        /// </remarks>
        /// <value>
        /// The option list.
        /// </value>
        [System.ComponentModel.CategoryAttribute("ICapeOptionParameterSpec")]
        [System.ComponentModel.DescriptionAttribute("Gets and Sets the list of valid values for the parameter if 'RestrictedtoList' public is true.")]
        public String[] OptionList
        {
            get
            {
                return m_OptionList;
            }
            set
            {
                ParameterOptionListChangedEventArgs args = new ParameterOptionListChangedEventArgs(this.ComponentName);
                m_OptionList = value;
                OnParameterOptionListChanged(args);
                NotifyPropertyChanged("OptionList");
            }
        }


        /// <summary>
        /// A list of Strings that the valueo f the parameter will be validated against.
        /// </summary>
        /// <remarks>
        /// If <c>true</c>, the parameter's value will be validated against the Strings
        /// in the <see cref = "OptionList">OptionList</see>.
        /// </remarks>
        /// <value>
        /// If <c>true</c>, the parameter's value will be validated against the Strings
        /// in the <see cref = "OptionList">OptionList</see>.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeOptionParameterSpec")]
        [System.ComponentModel.DescriptionAttribute("Limits values of the parameter to the values in the option list if true.")]
        public bool RestrictedToList
        {
            get
            {
                return m_Restricted;
            }
            set
            {
                ParameterRestrictedToListChangedEventArgs args = new ParameterRestrictedToListChangedEventArgs(this.ComponentName, m_Restricted, value);
                m_Restricted = value;
                OnParameterRestrictedToListChanged(args);
                NotifyPropertyChanged("RestrictedToList");
            }
        }

        /// <summary>
        /// Validates the value against the parameter's specification.
        /// </summary>
        /// <remarks>
        /// If the value of the <see cref = "RestrictedToList">RestrictedToList</see>
        /// public is set to <c>true</c>, the value to be tested is a valid value for the 
        /// parameter if it is included in the 
        /// <see cref = "OptionList">OptionList</see>. If the 
        /// value of <see cref = "RestrictedToList">RestrictedToList</see> public is <c>false</c>
        /// any valid String is a valid value for the parameter.
        /// </remarks>
        /// <returns>True if the string argument is valid, false if it is not.</returns>
        /// <param name = "value">The string to be tested for validity.</param>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.DescriptionAttribute("Checks whether the value is an accepatble value for the parameter.")]
        public bool Validate(String value, ref String message)
        {
            if (m_Restricted)
            {
                bool inList = false;
                for (int i = 0; i < m_OptionList.Length; i++)
                {
                    if (m_OptionList[i] == value)
                    {
                        inList = true;
                    }
                }
                if (!inList)
                {
                    message = "Value is not in the option list.";
                    return false;
                }
            }
            message = "Value is valid.";
            return true;
        }
    };


    /// <summary>
    /// Option(string)-Valued parameter for use in the CAPE-OPEN parameter collection.
    /// </summary>
    /// <remarks>
    /// Option(string)-Valued parameter for use in the CAPE-OPEN parameter collection.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(IOptionParameterSpecEvents))]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("70994E8C-179E-40E1-A51B-54A5C0F64A84")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    class OptionParameterWrapper : CapeParameter,
        ICapeParameter,
        ICapeParameterSpec,
        ICapeOptionParameterSpec,
        ICapeOptionParameterSpecCOM,
        ICloneable,
        System.ComponentModel.INotifyPropertyChanged
    {
        [NonSerialized]
        private ICapeParameter m_parameter = null;

        /// <summary>
        /// Gets and sets the value for this Parameter.
        /// </summary>
        /// <remarks>
        /// This value uses the System.Object data type for compatibility with 
        /// COM-based CAPE-OPEN.
        /// </remarks>
        /// <value>System.Object</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        override public Object value
        {
            get
            {
                return m_parameter.value;
            }
            set
            {
                String message = String.Empty; ;
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_parameter.value.ToString(), value);
                m_parameter.value = value.ToString();
                OnParameterValueChanged(args);
                NotifyPropertyChanged("Value");                
            }
        }

        /// <summary>
        /// Gets the list of valid values for the parameter if 'RestrictedtoList' public is true.
        /// </summary>
        /// <remarks>
        /// Used in validating the parameter if the <see cref = "RestrictedToList">RestrictedToList</see>
        /// is set to <c>true</c>.
        /// </remarks>
        /// <value>
        /// String array as a System.Object, COM Variant containing a SafeArray of BSTR.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        [System.ComponentModel.DescriptionAttribute("Gets the list of valid values for the parameter if 'RestrictedtoList' public is true.")]
        Object ICapeOptionParameterSpecCOM.OptionList
        {
            get
            {
                return ((ICapeOptionParameterSpecCOM)m_parameter.Specification).OptionList;
            }
        }


        /// <summary>
        /// Constructor for the String-valued parameter
        /// </summary>
        /// <remarks>
        /// This constructor sets the ICapeIdentification.ComponentName of the 
        /// parameter. The parameter's value and default value are set to the value. 
        /// </remarks>
        /// <param name = "parameter">Sets as the ComponentName of the parameter's ICapeIdentification interface.</param>
        public OptionParameterWrapper(ICapeParameter parameter)
            : base(String.Empty, string.Empty, parameter.Mode)
        {
            m_parameter = parameter;
            this.ComponentName = ((ICapeIdentification)parameter).ComponentName;
            this.ComponentDescription = ((ICapeIdentification)parameter).ComponentDescription;
            this.Mode = m_parameter.Mode;
            m_ValStatus = m_parameter.ValStatus;            
        }        

        /// <summary>
        /// Occurs when the user changes of the lower bound of the parameter changes.
        /// </summary>
        public event ParameterOptionListChangedHandler ParameterOptionListChanged;
        /// <summary>
        /// Occurs when the user changes of the option list of a parameter.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnParameterOptionListChanged</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnParameterOptionListChanged</c> in a derived class, be sure to call the base class's <c>OnParameterOptionListChanged</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "args">A <see cref = "ParameterValueChangedEventArgs">ParameterOptionListChangedEventArgs</see> that contains information about the event.</param>
        protected void OnParameterOptionListChanged(ParameterOptionListChangedEventArgs args)
        {
            if (ParameterOptionListChanged != null)
            {
                ParameterOptionListChanged(this, args);
            }
        }

        /// <summary>
        /// Occurs when the user changes of the upper bound of the parameter changes.
        /// </summary>
        public event ParameterRestrictedToListChangedHandler ParameterRestrictedToListChanged;
        /// <summary>
        /// Occurs when the user changes of the upper bound of a parameter.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnParameterUpperBoundChanged</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnParameterUpperBoundChanged</c> in a derived class, be sure to call the base class's <c>OnParameterUpperBoundChanged</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "args">A <see cref = "ParameterLowerBoundChangedEventArgs">ParameterUpperBoundChangedEventArgs</see> that contains information about the event.</param>
        protected void OnParameterRestrictedToListChanged(ParameterRestrictedToListChangedEventArgs args)
        {
            if (ParameterRestrictedToListChanged != null)
            {
                ParameterRestrictedToListChanged(this, args);
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
            return new OptionParameterWrapper(m_parameter);
        }

        /// <summary>
        /// Gets and sets the value of the parameter.
        /// </summary>
        /// <remarks>
        /// The value is returned as a String, which marshals as a BSTR to COM.
        /// </remarks>
        /// <returns>
        /// System.String
        /// </returns>
        /// <value>
        /// The value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.TypeConverter(typeof(OptionParameterValueConverter))]
        [System.ComponentModel.CategoryAttribute("ICapeParameter")]
        [System.ComponentModel.DescriptionAttribute("Gets and sets the value of the parameter.")]
        public String Value
        {
            [System.ComponentModel.DescriptionAttribute("Gets the value of the parameter.")]
            get
            {
                return m_parameter.value.ToString();
            }
            [System.ComponentModel.DescriptionAttribute("Sets the value of the parameter.")]
            set
            {
                String message = string.Empty;
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_parameter.value.ToString(), value);
                m_parameter.value = value;
                OnParameterValueChanged(args);
                NotifyPropertyChanged("Value");                
            }
        }

        /// <summary>
        /// Validates the current value of the parameter against the parameter's specification.
        /// </summary>
        /// <remarks>
        /// If the value of the <see cref = "RestrictedToList">RestrictedToList</see>
        /// public is set to <c>true</c>, the parameter is valid only if the current value
        /// is included in the <see cref = "OptionList">OptionList</see>. If the 
        /// value of <see cref = "RestrictedToList">RestrictedToList</see> public is <c>false</c>
        /// any valid String is a valid value for the parameter.
        /// </remarks>
        /// <returns>True if the string argument is valid, false if it is not.</returns>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public override bool Validate(ref String message)
        {
            ParameterValidatedEventArgs args;
            CapeValidationStatus valid = m_parameter.ValStatus;
            bool retVal = m_parameter.Validate(message);
            args = new ParameterValidatedEventArgs(this.ComponentName, message, valid, m_parameter.ValStatus);
            NotifyPropertyChanged("ValStatus");
            OnParameterValidated(args);
            return true;
        }

        /// <summary>
        /// Sets the value of the parameter to its default value.
        /// </summary>
        /// <remarks>
        ///  This method sets the parameter's value to the default value.
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        public override void Reset()
        {
            ParameterResetEventArgs args = new ParameterResetEventArgs(this.ComponentName);
            m_parameter.Reset();
            OnParameterReset(args);
            NotifyPropertyChanged("Value");
        }

        // ICapeParameterSpec
        /// <summary>
        /// Gets the type of the parameter. 
        /// </summary>
        /// <remarks>
        /// Gets the <see cref = "CapeParamType"/> of the parameter for which this is a specification: real 
        /// (CAPE_REAL), integer(CAPE_INT), option(CAPE_OPTION), boolean(CAPE_BOOLEAN) 
        /// or array(CAPE_ARRAY).
        /// </remarks>
        /// <value>The parameter type. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeParameterSpec")]
        public override CapeParamType Type
        {
            get
            {
                return CapeParamType.CAPE_OPTION;
            }
        }

        //ICapeOptionParameterSpec

        /// <summary>
        /// Gets and Sets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// Gets and sets the default value of the parameter.
        /// </remarks>
        /// <value>The default value for the parameter. </value>
        [System.ComponentModel.CategoryAttribute("ICapeOptionParameterSpec")]
        [System.ComponentModel.DescriptionAttribute("Gets and Sets the default value of the parameter.")]
        public String DefaultValue
        {
            get
            {
                return ((ICapeOptionParameterSpecCOM)m_parameter.Specification).DefaultValue;
            }
            set
            {
            }
        }

        /// <summary>
        /// Gets and Sets the list of valid values for the parameter if 'RestrictedtoList' public is true.
        /// </summary>
        /// <remarks>
        /// Used in validating the parameter if the <see cref = "RestrictedToList">RestrictedToList</see>
        /// is set to <c>true</c>.
        /// </remarks>
        /// <value>
        /// The option list.
        /// </value>
        [System.ComponentModel.CategoryAttribute("ICapeOptionParameterSpec")]
        [System.ComponentModel.DescriptionAttribute("Gets and Sets the list of valid values for the parameter if 'RestrictedtoList' public is true.")]
        public String[] OptionList
        {
            get
            {
                return (string[])((ICapeOptionParameterSpecCOM)m_parameter.Specification).OptionList;
            }
            set
            {
            }
        }


        /// <summary>
        /// A list of Strings that the valueo f the parameter will be validated against.
        /// </summary>
        /// <remarks>
        /// If <c>true</c>, the parameter's value will be validated against the Strings
        /// in the <see cref = "OptionList">OptionList</see>.
        /// </remarks>
        /// <value>
        /// If <c>true</c>, the parameter's value will be validated against the Strings
        /// in the <see cref = "OptionList">OptionList</see>.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeOptionParameterSpec")]
        [System.ComponentModel.DescriptionAttribute("Limits values of the parameter to the values in the option list if true.")]
        public bool RestrictedToList
        {
            get
            {
                return ((ICapeOptionParameterSpecCOM)m_parameter.Specification).RestrictedToList;
            }
            set
            {
            }
        }

        /// <summary>
        /// Validates the value against the parameter's specification.
        /// </summary>
        /// <remarks>
        /// If the value of the <see cref = "RestrictedToList">RestrictedToList</see>
        /// public is set to <c>true</c>, the value to be tested is a valid value for the 
        /// parameter if it is included in the 
        /// <see cref = "OptionList">OptionList</see>. If the 
        /// value of <see cref = "RestrictedToList">RestrictedToList</see> public is <c>false</c>
        /// any valid String is a valid value for the parameter.
        /// </remarks>
        /// <returns>True if the string argument is valid, false if it is not.</returns>
        /// <param name = "value">The string to be tested for validity.</param>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.DescriptionAttribute("Checks whether the value is an accepatble value for the parameter.")]
        public bool Validate(String value, ref String message)
        {
            return ((ICapeOptionParameterSpecCOM)m_parameter.Specification).Validate(value, ref message);
        }
    };
}

