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


// ---- The scope of the Reactions interfaces -------------------
// Reference document: Chemical Reactions

// Types of reactions 
/*[
uuid(CapeReactionType_IID),
version(1.0)
]
typedef enum eCapeReactionType {
CAPE_EQUILIBRIUM 	= 0,
CAPE_KINETIC		= 1,
} CapeReactionType;
*/
namespace CapeOpen
{

    // interface ICapeReactionProperties

    /// <summary>
    /// Enumeration for the type of reaction.
    /// </summary>
    /// <remarks>
    /// Indicates whether the reaction package is for equilibrium or kinetic type reactions.
    /// </remarks>
    [
        System.Runtime.InteropServices.ComVisibleAttribute(true),
        System.Runtime.InteropServices.GuidAttribute("678c0b00-0100-11d2-a67d-00105a42887f")//CapeReactionType_IID),
    ]
    public enum CapeReactionType
    {
        /// <summary>
        /// Equilibrium reactions.
        /// </summary>
        CAPE_EQUILIBRIUM = 0,
        /// <summary>
        /// Kinetic reactions.
        /// </summary>
        CAPE_KINETIC = 1,
    };
    //typedef CapeReactionType eCapeReactionType;
    /*
    [
    uuid(CapeReactionRateBasis_IID),
    version(1.0)
    ]
    typedef enum eCapeReactionRateBasis {
    CAPE_HOMOGENEOUS 	= 0,
    CAPE_HETEROGENEOUS 	= 1,
    } CapeReactionRateBasis;
    */
    /// <summary>
    /// Enumeration for the rate basis for the reaction.
    /// </summary>
    /// <remarks>
    /// Indicates whether the reaction occurs in a homgeneous phase of is a heterogeneous reaction..
    /// </remarks>
    [
        System.Runtime.InteropServices.ComVisibleAttribute(true),
        System.Runtime.InteropServices.GuidAttribute("678c0aff-0100-11d2-a67d-00105a42887f")//CapeReactionRateBasis_IID),
    ]
    public enum CapeReactionRateBasis
    {
        /// <summary>
        /// Homogeneous reaction.
        /// </summary>
        CAPE_HOMOGENEOUS = 0,
        /// <summary>
        /// Heterogeneous reaction.
        /// </summary>
        CAPE_HETEROGENEOUS = 1,
    };
    // typedef CapeReactionRateBasis eCapeReactionRateBasis;

    // ICapeKineticReactionContext interface
    /// <summary>
    /// Provides access to the properties of a set of kinetic reactions.
    /// </summary>
    /// <remarks>
    /// <para>
    /// This interface allows a reaction object to be passed to a
    /// component that needs access to the properties of a set of kinetic reactions.
    /// </para>
    /// <para>
    /// This interface is used to set the reaction object upon which reaction calculations
    /// will take place. Calculated reaction properties will be stored in this reaction object.
    /// </para>
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeKineticReactionContext_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeKineticReactionContext Interface")]
    interface ICapeKineticReactionContextCOM
    {
        /// <summary>
        /// Provides access to the properties of a set of kinetic reactions.
        /// </summary>
        /// <remarks>
        /// <para>
        /// Used to pass the <see cref = "ICapeReactionProperties"/> interface of a reaction object to a 
        /// component that needs to access the properties of a set of kinetic reactions.
        /// </para>
        /// </remarks>
        /// <param name = "reactionsObject">The ICapeReactionProperties interface of a reaction object.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("SetMaterial")]
        void SetReactionObject(ref Object reactionsObject);
    };

