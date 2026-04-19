using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{

    /// <summary>
    /// This is the abstract base class for all .Net based CAPE-OPEN exception classes.
    /// </summary>
    /// <remarks>
    /// <para>
    /// One of the principal advantages of .NET over COM is the additional information 
    /// included in exception handling. In COM, exceptions were handled through returning 
    /// an HRESULT value, which is an integer that indicated whether the function call had
    /// successfully returned (Rogerson, 1997). Because the HRESULT value was a 32-bit 
    /// integer, it could indicate more information than simply success or failure, but it
    /// was limited in that it did not include descriptive information about the exception
    /// that occurred.</para>
    /// <para>Under .NET, an application exception class is available 
    /// (System.ApplicationException) that can be used to provide information such as a message and the source 
    /// of the exception. The CAPE-OPEN exception definitions all derive from an 
    /// ECapeRoot interface (Belaud et al, 2001). In the current implementation of the 
    /// CAPE-OPEN exception classes, all exception classes derive from the 
    /// CapeUserException class, which itself is derived from the .NET 
    /// System.ApplicationException class. The CapeUserException class exposes the 
    /// <see c = "ECapeRoot"/> and <see c = "ECapeUser"/> interfaces. In this way, 
    /// all exceptions that are raised by the process modeling components can be caught 
    /// either as a CapeRootException or as a System.ApplicationException in addition to 
    /// being caught as the derived exception type. </para>
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("28686562-77AD-448f-8A41-8CF9C3264A3E")]
    [System.ComponentModel.Description("")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public abstract class CapeUserException : System.ApplicationException,
        ECapeRoot,
        ECapeUser
    {

        /// <summary>
        /// The name of the exception interface for the exception being thrown.
        /// </summary>
        /// <remarks>
        /// The m_interfaceName field is set in the <see c = "Initialize">Initialize</see> method for the exception. Any exception
        /// that derives from the CapeUserException class will need to set this value in the Initialize method.
        /// </remarks>
        protected String m_interfaceName;
        /// <summary>
        /// The name of the exception being thrown.
        /// </summary>
        /// <remarks>
        /// The m_name field is set in the <see c = "Initialize">Initialize</see> method for the exception. Any exception
        /// that derives from the CapeUserException class will need to set this value in the Initialize method.
        /// </remarks>
        protected String m_name;
        /// <summary>
        /// The description of the exception being thrown.
        /// </summary>
        /// <remarks>
        /// The m_name field is set in the <see c = "Initialize">Initialize</see> method for the exception. Any exception
        /// that derives from the CapeUserException class will need to set this value in the Initialize method.
        /// </remarks>
        protected String m_description;

        /// <summary>
        /// Initializes a new instance of the CapeUserException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeUserException()
            : base()
        {
            m_description = "An application error has occurred.";
            this.Initialize();
        }
        /// <summary>
        /// Initializes a new instance of the CapeUserException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeUserException(String message)
            : base(message)
        {
            m_description = message;
            this.Initialize();
        }

        /// <summary>
        /// Initializes a new instance of the CapeUserException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeUserException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context)
            : base(info, context)
        {
            this.Initialize();
        }

        /// <summary>
        /// Initializes a new instance of the CapeUserException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeUserException(String message, System.Exception inner)
            : base(message, inner)
        {
            m_description = message;
            this.Initialize();
        }


        /// <summary>
        /// A virtual abstract function that is inherieted by derived classes to 
        /// initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Derived classes should implement this class and set the values of the HResult, interface name and exception name.</para>
        /// <code>
        /// virtual void Initialize() override 
        /// {
        ///  HResult = (int)CapeErrorInterfaceHR.ECapeUnknownHR;
        ///  m_interfaceName = "ECapeUnknown";
        ///  m_name = "CUnknownException";
        /// }
        /// </code>
        /// </remarks>
        protected abstract void Initialize();

        /// <summary>
        ///	The function that controls COM registration. 
        /// </summary>
        /// <remarks>
        ///	This function adds the registration keys specified in the CAPE-OPEN Method and
        /// Tools specifications. In particular, it indicates that this unit operation implements
        /// the CAPE-OPEN Unit Operation Category Identification. It also adds the CapeDescription
        /// registry keys using the <see c ="CapeNameAttribute"/>, <see c ="CapeDescriptionAttribute"/>, <see c ="CapeVersionAttribute"/>
        /// <see c ="CapeVendorURLAttribute"/>, <see c ="CapeHelpURLAttribute"/>, 
        /// and <see c ="CapeAboutAttribute"/> attributes.
        /// </remarks>
        /// <param name = "t">The type of the class being registered.</param> 
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s), specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.ComRegisterFunction()]
        public static void RegisterFunction(Type t) { }
        /// <summary>
        ///	This function controls the removal of the class from the COM registry when the class is unistalled. 
        /// </summary>
        /// <remarks>
        ///	The method will remove all subkeys added to the class' regristration, including the CAPE-OPEN
        /// specific keys added in the <see c ="RegisterFunction"/> method.
        /// </remarks>
        /// <param name = "t">The type of the class being unregistered.</param> 
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s), specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.ComUnregisterFunction()]
        public static void UnregisterFunction(Type t) { }
        // ECapeRoot method
        // returns the message string in the System.ApplicationException.
        /// <summary>
        /// The name of the exception being thrown.
        /// </summary>
        /// <remarks>
        /// The name of the exception being thrown.
        /// </remarks>
        /// <value>
        /// The name of the exception being thrown.
        /// </value>
        public String Name
        {
            get
            {
                return m_name;
            }
        }

        /// <summary>
        /// Code to designate the subcategory of the error. 
        /// </summary>
        /// <remarks>
        /// The assignment of values is left to each implementation. So that is a 
        /// proprietary code specific to the CO component provider. By default, set to 
        /// the CAPE-OPEN error HRESULT <see c = "CapeErrorInterfaceHR"/>.
        /// </remarks>
        /// <value>
        /// The HRESULT value for the exception.
        /// </value>
        public int code
        {
            get
            {
                return this.HResult;
            }
        }

        /// <summary>
        /// The description of the error.
        /// </summary>
        /// <remarks>
        /// The error description can include a more verbose description of the condition that
        /// caused the error.
        /// </remarks>
        /// <value>
        /// A string description of the exception.
        /// </value>
        public String description
        {
            get
            {
                return m_description;
            }

            set
            {
                m_description = value;
            }
        }

        /// <summary>
        /// The scope of the error.
        /// </summary>
        /// <remarks>
        /// This property provides a list of packages where the error occurred. 
        /// For example <see c = "ICapeIdentification"/>.
        /// </remarks>
        /// <value>The source of the error.</value>
        public String scope
        {
            get
            {
                return this.Source;
            }
        }

        /// <summary>
        /// The name of the interface where the error is thrown. This is a mandatory field."
        /// </summary>
        /// <remarks>
        /// The interface that the error was thrown.
        /// </remarks>
        /// <value>The name of the interface.</value>
        public String interfaceName
        {
            get
            {
                return this.m_interfaceName;
            }
            set
            {
                m_interfaceName = value;
            }
        }

        /// <summary>
        /// The name of the operation where the error is thrown. This is a mandatory field.
        /// </summary>
        /// <remarks>
        /// This field provides the name of the operation being perfomed when the exception was raised.
        /// </remarks>
        /// <value>The operation name.</value>
        public String operation
        {
            get
            {
                return this.StackTrace;
            }
        }

        /// <summary>
        /// An URL to a page, document, web site, where more information on the error can be found. The content of this information is obviously implementation dependent.
        /// </summary>
        /// <remarks>
        /// This field provides an internet URL where more information about the error can be found.
        /// </remarks>
        /// <value>The URL.</value>
        public String moreInfo
        {
            get
            {
                return this.HelpLink;
            }
        }
    };


    /// <summary>
    /// This exception is raised when other error(s), specified by the operation, do not suit.
    /// </summary>
    /// <remarks>
    /// <para>
    /// A standard exception that can be thrown by a CAPE-OPEN object to indicate that the error
    /// that occurred was not one that was suitable for any of the other errors supported by the object. </para>
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("B550B2CA-6714-4e7f-813E-C93248142410")]
    [System.ComponentModel.Description("")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeUnknownException : CapeUserException,
        ECapeUnknown
    {

        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            this.HResult = unchecked((int)CapeErrorInterfaceHR.ECapeUnknownHR);
            m_interfaceName = "ECapeUnknown";
            m_name = "CUnknownException";
        }


        /// <summary>
        /// Initializes a new instance of the CapeUnknownException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeUnknownException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeUnknownException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeUnknownException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeUnknownException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeUnknownException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeUnknownException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeUnknownException(String message, System.Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// This exception is raised when other error(s), specified by the operation, do not suit.
    /// </summary>
    /// <remarks>
    /// <para>
    /// A standard exception that can be thrown by a CAPE-OPEN object to indicate that the error
    /// that occurred was not one that was suitable for any of the other errors supported by the object. </para>
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisible(true)]
    [System.Runtime.InteropServices.Guid("16049506-E086-4baf-9905-9ED13D50D0E3")]
    [System.ComponentModel.Description("")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeUnexpectedException : CapeUserException
    {

        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)0x8000ffff);
            m_interfaceName = "IPersistStreamInit";
            m_name = "CUnexpectedException";
        }


        /// <summary>
        /// Initializes a new instance of the CapeUnknownException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeUnexpectedException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeUnknownException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeUnexpectedException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeUnknownException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeUnexpectedException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeUnknownException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeUnexpectedException(String message, System.Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// The base class of the errors hierarchy related to any data.
    /// </summary>
    /// <remarks>
    /// <para>
    /// The CapeDataException class is a base class for errors related to data. The data are the 
    /// arguments of operations, the parameters coming from the Parameter Common Interface 
    /// and information on licence key.	
    /// </para>
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("53551E7C-ECB2-4894-B71A-CCD1E7D40995")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeDataException : CapeUserException,
        ECapeData
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeDataHR);
            m_name = "CapeDataException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeDataException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeDataException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeDataException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeDataException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeDataException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeDataException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeDataException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeDataException(String message, Exception inner) : base(message, inner) { }
    };


    /// <summary>
    /// A parameter, which is an object from the Parameter Common Interface, has an invalid status.
    /// </summary>
    /// <remarks>
    /// The name of the invalid parameter, along with the parameter itself are available from the exception.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("667D34E9-7EF7-4ca8-8D17-C7577F2C5B62")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeBadCOParameter : CapeDataException,
        ECapeBadCOParameter
    {
        private String m_Parametername;
        private Object m_Parameter;

        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        /// <param name = "parameterName">The name of the parameter with the invalid status.</param>
        /// <param name = "pParameter">The parameter with the invalid status.</param>
        protected void Initialize(String parameterName, Object pParameter)
        {
            m_Parametername = parameterName;
            m_Parameter = (ICapeParameter)(parameter);
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeBadArgumentHR);
            m_interfaceName = "ECapeBadArgument";
            m_name = "CapeBadArgumentException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeBadCOParameter class with the name of the parameter and the parameter that caused the exception. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        /// <param name = "parameterName">The name of the parameter with the invalid status.</param>
        /// <param name = "parameter">The parameter with the invalid status.</param>
        public CapeBadCOParameter(String parameterName, Object parameter) : base() { Initialize(parameterName, parameter); }
        /// <summary>
        /// Initializes a new instance of the CapeBadCOParameter class with a specified error message, the name of the parameter, and the parameter that caused the exception. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        /// <param name = "parameterName">The name of the parameter with the invalid status.</param>
        /// <param name = "parameter">The parameter with the invalid status.</param>
        public CapeBadCOParameter(String message, String parameterName, Object parameter) : base(message) { Initialize(parameterName, parameter); }
        /// <summary>
        /// Initializes a new instance of the CapeBadCOParameter class with serialized data, the name of the parameter, and the parameter that caused the exception. 
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        /// <param name = "parameterName">The name of the parameter with the invalid status.</param>
        /// <param name = "parameter">The parameter with the invalid status.</param>
        public CapeBadCOParameter(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context, String parameterName, Object parameter) : base(info, context) { Initialize(parameterName, parameter); }
        /// <summary>
        /// Initializes a new instance of the CapeBadCOParameter class with a specified error message and a reference to the inner exception, the name of the parameter, and the parameter that caused the exception. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        /// <param name = "parameterName">The name of the parameter with the invalid status.</param>
        /// <param name = "parameter">The parameter with the invalid status.</param>
        public CapeBadCOParameter(String message, Exception inner, String parameterName, Object parameter) : base(message, inner) { Initialize(parameterName, parameter); }

        /// <summary>
        /// The name of the CO parameter that is throwing the exception.
        /// </summary>
        /// <remarks>
        /// This provides the name of the parameter that threw the exception.
        /// </remarks>
        /// <value>The name of the parameter that threw the exception.</value>
        public virtual Object parameter
        {
            get
            {
                return m_Parameter;
            }
        }

        /// <summary>
        /// The name of the CO parameter that is throwing the exception.
        /// </summary>
        /// <remarks>
        /// This provides access to the parameter that threw the exception.
        /// </remarks>
        /// <value>The parameter that threw the exception.</value>
        public virtual String parameterName
        {
            get
            {
                return m_name;
            }
        }
    };

    /// <summary>
    /// An argument value of the operation is not correct.
    /// </summary>
    /// <remarks>
    /// An argument value of the operation is not correct. The position of the 
    /// argument value within the signature of the operation. First argument is as 
    /// position 1.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("D168E99F-C1EF-454c-8574-A8E26B62ADB1")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeBadArgumentException : CapeDataException,
        ECapeBadArgument,
        ECapeBadArgument093
    {
        private int m_Position;

        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        /// <param name = "position">The position of the argument value within the signature of the operation. First argument is as position 1.</param>
        protected void Initialize(int position)
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeBadArgumentHR);
            m_interfaceName = "ECapeBadArgument";
            m_name = "CapeBadArgumentException";
            m_Position = position;
        }

        /// <summary>
        /// Initializes a new instance of the CapeBadArgumentException class with the position of the error. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        /// <param name = "position">The position of the argument value within the signature of the operation. First argument is as position 1.</param>
        public CapeBadArgumentException(int position) : base() { Initialize(position); }
        /// <summary>
        /// Initializes a new instance of the CapeBadArgumentException class with a specified error message and the position of the error. 
        /// </summary>. 
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        /// <param name = "position">The position of the argument value within the signature of the operation. First argument is as position 1.</param>
        public CapeBadArgumentException(String message, int position) : base(message) { Initialize(position); }
        /// <summary>
        /// Initializes a new instance of the CapeBadArgumentException class with serialized data and the position of the error. 
        /// </summary>.
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        /// <param name = "position">The position of the argument value within the signature of the operation. First argument is as position 1.</param>
        public CapeBadArgumentException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context, int position) : base(info, context) { Initialize(position); }
        /// <summary>
        /// Initializes a new instance of the CapeBadArgumentException class with a specified error message and 
        /// a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        /// <param name = "position">The position of the argument value within the signature of the operation. First argument is as position 1.</param>
        public CapeBadArgumentException(String message, Exception inner, int position) : base(message, inner) { Initialize(position); }


        /// <summary>
        /// The position of the argument value within the signature of the operation. First argument is as position 1.
        /// </summary>
        /// <remarks>
        /// This provides the location of the invalid argument in the argument list for the function call.
        /// </remarks>
        /// <value>The position of the argument that is bad. The first argument is 1.
        /// </value>
        public virtual short position
        {
            get
            {
                return (short)m_Position;
            }
        }

        /// <summary>
        /// The position of the argument value within the signature of the operation. First argument is as position 1.
        /// </summary>
        /// <remarks>
        /// This provides the location of the invalid argument in the argument list for the function call.
        /// </remarks>
        /// <value>The position of the argument that is bad. The first argument is 1.
        /// </value>
        int ECapeBadArgument093.position
        {
            get
            {
                return (int)m_Position;
            }
        }
    };


    /// <summary>
    /// This is an abstract class that allows derived classes to provide information 
    /// about error that result from values that are outside of their bounds. It can be raised 
    /// to indicate that the value of either a method argument or the value of a object 
    /// parameter is out of range.
    /// </summary>
    /// <remarks>
    /// <para>CapeBoundariesException is a "utility" class which factorises a state which describes the value, its type and its boundaries.</para>
    /// <para>This is an abstract class. No real error can be raised from this class.</para>
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("62B1EE2F-E488-4679-AFA3-D490694D6B33")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public abstract class CapeBoundariesException : CapeUserException,
        ECapeBoundaries
    {
        private double m_Lower;
        private double m_Upper;
        private double m_Value;
        private String m_Type;

        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        /// <param name = "LowerBound">The value of the lower bound.</param>
        /// <param name = "UpperBound">The value of the upper bound.</param>
        /// <param name = "value">The current value which has led to an error.</param>
        /// <param name = "type">The type/nature of the value. The value could represent a thermodynamic property, a number of tables in a database, a quantity of memory, ...</param>
        protected void SetBoundaries(double LowerBound, double UpperBound, double value, String type)
        {
            m_Lower = LowerBound;
            m_Upper = UpperBound;
            m_Value = value;
            m_Type = type;
        }

        /// <summary>
        /// Initializes a new instance of the CapeBoundariesException class with the lower bound, upper bound, value, type, and position of the parameter that is the cause of this exception. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        /// <param name = "LowerBound">The value of the lower bound.</param>
        /// <param name = "UpperBound">The value of the upper bound.</param>
        /// <param name = "value">The current value which has led to an error.</param>
        /// <param name = "type">The type/nature of the value. The value could represent a thermodynamic property, a number of tables in a database, a quantity of memory, ...</param>
        public CapeBoundariesException(double LowerBound, double UpperBound, double value, String type)
            : base()
        {
            SetBoundaries(LowerBound, UpperBound, value, type);
        }
        /// <summary>
        /// Initializes a new instance of the CapeBoundariesException class with a specified error message, the lower bound, upper bound, value, type, and position of the parameter that is the cause of this exception. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        /// <param name = "LowerBound">The value of the lower bound.</param>
        /// <param name = "UpperBound">The value of the upper bound.</param>
        /// <param name = "value">The current value which has led to an error.</param>
        /// <param name = "type">The type/nature of the value. The value could represent a thermodynamic property, a number of tables in a database, a quantity of memory, ...</param>
        public CapeBoundariesException(String message, double LowerBound, double UpperBound, double value, String type)
            : base(message)
        {
            SetBoundaries(LowerBound, UpperBound, value, type);
        }
        /// <summary>
        /// Initializes a new instance of the CapeBoundariesException class with serialized data, the lower bound, upper bound, value, type, and position of the parameter that is the cause of this exception. 
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        /// <param name = "LowerBound">The value of the lower bound.</param>
        /// <param name = "UpperBound">The value of the upper bound.</param>
        /// <param name = "value">The current value which has led to an error.</param>
        /// <param name = "type">The type/nature of the value. The value could represent a thermodynamic property, a number of tables in a database, a quantity of memory, ...</param>
        public CapeBoundariesException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context, double LowerBound, double UpperBound, double value, String type)
            : base(info, context)
        {
            SetBoundaries(LowerBound, UpperBound, value, type);
        }
        /// <summary>
        /// Initializes a new instance of the CapeBoundariesException class with a specified error message, the lower bound, upper bound, value, type and position of the parameter, and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        /// <param name = "LowerBound">The value of the lower bound.</param>
        /// <param name = "UpperBound">The value of the upper bound.</param>
        /// <param name = "value">The current value which has led to an error.</param>
        /// <param name = "type">The type/nature of the value. The value could represent a thermodynamic property, a number of tables in a database, a quantity of memory, ...</param>
        public CapeBoundariesException(String message, Exception inner, double LowerBound, double UpperBound, double value, String type)
            : base(message, inner)
        {
            SetBoundaries(LowerBound, UpperBound, value, type);
        }

        /// <summary>
        /// The value of the lower bound.
        /// </summary>
        /// <remarks>
        /// <para>This provides the user with the acceptable lower bounds of the argument.</para>
        /// </remarks>
        /// <value>The lower bound for the argument.</value>
        public double lowerBound
        {
            get
            {
                return m_Lower;
            }
        }

        /// <summary>
        /// The value of the upper bound.
        /// </summary>
        /// <remarks>
        /// <para>This provides the user with the acceptable upper bounds of the argument.</para>
        /// </remarks>
        /// <value>The upper bound for the argument.</value>
        public double upperBound
        {
            get
            {
                return m_Upper;
            }
        }

        /// <summary>
        /// The current value which has led to an error.
        /// </summary>
        /// <remarks>
        /// <para>This provides the user with the value that caused the error condition.</para>
        /// </remarks>
        /// <value>The value that resulted in the error condition.</value>
        public double value
        {
            get
            {
                return m_Value;
            }
        }

        /// <summary>
        /// The type/nature of the value. 
        /// </summary>
        /// <remarks>
        /// The value could represent a thermodynamic property, a number of tables in a database, a quantity of memory, ..."
        /// </remarks>
        /// <value>A string that indicates the anture or type of the value required.</value>
        public String type
        {
            get
            {
                return m_Type;
            }
        }
    };

    /// <summary>
    /// An argument value is outside of the bounds..
    /// </summary>
    /// <remarks>
    /// <para>This class is derived from the <see c = "CapeBoundariesException">CapeBoundariesException</see> class.
    /// It is used to indicate that one of the parameters is outside of its bounds.</para>
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("4438458A-1659-48c2-9138-03AD8B4C38D8")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeOutOfBoundsException : CapeBoundariesException,
        ECapeOutOfBounds,
        ECapeBadArgument,
        ECapeBadArgument093,
        ECapeData
    {
        private int m_Position;

        /// <summary>
        /// The initialize method for all classes derived from CapeOutOfBoundsException need to include the
        /// pertinent information related to the boundaries.
        /// </summary>
        /// <remarks>
        /// <para>This method is sealed so that classes that derive from CapeOutOfBoundsException include the required information about the position of the argument.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeOutOfBoundsHR);
            m_interfaceName = "ECapeOutOfBounds";
            m_name = "CapeOutOfBoundsException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeOutOfBoundsException class with the position of the error. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        /// <param name = "position">The position of the argument value within the signature of the operation. First argument is as position 1.</param>
        /// <param name = "LowerBound">The value of the lower bound.</param>
        /// <param name = "UpperBound">The value of the upper bound.</param>
        /// <param name = "value">The current value which has led to an error.</param>
        /// <param name = "type">The type/nature of the value. The value could represent a thermodynamic property, a number of tables in a database, a quantity of memory, ...</param>
        public CapeOutOfBoundsException(int position, double LowerBound, double UpperBound, double value, String type) :
            base(LowerBound, UpperBound, value, type)
        {
            m_Position = position;
        }
        /// <summary>
        /// Initializes a new instance of the CapeOutOfBoundsException class with a specified error message and the position of the error. 
        /// </summary>. 
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        /// <param name = "position">The position of the argument value within the signature of the operation. First argument is as position 1.</param>
        /// <param name = "LowerBound">The value of the lower bound.</param>
        /// <param name = "UpperBound">The value of the upper bound.</param>
        /// <param name = "value">The current value which has led to an error.</param>
        /// <param name = "type">The type/nature of the value. The value could represent a thermodynamic property, a number of tables in a database, a quantity of memory, ...</param>
        public CapeOutOfBoundsException(String message, int position, double LowerBound, double UpperBound, double value, String type) :
            base(message, LowerBound, UpperBound, value, type)
        {
            m_Position = position;
        }
        /// <summary>
        /// Initializes a new instance of the CapeOutOfBoundsException class with serialized data and the position of the error. 
        /// </summary>.
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        /// <param name = "position">The position of the argument value within the signature of the operation. First argument is as position 1.</param>
        /// <param name = "LowerBound">The value of the lower bound.</param>
        /// <param name = "UpperBound">The value of the upper bound.</param>
        /// <param name = "value">The current value which has led to an error.</param>
        /// <param name = "type">The type/nature of the value. The value could represent a thermodynamic property, a number of tables in a database, a quantity of memory, ...</param>
        public CapeOutOfBoundsException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context, int position, double LowerBound, double UpperBound, double value, String type) :
            base(info, context, LowerBound, UpperBound, value, type)
        {
            m_Position = position;
        }
        /// <summary>
        /// Initializes a new instance of the CapeOutOfBoundsException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        /// <param name = "position">The position of the argument value within the signature of the operation. First argument is as position 1.</param>
        /// <param name = "LowerBound">The value of the lower bound.</param>
        /// <param name = "UpperBound">The value of the upper bound.</param>
        /// <param name = "value">The current value which has led to an error.</param>
        /// <param name = "type">The type/nature of the value. The value could represent a thermodynamic property, a number of tables in a database, a quantity of memory, ...</param>
        public CapeOutOfBoundsException(String message, Exception inner, int position, double LowerBound, double UpperBound, double value, String type) :
            base(message, inner, LowerBound, UpperBound, value, type)
        {
            m_Position = position;
        }

        /// <summary>
        /// The position of the argument value within the signature of the operation. First argument is as position 1.
        /// </summary>
        /// <remarks>
        /// This provides the location of the invalid argument in the argument list for the function call.
        /// </remarks>
        /// <value>The position of the argument that is bad. The first argument is 1.
        /// </value>
        public short position
        {
            get
            {
                return (short)m_Position;
            }
        }

        /// <summary>
        /// The position of the argument value within the signature of the operation. First argument is as position 1.
        /// </summary>
        /// <remarks>
        /// This provides the location of the invalid argument in the argument list for the function call.
        /// </remarks>
        /// <value>The position of the argument that is bad. The first argument is 1.
        /// </value>
        int ECapeBadArgument093.position
        {
            get
            {
                return (int)m_Position;
            }
        }
    };

    /// <summary>
    /// The base class of the errors hierarchy related to calculations.
    /// </summary>
    /// <remarks>
    /// This class is used to indicate that an error occurred in the performance of a calculation. 
    /// Other calculation-related classes such as 
    /// <see c = "CapeFailedInitialisationException">CapeOpen.CapeFailedInitialisationException</see>, 
    /// <see c = "CapeOutOfResourcesException">CapeOpen.CapeOutOfResourcesException</see>, 
    /// <see c = "CapeSolvingErrorException">CapeOpen.CapeSolvingErrorException</see>, 
    /// <see c = "CapeBadInvOrderException">CapeOpen.CapeBadInvOrderException</see>, 
    /// <see c = "CapeInvalidOperationException">CapeOpen.CapeInvalidOperationException</see>, 
    /// <see c = "CapeNoMemoryException">CapeOpen.CapeNoMemoryException</see>, and 
    /// <see c = "CapeTimeOutException">CapeOpen.CapeTimeOutException</see> 
    /// derive from this class.
    /// </remarks>
    [
        Serializable,
        System.Runtime.InteropServices.Guid("9D416BF5-B9E3-429a-B13A-222EE85A92A7"),
        System.Runtime.InteropServices.ComVisibleAttribute(true),
        System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)
    ]
    public class CapeComputationException : CapeUserException,
        ECapeComputation
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            this.HResult = unchecked((int)CapeErrorInterfaceHR.ECapeComputationHR);
            m_interfaceName = "ECapeComputation";
            m_name = "CapeComputationException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeComputationException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeComputationException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeComputationException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeComputationException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeComputationException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeComputationException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeComputationException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeComputationException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// This exception is thrown when necessary initialisation has not been performed or has failed.
    /// </summary>
    /// <remarks>
    /// The pre-requisites operations are not valid. The necessary initialisation has not been performed or has failed.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("E407595C-6D1C-4b8c-A29D-DB0BE73EFDDA")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeFailedInitialisationException : CapeComputationException,
        ECapeFailedInitialisation
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeFailedInitialisationHR);
            m_interfaceName = "ECapeFailedInitialisation";
            m_name = "CapeFailedInitialisationException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeFailedInitialisationException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeFailedInitialisationException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeFailedInitialisationException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeFailedInitialisationException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeFailedInitialisationException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeFailedInitialisationException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeFailedInitialisationException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeFailedInitialisationException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// The base class of the errors hierarchy related to the current implementation.
    /// </summary>
    /// <remarks>
    /// This class is used to indicate that an error occurred in the with the implementation of an object. 
    /// The implemenation-related classes such as 
    /// <see c = "CapeNoImplException ">CapeOpen.CapeNoImplException </see> and 
    /// <see c = "CapeLimitedImplException ">CapeOpen.CapeLimitedImplException </see>
    /// derive from this class.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("7828A87E-582D-4947-9E8F-4F56725B6D75")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeImplementationException : CapeUserException,
        ECapeImplementation
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeImplementationHR);
            m_interfaceName = "ECapeImplementation";
            m_name = "CapeImplementationException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeImplementationException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeImplementationException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeImplementationException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeImplementationException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeImplementationException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeImplementationException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeImplementationException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeImplementationException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// An invalid argument value was passed. For instance the passed name of 
    /// the phase does not belong to the CO Phase List.
    /// </summary>
    /// <remarks>
    /// An argument value of the operation is invalid. The position of the 
    /// argument value within the signature of the operation. First argument is as 
    /// position 1.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("B30127DA-8E69-4d15-BAB0-89132126BAC9")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeInvalidArgumentException : CapeBadArgumentException,
        ECapeInvalidArgument
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeInvalidArgumentHR);
            m_interfaceName = "ECapeInvalidArgument";
            m_name = "CapeInvalidArgumentException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeInvalidArgumentException class with the position of the error. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        /// <param name = "position">The position of the argument value within the signature of the operation. First argument is as position 1.</param>
        public CapeInvalidArgumentException(int position) : base(position) { }
        /// <summary>
        /// Initializes a new instance of the CapeInvalidArgumentException class with a specified error message and the position of the error. 
        /// </summary>. 
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        /// <param name = "position">The position of the argument value within the signature of the operation. First argument is as position 1.</param>
        public CapeInvalidArgumentException(String message, int position) : base(message, position) { }
        /// <summary>
        /// Initializes a new instance of the CapeInvalidArgumentException class with serialized data and the position of the error. 
        /// </summary>.
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        /// <param name = "position">The position of the argument value within the signature of the operation. First argument is as position 1.</param>
        public CapeInvalidArgumentException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context, int position) : base(info, context, position) { }
        /// <summary>
        /// Initializes a new instance of the CapeInvalidArgumentException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        /// <param name = "position">The position of the argument value within the signature of the operation. First argument is as position 1.</param>
        public CapeInvalidArgumentException(String message, Exception inner, int position) : base(message, inner, position) { }
    };

    /// <summary>
    /// This operation is not valid in the current context.
    /// </summary>
    /// <remarks>
    /// This exception is thrown when an operation is attempted that is not valid in the current context.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("C0B943FE-FB8F-46b6-A622-54D30027D18B")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeInvalidOperationException : CapeComputationException,
        ECapeInvalidOperation
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeInvalidOperationHR);
            m_interfaceName = "ECapeInvalidOperation";
            m_name = "CapeInvalidOperationException";
        }
        /// <summary>
        /// Initializes a new instance of the CapeInvalidOperationException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeInvalidOperationException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeInvalidOperationException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeInvalidOperationException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeInvalidOperationException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeInvalidOperationException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeInvalidOperationException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeInvalidOperationException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// The necessary pre-requisite operation has not been called prior to the operation request.
    /// </summary>
    /// <remarks>
    /// The specified prerequiste operation must be called prior to the operation throwing this exception.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("07EAD8B4-4130-4ca6-94C1-E8EC4E9B23CB")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeBadInvOrderException : CapeComputationException,
        ECapeBadInvOrder
    {
        private String m_requestedOperation;

        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeBadInvOrderHR);
            m_interfaceName = "ECapeBadInvOrder";
            m_name = "CapeBadInvOrderException";
        }


        /// <summary>
        /// Initializes a new instance of the CapeBadInvOrderException class with a specified error message and the name of the operation raising the exception. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "operation">The necessary prerequisite operation.</param>
        public CapeBadInvOrderException(String operation)
            : base()
        {
            m_requestedOperation = operation;
        }
        /// <summary>
        /// Initializes a new instance of the CapeBadInvOrderException class with a specified error message and the name of the operation raising the exception. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        /// <param name = "operation">The necessary prerequisite operation.</param>
        public CapeBadInvOrderException(String message, String operation)
            : base(message)
        {
            m_requestedOperation = operation;
        }
        /// <summary>
        /// Initializes a new instance of the CapeBadInvOrderException class with serialized data and the name of the operation raising the exception.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        /// <param name = "operation">The necessary prerequisite operation.</param>
        public CapeBadInvOrderException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context, String operation)
            : base(info, context)
        {
            m_requestedOperation = operation;
        }
        /// <summary>
        /// Initializes a new instance of the CapeBadInvOrderException class with a specified error message, a erence to the inner exception, and the name of the operation raising the exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        /// <param name = "operation">The necessary prerequisite operation.</param>
        public CapeBadInvOrderException(String message, Exception inner, String operation)
            : base(message, inner)
        {
            m_requestedOperation = operation;
        }

        /// <summary>
        /// The necessary prerequisite operation.
        /// </summary>
        /// <remarks>
        /// <para>The prerquisite operation must be called prior to calling the current operation.</para>
        /// </remarks>
        /// <value>The name of the necessary prerequisite operation.
        /// </value>
        public String requestedOperation
        {
            get
            {
                return m_requestedOperation;
            }
        }
    };

    /// <summary>
    /// An operation can not be completed because the licence agreement is not respected.
    /// </summary>
    /// <remarks>
    /// Of course, this type of error could also appear outside the CO scope. In this case, 
    /// the error does not belong to the CO error handling. It is specific to the platform.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("CF4C55E9-6B0A-4248-9A33-B8134EA393F6")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeLicenceErrorException : CapeDataException,
        ECapeLicenceError
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeLicenceErrorHR);
            m_interfaceName = "ECapeLicenceError";
            m_name = "CapeLicenceErrorException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeLicenceErrorException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeLicenceErrorException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeLicenceErrorException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeLicenceErrorException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeLicenceErrorException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeLicenceErrorException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeLicenceErrorException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeLicenceErrorException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// The limit of the implementation has been violated.
    /// </summary>
    /// <remarks>
    /// <para>An operation may be partially implemented for example a Property Package could 
    /// implement TP flash but not PH flash. If a caller requests for a PH flash, then 
    /// this error indicates that some flash calculations are supported but not the 
    /// requested one.
    /// </para>
    /// <para>The factory can only create one instance (because the component is an 
    /// evaluation copy), when the caller requests for a second creation this error shows 
    /// that this implementation is limited.
    /// </para>
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("5E6B74A2-D603-4e90-A92F-608E3F1CD39D")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeLimitedImplException : CapeImplementationException,
        ECapeLimitedImpl
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeLimitedImplHR);
            m_interfaceName = "ECapeLimitedImpl";
            m_name = "CapeLimitedImplException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeLimitedImplException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeLimitedImplException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeLimitedImplException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeLimitedImplException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeLimitedImplException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeLimitedImplException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeLimitedImplException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeLimitedImplException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// An exception class that indicates that the requested operation has not been implemented by the current object.
    /// </summary>
    /// <remarks>
    /// The operation is “not” implemented even if this operation can be called due 
    /// to the compatibility with the CO standard. That is to say that the operation 
    /// exists but it is not supported by the current implementation.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("1D2488A6-C428-4e38-AFA6-04F2107172DA")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeNoImplException : CapeImplementationException,
        ECapeNoImpl
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeNoImplHR);
            m_interfaceName = "ECapeNoImpl";
            m_name = "CapeNoImplException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeNoImplException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeNoImplException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeNoImplException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeNoImplException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeNoImplException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeNoImplException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeNoImplException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeNoImplException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// An exception class that indicates that the resources required by this operation are not available.
    /// </summary>
    /// <remarks>
    /// The physical resources necessary to the execution of the operation are out of limits.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("42B785A7-2EDD-4808-AC43-9E6E96373616")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeOutOfResourcesException : CapeUserException,
        ECapeOutOfResources,
        ECapeComputation
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeOutOfResourcesHR);
            m_interfaceName = "ECapeOutOfResources";
            m_name = "CapeOutOfResourcesException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeOutOfResourcesException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeOutOfResourcesException()
            : base()
        {
        }
        /// <summary>
        /// Initializes a new instance of the CapeOutOfResourcesException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeOutOfResourcesException(String message)
            : base(message)
        {
        }
        /// <summary>
        /// Initializes a new instance of the CapeOutOfResourcesException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeOutOfResourcesException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context)
            : base(info, context)
        {
        }
        /// <summary>
        /// Initializes a new instance of the CapeOutOfResourcesException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeOutOfResourcesException(String message, Exception inner)
            : base(message, inner)
        {
        }
    };

    /// <summary>
    /// An exception class that indicates that the memory required for this operation is not available.
    /// </summary>
    /// <remarks>
    /// The physical memory necessary to the execution of the operation is out of limit.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("1056A260-A996-4a1e-8BAE-9476D643282B")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeNoMemoryException : CapeUserException,
        ECapeNoMemory
    {

        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeNoMemoryHR);
            m_interfaceName = "ECapeNoMemory";
            m_name = "CapeNoMemoryException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeNoMemoryException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeNoMemoryException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeNoMemoryException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeNoMemoryException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeNoMemoryException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeNoMemoryException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeNoMemoryException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeNoMemoryException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// An exception class that indicates that the a persistence-related error has occurred.
    /// </summary>
    /// <remarks>
    /// The base class of the errors hierarchy related to the persistence.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("3237C6F8-3D46-47ee-B25F-52485A5253D8")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapePersistenceException : CapeUserException,
         ECapePersistence
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapePersistenceHR);
            m_interfaceName = "ECapePersistence";
            m_name = "CapePersistenceException";
        }

        /// <summary>
        /// Initializes a new instance of the CapePersistenceException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapePersistenceException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapePersistenceException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapePersistenceException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapePersistenceException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapePersistenceException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapePersistenceException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapePersistenceException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// An exception class that indicates that the persistence was not found.
    /// </summary>
    /// <remarks>
    /// The requested object, table, or something else within the persistence system was not found.
    /// </remarks>
    [
        Serializable,
        System.Runtime.InteropServices.Guid("271B9E29-637E-4eb0-9588-8A53203A3959"),
        System.Runtime.InteropServices.ComVisibleAttribute(true),
        System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)
    ]
    public class CapePersistenceNotFoundException : CapePersistenceException,
        ECapePersistenceNotFound
    {
        private String m_ItemName;

        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        /// <param name = "itemName">
        /// Name of the item that was not found that is the cause of this exception. 
        /// </param>
        protected void Initialize(String itemName)
        {
            m_ItemName = itemName;
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapePersistenceNotFoundHR);
            m_interfaceName = "ECapePersistenceNotFound";
            m_name = "CapePersistenceNotFoundException";
        }

        /// <summary>
        /// Initializes a new instance of the CapePersistenceNotFoundException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        /// <param name = "itemName">
        /// Name of the item that was not found that is the cause of this exception. 
        /// </param>
        public CapePersistenceNotFoundException(String itemName) : base() { Initialize(itemName); }
        /// <summary>
        /// Initializes a new instance of the CapePersistenceNotFoundException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        /// <param name = "itemName">
        /// Name of the item that was not found that is the cause of this exception. 
        /// </param>
        public CapePersistenceNotFoundException(String message, String itemName) : base(message) { Initialize(itemName); }
        /// <summary>
        /// Initializes a new instance of the CapePersistenceNotFoundException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        /// <param name = "itemName">
        /// Name of the item that was not found that is the cause of this exception. 
        /// </param>
        public CapePersistenceNotFoundException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context, String itemName) : base(info, context) { Initialize(itemName); }
        /// <summary>
        /// Initializes a new instance of the CapePersistenceNotFoundException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        /// <param name = "itemName">
        /// Name of the item that was not found that is the cause of this exception. 
        /// </param>
        public CapePersistenceNotFoundException(String message, Exception inner, String itemName) : base(message, inner) { Initialize(itemName); }

        /// <summary>
        /// Name of the item that was not found that is the cause of this exception. 
        /// </summary>
        /// <remarks>
        /// Name of the item that was not found that is the cause of this exception. 
        /// </remarks>
        /// <value>
        /// Name of the item that was not found that is the cause of this exception. 
        /// </value>
        public String itemName
        {
            get
            {
                return m_ItemName;
            }
        }

    };

    /// <summary>
    /// An exception class that indicates an overflow of internal persistence system.
    /// </summary>
    /// <remarks>
    /// During the persistence process, an overflow of internal persistence system occurred.
    /// </remarks>
    [
        Serializable,
        System.Runtime.InteropServices.Guid("A119DE0B-C11E-4a14-BA5E-9A2D20B15578"),
        System.Runtime.InteropServices.ComVisibleAttribute(true),
        System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)
    ]
    public class CapePersistenceOverflowException : CapeUserException,
        ECapePersistenceOverflow,
        ECapePersistence
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapePersistenceOverflowHR);
            m_interfaceName = "ECapePersistenceOverflow";
            m_name = "CapePersistenceOverflowException";
        }
        /// <summary>
        /// Initializes a new instance of the CapePersistenceOverflowException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapePersistenceOverflowException()
            : base()
        {
        }
        /// <summary>
        /// Initializes a new instance of the CapePersistenceOverflowException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapePersistenceOverflowException(String message)
            : base()
        {
        }
        /// <summary>
        /// Initializes a new instance of the CapePersistenceOverflowException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapePersistenceOverflowException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapePersistenceOverflowException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapePersistenceOverflowException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// An exception class that indicates a severe error occurred within the persistence system.
    /// </summary>
    /// <remarks>
    /// During the persistence process, a severe error occurred within the persistence system.
    /// </remarks>
    [
        Serializable,
        System.Runtime.InteropServices.Guid("85CB2D40-48F6-4c33-BF0C-79CB00684440"),
        System.Runtime.InteropServices.ComVisibleAttribute(true),
        System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)
    ]
    public class CapePersistenceSystemErrorException : CapePersistenceException,
        ECapePersistenceSystemError
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapePersistenceSystemErrorHR);
            m_interfaceName = "ECapePersistenceSystemError";
            m_name = "CapePersistenceSystemErrorException";
        }

        /// <summary>
        /// Initializes a new instance of the CapePersistenceSystemErrorException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapePersistenceSystemErrorException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapePersistenceSystemErrorException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapePersistenceSystemErrorException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapePersistenceSystemErrorException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapePersistenceSystemErrorException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapePersistenceSystemErrorException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapePersistenceSystemErrorException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// The access to something within the persistence system is not authorised.
    /// </summary>
    /// <remarks>
    /// This exception is thrown when the access to something within the persistence system is not authorised.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("45843244-ECC9-495d-ADC3-BF9980A083EB")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeIllegalAccessException : CapePersistenceException,
        ECapeIllegalAccess
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeIllegalAccessHR);
            m_interfaceName = "ECapeIllegalAccess";
            m_name = "CapeIllegalAccessException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeIllegalAccessException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeIllegalAccessException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeIllegalAccessException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeIllegalAccessException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeIllegalAccessException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeIllegalAccessException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeIllegalAccessException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeIllegalAccessException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// An exception class that indicates a numerical algorithm failed for any reason.
    /// </summary>
    /// <remarks>
    /// Indicates that a numerical algorithm failed for any reason.
    /// </remarks>
    [
        Serializable,
        System.Runtime.InteropServices.Guid("F617AFEA-0EEE-4395-8C82-288BF8F2A136"),
        System.Runtime.InteropServices.ComVisibleAttribute(true),
        System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)
    ]
    public class CapeSolvingErrorException : CapeComputationException,
        ECapeSolvingError
    {

        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeSolvingErrorHR);
            m_interfaceName = "ECapeSolvingError";
            m_name = "CapeSolvingErrorException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeSolvingErrorException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeSolvingErrorException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeSolvingErrorException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeSolvingErrorException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeSolvingErrorException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeSolvingErrorException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeSolvingErrorException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeSolvingErrorException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// Exception thrown when the Hessian for the MINLP problem is not available.
    /// </summary>
    /// <remarks>
    /// Exception thrown when the Hessian for the MINLP problem is not available.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.Guid("3044EA08-F054-4315-B67B-4E3CD2CF0B1E")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class CapeHessianInfoNotAvailableException : CapeSolvingErrorException,
        ECapeHessianInfoNotAvailable
    {

        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeHessianInfoNotAvailableHR);
            m_interfaceName = "ECapeHessianInfoNotAvailable";
            m_name = "CapeHessianInfoNotAvailableHR";
        }
        /// <summary>
        /// Initializes a new instance of the CapeHessianInfoNotAvailableException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeHessianInfoNotAvailableException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeHessianInfoNotAvailableException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeHessianInfoNotAvailableException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeHessianInfoNotAvailableException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeHessianInfoNotAvailableException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeHessianInfoNotAvailableException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeHessianInfoNotAvailableException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// Exception thrown when the time-out criterion is reached.
    /// </summary>
    /// <remarks>
    /// Exception thrown when the time-out criterion is reached.
    /// </remarks>
    [
        Serializable,
        System.Runtime.InteropServices.Guid("0D5CA7D8-6574-4c7b-9B5F-320AA8375A3C"),
        System.Runtime.InteropServices.ComVisibleAttribute(true),
        System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)
    ]
    public class CapeTimeOutException : CapeUserException,
        ECapeTimeOut
    {

        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeTimeOutHR);
            m_interfaceName = "ECapeTimeOut";
            m_name = "CapeTimeOutException";
        }

        /// <summary>
        /// Initializes a new instance of the CapeTimeOutException class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeTimeOutException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeTimeOutException class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeTimeOutException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeTimeOutException class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeTimeOutException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeTimeOutException class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeTimeOutException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// A wrapper class for COM-based exceptions.
    /// </summary>
    /// <remarks>
    /// <para>This class can be used when a COM-based CAPE-OPEN component returns a failure HRESULT.
    /// A failure HRESULT indicates an error condition has occurred. This class is used by the
    /// <see c = "COMExceptionHandler"/> to rethrow the COM-based
    /// error condition as a .Net-based exception.</para>
    /// <para>The CAPE-OPEN error handling process chose not to use the COM IErrorInfo API due to
    /// limitation of the Visual Basic programming language at the time that the error
    /// handling protocols were developed. Instead, the CAPE-OPEN error handling protocol 
    /// requires that component in which the error occurs expose the appropriate error
    /// interfaces. In practice, this typically means that all CAPE-OPEN objects
    /// implement the <see c = "ECapeRoot">ECapeRoot</see>,
    /// <see c = "ECapeUser">ECapeUser</see>, 
    /// and sometimes the <see c = "ECapeUnknown">ECapeUnknown</see> error interfaces.</para>
    /// <para>This class wraps the CAPE-OPEN object that threw the exception and creates the 
    /// appropriate .Net exception so users can use the .Net exception handling protocols.</para>
    /// </remarks>
    /// <see c = "COMExceptionHandler">COMExceptionHandler</see> 
    [Serializable]
    [System.Runtime.InteropServices.Guid("31CD55DE-AEFD-44ff-8BAB-F6252DD43F16")]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class COMCapeOpenExceptionWrapper : CapeUserException
    {

        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
        }

        /// <summary>
        /// Creates a new instance of the COMCapeOpenExceptionWrapper class.
        /// </summary>
        /// <remarks>
        /// <para>Creates a .Net based exception wrapper for COM-based CAPE-OPEN componets to 
        /// enable users to utilize .Net structured exception handling.</para>
        /// </remarks>
        /// <param name = "message">The error message text from the COM-based component.</param>
        /// <param name = "exceptionObject">The CAPE-OPEN object that raised the error.</param>
        /// <param name = "HRESULT">The COM HResult value.</param>
        /// <param name = "inner">An inner .Net-based exception obtained from the IErrorInfo
        /// object, if implemented or an accompanying .Net exception.</param>
        public COMCapeOpenExceptionWrapper(String message, Object exceptionObject, int HRESULT, System.Exception inner)
            : base(message, inner)
        {
            this.HResult = HRESULT;
            if (exceptionObject is ECapeRoot)
            {
                m_name = String.Concat("CAPE-OPEN Error: ", ((CapeOpen.ECapeRoot)exceptionObject).Name);
            }

            if (exceptionObject is ECapeUser)
            {
                CapeOpen.ECapeUser user = (CapeOpen.ECapeUser)exceptionObject;
                this.m_description = user.description;
                this.m_interfaceName = user.interfaceName;
                this.Source = user.scope;
                this.HelpLink = user.moreInfo;
            }
        }
    };

    /// <summary>
    /// Exception thrown when a requested theromdynamic property is not available.
    /// </summary>
    /// <remarks>
    /// Exception thrown when a requested theromdynamic property is not available.
    /// </remarks>
    [
        Serializable,
        System.Runtime.InteropServices.Guid("5BA36B8F-6187-4e5e-B263-0924365C9A81"),
        System.Runtime.InteropServices.ComVisibleAttribute(true),
        System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)
    ]
    public class CapeThrmPropertyNotAvailableException : CapeUserException,
        ECapeThrmPropertyNotAvailable
    {
        /// <summary>
        /// Initializes the description, interface name and name fields of this exception.
        /// </summary>
        /// <remarks>
        /// <para>Sets the values of the HResult, interface name and exception name.</para>
        /// </remarks>
        protected override void Initialize()
        {
            HResult = unchecked((int)CapeErrorInterfaceHR.ECapeThrmPropertyNotAvailableHR);
            m_interfaceName = "ECapePersistence";
            m_name = "CapeThrmPropertyNotAvailable";
        }

        /// <summary>
        /// Initializes a new instance of the CapeThrmPropertyNotAvailable class. 
        /// </summary>
        /// <remarks>
        /// This constructor initializes the Message property of the new instance to a 
        /// system-supplied message that describes the error, such as "An application 
        /// error has occurred." This message takes into account the current system 
        /// culture.
        /// </remarks>
        public CapeThrmPropertyNotAvailableException() : base() { }
        /// <summary>
        /// Initializes a new instance of the CapeThrmPropertyNotAvailable class with a specified error message. 
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>This message takes into account the current system culture.</para>
        /// </remarks>
        /// <param name = "message">A message that describes the error.</param>
        public CapeThrmPropertyNotAvailableException(String message) : base(message) { }
        /// <summary>
        /// Initializes a new instance of the CapeThrmPropertyNotAvailable class with serialized data.
        /// </summary>
        /// <remarks> This constructor is called during deserialization to reconstitute the 
        /// exception object transmitted over a stream. For more information, see XML and 
        /// SOAP Serialization.</remarks>
        /// <param name = "info">The object that holds the serialized object data.</param>
        /// <param name = "context">The contextual information about the source or 
        /// destination. </param>
        public CapeThrmPropertyNotAvailableException(System.Runtime.Serialization.SerializationInfo info, System.Runtime.Serialization.StreamingContext context) : base(info, context) { }
        /// <summary>
        /// Initializes a new instance of the CapeThrmPropertyNotAvailable class with a specified error message and a reference to the inner exception that is the cause of this exception.
        /// </summary>
        /// <remarks>
        /// <para>The content of the message parameter is intended to be understood by 
        /// humans. The caller of this constructor is required to ensure that this string 
        /// has been localized for the current system culture.</para>
        /// <para>An exception that is thrown as a direct result of a previous exception 
        /// should include a erence to the previous exception in the InnerException 
        /// property. The InnerException property returns the same value that is passed 
        /// into the constructor, or a null erence if the InnerException property does 
        /// not supply the inner exception value to the constructor.</para>
        /// </remarks>
        /// <param name = "message">The error message string.</param>
        /// <param name = "inner">The inner exception erence.</param>
        public CapeThrmPropertyNotAvailableException(String message, Exception inner) : base(message, inner) { }
    };

    /// <summary>
    /// A helper class for handling exceptions from COM-based CAPE-OPEN components.
    /// </summary>
    /// <remarks>
    /// <para>This class can be used when a COM-based CAPE-OPEN component returns a failure HRESULT.
    /// A failure HRESULT indicates an error condition has occurred. The <see c ="ExceptionForHRESULT">ExceptionForHRESULT</see> 
    /// ormats the .Net-bsed exception object and the COM-based CAPE-OPEN component to rethrow the COM-based
    /// error condition as a .Net-based exception using the <see c = "COMCapeOpenExceptionWrapper">COMCapeOpenExceptionWrapper</see> 
    /// wrapper class.</para>
    /// <para>The CAPE-OPEN error handling process chose not to use the COM IErrorInfo API due to
    /// limitation of the Visual Basic programming language at the time that the error
    /// handling protocols were developed. Instead, the CAPE-OPEN error handling protocol 
    /// requires that component in which the error occurs expose the appropriate error
    /// interfaces. In practice, this typically means that all CAPE-OPEN objects
    /// implement the <see c = "ECapeRoot">ECapeRoot</see>,
    /// <see c = "ECapeUser">ECapeUser</see>, 
    /// and sometimes the <see c = "ECapeUnknown">ECapeUnknown</see> error interfaces.</para>
    /// </remarks>
    /// <see c = "COMCapeOpenExceptionWrapper">COMCapeOpenExceptionWrapper</see> 
    [System.Runtime.InteropServices.ComVisible(false)]
    public class COMExceptionHandler
    {

        /// <summary>
        /// Creates and returns a new instance of the COMCapeOpenExceptionWrapper class.
        /// </summary>
        /// <remarks>
        /// <para>Creates a .Net based exception wrapper for COM-based CAPE-OPEN componets to 
        /// enable users to utilize .Net structured exception handling. This method ormats 
        /// the .Net-bsed exception object and the COM-based CAPE-OPEN component to rethrow 
        /// the COM-based error condition as a .Net-based exception using the 
        /// <see c = "COMCapeOpenExceptionWrapper">COMCapeOpenExceptionWrapper</see> 
        /// wrapper class.</para>
        /// </remarks>
        /// <returns>
        /// The COM-based object that returned the error HRESULT wrapper as the appropriate .Net-based exception.</returns>
        /// <param name = "exceptionObject">The CAPE-OPEN object that raised the error.</param>
        /// <param name = "inner">An inner .Net-based exception obtained from the IErrorInfo
        /// object, if implemented or an accompanying .Net exception.</param>
        /// <see c = "COMCapeOpenExceptionWrapper">COMCapeOpenExceptionWrapper</see> 
        public static System.Exception ExceptionForHRESULT(Object exceptionObject, System.Exception inner)
        {
            int HRESULT = ((System.Runtime.InteropServices.COMException)inner).ErrorCode;
            String message = "Exception thrown by COM PMC.";
            COMCapeOpenExceptionWrapper exception = new COMCapeOpenExceptionWrapper(message, exceptionObject, HRESULT, inner);
            switch (HRESULT)
            {
                case unchecked((int)0x80040501)://ECapeUnknownHR
                    return new CapeUnknownException(message, exception);
                case unchecked((int)0x80040502)://ECapeDataHR
                    return new CapeOpen.CapeDataException(message, exception);
                case unchecked((int)0x80040503)://ECapeLicenceErrorHR = 0x80040503 ,
                    return new CapeOpen.CapeLicenceErrorException(message, exception);
                case unchecked((int)0x80040504)://ECapeBadCOParameterHR = 0x80040504 ,
                    return new CapeOpen.CapeBadCOParameter(message, exception);
                case unchecked((int)0x80040505)://ECapeBadArgumentHR = 0x80040505 ,
                    return new CapeOpen.CapeBadArgumentException(message, exception, ((CapeOpen.ECapeBadArgument)exception).position);
                case unchecked((int)0x80040506)://ECapeInvalidArgumentHR = 0x80040506 ,
                    return new CapeOpen.CapeInvalidArgumentException(message, exception, ((CapeOpen.ECapeBadArgument)exception).position);
                case unchecked((int)0x80040507)://ECapeOutOfBoundsHR = 0x80040507 
                    {
                        System.Exception p_Ex = (System.Exception)(exception);
                        if (exception is ECapeBoundaries)
                        {
                            CapeOpen.ECapeBoundaries p_Ex1 = (CapeOpen.ECapeBoundaries)(exception);
                            if (exception is ECapeBadArgument)
                            {
                                CapeOpen.ECapeBadArgument p_Ex2 = (CapeOpen.ECapeBadArgument)(exception);
                                return new CapeOpen.CapeOutOfBoundsException(message, p_Ex, (int)p_Ex2.position, p_Ex1.lowerBound, p_Ex1.upperBound, p_Ex1.value, p_Ex1.type);
                            }
                            else return new CapeOpen.CapeOutOfBoundsException(message, p_Ex, 0, p_Ex1.lowerBound, p_Ex1.upperBound, p_Ex1.value, p_Ex1.type);
                        }
                        else return new CapeOpen.CapeOutOfBoundsException(message, p_Ex, 0, 0.0, 0.0, 0.0, "");
                    }
                case unchecked((int)(0x80040508))://ECapeImplementationHR = 0x80040508
                    return new CapeOpen.CapeImplementationException(message, exception);
                case unchecked((int)0x80040509)://ECapeNoImplHR = 0x80040509
                    return new CapeOpen.CapeNoImplException(message, exception);
                case unchecked((int)0x8004050A)://ECapeLimitedImplHR = 0x8004050A
                    return new CapeOpen.CapeLimitedImplException(message, exception);
                case unchecked((int)0x8004050B)://ECapeComputationHR = 0x8004050B 
                    return new CapeOpen.CapeComputationException(message, exception);
                case unchecked((int)0x8004050C)://ECapeOutOfResourcesHR = 0x8004050C
                    return new CapeOpen.CapeOutOfResourcesException(message, exception);
                case unchecked((int)0x8004050D)://ECapeNoMemoryHR = 0x8004050D
                    return new CapeOpen.CapeNoMemoryException(message, exception);
                case unchecked((int)0x8004050E)://ECapeTimeOutHR = 0x8004050E
                    return new CapeOpen.CapeTimeOutException(message, exception);
                case unchecked((int)0x8004050F)://ECapeFailedInitialisationHR = 0x8004050F 
                    return new CapeOpen.CapeFailedInitialisationException(message, exception);
                case unchecked((int)0x80040510)://ECapeSolvingErrorHR = 0x80040510
                    return new CapeOpen.CapeSolvingErrorException(message, exception);
                case unchecked((int)0x80040511)://ECapeBadInvOrderHR = 0x80040511 
                    {

                        if (exception is ECapeBadInvOrder)
                        {
                            CapeOpen.ECapeBadInvOrder p_Ex = (CapeOpen.ECapeBadInvOrder)(exception);
                            return new CapeOpen.CapeBadInvOrderException(message, exception, p_Ex.requestedOperation);
                        }
                        else return new CapeOpen.CapeBadInvOrderException(message, exception, "");
                    }
                case unchecked((int)0x80040512)://ECapeInvalidOperationHR = 0x80040512
                    return new CapeOpen.CapeInvalidOperationException(message, exception);
                case unchecked((int)0x80040513)://ECapePersistenceHR = 0x80040513
                    return new CapeOpen.CapePersistenceException(message, exception);
                case unchecked((int)0x80040514)://ECapeIllegalAccessHR = 0x80040514
                    return new CapeOpen.CapeIllegalAccessException(message, exception);
                case unchecked((int)0x80040515)://ECapePersistenceNotFoundHR = 0x80040515
                    return new CapeOpen.CapePersistenceNotFoundException(message, exception, ((CapeOpen.ECapePersistenceNotFound)exception).itemName);
                case unchecked((int)0x80040516)://ECapePersistenceSystemErrorHR = 0x80040516
                    return new CapeOpen.CapePersistenceSystemErrorException(message, exception);
                case unchecked((int)0x80040517)://ECapePersistenceOverflowHR = 0x80040517
                    return new CapeOpen.CapePersistenceOverflowException(message, exception);
                case unchecked((int)0x80040518)://ECapeOutsideSolverScopeHR = 0x80040518
                    return new CapeOpen.CapeSolvingErrorException(message, exception);
                case unchecked((int)0x80040519)://ECapeHessianInfoNotAvailableHR = 0x80040519
                    return new CapeOpen.CapeHessianInfoNotAvailableException(message, exception);
                case unchecked((int)0x80040520)://ECapeThrmPropertyNotAvailable = 0x80040520
                    return new CapeOpen.CapeThrmPropertyNotAvailableException(message, exception);
                default://ECapeDataHR
                    return new CapeOpen.CapeUnknownException(message, exception);
            }
        }
    };
}