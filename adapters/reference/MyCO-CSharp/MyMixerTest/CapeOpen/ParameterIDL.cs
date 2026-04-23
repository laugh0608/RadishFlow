using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

//#ifndef _PARAMETER_IDL_
//#define _PARAMETER_IDL_

/* IMPORTANT NOTICE
(c) The CAPE-OPEN Laboratory Network, 2002.
All rights are reserved unless specifically stated otherwise

Visit the web site at www.colan.org

This file has been edited using the editor from Microsoft Visual Studio 6.0
This file can viewed properly with any basic editors and browsers (validation done under MS Windows and Unix)
*/

// This file was developed/modified by JEAN-PIERRE BELAUD for CO-LaN organisation - March 2003
namespace CapeOpen
{

    /// <summary>
    /// Gets the type of the parameter for which this is a specification: 
    /// </summary>
    /// <remarks>
    ///    double-precision Real (CAPE_REAL),
    ///    integer(CAPE_INT),
    ///    String (or option)(CAPE_OPTION), 
    ///    boolean(CAPE_BOOLEAN)
    ///    array(CAPE_ARRAY)
    /// Reference document: Parameter Common Interface
    /// </remarks>
    [Serializable]
    public enum CapeParamType
    {
        /// <summary>
        /// Double-precision real-valued parameter
        /// </summary>
        /// <value>0</value>
        CAPE_REAL = 0,
        /// <summary>
        /// Integer-valued parameter
        /// </summary>
        CAPE_INT = 1,
        /// <summary>
        /// String/option parameter
        /// </summary>
        CAPE_OPTION = 2,
        /// <summary>
        /// Boolean-valued parameter
        /// </summary>
        CAPE_BOOLEAN = 3,
        /// <summary>
        /// Array parameter
        /// </summary>
        CAPE_ARRAY = 4
    };
    //typedef CapeParamType eCapeParamType;

    /// <summary>
    /// Modes of parameters.
    /// </summary>
    /// <remarks>
    /// <para> It allows the following values:</para>
    /// <list type="number">
    /// <item>Input (CAPE_INPUT): the Unit(or whichever owner component) will use its value to calculate.</item>
    /// <item>Output (CAPE_OUTPUT): the Unit will place in the parameter a result of its calculations.</item>
    /// <item>Input-Output (CAPE_INPUT_OUTPUT): the user inputs an initial estimation value and the user outputs a 
    /// calculated value.</item>
    /// </list>
    /// Reference document: Parameter Common Interface
    /// </remarks>
    [Serializable]
    public enum CapeParamMode
    {
        /// <summary>
        /// The Unit(or whichever owner component) will use the parameter's value as an 
        /// input to its calculation.
        /// </summary>
        CAPE_INPUT = 0,
        /// <summary>
        /// The Unit(or whichever owner component) will set the parameter's value as 
        /// an output to its calculation.
        /// </summary>
        CAPE_OUTPUT = 1,
        /// <summary>
        /// The Unit(or whichever owner component) will use the parameter's initial value as 
        /// an estimate and will calculate the final value.
        /// </summary>
        CAPE_INPUT_OUTPUT = 2
    };
    //typedef CapeParamMode eCapeParamMode;

