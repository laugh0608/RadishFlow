using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

/* IMPORTANT NOTICE
(c) The CAPE-OPEN Laboratory Network, 2002.
All rights are reserved unless specifically stated otherwise

Visit the web site at www.colan.org

This file has been edited using the editor from Microsoft Visual Studio 6.0
This file can viewed properly with any basic editors and browsers (validation done under MS Windows and Unix)
*/

// This file was developed/modified by JEAN-PIERRE BELAUD for CO-LaN organisation - March 2003

// This file was modified by Bill Barrett of USEPA to restore CAPE-OPENv0.93 interface - November 30,2005.


// ---- The scope of the error interface ------------------------------
// Reference document: Error Common Interface
namespace CapeOpen {

	//.NET translations of the Error Interfaces:
	/// <summary>
	/// Enumeration of the various HRESULT values for the CAPE-OPEN error handling
	/// interfaces.
	/// </summary>
	/// <remarks>
	/// <para>Extracted from "Strategies for Handling Errors in COM+", in Platform SDK help</para>
	/// <para>Use the FACILITY_ITF range of errors to report interface-specific errors.</para>
	/// <para>Interface-specific errors should be in the FACILITY_ITF range of errors,between 
	/// 0x0200 and 0xFFFF. However, since Microsoft uses some codes after 0x0200, the CAPE-OPEN
	/// error codes will start at 0x0500</para>
	/// <para>Use the MAKE_HRESULT macro in C++ to introduce an interface-specific error code, 
	/// as shown in the following example:</para>
	/// <para>const HRESULT ERROR_NUMBER = MAKE_HRESULT (SEVERITY_ERROR, FACILITY_ITF,10);</para>
	/// <para>So, offset on FIRST_E_INTERFACE_HR must be between 1 and 64255
	/// (0xFFFF-0x0500). We reserve the 0 offset.</para>
	/// <para>const int FIRST_E_INTERFACE_HR = (int)0x80040500;</para>
	/// <para>last HR value used for a CO error interface
	/// const int LAST_USED_E_INTERFACE_HR = (int)0x80040517;</para>
	/// <para>highest HR value that could be used to represent a CO error interface
	/// const int LAST_E_INTERFACE_HR = (int)0x8004FFFF;</para>
	/// </remarks>
	[Serializable]
	public enum CapeErrorInterfaceHR: int {
		/// <summary>
		/// 0x80040501
		/// </summary>
		ECapeUnknownHR = unchecked ((int)0x80040501),
		/// <summary>
		/// 0x80040502
		/// </summary>
		ECapeDataHR = unchecked ((int)0x80040502),
		/// <summary>
		/// 0x80040503
		/// </summary>
        ECapeLicenceErrorHR = unchecked((int)0x80040503),
		/// <summary>
		/// 0x80040504
		/// </summary>
		ECapeBadCOParameterHR = unchecked ((int)0x80040504),
		/// <summary>
		/// 0x80040505
		/// </summary>
		ECapeBadArgumentHR = unchecked ((int)0x80040505),
		/// <summary>
		/// 0x80040506
		/// </summary>
		ECapeInvalidArgumentHR = unchecked ((int)0x80040506),
		/// <summary>
		/// 0x80040507
		/// </summary>
		ECapeOutOfBoundsHR = unchecked ((int)0x80040507),
		/// <summary>
		/// 0x80040508
		/// </summary>
		ECapeImplementationHR = unchecked ((int)0x80040508),
		/// <summary>
		/// 0x80040509 
		/// </summary>
		ECapeNoImplHR = unchecked ((int)0x80040509),
		/// <summary>
		/// 0x8004050A
		/// </summary>
		ECapeLimitedImplHR = unchecked ((int)0x8004050A),
		/// <summary>
		/// 0x8004050B
		/// </summary>
		ECapeComputationHR = unchecked ((int)0x8004050B),
		/// <summary>
		/// 0x8004050C
		/// </summary>
		ECapeOutOfResourcesHR = unchecked ((int)0x8004050C),
		/// <summary>
		/// 0x8004050D
		/// </summary>
		ECapeNoMemoryHR = unchecked ((int)0x8004050D),
		/// <summary>
		/// 0x8004050E
		/// </summary>
		ECapeTimeOutHR = unchecked ((int)0x8004050E),
		/// <summary>
		/// 0x8004050F
		/// </summary>
		ECapeFailedInitialisationHR = unchecked ((int)0x8004050F),
		/// <summary>
		/// 0x80040510
		/// </summary>
		ECapeSolvingErrorHR = unchecked ((int)0x80040510),
		/// <summary>
		/// 0x80040511
		/// </summary>
		ECapeBadInvOrderHR = unchecked ((int)0x80040511),
		/// <summary>
		/// 0x80040512
		/// </summary>
		ECapeInvalidOperationHR = unchecked ((int)0x80040512),
		/// <summary>
		/// 0x80040513
		/// </summary>
		ECapePersistenceHR = unchecked ((int)0x80040513),
		/// <summary>
		/// 0x80040514
		/// </summary>
		ECapeIllegalAccessHR = unchecked ((int)0x80040514),
		/// <summary>
		/// 0x80040515
		/// </summary>
		ECapePersistenceNotFoundHR = unchecked ((int)0x80040515),
		/// <summary>
		/// 0x80040516
		/// </summary>
		ECapePersistenceSystemErrorHR = unchecked ((int)0x80040516),
		/// <summary>
		/// 0x80040517
		/// </summary>
		ECapePersistenceOverflowHR = unchecked ((int)0x80040517),
		/// <summary>
		/// 0x80040518, Specific to MINLP
		/// </summary>
		ECapeOutsideSolverScopeHR = unchecked ((int)0x80040518), // specific to MINLP
		/// <summary>
		/// 0x80040519, Specific to MINLP
		/// </summary>
		ECapeHessianInfoNotAvailableHR = unchecked ((int)0x80040519), // specific to MINLP
		/// <summary>
		/// 0x80040519, Specific to MINLP
		/// </summary>
		ECapeThrmPropertyNotAvailableHR = unchecked ((int)0x80040520)
	};

