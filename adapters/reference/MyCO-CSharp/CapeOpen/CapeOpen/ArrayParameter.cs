using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{
    class ArrayParameter
    {
    }


    /// <summary>
    /// Wrapper for an array-valued parameter for use in a CAPE-OPEN <see cref="ParameterCollection">parameter collection</see>.
    /// </summary>
    /// <remarks>
    /// Wraps a CAPE-OPEN array-valued parameter for use in a CAPE-OPEN <see cref="ParameterCollection">parameter collection</see>.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(IRealParameterSpecEvents))]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("277E2E39-70E7-4FBA-89C9-2A19B9D5E576")]//ICapeThermoMaterialObject_IID)
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    class ArrayParameterWrapper : CapeParameter,
        ICapeParameter,
        ICapeParameterSpec,
        ICapeArrayParameterSpec
    {

        [NonSerialized]
        private ICapeParameter m_parameter = null;

        /// <summary>
        /// Creates a new instance of a wrapper class for COM-based array-valued parameter class. 
        /// </summary>
        /// <remarks>
        /// The COM-based array parameter is wrapped and exposed to .NET-based PME and PMCs.
        /// </remarks>
        /// <param name = "parameter">The COM-based array parameter to be wrapped.</param>
        public ArrayParameterWrapper(ICapeParameter parameter)
            : base(((ICapeIdentification)parameter).ComponentName, ((ICapeIdentification)parameter).ComponentDescription, parameter.Mode)
        {
            m_parameter = parameter;
        }

        // ICloneable
        /// <summary>
        /// Creates a copy of the parameter. Both copies refer to the same COM-based array parameter.
        /// </summary>
        /// <remarks><para>The clone method is used to create a copy of the parameter. Both the original object and 
        /// the clone wrap the same instance of the wrapped parameter.</para>
        /// </remarks>
        /// <returns>A copy of the current parameter.</returns>
        override public Object Clone()
        {
            return new ArrayParameterWrapper(m_parameter);
        }

        /// <summary>
        /// Validates the current value of the parameter against the specification of the parameter. 
        /// </summary>        
        /// <remarks>
        /// The wrapped parameter validates iteself against its internal valication criteria.
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
        /// Gets the <see cref = "CapeParamType"/> of the parameter.
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
                return CapeParamType.CAPE_ARRAY;
            }
        }

        //ICapeArrayParameterSpec

        /// <summary>
        /// Gets an array of parameter specifications.
        /// </summary>
        /// <remarks>
        /// Gets an array of the specifications of each of the items in the value of a parameter. The Get method 
        /// returns an array of interfaces to the correct specification type (<see cref=" ICapeRealParameterSpec"/>, 
        /// <see cref=" ICapeOptionParameterSpec"/>, <see cref="ICapeIntegerParameterSpec"/>, or <see cref="ICapeBooleanParameterSpec"/>).
        /// Note that it is also possible, for example, to configure an array of arrays, which would a 
        /// similar but not identical concept to a two-dimensional matrix.
        /// </remarks>
        /// <value>An array of parameter specifications. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        object[] ICapeArrayParameterSpec.ItemsSpecifications
        {
            get
            {
                return (object[])((ICapeArrayParameterSpec)m_parameter.Specification).ItemsSpecifications;
            }
        }

        /// <summary>
        /// Gets the number of dimensions of the array value in the parameter.
        /// </summary>
        /// <remarks>
        /// The number of dimensions of the array value in the parameter.
        /// </remarks>
        /// <value>The number of dimensions of the array value in the parameter. </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.Category("Parameter Specification")]
        int ICapeArrayParameterSpec.NumDimensions
        {
            get
            {
                return ((ICapeArrayParameterSpec)m_parameter.Specification).NumDimensions;
            }
        }

        /// <summary>
        /// Gets the size of each one of the dimensions of the array.
        /// </summary>
        /// <remarks>
        /// Gets the size of each one of the dimensions of the array.
        /// </remarks>			
        /// <value>An integer array containing the size of each one of the dimensions of the array.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed.</exception>
        [System.ComponentModel.Category("Parameter Specification")]
        int[] ICapeArrayParameterSpec.Size
        {
            get
            {
                return (int[])((ICapeArrayParameterSpec)m_parameter.Specification).Size;
            }
        }

        /// <summary>
        /// Determines whether a value is valid for the wrapped parameter.
        /// </summary>
        /// <remarks>
        /// <para>Validates an array against the parameter's specification. It returns a flag to indicate the success or 
        /// failure of the validation together with a text message which can be used to convey the reasoning to 
        /// the client/user.</para>
        /// <para>The wrapped parameter validates the value against its internal validation criteria.</para>
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "value">The value to be checked.</param>
        /// <param name = "messages">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        object ICapeArrayParameterSpec.Validate(object value, ref string[] messages)
        {
            return ((ICapeArrayParameterSpec)m_parameter.Specification).Validate(value, ref messages);
        }
    };
}
