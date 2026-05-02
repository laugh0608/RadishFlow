using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Diagnostics;

namespace CapeOpen
{

    /// <summary>
    /// Indicates that the specification of a boolean-valued parameter has been changed.
    /// </summary>
    /// <remarks>
    /// <para>This interface is exposed to COM-based PMEs and serves as a source interface for events associated with changes
    /// to the specification of a boolean-valued parameters.</para>
    /// <para>This interface is not a part of the CAPE-OPEN specifications. This interface and its implementation is 
    /// provided to give COM-based developers similar functionality as .NET-based developers.</para>
    /// </remarks>
    [System.Runtime.InteropServices.InterfaceType(System.Runtime.InteropServices.ComInterfaceType.InterfaceIsIDispatch)]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("07D17ED3-B25A-48EA-8261-5ED2D076ABDD")]
    [System.ComponentModel.DescriptionAttribute("CapeRealParameterEvents Interface")]
    interface ICapeBooleanParameterSpecEvents
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
        /// Occurs when a parameter is validated.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnParameterValidated</c> method also allows derived classes to handle the event without 
        /// attaching a delegate. This is the preferred technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnParameterValidated</c> in a derived class, be sure to call the base class's 
        /// <c>OnParameterValidated</c> method so that registered delegates receive the event.</para>
        /// </remarks>
        /// <param name = "sender">The <see cref = "ICapeBooleanParameterSpec">ICapeBooleanParameterSpec</see> that raised the event.</param>
        /// <param name = "args">A <see cref = "ParameterValidatedEventArgs">ParameterValidatedEventArgs</see> that contains information about the event.</param>
        void ParameterValidated(object sender, object args);
    }


    /// <summary>
    /// Boolean-Valued parameter for use in the CAPE-OPEN parameter collection.
    /// </summary>
    /// <remarks>
    /// Boolean-Valued parameter for use in the CAPE-OPEN parameter collection.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(ICapeBooleanParameterSpecEvents))]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("8B8BC504-EEB5-4a13-B016-9614543E4536")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class BooleanParameter : CapeParameter,
        ICapeParameter,
        ICapeParameterSpec,
        ICapeParameterSpecCOM,
        ICapeBooleanParameterSpec,
        System.ComponentModel.INotifyPropertyChanged
    {
        private bool m_value;
        private bool m_DefaultValue;

        /// <summary>
        /// Gets and sets the value for this Parameter.
        /// </summary>
        /// <remarks>
        /// This value uses the System.Object data type for compatibility with COM-based CAPE-OPEN. The value is marshalled 
        /// to COM as a boolean-valued variant, which is also called a VARIANT_BOOL.
        /// </remarks>
        /// <seealso href="http://msdn.microsoft.com/en-us/library/cc235510.aspx"/>
        /// <seealso href="http://blogs.msdn.com/b/oldnewthing/archive/2004/12/22/329884.aspx"/>
        /// <value>A boxed boolean value of the parameter.</value>
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
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_value, Convert.ToBoolean(value));
                m_value = Convert.ToBoolean(value);
                OnParameterValueChanged(args);
            }
        }
        
        /// <summary>
        /// Constructor for the boolean-valued parameter
        /// </summary>
        /// <remarks>
        /// This constructor sets the <see cref = "ICapeIdentification.ComponentName"/> of the 
        /// parameter. The parameter's value and default value are set to the value. 
        /// Additionally, the parameters <see cref = "CapeOpen.CapeParamMode"/> is set.
        /// </remarks>
        /// <param name = "name">Sets as the ComponentName of the parameter's ICapeIdentification interface.</param>
        /// <param name = "value">Sets the inital and default value of the parameter.</param>
        /// <param name = "mode">Sets the CapeParamMode mode of the parameter.</param>
        public BooleanParameter(String name, bool value, CapeParamMode mode)
            : base(name, String.Empty, mode)
        {
            m_value = value;
            m_DefaultValue = value;
            this.Mode = mode;            
        }
        /// <summary>
        /// Constructor for the boolean-valued parameter
        /// </summary>
        /// <remarks>
        /// This constructor sets the <see cref = "ICapeIdentification.ComponentName"/> and 
        /// <see cref = "ICapeIdentification.ComponentDescription"/> of the 
        /// parameter. The parameter's value and default value are set to the value. 
        /// Additionally, the parameters CapeParamMode is set.
        /// </remarks>
        /// <param name = "name">Sets as the ComponentName of the parameter's ICapeIdentification interface.</param>
        /// <param name = "description">Sets as the ComponentDescription of the parameter's ICapeIdentification interface.</param>
        /// <param name = "value">Sets the inital value of the parameter.</param>
        /// <param name = "defaultValue">Sets the default value of the parameter.</param>
        /// <param name = "mode">Sets the CapeParamMode mode of the parameter.</param>
        public BooleanParameter(String name, String description, bool value, bool defaultValue, CapeParamMode mode)
            : base(name, description, mode)
        {
            m_value = value;
            this.Mode = mode;
            m_DefaultValue = defaultValue;
            m_ValStatus = CapeOpen.CapeValidationStatus.CAPE_VALID;
        }

        // ICloneable
        /// <summary>
        /// Creates a copy of the parameter.
        /// </summary>
        /// <remarks><para>The clone method is used to create a deep copy of the parameter.</para>
        /// </remarks>
        /// <returns>A copy of the current parameter.</returns>
        override public object Clone()
        {
            return new BooleanParameter(this.ComponentName, this.ComponentDescription, m_value, m_DefaultValue, this.Mode);
        }

        /// <summary>
        /// Gets and sets the value for this Parameter. 
        /// </summary>
        /// <remarks>
        /// The value of the parameter. This value of the parameter is only available through .NET, adn it is not made 
        /// available to COM.
        /// </remarks>
        /// <value>
        /// The value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeParameter")]
        public bool Value
        {
            get
            {
                return m_value;
            }
            set
            {
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_value, value);
                m_value = value;
                OnParameterValueChanged(args);
            }
        }

        /// <summary>
        /// Validates the current value of the parameter against the specification of the parameter.
        /// </summary>
        /// <remarks>
        /// This method checks the current value of the parameter to determine if it is an allowed value. Any valid boolean value (true/false) 
        /// valid for the <see cref = "ICapeBooleanParameterSpec"/> paramaters.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "message">The message is used to return the reason that the parameter is invalid.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        public override bool Validate(ref String message)
        {
            message = "Value is valid.";
            m_ValStatus = CapeValidationStatus.CAPE_VALID;
            ParameterValidatedEventArgs args = new ParameterValidatedEventArgs(this.ComponentName, message, CapeValidationStatus.CAPE_VALID, CapeValidationStatus.CAPE_VALID);
            OnParameterValidated(args);
            return true;
        }

        /// <summary>
        /// Sets the value of the parameter to its default value.
        /// </summary>
        /// <remarks>
        /// Sets the value of the parameter to its default value.
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        public override void Reset()
        {
            ParameterResetEventArgs args = new ParameterResetEventArgs(this.ComponentName);
            m_value = m_DefaultValue;
            OnParameterReset(args);
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
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeParameterSpec")]
        public override CapeParamType Type
        {
            get
            {
                return CapeParamType.CAPE_BOOLEAN;
            }
        }

        //ICapeBooleanParameterSpec

        /// <summary>
        /// Gets and sets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// Gets and sets the default value of the parameter.
        /// </remarks>
        /// <value>
        /// The default value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeBooleanParameterSpec")]
        public bool DefaultValue
        {
            get
            {
                return m_DefaultValue;
            }
            set
            {
                ParameterDefaultValueChangedEventArgs args = new ParameterDefaultValueChangedEventArgs(this.ComponentName, m_DefaultValue, value);
                m_DefaultValue = value;
                OnParameterDefaultValueChanged(args);
            }
        }

        /// <summary>
        /// Validates the value sent against the specification of the parameter.
        /// </summary>
        /// <remarks>
        /// Validates whether the argument is accepted by the parameter as a valid value. 
        /// It returns a flag to indicate the success or failure of the validation together 
        /// with a text message which can be used to convey the reasoning to the client/user.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "value">Boolean value that will be validated against the parameter's current specification.</param>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public bool Validate(bool value, ref String message)
        {
            message = "Value is valid.";
            return true;
        }
    };

    /// <summary>
    /// Boolean-Valued parameter for use in the CAPE-OPEN parameter collection.
    /// </summary>
    /// <remarks>
    /// Boolean-Valued parameter for use in the CAPE-OPEN parameter collection.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(ICapeBooleanParameterSpecEvents))]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("A6751A39-8A2C-4AFC-AD57-6395FFE0A7FE")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    class BooleanParameterWrapper : CapeParameter,
        ICapeParameter,
        ICapeParameterSpec,
        ICapeBooleanParameterSpec,        
        System.ComponentModel.INotifyPropertyChanged
    {
        private ICapeParameter m_Parameter;

        /// <summary>
        /// Gets and sets the value for this Parameter.
        /// </summary>
        /// <remarks>
        /// This value uses the System.Object data type for compatibility with 
        /// COM-based CAPE-OPEN.
        /// </remarks>
        /// <value>A boxed boolean value of the parameter.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        override public Object value
        {
            get
            {
                return m_Parameter.value;
            }
            set
            {
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_Parameter.value, Convert.ToBoolean(value));
                m_Parameter.value = value;
                OnParameterValueChanged(args);
            }
        }
        
        /// <summary>
        /// Constructor for the wrapper class for COM-based boolean-valued parameter.
        /// </summary>
        /// <remarks>
        /// This constructor creates an instance of a class that wraps a COM-based boolean-valued paramemter. This 
        /// wrapper exposes appropriate .NET-based parameter interfaces for the wrapped parameter.
        /// </remarks>
        /// <param name = "parameter">The parameter to be wrapped.</param>
        public BooleanParameterWrapper(ICapeParameter parameter)
           : base(String.Empty, string.Empty, parameter.Mode)
        {
            m_Parameter = parameter;
            this.ComponentName = ((ICapeIdentification)parameter).ComponentName;
            this.ComponentDescription = ((ICapeIdentification)parameter).ComponentDescription;
            this.Mode = parameter.Mode;
            m_ValStatus = parameter.ValStatus;
        }
        
        // ICloneable
        /// <summary>
        /// Creates a copy of the parameter.
        /// </summary>
        /// <remarks><para>The clone method is used to create a copy of the parameter. Both the original version 
        /// and the clone refer to the same COM-based parameter.</para>
        /// </remarks>
        /// <returns>A copy of the current parameter.</returns>
        override public object Clone()
        {
            return new BooleanParameterWrapper(m_Parameter);
        }

        /// <summary>
        /// Gets and sets the value for this Parameter. 
        /// </summary>
        /// <remarks>
        /// The value of the parameter.
        /// </remarks>
        /// <value>
        /// The value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeParameter")]
        public bool Value
        {
            get
            {
                return (bool)m_Parameter.value;
            }
            set
            {
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, (bool)m_Parameter.value, value);
                m_Parameter.value = value;
                OnParameterValueChanged(args);
            }
        }

        /// <summary>
        /// Validates the current value of the parameter against the 
        /// specification of the parameter.
        /// </summary>
        /// <remarks>
        /// This method checks the current value of the parameter to determine if it is an allowed value. Any valid 
        /// boolean value (true/false) is valid for the <see cref = "ICapeBooleanParameterSpec"/> paramaters.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "message">The message is used to return the reason that the parameter is invalid.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        public override bool Validate(ref String message)
        {
            CapeValidationStatus valid = m_Parameter.ValStatus;
            bool retVal = m_Parameter.Validate(message);
            ParameterValidatedEventArgs args = new ParameterValidatedEventArgs(this.ComponentName, message, valid, m_Parameter.ValStatus);
            OnParameterValidated(args);
            return retVal;
        }

        /// <summary>
        /// Sets the value of the parameter to its default value.
        /// </summary>
        /// <remarks>
        /// Sets the value of the parameter to its default value.
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        public override void Reset()
        {
            ParameterResetEventArgs args = new ParameterResetEventArgs(this.ComponentName);
            m_Parameter.Reset();
            OnParameterReset(args);
        }

        // ICapeParameterSpec
        /// <summary>
        /// Gets the type of the parameter. 
        /// </summary>
        /// <remarks>
        /// Gets the <see cref = "CapeParamType"/> of the parameter.
        /// </remarks>
        /// <value>The parameter type. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeParameterSpec")]
        public override CapeParamType Type
        {
            get
            {
                return CapeParamType.CAPE_BOOLEAN;
            }
        }

        //ICapeBooleanParameterSpec

        /// <summary>
        /// Gets the default value of the wrapped parameter.
        /// </summary>
        /// <remarks>
        /// The COM-based <see cref="ICapeBooleanParameterSpec"/> boolean interface does not provide a means to change the 
        /// default value of the parameter. As such, the default value of the wrapped parameter can not be changed.
        /// </remarks>
        /// <value>
        /// The default value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeBooleanParameterSpec")]
        public bool DefaultValue
        {
            get
            {
                return ((ICapeBooleanParameterSpec)m_Parameter.Specification).DefaultValue;
            }
            set
            {
            }
        }

        /// <summary>
        /// Validates the value sent against the specification of the parameter.
        /// </summary>
        /// <remarks>
        /// Validates whether the argument is accepted by the parameter as a valid value. 
        /// It returns a flag to indicate the success or failure of the validation together 
        /// with a text message which can be used to convey the reasoning to the client/user.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "value">Boolean value that will be validated against the parameter's current specification.</param>
        /// <param name = "message">Reference to a string that will contain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public bool Validate(bool value, ref String message)
        {
            return ((ICapeBooleanParameterSpec)m_Parameter.Specification).Validate(value, message);
        }
    };
}