	// ECapeBoundaries interface
	/// <summary>
	/// This interface provides information 
	/// about error that result from values that are outside of their bounds. It can be raised 
	/// to indicate that the value of either a method argument or the value of a object 
	/// parameter is out of range.
	/// </summary>
	/// <remarks>
	/// <para>ECapeBoundaries is a "utility" interface which factorises a state which describes the value, its type and its boundaries.</para>
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeBoundaries_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeBoundaries Interface")]
	public interface ECapeBoundaries
	{	
		/// <summary>
		/// The value of the lower bound.
		/// </summary>
		/// <remarks>
		/// <para>This provides the user with the acceptable lower bounds of the argument.</para>
		/// </remarks>
		/// <value>The lower bound for the argument.</value>
		[System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("The value of the lower bound.")] 
		double lowerBound
		{
			get;
		}

		/// <summary>
		/// The value of the upper bound.
		/// </summary>
		/// <remarks>
		/// <para>This provides the user with the acceptable upper bounds of the argument.</para>
		/// </remarks>
		/// <value>The upper bound for the argument.</value>
		[System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("The value of the upper bound.")] 
		double upperBound
		{
			get;
		}

		/// <summary>
		/// The current value which has led to an error.
		/// </summary>
		/// <remarks>
		/// <para>This provides the user with the value that caused the error condition.</para>
		/// </remarks>
		/// <value>The value that resulted in the error condition.</value>
		[System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("The current value which has led to an error..")] 
		double value
		{
			get;
		}

