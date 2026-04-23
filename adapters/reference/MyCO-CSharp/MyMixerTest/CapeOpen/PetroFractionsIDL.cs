using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CapeOpen
{

    /* IMPORTANT NOTICE
    (c) The CAPE-OPEN Laboratory Network, 2007.
    All rights are reserved unless specifically stated otherwise

    Visit the web site at www.colan.org
    */

    // This file was developed/modified by Ignasi Palou-Rivera for CO-LaN organisation - November 2007


    // ---- The scope of Petroleum Fractions
    // Reference document: Petroleum Fractions

    /**************************************
    Petroleum Fractions interfaces
    **************************************/



    /// <summary>
    /// ICapePetroFractions interface
    /// Provides methods to identify a CAPE-OPEN component.
    /// </summary>
    [System.Runtime.InteropServices.ComImport()]
    [System.Runtime.InteropServices.ComVisibleAttribute(false)]
    [System.Runtime.InteropServices.GuidAttribute("72A94DE9-9A69-4369-B508-C033CDFD4F81")]
    [System.ComponentModel.DescriptionAttribute("ICapePetroFractions Interface")]
    public interface ICapePetroFractions
    {
        /// <summary>
        /// Sets bulk characterization properties for the complete set of petroleum fractions
        /// </summary>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        // CAPE-OPEN exceptions
        // ECapeUnknown, ECapeInvalidArgument
        [System.Runtime.InteropServices.DispIdAttribute(1)]
        [System.ComponentModel.DescriptionAttribute("method SetPetroBulkProp")]
        void SetPetroBulkProp([System.Runtime.InteropServices.InAttribute] String propertyID,
            [System.Runtime.InteropServices.InAttribute] String basis,
            [System.Runtime.InteropServices.InAttribute] double value);

        /// <summary>
        /// Sets characterization properties for individual petroleum fractions
        /// </summary>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(2)]
        [System.ComponentModel.DescriptionAttribute("method SetPetroCompoundProp")]
        void SetPetroCompoundProp([System.Runtime.InteropServices.InAttribute] String propertyID,
            [System.Runtime.InteropServices.InAttribute] Object compID,
            [System.Runtime.InteropServices.InAttribute] String basis,
            [System.Runtime.InteropServices.InAttribute] Object values);

        /// <summary>
        /// Sets characterization property cruves for the complete set of petroleum fractions
        /// </summary>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(3)]
        [System.ComponentModel.DescriptionAttribute("method SetPetroCurveProp")]
        void SetPetroCurveProp([System.Runtime.InteropServices.InAttribute] String propertyID,
            [System.Runtime.InteropServices.InAttribute] String basis,
            [System.Runtime.InteropServices.InAttribute] Object Xvalues,
            [System.Runtime.InteropServices.InAttribute] Object Yvalues);

        /// <summary>
        /// Gets bulk characterization properties for the complete set of petroleum fractions
        /// </summary>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(4)]
        [System.ComponentModel.DescriptionAttribute("method GetPetroBulkProp")]
        double GetPetroBulkProp([System.Runtime.InteropServices.InAttribute] String propertyID,
            [System.Runtime.InteropServices.InAttribute] String basis);

        /// <summary>
        /// Gets characterization properties for individual petroleum fractions
        /// </summary>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(5)]
        [System.ComponentModel.DescriptionAttribute("method GetPetroCompoundProp")]
        Object GetPetroCompoundProp([System.Runtime.InteropServices.InAttribute] String propertyID,
            [System.Runtime.InteropServices.InAttribute] Object compID,
            [System.Runtime.InteropServices.InAttribute] String basis);

        /// <summary>
        /// Gets characterization property cruves for the complete set of petroleum fractions
        /// </summary>
        /// <exception cref ="ECapeUnknown">The error to be raised when other error(s),  specified for this operation, are not suitable.</exception>
        /// <exception cref = "ECapeInvalidArgument">To be used when an invalid argument value is passed, for example, an unrecognised Compound identifier or UNDEFINED for the props argument.</exception>
        [System.Runtime.InteropServices.DispIdAttribute(6)]
        [System.ComponentModel.DescriptionAttribute("method GetPetroCurveProp")]
        Object GetPetroCurveProp([System.Runtime.InteropServices.InAttribute] String propertyID,
            [System.Runtime.InteropServices.InAttribute] String basis);

    };

    // Validation status values 
    /*[
    uuid(CapeCompoundType_IID),
    version(1.0)
    ]

    typedef enum eCapeCompoundType{
    CAPE_COMPOUND_REAL,
    CAPE_COMPOUND_ION,
    CAPE_COMPOUND_ASSAY,
    CAPE_COMPOUND_PETROLEUMFRACTION
    } CapeCompoundType;
    */
    /// <summary>
    /// The type of compound for use in petroleum fractions
    /// </summary>
    [Serializable]
    [System.Runtime.InteropServices.GuidAttribute("8091E285-3CFA-4a41-A5C4-141D0D709D87")]
    public enum CapeCompoundType
    {
        /// <summary>
        /// 
        /// </summary>
        CAPE_COMPOUND_REAL,
        /// <summary>
        /// 
        /// </summary>
        CAPE_COMPOUND_ION,
        /// <summary>
        /// 
        /// </summary>
        CAPE_COMPOUND_ASSAY,

    };
};