    // ICapeKineticReactionContext interface
    /// <summary>
    /// Provides access to the properties of a set of kinetic reactions.
    /// </summary>
    /// <remarks>
    /// <para>
    /// This interface allows a reaction object to be passed to a
    /// component that needs access to the properties of a set of kinetic reactions.
    /// </para>
    /// <para>
    /// This interface is used to set the reaction object upon which reaction calculations
    /// will take place. Calculated reaction properties will be stored in this reaction object.
    /// </para>
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.ComponentModel.DescriptionAttribute("ICapeKineticReactionContext Interface")]
    public interface ICapeKineticReactionContext
    {
        /// <summary>
        /// Provides access to the properties of a set of kinetic reactions.
        /// </summary>
        /// <remarks>
        /// <para>
        /// Used to pass the <see cref = "ICapeReactionProperties"/> interface of a reaction object to a 
        /// component that needs to access the properties of a set of kinetic reactions.
        /// </para>
        /// </remarks>
        /// <param name = "reactionsObject">The ICapeReactionProperties interface of a reaction object.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("SetMaterial")]
        void SetReactionObject(ref ICapeReactionProperties reactionsObject);
    };

    /// <summary>
    /// Provides access to the properties of a set of electrolyte reactions.
    /// </summary>
    /// <remarks>
    /// <para>
    /// This interface allows a reaction object to be passed to a
    /// component that needs access to the properties of a set of equilibrium reactions.
    /// </para>
    /// <para>
    /// This interface is used to set the reaction object upon which reaction calculations
    /// will take place. Calculated reaction properties will be stored in this reaction object.
    /// </para>
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeElectrolyteReactionContext_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeElectrolyteReactionContext Interface")]
    interface ICapeElectrolyteReactionContextCOM
    {
        /// <summary>
        /// Provides access to the properties of a set of equilibrium reactions.
        /// </summary>
        /// <remarks>
        /// <para>
        /// Used to pass the <see cref = "ICapeReactionProperties"/> interface of a reaction object to a 
        /// component that needs to access the properties of a set of kinetic reactions.
        /// </para>
        /// </remarks>
        /// <param name = "reactionsObject">The ICapeReactionProperties interface of a reaction object.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("SetMaterial")]
        void SetReactionObject(ref Object reactionsObject);
    };


    /// <summary>
    /// Provides access to the properties of a set of electrolyte reactions.
    /// </summary>
    /// <remarks>
    /// <para>
    /// This interface allows a reaction object to be passed to a
    /// component that needs access to the properties of a set of equilibrium reactions.
    /// </para>
    /// <para>
    /// This interface is used to set the reaction object upon which reaction calculations
    /// will take place. Calculated reaction properties will be stored in this reaction object.
    /// </para>
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.ComponentModel.DescriptionAttribute("ICapeElectrolyteReactionContext Interface")]
    public interface ICapeElectrolyteReactionContext
    {
        /// <summary>
        /// Provides access to the properties of a set of equilibrium reactions.
        /// </summary>
        /// <remarks>
        /// <para>
        /// Used to pass the <see cref = "ICapeReactionProperties"/> interface of a reaction object to a 
        /// component that needs to access the properties of a set of kinetic reactions.
        /// </para>
        /// </remarks>
        /// <param name = "reactionsObject">The ICapeReactionProperties interface of a reaction object.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("SetMaterial")]
        void SetReactionObject(ref ICapeReactionProperties reactionsObject);
    };
    
    /// <summary>
    /// Similar in scope to the <see cref = "ICapeThermoSystem"/>. These interfaces will be implemented by a 
    /// Reactions Package Manager component.
    /// </summary>
    /// <remarks>
    /// Provides a list of all supported reaction packages and resolves the selected package.
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeReactionsPackageManager_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeReactionsPackageManager Interface")]
    interface ICapeReactionsPackageManagerCOM
    {
        /// <summary>
        /// A list of all available reaction packages.
        /// </summary>
        /// <remarks>
        /// Returns a list of the names of all Reactions Packages available within the Reactions Package Manager..
        /// </remarks>
        /// <returns>Returns a list of all available reaction packages.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("method GetListOfReactionsPackages")]
        Object GetListOfReactionsPackages();


        /// <summary>
        /// Resolves a reaction routine.
        /// </summary>
        /// <remarks>
        /// <para>
        /// Returns the Reactions Package specified by the client of the Reactions Package Manager.
        /// </para>
        /// </remarks>
        /// <param name = "reactionsPkg">The name of the reactions routine to be resolved.</param>
        /// <returns>Returns the Reactions Package specified by the client of the Reactions Package Manager.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("method ResolveReactionsPackage")]
        Object ResolveReactionsPackage(String reactionsPkg);

    };


    /// <summary>
    /// Similar in scope to the <see cref = "ICapeThermoSystem"/>. These interfaces will be implemented by a 
    /// Reactions Package Manager component.
    /// </summary>
    /// <remarks>
    /// Provides a list of all supported reaction packages and resolves the selected package.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.ComponentModel.DescriptionAttribute("ICapeReactionsPackageManager Interface")]
    public interface ICapeReactionsPackageManager
    {
        /// <summary>
        /// A list of all available reaction packages.
        /// </summary>
        /// <remarks>
        /// Returns a list of the names of all Reactions Packages available within the Reactions Package Manager..
        /// </remarks>
        /// <returns>Returns a list of all available reaction packages.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("method GetListOfReactionsPackages")]
        string[] GetListOfReactionsPackages();


        /// <summary>
        /// Resolves a reaction routine.
        /// </summary>
        /// <remarks>
        /// <para>
        /// Returns the Reactions Package specified by the client of the Reactions Package Manager.
        /// </para>
        /// </remarks>
        /// <param name = "reactionsPkg">The name of the reactions routine to be resolved.</param>
        /// <returns>Returns the Reactions Package specified by the client of the Reactions Package Manager.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("method ResolveReactionsPackage")]
        ICapeReactionChemistry ResolveReactionsPackage(String reactionsPkg);
    };

    /// <summary>
    /// Provides information about the reactions in the reaction package.
    /// </summary>
    /// <remarks>
    /// A component or a PME that needs to describe a set of reactions will implement this 
    /// interface. A set of reactions is described in terms of the compounds that take part
    /// in the reactions and the compounds that are produced. For example, in the case of 
    /// electrolyte	systems, salt complexes and ions. In the case of detailed reaction mechanisms, 
    /// radicals. 
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeReactionChemistry_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeChemistry Interface")]
    interface ICapeReactionChemistryCOM
    {
        /// <summary>
        /// Number of reactions contained within this reaction package.
        /// </summary>
        /// <remarks>
        /// Returns the number of reactions contained in this reactions package.
        /// </remarks>
        /// <returns>Returns the number of reactions contained in this reactions package.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("method GetNumberOfReactions")]
        int GetNumberOfReactions();

        /// <summary>
        /// The string identifiers of the reactions contained within this reaction package.
        /// </summary>
        /// <remarks>
        /// Returns the identifiers of all the reactions contained within the Reactions Package.
        /// </remarks>
        /// <returns>Returns the string identifiers for each one of the reactions.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionsIds")]
        Object GetReactionsIds();

        /// <summary>
        /// The <see cref = "CapeReactionType"/> of the reaction.
        /// </summary>
        /// <remarks>
        /// Returns the <see cref = "CapeReactionType"/> of a particular reaction. Only needed for non-electrolyte
        /// reactions. It informs whether the reaction is an equilibrium or kinetic
        /// reaction
        /// </remarks>
        /// <param name = "reacID">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <returns>Returns the <see cref = "CapeReactionType"/> type of a particular reaction.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionType")]
        CapeReactionType GetReactionType(String reacID);

        /// <summary>
        /// The number of compounds in the specified reaction.
        /// </summary>
        /// <remarks>
        /// Gets the number of compounds occurring in a particular reaction within a Reactions Package.
        /// </remarks>
        /// <param name = "reacID">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <returns>Returns the number of compounds participating in the specified reaction.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(4)]
        [System.ComponentModel.DescriptionAttribute("method GetNumberOfReactionCompounds")]
        int GetNumberOfReactionCompounds(String reacID);

        // returns the compIds, formulas, CAS numbers and compNames of all
        // compounds participating in the specified reaction

        /// <summary>
        /// Get the identifiers of the components participating in the specified reaction 
        /// within the reaction set defined in the Reactions Package.
        /// </summary>
        /// <remarks>
        /// This method returns both compound name and CAS registry number. The CAS Registry 
        /// number should be used to identify the compounds for validation purposes because 
        /// it is unambiguous.
        /// </remarks>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <param name = "compIds">List of compound IDs.</param>
        /// <param name = "compCharge">The charge for each compound.</param>
        /// <param name = "compCASNumber">The CAS Registry numbers for the compounds.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(5)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionCompoundIds")]
        void GetReactionCompoundIds(String reacId,
            ref System.Object compIds,
            ref System.Object compCharge,
            ref System.Object compCASNumber);

        // returns the stoichiometry of the specified reaction. Stoichiometric coefficients
        // are ordered consistently with the list of compounds returned by GetReactionsComponentsIDS
        // for the same reaction

        /// <summary>
        /// Get the stoichiometry of the specified reaction.
        /// </summary>
        /// <remarks>
        /// Returns the stoichiometric coefficients of the specified reaction (positive 
        /// numbers indicate products, negative numbers indicate reactants). Stoichiometric 
        /// coefficients are ordered consistently with the list of compounds returned by 
        /// the <see cref = "GetReactionCompoundIds"/> method for the same reaction.
        /// </remarks>
        /// <returns>The stoichiometry of the specified reaction.</returns>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(6)]
        [System.ComponentModel.DescriptionAttribute("method GetStoichiometricCoefficients")]
        System.Object GetStoichiometricCoefficients(String reacId);

        /// <summary>
        /// Gets the phase on which a particular reaction contained in the Reactions Package will take place.
        /// </summary>
        /// <remarks>
        /// The string returned by this method must match one of the phase labels known to the Property Package.
        /// </remarks>
        /// <returns>The phase label of the phase where the reaction tackes place.</returns>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(7)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionPhase")]
        String GetReactionPhase(String reacId);

        /// <summary>
        /// Get the basis for the reaction rate will be expressed in (i.e. homogeneous
        /// or heterogeneous).
        /// </summary>
        /// <remarks>
        /// Gets the phase on which the reactions contained in the package will take place. The 
        /// reaction rate basis (i.e. “Homogeneous” or “Heterogeneous”). Homogeneous reactions 
        /// will be provided in kgmole/h/m3 and heterogeneous will be provided in 
        /// kgmole/h/kg-cat.
        /// </remarks>
        /// <returns>A <see cref = "CapeReactionRateBasis"/> for the rate basis.</returns>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(8)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionRateBasis")]
        CapeReactionRateBasis GetReactionRateBasis(String reacId);

        /// <summary>
        /// Get the concentration basis the reaction package will use to calculate the
        /// specified reaction rate.
        /// </summary>
        /// <remarks>
        /// <para>
        /// Gets the concentration basis required that will be used by a particular reaction in 
        /// its rate equation.
        /// </para>
        /// <para>
        /// Qualifiers defined in the THRM spec can be used here (i.e. “fugacity”, 
        /// “moleFraction”, etc).
        /// </para>
        /// </remarks>
        /// <returns>The concentration basis the reaction package will use to calculate the
        /// specified reaction rate.</returns>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(9)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionConcBasis")]
        String GetReactionConcBasis(String reacId);

        // returns the base reactant for the specified reaction

        /// <summary>
        /// Get the base reactant for the specified reaction.
        /// </summary>
        /// <remarks>
        /// Returns the name of the base reactant for a particular reaction..
        /// </remarks>
        /// <returns>The name of the base reactant for a particular reaction.</returns>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(10)]
        [System.ComponentModel.DescriptionAttribute("method GetBaseReactant")]
        String GetBaseReactant(String reacId);

        /// <summary>
        /// Returns the number and ids of the compounds in the specified phase.
        /// </summary>
        /// <remarks>
        /// Returns the number and ids of the compounds in the specified phase.
        /// </remarks>
        /// <returns>The name of the base reactant for a particular reaction.</returns>
        /// <param name = "reacID">Label of the required phase.</param>
        /// <param name = "compNo">The number of compounds in the requested phase.</param>
        /// <param name = "compIds">The ids of the compounds present in the specified phase.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(11)]
        [System.ComponentModel.DescriptionAttribute("method GetPhaseCompounds")]
        void GetPhaseCompounds(String reacID, ref int compNo, ref Object compIds);

        /// <summary>
        /// Returns a collection containing the rate expression parameters for a particular reaction.
        /// </summary>
        /// <remarks>
        /// <para>
        /// GetReactionParameters returns a collection of CAPE-OPEN parameters [6] that 
        /// characterize the rate expression used by the reaction model in a Reaction Package. 
        /// For a PowerLaw model this collection would contain parameters for activation energy, 
        /// pre-exponential factor and compound exponents for example. It is up to the Reactions 
        /// Package implementor to decide whether a client can update the values of these 
        /// parameters. If this operation is allowed, then the implementor must also provide 
        /// support for persistence [5] interfaces, so that the updated values can be saved and 
        /// restored. In this case the COSE is also responsible for calling the persistence
        /// methods.
        /// </para>
        /// <para>
        /// Deliberately, the standard does not define the names of the parameters that may 
        /// appear in such a collection, even for well-known reaction models, such as PowerLaw 
        /// and Langmuir – Hinshelwood – Hougen – Watson (LHHW). This is because the formulation 
        /// of well-known models is not fixed, and because the standard needs to support custom 
        /// models as well as the well-known models. 
        /// </para>
        /// <para>
        /// This decision is not expected to be restrictive: in most cases the (software) client 
        /// of a Reactions Package does not need to know what model the package implements and 
        /// what parameters it has. However, the parameters may be of interest to an end-user who 
        /// wants to adjust or estimate the parameter values. In these cases the COSE can invoke 
        /// the Reaction Package’s own GUI, or, if it doesn’t have one, present the parameters in 
        /// a generic grid. It is the Reaction Package implementor’s responsibility to provide 
        /// documentation for the parameters so that an enduser can understand how they are used.
        /// </para>
        /// </remarks>
        /// <returns>A collection containing the rate expression parameters for a particular reaction.</returns>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(12)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionParameters")]
        Object GetReactionParameters(String reacId);
    };


    /// <summary>
    /// Provides information about the reactions in the reaction package.
    /// </summary>
    /// <remarks>
    /// A component or a PME that needs to describe a set of reactions will implement this 
    /// interface. A set of reactions is described in terms of the compounds that take part
    /// in the reactions and the compounds that are produced. For example, in the case of 
    /// electrolyte	systems, salt complexes and ions. In the case of detailed reaction mechanisms, 
    /// radicals. 
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.ComponentModel.DescriptionAttribute("ICapeChemistry Interface")]
    public interface ICapeReactionChemistry
    {
        /// <summary>
        /// Number of reactions contained within this reaction package.
        /// </summary>
        /// <remarks>
        /// Returns the number of reactions contained in this reactions package.
        /// </remarks>
        /// <returns>Returns the number of reactions contained in this reactions package.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("method GetNumberOfReactions")]
        int GetNumberOfReactions();

        /// <summary>
        /// The string identifiers of the reactions contained within this reaction package.
        /// </summary>
        /// <remarks>
        /// Returns the identifiers of all the reactions contained within the Reactions Package.
        /// </remarks>
        /// <returns>Returns the string identifiers for each one of the reactions.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionsIds")]
        string[] GetReactionsIds();

        /// <summary>
        /// The <see cref = "CapeReactionType"/> of the reaction.
        /// </summary>
        /// <remarks>
        /// Returns the <see cref = "CapeReactionType"/> of a particular reaction. Only needed for non-electrolyte
        /// reactions. It informs whether the reaction is an equilibrium or kinetic
        /// reaction
        /// </remarks>
        /// <param name = "reacID">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <returns>Returns the <see cref = "CapeReactionType"/> type of a particular reaction.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionType")]
        CapeReactionType GetReactionType(String reacID);

        /// <summary>
        /// The number of compounds in the specified reaction.
        /// </summary>
        /// <remarks>
        /// Gets the number of compounds occurring in a particular reaction within a Reactions Package.
        /// </remarks>
        /// <param name = "reacID">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <returns>Returns the number of compounds participating in the specified reaction.</returns>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(4)]
        [System.ComponentModel.DescriptionAttribute("method GetNumberOfReactionCompounds")]
        int GetNumberOfReactionCompounds(String reacID);

        // returns the compIds, formulas, CAS numbers and compNames of all
        // compounds participating in the specified reaction

        /// <summary>
        /// Get the identifiers of the components participating in the specified reaction 
        /// within the reaction set defined in the Reactions Package.
        /// </summary>
        /// <remarks>
        /// This method returns both compound name and CAS registry number. The CAS Registry 
        /// number should be used to identify the compounds for validation purposes because 
        /// it is unambiguous.
        /// </remarks>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <param name = "compIds">List of compound IDs.</param>
        /// <param name = "compCharge">The charge for each compound.</param>
        /// <param name = "compCASNumber">The CAS Registry numbers for the compounds.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(5)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionCompoundIds")]
        void GetReactionCompoundIds(String reacId,
            ref string[] compIds,
            ref double[] compCharge,
            ref string[] compCASNumber);

        // returns the stoichiometry of the specified reaction. Stoichiometric coefficients
        // are ordered consistently with the list of compounds returned by GetReactionsComponentsIDS
        // for the same reaction

        /// <summary>
        /// Get the stoichiometry of the specified reaction.
        /// </summary>
        /// <remarks>
        /// Returns the stoichiometric coefficients of the specified reaction (positive 
        /// numbers indicate products, negative numbers indicate reactants). Stoichiometric 
        /// coefficients are ordered consistently with the list of compounds returned by 
        /// the <see cref = "GetReactionCompoundIds"/> method for the same reaction.
        /// </remarks>
        /// <returns>The stoichiometry of the specified reaction.</returns>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(6)]
        [System.ComponentModel.DescriptionAttribute("method GetStoichiometricCoefficients")]
        double[] GetStoichiometricCoefficients(String reacId);

        /// <summary>
        /// Gets the phase on which a particular reaction contained in the Reactions Package will take place.
        /// </summary>
        /// <remarks>
        /// The string returned by this method must match one of the phase labels known to the Property Package.
        /// </remarks>
        /// <returns>The phase label of the phase where the reaction tackes place.</returns>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(7)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionPhase")]
        String GetReactionPhase(String reacId);

        /// <summary>
        /// Get the basis for the reaction rate will be expressed in (i.e. homogeneous
        /// or heterogeneous).
        /// </summary>
        /// <remarks>
        /// Gets the phase on which the reactions contained in the package will take place. The 
        /// reaction rate basis (i.e. “Homogeneous” or “Heterogeneous”). Homogeneous reactions 
        /// will be provided in kgmole/h/m3 and heterogeneous will be provided in 
        /// kgmole/h/kg-cat.
        /// </remarks>
        /// <returns>A <see cref = "CapeReactionRateBasis"/> for the rate basis.</returns>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(8)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionRateBasis")]
        CapeReactionRateBasis GetReactionRateBasis(String reacId);

        /// <summary>
        /// Get the concentration basis the reaction package will use to calculate the
        /// specified reaction rate.
        /// </summary>
        /// <remarks>
        /// <para>
        /// Gets the concentration basis required that will be used by a particular reaction in 
        /// its rate equation.
        /// </para>
        /// <para>
        /// Qualifiers defined in the THRM spec can be used here (i.e. “fugacity”, 
        /// “moleFraction”, etc).
        /// </para>
        /// </remarks>
        /// <returns>The concentration basis the reaction package will use to calculate the
        /// specified reaction rate.</returns>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(9)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionConcBasis")]
        String GetReactionConcBasis(String reacId);

        // returns the base reactant for the specified reaction

        /// <summary>
        /// Get the base reactant for the specified reaction.
        /// </summary>
        /// <remarks>
        /// Returns the name of the base reactant for a particular reaction..
        /// </remarks>
        /// <returns>The name of the base reactant for a particular reaction.</returns>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(10)]
        [System.ComponentModel.DescriptionAttribute("method GetBaseReactant")]
        String GetBaseReactant(String reacId);

        /// <summary>
        /// Returns the number and ids of the compounds in the specified phase.
        /// </summary>
        /// <remarks>
        /// Returns the number and ids of the compounds in the specified phase.
        /// </remarks>
        /// <returns>The name of the base reactant for a particular reaction.</returns>
        /// <param name = "reacID">Label of the required phase.</param>
        /// <param name = "compNo">The number of compounds in the requested phase.</param>
        /// <param name = "compIds">The ids of the compounds present in the specified phase.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(11)]
        [System.ComponentModel.DescriptionAttribute("method GetPhaseCompounds")]
        void GetPhaseCompounds(String reacID, ref int compNo, ref string[] compIds);

        /// <summary>
        /// Returns a collection containing the rate expression parameters for a particular reaction.
        /// </summary>
        /// <remarks>
        /// <para>
        /// GetReactionParameters returns a collection of CAPE-OPEN parameters [6] that 
        /// characterize the rate expression used by the reaction model in a Reaction Package. 
        /// For a PowerLaw model this collection would contain parameters for activation energy, 
        /// pre-exponential factor and compound exponents for example. It is up to the Reactions 
        /// Package implementor to decide whether a client can update the values of these 
        /// parameters. If this operation is allowed, then the implementor must also provide 
        /// support for persistence [5] interfaces, so that the updated values can be saved and 
        /// restored. In this case the COSE is also responsible for calling the persistence
        /// methods.
        /// </para>
        /// <para>
        /// Deliberately, the standard does not define the names of the parameters that may 
        /// appear in such a collection, even for well-known reaction models, such as PowerLaw 
        /// and Langmuir – Hinshelwood – Hougen – Watson (LHHW). This is because the formulation 
        /// of well-known models is not fixed, and because the standard needs to support custom 
        /// models as well as the well-known models. 
        /// </para>
        /// <para>
        /// This decision is not expected to be restrictive: in most cases the (software) client 
        /// of a Reactions Package does not need to know what model the package implements and 
        /// what parameters it has. However, the parameters may be of interest to an end-user who 
        /// wants to adjust or estimate the parameter values. In these cases the COSE can invoke 
        /// the Reaction Package’s own GUI, or, if it doesn’t have one, present the parameters in 
        /// a generic grid. It is the Reaction Package implementor’s responsibility to provide 
        /// documentation for the parameters so that an enduser can understand how they are used.
        /// </para>
        /// </remarks>
        /// <returns>A collection containing the rate expression parameters for a particular reaction.</returns>
        /// <param name = "reacId">The name of the reaction obtained from the <see cref = "GetReactionsIds"/> method.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(12)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionParameters")]
        ParameterCollection GetReactionParameters(String reacId);
    };

    /// <summary>
    /// Provides access to the properties of a particular reaction.
    /// </summary>
    /// <remarks>
    /// Similar in scope to ICapeThermoMaterialObject. A component or a PME that needs to 
    /// provide access to the properties of a particular reaction will implement this interface. 
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeReactionProperties_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeReactionProperties Interface")]
    interface ICapeReactionPropertiesCOM
    {
        /// <summary>
        /// Gets the value of the specified reaction property within a reactions object.
        /// </summary>
        /// <remarks>
        /// The qualifiers passed in determine the reactions, phase and calculation basis for 
        /// which the property will be got. The order of the array is the same as in the passed
        /// in reacIds array (i.e. property value for reaction reacIds[1] will be stored in 
        /// property[1]).
        /// </remarks>
        /// <returns>The name of the base reactant for a particular reaction.</returns>
        /// <param name = "property">The Reaction Property to be retrieved.</param>
        /// <param name = "phase">The qualified phase for the Reaction Property.</param>
        /// <param name = "reacIds">The qualified reactions for the Reaction Property. NULL to
        /// specify all reactions in the set.</param>
        /// <param name = "basis"><para>Qualifies the basis of the Reaction Property (i.e., mass 
        /// /mole). Default is mole. Use NULL only as a placeholder for property for which basis 
        /// does not apply.</para>
        /// <para> This qualifier could be extended with values such as activity, fugacity, 
        /// fractions, molality…This way when an equilibrium constant is requested its basis can 
        /// be specified.</para>
        /// </param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionProp")]
        Object GetReactionProp(String property, String phase, Object reacIds, String basis);

        /// <summary>
        /// Sets the values of the specified reaction property within a reactions object.
        /// </summary>
        /// <remarks>
        /// The qualifiers passed in determine the reactions, phase and calculation basis for 
        /// which the property will be retrieved.
        /// </remarks>
        /// <returns>The name of the base reactant for a particular reaction.</returns>
        /// <param name = "property">The Reaction Property to be retrieved.</param>
        /// <param name = "phase">The qualified phase for the Reaction Property.</param>
        /// <param name = "reacIds">The qualified reactions for the Reaction Property. NULL to
        /// specify all reactions in the set.</param>
        /// <param name = "basis"><para>Qualifies the basis of the Reaction Property (i.e., mass 
        /// /mole). Default is mole. Use NULL only as a placeholder for property for which basis 
        /// does not apply.</para>
        /// <para> This qualifier could be extended with values such as activity, fugacity, 
        /// fractions, molality…This way when an equilibrium constant is requested its basis can 
        /// be specified.</para>
        /// </param>
        /// <param name = "propVals">The values of the requested reaction property. The order of 
        /// the array is the same as in the passed in reacIds array (i.e. property value for 
        /// reaction reacIds[1] will be stored in property[1]).</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("method SetReactionProp")]
        void SetReactionProp(String property, String phase, Object reacIds, String basis, Object propVals);
    };

    /// <summary>
    /// Provides access to the properties of a particular reaction.
    /// </summary>
    /// <remarks>
    /// Similar in scope to ICapeThermoMaterialObject. A component or a PME that needs to 
    /// provide access to the properties of a particular reaction will implement this interface. 
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.ComponentModel.DescriptionAttribute("ICapeReactionProperties Interface")]
    public interface ICapeReactionProperties
    {
        /// <summary>
        /// Gets the value of the specified reaction property within a reactions object.
        /// </summary>
        /// <remarks>
        /// The qualifiers passed in determine the reactions, phase and calculation basis for 
        /// which the property will be got. The order of the array is the same as in the passed
        /// in reacIds array (i.e. property value for reaction reacIds[1] will be stored in 
        /// property[1]).
        /// </remarks>
        /// <returns>The name of the base reactant for a particular reaction.</returns>
        /// <param name = "property">The Reaction Property to be retrieved.</param>
        /// <param name = "phase">The qualified phase for the Reaction Property.</param>
        /// <param name = "reacIds">The qualified reactions for the Reaction Property. NULL to
        /// specify all reactions in the set.</param>
        /// <param name = "basis"><para>Qualifies the basis of the Reaction Property (i.e., mass 
        /// /mole). Default is mole. Use NULL only as a placeholder for property for which basis 
        /// does not apply.</para>
        /// <para> This qualifier could be extended with values such as activity, fugacity, 
        /// fractions, molality…This way when an equilibrium constant is requested its basis can 
        /// be specified.</para>
        /// </param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("method GetReactionProp")]
        double[] GetReactionProp(String property, String phase, string[] reacIds, String basis);

        /// <summary>
        /// Sets the values of the specified reaction property within a reactions object.
        /// </summary>
        /// <remarks>
        /// The qualifiers passed in determine the reactions, phase and calculation basis for 
        /// which the property will be retrieved.
        /// </remarks>
        /// <returns>The name of the base reactant for a particular reaction.</returns>
        /// <param name = "property">The Reaction Property to be retrieved.</param>
        /// <param name = "phase">The qualified phase for the Reaction Property.</param>
        /// <param name = "reacIds">The qualified reactions for the Reaction Property. NULL to
        /// specify all reactions in the set.</param>
        /// <param name = "basis"><para>Qualifies the basis of the Reaction Property (i.e., mass 
        /// /mole). Default is mole. Use NULL only as a placeholder for property for which basis 
        /// does not apply.</para>
        /// <para> This qualifier could be extended with values such as activity, fugacity, 
        /// fractions, molality…This way when an equilibrium constant is requested its basis can 
        /// be specified.</para>
        /// </param>
        /// <param name = "propVals">The values of the requested reaction property. The order of 
        /// the array is the same as in the passed in reacIds array (i.e. property value for 
        /// reaction reacIds[1] will be stored in property[1]).</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("method SetReactionProp")]
        void SetReactionProp(String property, String phase, string[] reacIds, String basis, double[] propVals);
    };

    /// <summary>
    /// Provides a material object for physical property calculations.
    /// </summary>
    /// <remarks>
    /// Allows a material object to be passed between a PME and the Reactions components it is 
    /// using so that the Reactions components can make Physical Property calculation calls. 
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.ComponentModel.DescriptionAttribute("ICapeThermoContext Interface")]
    public interface ICapeThermoContext
    {
        /// <summary>
        /// Allows the client of a component that implements this interface to pass an 
        /// <see cref = "ICapeThermoMaterialObject"/> interface to the component, so that 
        /// it can access the properties of a material and request property calculations.
        /// </summary>
        /// <remarks>
        /// The SetMaterial method allows a Reactions component to be given the 
        /// <see cref = "ICapeThermoMaterialObject"/> interface of a Material Object. 
        /// This interface gives the component access to the description of the material for 
        /// which Property calculations are required. A component can also use the 
        /// <see cref = "ICapeThermoMaterialObject"/> interface to to get lists of components 
        /// and phases.
        /// </remarks>
        /// <param name = "materialObject">The interface of an object support the 
        /// <see cref = "ICapeThermoMaterialObject"/> interface.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("SetMaterial")]
        void SetMaterial(ICapeThermoMaterial materialObject);

        /// <summary>
        /// Allows the client of a component that implements this interface to pass an 
        /// <see cref = "ICapeThermoMaterialObject"/> interface to the component, so that 
        /// it can access the properties of a material and request property calculations.
        /// </summary>
        /// <remarks>
        /// The SetMaterial method allows a Reactions component to be given the 
        /// <see cref = "ICapeThermoMaterialObject"/> interface of a Material Object. 
        /// This interface gives the component access to the description of the material for 
        /// which Property calculations are required. A component can also use the 
        /// <see cref = "ICapeThermoMaterialObject"/> interface to to get lists of components 
        /// and phases.
        /// </remarks>
        /// <param name = "materialObject">The interface of an object support the 
        /// <see cref = "ICapeThermoMaterialObject"/> interface.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("SetMaterial")]
        void SetMaterial(ICapeThermoMaterialObject materialObject);
    };

    /// <summary>
    /// Provides a material object for physical property calculations.
    /// </summary>
    /// <remarks>
    /// Allows a material object to be passed between a PME and the Reactions components it is 
    /// using so that the Reactions components can make Physical Property calculation calls. 
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeThermoContext_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeThermoContext Interface")]
    interface ICapeThermoContextCOM
    {
        /// <summary>
        /// Allows the client of a component that implements this interface to pass an 
        /// <see cref = "ICapeThermoMaterialObject"/> interface to the component, so that 
        /// it can access the properties of a material and request property calculations.
        /// </summary>
        /// <remarks>
        /// The SetMaterial method allows a Reactions component to be given the 
        /// <see cref = "ICapeThermoMaterialObject"/> interface of a Material Object. 
        /// This interface gives the component access to the description of the material for 
        /// which Property calculations are required. A component can also use the 
        /// <see cref = "ICapeThermoMaterialObject"/> interface to to get lists of components 
        /// and phases.
        /// </remarks>
        /// <param name = "materialObject">The interface of an object support the 
        /// <see cref = "ICapeThermoMaterialObject"/> interface.</param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("SetMaterial")]
        void SetMaterial(Object materialObject);
    };
    
    /// <summary>
    /// Calculates the values of reaction (or reaction related) properties.
    /// </summary>
    /// <remarks>
    /// Similar in scope to ICapeThermoPropertyPackage. A software component or a PME that can 
    /// calculate values of reaction (or reaction related) properties will implement this 
    /// interface. It may also be implemented by a Physical Property package component
    /// that deals with electrolytes.
    /// </remarks>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.Guid(COGuids.ICapeReactionsRoutine_IID)]
    [System.ComponentModel.DescriptionAttribute("ICapeReactionsRoutine Interface")]
    interface ICapeReactionsRoutineCOM
    {

        /// <summary>
        /// Sets the values of the specified reaction property within a reactions object.
        /// </summary>
        /// <remarks>
        /// <para>
        /// The Reactions Package is passed a list of reaction properties to be calculated, the 
        /// reaction IDS for which the properties are required, and the calculation basis for the 
        /// reaction properties (i.e. mole or mass). A material object containing the 
        /// thermodynamic state variables that need to be used for calculating the reaction 
        /// properties (e.g. T, P and compositions) is passed separately via a call to the 
        /// setMaterial method of the Reaction Package’s <see cref = "ICapeThermoContext"/> interface.
        /// </para>
        /// <para>
        /// The results of the calculation will be written to the reaction object passed to the 
        /// Reactions Package via either the <see cref = "ICapeKineticReactionContext"/> interface for a 
        /// kinetic reaction package, or the <see cref = "ICapeElectrolyteReactionContext"/> interface for an 
        /// Electrolyte Property Package.
        /// </para>
        /// </remarks>
        /// <returns>The name of the base reactant for a particular reaction.</returns>
        /// <param name = "props">The Reaction Property to be calculated.</param>
        /// <param name = "phase">The qualified phase for the results.</param>
        /// <param name = "reacIds">The qualified reactions for the Reaction Property. NULL to
        /// specify all reactions in the set.</param>
        /// <param name = "basis">Qualifies the basis of the Reaction Property (i.e., mass 
        /// /mole). Default is mole. Use NULL only as a placeholder for property for which basis 
        /// does not apply.
        /// </param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("method CalcReactionProp")]
        void CalcReactionProp(Object props, String phase, Object reacIds, String basis);
    };


    /// <summary>
    /// Calculates the values of reaction (or reaction related) properties.
    /// </summary>
    /// <remarks>
    /// Similar in scope to ICapeThermoPropertyPackage. A software component or a PME that can 
    /// calculate values of reaction (or reaction related) properties will implement this 
    /// interface. It may also be implemented by a Physical Property package component
    /// that deals with electrolytes.
    /// </remarks>
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.ComponentModel.DescriptionAttribute("ICapeReactionsRoutine Interface")]
    public interface ICapeReactionsRoutine
    {

        /// <summary>
        /// Sets the values of the specified reaction property within a reactions object.
        /// </summary>
        /// <remarks>
        /// <para>
        /// The Reactions Package is passed a list of reaction properties to be calculated, the 
        /// reaction IDS for which the properties are required, and the calculation basis for the 
        /// reaction properties (i.e. mole or mass). A material object containing the 
        /// thermodynamic state variables that need to be used for calculating the reaction 
        /// properties (e.g. T, P and compositions) is passed separately via a call to the 
        /// setMaterial method of the Reaction Package’s <see cref = "ICapeThermoContext"/> interface.
        /// </para>
        /// <para>
        /// The results of the calculation will be written to the reaction object passed to the 
        /// Reactions Package via either the <see cref = "ICapeKineticReactionContext"/> interface for a 
        /// kinetic reaction package, or the <see cref = "ICapeElectrolyteReactionContext"/> interface for an 
        /// Electrolyte Property Package.
        /// </para>
        /// </remarks>
        /// <returns>The name of the base reactant for a particular reaction.</returns>
        /// <param name = "props">The Reaction Property to be calculated.</param>
        /// <param name = "phase">The qualified phase for the results.</param>
        /// <param name = "reacIds">The qualified reactions for the Reaction Property. NULL to
        /// specify all reactions in the set.</param>
        /// <param name = "basis">Qualifies the basis of the Reaction Property (i.e., mass 
        /// /mole). Default is mole. Use NULL only as a placeholder for property for which basis 
        /// does not apply.
        /// </param>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeInvalidArgument">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeFailedInitialisation">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref ="ECapeNoImpl">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("method CalcReactionProp")]
        void CalcReactionProp(string[] props, String phase, string[] reacIds, String basis);
    };
};