		/// <summary>
		/// The type/nature of the value. 
		/// </summary>
		/// <remarks>
		/// The value could represent a thermodynamic property, a number of tables in a database, a quantity of memory, ..."
		/// </remarks>
		/// <value>A string that indicates the anture or type of the value required.</value>
		[System.Runtime.InteropServices.DispIdAttribute(4), System.ComponentModel.DescriptionAttribute("The type/nature of the value. The value could represent a thermodynamic property, a number of tables in a database, a quantity of memory, ...")] 
		String type
		{
			get;
		}
	};


	/// <summary>
	/// The root CAPE-OPEN Exception interface. 
	/// </summary>
	/// <remarks>
	/// The interface of the CAPE-OPEN errors hierarchy. The System package and the ECapeUser 
	/// interface depend on this error.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeRoot_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeRoot Interface")]
	public interface ECapeRoot
	{
		/// <summary>
		/// The name of the error. This is a mandatory field.
		/// </summary>
		/// <remarks>
		/// The name of the error. This is a mandatory field.
		/// </remarks>
		/// <value>
		/// The name of the error. This is a mandatory field.
		/// </value>
		[System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("error Name")] 
		String Name
		{
			get;
		}
	};



	// ECapeUser interface
	/// <summary>
	/// The base interface of the CO errors hierarchy. 
	/// </summary>
	/// <remarks>
	/// The ECapeUser interface defines the minimum state of a CO error.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeUser_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeUser Interface")]
	public interface ECapeUser 
	{	
		/// <summary>
		/// Code to designate the subcategory of the error. 
		/// </summary>
		/// <remarks>
		/// <para>The error code is used as the function return HRESULT in the COM calling pattern.
		/// When a .Net-based component throws an exception, the HRESULT assigned to the
		/// exception is returned to the COM-based caller. It is important to set the 
		/// exception HRESULT value to provide HRESULT information to a COM caller.
		/// </para>
		/// <para>The assignment of values is left to each implementation. So that is a 
		/// proprietary code specific to the CO component provider. By default, set to 
		/// the CAPE-OPEN error HRESULT <see cref = "CapeErrorInterfaceHR"/>.</para>
		/// </remarks>
		/// <value>
		/// The HRESULT value for the exception.
		/// </value>
		[System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("Code to designate the subcategory of the error. The assignment of values is left to each implementation. So that is a proprietary code specific to the CO component provider.")] 
		int code
		{
			get;
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
		[System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("The description of the error.")] 
		String description
		{
			get;
		}

		/// <summary>
		/// The scope of the error.
		/// </summary>
		/// <remarks>
		/// This property provides a list of packages where the error occurs separated by '.'. 
		/// For example CapeOpen.Common.Identification.
		/// </remarks>
		/// <value>The source of the error.</value>
		[System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("The scope of the error. The list of packages where the error occurs separated by '.'. For example CapeOpen.Common.Identification.")] 
		String scope
		{
			get;
		}

		/// <summary>
		/// The name of the interface where the error is thrown. This is a mandatory field."
		/// </summary>
		/// <remarks>
		/// The interface that the error was thrown.
		/// </remarks>
		/// <value>The name of the interface.</value>
		[System.Runtime.InteropServices.DispIdAttribute(4), System.ComponentModel.DescriptionAttribute("The name of the interface where the error is thrown. This is a mandatory field.")] 
		String interfaceName
		{
			get;
		}

		/// <summary>
		/// The name of the operation where the error is thrown. This is a mandatory field.
		/// </summary>
		/// <remarks>
		/// This field provides the name of the operation being perfomed when the exception was raised.
		/// </remarks>
		/// <value>The operation name.</value>
		[System.Runtime.InteropServices.DispIdAttribute(5), System.ComponentModel.DescriptionAttribute("The name of the operation where the error is thrown. This is a mandatory field.")] 
		System.String operation
		{
			get;
		}

		/// <summary>
		/// An URL to a page, document, web site,  where more information on the error can be found. The content of this information is obviously implementation dependent.
		/// </summary>
		/// <remarks>
		/// This field provides an internet URL where more information about the error can be found.
		/// </remarks>
		/// <value>The URL.</value>
		[System.Runtime.InteropServices.DispIdAttribute(6), System.ComponentModel.DescriptionAttribute("An URL to a page, document, web site,  where more information on the error can be found. The content of this information is obviously implementation dependent.")] 
		String moreInfo
		{
			get;
		}

	};

