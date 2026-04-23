using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

//#ifndef _COSE_IDL_
//#define _COSE_IDL_

/* IMPORTANT NOTICE
(c) The CAPE-OPEN Laboratory Network, 2002.
All rights are reserved unless specifically stated otherwise

Visit the web site at www.colan.org

This file has been edited using the editor from Microsoft Visual Studio 6.0
This file can viewed properly with any basic editors and browsers (validation done under MS Windows and Unix)
*/

// This file was developed/modified by JEAN-PIERRE BELAUD for CO-LaN organisation - August 2003


namespace CapeOpen {

	// -------------------------------------------------------------------
	// ---- The scope of the COSE interfaces ----------------------------------
	// -------------------------------------------------------------------


	/// <summary>
	/// Encloses the diagnostic functionality.
	/// </summary>
	/// <remarks>
	/// An intferace to be supported by the PME in order to pass a reference to the 
	/// ICapeUtilities:SetSimulation to the PMC. The PMC may then 
	/// use any of the PME COSE interfaces.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid(COGuids.ICapeSimulationContext_IID)]
	[System.ComponentModel.DescriptionAttribute("ICapeSimulation Context Interface")]
	public interface ICapeSimulationContext
	{
		// no methods defined here...
	};

	//.NET Translation of the ICapeDiagnostic Interface.
	/// <summary>
	/// Provides a mechanism to provide verbose messages to the user.
	/// </summary>
	/// <remarks>
	/// The communication of verbose information from the PMC to the PME (and hence to the
	///user). PMCs should be able to log or display information to the user while it is executing 
	/// a flowsheet. Rather than each PMC performing these tasks by the means of different
	/// mechanisms, it is much preferable to redirect them all to the PME services for
	/// communicating with the user. The Error Common Interfaces do not fulfil these requirements,
	/// since they stop the execution of the PMC code and signal an abnormal situation to the PME.
	/// The document deals with the transferral of simple informative or warning messages.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid(COGuids.ICapeDiagnostic_IID)]
	[System.ComponentModel.DescriptionAttribute("ICapeDiagnostic Interface")]
	public interface ICapeDiagnostic
	{
		/// <summary>
		/// Writes a message to the terminal.
		/// </summary>
		/// <remarks>
		/// <para>Write a string to the terminal.</para>
		/// <para>This method is called when a message needs to be brought to the user’s attention.
		/// The implementation should ensure that the string is written out to a dialogue box or 
		/// to a message list that the user can easily see.</para>
		/// <para>A priori this message has to be displayed as soon as possible to the user.</para>
		/// </remarks>
		/// <param name = "message">The text to be displayed.</param>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
		[System.Runtime.InteropServices.DispId(1)]
		[System.ComponentModel.DescriptionAttribute("method PopUpMessage")] 
		void PopUpMessage(String message); 

		/// <summary>
		/// Writes a string to the PME's log file.
		/// </summary>
		/// <remarks>
		/// <para>Write a string to a log.</para>
		/// <para>This method is called when a message needs to be recorded for logging purposes. 
		/// The implementation is expected to write the string to a log file or other journaling 
		/// device.</para>
		/// </remarks>
		/// <param name = "message">The text to be logged.</param>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
		[System.Runtime.InteropServices.DispId(2)]
		[System.ComponentModel.DescriptionAttribute("method LogMessage")] 
		void LogMessage(String message);
	};

	/// <summary>
	/// Creates a new thermo material template of the specified type.
	/// </summary>
	/// <remarks>
	/// When a Unit Operation needs to obtain thermodynamic calculations, it will 
	/// typically perform them on the material objects attached to the Unit ports. However, 
	/// in some cases, like distillation columns, there may be the need to utilise a different 
	/// Property Package. Even the user could be requested to choose which thermodynamic 
	/// model to must be used. All the mechanisms for accessing CAPE-OPEN Property Packages 
	/// are already in the COSE´s, as part of the functionality necessary for making use of 
	/// CAPE-OPEN Property Packages. Therefore, instead of each PMC implementing support for 
	/// performing this selection and creation of thermo engine, delegating that 
	/// responsibility to the COSE will result in thinner and easier to code Unit Operation
	/// Components. If configuration of Material Templates is in the PME side, the only 
	/// additional functionality the Unit Operation would require is that for accessing the 
	/// list of already configured Material Templates, and picking one of them.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid(COGuids.ICapeMaterialTemplateSystem_IID)]
	[System.ComponentModel.DescriptionAttribute("ICapeMaterialTemplateSystem Interface")]
	public interface ICapeMaterialTemplateSystem
	{
		/// <summary>
		/// Creates a new thermo material template of the specified type.
		/// </summary>
		/// <remarks>
		/// When a Unit Operation needs to obtain thermodynamic calculations, it will 
		/// typically perform them on the material objects attached to the Unit ports. However, 
		/// in some cases, like distillation columns, there may be the need to utilise a different 
		/// Property Package. Even the user could be requested to choose which thermodynamic 
		/// model to must be used. All the mechanisms for accessing CAPE-OPEN Property Packages 
		/// are already in the COSE´s, as part of the functionality necessary for making use of 
		/// CAPE-OPEN Property Packages. Therefore, instead of each PMC implementing support for 
		/// performing this selection and creation of thermo engine, delegating that 
		/// responsibility to the COSE will result in thinner and easier to code Unit Operation
		/// Components. If configuration of Material Templates is in the PME side, the only 
		/// additional functionality the Unit Operation would require is that for accessing the 
		/// list of already configured Material Templates, and picking one of them.
		/// </remarks>
        /// <value>Returns StringArray of material template names supported by the COSE. This can include:
        /// <list type="bullet">
        /// <item>CAPE-OPEN standalone property packages </item>
        /// <item>CAPE-OPEN property packages that depend on a Property System </item>
        /// <item>Property packages that are native to the COSE.</item>
        /// </list>
        /// </value>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
		[System.Runtime.InteropServices.DispId(1)]
		[System.ComponentModel.DescriptionAttribute("property MaterialTemplates")] 
		Object MaterialTemplates
		{
			get;
		}

