using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{

    class UnitConverter : System.ComponentModel.StringConverter
    {
        public override bool GetStandardValuesSupported(System.ComponentModel.ITypeDescriptorContext context)
        {
            return true;
        }
        public override System.ComponentModel.TypeConverter.StandardValuesCollection GetStandardValues(System.ComponentModel.ITypeDescriptorContext context)
        {
            CapeOpen.RealParameter unit = (CapeOpen.RealParameter)context.Instance;
            string[] retVal = CapeOpen.CDimensions.UnitsMatchingCategory(CapeOpen.CDimensions.UnitCategory(unit.Unit));
            return new System.ComponentModel.TypeConverter.StandardValuesCollection(retVal);
        }
        public override bool GetStandardValuesExclusive(System.ComponentModel.ITypeDescriptorContext context)
        {
            return true;
        }
    };

    class RealParameterTypeConverter : ParameterTypeConverter
    {
        public override bool CanConvertTo(System.ComponentModel.ITypeDescriptorContext context, System.Type destinationType)
        {
            if (typeof(CapeOpen.RealParameter).IsAssignableFrom(destinationType))
                return true;
            return base.CanConvertTo(context, destinationType);
        }
        public override Object ConvertTo(System.ComponentModel.ITypeDescriptorContext context, System.Globalization.CultureInfo culture, Object value, System.Type destinationType)
        {
            if ((typeof(System.String).IsAssignableFrom(destinationType)) && typeof(CapeOpen.RealParameter).IsAssignableFrom(value.GetType()))
            {
                CapeOpen.RealParameter param = (CapeOpen.RealParameter)value;
                return String.Concat(param.DimensionedValue.ToString(), " ", param.Unit);
            }
            return base.ConvertTo(context, culture, value, destinationType);
        }
    };


    /// <summary>
    /// </summary>
    /// <remarks>
    /// </remarks>
    [System.Runtime.InteropServices.InterfaceType(System.Runtime.InteropServices.ComInterfaceType.InterfaceIsIDispatch)]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.GuidAttribute("058B416C-FC61-4E64-802A-19070CB39703")]
    [System.ComponentModel.DescriptionAttribute("CapeRealParameterEvents Interface")]
    interface IRealParameterSpecEvents
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
        void ParameterLowerBoundChanged(object sender, object args);

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
        /// <param name = "sender">The <see cref = "RealParameter">RealParameter</see> that raised the event.</param>
        /// <param name = "args">A <see cref = "ParameterValidatedEventArgs">ParameterValidatedEventArgs</see> that contains information about the event.</param>
        void ParameterValidated(object sender, object args);
    }

    /// <summary>
    /// Real-Valued parameter for use in the CAPE-OPEN parameter collection.
    /// </summary>
    [Serializable]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(IParameterEvents), typeof(IRealParameterSpecEvents))]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("77E39C43-046B-4b1f-9EE0-AA9EFC55D2EE")]//ICapeThermoMaterialObject_IID)
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    [System.ComponentModel.TypeConverter(typeof(CapeOpen.RealParameterTypeConverter))]
    public class RealParameter : CapeParameter,
        ICapeParameter,
        ICapeParameterSpec,
        ICapeParameterSpecCOM,
        ICapeRealParameterSpec,
        ICapeRealParameterSpecCOM,
        IATCapeXRealParameterSpec
    {
        private double m_value = 0.0;
        private double m_DefaultValue = 0.0;
        private double m_LowerBound = 0.0;
        private double m_UpperBound = 0.0;
        //private String m_Category = String.Empty;
        private String m_unit = String.Empty;

        /// <summary>
        /// Creates a new instance of the double precision-valued parameter class. 
        /// </summary>
        /// <remarks>
        /// The mode is set to CapeParamMode.CAPE_INPUT_OUTPUT. The dimensionality 
        /// of the parameter is determined from the unit string.
        /// </remarks>
        /// <param name = "name">Sets as the ComponentName of the parameter's ICapeIdentification interface.</param>
        /// <param name = "description">Sets as the ComponentDescription of the parameter's ICapeIdentification interface.</param>
        /// <param name = "value">Sets the inital value of the parameter.</param>
        /// <param name = "defaultValue">Sets the default value of the parameter.</param>
        /// <param name = "lowerBound">Sets the lower bound of the parameter.</param>
        /// <param name = "upperBound">Sets the upper bound of the parameter.</param>
        /// <param name = "mode">Sets the CapeParamMode mode of the parameter.</param>
        /// <param name = "unit">Use to Set the dimensionality of the parameter.</param>
        public RealParameter(System.String name, System.String description, double value, double defaultValue, double lowerBound, double upperBound, CapeParamMode mode, string unit)
            : base(name, description, mode)
        {
            m_value = value;
            m_DefaultValue = defaultValue;
            m_ValStatus = CapeValidationStatus.CAPE_VALID;
            m_unit = unit;
            m_LowerBound = lowerBound;
            m_UpperBound = upperBound;
        }
        /// <summary>
        /// Creates a new instance of the double precision-valued parameter class. 
        /// </summary>
        /// <remarks>
        /// The upper bound is set to Double.MaxValue (1.79769313486232e308) and the 
        /// lower bound is set to Double.MinValue (negative 1.79769313486232e308). 
        /// The mode is set to CapeParamMode.CAPE_INPUT_OUTPUT. 
        /// The dimensionality of the parameter is determined from the unit string.
        /// </remarks>
        /// <param name = "name">Sets as the ComponentName of the parameter's ICapeIdentification interface.</param>
        /// <param name = "description">Sets as the ComponentDescription of the parameter's ICapeIdentification interface.</param>
        /// <param name = "value">Sets the inital value of the parameter.</param>
        /// <param name = "defaultValue">Sets the default value of the parameter.</param>
        /// <param name = "mode">Sets the CapeParamMode mode of the parameter.</param>
        /// <param name = "unit">Use to Set the dimensionality of the parameter.</param>
        public RealParameter(System.String name, System.String description, double value, double defaultValue, CapeParamMode mode, string unit)
            : base(name, description, mode)
        {
            m_value = value;
            m_DefaultValue = defaultValue;
            this.Mode = mode;
            m_ValStatus = CapeValidationStatus.CAPE_VALID;
            m_unit = unit;
            m_LowerBound = Double.MinValue;
            m_UpperBound = Double.MaxValue;
        }

        /// <summary>
        /// Creates a new instance of the double precision-valued parameter class. 
        /// </summary>
        /// <remarks>
        /// The upper bound is set to Double.MaxValue (1.79769313486232e308) and the 
        /// lower bound is set to Double.MinValue (negative 1.79769313486232e308). 
        /// The mode is set to CapeParamMode.CAPE_INPUT_OUTPUT. 
        /// The dimensionality of the parameter is determined from the unit string.
        /// </remarks>
        /// <param name = "name">Sets as the ComponentName of the parameter's ICapeIdentification interface.</param>
        /// <param name = "value">Sets the inital value of the parameter.</param>
        /// <param name = "defaultValue">Sets the default value of the parameter.</param>
        /// <param name = "unit">Use to Set the dimensionality of the parameter.</param>
        public RealParameter(String name, double value, double defaultValue, String unit)
            : base(name, "", CapeParamMode.CAPE_INPUT_OUTPUT)
        {
            m_value = value;
            m_DefaultValue = defaultValue;
            m_ValStatus = CapeValidationStatus.CAPE_VALID;
            m_unit = unit;
            m_LowerBound = Double.MinValue;
            m_UpperBound = Double.MaxValue;
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
            return new RealParameter(this.ComponentName, this.ComponentDescription, this.DimensionedValue, this.DimensionedDefaultValue, this.DimensionedLowerBound, this.DimensionedUpperBound, this.Mode, this.Unit);
        }

        /// <summary>
        /// Gets the dimensionality of the parameter.
        /// </summary>
        /// <remarks>
        /// <para>Gets the dimensionality of the parameter for which this is the 
        /// specification. The dimensionality represents the physical dimensional 
        /// axes of this parameter. It is expected that the dimensionality must cover 
        /// at least 6 fundamental axes (length, mass, time, angle, temperature and 
        /// charge). A possible implementation could consist in being a constant 
        /// length array vector that contains the exponents of each basic SI unit, 
        /// following directives of SI-brochure (from http://www.bipm.fr/). So if we 
        /// agree on order &lt;m kg s A K,&gt; ... velocity would be 
        /// &lt;1,0,-1,0,0,0&gt;: that is m1 * s-1 =m/s. We have suggested to the 
        /// CO Scientific Committee to use the SI base units plus the SI derived units 
        /// with special symbols (for a better usability and for allowing the 
        /// definition of angles).</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        Object ICapeParameterSpecCOM.Dimensionality
        {
            get
            {
                return CDimensions.Dimensionality(m_unit);
            }
        }


        /// <summary>
        /// Gets the dimensionality of the parameter.
        /// </summary>
        /// <remarks>
        /// <para>Gets the dimensionality of the parameter for which this is the 
        /// specification. The dimensionality represents the physical dimensional 
        /// axes of this parameter. It is expected that the dimensionality must cover 
        /// at least 6 fundamental axes (length, mass, time, angle, temperature and 
        /// charge). A possible implementation could consist in being a constant 
        /// length array vector that contains the exponents of each basic SI unit, 
        /// following directives of SI-brochure (from http://www.bipm.fr/). So if we 
        /// agree on order &lt;m kg s A K,&gt; ... velocity would be 
        /// &lt;1,0,-1,0,0,0&gt;: that is m1 * s-1 =m/s. We have suggested to the 
        /// CO Scientific Committee to use the SI base units plus the SI derived units 
        /// with special symbols (for a better usability and for allowing the 
        /// definition of angles).</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        override public double[] Dimensionality
        {
            get
            {
                return CDimensions.Dimensionality(m_unit);
            }
        }
        
        /// <summary>
        /// Gets and sets the value for this Parameter.
        /// </summary>
        /// <remarks>
        /// This value uses the System.Object data type for compatibility with 
        /// COM-based CAPE-OPEN.
        /// </remarks>
        /// <returns>A boxed boolean value of the parameter.</returns>
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
                string message = string.Empty;
                if (this.SIValidate((double)value, ref message))
                {
                    m_value = (double)value;
                    this.NotifyPropertyChanged("Value");
                    OnParameterValueChanged(args);
                }
                else
                {
                    throw new CapeInvalidArgumentException(message, 1);
                }
            }
        }


        /// <summary>
        /// Validates the value sent against the specification of the parameter.
        /// </summary>
        /// <remarks>
        /// The value is considered valid if it is between the upper and lower 
        /// bound of the parameter. The message is used to return the reason that 
        /// the parameter is invalid.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "value">The name of the unit that the value should be converted to.</param>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        bool ICapeRealParameterSpecCOM.Validate(double value, ref String message)
        {
            return this.SIValidate(value, ref message);
        }
        
        /// <summary>
        /// Returns the current value of the parameter in the dimensional unit
        /// specified.
        /// </summary>
        /// <remarks>
        /// The value of the parameter in the unit requested. The unit must be a valid
        /// parameter. For example, if the parameter was set as 101325 Pa, if the desiredUnit was "atm" the return
        /// value would be 1 (the value of 101325 Pa in atmospheres).
        /// </remarks>
        /// <returns>The value of the parameter in the requested unit of measurement.</returns>
        /// <param name = "desiredUnit">The unit that the parameter is desired to be reported in.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised unit identifier.</exception>
        public double ConvertTo(String desiredUnit)
        {
            bool bUnitExisting = false;
            double Factor1 = 1;
            double Factor2 = 0;
            double retVal = m_value;
            if ((desiredUnit != null) || (desiredUnit == String.Empty))
            {
                String[] units = CapeOpen.CDimensions.Units;
                for (int i = 0; i < units.Length; i++)
                {
                    if (units[i] == desiredUnit)
                    {
                        Factor1 = CapeOpen.CDimensions.ConverionsTimes(desiredUnit);
                        Factor2 = CapeOpen.CDimensions.ConverionsPlus(desiredUnit);
                        bUnitExisting = true;
                        break;
                    }
                }
            }
            if (!bUnitExisting)
                System.Windows.Forms.MessageBox.Show("There is no unit named" + desiredUnit, "Unit Warning!");
            else
            {
                retVal = (Convert.ToDouble(retVal) - Factor2) / Factor1;
                m_unit = desiredUnit;
            }
            return retVal;
        }

        /// <summary>
        /// Returns the value in the SI unit of the specified unit.
        /// </summary>
        /// <remarks>
        /// The value is returned in the SI units for the dimensionality of the 
        /// parameter. For example, if the parameter was set as 1 atm, the return
        /// value would be 101325 (the value of 1 atm in the SI pressure 
        /// unit of Pascals, Pa).
        /// </remarks>
        /// <returns>The value of the parameter in SI units.</returns>
        /// <param name = "value">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <param name = "unit">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised unit identifier.</exception>
        public double ConvertToSI(double value, String unit)
        {
            bool bUnitExisting = false;
            double Factor1 = 1;
            double Factor2 = 0;
            double retVal = value;
            if ((unit != null) || (unit == String.Empty))
            {
                String[] units = CapeOpen.CDimensions.Units;
                for (int i = 0; i < units.Length; i++)
                {
                    if (units[i] == unit)
                    {
                        Factor1 = CapeOpen.CDimensions.ConverionsTimes(unit);
                        Factor2 = CapeOpen.CDimensions.ConverionsPlus(unit);
                        bUnitExisting = true;
                        break;
                    }
                }
            }
            if (!bUnitExisting)
                System.Windows.Forms.MessageBox.Show("There is no unit named" + unit, "Unit Warning!");
            else
            {
                retVal = (retVal * Factor1) + Factor2;
            }
            return retVal;
        }


        /// <summary>
        /// Gets and sets the dimensional unit for the parameter.
        /// </summary>
        /// <remarks>
        /// The value is returned in the SI units for the dimensionality of the 
        /// parameter. For example, if the parameter was set as 1 atm, the return
        /// value would be 101325 (the value of 1 atm in the SI pressure 
        /// unit of Pascals, Pa).
        /// </remarks>
        /// <value>The value of the parameter in SI units.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised unit identifier.</exception>
        [System.ComponentModel.Browsable(true)]
        [System.ComponentModel.Category("Value")]
        [System.ComponentModel.TypeConverter(typeof(UnitConverter))]
        [System.ComponentModel.DescriptionAttribute("Dimensional unit of the parameter.")]
        public String Unit
        {
            get
            {
                return m_unit;
            }

            set
            {
                m_unit = value;
                //m_Category = CapeOpen.CDimensions.UnitCategory(m_unit); 
                this.NotifyPropertyChanged("Unit");
            }
        }

        /// <summary>
        /// Occurs when the user changes of the lower bound of the parameter changes.
        /// </summary>
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


        /// <summary>
        /// Gets and sets the parameter value in the SI unit for its current dimensionality.
        /// </summary>
        /// <remarks>
        /// The value is returned in the SI units for the dimensionality of the 
        /// parameter. For example, if the parameter was set as 1 atm, the return
        /// value would be 101325 (the value of 1 atm in the SI pressure 
        /// unit of Pascals, Pa).
        /// </remarks>
        /// <value>The value of the parameter in SI units.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.Browsable(true)]
        [System.ComponentModel.Category("Value")]
        public double SIValue
        {
            get
            {
                return m_value;
            }

            set
            {
                string message = string.Empty;
                if (this.SIValidate(value, ref message))
                {
                    ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_value, value);
                    OnParameterValueChanged(args);
                    m_value = value;
                    this.NotifyPropertyChanged("Value");
                }
            }
        }

        /// <summary>
        /// Gets and sets the parameter value in the current unit for its dimensionality.
        /// </summary>
        /// <remarks>
        /// The value is returned in the parameter's current unit. For example, if the 
        /// parameter was set as 101325 Pa and the unit was changed to atm, 
        /// the return value would be 1 (the value of 101325 Pa in atmospheres).
        /// </remarks>
        /// <value>The value of the parameter in the current unit.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.Browsable(true)]
        [System.ComponentModel.Category("Value")]
        public double DimensionedValue
        {
            get
            {
                return ConvertTo(this.Unit);
            }

            set
            {
                double newValue = this.ConvertToSI(value, this.Unit);
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_value, newValue);
                string message = string.Empty;
                if (this.DimensionedValidate((double)value, ref message))
                {
                    m_value = newValue;
                    this.NotifyPropertyChanged("Value");
                    OnParameterValueChanged(args);
                }
                else
                {
                    throw new CapeInvalidArgumentException(message, 1);
                }
            }
        }

        /// <summary>
        /// Validates the current value of the parameter against the 
        /// specification of the parameter. 
        /// </summary>
        /// <remarks>
        /// The parameter is considered valid if the current value is 
        /// between the upper and lower bound. The message is used to 
        /// return the reason that the parameter is invalid. This function also
        /// sets the CapeValidationStatus of the parameter based upon the results
        /// of the validation.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public override bool Validate(ref String message)
        {
            message = "Value is valid.";
            ParameterValidatedEventArgs args = new ParameterValidatedEventArgs(this.ComponentName, message, m_ValStatus, CapeValidationStatus.CAPE_VALID);            
            bool retval = true;
            if (m_value < m_LowerBound)
            {
                message = "Value below the Lower Bound.";
                args = new ParameterValidatedEventArgs(this.ComponentName, message, m_ValStatus, CapeValidationStatus.CAPE_INVALID);
                m_ValStatus = CapeValidationStatus.CAPE_INVALID;
                this.NotifyPropertyChanged("ValStatus");
                OnParameterValidated(args);
                retval = false;
            }
            if (m_value > m_UpperBound)
            {
                message = "Value greater than upper bound.";
                args = new ParameterValidatedEventArgs(this.ComponentName, message, m_ValStatus, CapeValidationStatus.CAPE_INVALID);
                m_ValStatus = CapeValidationStatus.CAPE_INVALID;
                this.NotifyPropertyChanged("ValStatus");
                OnParameterValidated(args);
                retval = false;
            }
            OnParameterValidated(args);
            m_ValStatus = CapeValidationStatus.CAPE_VALID;
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
            m_value = m_DefaultValue;
            this.NotifyPropertyChanged("Value");
            OnParameterReset(args);
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
        [System.ComponentModel.Browsable(false)]
        [System.ComponentModel.Category("ICapeParameterSpec")]
        public override CapeParamType Type
        {
            get
            {
                return CapeParamType.CAPE_REAL;
            }
        }

        //ICapeRealParameterSpec

        /// <summary>
        /// Gets and sets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// The default value is the value that the specification is set to after
        /// the Reset() method is called.
        /// </remarks>
        /// <value>The parameter type. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        double ICapeRealParameterSpecCOM.DefaultValue
        {
            get
            {
                return this.m_DefaultValue;
            }
        }

        /// <summary>
        /// Gets and sets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// The default value is the value that the specification is set to after
        /// the Reset() method is called.
        /// </remarks>
        /// <value>The parameter type. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.Category("Parameter Specification")]
        public double DimensionedDefaultValue
        {
            get
            {
                bool bUnitExisting = false;
                double Factor1 = 1;
                double Factor2 = 0;
                double retVal = m_DefaultValue;
                String[] units = CapeOpen.CDimensions.Units;
                for (int i = 0; i < units.Length; i++)
                {
                    if (units[i] == this.Unit)
                    {
                        Factor1 = CapeOpen.CDimensions.ConverionsTimes(this.Unit);
                        Factor2 = CapeOpen.CDimensions.ConverionsPlus(this.Unit);
                        bUnitExisting = true;
                        break;
                    }
                }
                retVal = (Convert.ToDouble(retVal) - Factor2) / Factor1;
                if (!bUnitExisting)
                    System.Windows.Forms.MessageBox.Show("There is no unit named" + this.Unit, "Unit Warning!");
                return retVal;
            }
            set
            {
                ParameterDefaultValueChangedEventArgs args = new ParameterDefaultValueChangedEventArgs(this.ComponentName, m_DefaultValue, this.ConvertToSI(value, this.Unit));
                m_DefaultValue = this.ConvertToSI(value, this.Unit);
                this.NotifyPropertyChanged("DefaultValue");
                OnParameterDefaultValueChanged(args);
            }
        }

        /// <summary>
        /// Gets and sets the lower bound of the parameter. 
        /// </summary>
        /// <remarks>
        /// The lower bound can be a valid double precision value. 
        /// By default, it is set to Double.MinValue, negative 1.79769313486232e308.
        /// </remarks>			
        /// <value>The parameter type. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.Category("Parameter Specification")]
        public double DimensionedLowerBound
        {
            get
            {
                if (m_LowerBound == Double.MinValue) return Double.MinValue;
                bool bUnitExisting = false;
                double Factor1 = 1;
                double Factor2 = 0;
                double retVal = m_LowerBound;
                String[] units = CapeOpen.CDimensions.Units;
                for (int i = 0; i < units.Length; i++)
                {
                    if (units[i] == this.Unit)
                    {
                        Factor1 = CapeOpen.CDimensions.ConverionsTimes(this.Unit);
                        Factor2 = CapeOpen.CDimensions.ConverionsPlus(this.Unit);
                        bUnitExisting = true;
                        break;
                    }
                }
                retVal = (Convert.ToDouble(retVal) - Factor2) / Factor1;
                if (!bUnitExisting)
                    System.Windows.Forms.MessageBox.Show("There is no unit named" + this.Unit, "Unit Warning!");
                return retVal;
            }
            set
            {
                ParameterLowerBoundChangedEventArgs args;
                if (value == Double.MinValue)
                {
                    args = new ParameterLowerBoundChangedEventArgs(this.ComponentName, m_UpperBound, Double.MinValue);
                    m_LowerBound = Double.MaxValue;
                    this.NotifyPropertyChanged("LowerBound");
                    OnParameterLowerBoundChanged(args);
                    return;
                }
                args = new ParameterLowerBoundChangedEventArgs(this.ComponentName, m_UpperBound, this.ConvertToSI(value, this.Unit));
                m_LowerBound = this.ConvertToSI(value, this.Unit);
                this.NotifyPropertyChanged("LowerBound");
                OnParameterLowerBoundChanged(args);
            }
        }

        /// <summary>
        /// Gets and sets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// The default value is the value that the specification is set to after
        /// the Reset() method is called.
        /// </remarks>
        /// <value>The parameter type. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.Category("Parameter Specification")]
        public double SIDefaultValue
        {
            get
            {
                return m_DefaultValue;
            }
            set
            {
                ParameterDefaultValueChangedEventArgs args = new ParameterDefaultValueChangedEventArgs(this.ComponentName, m_DefaultValue, this.ConvertToSI(value, this.Unit));
                m_DefaultValue = this.ConvertToSI(value, this.SIUnit);
                this.NotifyPropertyChanged("DefaultValue");
                OnParameterDefaultValueChanged(args);
            }
        }

        /// <summary>
        /// Gets and sets the lower bound of the parameter. 
        /// </summary>
        /// <remarks>
        /// The lower bound can be a valid double precision value. 
        /// By default, it is set to Double.MinValue, negative 1.79769313486232e308.
        /// </remarks>			
        /// <value>The parameter type. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.Category("Parameter Specification")]
        public double SILowerBound
        {
            get
            {                
                return m_LowerBound;
            }
            set
            {
                ParameterLowerBoundChangedEventArgs args;
                if (value == Double.MinValue)
                {
                    args = new ParameterLowerBoundChangedEventArgs(this.ComponentName, m_UpperBound, Double.MinValue);
                    m_LowerBound = Double.MaxValue;
                    this.NotifyPropertyChanged("LowerBound");
                    OnParameterLowerBoundChanged(args);
                    return;
                }
                args = new ParameterLowerBoundChangedEventArgs(this.ComponentName, m_UpperBound, value);
                m_LowerBound = this.ConvertToSI(value, this.SIUnit);
                this.NotifyPropertyChanged("LowerBound");
                OnParameterLowerBoundChanged(args);
            }
        }

        /// <summary>
        /// Gets and sets the lower bound of the parameter. 
        /// </summary>
        /// <remarks>
        /// The lower bound can be a valid double precision value. 
        /// By default, it is set to Double.MaxValue, 
        /// 1.79769313486232e308.
        /// </remarks>			
        /// <value>The upper bound for the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        double ICapeRealParameterSpecCOM.LowerBound
        {
            get
            {
                return this.m_LowerBound;
            }
        }

        /// <summary>
        /// Gets and sets the upper bound of the parameter. 
        /// </summary>
        /// <remarks>
        /// The lower bound can be a valid double precision value. 
        /// By default, it is set to Double.MaxValue, 
        /// 1.79769313486232e308.
        /// </remarks>			
        /// <value>The upper bound for the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.Category("Parameter Specification")]
        public double DimensionedUpperBound
        {
            get
            {
                bool bUnitExisting = false;
                double Factor1 = 1;
                double Factor2 = 0;
                double retVal = m_UpperBound;
                String[] units = CapeOpen.CDimensions.Units;
                for (int i = 0; i < units.Length; i++)
                {
                    if (units[i] == this.Unit)
                    {
                        Factor1 = CapeOpen.CDimensions.ConverionsTimes(this.Unit);
                        Factor2 = CapeOpen.CDimensions.ConverionsPlus(this.Unit);
                        bUnitExisting = true;
                        break;
                    }
                }
                retVal = (Convert.ToDouble(retVal) - Factor2) / Factor1;
                if (!bUnitExisting)
                    System.Windows.Forms.MessageBox.Show("There is no unit named" + this.Unit, "Unit Warning!");
                return retVal;

            }
            set
            {
                ParameterUpperBoundChangedEventArgs args;
                if (value == Double.MaxValue)
                {
                    args = new ParameterUpperBoundChangedEventArgs(this.ComponentName, m_UpperBound, Double.MaxValue);
                    m_UpperBound = Double.MaxValue;
                    this.NotifyPropertyChanged("UpperBound");
                    OnParameterUpperBoundChanged(args);
                    return;
                }
                args = new ParameterUpperBoundChangedEventArgs(this.ComponentName, m_UpperBound, this.ConvertToSI(value, this.Unit));
                m_UpperBound = this.ConvertToSI(value, this.Unit);
                this.NotifyPropertyChanged("UpperBound");
                OnParameterUpperBoundChanged(args);
            }
        }
        /// <summary>
        /// Gets and sets the upper bound of the parameter. 
        /// </summary>
        /// <remarks>
        /// The lower bound can be a valid double precision value. 
        /// By default, it is set to Double.MaxValue, 
        /// 1.79769313486232e308.
        /// </remarks>			
        /// <value>The upper bound for the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.Category("Parameter Specification")]
        public double SIUpperBound
        {
            get
            {                
                return m_UpperBound;
            }
            set
            {
                ParameterUpperBoundChangedEventArgs args;
                if (value == Double.MaxValue)
                {
                    args = new ParameterUpperBoundChangedEventArgs(this.ComponentName, m_UpperBound, Double.MaxValue);
                    m_UpperBound = Double.MaxValue;
                    this.NotifyPropertyChanged("UpperBound");
                    OnParameterUpperBoundChanged(args);
                    return;
                }
                args = new ParameterUpperBoundChangedEventArgs(this.ComponentName, m_UpperBound, value);
                m_UpperBound = value;
                this.NotifyPropertyChanged("UpperBound");
                OnParameterUpperBoundChanged(args);
            }
        }

        /// <summary>
        /// Gets and sets the upper bound of the parameter. 
        /// </summary>
        /// <remarks>
        /// The lower bound can be a valid double precision value. 
        /// By default, it is set to Double.MaxValue, 
        /// 1.79769313486232e308.
        /// </remarks>			
        /// <value>The upper bound for the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        double ICapeRealParameterSpecCOM.UpperBound
        {
            get
            {
                return this.m_UpperBound;
            }
        }

        /// <summary>
        /// Validates the SI value sent against the specification of the parameter.
        /// </summary>
        /// <remarks>
        /// The value, in the SI units of measurement appropriate to the <see cref="unitCategory"/> of the parameter
        /// is considered valid if it is between the upper and lower bound of the parameter. The message is used to 
        /// return the reason that the parameter is invalid.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "value">The name of the unit that the value should be converted to.</param>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public bool SIValidate(double value, ref String message)
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

        /// <summary>
        /// Validates the dimensioned value sent against the specification of the parameter.
        /// </summary>
        /// <remarks>
        /// <para>The value sent in is converted from its dimensioned value using the current unit of the parameter 
        /// to an SI value and is then validated against the parameter specification.</para>
        /// <para>The value is considered valid if it is between the upper and lower 
        /// bound of the parameter. The message is used to return the reason that 
        /// the parameter is invalid.</para>
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "value">The name of the unit that the value should be converted to.</param>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public bool DimensionedValidate(double value, ref String message)
        {
            double testVal = this.ConvertToSI(value, this.Unit);
            if (testVal < m_LowerBound)
            {
                message = "Value below the Lower Bound.";
                return false;
            }
            if (testVal > m_UpperBound)
            {
                message = "Value greater than upper bound.";
                return false;
            }
            message = "Value is valid.";
            return true;
        }
        
        /// <summary>
        /// Gets the SI unit for the <see cref="unitCategory"/> of the parameter.
        /// </summary>
        /// <remarks>
        /// <para>
        /// Provides the SI unit for the parameter. Parameter values are stored in their SI unit of measure for the 
        /// <see cref="unitCategory"/>. This provides the user with a mechanism to determine the SI unit of measure 
        /// used for the current parameter.
        /// </para>
        /// </remarks>
        /// <value>
        /// Defines the SI unit for the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.ComponentModel.Category("Value")]
        [System.ComponentModel.DescriptionAttribute(" Provide the Aspen Plus display units for for this parameter.")]
        [System.ComponentModel.Browsable(true)]
        [System.ComponentModel.TypeConverter(typeof(System.ComponentModel.StringConverter))]
        public String SIUnit
        {
            get
            {
                return CapeOpen.CDimensions.FindSIUnit(m_unit);
            }
        }
        /// <summary>
        /// Gets the Aspen display unit for the parameter.
        /// </summary>
        /// <remarks>
        /// <para>DisplayUnits defines the unit of measurement symbol for a parameter.</para>
        /// <para>Note: The symbol must be one of the uppercase strings recognized by Aspen
        /// Plus to ensure that it can perform unit of measurement conversions on the 
        /// parameter value. The system converts the parameter's value from SI units for
        /// display in the data browser and converts updated values back into SI.
        /// </para>
        /// </remarks>
        /// <value>
        /// Defines the display unit for the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.ComponentModel.Category("Dimensionality")]
        [System.ComponentModel.DescriptionAttribute(" Provide the Aspen Plus display units for for this parameter.")]
        [System.ComponentModel.Browsable(false)]
        String IATCapeXRealParameterSpec.DisplayUnits
        {
            get
            {
                return CapeOpen.CDimensions.AspenUnit(m_unit);
            }
        }

    };


    /// <summary>
    /// Real-Valued parameter for use in the CAPE-OPEN parameter collection.
    /// </summary>
    /// <remarks>
    /// Real-Valued parameter for use in the CAPE-OPEN parameter collection.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(IRealParameterSpecEvents))]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("C7095FE4-E61D-4FFF-BA02-013FD38DBAE9")]//ICapeThermoMaterialObject_IID)
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    [System.ComponentModel.TypeConverter(typeof(RealParameterTypeConverter))]
    class RealParameterWrapper : CapeParameter,
        ICapeParameter,
        ICapeParameterSpec,
        ICapeRealParameterSpec,
        ICloneable,
        IATCapeXRealParameterSpec
    {
        private String m_unit = String.Empty;
        [NonSerialized]
        private ICapeParameter m_parameter = null;



        /// <summary>
        /// Creates a new instance of the double precision-valued parameter class. 
        /// </summary>
        /// <remarks>
        /// The mode is set to CapeParamMode.CAPE_INPUT_OUTPUT. The dimensionality 
        /// of the parameter is determined from the unit string.
        /// </remarks>
        /// <param name = "parameter">Use to Set the dimensionality of the parameter.</param>
        public RealParameterWrapper(ICapeParameter parameter)
            : base(((ICapeIdentification)parameter).ComponentName, ((ICapeIdentification)parameter).ComponentDescription, parameter.Mode)
        {
            m_parameter = parameter;
            object dims = ((ICapeParameterSpecCOM)parameter.Specification).Dimensionality;
            if (typeof(int[]).IsAssignableFrom(dims.GetType()))
            {
                int[] dimVals = (int[])dims;
                m_unit = CDimensions.SIUnit(dimVals);
            }
            else if (typeof(double[]).IsAssignableFrom(dims.GetType()))
            {
                double[] dimVals = (double[])dims;
                m_unit = CDimensions.SIUnit(dimVals);
            }
        }

        /// <summary>
        /// Gets the dimensionality of the parameter.
        /// </summary>
        /// <remarks>
        /// <para>Gets the dimensionality of the parameter for which this is the 
        /// specification. The dimensionality represents the physical dimensional 
        /// axes of this parameter. It is expected that the dimensionality must cover 
        /// at least 6 fundamental axes (length, mass, time, angle, temperature and 
        /// charge). A possible implementation could consist in being a constant 
        /// length array vector that contains the exponents of each basic SI unit, 
        /// following directives of SI-brochure (from http://www.bipm.fr/). So if we 
        /// agree on order &lt;m kg s A K,&gt; ... velocity would be 
        /// &lt;1,0,-1,0,0,0&gt;: that is m1 * s-1 =m/s. We have suggested to the 
        /// CO Scientific Committee to use the SI base units plus the SI derived units 
        /// with special symbols (for a better usability and for allowing the 
        /// definition of angles).</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        double[] ICapeParameterSpec.Dimensionality
        {
            get
            {
                return CDimensions.Dimensionality(m_unit);
            }
        }

        /// <summary>
        /// Gets and sets the value for this Parameter.
        /// </summary>
        /// <remarks>
        /// This value uses the System.Object data type for compatibility with 
        /// COM-based CAPE-OPEN.
        /// </remarks>
        /// <returns>A boxed boolean value of the parameter.</returns>
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
                string message = string.Empty;
                if (((ICapeRealParameterSpecCOM)this.m_parameter).Validate((double)value, ref message))
                {
                    m_parameter.value = value;
                    this.NotifyPropertyChanged("Value");
                    OnParameterValueChanged(args);
                }
                else
                {
                    throw new CapeInvalidArgumentException(message, 1);
                }
            }
        }

        /// <summary>
        /// Returns the current value of the parameter in the dimensional unit
        /// specified.
        /// </summary>
        /// <remarks>
        /// The value of the parameter in the unit requested. The unit must be a valid
        /// parameter. For example, if the parameter was set as 101325 Pa, if the desiredUnit was "atm" the return
        /// value would be 1 (the value of 101325 Pa in atmospheres).
        /// </remarks>
        /// <value>The value of the parameter in the requested unit of measurement.</value>
        /// <param name = "desiredUnit">The unit that the parameter is desired to be reported in.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised unit identifier.</exception>
        public double ConvertTo(String desiredUnit)
        {
            bool bUnitExisting = false;
            double Factor1 = 1;
            double Factor2 = 0;
            double retVal = (double)m_parameter.value;
            if ((desiredUnit != null) || (desiredUnit == String.Empty))
            {
                String[] units = CapeOpen.CDimensions.Units;
                for (int i = 0; i < units.Length; i++)
                {
                    if (units[i] == desiredUnit)
                    {
                        Factor1 = CapeOpen.CDimensions.ConverionsTimes(desiredUnit);
                        Factor2 = CapeOpen.CDimensions.ConverionsPlus(desiredUnit);
                        bUnitExisting = true;
                        break;
                    }
                }
            }
            if (!bUnitExisting)
                System.Windows.Forms.MessageBox.Show("There is no unit named" + desiredUnit, "Unit Warning!");
            else
            {
                retVal = (Convert.ToDouble(retVal) - Factor2) / Factor1;
                m_unit = desiredUnit;
            }
            return retVal;
        }

        /// <summary>
        /// Returns the value in the SI unit of the specified unit.
        /// </summary>
        /// <remarks>
        /// The value is returned in the SI units for the dimensionality of the 
        /// parameter. For example, if the parameter was set as 1 atm, the return
        /// value would be 101325 (the value of 1 atm in the SI pressure 
        /// unit of Pascals, Pa).
        /// </remarks>
        /// <value>The value of the parameter in SI units.</value>
        /// <param name = "value">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <param name = "unit">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised unit identifier.</exception>
        public double ConvertToSI(double value, String unit)
        {
            bool bUnitExisting = false;
            double Factor1 = 1;
            double Factor2 = 0;
            double retVal = (double)m_parameter.value;
            if ((unit != null) || (unit == String.Empty))
            {
                String[] units = CapeOpen.CDimensions.Units;
                for (int i = 0; i < units.Length; i++)
                {
                    if (units[i] == unit)
                    {
                        Factor1 = CapeOpen.CDimensions.ConverionsTimes(unit);
                        Factor2 = CapeOpen.CDimensions.ConverionsPlus(unit);
                        bUnitExisting = true;
                        break;
                    }
                }
            }
            if (!bUnitExisting)
                System.Windows.Forms.MessageBox.Show("There is no unit named" + unit, "Unit Warning!");
            else
            {
                retVal = (retVal * Factor1) + Factor2;
            }
            return retVal;
        }


        /// <summary>
        /// Returns the value in the SI unit of the specified unit.
        /// </summary>
        /// <remarks>
        /// The value is returned in the SI units for the dimensionality of the 
        /// parameter. For example, if the parameter was set as 1 atm, the return
        /// value would be 101325 (the value of 1 atm in the SI pressure 
        /// unit of Pascals, Pa).
        /// </remarks>
        /// <value>The value of the parameter in SI units.</value>
        /// <param name = "value">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <param name = "unit">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised unit identifier.</exception>
        public double ConvertToDimensioned(double value, String unit)
        {
            bool bUnitExisting = false;
            double Factor1 = 1;
            double Factor2 = 0;
            double retVal = (double)m_parameter.value;
            if ((unit != null) || (unit == String.Empty))
            {
                String[] units = CapeOpen.CDimensions.Units;
                for (int i = 0; i < units.Length; i++)
                {
                    if (units[i] == unit)
                    {
                        Factor1 = CapeOpen.CDimensions.ConverionsTimes(unit);
                        Factor2 = CapeOpen.CDimensions.ConverionsPlus(unit);
                        bUnitExisting = true;
                        break;
                    }
                }
            }
            if (!bUnitExisting)
                System.Windows.Forms.MessageBox.Show("There is no unit named" + unit, "Unit Warning!");
            else
            {
                retVal = (retVal - Factor2)/Factor1;
            }
            return retVal;
        }
        /// <summary>
        /// Gets and sets the dimensional unit for the parameter.
        /// </summary>
        /// <remarks>
        /// The value is returned in the SI units for the dimensionality of the 
        /// parameter. For example, if the parameter was set as 1 atm, the return
        /// value would be 101325 (the value of 1 atm in the SI pressure 
        /// unit of Pascals, Pa).
        /// </remarks>
        /// <value>The value of the parameter in SI units.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised unit identifier.</exception>
        [System.ComponentModel.Browsable(true)]
        [System.ComponentModel.Category("Value")]
        [System.ComponentModel.TypeConverter(typeof(UnitConverter))]
        [System.ComponentModel.DescriptionAttribute("Dimensional unit of the parameter.")]
        public String Unit
        {
            get
            {
                return m_unit;
            }

            set
            {
                m_unit = value;
                //m_Category = CapeOpen.CDimensions.UnitCategory(m_unit); 
                this.NotifyPropertyChanged("Unit");
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
            return new RealParameterWrapper(m_parameter);
        }

        /// <summary>
        /// Occurs when the user changes of the lower bound of the parameter changes.
        /// </summary>
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


        /// <summary>
        /// Gets and sets the parameter value in the SI unit for its current dimensionality.
        /// </summary>
        /// <remarks>
        /// The value is returned in the SI units for the dimensionality of the 
        /// parameter. For example, if the parameter was set as 1 atm, the return
        /// value would be 101325 (the value of 1 atm in the SI pressure 
        /// unit of Pascals, Pa).
        /// </remarks>
        /// <value>The value of the parameter in SI units.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.Browsable(true)]
        [System.ComponentModel.Category("Value")]
        public double SIValue
        {
            get
            {
                return (double)m_parameter.value;
            }

            set
            {
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_parameter.value, value);
                OnParameterValueChanged(args);
                m_parameter.value = value;
                this.NotifyPropertyChanged("Value");
            }
        }

        /// <summary>
        /// Gets and sets the parameter value in the current unit for its dimensionality.
        /// </summary>
        /// <remarks>
        /// The value is returned in the parameter's current unit. For example, if the 
        /// parameter was set as 101325 Pa and the unit was changed to atm, 
        /// the return value would be 1 (the value of 101325 Pa in atmospheres).
        /// </remarks>
        /// <value>The value of the parameter in the current unit.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.Browsable(true)]
        [System.ComponentModel.Category("Value")]
        public double DimensionedValue
        {
            get
            {
                return ConvertTo(this.Unit);
            }

            set
            {
                double newValue = this.ConvertToSI(value, this.Unit);
                ParameterValueChangedEventArgs args = new ParameterValueChangedEventArgs(this.ComponentName, m_parameter.value, newValue);
                string message = string.Empty;
                if (((ICapeRealParameterSpecCOM)this.m_parameter).Validate((double)value, ref message))
                {
                    m_parameter.value = newValue;
                    this.NotifyPropertyChanged("Value");
                    OnParameterValueChanged(args);
                }
                else
                {
                    throw new CapeInvalidArgumentException(message, 1);
                }
            }
        }

        /// <summary>
        /// Validates the current value of the parameter against the 
        /// specification of the parameter. 
        /// </summary>
        /// <remarks>
        /// The parameter is considered valid if the current value is 
        /// between the upper and lower bound. The message is used to 
        /// return the reason that the parameter is invalid. This function also
        /// sets the CapeValidationStatus of the parameter based upon the results
        /// of the validation.
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
            this.NotifyPropertyChanged("Value");
            OnParameterReset(args);
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
        [System.ComponentModel.Browsable(false)]
        [System.ComponentModel.Category("ICapeParameterSpec")]
        public override CapeParamType Type
        {
            get
            {
                return CapeParamType.CAPE_REAL;
            }
        }

        //ICapeRealParameterSpec

        /// <summary>
        /// Gets and sets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// The default value is the value that the specification is set to after
        /// the Reset() method is called.
        /// </remarks>
        /// <value>The parameter type. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        public double DimensionedDefaultValue
        {
            get
            {
                return this.ConvertToDimensioned(((ICapeRealParameterSpecCOM)m_parameter.Specification).DefaultValue, this.Unit);
            }
            set
            {
            }
        }

        /// <summary>
        /// Gets and sets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// The default value is the value that the specification is set to after
        /// the Reset() method is called.
        /// </remarks>
        /// <value>The parameter type. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.Category("Parameter Specification")]
        public double SIDefaultValue
        {
            get
            {
                return (double)((ICapeRealParameterSpecCOM)m_parameter.Specification).DefaultValue;
            }
            set
            {
            }
        }

        /// <summary>
        /// Gets and sets the lower bound of the parameter in SI units corresponding to the <see cref="unitCategory"/> of 
        /// the parameter.
        /// </summary>
        /// <remarks>
        /// The lower bound can be a valid double precision value. 
        /// By default, it is set to Double.MinValue, negative 1.79769313486232e308.
        /// </remarks>			
        /// <value>The parameter type. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.Category("Parameter Specification")]
        public double SILowerBound
        {
            get
            {
                return (double)((ICapeRealParameterSpecCOM)m_parameter.Specification).LowerBound;
            }
            set
            {
            }
        }

        /// <summary>
        /// Gets and sets the lower bound of the parameter. 
        /// </summary>
        /// <remarks>
        /// The lower bound can be a valid double precision value. 
        /// By default, it is set to Double.MaxValue, 
        /// 1.79769313486232e308.
        /// </remarks>			
        /// <value>The upper bound for the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        public double DimensionedLowerBound
        {
            get
            {
                return this.ConvertToDimensioned((double)((ICapeRealParameterSpecCOM)m_parameter.Specification).LowerBound, this.Unit);
            }
            set
            {
            }
        }

        /// <summary>
        /// Gets and sets the upper bound of the parameter. 
        /// </summary>
        /// <remarks>
        /// The lower bound can be a valid double precision value. 
        /// By default, it is set to Double.MaxValue, 
        /// 1.79769313486232e308.
        /// </remarks>			
        /// <value>The upper bound for the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.Category("Parameter Specification")]
        public double SIUpperBound
        {
            get
            {
                return (double)((ICapeRealParameterSpecCOM)m_parameter.Specification).UpperBound;
            }
            set
            {
            }
        }

        /// <summary>
        /// Gets and sets the upper bound of the parameter. 
        /// </summary>
        /// <remarks>
        /// The lower bound can be a valid double precision value. 
        /// By default, it is set to Double.MaxValue, 
        /// 1.79769313486232e308.
        /// </remarks>			
        /// <value>The upper bound for the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        public double DimensionedUpperBound
        {
            get
            {
                return this.ConvertToDimensioned((double)((ICapeRealParameterSpecCOM)m_parameter.Specification).UpperBound, this.Unit);
            }
            set
            {
            }
        }

        /// <summary>
        /// Validates the value sent against the specification of the parameter.
        /// </summary>
        /// <remarks>
        /// The value is considered valid if it is between the upper and lower 
        /// bound of the parameter. The message is used to return the reason that 
        /// the parameter is invalid.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "value">The name of the unit that the value should be converted to.</param>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public bool SIValidate(double value, ref String message)
        {
            return ((ICapeRealParameterSpecCOM)m_parameter.Specification).Validate(value, message);
        }
        
        /// <summary>
        /// Validates the value sent against the specification of the parameter.
        /// </summary>
        /// <remarks>
        /// The value is considered valid if it is between the upper and lower 
        /// bound of the parameter. The message is used to return the reason that 
        /// the parameter is invalid.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "value">The name of the unit that the value should be converted to.</param>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        public bool DimensionedValidate(double value, ref String message)
        {
            return ((ICapeRealParameterSpecCOM)m_parameter.Specification).Validate(this.ConvertToSI(value, this.Unit), message);
        }

        /// <summary>
        /// Gets the Aspen display unit for the parameter.
        /// </summary>
        /// <remarks>
        /// <para>DisplayUnits defines the unit of measurement symbol for a parameter.</para>
        /// <para>Note: The symbol must be one of the uppercase strings recognized by Aspen
        /// Plus to ensure that it can perform unit of measurement conversions on the 
        /// parameter value. The system converts the parameter's value from SI units for
        /// display in the data browser and converts updated values back into SI.
        /// </para>
        /// </remarks>
        /// <value>
        /// Defines the display unit for the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.ComponentModel.Category("Value")]
        [System.ComponentModel.DescriptionAttribute(" Provide the Aspen Plus display units for for this parameter.")]
        [System.ComponentModel.Browsable(true)]
        [System.ComponentModel.TypeConverter(typeof(System.ComponentModel.StringConverter))]
        public String SIUnit
        {
            get
            {
                return CapeOpen.CDimensions.FindSIUnit(m_unit);
            }
        }
        /// <summary>
        /// Gets the Aspen display unit for the parameter.
        /// </summary>
        /// <remarks>
        /// <para>DisplayUnits defines the unit of measurement symbol for a parameter.</para>
        /// <para>Note: The symbol must be one of the uppercase strings recognized by Aspen
        /// Plus to ensure that it can perform unit of measurement conversions on the 
        /// parameter value. The system converts the parameter's value from SI units for
        /// display in the data browser and converts updated values back into SI.
        /// </para>
        /// </remarks>
        /// <value>
        /// Defines the display unit for the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.ComponentModel.Category("Dimensionality")]
        [System.ComponentModel.DescriptionAttribute(" Provide the Aspen Plus display units for for this parameter.")]
        [System.ComponentModel.Browsable(false)]
        public String DisplayUnits
        {
            get
            {
                return CapeOpen.CDimensions.AspenUnit(m_unit);
            }
        }
    };
}