	// ECapeUnknown interface
	/// <summary>
	/// This exception is raised when other error(s), specified by the operation, do not suit.
	/// </summary>
	/// <remarks>
	/// <para>
	/// A standard exception that can be thrown by a CAPE-OPEN object to indicate that the error
	/// that occurred was not one that was suitable for any of the other errors supported by the object. </para>
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeUnknown_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeUnknown Interface")]
	public interface ECapeUnknown {};



	// ECapeData interface
	/// <summary>
	/// The base interface for the errors hierarchy related to any data.
	/// </summary>
	/// <remarks>
	/// <para>
	/// The ECapeDataException interface is the base interface for errors related to data. The data are the 
	/// arguments of operations, the parameters coming from the Parameter Common Interface 
	/// and information on licence key.	
	/// </para>
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeData_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeData Interface")]
	public interface ECapeData {};



	// ECapeLicenceError interface
	/// <summary>
	/// An operation can not be completed because the licence agreement is not respected.
	/// </summary>
	/// <remarks>
	/// Of course, this type of error could also appear outside the CO scope. In this case, 
	/// the error does not belong to the CO error handling. It is specific to the platform.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeLicenceError_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeLicenceError Interface")]
	public interface ECapeLicenceError {};



	// ECapeBadCOParameter interface
	/// <summary>
	/// A parameter, which is an object from the Parameter Common Interface, has an invalid status.
	/// </summary>
	/// <remarks>
	/// The name of the invalid parameter, along with the parameter itself are available from the exception.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeBadCOParameter_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeBadCOParameter Interface")]
	public interface ECapeBadCOParameter {
		/// <summary>
		/// The name of the CO parameter that is throwing the exception.
		/// </summary>
		/// <remarks>
		/// This provides the name of the parameter that threw the exception.
		/// </remarks>
		/// <value>The name of the parameter that threw the exception.</value>
		[System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("The name of the CO parameter")] 
		String parameterName
		{
			get;
		}
		/// <summary>
		/// The parameter that threw the exception.
		/// </summary>
		/// <remarks>
		/// This method provides access directly to the parameter that threw the exception.
		/// </remarks>
		/// <value>A reference to the exception taht threw the exception.</value>
		[System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("The parameter")] 
		Object parameter
		{
			get;
		}
	};

	/// <summary>
	/// An invalid argument value was passed. 
	/// </summary>
	/// <remarks>
	/// The function call includes an invalid argument value. For instance the passed name of the phase 
	/// does not belong to the CO Phase List.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute("678c0b16-7d66-11d2-a67d-00105a42887f")]
	[System.ComponentModel.DescriptionAttribute("ECapeBadArgument Interface")]
	public interface ECapeBadArgument093
	{
		/// <summary>
		/// The position of the argument value within the signature of the operation. First argument is as position 1.
		/// </summary>
		/// <remarks>
		/// This provides the location of the invalid argument in the argument list for the function call.
		/// </remarks>
		/// <value>The position of the argument that is bad. The first argument is 1.
		/// </value>
		[System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("The position of the argument value within the signature of the operation. First argument is as position 1.")] 
		int position
		{
			get;
		}
	};

	// ECapeBadArgument interface for CAPE-OPENv1.0
	/// <summary>
	/// An invalid argument value was passed. 
	/// </summary>
	/// <remarks>
	/// The function call includes an invalid argument value. For instance the passed name of the phase 
	/// does not belong to the CO Phase List.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeBadArgument_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeBadArgument Interface")]
	public interface ECapeBadArgument {
		/// <summary>
		/// The position of the argument value within the signature of the operation. First argument is as position 1.
		/// </summary>
		/// <remarks>
		/// This provides the location of the invalid argument in the argument list for the function call.
		/// </remarks>
		/// <value>The position of the argument that is bad. The first argument is 1.
		/// </value>
		[System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("The position of the argument value within the signature of the operation. First argument is as position 1.")] 
		Int16 position
		{
            get;
		}
	};