		/// <summary>
		/// Creates a new thermo material template of the specified type.
		/// </summary>
		/// <remarks>
		/// When a Unit Operation needs to obtain thermodynamic calculations, it will 
		/// typically perform them on the material objects attached to the Unit ports. However, 
		/// in some cases, like distillation columns, there may be the need to utilise a different 
		/// Property Package. Even the user could be requested to choose which thermodynamic 
		/// model to must be used. All the mechanisms for accessing CAPE-OPEN Property Packages 
		/// are already in the COSE´s, as part of the functionality necessary for making use of 
		/// CAPE-OPEN Property Packages. Therefore, instead of each PMC implementing support for 
		/// performing this selection and creation of thermo engine, delegating that 
		/// responsibility to the COSE will result in thinner and easier to code Unit Operation
		/// Components. If configuration of Material Templates is in the PME side, the only 
		/// additional functionality the Unit Operation would require is that for accessing the 
		/// list of already configured Material Templates, and picking one of them.
		/// </remarks>
		/// <returns>
		/// Returns StringArray of material template names supported by the COSE. This can include:
		/// - CAPE-OPEN standalone property packages
		/// - CAPE-OPEN property packages that depend on a Property System
		/// - Property packages that are native to the COSE.
		/// </returns>
		/// <param name = "materialTemplateName">TThe name of the material template to be resolved (which 
		/// must be included in the list returned by MaterialTemplates)</param>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
		[System.Runtime.InteropServices.DispId(2)]
		[System.ComponentModel.DescriptionAttribute("method CreateMaterialTemplate")]
		[return: System.Runtime.InteropServices.MarshalAsAttribute(System.Runtime.InteropServices.UnmanagedType.IDispatch)] 
		Object CreateMaterialTemplate(String materialTemplateName) ;
	};

	//.NET Translation of the ICapeCOSEUtilities Interface.
	/// <summary>
	/// Provides a mechanism for the PMC to obtain a free FORTRAN channel from the PME.
	/// </summary>
	/// <remarks>
	/// When a PMC is wrapping a FORTRAN dll, there may be a technical problem when the PMC
	/// is loaded in the same process as the PME such as Simulator Execution. In this case, there
	/// may be a clash between different FORTRAN modules if two of them select the same output 
	/// channel for FORTRAN messaging. Hence the PME should centralise the generation of
	/// unique output channels for each PMC that may require them. This requirement only occurs
	/// when PME and PMC belong to the same computing process, obviously this FORTRAN
	/// channel functionality is only applicable when the architecture is not distributed. As we can
	/// have in the future this kind of information to exchange, a generic and extensible mechanism
	/// has to be set up. The calling pattern is a good candidate. Thus a specific string value for
	/// FORTRAN channel would be standardised.
	/// </remarks>
	[System.Runtime.InteropServices.ComImport()]
	[System.Runtime.InteropServices.ComVisibleAttribute(false)]
	[System.Runtime.InteropServices.Guid(COGuids.ICapeCOSEUtilities_IID)]
	[System.ComponentModel.DescriptionAttribute("ICapeCOSEUtilities Interface")]
	public interface ICapeCOSEUtilities
	{
		/// <summary>
		/// The list of named values supported by the PME.
		/// </summary>
		/// <remarks>
		/// The list of NamedValues provided by the PME.
		/// </remarks>
		/// <value>
		/// A String Array list of named values supported by the COSE. Included in this list
		/// should be the FreeFORTRANchannel named value which will provide the name of free FORTRAN
		/// channel.
		/// </value>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		[
			System.Runtime.InteropServices.DispId(1),
			System.ComponentModel.DescriptionAttribute("property NamedValueList")
		] 
		Object NamedValueList
		{
			get; 
		}

		/// <summary>
		/// Returns a value corresponding to the request name, including a free FORTRAN channel.
		/// </summary>
		/// <remarks>
		/// <para>Returns the value corresponding to the value named name. Be aware that two consecutive calls passing the 
        /// same name may return different values.</para>
		/// <para>The COSE will return a different FORTRAN channel each time the FreeFORTRANchannel NamedValue is 
		/// called for this property. The COSE may not use any of the returned FORTRAN channels 
		/// for any internally used FORTRAN module.</para>
        /// </remarks>
		/// <returns>
		/// Name of the requested value (which must be included in the list returned by NamedValueList).
		/// </returns>
		/// <param name = "value">Name of the requested value (which must be included in the list returned by 
        /// <see cref = "ICapeCOSEUtilities.NamedValueList"/>.</param>
		/// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
		/// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [
            System.Runtime.InteropServices.DispId(2),
            System.ComponentModel.DescriptionAttribute("property NamedValue")
        ]
        Object NamedValue(String value);
 
        };

	//#endif //_COSE_IDL_
}