    /// <remarks>
    /// Reference document: Parameter Common Interface
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.ComponentModel.DescriptionAttribute("ICapeParameterSpec Interface")]
    public interface ICapeParameterSpec
    {
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
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("property Type")]
        CapeParamType Type
        {
            get;
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
        /// <value>an integer array indicating the exponents of the various dimensional axes.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("property Dimensionality")]
        double[] Dimensionality
        {
            get;
        }
    };

    /// <remarks>
    /// Reference document: Parameter Common Interface
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeParameterSpec_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeParameterSpec Interface")]
    interface ICapeParameterSpecCOM
    {
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
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("property Type")]
        CapeParamType Type
        {
            get;
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
        /// <value>an integer array indicating the exponents of the various dimensional axes.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("property Dimensionality")]
        Object Dimensionality
        {
            get;
        }
    };

    /// <summary>
    /// This interface is for a parameter specification when the 
    /// parameter has a double-precision floating point value.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.ComponentModel.DescriptionAttribute("ICapeRealParameterSpec Interface")]
    public interface ICapeRealParameterSpec
    {
        /// <summary>
        /// Gets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// A default value for the parameter.
        /// </remarks>
        /// <value>
        /// The default value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("property Default")]
        double SIDefaultValue
        {
            get;
            set;
        }

        /// <summary>
        /// Gets the lower bound of the parameter.
        /// </summary>
        /// <remarks>
        /// A lower bound value for the parameter.
        /// </remarks>
        /// <value>
        /// The lower bound of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("property LowerBound")]
        double SILowerBound
        {
            get;
            set;
        }

        /// <summary>
        /// Gets the upper bound of the parameter.
        /// </summary>
        /// <remarks>
        /// A upper bound value for the parameter.
        /// </remarks>
        /// <value>
        /// The upper bound of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3)]
        [System.ComponentModel.DescriptionAttribute("property UpperBound")]
        double SIUpperBound
        {
            get;
            set;
        }

        /// <summary>
        /// Validates the value against the specification of the parameter.
        /// The message is used to return the reason that the parameter is invalid.
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
        [System.Runtime.InteropServices.DispIdAttribute(4)]
        [System.ComponentModel.DescriptionAttribute("Check if value is OK for this spec as double")]
        [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.VariantBool)]
        bool SIValidate(double value, ref String message);

        /// <summary>
        /// Gets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// A default value for the parameter.
        /// </remarks>
        /// <value>
        /// The default value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("property Default")]
        double DimensionedDefaultValue
        {
            get;
            set;
        }

        /// <summary>
        /// Gets the lower bound of the parameter.
        /// </summary>
        /// <remarks>
        /// A lower bound value for the parameter.
        /// </remarks>
        /// <value>
        /// The lower bound of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("property LowerBound")]
        double DimensionedLowerBound
        {
            get;
            set;
        }

        /// <summary>
        /// Gets the upper bound of the parameter.
        /// </summary>
        /// <remarks>
        /// A upper bound value for the parameter.
        /// </remarks>
        /// <value>
        /// The upper bound of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3)]
        [System.ComponentModel.DescriptionAttribute("property UpperBound")]
        double DimensionedUpperBound
        {
            get;
            set;
        }

        /// <summary>
        /// Validates the value against the specification of the parameter.
        /// The message is used to return the reason that the parameter is invalid.
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
        [System.Runtime.InteropServices.DispIdAttribute(4)]
        [System.ComponentModel.DescriptionAttribute("Check if value is OK for this spec as double")]
        [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.VariantBool)]
        bool DimensionedValidate(double value, ref String message);
    };


    /// <summary>
    /// This interface is for a parameter specification when the 
    /// parameter has a double-precision floating point value.
    /// </summary>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeRealParameterSpec_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeRealParameterSpec Interface")]
     interface ICapeRealParameterSpecCOM
    {
        /// <summary>
        /// Gets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// A default value for the parameter.
        /// </remarks>
        /// <value>
        /// The default value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("property Default")]
        double DefaultValue
        {
            get;
        }

        /// <summary>
        /// Gets the lower bound of the parameter.
        /// </summary>
        /// <remarks>
        /// A lower bound value for the parameter.
        /// </remarks>
        /// <value>
        /// The lower bound of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("property LowerBound")]
        double LowerBound
        {
            get;
        }

        /// <summary>
        /// Gets the upper bound of the parameter.
        /// </summary>
        /// <remarks>
        /// A upper bound value for the parameter.
        /// </remarks>
        /// <value>
        /// The upper bound of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3)]
        [System.ComponentModel.DescriptionAttribute("property UpperBound")]
        double UpperBound
        {
            get;
        }

        /// <summary>
        /// Validates the value against the specification of the parameter.
        /// The message is used to return the reason that the parameter is invalid.
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
        [System.Runtime.InteropServices.DispIdAttribute(4)]
        [System.ComponentModel.DescriptionAttribute("Check if value is OK for this spec as double")]
        [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.VariantBool)]
        bool Validate(double value, ref String message);
    };

    /// <summary>
    /// This interface is for a parameter specification
    /// when the parameter is an integer value.
    /// </summary>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeIntegerParameterSpec_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeIntegerParameterSpec Interface")]
    public interface ICapeIntegerParameterSpec
    {
        /// <summary>
        /// Gets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// A default value for the parameter.
        /// </remarks>
        /// <value>
        /// The default value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("property Default")]
        int DefaultValue
        {
            get;
            set;
        }

        /// <summary>
        /// Gets the lower bound of the parameter.
        /// </summary>
        /// <remarks>
        /// A lower bound value for the parameter.
        /// </remarks>
        /// <value>
        /// The lower bound of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("property LowerBound")]
        int LowerBound
        {
            get;
            set;
        }

        /// <summary>
        /// Gets the upper bound of the parameter.
        /// </summary>
        /// <remarks>
        /// A upper bound value for the parameter.
        /// </remarks>
        /// <value>
        /// The upper bound of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("property UpperBound")]
        int UpperBound
        {
            get;
            set;
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
        [System.Runtime.InteropServices.DispIdAttribute(4)]
        [System.ComponentModel.DescriptionAttribute("Check if value is OK for this spec as double")]
        [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.VariantBool)]
        bool Validate(int value, ref String message);
    };

    /// <summary>
    /// This interface is for a parameter specification
    /// when the parameter is an option, which represents
    /// a list of strings from which one is selected.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.ComponentModel.DescriptionAttribute("ICapeOptionParameterSpec Interface")]
    public interface ICapeOptionParameterSpec
    {
        /// <summary>
        /// Gets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// A default string value for the parameter.
        /// </remarks>
        /// <value>
        /// The default value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("property Default")]
        String DefaultValue
        {
            get;
            set;
        }

        /// <summary>
        /// Gets the list of valid values for the parameter if 'RestrictedtoList' property is true.
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
        [System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("The list of names of the items")]
        string[] OptionList
        {
            get;
            set;
        }

        /// <summary>
        /// A list of Strings that the valueo f the parameter will be validated against.
        /// </summary>
        /// <remarks>
        /// If <c>true</c>, the parameter's value will be validated against the Strings
        /// in the <see cref = "OptionList">OptionList</see>.
        /// </remarks>
        /// <value>
        /// Converted by COM interop to a COM-based CAPE-OPEN VARIANT_BOOL.
        /// </value>
        [System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("True if it only accepts values from the option list.")]
        bool RestrictedToList
        {
            get;
            set;
        }

        /// <summary>
        /// Validates the value against the parameter's specification.
        /// </summary>
        /// <remarks>
        /// If the value of the <see cref = "RestrictedToList">RestrictedToList</see>
        /// is set to <c>true</c>, the value is valid is valid value for the 
        /// parameter if it is included in the 
        /// <see cref = "OptionList">OptionList</see>. If the 
        /// value of <see cref = "RestrictedToList">RestrictedToList</see> is <c>false</c>
        /// any valid String is a valid value for the parameter.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "value">A candidate value for the parameter to be tested to determine whether the value is valid.</param>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(4)]
        [System.ComponentModel.DescriptionAttribute("Check if value is OK for this spec as string")]
        [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.VariantBool)]
        bool Validate(System.String value, ref String message);
    };


    /// <summary>
    /// This interface is for a parameter specification
    /// when the parameter is an option, which represents
    /// a list of strings from which one is selected.
    /// </summary>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeOptionParameterSpec_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeOptionParameterSpec Interface")]
    interface ICapeOptionParameterSpecCOM
    {
        /// <summary>
        /// Gets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// A default string value for the parameter.
        /// </remarks>
        /// <value>
        /// The default value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("property Default")]
        String DefaultValue
        {
            get;
        }

        /// <summary>
        /// Gets the list of valid values for the parameter if 'RestrictedtoList' property is true.
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
        [System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("The list of names of the items")]
        Object OptionList
        {
            get;
        }

        /// <summary>
        /// A list of Strings that the valueo f the parameter will be validated against.
        /// </summary>
        /// <remarks>
        /// If <c>true</c>, the parameter's value will be validated against the Strings
        /// in the <see cref = "OptionList">OptionList</see>.
        /// </remarks>
        /// <value>
        /// Converted by COM interop to a COM-based CAPE-OPEN VARIANT_BOOL.
        /// </value>
        [System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("True if it only accepts values from the option list.")]
        bool RestrictedToList
        {
            get;
        }

        /// <summary>
        /// Validates the value against the parameter's specification.
        /// </summary>
        /// <remarks>
        /// If the value of the <see cref = "RestrictedToList">RestrictedToList</see>
        /// is set to <c>true</c>, the value is valid is valid value for the 
        /// parameter if it is included in the 
        /// <see cref = "OptionList">OptionList</see>. If the 
        /// value of <see cref = "RestrictedToList">RestrictedToList</see> is <c>false</c>
        /// any valid String is a valid value for the parameter.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "value">A candidate value for the parameter to be tested to determine whether the value is valid.</param>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(4)]
        [System.ComponentModel.DescriptionAttribute("Check if value is OK for this spec as string")]
        [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.VariantBool)]
        bool Validate(System.String value, ref String message);
    };
    
    /// <summary>
    /// This interface is for a parameter specification when the parameter is a boolean
    /// </summary>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeBooleanParameterSpec_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeBooleanParameterSpec Interface")]
    public interface ICapeBooleanParameterSpec
    {
        /// <summary>
        /// Gets the default value of the parameter.
        /// </summary>
        /// <remarks>
        /// Gets the default value of the parameter.
        /// </remarks>
        /// <value>
        /// The default value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("property Default")]
        bool DefaultValue
        {
            get;
            set;
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
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("Check if value is OK for this spec")]
        [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.VariantBool)]
        bool Validate(bool value, ref String message);
    };

    /// <summary>
    /// This interface is for a parameter specification
    /// when the parameter is an array of values (maybebe integers,reals,
    /// booleans or arrays again, which represents.
    /// </summary>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeArrayParameterSpec_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeArrayParameterSpec Interface")]
    public interface ICapeArrayParameterSpec
    {
        /// <summary>
        /// Get the number of dimensions of the array.
        /// </summary>
        /// <remarks>
        /// The number of dimensions of the paramater array. 
        /// </remarks>
        /// <value>
        /// The number of dimensions of the array.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("Get the number of dimensions of the array")]
        int NumDimensions
        {
            get;
        }

        /// <summary>
        /// Gets the size of each one of the dimensions of the array.
        /// </summary>
        /// <remarks>
        /// An array containing the specfication of each member of the paramater array. 
        /// </remarks>
        /// <value>
        /// An integer array containing the size of each dimension of the array.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("Get the size of each one of the dimensions of the array")]
        int[] Size
        {
            get;
        }

        /// <summary>
        /// Gets an array of the specifications of each of the items in the 
        /// value of a parameter.
        /// </summary>
        /// <remarks>
        /// ﻿An array of interfaces to the correct specification type (<see cref = "ICapeRealParameterSpec"/> ,
        /// <see cref = "ICapeIntegerParameterSpec"/> , <see cref = "ICapeBooleanParameterSpec"/> , 
        /// <see cref = "ICapeOptionParameterSpec"/> ). Note that it is also possible, for 
        /// example, to configure an array of arrays of integers, which would a similar 
        /// but not identical concept to a two-dimensional matrix of integers.
        /// </remarks>
        /// <value>
        /// An array of <see cref = "ICapeParameterSpec"/> objects.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("Get the specification of each of the values in the array")]
        Object[] ItemsSpecifications
        {
            get;
        }

        /// <summary>
        /// Validates the value against the specification of the parameter.
        /// The message is used to return the reason that the parameter is invalid.
        /// </summary>
        /// <remarks>
        /// This method checks the current value of the parameter to determine if it is an allowed value. 
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "inputArray">The message is used to return the reason that the parameter is invalid.</param>
        /// <param name = "messages">A string array containing the message is used to return the reason that the parameter is invalid.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(4), System.ComponentModel.DescriptionAttribute("Check if value is OK for this spec ")]
        Object Validate(Object inputArray, ref string[] messages);

    };

    /// <summary>
    /// Interface defining the actual Parameter quantity.
    /// </summary>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeParameter_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeParameter Interface")]
    public interface ICapeParameter
    {
        /// <summary>
        /// Gets the Specification for this Parameter
        /// </summary>
        /// <remarks>
        /// Gets the specification of the parameter. The Get method returns the 
        /// specification as an interface to the correct specification type.
        /// </remarks>
        /// <value>
        /// An object implementing the <see cref = "ICapeParameterSpec"/>, as well as the
        /// appropraite specification for the parameter type, <see cref = "ICapeRealParameterSpec"/> ,
        /// <see cref = "ICapeIntegerParameterSpec"/> , <see cref = "ICapeBooleanParameterSpec"/> , 
        /// <see cref = "ICapeOptionParameterSpec"/> , or <see cref = "ICapeArrayParameterSpec"/> .
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("Gets and sets the specification for the parameter.")]
        Object Specification
        {
            [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
            get;
        }

        /// <summary>
        /// Gets and sets the value for this Parameter
        /// </summary>
        /// <remarks>
        /// Gets and sets the value of this parameter. Passed as a CapeVariant that 
        /// should be the same type as the Parameter type.
        /// </remarks>
        /// <value>
        /// The boxed value of the parameter.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("Get and sets the value of the parameter.")]
        Object value
        {
            get;
            set;
        }

        /// <summary>
        /// Gets the flag to indicate parameter validation's status.
        /// </summary>
        /// <remarks>
        /// <para>Gets the flag to indicate parameter validation status. It has three 
        /// possible values:</para>
        /// <para>   (i)   notValidated(CAPE_NOT_VALIDATED): The PMC's <c>Validate()</c>
        /// method has not been called after the last time that its value had been 
        /// changed.</para>
        /// <para>   (ii)  invalid(CAPE_INVALID): The last time that the PMC's 
        /// <c>Validate()</c> method was called it returned false.</para>
        /// <para>   (iii) valid(CAPE_VALID): the last time that the PMC's
        /// Validate() method was called it returned true.</para>
        /// </remarks>
        /// <value>The validity staus of the parameter, either valid, invalid, or "not validated".</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("Get the parameter validation status")]
        CapeValidationStatus ValStatus
        {
            get;
        }

        /// <summary>
        /// Gets and sets the mode of the parameter.
        /// </summary>
        /// <remarks>
        /// <para>Modes of parameters. It allows the following values:</para>
        /// <para>   (i)   Input (CAPE_INPUT): the Unit(or whichever owner component) will use 
        /// its value to calculate.</para>
        /// <para>   (ii)  Output (CAPE_OUTPUT): the Unit will place in the parameter a result 
        /// of its calculations.</para>
        /// <para>   (iii) Input-Output (CAPE_INPUT_OUTPUT): the user inputs an 
        /// initial estimation value and the user outputs a calculated value.</para>
        /// </remarks>
        /// <value>The mode of the parameter, input, output, or input/output.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(4), System.ComponentModel.DescriptionAttribute("Get the Mode - input,output - of the parameter.")]
        CapeParamMode Mode
        {
            get;
            set;
        }

        /// <summary>
        /// Validates the current value of the parameter against the 
        /// specification of the parameter.
        /// </summary>
        /// <remarks>
        /// This method checks the current value of the parameter to determine if it is an allowed value. In the case of 
        /// numeric parameters (<see cref = "ICapeRealParameterSpec"/> and <see cref = "ICapeIntegerParameterSpec"/>),
        /// the value is valid if it is between the upper and lower bound. For String (<see cref = "ICapeOptionParameterSpec"/>),
        /// if the <see cref = "ICapeOptionParameterSpec.RestrictedToList"/> property is true, the value must be included as one of the
        /// members of the <see cref = "ICapeOptionParameterSpec.OptionList"/>. Otherwise, any string value is valid. Any boolean value (true/false) 
        /// valid for the <see cref = "ICapeBooleanParameterSpec"/> paramaters.
        /// </remarks>
        /// <returns>
        /// True if the parameter is valid, false if not valid.
        /// </returns>
        /// <param name = "message">The message is used to return the reason that the parameter is invalid.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(5)]
        [System.ComponentModel.DescriptionAttribute("Validate the parameter's current value.")]
        [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.VariantBool)]
        bool Validate(ref String message);

        /// <summary>
        /// Sets the value of the parameter to its default value.
        /// </summary>
        /// <remarks>
        /// This method sets the parameter to its default value.
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(6)]
        [System.ComponentModel.DescriptionAttribute("Reset the value of the parameter to its default.")]
        void Reset();
    };

    /// <summary>
    /// </summary>
    /// <remarks>
    /// </remarks>
    [System.Runtime.InteropServices.InterfaceType(System.Runtime.InteropServices.ComInterfaceType.InterfaceIsIDispatch)]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("3C32AD8E-490D-4822-8A8E-073F5EDFF3F5")]
    [System.ComponentModel.DescriptionAttribute("CapeParameterEvents Interface")]
    interface IParameterEvents
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
        /// <param name = "args">A <see cref = "ParameterValueChanged">ParameterValueChanged</see> that contains information about the event.</param>
        void ParameterValueChanged(object sender, object args);

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
        /// <param name = "args">A <see cref = "ParameterModeChangedEventArgs">ParameterModeChangedEventArgs</see> that contains information about the event.</param>
        void ParameterModeChanged(object sender, object args);

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

        /// <summary>
        /// Occurs when the user resets a parameter.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnParameterReset</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnParameterReset</c> in a derived class, be sure to call the base class's <c>OnParameterReset</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "sender">The <see cref = "RealParameter">RealParameter</see> that raised the event.</param>
        /// <param name = "args">A <see cref = "ParameterResetEventArgs">ParameterResetEventArgs</see> that contains information about the event.</param>
        void ParameterReset(object sender, object args);
    }


    class ParameterTypeConverter : System.ComponentModel.ExpandableObjectConverter
    {
        public override bool CanConvertTo(System.ComponentModel.ITypeDescriptorContext context, System.Type destinationType)
        {
            if (typeof(CapeOpen.ICapeParameter).IsAssignableFrom(destinationType))
                return true;
            if (typeof(CapeOpen.ICapeIdentification).IsAssignableFrom(destinationType))
                return true;
            return base.CanConvertTo(context, destinationType);
        }
        
        public override Object ConvertTo(System.ComponentModel.ITypeDescriptorContext context, System.Globalization.CultureInfo culture, Object parameter, System.Type destinationType)
        {
            if (typeof(System.String).IsAssignableFrom(destinationType) && typeof(ArrayParameterWrapper).IsAssignableFrom(parameter.GetType()))
            {
                return "";//((CapeOpen.ArrayParameterWrapper)parameter).;
            }
            if (typeof(System.String).IsAssignableFrom(destinationType) && typeof(ICapeParameter).IsAssignableFrom(parameter.GetType()))
            {
                return ((CapeOpen.ICapeParameter)parameter).value.ToString();
            }
            if (typeof(String).IsAssignableFrom(destinationType) && typeof(ICapeIdentification).IsAssignableFrom(parameter.GetType()))
            {
                return ((CapeOpen.ICapeIdentification)parameter).ComponentName;
            }
            return base.ConvertTo(context, culture, parameter, destinationType);
        }
    };

    /// <summary>
    /// Aspen(TM) interface for providing dimension for a real-valued parameter.
    ///</summary>
    /// <remarks>
    /// <para>
    /// Aspen Plus(TM) does not use the <see cref = "CapeOpen.ICapeParameterSpec.Dimensionality">ICapeParameterSpec.Dimensionality</see> method. Instead a parameter
    /// can implement the IATCapeXRealParameterSpec interface which can be used to define the
    /// display unit for a parameter value. 
    /// </para>
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid("B777A1BD-0C88-11D3-822E-00C04F4F66C9")]
    [System.ComponentModel.DescriptionAttribute("IATCapeXRealParameterSpec Interface")]
    interface IATCapeXRealParameterSpec
    {
        /// <summary>
        /// Gets the display unit for the parameter. Used by AspenPlus(TM).
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
        [System.Runtime.InteropServices.DispIdAttribute(0x60040003), System.ComponentModel.DescriptionAttribute(" Provide the Aspen Plus display units for for this parameter.")]
        String DisplayUnits
        {
            get;
        }
    };


    /// <summary>
    /// Represents the method that will handle the changing of the value of a parameter.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void ParameterValueChangedHandler(Object sender, ParameterValueChangedEventArgs args);

    /// <summary>
    /// Provides data for the value changed event associated with the parameters.
    /// </summary>
    /// <remarks>
    /// The IParameterValueChangedEventArgs interface specifies the old and new value of the parameter.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("41E1A3C4-F23C-4B39-BC54-39851A1D09C9")]
    [System.ComponentModel.DescriptionAttribute("CapeIdentificationEvents Interface")]
    interface IParameterValueChangedEventArgs
    {
        /// <summary>
        /// The name of the parameter being changed.</summary>
        String ParameterName
        {
            get;
        }
        /// <summary>
        /// The value of the parameter prior to the change.</summary>
        /// <remarks>The former value of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The value of the parameter prior to the change.</value>
        Object OldValue
        {
            get;
        }
        /// <summary>
        /// The value of the parameter after the change.</summary>
        /// <remarks>The new nvalue of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The value of the parameter after the change.</value>
        Object NewValue
        {
            get;
        }
    };

    /// <summary>
    /// Provides data for the value changed event associated with the parameters.
    /// </summary>
    /// <remarks>
    /// The ParameterValueChangedEventArgs event specifies the old and new value of the parameter.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("C3592B59-92E8-4A24-A2EB-E23C38F88E7F")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class ParameterValueChangedEventArgs : System.EventArgs,
        IParameterValueChangedEventArgs
    {
        private String m_paramName;
        private Object m_oldValue;
        private Object m_newValue;

        /// <summary>Creates an instance of the ParameterValueChangedEventArgs class with the old and parameter value.</summary>
        /// <remarks>You can use this constructor when raising the ParameterValueChangedEvent at run time to specify a 
        /// specific the parameter having its value changed.
        /// </remarks>
        /// <param name = "paramName">The name of the parameter being changed.</param>
        /// <param name = "oldValue">The name of the PMC prior to the name change.</param>
        /// <param name = "newValue">The name of the PMC after the name change.</param>
        public ParameterValueChangedEventArgs(String paramName, Object oldValue, Object newValue)
            : base()
        {
            m_paramName = paramName;
            m_oldValue = oldValue;
            m_newValue = newValue;
        }

        /// <summary>
        /// The name of the parameter being changed.</summary>
        /// <value>The name of the parameter being changed.</value>
        public String ParameterName
        {
            get
            {
                return m_paramName;
            }
        }
        /// <summary>
        /// The value of the parameter prior to the name change.</summary>
        /// <remarks>The former value of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The value of the parameter prior to the change.</value>
        public Object OldValue
        {
            get
            {
                return m_oldValue;
            }
        }
        /// <summary>
        /// The value of the parameter after the change.</summary>
        /// <remarks>The new name of the unit can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The value of the parameter after the change.</value>
        public Object NewValue
        {
            get
            {
                return m_newValue;
            }
        }
    };


    /// <summary>
    /// Represents the method that will handle the changing of the default value of a parameter.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void ParameterDefaultValueChangedHandler(Object sender, ParameterDefaultValueChangedEventArgs args);

    /// <summary>
    /// Provides data for the value changed event associated with the parameters.
    /// </summary>
    /// <remarks>
    /// The IParameterDefaultValueChangedEventArgs interface specifies the old and new default value of the parameter.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("E5D9CE6A-9B10-4A81-9E06-1B6C6C5257F3")]
    [System.ComponentModel.DescriptionAttribute("CapeIdentificationEvents Interface")]
    interface IParameterDefaultValueChangedEventArgs
    {
        /// <summary>
        /// The name of the parameter being changed.</summary>
        String ParameterName
        {
            get;
        }
        /// <summary>
        /// The default value of the parameter prior to the change.</summary>
        /// <remarks>The default value of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The default value of the parameter prior to the change.</value>
        Object OldDefaultValue
        {
            get;
        }
        /// <summary>
        /// The default value of the parameter  after the name change.</summary>
        /// <remarks>The new default value of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The default value of the parameter after the change.</value>
        Object NewDefaultValue
        {
            get;
        }
    };

    /// <summary>
    /// Provides data for the value changed event associated with the parameters.
    /// </summary>
    /// <remarks>
    /// The ParameterDefaultValueChangedEventArgs event specifies the old and new default value of the parameter.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("355A5BDD-F6B5-4EEE-97C7-F1533DD28889")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class ParameterDefaultValueChangedEventArgs : System.EventArgs,
        IParameterDefaultValueChangedEventArgs
    {
        private String m_paramName;
        private Object m_oldDefaultValue;
        private Object m_newDefaultValue;

        /// <summary>Creates an instance of the ParameterDefaultValueChangedEventArgs class with the old and new default values.</summary>
        /// <remarks>You can use this constructor when raising the ParameterDefaultValueChangedEventArgs at run time to specify  
        /// that the default value of the parameter has changed.
        /// </remarks>
        /// <param name = "paramName">The name of the parameter being changed.</param>
        /// <param name = "oldDefaultValue">The default value of the parameter prior to the change.</param>
        /// <param name = "newDefaultValue">The default value of the parameter after the change.</param>
        public ParameterDefaultValueChangedEventArgs(String paramName, Object oldDefaultValue, Object newDefaultValue)
            : base()
        {
            m_paramName = paramName;
            m_oldDefaultValue = oldDefaultValue;
            m_newDefaultValue = newDefaultValue;
        }

        /// <summary>
        /// The name of the parameter being changed.</summary>
        /// <value>The name of the parameter being changed.</value>
        public String ParameterName
        {
            get
            {
                return m_paramName;
            }
        }
        /// <summary>
        /// The name of the PMC prior to the name change.</summary>
        /// <remarks>The former name of the unit can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The default of the parameter prior to the change.</value>
        public Object OldDefaultValue
        {
            get
            {
                return m_oldDefaultValue;
            }
        }
        /// <summary>
        /// The default value of the parameter after the name change.</summary>
        /// <remarks>The new default value for the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The default value of the parameter after the change.</value>
        public Object NewDefaultValue
        {
            get
            {
                return m_newDefaultValue;
            }
        }
    };

    /// <summary>
    /// Represents the method that will handle the changing of the lower bound of a parameter.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void ParameterLowerBoundChangedHandler(Object sender, ParameterLowerBoundChangedEventArgs args);

    /// <summary>
    /// Provides data for the value changed event associated with the parameters.
    /// </summary>
    /// <remarks>
    /// The IParameterLowerBoundChangedEventArgs interface specifies the old and new lower bound of the parameter.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("FBCE7FC9-0F58-492B-88F9-8A23A23F93B1")]
    [System.ComponentModel.DescriptionAttribute("CapeIdentificationEvents Interface")]
    interface IParameterLowerBoundChangedEventArgs
    {
        /// <summary>
        /// The name of the parameter being changed.</summary>
        String ParameterName
        {
            get;
        }
        /// <summary>
        /// The lower bound of the parameter prior to the change.</summary>
        /// <remarks>The former lower bound of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The lower bound of the parameter prior to the change.</value>
        Object OldLowerBound
        {
            get;
        }
        /// <summary>
        /// The lower bound of the parameter after to the change.</summary>
        /// <remarks>The former lower bound of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The lower bound of the parameter after to the change.</value>
        Object NewLowerBound
        {
            get;
        }
    };

    /// <summary>
    /// Provides data for the value changed event associated with the parameters.
    /// </summary>
    /// <remarks>
    /// The ParameterLowerBoundChangedEventArgs event specifies the old and new lower bound of the parameter.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("A982AD29-10B5-4C86-AF74-3914DD902384")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class ParameterLowerBoundChangedEventArgs : System.EventArgs,
        IParameterLowerBoundChangedEventArgs
    {
        private String m_paramName;
        private Object m_oldLowerBound;
        private Object m_newLowerBound;

        /// <summary>Creates an instance of the ParameterLowerBoundChangedEventArgs class with the old and new lower bound for the parameter.</summary>
        /// <remarks>You can use this constructor when raising the ParameterLowerBoundChangedEvent at run time to specify that 
        /// the lower bound of the parameter has changed.
        /// </remarks>
        /// <param name = "paramName">The name of the parameter being changed.</param>
        /// <param name = "oldLowerBound">The name of the PMC prior to the name change.</param>
        /// <param name = "newLowerBound">The name of the PMC after the name change.</param>
        public ParameterLowerBoundChangedEventArgs(String paramName, Object oldLowerBound, Object newLowerBound)
            : base()
        {
            m_paramName = paramName;
            m_oldLowerBound = oldLowerBound;
            m_newLowerBound = newLowerBound;
        }

        /// <summary>
        /// The name of the parameter being changed.</summary>
        /// <value>The name of the parameter being changed.</value>
        public String ParameterName
        {
            get
            {
                return m_paramName;
            }
        }
        /// <summary>
        /// The lower bound of the parameter prior to the change.</summary>
        /// <remarks>The former lower bound of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The lower bound of the parameter prior to the change.</value>
        public Object OldLowerBound
        {
            get
            {
                return m_oldLowerBound;
            }
        }
        /// <summary>
        /// The lower bound of the parameter after the change.</summary>
        /// <remarks>The new lower bound of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The lower bound of the parameter after the change.</value>
        public Object NewLowerBound
        {
            get
            {
                return m_newLowerBound;
            }
        }
    };

    /// <summary>
    /// Represents the method that will handle the changing of the upper bound of a parameter.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void ParameterUpperBoundChangedHandler(Object sender, ParameterUpperBoundChangedEventArgs args);

    /// <summary>
    /// Represents the method that will handle the changing of the upper bound of a parameter.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    delegate void ParameterUpperBoundChangedHandlerCOM(Object sender, object args);

    /// <summary>
    /// Provides data for the upper bound changed event associated with the parameters.
    /// </summary>
    /// <remarks>
    /// The IParameterUpperBoundChangedEventArgs interface specifies the old and new lower bound of the parameter.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("A2D0FAAB-F30E-48F5-82F1-4877F61950E9")]
    [System.ComponentModel.DescriptionAttribute("CapeIdentificationEvents Interface")]
    interface IParameterUpperBoundChangedEventArgs
    {
        /// <summary>
        /// The name of the parameter being changed.</summary>
        String ParameterName
        {
            get;
        }
        /// <summary>
        /// The upper bound of the parameter prior to the change.</summary>
        /// <remarks>The former upper bound of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The upper bound of the parameter prior to the change.</value>
        Object OldUpperBound
        {
            get;
        }
        /// <summary>
        /// The upper bound of the parameter after to the change.</summary>
        /// <remarks>The former upper bound of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The upper bound of the parameter after to the change.</value>
        Object NewUpperBound
        {
            get;
        }
    };

    /// <summary>
    /// Provides data for the upper bound changed event associated with the parameters.
    /// </summary>
    /// <remarks>
    /// The ParameterUpperBoundChangedEventArgs event specifies the old and new lower bound of the parameter.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("92BF83FE-0855-4382-A15E-744890B5BBF2")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class ParameterUpperBoundChangedEventArgs : System.EventArgs,
        IParameterUpperBoundChangedEventArgs
    {
        private String m_paramName;
        private Object m_oldUpperBound;
        private Object m_newUpperBound;

        /// <summary>Creates an instance of the ParameterUpperBoundChangedEventArgs class with the old and new upper bound for the parameter.</summary>
        /// <remarks>You can use this constructor when raising the ParameterUpperBoundChangedEvent at run time to specify 
        /// that the upper bound of the parameter has changed.
        /// </remarks>
        /// <param name = "paramName">The name of the parameter being changed.</param>
        /// <param name = "oldUpperBound">The upper bound of the parameter prior to the change.</param>
        /// <param name = "newUpperBound">The upper bound of the parameter after the change.</param>
        public ParameterUpperBoundChangedEventArgs(String paramName, Object oldUpperBound, Object newUpperBound)
            : base()
        {
            m_paramName = paramName;
            m_oldUpperBound = oldUpperBound;
            m_newUpperBound = newUpperBound;
        }

        /// <summary>
        /// The name of the parameter being changed.</summary>
        /// <value>The name of the parameter being changed.</value>
        public String ParameterName
        {
            get
            {
                return m_paramName;
            }
        }
        /// <summary>
        /// The upper bound of the parameter prior to the change.</summary>
        /// <remarks>The former upper bound of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The upper bound of the parameter prior to the change.</value>
        public Object OldUpperBound
        {
            get
            {
                return m_oldUpperBound;
            }
        }
        /// <summary>
        /// The upper bound of the parameter after the change.</summary>
        /// <remarks>The new upper bound of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The upper bound of the parameter after the change.</value>
        public Object NewUpperBound
        {
            get
            {
                return m_newUpperBound;
            }
        }
    };


    /// <summary>
    /// Represents the method that will handle the changing of the mode of a parameter.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void ParameterModeChangedHandler(Object sender, ParameterModeChangedEventArgs args);


    /// <summary>
    /// Provides data for the mode changed event associated with the parameters.
    /// </summary>
    /// <remarks>
    /// The IParameterModeChangedEventArgs interface specifies the old and new mode of the parameter.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("5405E831-4B5F-4A57-A410-8E91BBF9FFD3")]
    [System.ComponentModel.DescriptionAttribute("CapeIdentificationEvents Interface")]
    interface IParameterModeChangedEventArgs
    {
        /// <summary>
        /// The name of the parameter being changed.</summary>
        String ParameterName
        {
            get;
        }
        /// <summary>
        /// The mode of the parameter prior to the change.</summary>
        /// <remarks>The former mode of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The mode of the parameter prior to the change.</value>
        Object OldMode
        {
            get;
        }
        /// <summary>
        /// The mode of the parameter after to the change.</summary>
        /// <remarks>The former mode of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The mode of the parameter after to the change.</value>
        Object NewMode
        {
            get;
        }
    };

    /// <summary>
    /// Provides data for the mode changed event associated with the parameters.
    /// </summary>
    /// <remarks>
    /// The ParameterModeChangedEventArgs event specifies the old and new mode of the parameter.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("3C953F15-A1F3-47A9-829A-9F7590CEB5E9")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class ParameterModeChangedEventArgs : System.EventArgs,
        IParameterModeChangedEventArgs
    {
        private String m_paramName;
        private Object m_oldMode;
        private Object m_newMode;

        /// <summary>Creates an instance of the ParameterModeChangedEventArgs class with the old and new upper bound for the parameter.</summary>
        /// <remarks>You can use this constructor when raising the ParameterModeChangedEvent at run time to specify 
        /// that the mode of the parameter has changed.
        /// </remarks>
        /// <param name = "paramName">The name of the parameter being changed.</param>
        /// <param name = "oldMode">The mode of the parameter prior to the change.</param>
        /// <param name = "newMode">The mode of the parameter after the change.</param>
        public ParameterModeChangedEventArgs(String paramName, Object oldMode, Object newMode)
            : base()
        {
            m_paramName = paramName;
            m_oldMode = oldMode;
            m_newMode = newMode;
        }

        /// <summary>
        /// The name of the parameter being changed.
        /// </summary>
        /// <remarks>The name of the parameter being updated can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The name of the parameter being changed.</value>
        public String ParameterName
        {
            get
            {
                return m_paramName;
            }
        }
        /// <summary>
        /// The mode of the parameter prior to the change.</summary>
        /// <remarks>The former mode of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The mode of the parameter prior to the change.</value>
        public Object OldMode
        {
            get
            {
                return m_oldMode;
            }
        }
        /// <summary>
        /// The mode of the parameter after the change.</summary>
        /// <remarks>The new mode of the parameter can be used to update GUI inforamtion about the PMC.</remarks>
        /// <value>The mode of the parameter after the change.</value>
        public Object NewMode
        {
            get
            {
                return m_newMode;
            }
        }
    };

    /// <summary>
    /// Represents the method that will handle the validation of a parameter.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void ParameterValidatedHandler(Object sender, ParameterValidatedEventArgs args);

    /// <summary>
    /// The parameter was validated.
    /// </summary>
    /// <remarks>
    /// Provides information about the validation of the parameter.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("EFD819A4-E4EC-462E-90E6-5D994CA44F8E")]
    [System.ComponentModel.DescriptionAttribute("ParameterValidatedEvent Interface")]
    interface IParameterValidatedEventArgs
    {
        /// <summary>
        /// The name of the parameter being changed.
        /// </summary>
        /// <remarks>
        /// The name of the parameter being updated can be used to update GUI inforamtion about the PMC.
        /// </remarks>
        /// <value>The name of the parameter being changed.</value>
        String ParameterName
        {
            get;
        }
        /// <summary>
        /// The message reulting from the parameter validation.</summary>
        /// <remarks>The message provides information about the results of the validation process.</remarks>
        /// <value>Information regrading the validation process.</value>
        String Message
        {
            get;
        }
        /// <summary>
        /// The validation status of the parameter prior to the validation.</summary>
        /// <remarks>Informs the user of the results of the validation process.</remarks>
        /// <value>The validation status of the parameter prior to the validation.</value>
        CapeValidationStatus OldStatus
        {
            get;
        }
        /// <summary>
        /// The validation status of the parameter after the validation.</summary>
        /// <remarks>Informs the user of the results of the validation process.</remarks>
        /// <value>The validation status of the parameter after the validation.</value>
        CapeValidationStatus NewStatus
        {
            get;
        }
    };

    /// <summary>
    /// The parameter was validated.
    /// </summary>
    /// <remarks>
    /// Provides information about the validation of the parameter.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("5727414A-838D-49F8-AFEF-1CC8C578D756")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class ParameterValidatedEventArgs : System.EventArgs,
        IParameterValidatedEventArgs
    {
        private String m_paramName;
        private String m_message;
        private CapeValidationStatus m_oldStatus;
        private CapeValidationStatus m_newStatus;

        /// <summary>Creates an instance of the ParameterValidatedEventArgs class for the parameter.</summary>
        /// <remarks>You can use this constructor when raising the ParameterValidatedEventArgs at run time to  
        /// the message about the parameter validation.
        /// </remarks>
        /// <param name = "paramName">The name of the parameter being changed.</param>
        /// <param name = "message">The message indicating the results of the parameter validation.</param>
        /// <param name = "oldStatus">The status of the parameter prior to validation.</param>
        /// <param name = "newStatus">The status of the parameter after the validation.</param>
        public ParameterValidatedEventArgs(String paramName, String message, CapeValidationStatus oldStatus, CapeValidationStatus newStatus)
            : base()
        {
            m_paramName = paramName;
            m_message = message;
            m_oldStatus = oldStatus;
            m_newStatus = newStatus;
        }

        /// <summary>
        /// The name of the parameter being changed.
        /// </summary>
        /// <remarks>
        /// The name of the parameter being updated can be used to update GUI inforamtion about the PMC.
        /// </remarks>
        /// <value>The name of the parameter being changed.</value>
        public String ParameterName
        {
            get
            {
                return m_paramName;
            }
        }
        /// <summary>
        /// The message reulting from the parameter validation.</summary>
        /// <remarks>The message provides information about the results of the validation process.</remarks>
        /// <value>Information regrading the validation process.</value>
        public String Message
        {
            get
            {
                return m_message;
            }
        }
        /// <summary>
        /// The validation status of the parameter prior to the validation.</summary>
        /// <remarks>Informs the user of the results of the validation process.</remarks>
        /// <value>The validation status of the parameter prior to the validation.</value>
        public CapeValidationStatus OldStatus
        {
            get
            {
                return m_oldStatus;
            }
        }
        /// <summary>
        /// The validation status of the parameter after the validation.</summary>
        /// <remarks>Informs the user of the results of the validation process.</remarks>
        /// <value>The validation status of the parameter after the validation.</value>
        public CapeValidationStatus NewStatus
        {
            get
            {
                return m_newStatus;
            }
        }
    };

    /// <summary>
    /// Represents the method that will handle the resetting of a parameter.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void ParameterResetHandler(Object sender, ParameterResetEventArgs args);


    /// <summary>
    /// The parameter was reset.
    /// </summary>
    /// <remarks>
    /// The parameter was reset.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("12067518-B797-4895-9B26-EA71C60A8803")]
    [System.ComponentModel.DescriptionAttribute("ParameterResetEventArgs Interface")]
    interface IParameterResetEventArgs
    {
        /// <summary>
        /// The name of the parameter being changed.</summary>
        String ParameterName
        {
            get;
        }
    };

    /// <summary>
    /// The parameter was reset.
    /// </summary>
    /// <remarks>
    /// The parameter was reset.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("01BF391B-415E-4F5E-905D-395A707DC125")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class ParameterResetEventArgs : System.EventArgs,
        IParameterResetEventArgs
    {
        private String m_paramName;
        /// <summary>Creates an instance of the ParameterResetEventArgs class for the parameter.</summary>
        /// <remarks>You can use this constructor when raising the ParameterResetEventArgs at run time to  
        /// inform the system that the parameter was reset.
        /// </remarks>
        public ParameterResetEventArgs(String paramName)
            : base()
        {
            m_paramName = paramName;
        }
        /// <summary>
        /// The name of the parameter being changed.
        /// </summary>
        /// <remarks>
        /// The name of the parameter being updated can be used to update GUI inforamtion about the PMC.
        /// </remarks>
        /// <value>The name of the parameter being reset.</value>
        public String ParameterName
        {
            get
            {
                return m_paramName;
            }
        }
    };

    /// <summary>
    /// Represents the method that will handle the changing of the option list of a parameter.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void ParameterOptionListChangedHandler(Object sender, ParameterOptionListChangedEventArgs args);

    /// <summary>
    /// The parameter was reset.
    /// </summary>
    /// <remarks>
    /// The parameter was reset.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("78E06E7B-00AB-4295-9915-546DC1CD64A6")]
    [System.ComponentModel.DescriptionAttribute("ParameterOptionListChangedEventArgs Interface")]
    interface IParameterOptionListChangedEventArgs
    {
        /// <summary>
        /// The name of the parameter being changed.
        /// </summary>
        /// <remarks>
        /// The name of the parameter being updated can be used to update GUI inforamtion about the PMC.
        /// </remarks>
        /// <value>The name of the parameter being changed.</value>
        String ParameterName
        {
            get;
        }
    };
    ///// <summary>
    ///// The parameter was reset.
    ///// </summary>
    ///// <remarks>
    ///// The parameter was reset.
    ///// </remarks>
    //[System.Runtime.InteropServices.ComVisibleAttribute(true)]
    //[System.Runtime.InteropServices.GuidAttribute("7B4DE7D2-1E39-4239-B8C5-4F876DDB15A4")]
    //[System.ComponentModel.DescriptionAttribute("ParameterOptionListChangedEventArgs Interface")]
    //public interface IParameterOptionsListChangedEventArgs
    //{
    //    /// <summary>
    //    /// The name of the parameter being changed.</summary>
    //    String ParameterName
    //    {
    //        get;
    //    }
    //};

    /// <summary>
    /// The parameter option list was changed.
    /// </summary>
    /// <remarks>
    /// The parameter option list was changed.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("2AEC279F-EBEC-4806-AA00-CC215432DB82")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class ParameterOptionListChangedEventArgs : System.EventArgs,
        IParameterOptionListChangedEventArgs
    {
        private String m_paramName;
        /// <summary>Creates an instance of the ParameterOptionListChangedEventArgs class for the parameter.</summary>
        /// <remarks>You can use this constructor when raising the ParameterOptionListChangedEventArgs at run time to  
        /// inform the system that the parameter's option list was changed.
        /// </remarks>
        public ParameterOptionListChangedEventArgs(String paramName)
            : base()
        {
            m_paramName = paramName;
        }
        /// <summary>
        /// The name of the parameter being changed.
        /// </summary>
        /// <remarks>
        /// The name of the parameter being updated can be used to update GUI inforamtion about the PMC.
        /// </remarks>
        /// <value>The name of the parameter being changed.</value>
        public String ParameterName
        {
            get
            {
                return m_paramName;
            }
        }
    };

    /// <summary>
    /// The restiction to the options list of a parameter was changed.
    /// </summary>
    /// <remarks>
    /// The restiction to the options list of a parameter was changed.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("7F357261-095A-4FD4-99C1-ACDAEDA36141")]
    [System.ComponentModel.DescriptionAttribute("ParameterOptionListChangedEventArgs Interface")]
    interface IParameterRestrictedToListChangedEventArgs
    {
        /// <summary>
        /// The name of the parameter being changed.
        /// </summary>
        /// <remarks>
        /// The name of the parameter being updated can be used to update GUI inforamtion about the PMC.
        /// </remarks>
        /// <value>The name of the parameter being changed.</value>
        String ParameterName
        {
            get;
        }
    };

    /// <summary>
    /// The parameter restiction to the option list was changed.
    /// </summary>
    /// <remarks>
    /// The parameter restiction to the option list was changed.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("82E0E6C2-3103-4B5A-A5BC-EBAB971B069A")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class ParameterRestrictedToListChangedEventArgs : System.EventArgs,
        IParameterRestrictedToListChangedEventArgs
    {
        private String m_paramName;
        private bool m_wasRestricted;
        private bool m_isRestricted;
        /// <summary>Creates an instance of the ParameterRestrictedToListChangedEventArgs class for the parameter.</summary>
        /// <remarks>You can use this constructor when raising the ParameterRestrictedToListChangedEventArgs at run time to  
        /// inform the system that the parameter's option list was changed.
        /// </remarks>
        public ParameterRestrictedToListChangedEventArgs(String paramName, bool wasRestricted, bool isRestricted)
            : base()
        {
            m_paramName = paramName;
            m_isRestricted = isRestricted;
            m_wasRestricted = wasRestricted;
        }
        /// <summary>
        /// The name of the parameter being changed.
        /// </summary>
        /// <remarks>
        /// The name of the parameter being updated can be used to update GUI inforamtion about the PMC.
        /// </remarks>
        /// <value>The name of the parameter being changed.</value>
        public String ParameterName
        {
            get
            {
                return m_paramName;
            }
        }

        /// <summary>
        /// States whether the value of the parameter is restricted to the values in the options list.
        /// </summary>
        /// <remarks>
        /// The name of the parameter being updated can be used to update GUI inforamtion about the PMC.
        /// </remarks>
        /// <value>Is the parameter vlue restricted to the list?.</value>
        public bool IsRestricted
        {
            get
            {
                return m_isRestricted;
            }
        }
        /// <summary>
        /// States whether the value of the parameter was restricted to the values in the options list prior to the 
        /// change to the resticed to list property.
        /// </summary>
        /// <remarks>
        /// The name of the parameter being updated can be used to update GUI inforamtion about the PMC.
        /// </remarks>
        /// <value>Is the parameter vlue restricted to the list?.</value>
        public bool WasRestricted
        {
            get
            {
                return m_wasRestricted;
            }
        }
    };

    /// <summary>
    /// Represents the method that will handle the changing of whether a paratemer's value is restricted to those in the option list.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void ParameterRestrictedToListChangedHandler(Object sender, ParameterRestrictedToListChangedEventArgs args);

    /// <summary>
    /// Represents the method that will handle the changing of the Kinetic Reaction Chemistry of a PMC.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void KineticReactionsChangedHandler(Object sender, System.EventArgs args);


    /// <summary>
    /// Represents the method that will handle the changing of the Equilibrium Reaction Chemistry of a PMC.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void EquilibriumReactionsChangedHandler(Object sender, System.EventArgs args);

    /// <summary>
    /// Base Class defining the actual Parameter quantity.
    /// </summary>
    [Serializable]
    [System.Runtime.InteropServices.ComSourceInterfaces(typeof(IParameterEvents))]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("F027B4D1-A215-4107-AA75-34E929DD00A5")]
    [System.ComponentModel.DescriptionAttribute("CapeIdentification Interface")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    [System.ComponentModel.TypeConverter(typeof(ParameterTypeConverter))]
    abstract public class CapeParameter : CapeIdentification,
        ICapeParameter,
        ICapeParameterSpec,
        ICapeParameterSpecCOM
    {

        CapeParamMode m_mode = CapeParamMode.CAPE_INPUT_OUTPUT;
        
        /// <summary>
        /// The flag to indicate parameter validation's status.
        /// </summary>
        /// <remarks>
        /// <para>The flag to indicate parameter validation status. It has three 
        /// possible values:</para>
        /// <para>   (i)   notValidated(CAPE_NOT_VALIDATED): The PMC's <c>Validate()</c>
        /// method has not been called after the last time that its value had been 
        /// changed.</para>
        /// <para>   (ii)  invalid(CAPE_INVALID): The last time that the PMC's 
        /// <c>Validate()</c> method was called it returned false.</para>
        /// <para>   (iii) valid(CAPE_VALID): the last time that the PMC's
        /// Validate() method was called it returned true.</para>
        /// </remarks>
        protected CapeValidationStatus m_ValStatus = CapeValidationStatus.CAPE_NOT_VALIDATED;
        
        /// <summary>
        /// Creates a new instance of the abstract parameter base class. 
        /// </summary>
        /// <remarks>
        /// The mode is set to CapeParamMode.CAPE_INPUT_OUTPUT. 
        /// </remarks>
        /// <param name = "name">Sets as the ComponentName of the parameter's ICapeIdentification interface.</param>
        /// <param name = "description">Sets as the ComponentDescription of the parameter's ICapeIdentification interface.</param>
        /// <param name = "mode">Sets the CapeParamMode mode of the parameter.</param>
        public CapeParameter(string name, string description, CapeParamMode mode)
            : base(name, description)
        {
            this.m_mode = mode;
        }

        /// <summary>
        /// Occurs when the user validates the parameter.
        /// </summary>
        /// <remarks><para>Raises the ParameterValidated event through a delegate.</para>        
        /// </remarks>
        public event ParameterValidatedHandler ParameterValidated;
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
        /// <param name = "args">A <see cref = "ParameterValidatedEventArgs">ParameterValidatedEventArgs</see> that contains information about the event.</param>
        protected void OnParameterValidated(ParameterValidatedEventArgs args)
        {
            if (ParameterValidated != null)
            {
                ParameterValidated(this, args);
            }
        }

        /// <summary>
        /// Gets the Specification for this Parameter
        /// </summary>
        /// <remarks>
        /// Gets the specification of the parameter. The Get method returns the 
        /// specification as an interface to the correct specification type.
        /// </remarks>
        /// <value>
        /// An object implementing the <see cref = "ICapeParameterSpec"/>, as well as the
        /// appropraite specification for the parameter type, <see cref = "ICapeRealParameterSpec"/> ,
        /// <see cref = "ICapeIntegerParameterSpec"/> , <see cref = "ICapeBooleanParameterSpec"/> , 
        /// <see cref = "ICapeOptionParameterSpec"/> , or <see cref = "ICapeArrayParameterSpec"/> .
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        Object ICapeParameter.Specification
        {
            get
            {
                return this;
            }
        }

        /// <summary>
        /// Occurs when the user changes of the value of the parameter.
        /// </summary>
        /// <remarks><para>Raises the ParameterValueChanged event through a delegate.</para>        
        /// </remarks>
        public event ParameterValueChangedHandler ParameterValueChanged;
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
        /// <param name = "args">A <see cref = "OnParameterValueChanged">OnParameterValueChanged</see> that contains information about the event.</param>
        protected void OnParameterValueChanged(ParameterValueChangedEventArgs args)
        {
            if (ParameterValueChanged != null)
            {
                ParameterValueChanged(this, args);
            }
        }
        
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
        virtual public Object value
        {
            get;
            set;
        }


        /// <summary>
        /// Gets the dimensionality of the parameter.
        /// </summary>
        /// <remarks>
        /// Physical dimensions are only applicable to real-valued parameters.
        /// </remarks>
        /// <value>
        /// Null pointer.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        virtual public double[] Dimensionality
        {
            get
            {
                return (double[])null;
            }
        }

        /// <summary>
        /// Gets the dimensionality of the parameter.
        /// </summary>
        /// <remarks>
        /// Physical dimensions are only applicable to real-valued parameters.
        /// </remarks>
        /// <value>
        /// Null pointer.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.BrowsableAttribute(false)]
        Object ICapeParameterSpecCOM.Dimensionality
        {
            get
            {
                return null;
            }
        }


        /// <summary>
        /// Gets the flag to indicate parameter validation's status.
        /// </summary>
        /// <remarks>
        /// <para>Gets the flag to indicate parameter validation status. It has three 
        /// possible values:</para>
        /// <para>   (i)   notValidated(CAPE_NOT_VALIDATED): The PMC's <c>Validate()</c>
        /// method has not been called after the last time that its value had been 
        /// changed.</para>
        /// <para>   (ii)  invalid(CAPE_INVALID): The last time that the PMC's 
        /// <c>Validate()</c> method was called it returned false.</para>
        /// <para>   (iii) valid(CAPE_VALID): the last time that the PMC's
        /// Validate() method was called it returned true.</para>
        /// </remarks>
        /// <value>The validity staus of the parameter, either valid, invalid, or "not validated".</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.Category("ICapeParameter")]
        public CapeValidationStatus ValStatus
        {
            get
            {
                return m_ValStatus;
            }
        }

        /// <summary>
        /// Occurs when the user changes of the default value of the parameter changes.
        /// </summary>
        /// <remarks><para>Raises the ParameterDefaultValueChanged event through a delegate.</para>        
        /// </remarks>
        public event ParameterDefaultValueChangedHandler ParameterDefaultValueChanged;
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
        /// <param name = "args">A <see cref = "OnParameterDefaultValueChanged">OnParameterDefaultValueChanged</see> that contains information about the event.</param>
        protected void OnParameterDefaultValueChanged(ParameterDefaultValueChangedEventArgs args)
        {
            if (ParameterDefaultValueChanged != null)
            {
                ParameterDefaultValueChanged(this, args);
            }
            NotifyPropertyChanged("DefaultValue");
        }
        
        /// <summary>
        /// Occurs when the user changes of the mode of the parameter changes.
        /// </summary>
        /// <remarks><para>Raises the ParameterModeChanged event through a delegate.</para>        
        /// </remarks>
        public event ParameterModeChangedHandler ParameterModeChanged;
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
        /// <param name = "args">A <see cref = "ParameterModeChangedEventArgs">ParameterModeChangedEventArgs</see> that contains information about the event.</param>
        protected void OnParameterModeChanged(ParameterModeChangedEventArgs args)
        {
            if (ParameterModeChanged != null)
            {
                ParameterModeChanged(this, args);
            }
        }

        /// <summary>
        /// Gets and sets the mode of the parameter.
        /// </summary>
        /// <remarks>
        /// <para>Modes of parameters. It allows the following values:</para>
        /// <para>   (i)   Input (CAPE_INPUT): the Unit(or whichever owner component) will use 
        /// its value to calculate.</para>
        /// <para>   (ii)  Output (CAPE_OUTPUT): the Unit will place in the parameter a result 
        /// of its calculations.</para>
        /// <para>   (iii) Input-Output (CAPE_INPUT_OUTPUT): the user inputs an 
        /// initial estimation value and the user outputs a calculated value.</para>
        /// </remarks>
        /// <value>The mode of the parameter, input, output, or input/output.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.ComponentModel.Category("ICapeParameter")]
        public CapeParamMode Mode
        {
            get
            {
                return m_mode;
            }
            set
            {
                ParameterModeChangedEventArgs args = new ParameterModeChangedEventArgs(this.ComponentName, m_mode, value);
                OnParameterModeChanged(args);
                m_mode = value;
                this.NotifyPropertyChanged("Mode");
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
        abstract public bool Validate(ref String message);



        /// <summary>
        /// Occurs when the user changes of the parameter value is reset to the default value.
        /// </summary>
        /// <remarks><para>Raises the ParameterReset event through a delegate.</para>        
        /// </remarks>
        public event ParameterResetHandler ParameterReset;
        /// <summary>
        /// Occurs when the user resets a parameter.
        /// </summary>
        /// <remarks><para>Raising an event invokes the event handler through a delegate.</para>
        /// <para>The <c>OnParameterReset</c> method also allows derived classes to handle the event without attaching a delegate. This is the preferred 
        /// technique for handling the event in a derived class.</para>
        /// <para>Notes to Inheritors: </para>
        /// <para>When overriding <c>OnParameterReset</c> in a derived class, be sure to call the base class's <c>OnParameterReset</c> method so that registered 
        /// delegates receive the event.</para>
        /// </remarks>
        /// <param name = "args">A <see cref = "ParameterResetEventArgs">ParameterResetEventArgs</see> that contains information about the event.</param>
        protected void OnParameterReset(ParameterResetEventArgs args)
        {
            if (ParameterReset != null)
            {
                ParameterReset(this, args);
            }
        }

        /// <summary>
        /// Sets the value of the parameter to its default value.
        /// </summary>
        /// <remarks>
        ///  This method sets the parameter's value to the default value.
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        abstract public void Reset();


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
        [System.ComponentModel.Category("ICapeParameterSpec")]
        abstract public CapeParamType Type
        {
            get;
        }
 
    }
}
//#endif // _PARAMETER_IDL_