	// ECapeInvalidArgument interface
	/// <summary>
	/// An invalid argument value was passed. For instance the passed name of 
	/// the phase does not belong to the CO Phase List.
	/// </summary>
	/// <remarks>
	/// An argument value of the operation is invalid. The position of the 
	/// argument value within the signature of the operation. First argument is as 
	/// position 1.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeInvalidArgument_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeInvalidArgument Interface")]
	public interface ECapeInvalidArgument {
	};



	// ECapeOutOfBounds interface
	/// <summary>
	/// An argument value is outside of the bounds..
	/// </summary>
	/// <remarks>
	/// <para>This class is derived from the <see cref = "ECapeBoundaries"/> interface.
	/// It is used to indicate that one of the parameters is outside of its bounds.</para>
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeOutOfBounds_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeOutOfBounds Interface")]
	public interface ECapeOutOfBounds {};

	// ECapeNoImpl interface
	/// <summary>
	/// An exception that indicates that the requested operation has not been implemented by the current object.
	/// </summary>
	/// <remarks>
	/// The operation is “not” implemented even if this operation can be called due 
	/// to the compatibility with the CO standard. That is to say that the operation 
	/// exists but it is not supported by the current implementation.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeNoImpl_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeNoImpl Interface")]
	public interface ECapeNoImpl {};



	// ECapeLimitedImpl interface
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
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeLimitedImpl_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeLimitedImpl Interface")]
	public interface ECapeLimitedImpl {};

	// ECapeImplementation interface
	/// <summary>
	/// The base class of the errors hierarchy related to the current implementation.
	/// </summary>
	/// <remarks>
	/// This class is used to indicate that an error occurred in the with the implementation of an object. 
	/// The implemenation-related classes such as 
	/// <see cref = "ECapeNoImpl"/> and 
	/// <see cref = "ECapeLimitedImpl"/>
	/// derive from this class.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeImplementation_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeImplementation Interface")]
	public interface ECapeImplementation {};

	// ECapeOutOfResources interface
	/// <summary>
	/// An exception that indicates that the resources required by this operation are not available.
	/// </summary>
	/// <remarks>
	/// The physical resources necessary to the execution of the operation are out of limits.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeOutOfResources_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeOutOfResources Interface")]
	public interface ECapeOutOfResources {};


	// ECapeNoMemory interface
	/// <summary>
	/// An exception that indicates that the memory required for this operation is not available.
	/// </summary>
	/// <remarks>
	/// The physical memory necessary to the execution of the operation is out of limit.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeNoMemory_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeNoMemory Interface")]
	public interface ECapeNoMemory {};



	// ECapeTimeOut interface
	/// <summary>
	/// Exception thrown when the time-out criterion is reached.
	/// </summary>
	/// <remarks>
	/// Exception thrown when the time-out criterion is reached.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeTimeOut_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeTimeOut Interface")]
	public interface ECapeTimeOut {};


	// ECapeFailedInitialisation interface
	/// <summary>
	/// This exception is thrown when necessary initialisation has not been performed or has failed.
	/// </summary>
	/// <remarks>
	/// The pre-requisites operations are not valid. The necessary initialisation has not been performed or has failed.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeFailedInitialisation_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeFailedInitialisation Interface")]
	public interface ECapeFailedInitialisation {};



	// ECapeSolvingError interface
	/// <summary>
	/// An exception that indicates a numerical algorithm failed for any reason.
	/// </summary>
	/// <remarks>
	/// Indicates that a numerical algorithm failed for any reason.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeSolvingError_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeSolvingError Interface")]
	public interface ECapeSolvingError {};

