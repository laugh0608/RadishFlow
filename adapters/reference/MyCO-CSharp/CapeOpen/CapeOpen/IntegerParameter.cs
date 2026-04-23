using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{

    /// <summary>
    /// Indicates that the specification of an interger-valued parameter has been changed.
    /// </summary>
    /// <remarks>
    /// <para>This interface is exposed to COM-based PMEs and serves as a source interface for events associated with changes
    /// to the specification of an integer-valued parameters.</para>
    /// <para>This interface is not a part of the CAPE-OPEN specifications. This interface and its implementation is 
    /// provided to give COM-based developers similar functionality as .NET-based developers.</para>
    /// </remarks>
    [System.Runtime.InteropServices.InterfaceType(System.Runtime.InteropServices.ComInterfaceType.InterfaceIsIDispatch)]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("2EA7C47A-A4E0-47A2-8AC1-658F96A0B79D")]
    [System.ComponentModel.DescriptionAttribute("CapeIntegerParameterEvents Interface")]
    interface ICapeIntegerParameterSpecEvents
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
        /// <param name = "sender">The <see cref = "IntegerParameter">RealParameter</see> that raised the event.</param>
        /// <param name = "args">A <see cref = "ParameterDefaultValueChanged">ParameterDefaultValueChanged</see> that contains information about the event.</param>
        void ParameterDefaultValueChanged(
            [System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]object sender, 
            [System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]object args
            );

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
        /// <param name = "sender">The <see cref = "IntegerParameter">RealParameter</see> that raised the event.</param>
        /// <param name = "args">A <see cref = "ParameterValueChangedEventArgs">ParameterValueChangedEventArgs</see> that contains information about the event.</param>
        void ParameterLowerBoundChanged(
            [System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]object sender, 
            [System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]object args
            );

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
        /// <param name = "sender">The <see cref = "IntegerParameter">RealParameter</see> that raised the event.</param>
        /// <param name = "args">A <see cref = "ParameterLowerBoundChangedEventArgs">ParameterUpperBoundChangedEventArgs</see> that contains information about the event.</param>
        void ParameterUpperBoundChanged(object sender, object args);

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
        /// <param name = "sender">The <see cref = "IntegerParameter">RealParameter</see> that raised the event.</param>
        /// <param name = "args">A <see cref = "ParameterValidatedEventArgs">ParameterValidatedEventArgs</see> that contains information about the event.</param>
        void ParameterValidated(object sender, object args);
    }


    /// <summary>
    /// Intger-Valued parameter for use in the CAPE-OPEN parameter collection.
    /// </summary>
    /// <remarks>
    /// Intger-Valued parameter for use in the CAPE-OPEN parameter collection.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(ICapeIntegerParameterSpecEvents))]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("2C57DC9F-1368-42eb-888F-5BC6ED7DDFA7")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class IntegerParameter : CapeParameter,
        ICapeParameter,
        ICapeParameterSpec,
        ICapeParameterSpecCOM,
        ICapeIntegerParameterSpec,
        System.ComponentModel.INotifyPropertyChanged
    {

        private int m_value;
        private int m_DefaultValue, m_LowerBound, m_UpperBound;
        
        /// <summary>
        /// Gets and sets the value for this Parameter.
        /// </summary>
        /// <remarks>
        /// This value uses the System.Object data type for compatibility with 
        /// COM-based CAPE-OPEN.
        /// </remarks>
        /// <value>The value of the parameter.</value>
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
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_value, value);
                m_value = (int)value;
                OnParameterValueChanged(args);
                NotifyPropertyChanged("Value");
            }
        }

        /// <summary>
        /// Creates a new instance of the integer-valued parameter class.
        /// </summary>
        /// <remarks>
        /// <para>The default value is set to the inital value of the parameter. The upper
        /// bound is set to Int32.MaxValue (2,147,483,647) and the lower bound is set to 
        /// Int32.MinValue (-2,147,483,648). The mode is set to CapeParamMode.CAPE_INPUT_OUTPUT.</para>
        /// </remarks>
        /// <param name = "name">Sets as the ComponentName of the parameter's ICapeIdentification interface.</param>
        /// <param name = "value">Sets the inital value of the parameter.</param>
        /// <param name = "mode">Sets the CapeParamMode mode of the parameter</param>
        public IntegerParameter(String name, int value, CapeParamMode mode)
            : base(name, String.Empty, mode)
        {
            m_value = value;
            this.Mode = mode;
            m_LowerBound = int.MinValue;
            m_UpperBound = int.MaxValue;
            m_DefaultValue = value;
        }
        /// <summary>
        /// Creates a new instance of the integer-valued parameter class using the values enterred. 
        /// </summary>
        /// <remarks>
        /// The default value, upper and lower 
        /// bound, as well as the mode of the parameter are specified in this constructor.
        /// </remarks>
        /// <param name = "name">Sets as the ComponentName of the parameter's ICapeIdentification interface.</param>
        /// <param name = "description">Sets as the ComponentDescription of the parameter's ICapeIdentification interface.</param>
        /// <param name = "value">Sets the inital value of the parameter.</param>
        /// <param name = "defaultValue">Sets the default value of the parameter.</param>
        /// <param name = "minValue">Sets the lower bound of the parameter.</param>
        /// <param name = "maxValue">Sets the upper bound of the parameter.</param>
        /// <param name = "mode">Sets the CapeParamMode mode of the parameter.</param>
        public IntegerParameter(String name, String description, int value, int defaultValue, int minValue, int maxValue, CapeParamMode mode)
            : base(name, description, mode)
        {
            m_value = value;
            this.Mode = mode;
            m_LowerBound = minValue;
            m_UpperBound = maxValue;
            m_DefaultValue = defaultValue;
            String message = "";
            if (!this.Validate(ref message))
            {
                System.Windows.Forms.MessageBox.Show(message, String.Concat("Invalid Parameter Value: ", this.ComponentName));
            }
        }

        /// <summary>
        /// Occurs when the user changes of the lower bound of the parameter changes.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// </remarks>
        public event ParameterLowerBoundChangedHandler ParameterLowerBoundChanged;
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
        /// <param name = "args">A <see cref = "ParameterValueChangedEventArgs">ParameterValueChangedEventArgs</see> that contains information about the event.</param>
        protected void OnParameterLowerBoundChanged(ParameterLowerBoundChangedEventArgs args)
        {
            if (ParameterLowerBoundChanged != null)
            {
                ParameterLowerBoundChanged(this, args);
            }
        }

        /// <summary>
        /// Occurs when the user changes of the upper bound of the parameter changes.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// </remarks>
        public event ParameterUpperBoundChangedHandler ParameterUpperBoundChanged;
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
        protected void OnParameterUpperBoundChanged(ParameterUpperBoundChangedEventArgs args)
        {
            if (ParameterUpperBoundChanged != null)
            {
                ParameterUpperBoundChanged(this, args);
            }
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
            return new IntegerParameter(this.ComponentName, this.ComponentDescription, m_value, m_DefaultValue, m_LowerBound, m_UpperBound, this.Mode);
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
        public int Value
        {
            get
            {
                return m_value;
            }
            set
            {
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_value, value);
                m_value = (int)value;
                OnParameterValueChanged(args);
                NotifyPropertyChanged("Value");
            }
        }

        /// <summary>
        /// Validates the current value of the parameter against the 
        /// specification of the parameter.
        /// </summary>
        /// <remarks>
        /// The parameter is considered valid if the current value is between the 
        /// upper and lower bound. The message is used to return the reason that 
        /// the parameter is invalid.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public override bool Validate(ref String message)
        {
            ParameterValidatedEventArgs args;
            if (m_value < m_LowerBound)
            {
                message = "Value below the Lower Bound.";
                args = new ParameterValidatedEventArgs(this.ComponentName, message, m_ValStatus, CapeValidationStatus.CAPE_INVALID);
                m_ValStatus = CapeValidationStatus.CAPE_INVALID;
                NotifyPropertyChanged("ValStatus");
                OnParameterValidated(args);
                return false;
            }
            if (m_value > m_UpperBound)
            {
                message = "Value greater than upper bound.";
                args = new ParameterValidatedEventArgs(this.ComponentName, message, m_ValStatus, CapeValidationStatus.CAPE_INVALID);
                m_ValStatus = CapeValidationStatus.CAPE_INVALID;
                NotifyPropertyChanged("ValStatus");
                OnParameterValidated(args);
                return false;
            }
            message = "Value is valid.";
            args = new ParameterValidatedEventArgs(this.ComponentName, message, m_ValStatus, CapeValidationStatus.CAPE_VALID);
            m_ValStatus = CapeValidationStatus.CAPE_VALID;
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
                return CapeParamType.CAPE_INT;
            }
        }

        //ICapeIntegerParameterSpec

        /// <summary>
        /// Gets and sets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// Gets and sets the default value of the parameter.
        /// </remarks>
        /// <value>The default value for the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeIntegerParameterSpec")]
        public int DefaultValue
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
                NotifyPropertyChanged("DefaultValue");
            }
        }

        /// <summary>
        /// Gets and sets the lower bound of the parameter.
        /// </summary>
        /// <remarks>
        /// The lower bound can be an valid integer. By default, it is set to 
        /// Int32.MinValue, 2,147,483,648; that is, hexadecimal 0x80000000
        /// </remarks>
        /// <value>The lower bound for the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeIntegerParameterSpec")]
        public int LowerBound
        {
            get
            {
                return m_LowerBound;
            }
            set
            {
                ParameterLowerBoundChangedEventArgs args = new ParameterLowerBoundChangedEventArgs(this.ComponentName, m_LowerBound, value);
                m_LowerBound = value;
                OnParameterLowerBoundChanged(args);
                NotifyPropertyChanged("LowerBound");
            }
        }

        /// <summary>
        /// Gets and sets the upper bound of the parameter.
        /// </summary>
        /// <remarks>
        /// The lower bound can be an valid integer. By default, it is set to 
        /// Int32.MaxValue, 2,147,483,647; that is, hexadecimal 0x7FFFFFFF.
        /// </remarks>
        /// <value>The upper bound for the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeIntegerParameterSpec")]
        public int UpperBound
        {
            get
            {
                return m_UpperBound;
            }
            set
            {
                ParameterUpperBoundChangedEventArgs args = new ParameterUpperBoundChangedEventArgs(this.ComponentName, m_UpperBound, value);
                m_UpperBound = value;
                OnParameterUpperBoundChanged(args);
                NotifyPropertyChanged("UpperBound");
            }
        }

        /// <summary>
        /// Validates the value sent against the specification of the parameter. 
        /// </summary>
        /// <remarks>
        /// The parameter is considered valid if the current value is between 
        /// the upper and lower bound. The message is used to return the reason 
        /// that the parameter is invalid.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "value">Integer value that will be validated against the parameter's current specification.</param>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public bool Validate(int value, ref String message)
        {
            if (value < m_LowerBound)
            {
                message = "Value below the Lower Bound.";
                return false;
            }
            if (value > m_UpperBound)
            {
                message = "Value greater than upper bound.";
                return false;
            }
            message = "Value is valid.";
            return true;
        }
    };

    /// <summary>
    /// Integer-Valued parameter for use in the CAPE-OPEN parameter collection.
    /// </summary>
    /// <remarks>
    /// Integer-Valued parameter for use in the CAPE-OPEN parameter collection.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(ICapeIntegerParameterSpecEvents))]
    [System.Runtime.InteropServices.Guid("EFC01B53-9A6A-4AD9-97BE-3F0294B3BBFB")]//ICapeThermoMaterialObject_IID)
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    class IntegerParameterWrapper : CapeParameter,
        ICapeParameter,
        ICapeParameterSpec,
        ICapeIntegerParameterSpec,
        ICloneable,
        System.ComponentModel.INotifyPropertyChanged
    {
        ICapeParameter m_parameter;

        /// <summary>
        /// Gets and sets the value for this Parameter.
        /// </summary>
        /// <remarks>
        /// This value uses the System.Object data type for compatibility with 
        /// COM-based CAPE-OPEN.
        /// </remarks>
        /// <value>The value of the parameter.</value>
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
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_parameter.value, value);
                m_parameter.value = (int)value;
                OnParameterValueChanged(args);
                NotifyPropertyChanged("Value");
            }
        }

        /// <summary>
        /// Creates a new instance of the integer-valued parameter class.
        /// </summary>
        /// <remarks>
        /// <para>The default value is set to the inital value of the parameter. The upper
        /// bound is set to Int32.MaxValue (2,147,483,647) and the lower bound is set to 
        /// Int32.MinValue (-2,147,483,648). The mode is set to CapeParamMode.CAPE_INPUT_OUTPUT.</para>
        /// </remarks>
        /// <param name = "parameter">Sets the inital value of the parameter.</param>
        public IntegerParameterWrapper(ICapeParameter parameter)
            : base(String.Empty, string.Empty, parameter.Mode)
        {
            m_parameter = parameter;
            this.ComponentName = ((ICapeIdentification)parameter).ComponentName;
            this.ComponentDescription = ((ICapeIdentification)parameter).ComponentDescription;
            this.Mode = parameter.Mode;
            m_ValStatus = parameter.ValStatus;
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
            return new IntegerParameter(this.ComponentName, this.ComponentDescription, this.Value, this.DefaultValue, this.LowerBound, this.UpperBound, this.Mode);
        }

        /// <summary>
        /// Occurs when the user changes of the lower bound of the parameter changes.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// </remarks>
        public event ParameterLowerBoundChangedHandler ParameterLowerBoundChanged;
        /// <summary>
        /// Occurs when the user changes of the lower bound of a parameter.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnParameterLowerBoundChanged</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnParameterLowerBoundChanged</c> in a derived class, be sure to call the base class's <c>OnComponentNameChanged</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "args">A <see cref = "ParameterValueChangedEventArgs">ParameterValueChangedEventArgs</see> that contains information about the event.</param>
        protected void OnParameterLowerBoundChanged(ParameterLowerBoundChangedEventArgs args)
        {
            if (ParameterLowerBoundChanged != null)
            {
                ParameterLowerBoundChanged(this, args);
            }
        }

        /// <summary>
        /// Occurs when the user changes of the upper bound of the parameter changes.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// </remarks>
        public event ParameterUpperBoundChangedHandler ParameterUpperBoundChanged;
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
        protected void OnParameterUpperBoundChanged(ParameterUpperBoundChangedEventArgs args)
        {
            if (ParameterUpperBoundChanged != null)
            {
                ParameterUpperBoundChanged(this, args);
            }
        }
        // ICloneable
        /// <summary>
        /// Creates a copy of the parameter.
        /// </summary>
        /// <remarks><para>The clone method is used to create a deep copy of the parameter.</para>
        /// </remarks>
        /// <returns>A copy of the current parameter.</returns>
        Object ICloneable.Clone()
        {
            return new IntegerParameterWrapper(m_parameter);
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
        public int Value
        {
            get
            {
                return (int)m_parameter.value;
            }
            set
            {
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_parameter.value, value);
                m_parameter.value = value;
                OnParameterValueChanged(args);
                NotifyPropertyChanged("Value");
            }
        }

        /// <summary>
        /// Validates the current value of the parameter against the 
        /// specification of the parameter.
        /// </summary>
        /// <remarks>
        /// The parameter is considered valid if the current value is between the 
        /// upper and lower bound. The message is used to return the reason that 
        /// the parameter is invalid.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public override bool Validate(ref String message)
        {
            ParameterValidatedEventArgs args;
            CapeValidationStatus valStatus = m_parameter.ValStatus;
            bool retval = m_parameter.Validate(message);
            args = new ParameterValidatedEventArgs(this.ComponentName, message, ValStatus, m_parameter.ValStatus);
            OnParameterValidated(args);
            this.NotifyPropertyChanged("ValStatus");
            return retval;
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
                return CapeParamType.CAPE_INT;
            }
        }

        //ICapeIntegerParameterSpec

        /// <summary>
        /// Gets and sets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// Gets and sets the default value of the parameter.
        /// </remarks>
        /// <value>The default value for the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeIntegerParameterSpec")]
        public int DefaultValue
        {
            get
            {
                return ((ICapeIntegerParameterSpec)m_parameter.Specification).DefaultValue;
            }
            set
            {
            }
        }

        /// <summary>
        /// Gets and sets the lower bound of the parameter.
        /// </summary>
        /// <remarks>
        /// The lower bound can be an valid integer. By default, it is set to 
        /// Int32.MinValue, 2,147,483,648; that is, hexadecimal 0x80000000
        /// </remarks>
        /// <value>The lower bound for the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeIntegerParameterSpec")]
        public int LowerBound
        {
            get
            {
                return ((ICapeIntegerParameterSpec)m_parameter.Specification).LowerBound;
            }
            set
            {
            }
        }

        /// <summary>
        /// Gets and sets the upper bound of the parameter.
        /// </summary>
        /// <remarks>
        /// The lower bound can be an valid integer. By default, it is set to 
        /// Int32.MaxValue, 2,147,483,647; that is, hexadecimal 0x7FFFFFFF.
        /// </remarks>
        /// <value>The upper bound for the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.CategoryAttribute("ICapeIntegerParameterSpec")]
        public int UpperBound
        {
            get
            {
                return ((ICapeIntegerParameterSpec)m_parameter.Specification).UpperBound;
            }
            set
            {
            }
        }

        /// <summary>
        /// Validates the value sent against the specification of the parameter. 
        /// </summary>
        /// <remarks>
        /// The parameter is considered valid if the current value is between 
        /// the upper and lower bound. The message is used to return the reason 
        /// that the parameter is invalid.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "value">Integer value that will be validated against the parameter's current specification.</param>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public bool Validate(int value, ref String message)
        {
            return ((ICapeIntegerParameterSpec)m_parameter.Specification).Validate(value, message);
        }
    };
}

