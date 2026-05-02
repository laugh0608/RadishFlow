using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen {
	/// <summary>
	/// The direction that objects or information connected to the port is expected to flow (e.g. material, energy or information objects).
	/// </summary>
	/// <remarks>
	/// This enumeration provide the flowsheeting tool with information related to the direction of the port, that is, whether the port take in
	/// material, information, or energy; or outputs a material, information of energy. This can be used to by the flowsheet to
	/// aid in the selection of the port to which to attach the material, information or energy object.
	/// </remarks>
	[Serializable]
	public enum CapePortDirection{
		/// <summary>
		/// Signifies an inlet port to the unit operation.
		/// </summary>
		CAPE_INLET=0,
		/// <summary>
		/// Signifies an outlet port to the unit operation.
		/// </summary>
		CAPE_OUTLET=1,
		/// <summary>
		/// Signifies a port that can be either an inlet or an outlet to the unit operation.
		/// </summary>
		CAPE_INLET_OUTLET=2
	};

	/// <summary>
	/// The type of objects or information that can flow into the unit operation from 
	/// the connected object.
	/// </summary>
	/// <remarks>
	/// This enumeration provide the flowsheeting tool with information related to the type of the port, that is, whether the unit operation uses the object attaches to the port as a
	/// material, information, or energy. This can be used to by the flowsheet to
	/// aid in the selection of the port to which to attach the material, information or energy object.
	/// </remarks>
	[Serializable]
	public enum CapePortType{
		/// <summary>
		/// Indicates that a material flow is expected through this port to the unit operation.
		/// </summary>
		CAPE_MATERIAL = 0,
		/// <summary>
		/// Indicates that an energy flow is expected through this port to the unit operation.
		/// </summary>
		CAPE_ENERGY = 1,
		/// <summary>
		/// Indicates that an information flow is expected through this port to the unit operation.
		/// </summary>
		CAPE_INFORMATION = 2,
		/// <summary>
		/// Indicates that either material, energy, or information can flow through this port to the unit operation.
		/// </summary>
		CAPE_ANY = 3
	};

/*	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(ICapeUnit_IID)]
	public interface class ICapeUnit093
	{

		/// <summary>
		/// Gets the collection of unit operation ports.
		/// </summary>
		/// <remarks>
		/// <para>Return an interface to a collection containing the list of unit ports (e.g. 
		/// <see name = "ICapeCollection"/>).</para>
		/// <para>Return the collection of unit ports (i.e. ICapeUnitCollection). These are 
		/// delivered as a collection of elements exposing the interfaces <see name = "ICapeUnitPort"/>
		/// </para>
		/// </remarks>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
		/// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
		[System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("Gets the whole list of ports")]
		property Object^ ports
		{
			[returnvalue: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
			Object^ get();
		};

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
		[System.Runtime.InteropServices.DispIdAttribute(2)]
		[System.ComponentModel.DescriptionAttribute("Gets the whole list of parameters")]
		property Object^ parameters
		{
			[returnvalue: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
			Object^ get ();
		};

		/// <summary>
		/// Gets the flag to indicate the unit operation's validation status
		/// <see cref= "CapeValidationStatus">CapeValidationStatus</see>.
		/// </summary>
		/// <remarks>
		/// <para>Get the flag that indicates whether the Flowsheet Unit is valid (e.g. some 
		/// parameter values have changed but they have not been validated by using Validate). 
		/// It has three possible values:</para>
		/// <para>   (i)   notValidated(CAPE_NOT_VALIDATED): The PMC's <c>Validate()</c>
		/// method has not been called after the last time that its value had been 
		/// changed.</para>
		/// <para>   (ii)  invalid(CAPE_INVALID): The last time that the PMC's 
		/// <c>Validate()</c> method was called it returned false.</para>
		/// <para>   (iii) valid(CAPE_VALID): the last time that the PMC's
		/// Validate() method was called it returned true.</para>
		/// </remarks>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
		[System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("Get the unit's validation status")]
		property CapeValidationStatus ValStatus
		{
			CapeValidationStatus get();
		};

		/// <summary>
		///	Executes the necessary calculations involved in the unit operation model.
		/// </summary>
		/// <remarks>
		/// <para>The Flowsheet Unit performs its calculation, that is, computes the variables 
		/// that are missing at this stage in the complete description of the input and output 
		/// streams and computes any public parameter value that needs to be displayed. Calculate 
		/// will be able to do progress monitoring and checks for interrupts as required using 
		/// the simulation context. At present, there are no standards agreed for this.</para>
		/// <para>It is recommended that Flowsheet Units perform a suitable flash calculation on 
		/// all output streams. In some cases a Simulation Executive will be able to perform a 
		/// flash calculation but the writer of a Flowsheet Unit is in the best position to 
		/// decide the correct flash to use. </para>
		/// <para>Before performing the calculation, this method should perform any final 
		/// validation tests that are required. For example, at this point the validity of 
		/// Material Objects connected to ports can be checked.</para>
		/// <para>There are no input or output arguments for this method.</para>
		/// </remarks>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
		/// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
		/// <exception cref = "ECapeTimeOut">ECapeTimeOut</exception>
		/// <exception cref = "ECapeSolvingError">ECapeSolvingError</exception>
		/// <exception cref = "ECapeLicenceError">ECapeLicenceError</exception>
		[System.Runtime.InteropServices.DispIdAttribute(4), System.ComponentModel.DescriptionAttribute("Performs unit calculations")]
		void Calculate();

		/// <summary>
		/// The unit operation is asked to read its persistent state, from the storage location it has previously chosen in Save.
		/// </summary>
		/// <remarks>
		/// <para>The Flowsheet Unit is restored from a previously saved state. The [in] argument 
		/// identifies the location from which the Flowsheet Unit should read its data. Here, data
		/// refers to whatever data the writer of the Flowsheet Unit chooses to save. This 
		/// location may bear no relation to any location to which data has previously been saved. 
		/// It can be a CapeString type, identifying a full path of an ASCII file, or it can be a 
		/// type CapeVariant, hosting a reference to any standard COM interface for persistence 
		/// such as IStorage or IStream.</para>
		/// <para>It is strongly recommended that the IStorage, or IStream variations are not used 
		/// in a COM environment because a Flowsheet Unit written in Visual Basic (versions 6 and 
		/// lower) is not able to make use of an IStorage or IStream pointer. In a COM environment
		/// the preferred solution is for the Flowsheet Unit and the COSE to support the standard 
		/// COM persistence interfaces and protocols. See section 5.2.8 for a discussion of this 
		/// recommendation.</para>
		/// </remarks>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
		/// <exception cref = "ECapePersistenceNotFound">ECapePersistenceNotFound</exception>
		/// <exception cref = "ECapeIllegalAccess">ECapeIllegalAccess</exception>
		/// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
		[System.Runtime.InteropServices.DispIdAttribute(5), System.ComponentModel.DescriptionAttribute("Recovers unit persistent state")]
		void Restore(Object^ storage);

		/// <summary>
		/// The unit operation is asked to save its persistent state, in a given 
		/// location passed in as argument. This unit may choose to use that storage 
		/// (e.g. structured storage) or to use its own internal storage mechanism 
		/// (e.g. a plain ASCII text file). If the storage location is changed by 
		/// the unit, the new location is sent back to the client.
		/// </summary>
		/// <remarks>
		/// <para>The Flowsheet Unit saves its private data in the location indicated by the 
		/// simulator (i.e. storage), that can be a given position in a structured document or a 
		/// file name (CapeString type). The Flowsheet Unit is free to use the storage mechanism 
		/// that the simulator provides, or to use its own internal mechanisms. A valid scenario 
		/// would be that the Flowsheet Unit checks the type of the storage parameter passed in, 
		/// accepting it if it is of type CapeString (e.g. an ASCII file name) or rejecting it if 
		/// it is of another type that the Flowsheet Unit does not handle. In the latter case, the 
		/// Flowsheet Unit may decide to create its own text file, save its data there, and send 
		/// back the full path of the file to the simulator (notice the [in, out] argument) which 
		/// stores that string along with its own simulation data.</para>
		/// <para>It is recommended that in a COM implementation a component should support one 
		/// of the standard COM persistence methods in addition to this method. See section 
		/// 5.2.8 for a discussion of this recommendation.</para>
		/// <para>A Flowsheet Unit must save its state completely so that the Restore can 
		/// recreate that state. Flowsheet Unit authors should not rely on Validate being called 
		/// after Restore. The same requirement applies when COM persistence methods are 
		/// implemented.</para>
		/// </remarks>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
		/// <exception cref = "ECapePersistenceNotFound">ECapePersistenceNotFound</exception>
		/// <exception cref = "ECapeIllegalAccess">ECapeIllegalAccess</exception>
		/// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
		[System.Runtime.InteropServices.DispIdAttribute(6), System.ComponentModel.DescriptionAttribute("Saves unit state")]
		void Save(Object^ storage);

		/// <summary>
		/// The unit operation is asked to configure itself. Typically some ports 
		/// and parameters are created here.
		/// </summary>
		/// <remarks>
		/// <para>The Flowsheet Unit initialises itself. Any initialisation that could fail must 
		/// be placed here.</para>
		/// <para>Initialize is guaranteed to be the first method called by the client. Initialize 
		/// has to be called once when the Flowsheet Unit is instantiated in a particular 
		/// flowsheet.</para>
		/// <para>Note that the Initialize method is intended to implement software initialization 
		/// not to perform an initial solution of the Flowsheet Unit. It is expected that the 
		/// method would be used to create and configure ports and parameters and to define 
		/// default values for variables and parameters.</para>
		/// <para>There are no input or output arguments for this method.</para>
		/// </remarks>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeLicenceError">ECapeLicenceError</exception>
		/// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
		/// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
		[System.Runtime.InteropServices.DispIdAttribute(7), System.ComponentModel.DescriptionAttribute("Configuration has to take place here")]
		void Initialize();

		/// <summary>
		///	Clean-up tasks for the unit operationcan be performed here. In 
		/// particular, references to parameters and objects connected to 
		/// ports are released here.
		/// </summary>
		/// <remarks>
		/// <para>The Flowsheet Unit releases all of its allocated resources. This is called 
		/// before the object destructor. Terminate may check if the data has been saved and return an 
		/// error if not.</para>
		/// <para>There are no input or output arguments for this method.</para>
		/// </remarks>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
		[System.Runtime.InteropServices.DispIdAttribute(8), System.ComponentModel.DescriptionAttribute("Clean up has to take place here")]
		void Terminate();

		/// <summary>
		///	Validate the unit operation to verify that the parameters and ports are 
		/// all valid. If invalid, this method returns a message indicating the 
		/// reason that the unit is invalid.
		/// </summary>
		/// <remarks>
		/// <para>Sets the flag that indicates whether the Flowsheet Unit is valid by validating 
		/// the ports and parameters of the Flowsheet Unit. For example, this method could check 
		/// that all mandatory ports have connections and that the values of all parameters are 
		/// within bounds.</para>
		/// <para>Note that the Simulation Executive can call the Validate routine at any time, 
		/// in particular it may be called before the executive is ready to call the Calculate 
		/// method. This means that Material Objects connected to unit ports may not be correctly 
		/// configured when Validate is called. The recommended approach is for this method to 
		/// validate parameters and ports but not Material Object configuration. A second level 
		/// of validation to check Material Objects can be implemented as part of Calculate, when 
		/// it is reasonable to expect that the Material Objects connected to ports will be 
		/// correctly configured.</para>
		/// </remarks>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeBadCOParameter">ECapeBadCOParameter</exception>
		/// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
		[System.Runtime.InteropServices.DispIdAttribute(9), System.ComponentModel.DescriptionAttribute("Validate the Unit"), returnvalue: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.VariantBool)]
		bool Validate(System.String^ %message);

		/// <summary>
		///	Set the simulation context for the unit operation.
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
		/// PMC, ICapeUnit. SetSimulationContext is deprecated.</para>
		/// </remarks>
		/// <param name = "simContext">
		/// The reference to the PME’s simulation context class. For the PMC to use 
		/// this class, this reference will have to be converted to each of the 
		/// defined CO Simulation Context interfaces.
		/// </param>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
		/// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
		/// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
		[System.Runtime.InteropServices.DispIdAttribute(11)]
		[System.ComponentModel.DescriptionAttribute("Set the simulation context")]
		property Object^ simulationContext
		{
			void set([System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]Object^ simContext);
		};
	};

*/

    /// <summary>
    /// Represents the method that will handle the validation of a unit operation.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void UnitOperationValidatedHandler(Object sender, UnitOperationValidatedEventArgs args);

    /// <summary>
    /// The unit operation was validated.
    /// </summary>
    /// <remarks>
    /// Provides information about the validation of the unit operation.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("50A759AF-5E38-4399-9050-93F823E5A6E6")]
    [System.ComponentModel.DescriptionAttribute("IUnitOperationValidatedEventArgs Interface")]
    interface IUnitOperationValidatedEventArgs
    {
        /// <summary>
        /// The name of the unit operation being changed.</summary>
        String UnitOperationName
        {
            get;
        }
        /// <summary>
        /// The message reulting from the unit operation validation.</summary>
        /// <remarks>The message provides information about the results of the validation process.</remarks>
        /// <value>Information regrading the validation process.</value>
        String Message
        {
            get;
        }
        /// <summary>
        /// The validation status of the unit operation prior to the validation.</summary>
        /// <remarks>Informs the user of the results of the validation process.</remarks>
        /// <value>The validation status of the unit operation prior to the validation.</value>
        CapeValidationStatus OldStatus
        {
            get;
        }
        /// <summary>
        /// The validation status of the unit operation after the validation.</summary>
        /// <remarks>Informs the user of the results of the validation process.</remarks>
        /// <value>The validation status of the unit operation after the validation.</value>
        CapeValidationStatus NewStatus
        {
            get;
        }
    };

    /// <summary>
    /// The unit operation was validated.
    /// </summary>
    /// <remarks>
    /// Provides information about the validation of the unit operation.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("9147E78B-29D6-4D91-956E-75D0FB90CEA7")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class UnitOperationValidatedEventArgs : System.EventArgs,
        IUnitOperationValidatedEventArgs
    {
        private String m_unitName;
        private String m_message;
        private CapeValidationStatus m_oldStatus;
        private CapeValidationStatus m_newStatus;

        /// <summary>Creates an instance of the UnitValidatedEventArgs class for the unit operation .</summary>
        /// <remarks>You can use this constructor when raising the UnitValidatedEventArgs at run time to  
        /// the message about the parameter validation.
        /// </remarks>
        /// <param name = "unitName">The name of the unit operation  being changed.</param>
        /// <param name = "message">The message indicating the results of the unit operation validation.</param>
        /// <param name = "oldStatus">The status of the unit operation prior to validation.</param>
        /// <param name = "newStatus">The status of the unit operation after the validation.</param>
        public UnitOperationValidatedEventArgs(String unitName, String message, CapeValidationStatus oldStatus, CapeValidationStatus newStatus)
            : base()
        {
            m_unitName = unitName;
            m_message = message;
            m_oldStatus = oldStatus;
            m_newStatus = newStatus;
        }

        /// <summary>
        /// The name of the unit operation being validated.</summary>
        /// <remarks>The name of teh unit operation being validated.</remarks>
        /// <value>The name of the unit operation being changed.</value>
        public String UnitOperationName
        {
            get
            {
                return m_unitName;
            }
        }
        /// <summary>
        /// The message reulting from the unit operation validation.</summary>
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
        /// The validation status of the unit operation prior to the validation.</summary>
        /// <remarks>Informs the user of the results of the validation process.</remarks>
        /// <value>The validation status of the unit operation prior to the validation.</value>
        public CapeValidationStatus OldStatus
        {
            get
            {
                return m_oldStatus;
            }
        }
        /// <summary>
        /// The validation status of the unit operation after the validation.</summary>
        /// <remarks>Informs the user of the results of the validation process.</remarks>
        /// <value>The validation status of the unit operation after the validation.</value>
        public CapeValidationStatus NewStatus
        {
            get
            {
                return m_newStatus;
            }
        }
    };

    /// <summary>
    /// Represents the method that will handle the beginning of a calculation of a unit operation.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void UnitOperationCalculateHandler(Object sender, UnitOperationCalculateEventArgs args);


    /// <summary>
    /// Event fired at the start of a unit operation was calculation.
    /// </summary>
    /// <remarks>
    /// Provides information about the start of the calculation of the unit operation.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("DDCA3348-074C-4860-AD00-58386327D9AC")]
    [System.ComponentModel.DescriptionAttribute("IUnitOperationCalculateEventArgs Interface")]
    interface IUnitOperationCalculateEventArgs
    {
        /// <summary>
        /// The name of the unit operation being calculated.</summary>
        String UnitOperationName
        {
            get;
        }
        /// <summary>
        /// The message reulting from the start of the unit operation calculation.</summary>
        /// <remarks>The message provides information about the start of the unit operation calculation process.</remarks>
        /// <value>Information regrading the validation process.</value>
        String Message
        {
            get;
        }
    };

    /// <summary>
    /// Event fired at the start of a unit operation was calculation.
    /// </summary>
    /// <remarks>
    /// Provides information about the start of the calculation of the unit operation.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("7831C38B-A1C6-40C5-B9FC-DAC43426AAD4")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class UnitOperationCalculateEventArgs : System.EventArgs,
        IUnitOperationCalculateEventArgs
    {
        private String m_unitName;
        private String m_message;

        /// <summary>Creates an instance of the UnitOperationBeginCalculationEventArgs class for the unit operation .</summary>
        /// <remarks>You can use this constructor when raising the UnitOperationBeginCalculationEventArgs at run time to  
        /// the message about the parameter validation.
        /// </remarks>
        /// <param name = "unitName">The name of the unit operation  being calulcated.</param>
        /// <param name = "message">The message indicating the conditions at the start of the unit operation calculation.</param>
        public UnitOperationCalculateEventArgs(String unitName, String message)
            : base()
        {
            m_unitName = unitName;
            m_message = message;
        }

        /// <summary>
        /// The name of the unit operation being calculated.</summary>
        /// <remarks>The name of the unit operation being calculated.</remarks>
        /// <value>The name of the unit operation being calculated.</value>
        public String UnitOperationName
        {
            get
            {
                return m_unitName;
            }
        }
        /// <summary>
        /// The message from the unit operation regarding the start of the calculation process.</summary>
        /// <remarks>The message provides information about the start of the calculated process.</remarks>
        /// <value>Information regarding the start of the calculated process.</value>
        public String Message
        {
            get
            {
                return m_message;
            }
        }
    };
    /// <summary>
    /// Represents the method that will handle the beginning of a calculation of a unit operation.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void UnitOperationBeginCalculationHandler(Object sender, UnitOperationBeginCalculationEventArgs args);

    /// <summary>
    /// Event fired at the start of a unit operation was calculation.
    /// </summary>
    /// <remarks>
    /// Provides information about the start of the calculation of the unit operation.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("3E827FD8-5BDB-41E4-81D9-AC438BC9B957")]
    [System.ComponentModel.DescriptionAttribute("IUnitOperationBeginCalculationEventArgs Interface")]
    interface IUnitOperationBeginCalculationEventArgs
    {
        /// <summary>
        /// The name of the unit operation being calculated.</summary>
        String UnitOperationName
        {
            get;
        }
        /// <summary>
        /// The message reulting from the start of the unit operation calculation.</summary>
        /// <remarks>The message provides information about the start of the unit operation calculation process.</remarks>
        /// <value>Information regrading the validation process.</value>
        String Message
        {
            get;
        }
    };

    /// <summary>
    /// Event fired at the start of a unit operation was calculation.
    /// </summary>
    /// <remarks>
    /// Provides information about the start of the calculation of the unit operation.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("763691E8-D792-4B97-A12A-D4AD7F66B5E4")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class UnitOperationBeginCalculationEventArgs : System.EventArgs,
        IUnitOperationBeginCalculationEventArgs
    {
        private String m_unitName;
        private String m_message;

        /// <summary>Creates an instance of the UnitOperationBeginCalculationEventArgs class for the unit operation .</summary>
        /// <remarks>You can use this constructor when raising the UnitOperationBeginCalculationEventArgs at run time to  
        /// the message about the parameter validation.
        /// </remarks>
        /// <param name = "unitName">The name of the unit operation  being calulcated.</param>
        /// <param name = "message">The message indicating the conditions at the start of the unit operation calculation.</param>
        public UnitOperationBeginCalculationEventArgs(String unitName, String message)
            : base()
        {
            m_unitName = unitName;
            m_message = message;
        }

        /// <summary>
        /// The name of the unit operation being calculated.</summary>
        /// <value>The name of the unit operation being calculated.</value>
        public String UnitOperationName
        {
            get
            {
                return m_unitName;
            }
        }
        /// <summary>
        /// The message from the unit operation regarding the start of the calculation process.</summary>
        /// <remarks>The message provides information about the start of the calculated process.</remarks>
        /// <value>Information regarding the start of the calculated process.</value>
        public String Message
        {
            get
            {
                return m_message;
            }
        }
    };

    /// <summary>
    /// Represents the method that will handle the completion of a unit operation calculation process.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void UnitOperationEndCalculationHandler(Object sender, UnitOperationEndCalculationEventArgs args);

    /// <summary>
    /// The unit operation calculation prcess has been completed.
    /// </summary>
    /// <remarks>
    /// Provides information about the completion of the unit operation calculation process.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("951D755F-8831-4691-9B54-CC9935A5B7CC")]
    [System.ComponentModel.DescriptionAttribute("IUnitOperationEndCalculationEventArgs Interface")]
    interface IUnitOperationEndCalculationEventArgs
    {
        /// <summary>
        /// The name of the unit operation being calculated.</summary>
        /// <value>The name of the unit operation being calculated.</value>
        String UnitOperationName
        {
            get;
        }
        /// <summary>
        /// The message from the unit operation regarding the completion of the calculation process.</summary>
        /// <remarks>The message provides information about the completion of the calculated process.</remarks>
        /// <value>Information regarding the completion of the calculated process.</value>
        String Message
        {
            get;
        }
    };

    /// <summary>
    /// The unit operation was validated.
    /// </summary>
    /// <remarks>
    /// Provides information about the validation of the unit operation.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("172F4D6E-65D1-4D9E-A275-7880FA3A40A5")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class UnitOperationEndCalculationEventArgs : System.EventArgs,
        IUnitOperationEndCalculationEventArgs
    {
        private String m_unitName;
        private String m_message;

        /// <summary>Creates an instance of the UnitOperationEndCalculationEventArgs class for the unit operation .</summary>
        /// <remarks>You can use this constructor when raising the UnitOperationEndCalculationEventArgs at run time to  
        /// the message about the parameter validation.
        /// </remarks>
        /// <param name = "unitName">The name of the unit operation  being calulcated.</param>
        /// <param name = "message">The message indicating the results of the unit operation calculation.</param>
        public UnitOperationEndCalculationEventArgs(String unitName, String message)
            : base()
        {
            m_unitName = unitName;
            m_message = message;
        }

        /// <summary>
        /// The name of the unit operation being calculated.</summary>
        /// <value>The name of the unit operation being calculated.</value>
        public String UnitOperationName
        {
            get
            {
                return m_unitName;
            }
        }
        /// <summary>
        /// The message from the unit operation regarding the completion of the calculation process.</summary>
        /// <remarks>The message provides information about the completion of the calculated process.</remarks>
        /// <value>Information regarding the completion of the calculated process.</value>
        public String Message
        {
            get
            {
                return m_message;
            }
        }
    };

    // 	This interface provides the basic functionality for a Unit 
	//	Operation component
	// CAPE-OPEN v1.0
	/// <summary>
	/// This interface handles most of the interaction with the Flowsheet Unit.
	/// </summary>
	/// <remarks>
	/// This interface provides the basic funcational requirements for a unit operation 
	/// component that can be inserted into a flowsheeting package.
	/// </remarks>
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.ComponentModel.DescriptionAttribute("ICapeUnit Interface")]
	public interface ICapeUnit
	{
		/// <summary>
		/// Gets the collection of unit operation ports.
		/// </summary>
		/// <remarks>
		/// <para>Return an interface to a collection containing the list of unit ports (e.g. 
		/// <see name = "ICapeCollection"/>).</para>
		/// <para>Return the collection of unit ports (i.e. ICapeUnitCollection). These are 
		/// delivered as a collection of elements exposing the interfaces <see name = "ICapeUnitPort"/>
		/// </para>
		/// </remarks>
		/// <value>The port collection of the unit operation.</value>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
		/// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
		[System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("Gets the whole list of ports")]
		PortCollection Ports
		{
			[return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
			get;
		}

		/// <summary>
		/// Gets the flag to indicate the unit operation's validation status
		/// <see cref= "CapeValidationStatus">CapeValidationStatus</see>.
		/// </summary>
		/// <remarks>
		/// <para>Get the flag that indicates whether the Flowsheet Unit is valid (e.g. some 
		/// parameter values have changed but they have not been validated by using Validate). 
		/// It has three possible values:</para>
		/// <para>   (i)   notValidated(CAPE_NOT_VALIDATED): The PMC's <c>Validate()</c>
		/// method has not been called after the last time that its value had been 
		/// changed.</para>
		/// <para>   (ii)  invalid(CAPE_INVALID): The last time that the PMC's 
		/// <c>Validate()</c> method was called it returned false.</para>
		/// <para>   (iii) valid(CAPE_VALID): the last time that the PMC's
		/// Validate() method was called it returned true.</para>
		/// </remarks>
		/// <value>A flag that indiciates the validation status of the unit operation.</value>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
		[System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("Get the unit's validation status")]
		CapeValidationStatus ValStatus
		{
			get;
		}

		/// <summary>
		///	Executes the necessary calculations involved in the unit operation model.
		/// </summary>
		/// <remarks>
		/// <para>The Flowsheet Unit performs its calculation, that is, computes the variables 
		/// that are missing at this stage in the complete description of the input and output 
		/// streams and computes any public parameter value that needs to be displayed. Calculate 
		/// will be able to do progress monitoring and checks for interrupts as required using 
		/// the simulation context. At present, there are no standards agreed for this.</para>
		/// <para>It is recommended that Flowsheet Units perform a suitable flash calculation on 
		/// all output streams. In some cases a Simulation Executive will be able to perform a 
		/// flash calculation but the writer of a Flowsheet Unit is in the best position to 
		/// decide the correct flash to use. </para>
		/// <para>Before performing the calculation, this method should perform any final 
		/// validation tests that are required. For example, at this point the validity of 
		/// Material Objects connected to ports can be checked.</para>
		/// <para>There are no input or output arguments for this method.</para>
		/// </remarks>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
		/// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
		/// <exception cref = "ECapeTimeOut">ECapeTimeOut</exception>
		/// <exception cref = "ECapeSolvingError">ECapeSolvingError</exception>
		/// <exception cref = "ECapeLicenceError">ECapeLicenceError</exception>
		[System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("Performs unit calculations")]
		void Calculate();

		/// <summary>
		///	Validate the unit operation to verify that the parameters and ports are 
		/// all valid. If invalid, this method returns a message indicating the 
		/// reason that the unit is invalid.
		/// </summary>
		/// <remarks>
		/// <para>Sets the flag that indicates whether the Flowsheet Unit is valid by validating 
		/// the ports and parameters of the Flowsheet Unit. For example, this method could check 
		/// that all mandatory ports have connections and that the values of all parameters are 
		/// within bounds.</para>
		/// <para>Note that the Simulation Executive can call the Validate routine at any time, 
		/// in particular it may be called before the executive is ready to call the Calculate 
		/// method. This means that Material Objects connected to unit ports may not be correctly 
		/// configured when Validate is called. The recommended approach is for this method to 
		/// validate parameters and ports but not Material Object configuration. A second level 
		/// of validation to check Material Objects can be implemented as part of Calculate, when 
		/// it is reasonable to expect that the Material Objects connected to ports will be 
		/// correctly configured.</para>
		/// </remarks>
		/// <returns>
		/// <para>true, if the unit is valid.</para>
		/// <para>false, if the unit is not valid.</para>
		/// </returns>
		/// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeBadCOParameter">ECapeBadCOParameter</exception>
		/// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
		[System.Runtime.InteropServices.DispIdAttribute(4)]
		[System.ComponentModel.DescriptionAttribute("Validate the Unit")]
		[return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.VariantBool)]
		bool Validate(ref String message);
	};


    // 	This interface provides the basic functionality for a Unit 
    //	Operation component
    // CAPE-OPEN v1.0
    /// <summary>
    /// This interface handles most of the interaction with the Flowsheet Unit.
    /// </summary>
    /// <remarks>
    /// This interface provides the basic funcational requirements for a unit operation 
    /// component that can be inserted into a flowsheeting package.
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeUnit_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeUnit Interface")]
    interface ICapeUnitCOM
    {
        /// <summary>
        /// Gets the collection of unit operation ports.
        /// </summary>
        /// <remarks>
        /// <para>Return an interface to a collection containing the list of unit ports (e.g. 
        /// <see name = "ICapeCollection"/>).</para>
        /// <para>Return the collection of unit ports (i.e. ICapeUnitCollection). These are 
        /// delivered as a collection of elements exposing the interfaces <see name = "ICapeUnitPort"/>
        /// </para>
        /// </remarks>
        /// <value>The port collection of the unit operation.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("Gets the whole list of ports")]
        Object ports
        {
            [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
            get;
        }

        /// <summary>
        /// Gets the flag to indicate the unit operation's validation status
        /// <see cref= "CapeValidationStatus">CapeValidationStatus</see>.
        /// </summary>
        /// <remarks>
        /// <para>Get the flag that indicates whether the Flowsheet Unit is valid (e.g. some 
        /// parameter values have changed but they have not been validated by using Validate). 
        /// It has three possible values:</para>
        /// <para>   (i)   notValidated(CAPE_NOT_VALIDATED): The PMC's <c>Validate()</c>
        /// method has not been called after the last time that its value had been 
        /// changed.</para>
        /// <para>   (ii)  invalid(CAPE_INVALID): The last time that the PMC's 
        /// <c>Validate()</c> method was called it returned false.</para>
        /// <para>   (iii) valid(CAPE_VALID): the last time that the PMC's
        /// Validate() method was called it returned true.</para>
        /// </remarks>
        /// <value>A flag that indiciates the validation status of the unit operation.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("Get the unit's validation status")]
        CapeValidationStatus ValStatus
        {
            get;
        }

        /// <summary>
        ///	Executes the necessary calculations involved in the unit operation model.
        /// </summary>
        /// <remarks>
        /// <para>The Flowsheet Unit performs its calculation, that is, computes the variables 
        /// that are missing at this stage in the complete description of the input and output 
        /// streams and computes any public parameter value that needs to be displayed. Calculate 
        /// will be able to do progress monitoring and checks for interrupts as required using 
        /// the simulation context. At present, there are no standards agreed for this.</para>
        /// <para>It is recommended that Flowsheet Units perform a suitable flash calculation on 
        /// all output streams. In some cases a Simulation Executive will be able to perform a 
        /// flash calculation but the writer of a Flowsheet Unit is in the best position to 
        /// decide the correct flash to use. </para>
        /// <para>Before performing the calculation, this method should perform any final 
        /// validation tests that are required. For example, at this point the validity of 
        /// Material Objects connected to ports can be checked.</para>
        /// <para>There are no input or output arguments for this method.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        /// <exception cref = "ECapeOutOfResources">ECapeOutOfResources</exception>
        /// <exception cref = "ECapeTimeOut">ECapeTimeOut</exception>
        /// <exception cref = "ECapeSolvingError">ECapeSolvingError</exception>
        /// <exception cref = "ECapeLicenceError">ECapeLicenceError</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("Performs unit calculations")]
        void Calculate();

        /// <summary>
        ///	Validate the unit operation to verify that the parameters and ports are 
        /// all valid. If invalid, this method returns a message indicating the 
        /// reason that the unit is invalid.
        /// </summary>
        /// <remarks>
        /// <para>Sets the flag that indicates whether the Flowsheet Unit is valid by validating 
        /// the ports and parameters of the Flowsheet Unit. For example, this method could check 
        /// that all mandatory ports have connections and that the values of all parameters are 
        /// within bounds.</para>
        /// <para>Note that the Simulation Executive can call the Validate routine at any time, 
        /// in particular it may be called before the executive is ready to call the Calculate 
        /// method. This means that Material Objects connected to unit ports may not be correctly 
        /// configured when Validate is called. The recommended approach is for this method to 
        /// validate parameters and ports but not Material Object configuration. A second level 
        /// of validation to check Material Objects can be implemented as part of Calculate, when 
        /// it is reasonable to expect that the Material Objects connected to ports will be 
        /// correctly configured.</para>
        /// </remarks>
        /// <returns>
        /// <para>true, if the unit is valid.</para>
        /// <para>false, if the unit is not valid.</para>
        /// </returns>
        /// <param name = "message">Reference to a string that will conain a message regarding the validation of the parameter.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeBadCOParameter">ECapeBadCOParameter</exception>
        /// <exception cref = "ECapeBadInvOrder">ECapeBadInvOrder</exception>
        [System.Runtime.InteropServices.DispIdAttribute(4)]
        [System.ComponentModel.DescriptionAttribute("Validate the Unit")]
        [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.VariantBool)]
        bool Validate(ref String message);
    };

    /// <summary>
    /// This interface provides access to the active unit report and the available list of options. 
    /// </summary>
    /// <remarks>
    ///	It also provides a trigger for the creation of a report.
    /// </remarks>
    ///	
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.ComponentModel.DescriptionAttribute("ICapeUnitReport Interface")]
    public interface ICapeUnitReport
    {
        /// <summary>
        ///	Gets the list of possible reports for the unit operation.
        /// </summary>
        /// <remarks>
        /// Return the list of available Flowsheet Unit reports.
        /// </remarks>
        /// <value>
        ///	The list of possible reports for the unit operation.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("Gets the list of unit reports")]
        System.Collections.Generic.List<String> Reports
        {
            get;
        }

        /// <summary>
        ///	Gets and sets the current active report for the unit operation.
        /// </summary>
        /// <remarks>
        /// Return/set the active report in the Flowsheet Unit.
        /// </remarks>
        /// <value>
        ///	The current active report for the unit operation.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("Gets the active unit report")]
        String selectedReport
        {
            get;
            set;
        }

        /// <summary>
        ///	Produces the active report for the unit operation.
        /// </summary>
        /// <remarks>
        /// Produce the designated report. If no value has been set, it produces the default 
        /// report.
        /// </remarks>
        /// <returns>String containing the text for the currently selected report.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("Creates the active report")]
        String ProduceReport();
    };    
    
    /// <summary>
    /// This interface provides access to the active unit report and the available list of options. 
    /// </summary>
    /// <remarks>
    ///	It also provides a trigger for the creation of a report.
    /// </remarks>
    ///	
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeUnitReport_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeUnitReport Interface")]
    interface ICapeUnitReportCOM
    {
        /// <summary>
        ///	Gets the list of possible reports for the unit operation.
        /// </summary>
        /// <remarks>
        /// Return the list of available Flowsheet Unit reports.
        /// </remarks>
        /// <value>
        ///	The list of possible reports for the unit operation.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("Gets the list of unit reports")]
        Object reports
        {
            get;
        }

        /// <summary>
        ///	Gets and sets the current active report for the unit operation.
        /// </summary>
        /// <remarks>
        /// Return/set the active report in the Flowsheet Unit.
        /// </remarks>
        /// <value>
        ///	The current active report for the unit operation.
        /// </value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("Gets the active unit report")]
        String selectedReport
        {
            get;
            set;
        }

        /// <summary>
        ///	Produces the active report for the unit operation.
        /// </summary>
        /// <remarks>
        /// Produce the designated report. If no value has been set, it produces the default 
        /// report.
        /// </remarks>
        /// <param name = "message">String containing the text for the currently selected report.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeNoImpl">ECapeNoImpl</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("Creates the active report")]
        void ProduceReport(ref String message);
    };

    /// <summary>
    /// This interface represents the behaviour of a Unit 
    ///	Operation connection point (Unit Operation Port). It provides different 
    ///	attributes for configuring the port as well as to connect 
    ///	it to a material, energy or information object.
    /// </summary>
    /// <remarks>
    /// <para>
    /// The unit port provides the the means by which a Flowsheet Unit is connected to its streams. 
    /// Streams are implemented by means of material objects.
    /// </para>
    /// <para>
    /// The three types of port: material, energy and 
    ///	information, have a lot of functionality in common. By combining the three into one we can simplify 
    ///	the interface to a useful degree. Each port type is to be distinguished by the value of an attribute.
    /// </para>
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeUnitPort_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeUnitPort Interface")]
    interface ICapeUnitPortCOM
    {
        /// <summary>
        ///	Returns port type.
        /// </summary>
        /// <remarks>
        ///	Returns the type of this port. Allowed types are among 
        ///	the ones included in the CapePortType type.
        /// </remarks>
        /// <value>The type of the port.</value>
        /// <see cref="CapeOpen.CapePortType">CapePortType</see>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("type of port, e.g. material, energy or information")]
        CapePortType portType
        {
            get;
        }

        /// <summary>
        ///	Returns port direction.
        /// </summary>
        /// <remarks>
        ///	Returns the direction in which the object connected to this 
        ///	port is expected to flow. Allowed values are among those included 
        /// in the CapePortDirection type.
        /// </remarks>
        /// <value>The direction of the port.</value>
        /// <see cref="CapeOpen.CapePortDirection">CapePortDirection</see>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("direction of port, e.g. input, output or unspecified")]
        CapePortDirection direction
        {
            get;
        }

        /// <summary>
        ///	Returns to the client the object that is connected to this port.
        /// </summary>
        /// <remarks>
        /// Returns the object that is connected to the Port. A client is provided with the 
        /// Material, Energy or Information object that was previously connected to the Port, 
        /// using the Connect method.
        /// </remarks>
        /// <value>The object connected to the port.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("gets the objet connected to the port, e.g. material, energy or information")]
        Object connectedObject
        {
            [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
            get;
        }

        /// <summary>
        ///	Connects an object to the port. For a material port it must 
        /// be an object implementing the ICapeThermoMaterialObject interface, 
        /// for Energy and Information ports it must be an object implementing 
        /// the ICapeParameter interface. 
        /// </summary>
        /// <remarks>
        /// Method used by clients, when they request that a Port connect itself with the object 
        /// that is passed in as argument of the method. Probably, before accepting the connection, 
        /// a Port will check that the Object sent as argument is of the expected type and 
        /// according to the value of its attribute portType.
        /// </remarks>
        /// <param name = "objectToConnect">The object to connect to the port.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(4), System.ComponentModel.DescriptionAttribute("connects the port to the object sent as argument, e.g. material, energy or information")]
        void Connect([System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]Object objectToConnect);

        /// <summary>
        ///	Disconnects whatever object is connected to this port.
        /// </summary>
        /// <remarks>
        /// <para>Disconnects the port from whichever object is connected to it.</para>
        /// <para>There are no input or output arguments for this method.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(5), System.ComponentModel.DescriptionAttribute("disconnects the port")]
        void Disconnect();
    };

    /// <summary>
    /// This interface represents the behaviour of a Unit 
    ///	Operation connection point (Unit Operation Port). It provides different 
    ///	attributes for configuring the port as well as to connect 
    ///	it to a material, energy or information object.
    /// </summary>
    /// <remarks>
    /// <para>
    /// The unit port provides the the means by which a Flowsheet Unit is connected to its streams. 
    /// Streams are implemented by means of material objects.
    /// </para>
    /// <para>
    /// The three types of port: material, energy and 
    ///	information, have a lot of functionality in common. By combining the three into one we can simplify 
    ///	the interface to a useful degree. Each port type is to be distinguished by the value of an attribute.
    /// </para>
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.ComponentModel.DescriptionAttribute("ICapeUnitPort Interface")]
    public interface ICapeUnitPort
    {
        /// <summary>
        ///	Returns port type.
        /// </summary>
        /// <remarks>
        ///	Returns the type of this port. Allowed types are among 
        ///	the ones included in the CapePortType type.
        /// </remarks>
        /// <value>The type of the port.</value>
        /// <see cref="CapeOpen.CapePortType">CapePortType</see>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("type of port, e.g. material, energy or information")]
        CapePortType portType
        {
            get;
        }

        /// <summary>
        ///	Returns port direction.
        /// </summary>
        /// <remarks>
        ///	Returns the direction in which the object connected to this 
        ///	port is expected to flow. Allowed values are among those included 
        /// in the CapePortDirection type.
        /// </remarks>
        /// <value>The direction of the port.</value>
        /// <see cref="CapeOpen.CapePortDirection">CapePortDirection</see>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("direction of port, e.g. input, output or unspecified")]
        CapePortDirection direction
        {
            get;
        }

        /// <summary>
        ///	Returns to the client the object that is connected to this port.
        /// </summary>
        /// <remarks>
        /// Returns the object that is connected to the Port. A client is provided with the 
        /// Material, Energy or Information object that was previously connected to the Port, 
        /// using the Connect method.
        /// </remarks>
        /// <value>The object connected to the port.</value>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeFailedInitialisation">ECapeFailedInitialisation</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("gets the objet connected to the port, e.g. material, energy or information")]
        Object connectedObject
        {
            [return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
            get;
        }

        /// <summary>
        ///	Connects an object to the port. For a material port it must 
        /// be an object implementing the ICapeThermoMaterialObject interface, 
        /// for Energy and Information ports it must be an object implementing 
        /// the ICapeParameter interface. 
        /// </summary>
        /// <remarks>
        /// Method used by clients, when they request that a Port connect itself with the object 
        /// that is passed in as argument of the method. Probably, before accepting the connection, 
        /// a Port will check that the Object sent as argument is of the expected type and 
        /// according to the value of its attribute portType.
        /// </remarks>
        /// <param name = "objectToConnect">The object to connect to the port.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(4), System.ComponentModel.DescriptionAttribute("connects the port to the object sent as argument, e.g. material, energy or information")]
        void Connect([System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]Object objectToConnect);

        /// <summary>
        ///	Disconnects whatever object is connected to this port.
        /// </summary>
        /// <remarks>
        /// <para>Disconnects the port from whichever object is connected to it.</para>
        /// <para>There are no input or output arguments for this method.</para>
        /// </remarks>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(5), System.ComponentModel.DescriptionAttribute("disconnects the port")]
        void Disconnect();
    };

    /// <summary>
    /// An object was connected to the port.
    /// </summary>
    /// <remarks>
    /// An object was connected to the port.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("DC735166-8008-4B39-BE1C-6E94A723AD65")]
    [System.ComponentModel.DescriptionAttribute("PortConnectedEventArgs Interface")]
    interface IPortConnectedEventArgs
    {
        /// <summary>
        /// The name of the port being connected.
        /// </summary>
        String PortName
        {
            get;
        }
    };

    /// <summary>
    /// An object was connected to the port.
    /// </summary>
    /// <remarks>
    /// An object was connected to the port.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("962B9FDE-842E-43F8-9280-41C5BF80DDEC")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class PortConnectedEventArgs : System.EventArgs,
        IPortConnectedEventArgs
    {
        private String m_portName;
        /// <summary>Creates an instance of the PortConnectedEventArgs class for the port.</summary>
        /// <remarks>You can use this constructor when raising the PortConnectedEventArgs at run time to  
        /// inform the system that the poert was connected.
        /// </remarks>
        public PortConnectedEventArgs(String portName)
            : base()
        {
            m_portName = portName;
        }
        /// <summary>
        /// The name of the port being connected.</summary>
        /// <value>The name of the port being connected.</value>
        public String PortName
        {
            get
            {
                return m_portName;
            }
        }
    };


    /// <summary>
    /// Represents the method that will handle disconnecting an object from a unit port.
    /// </summary>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    public delegate void PortDisconnectedHandler(Object sender, PortDisconnectedEventArgs args);


    /// <summary>
    /// The port was disconnected.
    /// </summary>
    /// <remarks>
    /// The port was disconnected.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("5EFDEE16-7858-4119-B8BB-7394FFBCC02D")]
    [System.ComponentModel.DescriptionAttribute("PortDisconnectedEventArgs Interface")]
    interface IPortDisconnectedEventArgs
    {
        /// <summary>
        /// The name of the port being disconnected.</summary>
        String PortName
        {
            get;
        }
    };
    /// <summary>
    /// The port was disconnected.
    /// </summary>
    /// <remarks>
    /// The port was disconnected.
    /// </remarks>
    [Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(true)]
    [System.Runtime.InteropServices.GuidAttribute("693F33AA-EE4A-4CDF-9BA1-8889086BC8AB")]
    [System.Runtime.InteropServices.ClassInterface(System.Runtime.InteropServices.ClassInterfaceType.None)]
    public class PortDisconnectedEventArgs : System.EventArgs,
        IPortDisconnectedEventArgs
    {
        private String m_portName;
        /// <summary>Creates an instance of the PortDisconnectedEventArgs class for the port.</summary>
        /// <remarks>You can use this constructor when raising the PortDisconnectedEventArgs at run time to  
        /// inform the system that the port was disconnected.
        /// </remarks>
        public PortDisconnectedEventArgs(String portName)
            : base()
        {
            m_portName = portName;
        }
        /// <summary>
        /// The name of the port being disconnected.</summary>
        /// <value>The name of the port being disconnected.</value>
        public String PortName
        {
            get
            {
                return m_portName;
            }
        }
    };


    /// <summary>
	///	Port variables for equation-oriented simulators.
	/// </summary>
	/// <remarks>
	/// This interface is optional and would be implemented by a port object. It is intended 
	/// to allow a port to describe which Equation-oriented variables are associated with it and 
	/// should only be implemented for the ports contained in a unit operation which supports the
	/// ICapeNumericESO interface described in “CAPE-OPEN Interface Specification – Numerical 
	/// Solvers”.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid(COGuids.ICapeUnitPortVariables_IID)]
	[System.ComponentModel.DescriptionAttribute("ICapeUnitPortVariables Interface")]
	public interface ICapeUnitPortVariables
	{	
		/// <summary>
		///	The position of a port variable in the EO model.
		/// </summary>
		/// <remarks>
		///	Gets the position of a port variable in the EO model - used to 
		/// correctly build the equations representing a connection to this port.
		///  Variable type can be - flowrate, temperature, pressure, 
		/// specificEnthalpy, VaporFraction and for Vapour fraction component 
		/// name must also be specified. 
		/// </remarks>
		/// <param name = "Variable_type">The Type of the variable.</param>
		/// <param name = "Component">The compnent of the variable.</param>
		/// <value>The position of the variable.</value>
        [System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("Return index of port variable in EO Model given its type")]
        int Variable(String Variable_type, String Component);

        /// <summary>
		/// Sets the position of port variables: this should ultimately 
		/// be a private member function.
		/// </summary>
		/// <remarks>
		/// Sets the position of port variables: this should ultimately 
		/// be a private member function.
		/// </remarks>
		/// <param name = "Variable_type">The Type of the variable.</param>
		/// <param name = "Component">The compnent of the variable.</param>
		/// <param name = "index">The index of the variable.</param>
		[System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("Set index of port variable in EO model given its type")]
		void SetIndex(String Variable_type, String Component, int index);
	};
};