	// ECapeBadInvOrder interface
	/// <summary>
	/// The necessary pre-requisite operation has not been called prior to the operation request.
	/// </summary>
	/// <remarks>
	/// The specified prerequiste operation must be called prior to the operation throwing this exception.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeBadInvOrder_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeBadInvOrder Interface")]
	public interface ECapeBadInvOrder {
		/// <summary>
		/// The necessary prerequisite operation.
		/// </summary>
		[System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("The necessary prerequisite operation.")] 
		String requestedOperation
		{
			get;
		}
	};

	// ECapeInvalidOperation interface
	/// <summary>
	/// This operation is not valid in the current context.
	/// </summary>
	/// <remarks>
	/// This exception is thrown when an operation is attempted that is not valid in the current context.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid(COGuids.ECapeInvalidOperation_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeInvalidOperation Interface")]
	public interface ECapeInvalidOperation {};

	// ECapeComputation interface
	/// <summary>
	/// The base interface of the errors hierarchy related to calculations.
	/// </summary>
	/// <remarks>
	/// This class is used to indicate that an error occurred in the performance of a calculation. 
	/// Other calculation-related classes such as 
	/// <see cref = "ECapeFailedInitialisation"/>, 
	/// <see cref = "ECapeOutOfResources"/>, 
	/// <see cref = "ECapeSolvingError"/>, 
	/// <see cref = "ECapeBadInvOrder"/>, 
	/// <see cref = "ECapeInvalidOperation"/>, 
	/// <see cref = "ECapeNoMemory"/>, and 
	/// <see cref = "ECapeTimeOut"/> 
	/// derive from this class.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeComputation_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeComputation Interface")]
	public interface ECapeComputation {};

	// ECapePersistence interface
	/// <summary>
	/// An exception that indicates that the a persistence-related error has occurred.
	/// </summary>
	/// <remarks>
	/// The base of the errors hierarchy related to the persistence.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid(COGuids.ECapePersistence_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapePersistence Interface")]
	public interface ECapePersistence {};

	// ECapeIllegalAccess interface
	/// <summary>
	/// The access to something within the persistence system is not authorised.
	/// </summary>
	/// <remarks>
	/// This exception is thrown when the access to something within the persistence system is not authorised.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid(COGuids.ECapeIllegalAccess_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeIllegalAccess Interface")]
	public interface ECapeIllegalAccess {};



	// ECapePersistenceNotFound interface
	/// <summary>
	/// An exception that indicates that the persistence was not found.
	/// </summary>
	/// <remarks>
	/// The requested object, table, or something else within the persistence system was not found.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid(COGuids.ECapePersistenceNotFound_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapePersistenceNotFound Interface")]
	public interface ECapePersistenceNotFound {
		/// <summary>
		/// The name of the item.
		/// </summary>
		/// <remarks>
		/// The name of the requested object, table, or something else within the persistence system 
		/// that was not found.
		/// </remarks>
		/// <value>
		/// The name of the item not found.
		/// </value>
		[System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("The name of the item")] 
		String itemName
		{
			get;
		}

	};

	// ECapePersistenceSystemError interface
	/// <summary>
	/// An exception that indicates a severe error occurred within the persistence system.
	/// </summary>
	/// <remarks>
	/// During the persistence process, a severe error occurred within the persistence system.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid(COGuids.ECapePersistenceSystemError_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapePersistenceSystemError Interface")]
	public interface ECapePersistenceSystemError {};



	// ECapePersistenceOverflow interface
	/// <summary>
	/// An exception that indicates an overflow of internal persistence system.
	/// </summary>
	/// <remarks>
	/// During the persistence process, an overflow of internal persistence system occurred.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid(COGuids.ECapePersistenceOverflow_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapePersistenceOverflow Interface")]
	public interface ECapePersistenceOverflow {};

