using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{
	/* IMPORTANT NOTICE

	(c) AmsterCHEM, 2008.
	All rights are reserved unless specifically stated otherwise

	Visit the web site at www.amsterchem.com or www.cocosimulator.org

	*/

	// This file was developed/modified by Jasper van Baten

	/*****************************************
	CAPE-OPEN Flowsheet monitoring interface
	*****************************************/

	/// <summary>
	/// Indicates solution status of the monitored flowsheet.
	/// </summary>
	/// <remarks>
	/// This enumeration provides the flowsheeting monitoring object with information about the solution status of the flowsheet.
	/// </remarks>
	[Serializable]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.GuidAttribute("D1B15843-C0F5-4CB7-B462-E1B80456808E")]
    public enum CapeSolutionStatus
    {
		/// <summary>
		/// The flowsheet solved without error.
		/// </summary>
		CAPE_SOLVED = 0,
		/// <summary>
		/// Signifies that there has been no attempt to solve the flowsheet.
		/// </summary>
		CAPE_NOT_SOLVED = 1,
		/// <summary>
		/// The last attempt to solve the flowsheet did not converge.
		/// </summary>
		CAPE_FAILED_TO_CONVERGE = 2,
		/// <summary>
		/// The last attempt to solve the flowsheet timed out.
		/// </summary>
		CAPE_TIMED_OUT = 3,
		/// <summary>
		/// The last attempt to solve the flowsheet failed to solve due to lack of memory.
		/// </summary>
		CAPE_NO_MEMORY = 4,
		/// <summary>
		/// The last attempt to solve the flowsheet failed to initialize.
		/// </summary>
		CAPE_FAILED_INITIALIZATION = 5,
		/// <summary>
		/// The last attempt to solve the flowsheet produced a solving error.
		/// </summary>
		CAPE_SOLVING_ERROR = 6,
		/// <summary>
		/// The last attempt to solve the flowsheet failed due to an invalid operation.
		/// </summary>
		CAPE_INVALID_OPERATION = 7,
		/// <summary>
		/// The last attempt to solve the flowsheet failed due to an invalid invocation order.
		/// </summary>
		CAPE_BAD_INVOCATION_ORDER = 8,
		/// <summary>
		/// The last attempt to solve the flowsheet produced a computation error.
		/// </summary>
		CAPE_COMPUTATION_ERROR = 9
	};


	/// <summary>
	/// This interface provides information 
	/// about error that result from values that are outside of their bounds. It can be raised 
	/// to indicate that the value of either a method argument or the value of a object 
	/// parameter is out of range.
	/// </summary>
	/// <remarks>
	/// <para>Moniting object should implement:</para>
	/// <list type="bullet">
	/// <item>ICapeIdentification</item>
	/// <item>IPersistStream or IPersistStreamInit in case of persistence implementation</item>
	/// <item>ICapeUtilities for parameter collection and Edit and for accepting an ICapeSimulationContext</item>
	/// <item>CAPE-OPEN error handling interfaces (ECape...)</item>
	/// </list>
	/// <para>Monitoring objects can possibly access the following interface via the ICapeSimulationContext interface:</para>
	/// <list type="bullet">
	/// <item>ICapeCOSEUtilities - for named values</item>
	/// <item>ICapeDiagnostic - for logging and popping up messages</item>
	/// <item>ICapeMaterialTemplateSystem - for accessing material templates and creation of material objects</item>
	/// <item>ICapeFlowsheetMonitoring - for accessing collections of streams and unit operations</item>
	/// </list>
	/// <para>Monitoring object are NOT SUPPOSED to change flowsheet configuration by:</para>
	/// <list type="bullet">
	/// <item>modifying unit operation paramaters</item>
	/// <item>connecting or disconnecting streams to unit operations</item>
	/// <item>calculating unit operations</item>
	/// <item>modifying streams by setting values</item>
	/// <item>any other action that will change the flowsheet state</item>
	/// </list>
	/// <para>Monitoring object can:</para>
	/// <list type="bullet">
	/// <item>obtain stream information</item>
	/// <item>obtain unit operation information</item>
	/// <item>duplicate material objects of streams for performing thermodynamic calculations</item>
	/// <item>create material objects via the ICapeMaterialTemplateSystem for creating thermodynamic calculations</item>
	/// <item>... </item>
	/// </list>
	/// <para>In addition to the CAPE-OPEN object caterogy ID, monitoring objects 
	/// should expose the Monitoring Object category ID:</para>
	/// <para>CATID_MONITORING_OBJECT = 7BA1AF89-B2E4-493d-BD80-2970BF4CBE99</para>
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute("834F65CC-29AE-41c7-AA32-EE8B2BAB7FC2")]
	[System.ComponentModel.DescriptionAttribute("ICapeFlowsheetMonitoring Interface")]
	public interface ICapeFlowsheetMonitoring
	{
		/// <summary>
		/// Get the collection of streams.
		/// </summary>
		/// <remarks>
		/// Get the collection of streams
		/// returns an ICapeCollection object enumerating streams
		/// each stream is identified via ICapeIdentification
		/// material streams expose ICapeThermoMaterial or ICapeThermoMaterialObject
		/// energy streams and information streams expose ICapeCollection, each item in the collection is an ICapeParameter object
		/// </remarks>
		/// <returns>
		/// An <see cref = "ICapeCollection"/> of unit operations.
		/// </returns>
		[System.Runtime.InteropServices.DispIdAttribute(1)]
		[System.ComponentModel.DescriptionAttribute("method GetStreamCollection.")] 
		[return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
		Object  GetStreamCollection();

		/// <summary>
		/// Get the collection of unit operations.
		/// </summary>
		/// <remarks>
		/// Get the collection of unit operations
		/// returns an ICapeCollection object enumerating unit operations
		/// each unit operation is identified via ICapeIdentification
		/// unit operations also expose ICapeUnit, and possibly ICapeUtilities (for parameter access)
		/// </remarks>
		/// <returns>
		/// An <see cref = "ICapeCollection"/>  of streams.
		/// </returns>
		[System.Runtime.InteropServices.DispIdAttribute(2)]
		[System.ComponentModel.DescriptionAttribute("method GetUnitOperationCollection")] 
		[return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
		Object GetUnitOperationCollection();

		/// <summary>
		/// Gets the current solution status of the flowsheet.
		/// </summary>
		/// <remarks>
        /// Gets the <see cref = "CapeSolutionStatus"/> of the flowsheet.
		/// </remarks>
		/// <value>
        /// The <see cref = "CapeSolutionStatus"/> of the flowsheet.
		/// </value>
		//Get the solution status
		//[id(3), propget, helpstring("Solution status")]
		[System.Runtime.InteropServices.DispIdAttribute(3)]
		[System.ComponentModel.DescriptionAttribute("Solution status.")] 
		CapeOpen.CapeSolutionStatus SolutionStatus 
		{
			get;
		}

		/// <summary>
		/// Gets the validation status of the flowsheet.
		/// </summary>
		/// <remarks>
        /// Gets the The <see cref = "CapeValidationStatus"/> of the flowsheet.
		/// </remarks>
		/// <returns>
        /// the The <see cref = "CapeValidationStatus"/> of the flowsheet.
		/// </returns>
		[System.Runtime.InteropServices.DispIdAttribute(4)]
		[System.ComponentModel.DescriptionAttribute("Get the flowsheet validation status.")] 
		CapeOpen.CapeValidationStatus ValStatus
		{
			get;
		}


	};


	/// <summary>
	/// This interface provides information 
	/// about error that result from values that are outside of their bounds. It can be raised 
	/// to indicate that the value of either a method argument or the value of a object 
	/// parameter is out of range.
	/// </summary>
	/// <remarks>
	/// <para>Moniting object should implement:</para>
	/// <list type="bullet">
	/// <item>ICapeIdentification</item>
	/// <item>IPersistStream or IPersistStreamInit in case of persistence implementation</item>
	/// <item>ICapeUtilities for parameter collection and Edit and for accepting an ICapeSimulationContext</item>
	/// <item>CAPE-OPEN error handling interfaces (ECape...)</item>
	/// </list>
	/// <para>Monitoring objects can possibly access the following interface via the ICapeSimulationContext interface:</para>
	/// <list type="bullet">
	/// <item>ICapeCOSEUtilities - for named values</item>
	/// <item>ICapeDiagnostic - for logging and popping up messages</item>
	/// <item>ICapeMaterialTemplateSystem - for accessing material templates and creation of material objects</item>
	/// <item>ICapeFlowsheetMonitoring - for accessing collections of streams and unit operations</item>
	/// </list>
	/// <para>Monitoring object are NOT SUPPOSED to change flowsheet configuration by:</para>
	/// <list type="bullet">
	/// <item>modifying unit operation paramaters</item>
	/// <item>connecting or disconnecting streams to unit operations</item>
	/// <item>calculating unit operations</item>
	/// <item>modifying streams by setting values</item>
	/// <item>any other action that will change the flowsheet state</item>
	/// </list>
	/// <para>Monitoring object can:</para>
	/// <list type="bullet">
	/// <item>obtain stream information</item>
	/// <item>obtain unit operation information</item>
	/// <item>duplicate material objects of streams for performing thermodynamic calculations</item>
	/// <item>create material objects via the ICapeMaterialTemplateSystem for creating thermodynamic calculations</item>
	/// <item>... </item>
	/// </list>
	/// <para>In addition to the CAPE-OPEN object caterogy ID, monitoring objects 
	/// should expose the Monitoring Object category ID:</para>
	/// <para>CATID_MONITORING_OBJECT = 7BA1AF89-B2E4-493d-BD80-2970BF4CBE99</para>
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.GuidAttribute(COGuids.ICapeFlowsheetMonitoring_IID)]
	[System.ComponentModel.DescriptionAttribute("ICapeFlowsheetMonitoring Interface")]
	public interface ICapeFlowsheetMonitoringOld
	{
		/// <summary>
		/// Get the collection of streams.
		/// </summary>
		/// <remarks>
		/// Get the collection of streams
		/// returns an ICapeCollection object enumerating streams
		/// each stream is identified via ICapeIdentification
		/// material streams expose ICapeThermoMaterial or ICapeThermoMaterialObject
		/// energy streams and information streams expose ICapeCollection, each item in the collection is an ICapeParameter object
		/// </remarks>
		/// <returns>
		/// An ICapeCollection of unit operations.
		/// </returns>
		[System.Runtime.InteropServices.DispIdAttribute(1)]
		[System.ComponentModel.DescriptionAttribute("method GetStreamCollection.")] 
		[return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
		Object  GetStreamCollection();

		/// <summary>
		/// Get the collection of unit operations.
		/// </summary>
		/// <remarks>
		/// Get the collection of unit operations
		/// returns an ICapeCollection object enumerating unit operations
		/// each unit operation is identified via ICapeIdentification
		/// unit operations also expose ICapeUnit, and possibly ICapeUtilities (for parameter access)
		/// </remarks>
		/// <returns>
		/// An ICapeCollection of streams.
		/// </returns>
		[System.Runtime.InteropServices.DispIdAttribute(2)]
		[System.ComponentModel.DescriptionAttribute("method GetUnitOperationCollection")] 
		[return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.IDispatch)]
		Object GetUnitOperationCollection();

		/// <summary>
		/// Check whether the flowsheet is currently solved.
		/// </summary>
		/// <remarks>
		/// Check whether the flowsheet is currently solved
		/// </remarks>
		/// <returns>
		/// <para>true, if the unit is solved.</para>
		/// <para>false, if the unit is not solved.</para>
		/// </returns>
		[System.Runtime.InteropServices.DispIdAttribute(3)]
		[System.ComponentModel.DescriptionAttribute("method IsSolved")] 
		[return: System.Runtime.InteropServices.MarshalAs(System.Runtime.InteropServices.UnmanagedType.VariantBool)]
		bool IsSolved();

		/// <summary>
		/// Check whether the flowsheet is valid.
		/// </summary>
		/// <remarks>
		/// Check whether the flowsheet is valid
		/// </remarks>
		/// <value>
		/// The validation status of the flowsheet.
		/// </value>
        [System.Runtime.InteropServices.DispIdAttribute(4)]
        [System.ComponentModel.DescriptionAttribute("Get the flowsheet validation status.")]
        CapeOpen.CapeValidationStatus ValStatus
        {
            get;
        }
	};
}