	/// <summary>
	/// An exception that indicates the requested thermodynamic property was not available.
	/// </summary>
	/// <remarks>
	/// At least one item in the requested properties cannot be returned. This could be 
	/// because the property cannot be calculated at the specified conditions or for the 
	/// specified Phase. If the property calculation is not implemented then 
	/// <see cref = "ECapeLimitedImpl"/> should be returned.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid("678C09B6-7D66-11D2-A67D-00105A42887F")]
	[System.ComponentModel.DescriptionAttribute("ECapeThrmPropertyNotAvailable Interface")]
	public interface ECapeThrmPropertyNotAvailable
	{ };

	/// <summary>
	/// Exception thrown when the Hessian for the MINLP problem is not available.
	/// </summary>
	/// <remarks>
	/// Exception thrown when the Hessian for the MINLP problem is not available.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid("3FF0B24B-4299-4DAC-A46E-7843728AD205")]
	[System.ComponentModel.DescriptionAttribute("ECapeHessianInfoNotAvailable Interface")]
	public interface ECapeHessianInfoNotAvailable 
	{
		/// <summary>
		/// Code to designate the subcategory of the error. 
		/// </summary>
		/// <remarks>
		/// The assignment of values is left to each implementation. So that is a 
		/// proprietary code specific to the CO component provider. By default, set to 
		/// the CAPE-OPEN error HRESULT <see cref = "CapeErrorInterfaceHR"/>.
		/// </remarks>
		/// <value>
		/// The HRESULT value for the exception.
		/// </value>
		[System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("Code to designate the subcategory of the error. The assignment of values is left to each implementation. So that is a proprietary code specific to the CO component provider.")] 
		int code
		{
			get;
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
		[System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("The description of the error.")] 
		String description
		{
			get;
		}

		/// <summary>
		/// The scope of the error.
		/// </summary>
		/// <remarks>
		/// This property provides a list of packages where the error occurs separated by '.'. 
		/// For example CapeOpen.Common.Identification.
		/// </remarks>
		/// <value>The source of the error.</value>
		[System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("The scope of the error. The list of packages where the error occurs separated by '.'. For example CapeOpen.Common.Identification.")] 
		String scope
		{
			get;
		}

		/// <summary>
		/// The name of the interface where the error is thrown. This is a mandatory field."
		/// </summary>
		/// <remarks>
		/// The interface that the error was thrown.
		/// </remarks>
		/// <value>The name of the interface.</value>
		[System.Runtime.InteropServices.DispIdAttribute(4), System.ComponentModel.DescriptionAttribute("The name of the interface where the error is thrown. This is a mandatory field.")] 
		String interfaceName
		{
			get;
		}

		/// <summary>
		/// The name of the operation where the error is thrown. This is a mandatory field.
		/// </summary>
		/// <remarks>
		/// This field provides the name of the operation being perfomed when the exception was raised.
		/// </remarks>
		/// <value>The operation name.</value>
		[System.Runtime.InteropServices.DispIdAttribute(5), System.ComponentModel.DescriptionAttribute("The name of the operation where the error is thrown. This is a mandatory field.")] 
		String operation
		{
			get;
		}

		/// <summary>
		/// An URL to a page, document, web site,  where more information on the error can be found. The content of this information is obviously implementation dependent.
		/// </summary>
		/// <remarks>
		/// This field provides an internet URL where more information about the error can be found.
		/// </remarks>
		/// <value>The URL.</value>
		[System.Runtime.InteropServices.DispIdAttribute(6), System.ComponentModel.DescriptionAttribute("An URL to a page, document, web site,  where more information on the error can be found. The content of this information is obviously implementation dependent.")] 
		String moreInfo
		{
			get;
		}
	};
	/// <summary>
	/// Exception thrown when the problem is outside the scope of the solver.
	/// </summary>
	/// <remarks>
	/// Exception thrown when the problem is outside the scope of the solver.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid("678c0b0f-7d66-11d2-a67d-00105a42887f")]
	[System.ComponentModel.DescriptionAttribute("ECapeOutsideSolverScope Interface")]
	public interface ECapeOutsideSolverScope 
	{
		/// <summary>
		/// Code to designate the subcategory of the error. 
		/// </summary>
		/// <remarks>
		/// The assignment of values is left to each implementation. So that is a 
		/// proprietary code specific to the CO component provider. By default, set to 
		/// the CAPE-OPEN error HRESULT <see cref = "CapeErrorInterfaceHR"/>.
		/// </remarks>
		/// <value>
		/// The HRESULT value for the exception.
		/// </value>
		[System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("Code to designate the subcategory of the error. The assignment of values is left to each implementation. So that is a proprietary code specific to the CO component provider.")] 
		int code
		{
			get;
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
		[System.Runtime.InteropServices.DispIdAttribute(2), System.ComponentModel.DescriptionAttribute("The description of the error.")] 
		String description
		{
			get;
		}

		/// <summary>
		/// The scope of the error.
		/// </summary>
		/// <remarks>
		/// This property provides a list of packages where the error occurs separated by '.'. 
		/// For example CapeOpen.Common.Identification.
		/// </remarks>
		/// <value>The source of the error.</value>
		[System.Runtime.InteropServices.DispIdAttribute(3), System.ComponentModel.DescriptionAttribute("The scope of the error. The list of packages where the error occurs separated by '.'. For example CapeOpen.Common.Identification.")] 
		String scope
		{
			get;
		}

		/// <summary>
		/// The name of the interface where the error is thrown. This is a mandatory field."
		/// </summary>
		/// <remarks>
		/// The interface that the error was thrown.
		/// </remarks>
		/// <value>The name of the interface.</value>
		[System.Runtime.InteropServices.DispIdAttribute(4), System.ComponentModel.DescriptionAttribute("The name of the interface where the error is thrown. This is a mandatory field.")] 
		String interfaceName
		{
			get;
		}

		/// <summary>
		/// The name of the operation where the error is thrown. This is a mandatory field.
		/// </summary>
		/// <remarks>
		/// This field provides the name of the operation being perfomed when the exception was raised.
		/// </remarks>
		/// <value>The operation name.</value>
		[System.Runtime.InteropServices.DispIdAttribute(5), System.ComponentModel.DescriptionAttribute("The name of the operation where the error is thrown. This is a mandatory field.")] 
		String operation
		{
			get;
		}

		/// <summary>
		/// An URL to a page, document, web site,  where more information on the error can be found. The content of this information is obviously implementation dependent.
		/// </summary>
		/// <remarks>
		/// This field provides an internet URL where more information about the error can be found.
		/// </remarks>
		/// <value>The URL.</value>
		[System.Runtime.InteropServices.DispIdAttribute(6), System.ComponentModel.DescriptionAttribute("An URL to a page, document, web site,  where more information on the error can be found. The content of this information is obviously implementation dependent.")] 
		String moreInfo
		{
			get;
		}
	};

	// typedef CapeErrorInterfaceHR eCapeErrorInterfaceHR;



	/// <summary>
	/// The ECapeErrorDummy interface is not intended to be used. 
	/// </summary>
	/// <remarks>
	/// It is only here to ensure that 
	/// the MIDL compiler exports the CapeErrorInterfaceHR enumeration. The compiler only exports 
	/// an enumeration if it is used in a method of an exported interface. 
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ECapeErrorDummy_IID)]
	[System.ComponentModel.DescriptionAttribute("ECapeErrorDummy Interface")]
	public interface ECapeErrorDummy
	{
		/// <summary>
		/// The HRESULT of the Dummy Error.
		/// </summary>
		/// <remarks>
		/// The HRESULT of the Dummy Error.
		/// </remarks>
		/// <value>
		/// The HRESULT of the Dummy Error.
		/// </value>
		[System.Runtime.InteropServices.DispIdAttribute(1), System.ComponentModel.DescriptionAttribute("property Name")] 
		int dummy
		{
			get;
		}

	};
}
